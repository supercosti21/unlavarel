use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::process::Command;

use crate::platform::detect::{detect_platform, DetectedPlatform};
use crate::platform::{current_os, OsType};


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetupState {
    pub platform: DetectedPlatform,
    pub package_manager_available: bool,
    pub package_manager_name: String,
    pub first_run: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StackSelection {
    pub php_version: String,
    pub database: String,
    pub database_version: Option<String>,
    pub extras: Vec<String>,
    pub node_version: Option<String>,
}

/// What's already installed on the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreScanResult {
    pub installed: Vec<PreScanItem>,
    pub missing: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreScanItem {
    pub name: String,
    pub version: String,
    /// Machine-readable ID (e.g. "php", "mysql", "redis")
    pub id: String,
    /// Extracted version number (e.g. "8.3" for PHP, "8.0" for MySQL)
    pub version_number: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResult {
    pub checks: Vec<HealthCheck>,
    pub all_ok: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheck {
    pub name: String,
    pub status: String,
    pub detail: String,
}

fn config_dir() -> PathBuf {
    let dir = dirs_next::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("unlavarel");
    std::fs::create_dir_all(&dir).ok();
    dir
}

fn setup_complete_file() -> PathBuf {
    config_dir().join("setup_complete")
}

#[tauri::command]
pub async fn check_setup() -> Result<SetupState, String> {
    let platform = detect_platform().await;
    let pm = crate::package_manager::create_package_manager();
    let available = pm.is_available().await;
    let first_run = !setup_complete_file().exists();

    Ok(SetupState {
        platform,
        package_manager_available: available,
        package_manager_name: pm.name().to_string(),
        first_run,
    })
}

#[tauri::command]
pub async fn bootstrap_package_manager() -> Result<String, String> {
    let pm = crate::package_manager::create_package_manager();
    if pm.is_available().await {
        return Ok(format!("{} is already installed", pm.name()));
    }
    pm.bootstrap().await.map_err(|e| e.to_string())?;
    Ok(format!("{} installed successfully", pm.name()))
}

/// Pre-scan: check what's already installed before showing install options
#[tauri::command]
pub async fn pre_scan_system() -> Result<PreScanResult, String> {
    // (display_name, id, binary, args)
    let checks: Vec<(&str, &str, &str, Vec<&str>)> = vec![
        ("PHP", "php", "php", vec!["-v"]),
        ("Composer", "composer", "composer", vec!["--version"]),
        ("Nginx", "nginx", "nginx", vec!["-v"]),
        ("MySQL/MariaDB", "mysql", "mysql", vec!["--version"]),
        ("PostgreSQL", "postgresql", "psql", vec!["--version"]),
        ("Redis", "redis", "redis-server", vec!["--version"]),
        ("Memcached", "memcached", "memcached", vec!["-h"]),
        ("Node.js", "node", "node", vec!["--version"]),
        ("dnsmasq", "dnsmasq", "dnsmasq", vec!["--version"]),
        ("mkcert", "mkcert", "mkcert", vec!["--version"]),
        ("Mailpit", "mailpit", "mailpit", vec!["version"]),
    ];

    let mut installed = Vec::new();
    let mut missing = Vec::new();

    for (name, id, binary, args) in &checks {
        let output = Command::new(binary).args(args.as_slice()).output().await;
        match output {
            Ok(o) => {
                let text = if o.status.success() {
                    String::from_utf8_lossy(&o.stdout).to_string()
                } else {
                    String::from_utf8_lossy(&o.stderr).to_string()
                };
                let first_line = text.lines().next().unwrap_or("").trim();
                if !first_line.is_empty() {
                    // Extract version number from output
                    let version_number = extract_version_number(first_line, id);

                    // Detect if "mysql" is actually MariaDB
                    let actual_id = if *id == "mysql" && first_line.to_lowercase().contains("mariadb") {
                        "mariadb"
                    } else {
                        id
                    };

                    installed.push(PreScanItem {
                        name: name.to_string(),
                        version: first_line.to_string(),
                        id: actual_id.to_string(),
                        version_number,
                    });
                } else {
                    missing.push(name.to_string());
                }
            }
            Err(_) => {
                missing.push(name.to_string());
            }
        }
    }

    Ok(PreScanResult { installed, missing })
}

/// Extract a meaningful version number from command output
fn extract_version_number(output: &str, id: &str) -> Option<String> {
    // Find patterns like X.Y or X.Y.Z
    let re_pattern = regex_lite::Regex::new(r"(\d+\.\d+(?:\.\d+)?)").ok()?;
    let captures = re_pattern.captures(output)?;
    let full_version = captures.get(1)?.as_str();

    match id {
        // PHP: return major.minor (e.g. "8.3" from "8.3.15")
        "php" => {
            let parts: Vec<&str> = full_version.split('.').collect();
            if parts.len() >= 2 {
                Some(format!("{}.{}", parts[0], parts[1]))
            } else {
                Some(full_version.to_string())
            }
        }
        // Node: return major (e.g. "22" from "22.5.0")
        "node" => {
            let parts: Vec<&str> = full_version.split('.').collect();
            parts.first().map(|s| s.to_string())
        }
        _ => Some(full_version.to_string()),
    }
}

#[tauri::command]
pub async fn install_stack(selection: StackSelection) -> Result<Vec<String>, String> {
    match current_os() {
        OsType::MacOS => install_stack_homebrew(selection).await,
        OsType::Linux => install_stack_linux(selection).await,
        OsType::Windows => Err("Windows installation not yet supported".into()),
    }
}

/// macOS: Homebrew doesn't need sudo
async fn install_stack_homebrew(selection: StackSelection) -> Result<Vec<String>, String> {
    let pm = crate::package_manager::create_package_manager();
    let packages = build_homebrew_list(&selection);
    let mut results = Vec::new();

    for (display_name, pkg) in &packages {
        let id = crate::package_manager::PackageId {
            canonical: pkg.clone(),
            version: None,
        };
        match pm.install(&id).await {
            Ok(_) => results.push(format!("{} installed", display_name)),
            Err(e) => results.push(format!("{} failed: {}", display_name, e)),
        }
    }

    std::fs::write(setup_complete_file(), "done").ok();
    Ok(results)
}

/// Linux: write a SINGLE bash script and run it ONCE with pkexec
/// This means the user enters the password exactly ONE time.
async fn install_stack_linux(selection: StackSelection) -> Result<Vec<String>, String> {
    let is_arch = PathBuf::from("/usr/bin/pacman").exists();
    let packages = build_linux_package_list(&selection, is_arch);
    let aur_packages = build_aur_list(&selection);

    if packages.is_empty() && aur_packages.is_empty() {
        std::fs::write(setup_complete_file(), "done").ok();
        return Ok(vec!["Nothing to install".into()]);
    }

    // Build a single bash script that does everything
    // NOTE: no `set -e` — we want to continue even if one package fails
    let mut script = String::from("#!/bin/bash\nFAILED=0\n\n");

    // Package manager install — one package at a time so failures don't block others
    if !packages.is_empty() {
        if is_arch {
            for pkg in &packages {
                script.push_str(&format!(
                    "pacman -S --noconfirm --needed {} || {{ echo \"FAIL:{}\"; FAILED=1; }}\n",
                    pkg, pkg
                ));
            }
        } else {
            script.push_str("export DEBIAN_FRONTEND=noninteractive\n");
            for pkg in &packages {
                script.push_str(&format!(
                    "apt-get install -y {} || {{ echo \"FAIL:{}\"; FAILED=1; }}\n",
                    pkg, pkg
                ));
            }
        }
    }

    // AUR / direct download packages
    for (name, url) in &aur_packages {
        script.push_str(&format!(
            "\n# Install {} from binary\n\
             curl -L -o /tmp/{name}.tar.gz {url}\n\
             tar -xzf /tmp/{name}.tar.gz -C /tmp/\n\
             install -m 755 /tmp/{name} /usr/local/bin/{name}\n\
             rm -f /tmp/{name}.tar.gz /tmp/{name}\n",
            name,
            name = name,
            url = url,
        ));
    }

    // Run with elevated privileges — uses session-cached password if available,
    // otherwise falls back to pkexec (ONE password prompt for everything)
    let output = crate::elevated::run_script_elevated(&script)
        .await
        .map_err(|e| format!("Failed to run installer: {}", e))?;

    let mut results = Vec::new();
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Parse output — look for FAIL: markers per package
    let failed_pkgs: Vec<&str> = stdout
        .lines()
        .filter_map(|l| l.strip_prefix("FAIL:"))
        .collect();

    for pkg in &packages {
        if failed_pkgs.iter().any(|f| f == pkg) {
            results.push(format!("{} failed", pkg));
        } else {
            results.push(format!("{} installed", pkg));
        }
    }
    for (name, _) in &aur_packages {
        if failed_pkgs.iter().any(|f| f == name) {
            results.push(format!("{} failed", name));
        } else {
            results.push(format!("{} installed", name));
        }
    }

    std::fs::write(setup_complete_file(), "done").ok();
    Ok(results)
}

fn build_homebrew_list(selection: &StackSelection) -> Vec<(String, String)> {
    let mut packages = Vec::new();

    packages.push((format!("PHP {}", selection.php_version), format!("php@{}", selection.php_version)));

    match selection.database.as_str() {
        "mysql" => {
            let ver = selection.database_version.as_deref().unwrap_or("8.4");
            packages.push((format!("MySQL {}", ver), format!("mysql@{}", ver)));
        }
        "mariadb" => packages.push(("MariaDB".into(), "mariadb".into())),
        "postgresql" => {
            let ver = selection.database_version.as_deref().unwrap_or("17");
            packages.push((format!("PostgreSQL {}", ver), format!("postgresql@{}", ver)));
        }
        _ => {}
    }

    packages.push(("Nginx".into(), "nginx".into()));
    packages.push(("Composer".into(), "composer".into()));

    for extra in &selection.extras {
        match extra.as_str() {
            "redis" => packages.push(("Redis".into(), "redis".into())),
            "memcached" => packages.push(("Memcached".into(), "memcached".into())),
            "mailpit" => packages.push(("Mailpit".into(), "mailpit".into())),
            "node" => {
                let ver = selection.node_version.as_deref().unwrap_or("22");
                packages.push((format!("Node.js {}", ver), format!("node@{}", ver)));
            }
            _ => {}
        }
    }

    packages.push(("dnsmasq".into(), "dnsmasq".into()));
    packages.push(("mkcert".into(), "mkcert".into()));
    packages
}

fn build_linux_package_list(selection: &StackSelection, is_arch: bool) -> Vec<String> {
    let mut packages = Vec::new();

    if is_arch {
        packages.extend(["php", "php-fpm"].iter().map(|s| s.to_string()));
    } else {
        let v = &selection.php_version;
        for suffix in &["fpm", "cli", "common", "mysql", "xml", "curl", "mbstring", "zip"] {
            packages.push(format!("php{}-{}", v, suffix));
        }
    }

    match selection.database.as_str() {
        "mysql" | "mariadb" => {
            if is_arch {
                packages.push("mariadb".into());
            } else if selection.database == "mysql" {
                packages.push("mysql-server".into());
            } else {
                packages.push("mariadb-server".into());
            }
        }
        "postgresql" => packages.push("postgresql".into()),
        _ => {}
    }

    packages.push("nginx".into());
    packages.push("composer".into());

    for extra in &selection.extras {
        match extra.as_str() {
            "redis" => packages.push(if is_arch { "redis".into() } else { "redis-server".into() }),
            "memcached" => packages.push("memcached".into()),
            "node" => {
                packages.push(if is_arch { "nodejs".into() } else { "nodejs".into() });
                if is_arch { packages.push("npm".into()); }
            }
            "mailpit" => {} // handled as AUR
            _ => {}
        }
    }

    packages.push("dnsmasq".into());
    packages.push("mkcert".into());
    packages
}

fn build_aur_list(selection: &StackSelection) -> Vec<(String, String)> {
    let mut packages = Vec::new();
    if selection.extras.contains(&"mailpit".to_string()) {
        packages.push((
            "mailpit".into(),
            "https://github.com/axllent/mailpit/releases/latest/download/mailpit-linux-amd64.tar.gz".into(),
        ));
    }
    packages
}

/// Install a single package by its canonical ID (e.g. "php", "mysql", "redis")
#[tauri::command]
pub async fn install_single_package(package_id: String) -> Result<String, String> {
    let pm = crate::package_manager::create_package_manager();

    let pkg = crate::package_manager::PackageId {
        canonical: package_id.clone(),
        version: None,
    };

    // Check if already installed
    if pm.is_installed(&pkg).await.unwrap_or(false) {
        return Ok(format!("{} is already installed", package_id));
    }

    match pm.install(&pkg).await {
        Ok(installed) => Ok(format!(
            "{} ({}) installed successfully",
            package_id, installed.installed_version
        )),
        Err(e) => Err(format!("Failed to install {}: {}", package_id, e)),
    }
}

async fn check_binary(name: &str) -> bool {
    Command::new("which")
        .arg(name)
        .output()
        .await
        .map(|o| o.status.success())
        .unwrap_or(false)
}

#[tauri::command]
pub async fn mark_setup_complete() -> Result<(), String> {
    std::fs::write(setup_complete_file(), "done").map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn health_check() -> Result<HealthResult, String> {
    let mut checks = Vec::new();

    let binaries = [
        ("PHP", "php", "--version"),
        ("Composer", "composer", "--version"),
        ("Nginx", "nginx", "-v"),
        ("dnsmasq", "dnsmasq", "--version"),
        ("mkcert", "mkcert", "--version"),
    ];

    for (name, binary, arg) in &binaries {
        let output = Command::new(binary).arg(arg).output().await;
        match output {
            Ok(o) => {
                let text = if o.status.success() {
                    String::from_utf8_lossy(&o.stdout)
                } else {
                    String::from_utf8_lossy(&o.stderr)
                };
                let first = text.lines().next().unwrap_or("").trim().to_string();
                if !first.is_empty() {
                    checks.push(HealthCheck { name: name.to_string(), status: "ok".into(), detail: first });
                } else {
                    checks.push(HealthCheck { name: name.to_string(), status: "error".into(), detail: "Returns empty".into() });
                }
            }
            Err(_) => {
                checks.push(HealthCheck { name: name.to_string(), status: "missing".into(), detail: format!("{} not found", binary) });
            }
        }
    }

    let optionals = [
        ("MySQL/MariaDB", &["mysql", "mariadb"][..]),
        ("PostgreSQL", &["psql"][..]),
        ("Redis", &["redis-server"][..]),
        ("Node.js", &["node"][..]),
        ("Mailpit", &["mailpit"][..]),
    ];

    for (name, candidates) in &optionals {
        let mut found = false;
        for binary in *candidates {
            if let Ok(o) = Command::new(binary).arg("--version").output().await {
                let text = if o.status.success() {
                    String::from_utf8_lossy(&o.stdout)
                } else {
                    String::from_utf8_lossy(&o.stderr)
                };
                let first = text.lines().next().unwrap_or("").trim().to_string();
                if !first.is_empty() {
                    checks.push(HealthCheck { name: name.to_string(), status: "ok".into(), detail: first });
                    found = true;
                    break;
                }
            }
        }
        if !found {
            checks.push(HealthCheck { name: name.to_string(), status: "missing".into(), detail: "Not installed".into() });
        }
    }

    let dns_ok = crate::dns::is_configured().await.unwrap_or(false);
    checks.push(HealthCheck {
        name: "DNS (*.test)".into(),
        status: if dns_ok { "ok" } else { "missing" }.into(),
        detail: if dns_ok { "dnsmasq configured".into() } else { "Not configured".into() },
    });

    let ca_ok = crate::ssl::is_ca_installed().await;
    checks.push(HealthCheck {
        name: "SSL CA".into(),
        status: if ca_ok { "ok" } else { "missing" }.into(),
        detail: if ca_ok { "Local CA trusted".into() } else { "Run mkcert -install".into() },
    });

    let all_ok = checks.iter().all(|c| c.status == "ok");
    Ok(HealthResult { checks, all_ok })
}
