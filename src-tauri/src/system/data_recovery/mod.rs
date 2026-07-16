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
        // decode_output : noms de fichiers/dossiers récupérés (Documents,
        // Téléchargements…) sont couramment accentués FR — sans $OutputEncoding
        // préalable dans les scripts, from_utf8_lossy les mojibake.
        Some(crate::maintenance::commands::decode_output(&o.stdout))
    }
    #[cfg(not(target_os = "windows"))]
    None
}

/// Comme `run_ps` mais passe des arguments supplémentaires accessibles via `$args[0]`, `$args[1]`, …
/// Les valeurs sont transmises comme arguments de processus séparés — aucune interpolation PS possible.
/// Le script est enveloppé dans `& { ... }` : passé nu à `-Command`, il tourne dans la
/// portée top-level de powershell.exe où `$args` ne recoit JAMAIS les arguments de ligne
/// de commande suivants (silencieusement $null, jamais d'erreur) — seule une invocation
/// de scriptblock reelle (`& { }`) les reçoit. Confirmé en testant les deux formes cote a
/// cote sur une machine reelle : nu -> $args[0] est $null ; enveloppé -> $args[0] correct.
pub(super) fn run_ps_with_args(script: &str, extra_args: &[&str]) -> Option<String> {
    #[cfg(target_os = "windows")]
    {
        let wrapped = format!("& {{ {} }}", script);
        let mut cmd = std::process::Command::new("powershell");
        cmd.args(["-NoProfile", "-NonInteractive", "-Command", &wrapped]);
        for arg in extra_args {
            cmd.arg(arg);
        }
        let o = cmd.creation_flags(0x08000000).output().ok()?;
        Some(crate::maintenance::commands::decode_output(&o.stdout))
    }
    #[cfg(not(target_os = "windows"))]
    None
}
