use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub theme: String,               // "dark" or "light"
    pub default_php_version: String,  // e.g. "8.3"
    pub project_root: String,         // default directory for new projects
    pub auto_start_services: bool,    // start services on app launch
    pub editor_command: String,       // e.g. "code", "phpstorm", "subl"
    pub browser_command: String,      // e.g. "open", "xdg-open"
}

impl Default for AppSettings {
    fn default() -> Self {
        let project_root = if cfg!(target_os = "macos") {
            dirs_next::home_dir()
                .map(|h| h.join("Code").to_string_lossy().to_string())
                .unwrap_or_else(|| "~/Code".into())
        } else if cfg!(target_os = "windows") {
            dirs_next::home_dir()
                .map(|h| h.join("Projects").to_string_lossy().to_string())
                .unwrap_or_else(|| "C:\\Projects".into())
        } else {
            dirs_next::home_dir()
                .map(|h| h.join("projects").to_string_lossy().to_string())
                .unwrap_or_else(|| "~/projects".into())
        };

        let editor_command = "code".to_string();

        let browser_command = if cfg!(target_os = "macos") {
            "open".into()
        } else if cfg!(target_os = "windows") {
            "start".into()
        } else {
            "xdg-open".into()
        };

        Self {
            theme: "dark".into(),
            default_php_version: "8.3".into(),
            project_root,
            auto_start_services: false,
            editor_command,
            browser_command,
        }
    }
}

fn settings_file() -> PathBuf {
    let dir = dirs_next::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("macenv");
    std::fs::create_dir_all(&dir).ok();
    dir.join("settings.json")
}

fn load_from_disk() -> AppSettings {
    let path = settings_file();
    match std::fs::read_to_string(&path) {
        Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
        Err(_) => AppSettings::default(),
    }
}

fn save_to_disk(settings: &AppSettings) -> Result<(), String> {
    let path = settings_file();
    let json = serde_json::to_string_pretty(settings).map_err(|e| e.to_string())?;
    std::fs::write(&path, json).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn get_settings() -> Result<AppSettings, String> {
    Ok(load_from_disk())
}

#[tauri::command]
pub async fn save_settings(settings: AppSettings) -> Result<(), String> {
    save_to_disk(&settings)
}

#[tauri::command]
pub async fn open_in_editor(path: String) -> Result<(), String> {
    let settings = load_from_disk();
    tokio::process::Command::new(&settings.editor_command)
        .arg(&path)
        .spawn()
        .map_err(|e| format!("Failed to open editor: {}", e))?;
    Ok(())
}

#[tauri::command]
pub async fn open_in_browser(url: String) -> Result<(), String> {
    let settings = load_from_disk();
    tokio::process::Command::new(&settings.browser_command)
        .arg(&url)
        .spawn()
        .map_err(|e| format!("Failed to open browser: {}", e))?;
    Ok(())
}
