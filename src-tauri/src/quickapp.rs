use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuickAppTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub command: String,
}

/// Available quick app templates.
pub fn available_templates() -> Vec<QuickAppTemplate> {
    vec![
        QuickAppTemplate {
            id: "laravel".into(),
            name: "Laravel".into(),
            description: "Full-stack PHP framework with Eloquent, Blade, and Artisan".into(),
            command: "composer create-project laravel/laravel".into(),
        },
        QuickAppTemplate {
            id: "laravel-filament".into(),
            name: "Laravel + Filament".into(),
            description: "Laravel with Filament admin panel pre-installed".into(),
            command: "composer create-project laravel/laravel".into(),
            // Filament is added as a post-install step
        },
        QuickAppTemplate {
            id: "symfony".into(),
            name: "Symfony".into(),
            description: "Symfony web application skeleton".into(),
            command: "composer create-project symfony/skeleton".into(),
        },
        QuickAppTemplate {
            id: "wordpress".into(),
            name: "WordPress".into(),
            description: "WordPress CMS".into(),
            command: "wp core download".into(), // requires wp-cli
        },
        QuickAppTemplate {
            id: "blank".into(),
            name: "Blank PHP".into(),
            description: "Empty project with index.php".into(),
            command: "".into(), // handled manually
        },
    ]
}

#[tauri::command]
pub async fn get_templates() -> Result<Vec<QuickAppTemplate>, String> {
    Ok(available_templates())
}

#[tauri::command]
pub async fn create_app(
    template_id: String,
    name: String,
    parent_dir: String,
) -> Result<String, String> {
    let project_path = PathBuf::from(&parent_dir).join(&name);

    if project_path.exists() {
        return Err(format!(
            "Directory {} already exists",
            project_path.display()
        ));
    }

    match template_id.as_str() {
        "laravel" => create_laravel(&name, &parent_dir, false).await,
        "laravel-filament" => create_laravel(&name, &parent_dir, true).await,
        "symfony" => create_symfony(&name, &parent_dir).await,
        "wordpress" => create_wordpress(&name, &parent_dir).await,
        "blank" => create_blank(&name, &parent_dir).await,
        _ => Err(format!("Unknown template: {}", template_id)),
    }
}

async fn create_laravel(
    name: &str,
    parent_dir: &str,
    with_filament: bool,
) -> Result<String, String> {
    let output = Command::new("composer")
        .args(["create-project", "laravel/laravel", name])
        .current_dir(parent_dir)
        .output()
        .await
        .map_err(|e| format!("Failed to run composer: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "Laravel creation failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let project_path = PathBuf::from(parent_dir).join(name);

    // Install Filament if requested
    if with_filament {
        let filament = Command::new("composer")
            .args(["require", "filament/filament"])
            .current_dir(&project_path)
            .output()
            .await
            .map_err(|e| format!("Failed to install Filament: {}", e))?;

        if !filament.status.success() {
            eprintln!(
                "Filament install warning: {}",
                String::from_utf8_lossy(&filament.stderr)
            );
        }

        // Run Filament install command
        let install = Command::new("php")
            .args(["artisan", "filament:install", "--panels"])
            .current_dir(&project_path)
            .output()
            .await;

        if let Ok(o) = install {
            if !o.status.success() {
                eprintln!(
                    "Filament setup warning: {}",
                    String::from_utf8_lossy(&o.stderr)
                );
            }
        }
    }

    // Generate app key
    let _ = Command::new("php")
        .args(["artisan", "key:generate"])
        .current_dir(&project_path)
        .output()
        .await;

    Ok(project_path.to_string_lossy().to_string())
}

async fn create_symfony(name: &str, parent_dir: &str) -> Result<String, String> {
    let output = Command::new("composer")
        .args(["create-project", "symfony/skeleton", name])
        .current_dir(parent_dir)
        .output()
        .await
        .map_err(|e| format!("Failed to run composer: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "Symfony creation failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let project_path = PathBuf::from(parent_dir).join(name);
    Ok(project_path.to_string_lossy().to_string())
}

async fn create_wordpress(name: &str, parent_dir: &str) -> Result<String, String> {
    let project_path = PathBuf::from(parent_dir).join(name);
    tokio::fs::create_dir_all(&project_path)
        .await
        .map_err(|e| e.to_string())?;

    // Try wp-cli first
    let wp = Command::new("wp")
        .args(["core", "download"])
        .current_dir(&project_path)
        .output()
        .await;

    match wp {
        Ok(o) if o.status.success() => {
            return Ok(project_path.to_string_lossy().to_string());
        }
        _ => {}
    }

    // Fallback: download WordPress via curl
    let download = Command::new("curl")
        .args([
            "-L",
            "-o",
            "/tmp/wordpress-latest.tar.gz",
            "https://wordpress.org/latest.tar.gz",
        ])
        .output()
        .await
        .map_err(|e| format!("Failed to download WordPress: {}", e))?;

    if !download.status.success() {
        return Err("Failed to download WordPress".into());
    }

    let extract = Command::new("tar")
        .args([
            "-xzf",
            "/tmp/wordpress-latest.tar.gz",
            "--strip-components=1",
            "-C",
        ])
        .arg(&project_path)
        .output()
        .await
        .map_err(|e| format!("Failed to extract WordPress: {}", e))?;

    if !extract.status.success() {
        return Err("Failed to extract WordPress".into());
    }

    Ok(project_path.to_string_lossy().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_available_templates_count() {
        let templates = available_templates();
        assert!(templates.len() >= 4);
    }

    #[test]
    fn test_template_ids_unique() {
        let templates = available_templates();
        let ids: Vec<&str> = templates.iter().map(|t| t.id.as_str()).collect();
        let mut unique = ids.clone();
        unique.sort();
        unique.dedup();
        assert_eq!(ids.len(), unique.len());
    }

    #[test]
    fn test_laravel_template_exists() {
        let templates = available_templates();
        let laravel = templates.iter().find(|t| t.id == "laravel");
        assert!(laravel.is_some());
        assert!(laravel.unwrap().command.contains("composer"));
    }

    #[test]
    fn test_blank_template_exists() {
        let templates = available_templates();
        let blank = templates.iter().find(|t| t.id == "blank");
        assert!(blank.is_some());
        assert!(blank.unwrap().command.is_empty());
    }

    #[test]
    fn test_template_serialization() {
        let templates = available_templates();
        let json = serde_json::to_string(&templates).unwrap();
        let parsed: Vec<QuickAppTemplate> = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.len(), templates.len());
    }
}

async fn create_blank(name: &str, parent_dir: &str) -> Result<String, String> {
    let project_path = PathBuf::from(parent_dir).join(name);
    let public_dir = project_path.join("public");

    tokio::fs::create_dir_all(&public_dir)
        .await
        .map_err(|e| e.to_string())?;

    let index_content = r#"<?php

phpinfo();
"#;

    tokio::fs::write(public_dir.join("index.php"), index_content)
        .await
        .map_err(|e| e.to_string())?;

    Ok(project_path.to_string_lossy().to_string())
}
