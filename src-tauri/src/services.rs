use serde::{Deserialize, Serialize};

use crate::discovery::{get_cached_services, InstalledService};
use crate::service_manager::{create_service_manager, ServiceStatus};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Service {
    pub id: String,
    pub name: String,
    pub status: String,
    pub version: String,
    pub category: String,
    pub has_service: bool,
    pub pid: Option<u32>,
}

impl Service {
    fn from_discovery(svc: &InstalledService, status: &str, pid: Option<u32>) -> Self {
        Self {
            id: svc.id.clone(),
            name: svc.display_name.clone(),
            status: status.to_string(),
            version: svc.version.clone(),
            category: svc.category.clone(),
            has_service: svc.has_service,
            pid,
        }
    }
}

#[tauri::command]
pub async fn get_services() -> Result<Vec<Service>, String> {
    let discovered = get_cached_services().await.unwrap_or_default();

    // If cache is empty, run discovery first
    let discovered = if discovered.is_empty() {
        crate::discovery::discover_services().await.unwrap_or_default()
    } else {
        discovered
    };

    let manager = create_service_manager();
    let mut services = Vec::new();

    for svc in &discovered {
        if svc.has_service {
            let info = manager.status(&svc.id).await;
            match info {
                Ok(info) => {
                    let status = match &info.status {
                        ServiceStatus::Running => "Running",
                        ServiceStatus::Stopped => "Stopped",
                        ServiceStatus::Errored(_e) => "Error",
                        ServiceStatus::Unknown => "Stopped",
                    };
                    let status_str = match &info.status {
                        ServiceStatus::Errored(e) => format!("Error: {}", e),
                        _ => status.to_string(),
                    };
                    services.push(Service::from_discovery(svc, &status_str, info.pid));
                }
                Err(_) => {
                    services.push(Service::from_discovery(svc, "Stopped", None));
                }
            }
        } else {
            // Tools like Composer, Node — no service, just show as installed
            services.push(Service::from_discovery(svc, "Installed", None));
        }
    }

    Ok(services)
}

#[tauri::command]
pub async fn start_service(name: String) -> Result<Service, String> {
    let manager = create_service_manager();
    manager.start(&name).await.map_err(|e| e.to_string())?;

    // Return updated status
    let discovered = get_cached_services().await.unwrap_or_default();
    let svc = discovered.iter().find(|s| s.id == name);
    let info = manager.status(&name).await.map_err(|e| e.to_string())?;

    let status = match &info.status {
        ServiceStatus::Running => "Running".to_string(),
        ServiceStatus::Errored(e) => format!("Error: {}", e),
        _ => "Stopped".to_string(),
    };

    match svc {
        Some(s) => Ok(Service::from_discovery(s, &status, info.pid)),
        None => Ok(Service {
            id: name.clone(),
            name,
            status,
            version: String::new(),
            category: String::new(),
            has_service: true,
            pid: info.pid,
        }),
    }
}

#[tauri::command]
pub async fn stop_service(name: String) -> Result<Service, String> {
    let manager = create_service_manager();
    manager.stop(&name).await.map_err(|e| e.to_string())?;

    let discovered = get_cached_services().await.unwrap_or_default();
    let svc = discovered.iter().find(|s| s.id == name);
    let info = manager.status(&name).await.map_err(|e| e.to_string())?;

    let status = match &info.status {
        ServiceStatus::Running => "Running".to_string(),
        ServiceStatus::Errored(e) => format!("Error: {}", e),
        _ => "Stopped".to_string(),
    };

    match svc {
        Some(s) => Ok(Service::from_discovery(s, &status, info.pid)),
        None => Ok(Service {
            id: name.clone(),
            name,
            status,
            version: String::new(),
            category: String::new(),
            has_service: true,
            pid: info.pid,
        }),
    }
}

#[tauri::command]
pub async fn restart_service(name: String) -> Result<Service, String> {
    let manager = create_service_manager();
    manager.restart(&name).await.map_err(|e| e.to_string())?;

    let discovered = get_cached_services().await.unwrap_or_default();
    let svc = discovered.iter().find(|s| s.id == name);
    let info = manager.status(&name).await.map_err(|e| e.to_string())?;

    let status = match &info.status {
        ServiceStatus::Running => "Running".to_string(),
        ServiceStatus::Errored(e) => format!("Error: {}", e),
        _ => "Stopped".to_string(),
    };

    match svc {
        Some(s) => Ok(Service::from_discovery(s, &status, info.pid)),
        None => Ok(Service {
            id: name.clone(),
            name,
            status,
            version: String::new(),
            category: String::new(),
            has_service: true,
            pid: info.pid,
        }),
    }
}

#[tauri::command]
pub async fn start_all_services() -> Result<Vec<Service>, String> {
    let manager = create_service_manager();
    let discovered = get_cached_services().await.unwrap_or_default();

    for svc in &discovered {
        if svc.has_service {
            manager.start(&svc.id).await.ok();
        }
    }

    get_services().await
}

#[tauri::command]
pub async fn stop_all_services() -> Result<Vec<Service>, String> {
    let manager = create_service_manager();
    let discovered = get_cached_services().await.unwrap_or_default();

    for svc in &discovered {
        if svc.has_service {
            manager.stop(&svc.id).await.ok();
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

/// Uninstall a package
#[tauri::command]
pub async fn uninstall_package(package_id: String) -> Result<String, String> {
    let pm = crate::package_manager::create_package_manager();
    let id = crate::package_manager::PackageId {
        canonical: package_id.clone(),
        version: None,
    };

    // Stop the service first if running
    let manager = create_service_manager();
    manager.stop(&package_id).await.ok();

    pm.uninstall(&id).await.map_err(|e| e.to_string())?;

    // Refresh discovery cache
    crate::discovery::discover_services().await.ok();

    Ok(format!("{} uninstalled", package_id))
}
