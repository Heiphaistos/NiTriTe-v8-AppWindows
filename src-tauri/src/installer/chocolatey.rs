use serde::Serialize;
use std::process::Command;
#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

use crate::error::NiTriTeError;

#[derive(Debug, Clone, Serialize)]
pub struct ChocoPackage {
    pub name: String,
    pub current_version: String,
    pub available_version: String,
    pub pinned: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ChocoUpgradeResult {
    pub success: bool,
    pub upgraded_count: u32,
    pub message: String,
}

pub fn check_chocolatey() -> bool {
    Command::new("choco")
        .arg("--version")
        .creation_flags(0x08000000)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

pub fn list_chocolatey_upgrades() -> Result<Vec<ChocoPackage>, NiTriTeError> {
    let output = Command::new("choco")
        .args(["outdated", "-r", "--no-color"])
        .creation_flags(0x08000000)
        .output()
        .map_err(|e| NiTriTeError::System(format!("Chocolatey introuvable: {}", e)))?;

    let text = String::from_utf8_lossy(&output.stdout).to_string();
    let mut packages = Vec::new();

    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        // Format choco -r : "packagename|currentversion|availableversion|ispinned"
        let parts: Vec<&str> = line.splitn(4, '|').collect();
        if parts.len() >= 3 {
            let pinned = parts.get(3).map(|v| *v == "true").unwrap_or(false);
            packages.push(ChocoPackage {
                name: parts[0].to_string(),
                current_version: parts[1].to_string(),
                available_version: parts[2].to_string(),
                pinned,
            });
        }
    }

    Ok(packages)
}

pub fn upgrade_chocolatey_all(excluded: Vec<String>) -> Result<ChocoUpgradeResult, NiTriTeError> {
    let mut args: Vec<String> = vec!["upgrade".into(), "all".into(), "-y".into(), "--no-color".into()];
    // Exclusions : choco supporte `--except="pkg1,pkg2"`. Sans ça, les paquets
    // exclus par l'utilisateur étaient quand même mis à jour. Les noms non-choco
    // (ids winget de la liste d'exclusion partagée) sont simplement ignorés.
    let cleaned: Vec<String> = excluded.into_iter().map(|s| s.replace(['"', ','], "")).filter(|s| !s.trim().is_empty()).collect();
    if !cleaned.is_empty() {
        args.push(format!("--except=\"{}\"", cleaned.join(",")));
    }
    let output = Command::new("choco")
        .args(&args)
        .creation_flags(0x08000000)
        .output()
        .map_err(|e| NiTriTeError::System(format!("Erreur upgrade choco: {}", e)))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let success = output.status.success();

    // Nombre de paquets mis à jour : choco termine par une ligne récapitulative
    // "Chocolatey upgraded 2/3 packages." — on en extrait N (fiable), plutôt que
    // de compter les lignes "upgraded"/"successful" qui sur-comptaient (la ligne
    // récap + une ligne par paquet, et "0/3" comptait comme 1).
    let upgraded = stdout
        .lines()
        .find_map(|l| {
            let ll = l.to_lowercase();
            if !ll.contains('/') {
                return None;
            }
            ll.split_once("upgraded ")
                .and_then(|(_, rest)| rest.split('/').next())
                .and_then(|n| n.trim().parse::<u32>().ok())
        })
        .unwrap_or_else(|| {
            // Repli si le format récap change : compter les lignes "successful".
            stdout.lines().filter(|l| l.to_lowercase().contains("successful")).count() as u32
        });

    let message = if success {
        format!("{} paquet(s) mis à jour", upgraded)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        format!("Erreur: {}", stderr.lines().next().unwrap_or("inconnue"))
    };

    Ok(ChocoUpgradeResult {
        success,
        upgraded_count: upgraded,
        message,
    })
}
