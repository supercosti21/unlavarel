use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::process::Command;

/// A service that is actually installed on this system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledService {
    pub id: String,           // unique key: "php", "mariadb", "nginx"...
    pub display_name: String, // "PHP 8.5.4", "MariaDB 12.2"
    pub category: String,     // "language", "database", "webserver", "cache", "dns", "mail", "tool"
    pub binary: String,       // path to binary
    pub systemd_unit: Option<String>, // "php-fpm", "mariadb", "nginx"...
    pub brew_service: Option<String>, // for macOS
    pub version: String,
    pub has_service: bool,    // can be started/stopped
}

/// Scan the system and return only what's actually installed.
#[tauri::command]
pub async fn discover_services() -> Result<Vec<InstalledService>, String> {
    let mut found = Vec::new();

    // PHP
    if let Some(svc) = detect("php", "php", &["-v"], "language", parse_php_version).await {
        let mut svc = svc;
        svc.display_name = format!("PHP {}", svc.version);
        svc.systemd_unit = Some("php-fpm".into());
        svc.brew_service = Some("php".into());
        svc.has_service = true;
        found.push(svc);
    }

    // Database: check what's ACTUALLY installed — only one
    if let Some(svc) = detect("mariadb", "mariadb", &["--version"], "database", parse_mariadb_version).await {
        let mut svc = svc;
        svc.display_name = format!("MariaDB {}", svc.version);
        svc.systemd_unit = Some("mariadb".into());
        svc.brew_service = Some("mariadb".into());
        svc.has_service = true;
        found.push(svc);
    } else if let Some(svc) = detect("mysql", "mysql", &["--version"], "database", parse_mysql_version).await {
        // Only check mysql if mariadb is NOT found (mariadb provides mysql binary too)
        // Distinguish real MySQL from MariaDB's mysql binary
        let output = Command::new("mysql").arg("--version").output().await;
        if let Ok(o) = output {
            let text = String::from_utf8_lossy(&o.stdout);
            if !text.contains("MariaDB") {
                let mut svc = svc;
                svc.display_name = format!("MySQL {}", svc.version);
                svc.systemd_unit = Some("mysqld".into());
                svc.brew_service = Some("mysql".into());
                svc.has_service = true;
                found.push(svc);
            }
        }
    }

    // PostgreSQL
    if let Some(svc) = detect("postgresql", "psql", &["--version"], "database", parse_simple_version).await {
        let mut svc = svc;
        svc.display_name = format!("PostgreSQL {}", svc.version);
        svc.systemd_unit = Some("postgresql".into());
        svc.brew_service = Some("postgresql".into());
        svc.has_service = true;
        found.push(svc);
    }

    // Nginx
    if let Some(svc) = detect("nginx", "nginx", &["-v"], "webserver", parse_nginx_version).await {
        let mut svc = svc;
        svc.display_name = format!("Nginx {}", svc.version);
        svc.systemd_unit = Some("nginx".into());
        svc.brew_service = Some("nginx".into());
        svc.has_service = true;
        found.push(svc);
    }

    // Redis / Valkey (Arch uses valkey which provides redis-server)
    if let Some(svc) = detect("redis", "redis-server", &["--version"], "cache", parse_redis_version).await {
        let mut svc = svc;
        let is_valkey = svc.version.contains("Valkey") || {
            let o = Command::new("redis-server").arg("--version").output().await;
            o.map(|o| String::from_utf8_lossy(&o.stdout).contains("Valkey")).unwrap_or(false)
        };
        svc.display_name = if is_valkey {
            format!("Valkey (Redis) {}", svc.version)
        } else {
            format!("Redis {}", svc.version)
        };
        svc.systemd_unit = Some("redis".into());
        svc.brew_service = Some("redis".into());
        svc.has_service = true;
        found.push(svc);
    }

    // Memcached
    if let Some(svc) = detect("memcached", "memcached", &["-h"], "cache", parse_memcached_version).await {
        let mut svc = svc;
        svc.display_name = format!("Memcached {}", svc.version);
        svc.systemd_unit = Some("memcached".into());
        svc.brew_service = Some("memcached".into());
        svc.has_service = true;
        found.push(svc);
    }

    // dnsmasq
    if let Some(svc) = detect("dnsmasq", "dnsmasq", &["--version"], "dns", parse_simple_version).await {
        let mut svc = svc;
        svc.display_name = format!("dnsmasq {}", svc.version);
        svc.systemd_unit = Some("dnsmasq".into());
        svc.brew_service = Some("dnsmasq".into());
        svc.has_service = true;
        found.push(svc);
    }

    // Mailpit
    if let Some(svc) = detect("mailpit", "mailpit", &["version"], "mail", parse_simple_version).await {
        let mut svc = svc;
        svc.display_name = format!("Mailpit {}", svc.version);
        svc.systemd_unit = Some("mailpit".into());
        svc.brew_service = Some("mailpit".into());
        svc.has_service = true;
        found.push(svc);
    }

    // Composer (no service, just a tool)
    if let Some(svc) = detect("composer", "composer", &["--version"], "tool", parse_simple_version).await {
        let mut svc = svc;
        svc.display_name = format!("Composer {}", svc.version);
        svc.has_service = false;
        found.push(svc);
    }

    // Node.js (no service, just a tool)
    if let Some(svc) = detect("node", "node", &["--version"], "tool", parse_simple_version).await {
        let mut svc = svc;
        svc.version = svc.version.trim_start_matches('v').to_string();
        svc.display_name = format!("Node.js {}", svc.version);
        svc.has_service = false;
        found.push(svc);
    }

    // Save to cache
    save_cache(&found);

    Ok(found)
}

