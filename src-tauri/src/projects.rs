use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub name: String,
    pub path: String,
    pub domain: String,
    pub ssl: bool,
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
pub async fn add_project(name: String, path: String) -> Result<Project, String> {
    let mut projects = load_projects_from_disk();

    if projects.iter().any(|p| p.name == name) {
        return Err(format!("Project '{}' already exists", name));
    }

    let project = Project {
        domain: format!("{}.test", name),
        name,
        path,
        ssl: false,
    };

    projects.push(project.clone());
    save_projects_to_disk(&projects)?;

    // TODO: Generate Nginx vhost config
    // TODO: Generate SSL certificate via mkcert
    // TODO: Configure dnsmasq entry
    // TODO: Auto-create database

    Ok(project)
}

#[tauri::command]
pub async fn remove_project(name: String) -> Result<(), String> {
    let mut projects = load_projects_from_disk();
    projects.retain(|p| p.name != name);
    save_projects_to_disk(&projects)?;

    // TODO: Remove Nginx vhost config
    // TODO: Remove SSL certificate
    // TODO: Remove dnsmasq entry

    Ok(())
}
