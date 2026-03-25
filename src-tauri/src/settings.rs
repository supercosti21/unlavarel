use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub theme: String,               // "dark" or "light"
    pub default_php_version: String,  // e.g. "8.3"
    pub project_root: String,         // default directory for new projects
    pub auto_start_services: bool,    // start services on app launch
    pub start_minimized: bool,        // start app minimized to tray
    pub editor_command: String,       // e.g. "code", "phpstorm", "subl"
    pub browser_command: String,      // e.g. "open", "xdg-open"
    #[serde(default = "default_tld")]
    pub tld: String,                  // e.g. "test", "local", "localhost"
    #[serde(default = "default_web_server")]
    pub web_server: String,           // "nginx" or "apache"
    #[serde(default)]
    pub auto_create_vhost: bool,      // auto-create vhost when folder appears in project_root
    #[serde(default = "default_http_port")]
    pub http_port: u16,              // default 80
    #[serde(default = "default_https_port")]
    pub https_port: u16,             // default 443
    #[serde(default = "default_db_port")]
    pub db_port: u16,                // default 3306
}

fn default_tld() -> String { "test".into() }
fn default_web_server() -> String { "nginx".into() }
fn default_http_port() -> u16 { 80 }
fn default_https_port() -> u16 { 443 }
fn default_db_port() -> u16 { 3306 }

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
            start_minimized: false,
            editor_command,
            browser_command,
            tld: default_tld(),
            web_server: default_web_server(),
            auto_create_vhost: false,
            http_port: default_http_port(),
            https_port: default_https_port(),
            db_port: default_db_port(),
        }
    }
}

