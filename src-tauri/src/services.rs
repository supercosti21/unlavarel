use serde::{Deserialize, Serialize};

use crate::service_manager::{create_service_manager, ServiceStatus};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Service {
    pub name: String,
    pub status: String,
    pub version: String,
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
        }
    }
}

#[tauri::command]
pub async fn get_services() -> Result<Vec<Service>, String> {
    let manager = create_service_manager();
    let infos = manager.list_managed().await.map_err(|e| e.to_string())?;
    Ok(infos.iter().map(Service::from_info).collect())
}

#[tauri::command]
pub async fn start_service(name: String) -> Result<Service, String> {
    let manager = create_service_manager();
    manager.start(&name).await.map_err(|e| e.to_string())?;
    let info = manager.status(&name).await.map_err(|e| e.to_string())?;
    Ok(Service::from_info(&info))
}

#[tauri::command]
pub async fn stop_service(name: String) -> Result<Service, String> {
    let manager = create_service_manager();
    manager.stop(&name).await.map_err(|e| e.to_string())?;
    let info = manager.status(&name).await.map_err(|e| e.to_string())?;
    Ok(Service::from_info(&info))
}
