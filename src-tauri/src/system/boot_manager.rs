use serde::Serialize;
use std::process::Command;
#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;
#[cfg(target_os = "windows")]
use crate::maintenance::commands::decode_output;

#[derive(Debug, Clone, Serialize, Default)]
pub struct BcdEntry {
    pub id: String,
    pub description: String,
    pub entry_type: String,
    pub device: String,
    pub path: String,
    pub locale: String,
    pub is_default: bool,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct BootConfig {
    pub entries: Vec<BcdEntry>,
    pub default_id: String,
    pub timeout_secs: u32,
    pub safe_mode: bool,
    pub debug_mode: bool,
}

fn ps_out(ps: &str) -> String {
    #[cfg(target_os = "windows")]
    {
        let o = Command::new("powershell").args(["-NoProfile","-NonInteractive","-Command",ps]).creation_flags(0x08000000).output();
        if let Ok(o) = o { return decode_output(&o.stdout).to_string(); }
    }
    #[cfg(not(target_os = "windows"))]
    let _ = ps;
    String::new()
}

#[tauri::command]
pub fn get_boot_config() -> BootConfig {
    // Libellés bcdedit partiellement localisés : sur Windows FR « identifier »
    // devient « identificateur » (vérifié) tandis que device/path/description/
    // locale/default/timeout restent en anglais. L'ancien `^identifier` ne
    // matchait jamais en FR → tous les IDs vides, is_default toujours faux et
    // « définir par défaut » inopérant. `^identif\S*` couvre EN + FR.
    let ps = r#"
try {
    $bcd = & bcdedit /enum ALL 2>&1
    $entries = @()
    $current = $null
    $defaultId = ''
    $timeout = 30
    $prev = ''
    $bcd | ForEach-Object {
        $line = "$_"
        if ($line -match '^-+$') {
            if ($current) { $entries += $current }
            # Le type d'entrée = ligne d'en-tête juste avant les tirets
            # (« Gestionnaire de démarrage Windows », « Windows Boot Loader »…).
            $current = @{ id=''; desc=''; type=$prev.Trim(); device=''; path=''; locale=''; default=$false }
        } elseif ($line -match '^identif\S*\s+(\S+)') {
            if ($current) { $current.id = $Matches[1] }
        } elseif ($line -match '^description\s+(.+)') {
            if ($current) { $current.desc = $Matches[1].Trim() }
        } elseif ($line -match '^device\s+(.+)') {
            if ($current) { $current.device = $Matches[1].Trim() }
        } elseif ($line -match '^path\s+(.+)') {
            if ($current) { $current.path = $Matches[1].Trim() }
        } elseif ($line -match '^locale\s+(.+)') {
            if ($current) { $current.locale = $Matches[1].Trim() }
        } elseif ($line -match '^timeout\s+(\d+)') {
            # Timeout du bloc {bootmgr} uniquement : /enum ALL liste {fwbootmgr}
            # en premier (timeout firmware, souvent 0) — le « premier timeout
            # rencontré » renvoyait cette valeur au lieu du délai du menu Windows.
            if ($current -and $current.id -eq '{bootmgr}') { $timeout = [int]$Matches[1] }
        }
        $prev = $line
    }
    if ($current) { $entries += $current }

    # Get default
    $defLine = $bcd | Where-Object { $_ -match 'default\s+(\{[^\}]+\})' } | Select-Object -First 1
    if ($defLine -match '\{[^\}]+\}') { $defaultId = $Matches[0] }

    @{ entries=$entries; default=$defaultId; timeout=$timeout; safe=$false; debug=$false } | ConvertTo-Json -Depth 4 -Compress
} catch {
    @{ entries=@(); default=''; timeout=30; safe=$false; debug=$false } | ConvertTo-Json -Compress
}
"#;
    #[cfg(target_os = "windows")]
    {
        let o = Command::new("powershell").args(["-NoProfile","-NonInteractive","-Command",ps]).creation_flags(0x08000000).output();
        if let Ok(o) = o {
            // decode_output : descriptions/en-têtes accentués (« Gestionnaire de
            // démarrage Windows ») sortent en OEM → mojibake avec from_utf8_lossy.
            let t = decode_output(&o.stdout);
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(t.trim()) {
                let entries = v["entries"].as_array().map(|arr| arr.iter().map(|e| {
                    let id = e["id"].as_str().unwrap_or("").to_string();
                    let default_id = v["default"].as_str().unwrap_or("").to_string();
                    let entry_type = match e["type"].as_str().map(str::trim) {
                        Some(t) if !t.is_empty() => t.to_string(),
                        _ => "osloader".to_string(),
                    };
                    BcdEntry {
                        is_default: id == default_id,
                        id,
                        description: e["desc"].as_str().unwrap_or("").to_string(),
                        entry_type,
                        device: e["device"].as_str().unwrap_or("").to_string(),
                        path: e["path"].as_str().unwrap_or("").to_string(),
                        locale: e["locale"].as_str().unwrap_or("").to_string(),
                    }
                }).collect()).unwrap_or_default();
                return BootConfig {
                    entries,
                    default_id: v["default"].as_str().unwrap_or("").to_string(),
                    timeout_secs: v["timeout"].as_u64().unwrap_or(30) as u32,
                    safe_mode: false,
                    debug_mode: false,
                };
            }
        }
    }
    BootConfig::default()
}

/// Exécute bcdedit directement et juge le résultat sur le CODE DE SORTIE
/// (0 = succès), pas sur le texte de sortie — bcdedit est localisé
/// ("successfully" en EN, "L'opération a réussi." en FR). Args array : pas d'injection.
#[cfg(target_os = "windows")]
fn run_bcdedit(args: &[&str]) -> Result<String, String> {
    let o = Command::new("bcdedit").args(args).creation_flags(0x08000000).output()
        .map_err(|e| e.to_string())?;
    let stdout = decode_output(&o.stdout).trim().to_string();
    if o.status.success() {
        Ok(stdout)
    } else {
        let stderr = decode_output(&o.stderr).trim().to_string();
        Err(if stderr.is_empty() {
            if stdout.is_empty() { "Échec bcdedit (droits admin requis ?)".into() } else { stdout }
        } else { stderr })
    }
}
#[cfg(not(target_os = "windows"))]
fn run_bcdedit(_args: &[&str]) -> Result<String, String> { Err("Windows uniquement".into()) }

#[tauri::command]
pub fn set_boot_timeout(seconds: u32) -> Result<String, String> {
    let s = seconds.min(999);
    run_bcdedit(&["/timeout", &s.to_string()])?;
    Ok(format!("Timeout défini à {} secondes", s))
}

#[tauri::command]
pub fn set_default_boot(entry_id: String) -> Result<String, String> {
    let id = entry_id.trim().trim_matches(|c| c == '{' || c == '}').to_string();
    if id.is_empty() || id.len() > 64 || !id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
        return Err(format!("Identifiant BCD invalide : '{}'", id));
    }
    run_bcdedit(&["/default", &format!("{{{}}}", id)])?;
    Ok(format!("Entrée de démarrage par défaut définie : {{{}}}", id))
}

#[tauri::command]
pub fn boot_to_recovery() -> String {
    ps_out("shutdown /r /o /f /t 0")
}
