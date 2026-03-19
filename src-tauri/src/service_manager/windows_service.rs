use async_trait::async_trait;

use super::{ServiceInfo, ServiceManager, ServiceStatus};
use crate::error::{MacEnvError, Result};

pub struct WindowsService;

impl Default for WindowsService {
    fn default() -> Self {
        Self::new()
    }
}

impl WindowsService {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl ServiceManager for WindowsService {
    fn name(&self) -> &'static str {
        "windows services"
    }

    async fn start(&self, _service: &str) -> Result<()> {
        Err(MacEnvError::UnsupportedPlatform {
            os: "Windows service management not yet implemented".into(),
        })
    }

    async fn stop(&self, _service: &str) -> Result<()> {
        Err(MacEnvError::UnsupportedPlatform {
            os: "Windows service management not yet implemented".into(),
        })
    }

    async fn restart(&self, _service: &str) -> Result<()> {
        Err(MacEnvError::UnsupportedPlatform {
            os: "Windows service management not yet implemented".into(),
        })
    }

    async fn status(&self, service: &str) -> Result<ServiceInfo> {
        Ok(ServiceInfo {
            name: service.to_string(),
            status: ServiceStatus::Unknown,
            version: String::new(),
            pid: None,
        })
    }

    async fn list_managed(&self) -> Result<Vec<ServiceInfo>> {
        Ok(vec![])
    }

    async fn logs(&self, _service: &str, _lines: usize) -> Result<String> {
        Ok("Windows log viewing not yet implemented".into())
    }
}
