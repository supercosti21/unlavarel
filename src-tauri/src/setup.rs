use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::package_manager::{create_package_manager, PackageId, VersionSpec};
use crate::platform::detect::{detect_platform, DetectedPlatform};

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
    pub extras: Vec<String>, // "redis", "memcached", "mailpit", "node"
    pub node_version: Option<String>,
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
    let pm = create_package_manager();
    let mut results = Vec::new();

    // PHP
    let php_id = PackageId {
        canonical: "php".into(),
        version: Some(VersionSpec::Minor(selection.php_version.clone())),
    };
    match pm.install(&php_id).await {
        Ok(_) => results.push(format!("PHP {} installed", selection.php_version)),
        Err(e) => results.push(format!("PHP {} failed: {}", selection.php_version, e)),
    }

    // Database
    let db_id = PackageId {
        canonical: selection.database.clone(),
        version: selection
            .database_version
            .as_ref()
            .map(|v| VersionSpec::Minor(v.clone())),
    };
    match pm.install(&db_id).await {
        Ok(_) => results.push(format!("{} installed", selection.database)),
        Err(e) => results.push(format!("{} failed: {}", selection.database, e)),
    }

    // Nginx (always)
    let nginx_id = PackageId {
        canonical: "nginx".into(),
        version: None,
    };
    match pm.install(&nginx_id).await {
        Ok(_) => results.push("Nginx installed".into()),
        Err(e) => results.push(format!("Nginx failed: {}", e)),
    }

    // Composer (always for Laravel devs)
    let composer_id = PackageId {
        canonical: "composer".into(),
        version: None,
    };
    match pm.install(&composer_id).await {
        Ok(_) => results.push("Composer installed".into()),
        Err(e) => results.push(format!("Composer failed: {}", e)),
    }

    // Extras
    for extra in &selection.extras {
        let id = match extra.as_str() {
            "node" => PackageId {
                canonical: "node".into(),
                version: selection
                    .node_version
                    .as_ref()
                    .map(|v| VersionSpec::Major(v.clone())),
            },
            other => PackageId {
                canonical: other.to_string(),
                version: None,
            },
        };
        match pm.install(&id).await {
            Ok(_) => results.push(format!("{} installed", extra)),
            Err(e) => results.push(format!("{} failed: {}", extra, e)),
        }
    }

    // dnsmasq + mkcert (always for local dev)
    for tool in &["dnsmasq", "mkcert"] {
        let id = PackageId {
            canonical: tool.to_string(),
            version: None,
        };
        match pm.install(&id).await {
            Ok(_) => results.push(format!("{} installed", tool)),
            Err(e) => results.push(format!("{} failed: {}", tool, e)),
        }
    }

    // Mark setup as complete
    std::fs::write(setup_complete_file(), "done").ok();

    Ok(results)
}

#[tauri::command]
pub async fn mark_setup_complete() -> Result<(), String> {
    std::fs::write(setup_complete_file(), "done").map_err(|e| e.to_string())?;
    Ok(())
}
