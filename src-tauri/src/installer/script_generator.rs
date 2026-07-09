use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct AppEntry {
    pub name: String,
    pub winget_id: Option<String>,
    pub choco_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct GeneratedScript {
    pub content: String,
    pub filename: String,
    pub app_count: usize,
}

/// Valide un identifiant de paquet (winget/choco) : alphanumeric + . - _
fn safe_pkg_id(id: &str) -> bool {
    !id.is_empty() && id.len() <= 200
        && id.chars().all(|c| c.is_alphanumeric() || c == '.' || c == '-' || c == '_')
}

/// Nettoie un nom d'app pour l'affichage dans echo (supprime chars dangereux batch)
fn safe_display_name(name: &str) -> String {
    name.chars()
        .filter(|c| c.is_alphanumeric() || " .-_()[]".contains(*c))
        .take(80)
        .collect()
}

/// Génère un script de déploiement (.bat ou .ps1) à partir d'une liste d'apps
#[tauri::command]
pub fn generate_deploy_script(apps: Vec<AppEntry>, format: String) -> GeneratedScript {
    let use_ps = format.to_lowercase() == "ps1";
    let app_count = apps.len();

    let content = if use_ps {
        generate_powershell(&apps)
    } else {
        generate_batch(&apps)
    };

    let filename = if use_ps {
        "deploy_nitrite.ps1".to_string()
    } else {
        "deploy_nitrite.bat".to_string()
    };

    GeneratedScript { content, filename, app_count }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn safe_pkg_id_accepts_valid_ids() {
        assert!(safe_pkg_id("Google.Chrome"));
        assert!(safe_pkg_id("VideoLAN.VLC"));
        assert!(safe_pkg_id("7zip.7zip"));
        assert!(safe_pkg_id("Microsoft.VisualStudioCode"));
        assert!(safe_pkg_id("choco-pkg_name-1.0"));
    }

    #[test]
    fn safe_pkg_id_rejects_empty() {
        assert!(!safe_pkg_id(""));
    }

    #[test]
    fn safe_pkg_id_rejects_injection_chars() {
        assert!(!safe_pkg_id("pkg; rm -rf /"));
        assert!(!safe_pkg_id("pkg && evil"));
        assert!(!safe_pkg_id("pkg|cmd"));
        assert!(!safe_pkg_id("pkg$(whoami)"));
        assert!(!safe_pkg_id("pkg`cmd`"));
        assert!(!safe_pkg_id("pkg\ninjection"));
    }

    #[test]
    fn safe_pkg_id_rejects_too_long() {
        let long = "a".repeat(201);
        assert!(!safe_pkg_id(&long));
    }

    #[test]
    fn safe_display_name_strips_dangerous_chars() {
        let result = safe_display_name("App & Company; evil");
        assert!(!result.contains('&'));
        assert!(!result.contains(';'));
        assert!(result.contains("App"));
    }

    #[test]
    fn safe_display_name_truncates_at_80() {
        let long = "A".repeat(100);
        let result = safe_display_name(&long);
        assert!(result.len() <= 80);
    }

    #[test]
    fn generate_deploy_script_batch_skips_injection() {
        let apps = vec![AppEntry {
            name: "Evil App".to_string(),
            winget_id: Some("evil;cmd".to_string()),
            choco_id: None,
        }];
        let s = generate_deploy_script(apps, "bat".to_string());
        assert!(!s.content.contains("evil;cmd"));
    }

    #[test]
    fn generate_deploy_script_ps1_includes_valid_pkg() {
        let apps = vec![AppEntry {
            name: "VLC".to_string(),
            winget_id: Some("VideoLAN.VLC".to_string()),
            choco_id: None,
        }];
        let s = generate_deploy_script(apps, "ps1".to_string());
        assert_eq!(s.filename, "deploy_nitrite.ps1");
        assert_eq!(s.app_count, 1);
        assert!(s.content.contains("VideoLAN.VLC"));
    }

    #[test]
    fn generate_deploy_script_batch_includes_valid_pkg() {
        let apps = vec![AppEntry {
            name: "7-Zip".to_string(),
            winget_id: None,
            choco_id: Some("7zip".to_string()),
        }];
        let s = generate_deploy_script(apps, "bat".to_string());
        assert_eq!(s.filename, "deploy_nitrite.bat");
        assert!(s.content.contains("7zip"));
    }
}

fn generate_batch(apps: &[AppEntry]) -> String {
    let mut lines = vec![
        "@echo off".to_string(),
        ":: Script de déploiement généré par NiTriTe".to_string(),
        ":: Nécessite WinGet installé".to_string(),
        "".to_string(),
        "echo === NiTriTe — Installation des logiciels ===".to_string(),
        "echo.".to_string(),
    ];

    for app in apps {
        let display = safe_display_name(&app.name);
        if let Some(wid) = &app.winget_id {
            if !safe_pkg_id(wid) { continue; }
            lines.push(format!("echo [WinGet] Installation de {}...", display));
            lines.push(format!(
                "winget install --id {} --silent --accept-package-agreements --accept-source-agreements",
                wid
            ));
        } else if let Some(cid) = &app.choco_id {
            if !safe_pkg_id(cid) { continue; }
            lines.push(format!("echo [Choco] Installation de {}...", display));
            lines.push(format!("choco install {} -y", cid));
        }
        lines.push("".to_string());
    }

    lines.push("echo === Terminé ===".to_string());
    lines.push("pause".to_string());
    lines.join("\r\n")
}

fn generate_powershell(apps: &[AppEntry]) -> String {
    let mut lines = vec![
        "# Script de déploiement généré par NiTriTe".to_string(),
        "# Nécessite WinGet (winget) installé".to_string(),
        "".to_string(),
        "Write-Host '=== NiTriTe — Installation des logiciels ===' -ForegroundColor Cyan".to_string(),
        "".to_string(),
    ];

    for app in apps {
        let display = safe_display_name(&app.name);
        if let Some(wid) = &app.winget_id {
            if !safe_pkg_id(wid) { continue; }
            lines.push(format!("Write-Host '[WinGet] {}...' -ForegroundColor Yellow", display));
            lines.push(format!(
                "winget install --id {} --silent --accept-package-agreements --accept-source-agreements",
                wid
            ));
        } else if let Some(cid) = &app.choco_id {
            if !safe_pkg_id(cid) { continue; }
            lines.push(format!("Write-Host '[Choco] {}...' -ForegroundColor Yellow", display));
            lines.push(format!("choco install {} -y", cid));
        }
        lines.push("".to_string());
    }

    lines.push("Write-Host '=== Terminé ===' -ForegroundColor Green".to_string());
    lines.join("\n")
}
