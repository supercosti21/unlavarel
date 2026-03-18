use serde::{Deserialize, Serialize};
use tokio::process::Command;

use crate::service_manager::{create_service_manager, ServiceStatus};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Service {
    pub name: String,
    pub status: String,
    pub version: String,
    pub pid: Option<u32>,
}

impl Service {
    fn from_info(info: &crate::service_manager::ServiceInfo) -> Self {
        let status = match &info.status {
            ServiceStatus::Running => "Running".to_string(),
            ServiceStatus::Stopped => "Stopped".to_string(),
            ServiceStatus::Errored(e) => format!("Error: {}", e),
            ServiceStatus::Unknown => "Unknown".to_string(),
        };

        Self {
            name: info.name.clone(),
            status,
            version: info.version.clone(),
            pid: info.pid,
        }
    }
}

#[tauri::command]
pub async fn get_services() -> Result<Vec<Service>, String> {
    let manager = create_service_manager();
    let infos = manager.list_managed().await.map_err(|e| e.to_string())?;
    let mut services: Vec<Service> = infos.iter().map(Service::from_info).collect();

    // Enrich with version info from binaries
    for svc in &mut services {
        if svc.version.is_empty() {
            svc.version = detect_version(&svc.name).await;
        }
    }

    Ok(services)
}

#[tauri::command]
pub async fn start_service(name: String) -> Result<Service, String> {
    let manager = create_service_manager();
    manager.start(&name).await.map_err(|e| e.to_string())?;
    let info = manager.status(&name).await.map_err(|e| e.to_string())?;
    let mut svc = Service::from_info(&info);
    if svc.version.is_empty() {
        svc.version = detect_version(&svc.name).await;
    }
    Ok(svc)
}

#[tauri::command]
pub async fn stop_service(name: String) -> Result<Service, String> {
    let manager = create_service_manager();
    manager.stop(&name).await.map_err(|e| e.to_string())?;
    let info = manager.status(&name).await.map_err(|e| e.to_string())?;
    let mut svc = Service::from_info(&info);
    if svc.version.is_empty() {
        svc.version = detect_version(&svc.name).await;
    }
    Ok(svc)
}

#[tauri::command]
pub async fn restart_service(name: String) -> Result<Service, String> {
    let manager = create_service_manager();
    manager.restart(&name).await.map_err(|e| e.to_string())?;
    let info = manager.status(&name).await.map_err(|e| e.to_string())?;
    let mut svc = Service::from_info(&info);
    if svc.version.is_empty() {
        svc.version = detect_version(&svc.name).await;
    }
    Ok(svc)
}

#[tauri::command]
pub async fn start_all_services() -> Result<Vec<Service>, String> {
    let manager = create_service_manager();
    let infos = manager.list_managed().await.map_err(|e| e.to_string())?;

    for info in &infos {
        if info.status != ServiceStatus::Running {
            manager.start(&info.name).await.ok(); // best effort
        }
    }

    get_services().await
}

#[tauri::command]
pub async fn stop_all_services() -> Result<Vec<Service>, String> {
    let manager = create_service_manager();
    let infos = manager.list_managed().await.map_err(|e| e.to_string())?;

    for info in &infos {
        if info.status == ServiceStatus::Running {
            manager.stop(&info.name).await.ok();
        }
    }

    get_services().await
}

#[tauri::command]
pub async fn get_service_logs(name: String, lines: Option<usize>) -> Result<Vec<String>, String> {
    let manager = create_service_manager();
    let log_text = manager
        .logs(&name, lines.unwrap_or(100))
        .await
        .map_err(|e| e.to_string())?;
    Ok(log_text.lines().map(|l| l.to_string()).collect())
}

/// Detect the version of a service binary.
async fn detect_version(service: &str) -> String {
    let (binary, args) = if service == "php" || service.starts_with("php@") {
        ("php", vec!["-v"])
    } else if service == "mysql" || service.starts_with("mysql@") {
        ("mysql", vec!["--version"])
    } else if service == "mariadb" {
        ("mariadb", vec!["--version"])
    } else if service == "nginx" {
        ("nginx", vec!["-v"])
    } else if service == "redis" {
        ("redis-server", vec!["--version"])
    } else if service == "memcached" {
        ("memcached", vec!["-h"])
    } else if service == "postgresql" || service.starts_with("postgresql@") {
        ("psql", vec!["--version"])
    } else if service == "node" || service.starts_with("node@") {
        ("node", vec!["--version"])
    } else if service == "composer" {
        ("composer", vec!["--version"])
    } else if service == "dnsmasq" {
        ("dnsmasq", vec!["--version"])
    } else if service == "mailpit" {
        ("mailpit", vec!["version"])
    } else {
        return String::new();
    };

    let output = Command::new(binary).args(&args).output().await;

    match output {
        Ok(o) => {
            let text = if o.status.success() {
                String::from_utf8_lossy(&o.stdout).to_string()
            } else {
                // Some tools (like nginx -v) write to stderr
                String::from_utf8_lossy(&o.stderr).to_string()
            };
            // Extract just the version number from the first line
            parse_version_string(service, text.trim())
        }
        Err(_) => String::new(),
    }
}

/// Parse version string from command output.
fn parse_version_string(service: &str, output: &str) -> String {
    let first_line = output.lines().next().unwrap_or("");

    match service {
        s if s.starts_with("php") => {
            // "PHP 8.3.14 (cli) ..." -> "8.3.14"
            first_line
                .split_whitespace()
                .nth(1)
                .unwrap_or(first_line)
                .to_string()
        }
        s if s.contains("mysql") || s.contains("mariadb") => {
            // "mysql  Ver 8.4.3 for ..." -> "8.4.3"
            // or "mariadb  Ver 15.1 Distrib 11.6.2-MariaDB" -> "11.6.2"
            first_line
                .split("Distrib ")
                .nth(1)
                .and_then(|s| s.split(['-', ',']).next())
                .or_else(|| {
                    first_line
                        .split("Ver ")
                        .nth(1)
                        .and_then(|s| s.split_whitespace().next())
                })
                .unwrap_or(first_line)
                .to_string()
        }
        "nginx" => {
            // "nginx version: nginx/1.27.3" -> "1.27.3"
            first_line
                .split('/')
                .nth(1)
                .unwrap_or(first_line)
                .to_string()
        }
        "redis" => {
            // "Redis server v=7.4.2 ..." -> "7.4.2"
            first_line
                .split("v=")
                .nth(1)
                .and_then(|s| s.split_whitespace().next())
                .unwrap_or(first_line)
                .to_string()
        }
        s if s.starts_with("postgresql") => {
            // "psql (PostgreSQL) 17.2" -> "17.2"
            first_line
                .split(')')
                .nth(1)
                .map(|s| s.trim())
                .unwrap_or(first_line)
                .to_string()
        }
        s if s.starts_with("node") => {
            // "v22.12.0" -> "22.12.0"
            first_line.trim_start_matches('v').to_string()
        }
        "composer" => {
            // "Composer version 2.8.4 2024-12-11 ..." -> "2.8.4"
            first_line
                .split("version ")
                .nth(1)
                .and_then(|s| s.split_whitespace().next())
                .unwrap_or(first_line)
                .to_string()
        }
        _ => first_line.to_string(),
    }
}
