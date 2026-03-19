pub mod brew_services;
pub mod systemd;
pub mod windows_service;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::error::Result;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ServiceStatus {
    Running,
    Stopped,
    #[serde(rename = "Error")]
    Errored(String),
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInfo {
    pub name: String,
    pub status: ServiceStatus,
    pub version: String,
    pub pid: Option<u32>,
}

#[async_trait]
pub trait ServiceManager: Send + Sync {
    fn name(&self) -> &'static str;
    async fn start(&self, service: &str) -> Result<()>;
    async fn stop(&self, service: &str) -> Result<()>;
    async fn restart(&self, service: &str) -> Result<()>;
    async fn status(&self, service: &str) -> Result<ServiceInfo>;
    async fn list_managed(&self) -> Result<Vec<ServiceInfo>>;
    async fn logs(&self, service: &str, lines: usize) -> Result<String>;
}

use crate::platform::{current_os, OsType};

pub fn create_service_manager() -> Box<dyn ServiceManager> {
    match current_os() {
        OsType::MacOS => Box::new(brew_services::BrewServices::new()),
        OsType::Linux => Box::new(systemd::Systemd::new()),
        OsType::Windows => Box::new(windows_service::WindowsService::new()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_status_serialization() {
        let running = ServiceStatus::Running;
        let json = serde_json::to_string(&running).unwrap();
        assert_eq!(json, "\"Running\"");

        let stopped = ServiceStatus::Stopped;
        let json = serde_json::to_string(&stopped).unwrap();
        assert_eq!(json, "\"Stopped\"");

        let errored = ServiceStatus::Errored("timeout".into());
        let json = serde_json::to_string(&errored).unwrap();
        assert!(json.contains("timeout"));
    }

    #[test]
    fn test_service_status_equality() {
        assert_eq!(ServiceStatus::Running, ServiceStatus::Running);
        assert_eq!(ServiceStatus::Stopped, ServiceStatus::Stopped);
        assert_ne!(ServiceStatus::Running, ServiceStatus::Stopped);
    }

    #[test]
    fn test_service_info_serialization() {
        let info = ServiceInfo {
            name: "nginx".into(),
            status: ServiceStatus::Running,
            version: "1.27.3".into(),
            pid: Some(1234),
        };
        let json = serde_json::to_string(&info).unwrap();
        let parsed: ServiceInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.name, "nginx");
        assert_eq!(parsed.pid, Some(1234));
    }

    #[test]
    fn test_create_service_manager_returns_impl() {
        let manager = create_service_manager();
        assert!(!manager.name().is_empty());
    }
}
