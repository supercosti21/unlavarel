pub mod homebrew;
pub mod apt;
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
