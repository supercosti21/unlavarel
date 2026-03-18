use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::process::Command;

use crate::package_manager::create_package_manager;
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResult {
    pub checks: Vec<HealthCheck>,
    pub all_ok: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheck {
    pub name: String,
    pub status: String, // "ok", "missing", "error"
    pub detail: String,
}

fn config_dir() -> PathBuf {
    let dir = dirs_next::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("macenv");
    std::fs::create_dir_all(&dir).ok();
    dir
}

fn setup_complete_file() -> PathBuf {
    config_dir().join("setup_complete")
}

#[tauri::command]
pub async fn check_setup() -> Result<SetupState, String> {
    let platform = detect_platform().await;
    let pm = create_package_manager();
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
    let pm = create_package_manager();

    if pm.is_available().await {
        return Ok(format!("{} is already installed", pm.name()));
    }

    pm.bootstrap().await.map_err(|e| e.to_string())?;
    Ok(format!("{} installed successfully", pm.name()))
}

#[tauri::command]
pub async fn install_stack(selection: StackSelection) -> Result<Vec<String>, String> {
    match current_os() {
        OsType::MacOS => install_stack_homebrew(selection).await,
        OsType::Linux => install_stack_linux(selection).await,
        OsType::Windows => Err("Windows installation not yet supported".into()),
    }
}

/// macOS: Homebrew doesn't need sudo — install one by one
async fn install_stack_homebrew(selection: StackSelection) -> Result<Vec<String>, String> {
    let pm = create_package_manager();
    let packages = build_package_list(&selection);
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

/// Linux: batch ALL packages into ONE pkexec call (single password prompt)
async fn install_stack_linux(selection: StackSelection) -> Result<Vec<String>, String> {
    let packages = build_native_package_list_linux(&selection);
    let mut results = Vec::new();

    if packages.is_empty() {
        return Ok(vec!["No packages to install".into()]);
    }

    // Detect package manager
    let (cmd, install_args) = if PathBuf::from("/usr/bin/pacman").exists() {
        ("pacman", vec!["-S", "--noconfirm", "--needed"])
    } else if PathBuf::from("/usr/bin/apt-get").exists() {
        ("apt-get", vec!["install", "-y"])
    } else {
        return Err("No supported package manager found".into());
    };

    // Single pkexec call with all packages
    let mut args = install_args;
    for pkg in &packages {
        args.push(pkg);
    }

    let output = Command::new("pkexec")
        .arg(cmd)
        .args(&args)
        .output()
        .await
        .map_err(|e| format!("Failed to run pkexec: {}", e))?;

    let stderr = String::from_utf8_lossy(&output.stderr);

    if output.status.success() {
        for pkg in &packages {
            results.push(format!("{} installed", pkg));
        }
    } else {
        // Some may have succeeded, some failed — report the error
        results.push(format!("Installation output: {}", stderr.trim()));
        // Still check which ones are now installed
        for pkg in &packages {
            if check_binary_exists(pkg).await {
                results.push(format!("{} installed", pkg));
            } else {
                results.push(format!("{} may have failed", pkg));
            }
        }
    }

    // Handle AUR packages separately (mailpit etc.)
    let aur_packages = build_aur_package_list(&selection);
    for (name, url) in &aur_packages {
        match install_from_url(name, url).await {
            Ok(_) => results.push(format!("{} installed", name)),
            Err(e) => results.push(format!("{} failed: {}", name, e)),
        }
    }

    std::fs::write(setup_complete_file(), "done").ok();
    Ok(results)
}

/// Build list of (display_name, canonical_name) for Homebrew
fn build_package_list(selection: &StackSelection) -> Vec<(String, String)> {
    let mut packages = Vec::new();

    // PHP
    packages.push((
        format!("PHP {}", selection.php_version),
        format!("php@{}", selection.php_version),
    ));

    // Database
    match selection.database.as_str() {
        "mysql" => {
            let ver = selection.database_version.as_deref().unwrap_or("8.4");
            packages.push((format!("MySQL {}", ver), format!("mysql@{}", ver)));
        }
        "mariadb" => {
            packages.push(("MariaDB".into(), "mariadb".into()));
        }
        "postgresql" => {
            let ver = selection.database_version.as_deref().unwrap_or("17");
            packages.push((format!("PostgreSQL {}", ver), format!("postgresql@{}", ver)));
        }
        _ => {}
    }

    // Always
    packages.push(("Nginx".into(), "nginx".into()));
    packages.push(("Composer".into(), "composer".into()));

    // Extras
    for extra in &selection.extras {
        match extra.as_str() {
            "redis" => packages.push(("Redis".into(), "redis".into())),
            "memcached" => packages.push(("Memcached".into(), "memcached".into())),
            "mailpit" => packages.push(("Mailpit".into(), "mailpit".into())),
            "node" => {
                let ver = selection.node_version.as_deref().unwrap_or("22");
                packages.push((format!("Node.js {}", ver), format!("node@{}", ver)));
            }
            _ => packages.push((extra.clone(), extra.clone())),
        }
    }

    // Tools
    packages.push(("dnsmasq".into(), "dnsmasq".into()));
    packages.push(("mkcert".into(), "mkcert".into()));

    packages
}

/// Build list of native package names for pacman/apt (NOT AUR)
fn build_native_package_list_linux(selection: &StackSelection) -> Vec<String> {
    let is_arch = PathBuf::from("/usr/bin/pacman").exists();
    let mut packages = Vec::new();

    // PHP
    if is_arch {
        packages.push("php".into());
        packages.push("php-fpm".into());
    } else {
        let v = &selection.php_version;
        packages.push(format!("php{}-fpm", v));
        packages.push(format!("php{}-cli", v));
        packages.push(format!("php{}-common", v));
        packages.push(format!("php{}-mysql", v));
        packages.push(format!("php{}-xml", v));
        packages.push(format!("php{}-curl", v));
        packages.push(format!("php{}-mbstring", v));
        packages.push(format!("php{}-zip", v));
    }

    // Database
    match selection.database.as_str() {
        "mysql" => {
            if is_arch {
                // Arch doesn't have MySQL in official repos, use MariaDB
                packages.push("mariadb".into());
            } else {
                packages.push("mysql-server".into());
            }
        }
        "mariadb" => packages.push("mariadb".into()),
        "postgresql" => packages.push("postgresql".into()),
        _ => {}
    }

    packages.push("nginx".into());
    packages.push("composer".into());

    for extra in &selection.extras {
        match extra.as_str() {
            "redis" => {
                if is_arch {
                    packages.push("redis".into());
                } else {
                    packages.push("redis-server".into());
                }
            }
            "memcached" => packages.push("memcached".into()),
            "node" => {
                if is_arch {
                    packages.push("nodejs".into());
                    packages.push("npm".into());
                } else {
                    packages.push("nodejs".into());
                }
            }
            // mailpit is AUR — handled separately
            "mailpit" => {}
            _ => packages.push(extra.clone()),
        }
    }

    packages.push("dnsmasq".into());
    packages.push("mkcert".into());

    packages
}

/// Build list of (name, download_url) for AUR/non-repo packages
fn build_aur_package_list(selection: &StackSelection) -> Vec<(String, String)> {
    let mut packages = Vec::new();

    if selection.extras.contains(&"mailpit".to_string()) {
        // Install mailpit via direct binary download
        packages.push((
            "mailpit".into(),
            "https://github.com/axllent/mailpit/releases/latest/download/mailpit-linux-amd64.tar.gz".into(),
        ));
    }

    packages
}

/// Install a binary from a tar.gz URL to /usr/local/bin
async fn install_from_url(name: &str, url: &str) -> Result<(), String> {
    let tmp = format!("/tmp/{}.tar.gz", name);

    // Download
    let dl = Command::new("curl")
        .args(["-L", "-o", &tmp, url])
        .output()
        .await
        .map_err(|e| e.to_string())?;

    if !dl.status.success() {
        return Err(format!("Download failed: {}", String::from_utf8_lossy(&dl.stderr)));
    }

    // Extract to /tmp
    let extract = Command::new("tar")
        .args(["-xzf", &tmp, "-C", "/tmp"])
        .output()
        .await
        .map_err(|e| e.to_string())?;

    if !extract.status.success() {
        return Err("Extract failed".into());
    }

    // Move binary to /usr/local/bin with pkexec
    let mv = Command::new("pkexec")
        .args(["install", "-m", "755", &format!("/tmp/{}", name), &format!("/usr/local/bin/{}", name)])
        .output()
        .await
        .map_err(|e| e.to_string())?;

    if !mv.status.success() {
        return Err(format!("Install to /usr/local/bin failed: {}", String::from_utf8_lossy(&mv.stderr)));
    }

    // Cleanup
    tokio::fs::remove_file(&tmp).await.ok();

    Ok(())
}

async fn check_binary_exists(name: &str) -> bool {
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

/// Health check — verify all dependencies and services
#[tauri::command]
pub async fn health_check() -> Result<HealthResult, String> {
    let mut checks = Vec::new();

    // Check binaries
    let binaries = [
        ("PHP", "php"),
        ("Composer", "composer"),
        ("Nginx", "nginx"),
        ("dnsmasq", "dnsmasq"),
        ("mkcert", "mkcert"),
    ];

    for (name, binary) in &binaries {
        let output = Command::new(binary).arg("--version").output().await;
        match output {
            Ok(o) if o.status.success() => {
                let ver = String::from_utf8_lossy(&o.stdout);
                let first = ver.lines().next().unwrap_or("").trim().to_string();
                checks.push(HealthCheck {
                    name: name.to_string(),
                    status: "ok".into(),
                    detail: first,
                });
            }
            Ok(o) => {
                // Some tools output version to stderr (nginx -v)
                let ver = String::from_utf8_lossy(&o.stderr);
                let first = ver.lines().next().unwrap_or("").trim().to_string();
                if !first.is_empty() {
                    checks.push(HealthCheck {
                        name: name.to_string(),
                        status: "ok".into(),
                        detail: first,
                    });
                } else {
                    checks.push(HealthCheck {
                        name: name.to_string(),
                        status: "error".into(),
                        detail: "Binary found but returned error".into(),
                    });
                }
            }
            Err(_) => {
                checks.push(HealthCheck {
                    name: name.to_string(),
                    status: "missing".into(),
                    detail: format!("{} not found in PATH", binary),
                });
            }
        }
    }

    // Check optional binaries
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
            let output = Command::new(binary).arg("--version").output().await;
            if let Ok(o) = output {
                let text = if o.status.success() {
                    String::from_utf8_lossy(&o.stdout).to_string()
                } else {
                    String::from_utf8_lossy(&o.stderr).to_string()
                };
                let first = text.lines().next().unwrap_or("").trim().to_string();
                if !first.is_empty() {
                    checks.push(HealthCheck {
                        name: name.to_string(),
                        status: "ok".into(),
                        detail: first,
                    });
                    found = true;
                    break;
                }
            }
        }
        if !found {
            checks.push(HealthCheck {
                name: name.to_string(),
                status: "missing".into(),
                detail: "Not installed".into(),
            });
        }
    }

    // Check dnsmasq config
    let dns_configured = crate::dns::is_configured().await.unwrap_or(false);
    checks.push(HealthCheck {
        name: "DNS (*.test)".into(),
        status: if dns_configured { "ok" } else { "missing" }.into(),
        detail: if dns_configured {
            "dnsmasq configured for .test TLD".into()
        } else {
            "dnsmasq not configured — .test domains won't resolve".into()
        },
    });

    // Check mkcert CA
    let ca_installed = crate::ssl::is_ca_installed().await;
    checks.push(HealthCheck {
        name: "SSL CA".into(),
        status: if ca_installed { "ok" } else { "missing" }.into(),
        detail: if ca_installed {
            "Local CA installed and trusted".into()
        } else {
            "Local CA not installed — run mkcert -install".into()
        },
    });

    // Check nginx sites dir
    let sites_dir = crate::vhosts::nginx_sites_dir();
    checks.push(HealthCheck {
        name: "Nginx sites dir".into(),
        status: if sites_dir.exists() { "ok" } else { "missing" }.into(),
        detail: sites_dir.to_string_lossy().to_string(),
    });

    let all_ok = checks.iter().all(|c| c.status == "ok");

    Ok(HealthResult { checks, all_ok })
}
