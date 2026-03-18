use async_trait::async_trait;
use std::path::PathBuf;
use tokio::process::Command;

use crate::error::{MacEnvError, Result};
use crate::platform::permissions::Privilege;
use super::{InstalledPackage, PackageId, PackageManager, VersionSpec};

pub struct Homebrew {
    brew_path: PathBuf,
}

impl Default for Homebrew {
    fn default() -> Self {
        Self::new()
    }
}

impl Homebrew {
    pub fn new() -> Self {
        // Apple Silicon path first, then Intel
        let brew_path = if PathBuf::from("/opt/homebrew/bin/brew").exists() {
            PathBuf::from("/opt/homebrew/bin/brew")
        } else {
            PathBuf::from("/usr/local/bin/brew")
        };
        Self { brew_path }
    }

    async fn run_brew(&self, args: &[&str]) -> Result<String> {
        let output = Command::new(&self.brew_path)
            .args(args)
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
impl PackageManager for Homebrew {
    fn name(&self) -> &'static str {
        "homebrew"
    }

    async fn is_available(&self) -> bool {
        self.brew_path.exists()
    }

    async fn bootstrap(&self) -> Result<()> {
        let output = Command::new("/bin/bash")
            .arg("-c")
            .arg("NONINTERACTIVE=1 /bin/bash -c \"$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)\"")
            .output()
            .await?;

        if output.status.success() {
            Ok(())
        } else {
            Err(MacEnvError::BootstrapFailed {
                manager: "Homebrew".into(),
                reason: String::from_utf8_lossy(&output.stderr).into(),
            })
        }
    }

    fn resolve_native_name(&self, id: &PackageId) -> Result<String> {
        let version_str = match &id.version {
            Some(VersionSpec::Exact(v)) | Some(VersionSpec::Minor(v)) => Some(v.as_str()),
            Some(VersionSpec::Major(v)) => Some(v.as_str()),
            Some(VersionSpec::Latest) | None => None,
        };

        match version_str {
            Some(v) => Ok(format!("{}@{}", id.canonical, v)),
            None => Ok(id.canonical.clone()),
        }
    }

    async fn install(&self, id: &PackageId) -> Result<InstalledPackage> {
        let native = self.resolve_native_name(id)?;
        self.run_brew(&["install", &native]).await?;

        Ok(InstalledPackage {
            id: id.clone(),
            native_name: native.clone(),
            installed_version: "installed".into(),
            binary_path: self.prefix().join("opt").join(&native).join("bin"),
        })
    }

    async fn uninstall(&self, id: &PackageId) -> Result<()> {
        let native = self.resolve_native_name(id)?;
        self.run_brew(&["uninstall", &native]).await?;
        Ok(())
    }

    async fn upgrade(&self, id: &PackageId) -> Result<InstalledPackage> {
        let native = self.resolve_native_name(id)?;
        self.run_brew(&["upgrade", &native]).await?;

        Ok(InstalledPackage {
            id: id.clone(),
            native_name: native.clone(),
            installed_version: "upgraded".into(),
            binary_path: self.prefix().join("opt").join(&native).join("bin"),
        })
    }

    async fn list_installed(&self) -> Result<Vec<InstalledPackage>> {
        let output = self.run_brew(&["list", "--formula", "-1"]).await?;
        let packages = output
            .lines()
            .filter(|l| !l.is_empty())
            .map(|name| InstalledPackage {
                id: PackageId {
                    canonical: name.to_string(),
                    version: None,
                },
                native_name: name.to_string(),
                installed_version: String::new(),
                binary_path: self.prefix().join("opt").join(name).join("bin"),
            })
            .collect();
        Ok(packages)
    }

    async fn is_installed(&self, id: &PackageId) -> Result<bool> {
        let native = self.resolve_native_name(id)?;
        let result = self.run_brew(&["list", &native]).await;
        Ok(result.is_ok())
    }

    async fn available_versions(&self, canonical: &str) -> Result<Vec<String>> {
        let output = self.run_brew(&["search", &format!("{}@", canonical)]).await?;
        let versions: Vec<String> = output
            .lines()
            .filter_map(|l| {
                l.strip_prefix(&format!("{}@", canonical))
                    .map(|v| v.to_string())
            })
            .collect();
        Ok(versions)
    }

    fn prefix(&self) -> PathBuf {
        if cfg!(target_arch = "aarch64") {
            PathBuf::from("/opt/homebrew")
        } else {
            PathBuf::from("/usr/local")
        }
    }

    fn install_privilege(&self) -> Privilege {
        Privilege::User // Homebrew never needs sudo
    }

    async fn update_index(&self) -> Result<()> {
        self.run_brew(&["update"]).await?;
        Ok(())
    }
}
