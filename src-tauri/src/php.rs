use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhpVersion {
    pub version: String,
    pub active: bool,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhpExtension {
    pub name: String,
    pub enabled: bool,
}

/// List installed PHP versions.
#[tauri::command]
pub async fn get_php_versions() -> Result<Vec<PhpVersion>, String> {
    if cfg!(target_os = "macos") {
        get_php_versions_homebrew().await
    } else if cfg!(target_os = "linux") {
        if PathBuf::from("/usr/bin/pacman").exists() {
            get_php_versions_pacman().await
        } else {
            get_php_versions_apt().await
        }
    } else {
        Ok(vec![])
    }
}

async fn get_php_versions_homebrew() -> Result<Vec<PhpVersion>, String> {
    let prefix = if cfg!(target_arch = "aarch64") {
        "/opt/homebrew"
    } else {
        "/usr/local"
    };

    let mut versions = Vec::new();

    // Check active PHP version
    let enriched_path = crate::setup::build_enriched_path();
    let active_output = Command::new("php")
        .arg("-v")
        .env("PATH", &enriched_path)
        .output()
        .await;
    let active_version = match active_output {
        Ok(o) if o.status.success() => {
            let text = String::from_utf8_lossy(&o.stdout);
            // Parse "PHP 8.4.19 ..." -> "8.4"
            text.split_whitespace()
                .nth(1)
                .and_then(|v| {
                    let parts: Vec<&str> = v.split('.').collect();
                    if parts.len() >= 2 {
                        Some(format!("{}.{}", parts[0], parts[1]))
                    } else {
                        None
                    }
                })
                .unwrap_or_default()
        }
        _ => String::new(),
    };

    // Check unversioned php first (Homebrew `brew install php` -> latest)
    let php_opt = PathBuf::from(prefix).join("opt/php");
    let mut unversioned_ver = String::new();
    if php_opt.exists() {
        // Detect what version unversioned php actually is
        if let Ok(o) = Command::new(php_opt.join("bin/php").to_string_lossy().to_string())
            .arg("-v")
            .env("PATH", &enriched_path)
            .output()
            .await
        {
            if o.status.success() {
                let text = String::from_utf8_lossy(&o.stdout);
                if let Some(v) = text.split_whitespace().nth(1) {
                    let parts: Vec<&str> = v.split('.').collect();
                    if parts.len() >= 2 {
                        unversioned_ver = format!("{}.{}", parts[0], parts[1]);
                    }
                }
            }
        }
    }

    for ver in &["8.1", "8.2", "8.3", "8.4", "8.5"] {
        let opt_path = PathBuf::from(prefix).join(format!("opt/php@{}", ver));
        if opt_path.exists() {
            versions.push(PhpVersion {
                version: ver.to_string(),
                active: active_version == *ver,
                path: opt_path.to_string_lossy().to_string(),
            });
        } else if *ver == unversioned_ver && php_opt.exists() {
            // Unversioned php matches this version
            versions.push(PhpVersion {
                version: ver.to_string(),
                active: active_version == *ver,
                path: php_opt.to_string_lossy().to_string(),
            });
        }
    }

    // Fallback: if nothing found but php exists
    if versions.is_empty() && php_opt.exists() && !active_version.is_empty() {
        versions.push(PhpVersion {
            version: active_version.clone(),
            active: true,
            path: php_opt.to_string_lossy().to_string(),
        });
    }

    Ok(versions)
}

async fn get_php_versions_apt() -> Result<Vec<PhpVersion>, String> {
    let mut versions = Vec::new();

    let enriched_path = crate::setup::build_enriched_path();
    let active_output = Command::new("php")
        .arg("-v")
        .env("PATH", &enriched_path)
        .output()
        .await;
    let active_version = match active_output {
        Ok(o) if o.status.success() => {
            let text = String::from_utf8_lossy(&o.stdout);
            // Parse "PHP 8.4.19 ..." -> "8.4"
            text.split_whitespace()
                .nth(1)
                .and_then(|v| {
                    let parts: Vec<&str> = v.split('.').collect();
                    if parts.len() >= 2 {
                        Some(format!("{}.{}", parts[0], parts[1]))
                    } else {
                        None
                    }
                })
                .unwrap_or_default()
        }
        _ => String::new(),
    };

    for ver in &["8.1", "8.2", "8.3", "8.4"] {
        let fpm_path = PathBuf::from(format!("/usr/sbin/php-fpm{}", ver));
        let cli_path = PathBuf::from(format!("/usr/bin/php{}", ver));
        if fpm_path.exists() || cli_path.exists() {
            versions.push(PhpVersion {
                version: ver.to_string(),
                active: active_version == *ver,
                path: cli_path.to_string_lossy().to_string(),
            });
        }
    }

    Ok(versions)
}

async fn get_php_versions_pacman() -> Result<Vec<PhpVersion>, String> {
    // Arch only has one PHP version in official repos
    let enriched_path = crate::setup::build_enriched_path();
    let output = Command::new("php").arg("-v").env("PATH", &enriched_path).output().await;
    match output {
        Ok(o) if o.status.success() => {
            let text = String::from_utf8_lossy(&o.stdout);
            let version = text
                .split_whitespace()
                .nth(1)
                .unwrap_or("unknown")
                .to_string();
            let minor = version
                .split('.')
                .take(2)
                .collect::<Vec<_>>()
                .join(".");
            Ok(vec![PhpVersion {
                version: minor,
                active: true,
                path: "/usr/bin/php".into(),
            }])
        }
        _ => Ok(vec![]),
    }
}

/// Switch the active PHP version (macOS Homebrew only — Linux uses update-alternatives).
#[tauri::command]
pub async fn switch_php_version(version: String) -> Result<String, String> {
    if cfg!(target_os = "macos") {
        switch_php_homebrew(&version).await
    } else if cfg!(target_os = "linux") && PathBuf::from("/usr/bin/update-alternatives").exists() {
        switch_php_apt(&version).await
    } else {
        Err("PHP version switching not supported on this platform".into())
    }
}

async fn switch_php_homebrew(version: &str) -> Result<String, String> {
    let brew = if cfg!(target_arch = "aarch64") {
        "/opt/homebrew/bin/brew"
    } else {
        "/usr/local/bin/brew"
    };

    // Unlink all PHP versions first
    for v in &["8.1", "8.2", "8.3", "8.4"] {
        let _ = Command::new(brew)
            .args(["unlink", &format!("php@{}", v)])
            .output()
            .await;
    }
    let _ = Command::new(brew).args(["unlink", "php"]).output().await;

    // Link the requested version
    let output = Command::new(brew)
        .args(["link", "--force", &format!("php@{}", version)])
        .output()
        .await
        .map_err(|e| e.to_string())?;

    if output.status.success() {
        // Restart PHP-FPM for the new version
        let _ = Command::new(brew)
            .args(["services", "restart", &format!("php@{}", version)])
            .output()
            .await;
        Ok(format!("Switched to PHP {}", version))
    } else {
        Err(format!(
            "Failed to switch PHP: {}",
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}

async fn switch_php_apt(version: &str) -> Result<String, String> {
    let output = Command::new("sudo")
        .args([
            "update-alternatives",
            "--set",
            "php",
            &format!("/usr/bin/php{}", version),
        ])
        .output()
        .await
        .map_err(|e| e.to_string())?;

    if output.status.success() {
        Ok(format!("Switched to PHP {}", version))
    } else {
        Err(format!(
            "Failed to switch PHP: {}",
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}

/// List PHP extensions and their enabled state.
#[tauri::command]
pub async fn get_php_extensions() -> Result<Vec<PhpExtension>, String> {
    let output = Command::new("php")
        .args(["-m"])
        .output()
        .await
        .map_err(|e| e.to_string())?;

    let enabled: Vec<String> = String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter(|l| !l.is_empty() && !l.starts_with('['))
        .map(|l| l.trim().to_lowercase())
        .collect();

    // Common extensions we track
    let tracked = [
        "xdebug", "opcache", "redis", "memcached", "imagick", "gd",
        "curl", "mbstring", "xml", "zip", "bcmath", "pdo_mysql",
        "pdo_pgsql", "intl", "soap", "sodium",
    ];

    let extensions = tracked
        .iter()
        .map(|name| PhpExtension {
            name: name.to_string(),
            enabled: enabled.contains(&name.to_string()),
        })
        .collect();

    Ok(extensions)
}

/// Toggle a PHP extension (enable/disable in php.ini).
#[tauri::command]
pub async fn toggle_php_extension(name: String, enable: bool) -> Result<String, String> {
    // Find php.ini path
    let output = Command::new("php")
        .args(["--ini"])
        .output()
        .await
        .map_err(|e| e.to_string())?;

    let ini_output = String::from_utf8_lossy(&output.stdout);

    // Find the loaded php.ini
    let ini_path = ini_output
        .lines()
        .find(|l| l.contains("Loaded Configuration File"))
        .and_then(|l| l.split(':').nth(1))
        .map(|p| p.trim().to_string())
        .ok_or("Could not find php.ini path")?;

    if ini_path == "(none)" {
        return Err("No php.ini file loaded".into());
    }

    let content = tokio::fs::read_to_string(&ini_path)
        .await
        .map_err(|e| e.to_string())?;

    let extension_line = format!("extension={}", name);
    let zend_line = format!("zend_extension={}", name);
    let is_zend = name == "xdebug" || name == "opcache";

    let line = if is_zend { &zend_line } else { &extension_line };
    let commented = format!(";{}", line);

    let updated = if enable {
        // Uncomment the extension line
        if content.contains(&commented) {
            content.replace(&commented, line)
        } else if !content.contains(line) {
            format!("{}\n{}\n", content, line)
        } else {
            content
        }
    } else {
        // Comment out the extension line
        if content.contains(line) && !content.contains(&commented) {
            content.replace(line, &commented)
        } else {
            content
        }
    };

    // Write back (may need sudo on Linux)
    if cfg!(target_os = "linux") {
        let mut child = tokio::process::Command::new("sudo")
            .args(["tee", &ini_path])
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::null())
            .spawn()
            .map_err(|e| e.to_string())?;

        if let Some(stdin) = child.stdin.as_mut() {
            use tokio::io::AsyncWriteExt;
            stdin
                .write_all(updated.as_bytes())
                .await
                .map_err(|e| e.to_string())?;
            stdin.shutdown().await.map_err(|e| e.to_string())?;
        }
        child.wait().await.map_err(|e| e.to_string())?;
    } else {
        tokio::fs::write(&ini_path, &updated)
            .await
            .map_err(|e| e.to_string())?;
    }

    let action = if enable { "enabled" } else { "disabled" };
    Ok(format!("{} {} — restart PHP-FPM to apply", name, action))
}
