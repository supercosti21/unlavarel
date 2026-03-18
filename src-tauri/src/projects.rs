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
        .join("macenv");
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