/// Load cached discovery (fast, no binary checks)
#[tauri::command]
pub async fn get_cached_services() -> Result<Vec<InstalledService>, String> {
    Ok(load_cache())
}

// --- Detection helpers ---

async fn detect(
    id: &str,
    binary: &str,
    args: &[&str],
    category: &str,
    version_parser: fn(&str) -> String,
) -> Option<InstalledService> {
    let output = Command::new(binary).args(args).output().await.ok()?;

    let text = if output.status.success() {
        String::from_utf8_lossy(&output.stdout).to_string()
    } else {
        // Some tools write version to stderr (nginx -v)
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        if stderr.is_empty() { return None; }
        stderr
    };

    let version = version_parser(text.trim());
    let binary_path = which_path(binary).await.unwrap_or_else(|| binary.into());

    Some(InstalledService {
        id: id.to_string(),
        display_name: String::new(), // filled by caller
        category: category.to_string(),
        binary: binary_path,
        systemd_unit: None,
        brew_service: None,
        version,
        has_service: false,
    })
}

async fn which_path(cmd: &str) -> Option<String> {
    let output = Command::new("which").arg(cmd).output().await.ok()?;
    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        None
    }
}

// --- Version parsers ---

fn parse_php_version(output: &str) -> String {
    // "PHP 8.5.4 (cli) ..." -> "8.5.4"
    output.split_whitespace().nth(1).unwrap_or("").to_string()
}

fn parse_mariadb_version(output: &str) -> String {
    // "mariadb from 12.2.2-MariaDB..." or "mariadb  Ver 15.1 Distrib 11.6.2-MariaDB"
    output
        .split("Distrib ")
        .nth(1)
        .and_then(|s| s.split(['-', ',']).next())
        .or_else(|| {
            output.split("Ver ").nth(1).and_then(|s| s.split_whitespace().next())
        })
        .unwrap_or("")
        .to_string()
}

fn parse_mysql_version(output: &str) -> String {
    output
        .split("Ver ").nth(1)
        .and_then(|s| s.split_whitespace().next())
        .unwrap_or("")
        .to_string()
}

fn parse_nginx_version(output: &str) -> String {
    // "nginx version: nginx/1.27.3" -> "1.27.3"
    output.split('/').nth(1).and_then(|s| s.split_whitespace().next()).unwrap_or("").to_string()
}

fn parse_redis_version(output: &str) -> String {
    // "Redis server v=7.4.2 ..." or "Valkey server v=9.0.3 ..."
    output.split("v=").nth(1).and_then(|s| s.split_whitespace().next()).unwrap_or("").to_string()
}

fn parse_memcached_version(output: &str) -> String {
    // "memcached 1.6.41"
    output.split_whitespace().nth(1).unwrap_or("").to_string()
}

fn parse_simple_version(output: &str) -> String {
    let first = output.lines().next().unwrap_or("");
    // Try to extract a version-like pattern
    for word in first.split_whitespace() {
        if word.chars().next().is_some_and(|c| c.is_ascii_digit()) {
            return word.to_string();
        }
        if word.starts_with('v') && word.len() > 1 && word.chars().nth(1).is_some_and(|c| c.is_ascii_digit()) {
            return word.to_string();
        }
    }
    first.to_string()
}

// --- Cache ---

fn cache_path() -> PathBuf {
    let dir = dirs_next::cache_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("unlavarel");
    std::fs::create_dir_all(&dir).ok();
    dir.join("services_cache.json")
}

fn save_cache(services: &[InstalledService]) {
    if let Ok(json) = serde_json::to_string_pretty(services) {
        std::fs::write(cache_path(), json).ok();
    }
}

fn load_cache() -> Vec<InstalledService> {
    match std::fs::read_to_string(cache_path()) {
        Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
        Err(_) => vec![],
    }
}
