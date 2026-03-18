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
