/// extras/mod.rs — Re-exports de tous les sous-modules extras
#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

pub mod hash_dns_ports;
pub mod disk_files;
pub mod temps_wifi_turbo;
pub mod docker;
pub mod security_tools;

pub use hash_dns_ports::*;
pub use disk_files::*;
pub use temps_wifi_turbo::*;
pub use docker::*;
pub use security_tools::*;

// ─── Utilitaires partagés ──────────────────────────────────────────────────────

pub(super) fn parse_json_arr(s: &str) -> Vec<serde_json::Value> {
    let json = if s.starts_with('[') { s.to_string() } else { format!("[{}]", s) };
    serde_json::from_str(&json).unwrap_or_default()
}

pub(super) fn ps(script: &str) -> Result<String, String> {
    let out = std::process::Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", script])
        .creation_flags(0x08000000)
        .output()
        .map_err(|e| e.to_string())?;
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

/// Comme `ps()` mais passe des arguments supplémentaires accessibles via `$args[0]`, `$args[1]`, …
/// Les valeurs sont transmises comme arguments de processus séparés — aucune interpolation PS possible.
pub(super) fn ps_with_args(script: &str, extra_args: &[&str]) -> Result<String, String> {
    let mut cmd = std::process::Command::new("powershell");
    cmd.args(["-NoProfile", "-NonInteractive", "-Command", script]);
    for arg in extra_args {
        cmd.arg(arg);
    }
    let out = cmd.creation_flags(0x08000000).output().map_err(|e| e.to_string())?;
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}
