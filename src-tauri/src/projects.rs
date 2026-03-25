use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::process::Command;

use crate::ssl;
use crate::vhosts;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub name: String,
    pub path: String,
    pub domain: String,
    pub ssl: bool,
    pub database: Option<String>,
}

fn projects_file() -> PathBuf {
    let data_dir = dirs_next::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("unlavarel");
    std::fs::create_dir_all(&data_dir).ok();
    data_dir.join("projects.json")
}

fn load_projects_from_disk() -> Vec<Project> {
    let path = projects_file();
    match std::fs::read_to_string(&path) {
        Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
        Err(_) => vec![],
    }
}

fn save_projects_to_disk(projects: &[Project]) -> Result<(), String> {
    let path = projects_file();
    let json = serde_json::to_string_pretty(projects).map_err(|e| e.to_string())?;
    std::fs::write(&path, json).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn get_projects() -> Result<Vec<Project>, String> {
    Ok(load_projects_from_disk())
}

#[tauri::command]
pub async fn add_project(
    name: String,
    path: String,
    create_db: Option<bool>,
) -> Result<Project, String> {
    let mut projects = load_projects_from_disk();

    if projects.iter().any(|p| p.name == name) {
        return Err(format!("Project '{}' already exists", name));
    }

    let domain = format!("{}.test", name);
    let create_db = create_db.unwrap_or(true);

    // 1. Generate SSL certificate
    let has_ssl = match ssl::generate_cert(&domain).await {
        Ok(_) => true,
        Err(e) => {
            eprintln!("SSL cert generation failed for {}: {}", domain, e);
            false
        }
    };

    // 2. Create Nginx vhost
    if let Err(e) = vhosts::add_vhost(&domain, &path, has_ssl).await {
        eprintln!("Vhost creation failed for {}: {}", domain, e);
    }

    // 3. Create database (if requested)
    let database = if create_db {
        match create_database(&name).await {
            Ok(db_name) => Some(db_name),
            Err(e) => {
                eprintln!("Database creation failed for {}: {}", name, e);
                None
            }
        }
    } else {
        None
    };

    let project = Project {
        name,
        path,
        domain,
        ssl: has_ssl,
        database,
    };

    projects.push(project.clone());
    save_projects_to_disk(&projects)?;

    Ok(project)
}

#[tauri::command]
pub async fn remove_project(name: String) -> Result<(), String> {
    let mut projects = load_projects_from_disk();
    let project = projects.iter().find(|p| p.name == name).cloned();

    if let Some(proj) = &project {
        // Remove Nginx vhost
        if let Err(e) = vhosts::remove_vhost(&proj.domain).await {
            eprintln!("Failed to remove vhost for {}: {}", proj.domain, e);
        }

        // Remove SSL certificate
        if let Err(e) = ssl::remove_cert(&proj.domain).await {
            eprintln!("Failed to remove cert for {}: {}", proj.domain, e);
        }

        // Note: we do NOT drop the database automatically — too destructive
    }

    projects.retain(|p| p.name != name);
    save_projects_to_disk(&projects)?;

    Ok(())
}

/// Detect project environment from .env file (DB_DATABASE, DB_CONNECTION, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectEnvInfo {
    pub db_connection: Option<String>,  // mysql, pgsql, sqlite, mariadb
    pub db_database: Option<String>,
    pub db_host: Option<String>,
    pub db_port: Option<String>,
    pub db_username: Option<String>,
    pub project_type: String,           // laravel, symfony, wordpress, php, node, unknown
    pub app_name: Option<String>,
}

#[tauri::command]
pub async fn detect_project_env(path: String) -> Result<ProjectEnvInfo, String> {
    let dir = PathBuf::from(&path);
    if !dir.is_dir() {
        return Err(format!("'{}' is not a valid directory", path));
    }

    // Detect project type
    let project_type = if dir.join("artisan").exists() {
        "laravel"
    } else if dir.join("symfony.lock").exists() {
        "symfony"
    } else if dir.join("wp-config.php").exists() || dir.join("wp-config-sample.php").exists() {
        "wordpress"
    } else if dir.join("composer.json").exists() {
        "php"
    } else if dir.join("package.json").exists() {
        "node"
    } else {
        "unknown"
    };

    let mut info = ProjectEnvInfo {
        db_connection: None,
        db_database: None,
        db_host: None,
        db_port: None,
        db_username: None,
        project_type: project_type.to_string(),
        app_name: None,
    };

    // Try to read .env file (Laravel/PHP projects)
    let env_path = dir.join(".env");
    if env_path.exists() {
        if let Ok(content) = tokio::fs::read_to_string(&env_path).await {
            for line in content.lines() {
                let line = line.trim();
                if line.starts_with('#') || !line.contains('=') { continue; }
                let mut parts = line.splitn(2, '=');
                let key = parts.next().unwrap_or("").trim();
                let val = parts.next().unwrap_or("").trim().trim_matches('"').trim_matches('\'');
                if val.is_empty() { continue; }
                match key {
                    "DB_CONNECTION" => info.db_connection = Some(val.to_string()),
                    "DB_DATABASE" => info.db_database = Some(val.to_string()),
                    "DB_HOST" => info.db_host = Some(val.to_string()),
                    "DB_PORT" => info.db_port = Some(val.to_string()),
                    "DB_USERNAME" => info.db_username = Some(val.to_string()),
                    "APP_NAME" => info.app_name = Some(val.to_string()),
                    _ => {}
                }
            }
        }
    }

    // WordPress: read wp-config.php for DB_NAME
    if project_type == "wordpress" {
        let wp_config = dir.join("wp-config.php");
        if wp_config.exists() {
            if let Ok(content) = tokio::fs::read_to_string(&wp_config).await {
                for line in content.lines() {
                    if line.contains("DB_NAME") {
                        // define( 'DB_NAME', 'mydb' );
                        if let Some(val) = line.split('\'').nth(3).or_else(|| line.split('"').nth(3)) {
                            info.db_database = Some(val.to_string());
                        }
                    }
                    if line.contains("DB_HOST") {
                        if let Some(val) = line.split('\'').nth(3).or_else(|| line.split('"').nth(3)) {
                            info.db_host = Some(val.to_string());
                        }
                    }
                    if line.contains("DB_USER") && !line.contains("DB_USER\"") {
                        if let Some(val) = line.split('\'').nth(3).or_else(|| line.split('"').nth(3)) {
                            info.db_username = Some(val.to_string());
                        }
                    }
                }
                if info.db_connection.is_none() {
                    info.db_connection = Some("mysql".to_string());
                }
            }
        }
    }

    Ok(info)
}

