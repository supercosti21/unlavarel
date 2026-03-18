use std::collections::HashMap;
use serde::Deserialize;

use crate::error::{MacEnvError, Result};

#[derive(Debug, Deserialize)]
pub struct PackageRegistry {
    #[serde(flatten)]
    pub packages: HashMap<String, PackageDefinition>,
}

#[derive(Debug, Deserialize)]
pub struct PackageDefinition {
    pub description: String,
    pub category: String,
    pub has_service: bool,
    #[serde(default)]
    pub versioned: bool,
    pub managers: HashMap<String, ManagerMapping>,
}

#[derive(Debug, Deserialize)]
pub struct ManagerMapping {
    pub pattern: String,
    pub service_name: Option<String>,
    pub tap: Option<String>,
    pub ppa: Option<String>,
    pub available_versions: Option<Vec<String>>,
    pub binary: Option<String>,
    pub binary_path_template: Option<String>,
    pub config_path_template: Option<String>,
    pub additional_packages: Option<Vec<String>>,
}

impl PackageRegistry {
    pub fn load() -> Result<Self> {
        let toml_str = include_str!("../../resources/packages.toml");
        toml::from_str(toml_str).map_err(|e| MacEnvError::Other(format!("Registry parse error: {e}")))
    }

    pub fn resolve(
        &self,
        canonical: &str,
        version: Option<&str>,
        manager_name: &str,
    ) -> Result<String> {
        let pkg = self.packages.get(canonical).ok_or_else(|| {
            MacEnvError::PackageNotInRegistry {
                package: canonical.to_string(),
                manager: manager_name.to_string(),
            }
        })?;

        let mapping = pkg.managers.get(manager_name).ok_or_else(|| {
            MacEnvError::PackageNotInRegistry {
                package: canonical.to_string(),
                manager: manager_name.to_string(),
            }
        })?;

        let resolved = match version {
            Some(v) => mapping.pattern.replace("{version}", v),
            None => mapping
                .pattern
                .replace("@{version}", "")
                .replace("{version}", ""),
        };

        Ok(resolved)
    }

    pub fn get_service_name(
        &self,
        canonical: &str,
        version: Option<&str>,
        manager_name: &str,
    ) -> Option<String> {
        let pkg = self.packages.get(canonical)?;
        let mapping = pkg.managers.get(manager_name)?;
        let svc = mapping.service_name.as_ref()?;

        let resolved = match version {
            Some(v) => svc.replace("{version}", v),
            None => svc.replace("@{version}", "").replace("{version}", ""),
        };

        Some(resolved)
    }
}
