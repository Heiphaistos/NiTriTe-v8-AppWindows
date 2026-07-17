use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct InstallRecord {
    pub app_id: String,
    pub app_name: String,
    pub installed_at: String,
    pub success: bool,
    pub method: String, // "winget" | "choco" | "url"
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FavoritesData {
    pub favorites: Vec<String>,       // app_ids
    pub history: Vec<InstallRecord>,  // dernières installations
}

fn data_path() -> PathBuf {
    // Portable d'abord (dossier config a cote de l'exe) — sans ca, favoris et
    // historique d'installation atterrissaient dans %LOCALAPPDATA%\NiTriTe,
    // une trace sur le PC client que la version portable est censee eviter.
    crate::utils::paths::config_dir().join("favorites.json")
}

fn load_data() -> FavoritesData {
    let path = data_path();
    if let Ok(raw) = fs::read_to_string(&path) {
        serde_json::from_str(&raw).unwrap_or_default()
    } else {
        FavoritesData::default()
    }
}

fn save_data(data: &FavoritesData) -> Result<(), String> {
    let path = data_path();
    let json = serde_json::to_string_pretty(data).map_err(|e| e.to_string())?;
    fs::write(&path, json).map_err(|e| e.to_string())
}

/// Retourne les favoris et l'historique
#[tauri::command]
pub fn get_favorites_data() -> FavoritesData {
    load_data()
}

/// Ajoute ou retire un app_id des favoris
#[tauri::command]
pub fn toggle_favorite(app_id: String) -> Result<bool, String> {
    let mut data = load_data();
    if let Some(pos) = data.favorites.iter().position(|f| f == &app_id) {
        data.favorites.remove(pos);
        save_data(&data)?;
        Ok(false)
    } else {
        data.favorites.push(app_id);
        save_data(&data)?;
        Ok(true)
    }
}

/// Enregistre une installation dans l'historique
#[tauri::command]
pub fn log_install(app_id: String, app_name: String, success: bool, method: String) -> Result<(), String> {
    let mut data = load_data();
    let now = chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string();
    data.history.insert(0, InstallRecord {
        app_id,
        app_name,
        installed_at: now,
        success,
        method,
    });
    // Garder 200 entrées max
    data.history.truncate(200);
    save_data(&data)
}

/// Efface l'historique d'installation
#[tauri::command]
pub fn clear_install_history() -> Result<(), String> {
    let mut data = load_data();
    data.history.clear();
    save_data(&data)
}
