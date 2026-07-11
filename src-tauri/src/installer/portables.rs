use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortableApp {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub size: String,
    pub url: String,
    pub exe_name: String,
    pub folder_name: String,
}

/// Dossiers exclus du scan (installateurs, dossiers vides ou non-portables)
const EXCLUDED_DIRS: &[&str] = &[
    "Executable",
    "Installateurs version portable",
    "Custom",
    "PortableApps",
    "Standalone Tools",
];

/// Scan dynamique du dossier logiciel/ adjacent à l'exe
pub fn get_all_portables() -> Vec<PortableApp> {
    let dir = crate::utils::paths::portables_dir();
    if !dir.exists() {
        tracing::warn!("Dossier portables introuvable: {:?}", dir);
        return vec![];
    }

    let Ok(entries) = std::fs::read_dir(&dir) else {
        return vec![];
    };

    let mut result = Vec::new();

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let folder_name = entry.file_name().to_string_lossy().to_string();

        // Exclure les dossiers non-portables
        if EXCLUDED_DIRS.iter().any(|e| {
            e.to_lowercase() == folder_name.to_lowercase()
        }) {
            continue;
        }

        // Trouver l'exe principal dans le dossier
        let Some(exe_name) = find_main_exe(&path, &folder_name) else {
            continue;
        };

        let id = to_id(&folder_name);
        let name = to_display_name(&folder_name);

        result.push(PortableApp {
            id,
            name,
            description: String::new(),
            category: "Portable".to_string(),
            size: String::new(),
            url: String::new(),
            exe_name,
            folder_name: folder_name.clone(),
        });
    }

    result.sort_by(|a, b| a.name.cmp(&b.name));
    result
}

/// Cherche le .exe principal dans un dossier portable
fn find_main_exe(dir: &std::path::Path, folder_name: &str) -> Option<String> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return None;
    };

    // Collecter tous les .exe à la racine (pas dans les sous-dossiers)
    let mut exes: Vec<String> = entries
        .flatten()
        .filter(|e| {
            let p = e.path();
            p.is_file()
                && p.extension()
                    .map(|x| x.eq_ignore_ascii_case("exe"))
                    .unwrap_or(false)
        })
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect();

    if exes.is_empty() {
        return None;
    }

    // Éliminer les installateurs / désinstallateurs évidents
    exes.retain(|e| {
        let l = e.to_lowercase();
        !l.starts_with("uninst")
            && !l.starts_with("setup")
            && !l.starts_with("install")
            && !l.ends_with("setup.exe")
            && !l.contains("mbsetup")
    });

    if exes.is_empty() {
        return None;
    }

    if exes.len() == 1 {
        return Some(exes.remove(0));
    }

    let folder_lower = folder_name.to_lowercase();

    // 1. Préférer un exe dont le nom contient le nom du dossier (ou vice-versa)
    for exe in &exes {
        let base = exe.to_lowercase().replace(".exe", "");
        if base == folder_lower
            || folder_lower.contains(&base)
            || base.contains(&folder_lower.replace(" ", "").replace("-", ""))
        {
            return Some(exe.clone());
        }
    }

    // 2. Préférer la version 64-bit
    for exe in &exes {
        let l = exe.to_lowercase();
        if (l.contains("64") || l.contains("x64")) && !l.contains("32") {
            return Some(exe.clone());
        }
    }

    // 3. Dernier recours : premier exe de la liste
    Some(exes.remove(0))
}

/// Convertit un nom de dossier en identifiant URL-safe
fn to_id(name: &str) -> String {
    let mut id: String = name
        .chars()
        .map(|c| {
            // is_ascii_alphanumeric (pas is_alphanumeric) : les lettres accentuées
            // (é, à…) ne sont pas URL-safe et to_ascii_lowercase les laisse intactes.
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect();

    // Dédupliquer les tirets consécutifs
    while id.contains("--") {
        id = id.replace("--", "-");
    }
    id.trim_matches('-').to_string()
}

/// Nettoie le nom du dossier pour l'affichage
fn to_display_name(folder_name: &str) -> String {
    // Supprimer les suffixes de version courants
    let patterns = [
        "_net6.0-windows10.0.18362.0",
        "_net6.0-windows",
        "Portable",
        "portable",
    ];

    let mut name = folder_name.to_string();
    for pat in &patterns {
        name = name.replace(pat, " ");
    }

    // Remplacer underscores par espaces, supprimer numéros de version isolés
    name = name.replace('_', " ");

    // Supprimer les blocs purement numériques ou type version (ex: "5.9.0")
    let words: Vec<&str> = name
        .split_whitespace()
        .filter(|w| {
            !w.chars().all(|c| c.is_ascii_digit() || c == '.')
        })
        .collect();

    let clean = words.join(" ").trim().to_string();
    if clean.is_empty() {
        folder_name.to_string()
    } else {
        clean
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_id_alphanumeric_lowercased() {
        assert_eq!(to_id("VLC"), "vlc");
        assert_eq!(to_id("Firefox123"), "firefox123");
    }

    #[test]
    fn to_id_spaces_become_hyphens() {
        assert_eq!(to_id("VLC Media Player"), "vlc-media-player");
    }

    #[test]
    fn to_id_consecutive_hyphens_deduped() {
        assert_eq!(to_id("App  Name"), "app-name");
        assert_eq!(to_id("App---Name"), "app-name");
    }

    #[test]
    fn to_id_leading_trailing_hyphens_trimmed() {
        assert_eq!(to_id("_App_"), "app");
        assert_eq!(to_id("-name-"), "name");
    }

    #[test]
    fn to_id_empty_stays_empty() {
        assert_eq!(to_id(""), "");
    }

    #[test]
    fn to_display_name_removes_portable_suffix() {
        let result = to_display_name("VLCPortable");
        assert!(!result.contains("Portable"), "Got: {}", result);
    }

    #[test]
    fn to_display_name_removes_net_suffix() {
        let result = to_display_name("MyApp_net6.0-windows");
        assert!(!result.contains("net6.0"), "Got: {}", result);
    }

    #[test]
    fn to_display_name_removes_version_numbers() {
        // Pure version tokens like "5.9.0" should be removed
        let result = to_display_name("Firefox_5.9.0");
        assert!(!result.contains("5.9.0"), "Got: {}", result);
    }

    #[test]
    fn to_display_name_underscores_become_spaces() {
        let result = to_display_name("My_App_Name");
        assert!(result.contains("My") && !result.contains('_'), "Got: {}", result);
    }

    #[test]
    fn to_display_name_empty_falls_back_to_original() {
        // If all tokens are filtered out, return original name
        let result = to_display_name("1.2.3");
        assert_eq!(result, "1.2.3");
    }
}
