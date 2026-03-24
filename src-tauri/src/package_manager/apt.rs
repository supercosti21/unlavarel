use async_trait::async_trait;
use std::path::PathBuf;
use tokio::process::Command;

use crate::error::{MacEnvError, Result};
use crate::platform::permissions::Privilege;
use super::{InstalledPackage, PackageId, PackageManager, VersionSpec};

pub struct Apt;

impl Default for Apt {
    fn default() -> Self {
        Self::new()
    }
}

impl Apt {
    pub fn new() -> Self {
        Self
    }

    /// Run apt-get with elevation using session-cached password.
    /// No repeated pkexec prompts — uses run_script_elevated.
    async fn run_apt(&self, args: &[&str]) -> Result<String> {
        let script = format!(
            "export DEBIAN_FRONTEND=noninteractive; apt-get {}",
            args.join(" ")
        );
        let output = crate::elevated::run_script_elevated(&script)
            .await
            .map_err(MacEnvError::Other)?;

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
impl PackageManager for Apt {
    fn name(&self) -> &'static str {
        "apt"
    }

    async fn is_available(&self) -> bool {
        PathBuf::from("/usr/bin/apt-get").exists()
    }

    async fn bootstrap(&self) -> Result<()> {
        // apt is always available on Debian/Ubuntu
        Ok(())
    }

    fn resolve_native_name(&self, id: &PackageId) -> Result<String> {
        // apt uses different naming: php8.3-fpm, mysql-server-8.0, etc.
        let version_str = match &id.version {
            Some(VersionSpec::Exact(v)) | Some(VersionSpec::Minor(v)) => Some(v.as_str()),
            Some(VersionSpec::Major(v)) => Some(v.as_str()),
            Some(VersionSpec::Latest) | None => None,
        };

        match (id.canonical.as_str(), version_str) {
            ("php", Some(v)) => Ok(format!("php{}-fpm", v)),
            ("mysql", Some(v)) => Ok(format!("mysql-server-{}", v)),
            ("nginx", _) => Ok("nginx".into()),
            ("redis", _) => Ok("redis-server".into()),
            ("memcached", _) => Ok("memcached".into()),
            ("node", _) => Ok("nodejs".into()),
            ("composer", _) => Ok("composer".into()),
            (name, _) => Ok(name.to_string()),
        }
    }

    async fn install(&self, id: &PackageId) -> Result<InstalledPackage> {
        let native = self.resolve_native_name(id)?;
        self.run_apt(&["install", "-y", &native]).await?;

        Ok(InstalledPackage {
            id: id.clone(),
            native_name: native,
            installed_version: "installed".into(),
            binary_path: PathBuf::from("/usr/bin"),
        })
    }

    async fn uninstall(&self, id: &PackageId) -> Result<()> {
        let native = self.resolve_native_name(id)?;
        self.run_apt(&["remove", "-y", &native]).await?;
        Ok(())
    }

    async fn upgrade(&self, id: &PackageId) -> Result<InstalledPackage> {
        let native = self.resolve_native_name(id)?;
        self.run_apt(&["install", "--only-upgrade", "-y", &native]).await?;

        Ok(InstalledPackage {
            id: id.clone(),
            native_name: native,
            installed_version: "upgraded".into(),
            binary_path: PathBuf::from("/usr/bin"),
        })
    }

    async fn list_installed(&self) -> Result<Vec<InstalledPackage>> {
        let output = Command::new("dpkg")
            .args(["--get-selections"])
            .output()
            .await?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let packages = stdout
            .lines()
            .filter(|l| l.contains("install"))
            .filter_map(|l| l.split_whitespace().next())
            .map(|name| InstalledPackage {
                id: PackageId {
                    canonical: name.to_string(),
                    version: None,
                },
                native_name: name.to_string(),
                installed_version: String::new(),
                binary_path: PathBuf::from("/usr/bin"),
            })
            .collect();
        Ok(packages)
    }

    async fn is_installed(&self, id: &PackageId) -> Result<bool> {
        let native = self.resolve_native_name(id)?;
        let output = Command::new("dpkg")
            .args(["-l", &native])
            .output()
            .await?;
        Ok(output.status.success())
    }

    async fn available_versions(&self, _canonical: &str) -> Result<Vec<String>> {
        // Would need apt-cache for this
        Ok(vec![])
    }

    fn prefix(&self) -> PathBuf {
        PathBuf::from("/usr")
    }

    fn install_privilege(&self) -> Privilege {
        Privilege::Elevated // apt needs sudo
    }

    async fn update_index(&self) -> Result<()> {
        self.run_apt(&["update"]).await?;
        Ok(())
    }
}