/// Import an existing project folder into Unlavarel (creates vhost + SSL, optionally DB)
#[tauri::command]
pub async fn import_project(
    name: String,
    path: String,
    create_db: Option<bool>,
) -> Result<Project, String> {
    let dir = std::path::Path::new(&path);
    if !dir.is_dir() {
        return Err(format!("'{}' is not a valid directory", path));
    }

    // Reuse add_project — it already handles duplicate check, SSL, vhost, DB
    add_project(name, path, create_db).await
}

/// Scan a directory for PHP/Laravel projects (looks for artisan, composer.json, index.php)
#[tauri::command]
pub async fn scan_projects(directory: String) -> Result<Vec<ScannedProject>, String> {
    let dir = PathBuf::from(&directory);
    if !dir.is_dir() {
        return Err(format!("'{}' is not a directory", directory));
    }

    let mut found = Vec::new();
    let existing = load_projects_from_disk();
    let existing_paths: std::collections::HashSet<String> = existing.iter().map(|p| p.path.clone()).collect();

    let mut entries = tokio::fs::read_dir(&dir).await.map_err(|e| e.to_string())?;
    while let Ok(Some(entry)) = entries.next_entry().await {
        let path = entry.path();
        if !path.is_dir() { continue; }

        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string();
        if name.starts_with('.') { continue; }

        let path_str = path.to_string_lossy().to_string();
        let already_added = existing_paths.contains(&path_str);

        // Detect project type
        let project_type = if path.join("artisan").exists() {
            "laravel"
        } else if path.join("symfony.lock").exists() {
            "symfony"
        } else if path.join("wp-config.php").exists() || path.join("wp-config-sample.php").exists() {
            "wordpress"
        } else if path.join("composer.json").exists() {
            "php"
        } else if path.join("package.json").exists() {
            "node"
        } else {
            continue; // Not a recognized project
        };

        found.push(ScannedProject {
            name,
            path: path_str,
            project_type: project_type.to_string(),
            already_added,
        });
    }

    found.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(found)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScannedProject {
    pub name: String,
    pub path: String,
    pub project_type: String,
    pub already_added: bool,
}

/// Create a MySQL database with the given name.
/// Tries mysql first, then falls back to mariadb.
async fn create_database(name: &str) -> Result<String, String> {
    // Sanitize: only allow alphanumeric and underscores
    let db_name: String = name
        .chars()
        .map(|c| if c.is_alphanumeric() || c == '_' { c } else { '_' })
        .collect();

    // Try MySQL socket connection (no password for local dev)
    let mysql_cmd = Command::new("mysql")
        .args(["-u", "root", "-e", &format!("CREATE DATABASE IF NOT EXISTS `{}`", db_name)])
        .output()
        .await;

    match mysql_cmd {
        Ok(output) if output.status.success() => return Ok(db_name),
        _ => {}
    }

    // Try MariaDB (Arch Linux default)
    let mariadb_cmd = Command::new("mariadb")
        .args(["-u", "root", "-e", &format!("CREATE DATABASE IF NOT EXISTS `{}`", db_name)])
        .output()
        .await;

    match mariadb_cmd {
        Ok(output) if output.status.success() => return Ok(db_name),
        _ => {}
    }

    // Try PostgreSQL
    let psql_cmd = Command::new("createdb")
        .arg(&db_name)
        .output()
        .await;

    match psql_cmd {
        Ok(output) if output.status.success() => return Ok(db_name),
        _ => {}
    }

    Err(format!(
        "Could not create database '{}': no database server responded",
        db_name
    ))
}
