use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use crate::utils::paths;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    pub name: String,
    pub description: String,
    pub created_at: String,
    pub version: String,
    pub config: serde_json::Value,
}

fn profiles_dir() -> PathBuf {
    let dir = paths::config_dir().join("profiles");
    let _ = std::fs::create_dir_all(&dir);
    dir
}

fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
        .collect()
}

pub fn list_profiles() -> Vec<Profile> {
    let dir = profiles_dir();
    let mut profiles = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if let Ok(profile) = serde_json::from_str::<Profile>(&content) {
                        profiles.push(profile);
                    }
                }
            }
        }
    }
    profiles.sort_by(|a, b| a.name.cmp(&b.name));
    profiles
}

pub fn save_profile(profile: &Profile) -> Result<(), std::io::Error> {
    let dir = profiles_dir();
    let filename = sanitize_filename(&profile.name);
    let path = dir.join(format!("{}.json", filename));
    let json = serde_json::to_string_pretty(profile)
        .map_err(std::io::Error::other)?;
    std::fs::write(path, json)
}

pub fn delete_profile(name: &str) -> Result<(), std::io::Error> {
    let dir = profiles_dir();
    let filename = sanitize_filename(name);
    let path = dir.join(format!("{}.json", filename));
    if path.exists() {
        std::fs::remove_file(path)
    } else {
        Ok(())
    }
}

pub fn profile_exists(name: &str) -> bool {
    let dir = profiles_dir();
    let filename = sanitize_filename(name);
    dir.join(format!("{}.json", filename)).exists()
}

pub fn export_profile_json(name: &str) -> Option<String> {
    let dir = profiles_dir();
    let filename = sanitize_filename(name);
    let path = dir.join(format!("{}.json", filename));
    std::fs::read_to_string(path).ok()
}

pub fn import_profile_from_json(json: &str) -> Result<Profile, String> {
    serde_json::from_str::<Profile>(json).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_alphanumeric_unchanged() {
        assert_eq!(sanitize_filename("myProfile123"), "myProfile123");
        assert_eq!(sanitize_filename("default-profile"), "default-profile");
        assert_eq!(sanitize_filename("my_profile"), "my_profile");
    }

    #[test]
    fn sanitize_path_traversal_blocked() {
        // "../evil" → 3 replaced chars (./.) + "evil"
        assert_eq!(sanitize_filename("../evil"), "___evil");
        // "../../etc/passwd" → 6 replaced chars (../..) + etc + _ + passwd
        assert_eq!(sanitize_filename("../../etc/passwd"), "______etc_passwd");
    }

    #[test]
    fn sanitize_spaces_replaced() {
        assert_eq!(sanitize_filename("my profile name"), "my_profile_name");
    }

    #[test]
    fn sanitize_special_chars_replaced() {
        assert_eq!(sanitize_filename("profile<>|*"), "profile____");
        assert_eq!(sanitize_filename("name.json"), "name_json");
    }

    #[test]
    fn sanitize_empty_stays_empty() {
        assert_eq!(sanitize_filename(""), "");
    }

    #[test]
    fn import_valid_json_profile() {
        let json = r#"{"name":"Test","description":"desc","created_at":"2026-01-01","version":"1.0","config":{}}"#;
        let p = import_profile_from_json(json).unwrap();
        assert_eq!(p.name, "Test");
        assert_eq!(p.version, "1.0");
    }

    #[test]
    fn import_invalid_json_returns_error() {
        assert!(import_profile_from_json("{not valid json}").is_err());
    }
}
