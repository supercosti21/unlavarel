pub mod apt;
pub mod homebrew;
pub mod pacman;
pub mod winget;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::error::Result;
use crate::platform::permissions::Privilege;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct PackageId {
    pub canonical: String,
    pub version: Option<VersionSpec>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum VersionSpec {
    Exact(String),
    Minor(String),
    Major(String),
    Latest,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledPackage {
    pub id: PackageId,
    pub native_name: String,
    pub installed_version: String,
    pub binary_path: PathBuf,
}

#[async_trait]
pub trait PackageManager: Send + Sync {
    fn name(&self) -> &'static str;
    async fn is_available(&self) -> bool;
    async fn bootstrap(&self) -> Result<()>;
    fn resolve_native_name(&self, id: &PackageId) -> Result<String>;
    async fn install(&self, id: &PackageId) -> Result<InstalledPackage>;
    async fn uninstall(&self, id: &PackageId) -> Result<()>;
    async fn upgrade(&self, id: &PackageId) -> Result<InstalledPackage>;
    async fn list_installed(&self) -> Result<Vec<InstalledPackage>>;
    async fn is_installed(&self, id: &PackageId) -> Result<bool>;
    async fn available_versions(&self, canonical: &str) -> Result<Vec<String>>;
    fn prefix(&self) -> PathBuf;
    fn install_privilege(&self) -> Privilege;
    async fn update_index(&self) -> Result<()>;
}

use crate::platform::{current_os, OsType};

/// Create the appropriate package manager for the current OS.
pub fn create_package_manager() -> Box<dyn PackageManager> {
    match current_os() {
        OsType::MacOS => Box::new(homebrew::Homebrew::new()),
        OsType::Linux => {
            // Try pacman first (Arch), then fallback to apt
            if std::path::Path::new("/usr/bin/pacman").exists() {
                Box::new(pacman::Pacman::new())
            } else {
                Box::new(apt::Apt::new())
            }
        }
        OsType::Windows => Box::new(winget::Winget::new()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_package_id_serialization() {
        let id = PackageId {
            canonical: "php".into(),
            version: Some(VersionSpec::Minor("8.3".into())),
        };
        let json = serde_json::to_string(&id).unwrap();
        let parsed: PackageId = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.canonical, "php");
        assert_eq!(parsed.version, Some(VersionSpec::Minor("8.3".into())));
    }

    #[test]
    fn test_package_id_no_version() {
        let id = PackageId {
            canonical: "nginx".into(),
            version: None,
        };
        let json = serde_json::to_string(&id).unwrap();
        let parsed: PackageId = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.canonical, "nginx");
        assert_eq!(parsed.version, None);
    }

    #[test]
    fn test_version_spec_variants() {
        let exact = VersionSpec::Exact("8.3.12".into());
        let minor = VersionSpec::Minor("8.3".into());
        let major = VersionSpec::Major("8".into());
        let latest = VersionSpec::Latest;

        assert_ne!(exact, minor);
        assert_ne!(minor, major);
        assert_ne!(major, latest);
    }

    #[test]
    fn test_installed_package_serialization() {
        let pkg = InstalledPackage {
            id: PackageId {
                canonical: "nginx".into(),
                version: None,
            },
            native_name: "nginx".into(),
            installed_version: "1.27.3".into(),
            binary_path: PathBuf::from("/usr/sbin/nginx"),
        };
        let json = serde_json::to_string(&pkg).unwrap();
        let parsed: InstalledPackage = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.native_name, "nginx");
        assert_eq!(parsed.installed_version, "1.27.3");
    }

    #[test]
    fn test_create_package_manager_returns_impl() {
        let pm = create_package_manager();
        assert!(!pm.name().is_empty());
    }
}
