use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub theme: String,               // "dark" or "light"
    pub default_php_version: String, // e.g. "8.3"
    pub project_root: String,        // default directory for new projects
    pub auto_start_services: bool,   // start services on app launch
    pub editor_command: String,      // e.g. "code", "phpstorm", "subl"
    pub browser_command: String,     // e.g. "open", "xdg-open"
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_settings() {
        let settings = AppSettings::default();
        assert_eq!(settings.theme, "dark");
        assert_eq!(settings.default_php_version, "8.3");
        assert!(!settings.auto_start_services);
        assert_eq!(settings.editor_command, "code");
    }

    #[test]
    fn test_default_browser_command_platform() {
        let settings = AppSettings::default();
        if cfg!(target_os = "macos") {
            assert_eq!(settings.browser_command, "open");
        } else if cfg!(target_os = "windows") {
            assert_eq!(settings.browser_command, "start");
        } else {
            assert_eq!(settings.browser_command, "xdg-open");
        }
    }

    #[test]
    fn test_settings_serialization_roundtrip() {
        let settings = AppSettings {
            theme: "light".into(),
            default_php_version: "8.4".into(),
            project_root: "/tmp/projects".into(),
            auto_start_services: true,
            editor_command: "vim".into(),
            browser_command: "firefox".into(),
        };

        let json = serde_json::to_string_pretty(&settings).unwrap();
        let parsed: AppSettings = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.theme, "light");
        assert_eq!(parsed.default_php_version, "8.4");
        assert_eq!(parsed.project_root, "/tmp/projects");
        assert!(parsed.auto_start_services);
        assert_eq!(parsed.editor_command, "vim");
        assert_eq!(parsed.browser_command, "firefox");
    }

    #[test]
    fn test_settings_file_persistence() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.json");

        let settings = AppSettings::default();
        let json = serde_json::to_string_pretty(&settings).unwrap();
        std::fs::write(&path, &json).unwrap();

        let content = std::fs::read_to_string(&path).unwrap();
        let loaded: AppSettings = serde_json::from_str(&content).unwrap();
        assert_eq!(loaded.theme, "dark");
    }
}
