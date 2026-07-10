/// data_recovery/mod.rs — Re-exports de tous les sous-modules
#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

pub mod shadow_vss;
pub mod disk_recovery;
pub mod backup_folders;

pub use shadow_vss::*;
pub use disk_recovery::*;
pub use backup_folders::*;

// ─── Utilitaire PowerShell partagé ────────────────────────────────────────────

pub(super) fn run_ps(script: &str) -> Option<String> {
    #[cfg(target_os = "windows")]
    {
        let o = std::process::Command::new("powershell")
            .args(["-NoProfile", "-NonInteractive", "-Command", script])
            .creation_flags(0x08000000)
            .output().ok()?;
        Some(String::from_utf8_lossy(&o.stdout).to_string())
    }
    #[cfg(not(target_os = "windows"))]
    None
}

/// Comme `run_ps` mais passe des arguments supplémentaires accessibles via `$args[0]`, `$args[1]`, …
/// Les valeurs sont transmises comme arguments de processus séparés — aucune interpolation PS possible.
pub(super) fn run_ps_with_args(script: &str, extra_args: &[&str]) -> Option<String> {
    #[cfg(target_os = "windows")]
    {
        let mut cmd = std::process::Command::new("powershell");
        cmd.args(["-NoProfile", "-NonInteractive", "-Command", script]);
        for arg in extra_args {
            cmd.arg(arg);
        }
        let o = cmd.creation_flags(0x08000000).output().ok()?;
        Some(String::from_utf8_lossy(&o.stdout).to_string())
    }
    #[cfg(not(target_os = "windows"))]
    None
}
