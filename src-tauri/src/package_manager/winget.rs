use async_trait::async_trait;
use std::path::PathBuf;

use crate::error::{MacEnvError, Result};
use crate::platform::permissions::Privilege;
use super::{InstalledPackage, PackageId, PackageManager};

pub struct Winget;

impl Default for Winget {
    fn default() -> Self {
        Self::new()
    }
}

impl Winget {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl PackageManager for Winget {
    fn name(&self) -> &'static str {
        "winget"
    }

    async fn is_available(&self) -> bool {
        #[cfg(target_os = "windows")]
        {
            tokio::process::Command::new("winget")
                .arg("--version")
                .output()
                .await
                .map(|o| o.status.success())
                .unwrap_or(false)
        }
        #[cfg(not(target_os = "windows"))]
        false
    }

    async fn bootstrap(&self) -> Result<()> {
        Err(MacEnvError::UnsupportedPlatform {
            os: "Windows winget bootstrap not yet implemented".into(),
        })
    }

    fn resolve_native_name(&self, id: &PackageId) -> Result<String> {
        // Winget uses publisher.package format
        match id.canonical.as_str() {
            "php" => Ok("PHP.PHP".into()),
            "mysql" => Ok("Oracle.MySQL".into()),
            "nginx" => Ok("Nginx.Nginx".into()),
            "redis" => Ok("Redis.Redis".into()),
            "node" => Ok("OpenJS.NodeJS".into()),
            "composer" => Ok("Composer.Composer".into()),
            name => Ok(name.to_string()),
        }
    }

    async fn install(&self, _id: &PackageId) -> Result<InstalledPackage> {
        Err(MacEnvError::UnsupportedPlatform {
            os: "Windows winget install not yet implemented".into(),
        })
    }

    async fn uninstall(&self, _id: &PackageId) -> Result<()> {
        Err(MacEnvError::UnsupportedPlatform {
            os: "Windows winget uninstall not yet implemented".into(),
        })
    }

    async fn upgrade(&self, _id: &PackageId) -> Result<InstalledPackage> {
        Err(MacEnvError::UnsupportedPlatform {
            os: "Windows winget upgrade not yet implemented".into(),
        })
    }

    async fn list_installed(&self) -> Result<Vec<InstalledPackage>> {
        Ok(vec![])
    }

    async fn is_installed(&self, _id: &PackageId) -> Result<bool> {
        Ok(false)
    }

    async fn available_versions(&self, _canonical: &str) -> Result<Vec<String>> {
        Ok(vec![])
    }

    fn prefix(&self) -> PathBuf {
        PathBuf::from("C:\\Program Files")
    }

    fn install_privilege(&self) -> Privilege {
        Privilege::Elevated
    }

    async fn update_index(&self) -> Result<()> {
        Ok(())
    }
}