fn settings_file() -> PathBuf {
    let dir = dirs_next::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("unlavarel");
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

/// Synchronous settings read (for use in setup, before async context)
pub fn get_settings_sync() -> AppSettings {
    load_from_disk()
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
        .env("PATH", crate::setup::build_enriched_path())
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

/// Open the system terminal with enriched PATH (Homebrew, etc.)
#[tauri::command]
pub async fn open_terminal(path: Option<String>) -> Result<(), String> {
    let enriched_path = crate::setup::build_enriched_path();
    let dir = path.unwrap_or_else(|| {
        let s = load_from_disk();
        s.project_root.clone()
    });

    if cfg!(target_os = "macos") {
        // Open Terminal.app with the correct directory and PATH
        let script = format!(
            r#"tell application "Terminal"
                activate
                do script "cd '{}' && export PATH='{}'"
            end tell"#,
            dir, enriched_path
        );
        tokio::process::Command::new("osascript")
            .args(["-e", &script])
            .spawn()
            .map_err(|e| format!("Failed to open terminal: {}", e))?;
    } else if cfg!(target_os = "linux") {
        // Try common Linux terminals
        for term in &["gnome-terminal", "konsole", "xfce4-terminal", "xterm"] {
            let result = tokio::process::Command::new(term)
                .arg("--working-directory")
                .arg(&dir)
                .env("PATH", &enriched_path)
                .spawn();
            if result.is_ok() { return Ok(()); }
        }
        return Err("No terminal emulator found".into());
    } else if cfg!(target_os = "windows") {
        tokio::process::Command::new("cmd")
            .args(["/k", &format!("cd /d {}", dir)])
            .env("PATH", &enriched_path)
            .spawn()
            .map_err(|e| format!("Failed to open terminal: {}", e))?;
    }

    Ok(())
}

/// Read a config file and return its content
#[tauri::command]
pub async fn read_config_file(path: String) -> Result<String, String> {
    tokio::fs::read_to_string(&path)
        .await
        .map_err(|e| format!("Failed to read {}: {}", path, e))
}

/// Write content to a config file (may need elevation on Linux)
#[tauri::command]
pub async fn write_config_file(path: String, content: String) -> Result<(), String> {
    // Try direct write first
    match tokio::fs::write(&path, &content).await {
        Ok(_) => Ok(()),
        Err(_) => {
            // Need elevation (Linux: /etc/nginx/*, /etc/php/*)
            let output = crate::elevated::run_script_elevated(
                &format!("cat > '{}' << 'CONFIGEOF'\n{}\nCONFIGEOF\n", path, content)
            ).await.map_err(|e| format!("Failed to write config: {}", e))?;

            if output.status.success() {
                Ok(())
            } else {
                Err(format!("Failed to write config: {}", String::from_utf8_lossy(&output.stderr)))
            }
        }
    }
}

/// List editable config files for the current platform
#[tauri::command]
pub async fn list_config_files() -> Result<Vec<ConfigFile>, String> {
    let mut files = Vec::new();
    let enriched_path = crate::setup::build_enriched_path();

    // PHP config files
    let php_ini_paths = find_php_ini_files().await;
    for path in php_ini_paths {
        files.push(ConfigFile {
            name: format!("php.ini ({})", path.file_name().and_then(|n| n.to_str()).unwrap_or("php")),
            path: path.to_string_lossy().to_string(),
            category: "PHP".into(),
        });
    }

    // Nginx config
    if cfg!(target_os = "macos") {
        let prefix = if cfg!(target_arch = "aarch64") { "/opt/homebrew" } else { "/usr/local" };
        let nginx_conf = format!("{}/etc/nginx/nginx.conf", prefix);
        if PathBuf::from(&nginx_conf).exists() {
            files.push(ConfigFile { name: "nginx.conf".into(), path: nginx_conf, category: "Nginx".into() });
        }
    } else if cfg!(target_os = "linux") {
        if PathBuf::from("/etc/nginx/nginx.conf").exists() {
            files.push(ConfigFile { name: "nginx.conf".into(), path: "/etc/nginx/nginx.conf".into(), category: "Nginx".into() });
        }
    }

    // MySQL/MariaDB config
    for path in &["/etc/my.cnf", "/etc/mysql/my.cnf", "/opt/homebrew/etc/my.cnf", "/usr/local/etc/my.cnf"] {
        if PathBuf::from(path).exists() {
            files.push(ConfigFile { name: "my.cnf".into(), path: path.to_string(), category: "MySQL".into() });
            break;
        }
    }

    // dnsmasq config
    let dns_conf = crate::dns::dnsmasq_config_path();
    if dns_conf.exists() {
        files.push(ConfigFile { name: "dnsmasq.conf".into(), path: dns_conf.to_string_lossy().into(), category: "DNS".into() });
    }

    Ok(files)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigFile {
    pub name: String,
    pub path: String,
    pub category: String,
}

/// Find all php.ini files on the system
async fn find_php_ini_files() -> Vec<PathBuf> {
    let mut found = Vec::new();

    if cfg!(target_os = "macos") {
        let prefix = if cfg!(target_arch = "aarch64") { "/opt/homebrew" } else { "/usr/local" };
        for ver in &["8.5", "8.4", "8.3", "8.2", "8.1", "8.0"] {
            let path = PathBuf::from(prefix).join(format!("etc/php/{}/php.ini", ver));
            if path.exists() { found.push(path); }
        }
        // Generic
        let generic = PathBuf::from(prefix).join("etc/php.ini");
        if generic.exists() { found.push(generic); }
    } else if cfg!(target_os = "linux") {
        // Debian/Ubuntu versioned
        for ver in &["8.5", "8.4", "8.3", "8.2", "8.1", "8.0"] {
            let cli = PathBuf::from(format!("/etc/php/{}/cli/php.ini", ver));
            let fpm = PathBuf::from(format!("/etc/php/{}/fpm/php.ini", ver));
            if cli.exists() { found.push(cli); }
            if fpm.exists() { found.push(fpm); }
        }
        // Arch
        let arch = PathBuf::from("/etc/php/php.ini");
        if arch.exists() { found.push(arch); }
    }

    found
}
