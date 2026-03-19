use serde::Deserialize;
use std::collections::HashMap;

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
        toml::from_str(toml_str)
            .map_err(|e| MacEnvError::Other(format!("Registry parse error: {e}")))
    }

    pub fn resolve(
        &self,
        canonical: &str,
        version: Option<&str>,
        manager_name: &str,
    ) -> Result<String> {
        let pkg =
            self.packages
                .get(canonical)
                .ok_or_else(|| MacEnvError::PackageNotInRegistry {
                    package: canonical.to_string(),
                    manager: manager_name.to_string(),
                })?;

        let mapping =
            pkg.managers
                .get(manager_name)
                .ok_or_else(|| MacEnvError::PackageNotInRegistry {
                    package: canonical.to_string(),
                    manager: manager_name.to_string(),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_registry() {
        let registry = PackageRegistry::load().unwrap();
        assert!(registry.packages.contains_key("php"));
        assert!(registry.packages.contains_key("nginx"));
        assert!(registry.packages.contains_key("redis"));
        assert!(registry.packages.contains_key("mysql"));
        assert!(registry.packages.contains_key("postgresql"));
    }

    #[test]
    fn test_resolve_php_homebrew_with_version() {
        let registry = PackageRegistry::load().unwrap();
        let resolved = registry.resolve("php", Some("8.3"), "homebrew").unwrap();
        assert_eq!(resolved, "php@8.3");
    }

    #[test]
    fn test_resolve_php_apt_with_version() {
        let registry = PackageRegistry::load().unwrap();
        let resolved = registry.resolve("php", Some("8.3"), "apt").unwrap();
        assert_eq!(resolved, "php8.3-fpm");
    }

    #[test]
    fn test_resolve_nginx_no_version() {
        let registry = PackageRegistry::load().unwrap();
        let resolved = registry.resolve("nginx", None, "homebrew").unwrap();
        assert_eq!(resolved, "nginx");
    }

    #[test]
    fn test_resolve_unknown_package() {
        let registry = PackageRegistry::load().unwrap();
        let result = registry.resolve("nonexistent", None, "homebrew");
        assert!(result.is_err());
    }

    #[test]
    fn test_resolve_unknown_manager() {
        let registry = PackageRegistry::load().unwrap();
        let result = registry.resolve("php", Some("8.3"), "nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_get_service_name_php_homebrew() {
        let registry = PackageRegistry::load().unwrap();
        let name = registry.get_service_name("php", Some("8.3"), "homebrew");
        assert_eq!(name, Some("php@8.3".to_string()));
    }

    #[test]
    fn test_get_service_name_php_apt() {
        let registry = PackageRegistry::load().unwrap();
        let name = registry.get_service_name("php", Some("8.3"), "apt");
        assert_eq!(name, Some("php8.3-fpm".to_string()));
    }

    #[test]
    fn test_get_service_name_nginx() {
        let registry = PackageRegistry::load().unwrap();
        let name = registry.get_service_name("nginx", None, "homebrew");
        assert_eq!(name, Some("nginx".to_string()));
    }

    #[test]
    fn test_get_service_name_no_service() {
        let registry = PackageRegistry::load().unwrap();
        // composer has no service
        let name = registry.get_service_name("composer", None, "homebrew");
        assert_eq!(name, None);
    }

    #[test]
    fn test_package_categories() {
        let registry = PackageRegistry::load().unwrap();
        assert_eq!(registry.packages["php"].category, "language");
        assert_eq!(registry.packages["nginx"].category, "webserver");
        assert_eq!(registry.packages["redis"].category, "cache");
        assert_eq!(registry.packages["mysql"].category, "database");
        assert_eq!(registry.packages["composer"].category, "tool");
        assert_eq!(registry.packages["dnsmasq"].category, "dns");
        assert_eq!(registry.packages["mailpit"].category, "mail");
    }

    #[test]
    fn test_package_has_service() {
        let registry = PackageRegistry::load().unwrap();
        assert!(registry.packages["php"].has_service);
        assert!(registry.packages["nginx"].has_service);
        assert!(!registry.packages["composer"].has_service);
        assert!(!registry.packages["node"].has_service);
    }

    #[test]
    fn test_available_versions() {
        let registry = PackageRegistry::load().unwrap();
        let php_brew = &registry.packages["php"].managers["homebrew"];
        let versions = php_brew.available_versions.as_ref().unwrap();
        assert!(versions.contains(&"8.3".to_string()));
        assert!(versions.contains(&"8.4".to_string()));
    }
}
