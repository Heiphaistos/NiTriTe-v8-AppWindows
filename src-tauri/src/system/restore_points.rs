use serde::Serialize;
use std::process::Command;
#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

use crate::error::NiTriTeError;

#[derive(Debug, Clone, Serialize)]
pub struct RestorePoint {
    pub sequence_number: u32,
    pub description: String,
    pub creation_time: String,
    pub restore_type: String,
}

pub fn list_restore_points() -> Result<Vec<RestorePoint>, NiTriTeError> {
    let ps = r#"
try {
    $rps = Get-ComputerRestorePoint -ErrorAction Stop | Select-Object SequenceNumber, Description, CreationTime, @{Name='RestorePointType';Expression={$_.RestorePointType.ToString()}}
    $rps | ConvertTo-Json -Compress
} catch {
    "[]"
}
"#;
    let output = Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", ps])
        .creation_flags(0x08000000)
        .output()
        .map_err(|e| NiTriTeError::System(e.to_string()))?;

    let raw = String::from_utf8_lossy(&output.stdout);
    let trimmed = raw.trim();

    if trimmed.is_empty() || trimmed == "[]" {
        return Ok(vec![]);
    }

    // PowerShell peut retourner un objet unique au lieu d'un tableau
    let json_str = if trimmed.starts_with('{') {
        format!("[{}]", trimmed)
    } else {
        trimmed.to_string()
    };

    let parsed: Vec<serde_json::Value> = serde_json::from_str(&json_str)
        .unwrap_or_default();

    let points: Vec<RestorePoint> = parsed
        .iter()
        .filter_map(|v| {
            let seq = v["SequenceNumber"].as_u64()? as u32;
            let desc = v["Description"].as_str().unwrap_or("").to_string();
            let time = v["CreationTime"].as_str().unwrap_or("").to_string();
            let rtype = v["RestorePointType"].as_str().unwrap_or("APPLICATION_INSTALL").to_string();
            Some(RestorePoint {
                sequence_number: seq,
                description: desc,
                creation_time: time,
                restore_type: rtype,
            })
        })
        .collect();

    Ok(points)
}

pub fn create_restore_point(description: &str) -> Result<(), NiTriTeError> {
    // Nécessite les droits admin
    let ps = format!(
        r#"
$desc = '{}'
try {{
    # Contourne la limite de fréquence Windows (1 point / 24 h par défaut) : sans
    # ça, Checkpoint-Computer ne fait RIEN et sort quand même en 0 → faux succès
    # (« point créé » sans point réel). Mettre la fréquence à 0 force la création.
    try {{
        New-ItemProperty -Path 'HKLM:\SOFTWARE\Microsoft\Windows NT\CurrentVersion\SystemRestore' `
            -Name 'SystemRestorePointCreationFrequency' -Value 0 -PropertyType DWord -Force -EA SilentlyContinue | Out-Null
    }} catch {{}}
    Checkpoint-Computer -Description $desc -RestorePointType "APPLICATION_INSTALL" -ErrorAction Stop
    Write-Output "OK"
}} catch {{
    Write-Error $_.Exception.Message
    exit 1
}}
"#,
        description.replace('\'', "''")
    );

    let output = Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", &ps])
        .creation_flags(0x08000000)
        .output()
        .map_err(|e| NiTriTeError::System(e.to_string()))?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(NiTriTeError::System(format!(
            "Échec création point de restauration : {}",
            err.trim()
        )));
    }

    Ok(())
}

#[tauri::command]
pub async fn list_restore_points_cmd() -> Result<Vec<RestorePoint>, NiTriTeError> {
    tokio::task::spawn_blocking(list_restore_points)
        .await
        .map_err(|e| NiTriTeError::System(e.to_string()))?
}

#[tauri::command]
pub async fn create_restore_point_cmd(description: String) -> Result<(), NiTriTeError> {
    tokio::task::spawn_blocking(move || create_restore_point(&description))
        .await
        .map_err(|e| NiTriTeError::System(e.to_string()))?
}

#[tauri::command]
pub async fn delete_restore_point_cmd(sequence_number: u32) -> Result<(), NiTriTeError> {
    tokio::task::spawn_blocking(move || {
        // Find the restore point's creation time, then locate its VSS shadow copy
        // via Win32_ShadowCopy matching by date (within 60s tolerance), and delete it.
        let ps = format!(
            r#"
$seq = {seq}
$rp = Get-ComputerRestorePoint | Where-Object {{ $_.SequenceNumber -eq $seq }} | Select-Object -First 1
if (-not $rp) {{ throw "Point de restauration #{seq} introuvable." }}
$rpDate = $rp.ConvertToDateTime($rp.CreationTime)
$shadow = Get-WmiObject Win32_ShadowCopy | Where-Object {{
    try {{
        $sd = $_.ConvertToDateTime($_.InstallDate)
        [Math]::Abs(($sd - $rpDate).TotalSeconds) -lt 60
    }} catch {{ $false }}
}} | Select-Object -First 1
if ($shadow) {{
    $shadow.Delete() | Out-Null
    Write-Output "OK"
}} else {{
    throw "Shadow copy correspondant au point #{seq} introuvable — essayez via l'outil Restauration du système Windows."
}}
"#,
            seq = sequence_number
        );
        let output = Command::new("powershell")
            .args(["-NoProfile", "-NonInteractive", "-Command", &ps])
            .creation_flags(0x08000000)
            .output()
            .map_err(|e| NiTriTeError::System(e.to_string()))?;
        if output.status.success() {
            Ok(())
        } else {
            let err = String::from_utf8_lossy(&output.stderr);
            Err(NiTriTeError::System(format!(
                "Suppression échouée : {}",
                err.trim()
            )))
        }
    })
    .await
    .map_err(|e| NiTriTeError::System(e.to_string()))?
}

pub fn rollback_last_windows_update() -> Result<String, NiTriTeError> {
    let ps = r#"
try {
    $kb = Get-HotFix | Sort-Object InstalledOn -Descending | Select-Object -First 1 -ErrorAction Stop
    if (-not $kb) { throw "Aucune mise a jour trouvee" }
    $kbId = $kb.HotFixID -replace 'KB', ''
    $proc = Start-Process -FilePath "wusa.exe" -ArgumentList "/uninstall", "/kb:$kbId", "/quiet", "/norestart" -Wait -PassThru -ErrorAction Stop
    if ($proc.ExitCode -eq 0 -or $proc.ExitCode -eq 3010) {
        Write-Output "OK:$($kb.HotFixID)"
    } else {
        throw "wusa.exe code de sortie: $($proc.ExitCode)"
    }
} catch {
    Write-Error $_.Exception.Message
    exit 1
}
"#;

    let output = Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", ps])
        .creation_flags(0x08000000)
        .output()
        .map_err(|e| NiTriTeError::System(e.to_string()))?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(NiTriTeError::System(format!(
            "Rollback échoué : {}",
            err.trim()
        )));
    }

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let kb_id = stdout.strip_prefix("OK:").unwrap_or(&stdout).to_string();
    Ok(kb_id)
}

#[tauri::command]
pub async fn rollback_last_windows_update_cmd() -> Result<String, NiTriTeError> {
    tokio::task::spawn_blocking(rollback_last_windows_update)
        .await
        .map_err(|e| NiTriTeError::System(e.to_string()))?
}
