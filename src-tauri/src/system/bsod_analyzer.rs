use serde::Serialize;
use std::process::Command;
#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

#[derive(Debug, Clone, Serialize, Default)]
pub struct BsodEntry {
    pub timestamp: String,
    pub bug_check_code: String,
    pub bug_check_hex: String,
    pub description: String,
    pub parameters: Vec<String>,
    pub module: String,
    pub dump_file: String,
    pub dump_size_kb: u64,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct BsodReport {
    pub entries: Vec<BsodEntry>,
    pub total_count: u32,
    pub last_bsod: String,
    pub dump_folder: String,
    pub dump_count: u32,
}

#[tauri::command]
pub async fn get_bsod_history() -> BsodReport {
    tokio::task::spawn_blocking(get_bsod_report).await.unwrap_or_default()
}

pub fn get_bsod_report() -> BsodReport {
    let ps = r#"
$report = @{}
try {
    # Get BSOD events from System log (ID 1001 = BugCheck)
    $events = @(Get-WinEvent -FilterHashtable @{LogName='System';Id=1001;ProviderName='Microsoft-Windows-WER-SystemErrorReporting'} -MaxEvents 20 -EA SilentlyContinue |
        ForEach-Object {
            $msg = $_.Message
            $hex = if($msg -match '0x[0-9A-Fa-f]+'){'0x'+($msg | Select-String '0x([0-9A-Fa-f]+)').Matches[0].Groups[1].Value}else{''}
            @{
                ts   = $_.TimeCreated.ToString('yyyy-MM-dd HH:mm:ss')
                code = $hex
                desc = ($msg -split '\n')[0].Trim()
                mod  = if($msg -match 'caused by.*?(\S+\.sys)'){$Matches[1]}else{''}
                dump = ''
                kb   = 0
                params = @()
            }
        })

    # Supplement with WER reports
    $werDir = "$env:ProgramData\Microsoft\Windows\WER\ReportArchive"
    if (Test-Path $werDir) {
        $werEntries = @(Get-ChildItem $werDir -Directory -EA SilentlyContinue |
            Where-Object { $_.Name -like 'Kernel*' } |
            Sort-Object LastWriteTime -Descending |
            Select-Object -First 10 |
            ForEach-Object {
                $rptFile = Get-Item (Join-Path $_.FullName 'Report.wer') -EA SilentlyContinue
                $bcFile = Get-Item (Join-Path $_.FullName 'memory.dmp') -EA SilentlyContinue
                @{
                    ts   = $_.LastWriteTime.ToString('yyyy-MM-dd HH:mm:ss')
                    code = $_.Name
                    desc = "WER Kernel Crash Report"
                    mod  = ''
                    dump = if($bcFile){$bcFile.FullName}else{''}
                    kb   = if($bcFile){[int]($bcFile.Length/1024)}else{0}
                    params = @()
                }
            })
        $events += $werEntries
    }

    # Minidumps
    $dumpDir = 'C:\Windows\Minidump'
    $dumpFiles = @(Get-ChildItem $dumpDir -Filter '*.dmp' -EA SilentlyContinue | Sort-Object LastWriteTime -Descending)

    $report.entries = $events
    $report.total   = [int]$events.Count
    $report.last    = if($events.Count -gt 0){$events[0].ts}else{'Aucun'}
    $report.dumpDir = $dumpDir
    $report.dumpCt  = [int]$dumpFiles.Count
} catch {
    $report.entries = @()
    $report.total   = 0
    $report.last    = 'Erreur: '+$_.Exception.Message
    $report.dumpDir = 'C:\Windows\Minidump'
    $report.dumpCt  = 0
}
$report | ConvertTo-Json -Depth 4 -Compress
"#;

    #[cfg(target_os = "windows")]
    {
        let o = Command::new("powershell").args(["-NoProfile","-NonInteractive","-Command",ps]).creation_flags(0x08000000).output();
        if let Ok(o) = o {
            // decode_output : $_.Message (texte d'event log Windows) est pleinement
            // localisé sur Windows FR — from_utf8_lossy brut mojibake la description.
            let t = crate::maintenance::commands::decode_output(&o.stdout);
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(t.trim()) {
                let entries = v["entries"].as_array().map(|arr| arr.iter().map(|e| BsodEntry {
                    timestamp: e["ts"].as_str().unwrap_or("").to_string(),
                    bug_check_code: e["code"].as_str().unwrap_or("").to_string(),
                    bug_check_hex: e["code"].as_str().unwrap_or("").to_string(),
                    description: e["desc"].as_str().unwrap_or("").to_string(),
                    parameters: e["params"].as_array().map(|a| a.iter().filter_map(|s| s.as_str().map(|x| x.to_string())).collect()).unwrap_or_default(),
                    module: e["mod"].as_str().unwrap_or("").to_string(),
                    dump_file: e["dump"].as_str().unwrap_or("").to_string(),
                    dump_size_kb: e["kb"].as_u64().unwrap_or(0),
                }).collect()).unwrap_or_default();
                return BsodReport {
                    entries,
                    total_count: v["total"].as_u64().unwrap_or(0) as u32,
                    last_bsod: v["last"].as_str().unwrap_or("Aucun").to_string(),
                    dump_folder: v["dumpDir"].as_str().unwrap_or("C:\\Windows\\Minidump").to_string(),
                    dump_count: v["dumpCt"].as_u64().unwrap_or(0) as u32,
                };
            }
        }
    }
    BsodReport { last_bsod: "Aucun".to_string(), dump_folder: "C:\\Windows\\Minidump".to_string(), ..Default::default() }
}

#[tauri::command]
pub fn get_bugcheck_description(code: String) -> String {
    match code.to_uppercase().trim_start_matches("0X") {
        "3B" => "SYSTEM_SERVICE_EXCEPTION — Erreur dans un service système ou driver".to_string(),
        "50" => "PAGE_FAULT_IN_NONPAGED_AREA — Accès mémoire invalide (driver défectueux)".to_string(),
        "7E" => "SYSTEM_THREAD_EXCEPTION_NOT_HANDLED — Exception non gérée dans un thread système".to_string(),
        "7F" => "UNEXPECTED_KERNEL_MODE_TRAP — Trap CPU imprévue (surchauffe/RAM défectueuse)".to_string(),
        "9F" => "DRIVER_POWER_STATE_FAILURE — Driver ne gère pas correctement l'alimentation".to_string(),
        "A" | "0A" => "IRQL_NOT_LESS_OR_EQUAL — Driver accède à une mauvaise adresse mémoire".to_string(),
        "D1" => "DRIVER_IRQL_NOT_LESS_OR_EQUAL — Même que 0A mais spécifique aux drivers".to_string(),
        "1E" => "KMODE_EXCEPTION_NOT_HANDLED — Exception non gérée en mode kernel".to_string(),
        "C4" => "DRIVER_VERIFIER_DETECTED_VIOLATION — Driver Verifier a détecté un problème".to_string(),
        "C5" => "DRIVER_CORRUPTED_EXPOOL — Pool mémoire corrompu par un driver".to_string(),
        "E3" => "RESOURCE_NOT_OWNED — Thread libère une ressource qu'il ne possède pas".to_string(),
        "1A" => "MEMORY_MANAGEMENT — Problème grave de gestion mémoire (souvent RAM)".to_string(),
        "19" => "BAD_POOL_HEADER — Corruption du pool mémoire du kernel".to_string(),
        "F4" => "CRITICAL_OBJECT_TERMINATION — Processus critique terminé de façon inattendue".to_string(),
        "124" => "WHEA_UNCORRECTABLE_ERROR — Erreur matérielle non corrigeable (CPU/BIOS/RAM)".to_string(),
        "133" => "DPC_WATCHDOG_VIOLATION — Timeout DPC (disque lent ou driver bloqué)".to_string(),
        "139" => "KERNEL_SECURITY_CHECK_FAILURE — Corruption mémoire kernel détectée".to_string(),
        "154" => "UNEXPECTED_STORE_EXCEPTION — Erreur de stockage (SSD/HDD défaillant)".to_string(),
        "101" => "CLOCK_WATCHDOG_TIMEOUT — CPU secondaire ne répond plus (overclocking)".to_string(),
        _ => format!("Code {} — voir https://docs.microsoft.com/en-us/windows-hardware/drivers/debugger/bug-check-code-reference2", code),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_code_uppercase() {
        let d = get_bugcheck_description("0x50".to_string());
        assert!(d.contains("PAGE_FAULT_IN_NONPAGED_AREA"));
    }

    #[test]
    fn known_code_without_prefix() {
        let d = get_bugcheck_description("3B".to_string());
        assert!(d.contains("SYSTEM_SERVICE_EXCEPTION"));
    }

    #[test]
    fn known_code_lowercase_input() {
        let d = get_bugcheck_description("0xa".to_string());
        assert!(d.contains("IRQL_NOT_LESS_OR_EQUAL"));
    }

    #[test]
    fn code_0a_and_a_same_result() {
        let a = get_bugcheck_description("A".to_string());
        let b = get_bugcheck_description("0A".to_string());
        assert_eq!(a, b);
    }

    #[test]
    fn whea_code_124() {
        let d = get_bugcheck_description("124".to_string());
        assert!(d.contains("WHEA_UNCORRECTABLE_ERROR"));
    }

    #[test]
    fn unknown_code_links_to_docs() {
        let d = get_bugcheck_description("DEAD".to_string());
        assert!(d.contains("DEAD"));
        assert!(d.contains("docs.microsoft.com"));
    }

    #[test]
    fn memory_management_code() {
        let d = get_bugcheck_description("1A".to_string());
        assert!(d.contains("MEMORY_MANAGEMENT"));
    }
}
