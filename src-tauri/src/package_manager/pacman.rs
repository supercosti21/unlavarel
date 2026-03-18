use async_trait::async_trait;
use std::path::PathBuf;
use tokio::process::Command;

use crate::error::{MacEnvError, Result};
use crate::platform::permissions::Privilege;
use super::{InstalledPackage, PackageId, PackageManager};

pub struct Pacman;

impl Default for Pacman {
    fn default() -> Self {
        Self::new()
    }
}

impl Pacman {
    pub fn new() -> Self {
        Self
    }

    async fn run_pacman(&self, args: &[&str]) -> Result<String> {
        let output = Command::new("pacman")
            .args(args)
            .arg("--noconfirm")
            .output()
            .await?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(MacEnvError::Other(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ))
        }
    }
}

#[async_trait]
impl PackageManager for Pacman {
    fn name(&self) -> &'static str {
        "pacman"
    }

    async fn is_available(&self) -> bool {
        PathBuf::from("/usr/bin/pacman").exists()
    }

    async fn bootstrap(&self) -> Result<()> {
        // pacman is always available on Arch
        Ok(())
    }

    fn resolve_native_name(&self, id: &PackageId) -> Result<String> {
        // pacman uses simple names, no versioning in package name
        match id.canonical.as_str() {
            "php" => Ok("php".into()),
            "mysql" => Ok("mariadb".into()), // Arch defaults to MariaDB
            "nginx" => Ok("nginx".into()),
            "redis" => Ok("redis".into()),
            "memcached" => Ok("memcached".into()),
            "node" => Ok("nodejs".into()),
            "composer" => Ok("composer".into()),
            "dnsmasq" => Ok("dnsmasq".into()),
            "mailpit" => Ok("mailpit".into()), // AUR
            name => Ok(name.to_string()),
        }
    }

    async fn install(&self, id: &PackageId) -> Result<InstalledPackage> {
        let native = self.resolve_native_name(id)?;
        self.run_pacman(&["-S", &native]).await?;

        Ok(InstalledPackage {
            id: id.clone(),
            native_name: native,
            installed_version: "installed".into(),
            binary_path: PathBuf::from("/usr/bin"),
        })
    }

    async fn uninstall(&self, id: &PackageId) -> Result<()> {
        let native = self.resolve_native_name(id)?;
        self.run_pacman(&["-R", &native]).await?;
        Ok(())
    }

    async fn upgrade(&self, id: &PackageId) -> Result<InstalledPackage> {
        let native = self.resolve_native_name(id)?;
        self.run_pacman(&["-S", &native]).await?;

        Ok(InstalledPackage {
            id: id.clone(),
            native_name: native,
            installed_version: "upgraded".into(),
            binary_path: PathBuf::from("/usr/bin"),
        })
    }

    async fn list_installed(&self) -> Result<Vec<InstalledPackage>> {
        let output = self.run_pacman(&["-Q"]).await?;
        let packages = output
            .lines()
            .filter_map(|l| {
                let mut parts = l.split_whitespace();
                let name = parts.next()?;
                let version = parts.next().unwrap_or("");
                Some(InstalledPackage {
                    id: PackageId {
                        canonical: name.to_string(),
                        version: None,
                    },
                    native_name: name.to_string(),
                    installed_version: version.to_string(),
                    binary_path: PathBuf::from("/usr/bin"),
                })
            })
            .collect();
        Ok(packages)
    }

    async fn is_installed(&self, id: &PackageId) -> Result<bool> {
        let native = self.resolve_native_name(id)?;
        let result = self.run_pacman(&["-Q", &native]).await;
        Ok(result.is_ok())
    }

    async fn available_versions(&self, _canonical: &str) -> Result<Vec<String>> {
        // Arch typically only has the latest version in repos
        Ok(vec!["latest".into()])
    }

    fn prefix(&self) -> PathBuf {
        PathBuf::from("/usr")
    }

    fn install_privilege(&self) -> Privilege {
        Privilege::Elevated // pacman needs sudo
    }

    async fn update_index(&self) -> Result<()> {
        self.run_pacman(&["-Sy"]).await?;
        Ok(())
    }
}
