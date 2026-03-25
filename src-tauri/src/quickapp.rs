use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuickAppTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub command: String,
    #[serde(default)]
    pub user_defined: bool,
}

/// Built-in templates + user-defined from custom_templates.json
pub fn available_templates() -> Vec<QuickAppTemplate> {
    let mut templates = vec![
        QuickAppTemplate {
            id: "laravel".into(),
            name: "Laravel".into(),
            description: "Full-stack PHP framework with Eloquent, Blade, and Artisan".into(),
            command: "composer create-project laravel/laravel".into(),
            user_defined: false,
        },
        QuickAppTemplate {
            id: "laravel-filament".into(),
            name: "Laravel + Filament".into(),
            description: "Laravel with Filament admin panel pre-installed".into(),
            command: "composer create-project laravel/laravel".into(),
            user_defined: false,
        },
        QuickAppTemplate {
            id: "symfony".into(),
            name: "Symfony".into(),
            description: "Symfony web application skeleton".into(),
            command: "composer create-project symfony/skeleton".into(),
            user_defined: false,
        },
        QuickAppTemplate {
            id: "wordpress".into(),
            name: "WordPress".into(),
            description: "WordPress CMS".into(),
            command: "wp core download".into(),
            user_defined: false,
        },
        QuickAppTemplate {
            id: "drupal".into(),
            name: "Drupal".into(),
            description: "Drupal CMS via Composer".into(),
            command: "composer create-project drupal/recommended-project".into(),
            user_defined: false,
        },
        QuickAppTemplate {
            id: "joomla".into(),
            name: "Joomla".into(),
            description: "Joomla CMS".into(),
            command: "".into(),
            user_defined: false,
        },
        QuickAppTemplate {
            id: "prestashop".into(),
            name: "PrestaShop".into(),
            description: "PrestaShop e-commerce platform".into(),
            command: "".into(),
            user_defined: false,
        },
        QuickAppTemplate {
            id: "cakephp".into(),
            name: "CakePHP".into(),
            description: "CakePHP rapid development framework".into(),
            command: "composer create-project cakephp/app".into(),
            user_defined: false,
        },
        QuickAppTemplate {
            id: "lumen".into(),
            name: "Lumen".into(),
            description: "Laravel's micro-framework for APIs".into(),
            command: "composer create-project laravel/lumen".into(),
            user_defined: false,
        },
        QuickAppTemplate {
            id: "blank".into(),
            name: "Blank PHP".into(),
            description: "Empty project with index.php".into(),
            command: "".into(),
            user_defined: false,
        },
    ];

    if let Ok(user_templates) = load_user_templates() {
        templates.extend(user_templates);
    }

    templates
}

fn user_templates_file() -> PathBuf {
    let dir = dirs_next::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("unlavarel");
    std::fs::create_dir_all(&dir).ok();
    dir.join("custom_templates.json")
}

fn load_user_templates() -> Result<Vec<QuickAppTemplate>, String> {
    let path = user_templates_file();
    if !path.exists() { return Ok(vec![]); }
    let content = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    serde_json::from_str(&content).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_templates() -> Result<Vec<QuickAppTemplate>, String> {
    Ok(available_templates())
}

#[tauri::command]
pub async fn save_custom_template(template: QuickAppTemplate) -> Result<(), String> {
    let mut templates = load_user_templates().unwrap_or_default();
    templates.retain(|t| t.id != template.id);
    let mut t = template;
    t.user_defined = true;
    templates.push(t);
    let json = serde_json::to_string_pretty(&templates).map_err(|e| e.to_string())?;
    std::fs::write(user_templates_file(), json).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn delete_custom_template(template_id: String) -> Result<(), String> {
    let mut templates = load_user_templates().unwrap_or_default();
    templates.retain(|t| t.id != template_id);
    let json = serde_json::to_string_pretty(&templates).map_err(|e| e.to_string())?;
    std::fs::write(user_templates_file(), json).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn create_app(
    template_id: String,
    name: String,
    parent_dir: String,
) -> Result<String, String> {
    let project_path = PathBuf::from(&parent_dir).join(&name);
    if project_path.exists() {
        return Err(format!("Directory {} already exists", project_path.display()));
    }

    let enriched_path = crate::setup::build_enriched_path();

    match template_id.as_str() {
        "laravel" => create_laravel(&name, &parent_dir, false, &enriched_path).await,
        "laravel-filament" => create_laravel(&name, &parent_dir, true, &enriched_path).await,
        "symfony" => create_composer_project(&name, &parent_dir, "symfony/skeleton", &enriched_path).await,
        "wordpress" => create_wordpress(&name, &parent_dir, &enriched_path).await,
        "drupal" => create_composer_project(&name, &parent_dir, "drupal/recommended-project", &enriched_path).await,
        "joomla" => create_download_project(&name, &parent_dir, "https://downloads.joomla.org/cms/joomla5/latest/Joomla_latest-Stable-Full_Package.tar.gz", &enriched_path).await,
        "prestashop" => create_download_project(&name, &parent_dir, "https://github.com/PrestaShop/PrestaShop/releases/latest/download/prestashop_latest.zip", &enriched_path).await,
        "cakephp" => create_composer_project(&name, &parent_dir, "cakephp/app", &enriched_path).await,
        "lumen" => create_composer_project(&name, &parent_dir, "laravel/lumen", &enriched_path).await,
        "blank" => create_blank(&name, &parent_dir).await,
        _ => {
            let templates = available_templates();
            if let Some(tmpl) = templates.iter().find(|t| t.id == template_id) {
                if tmpl.command.starts_with("composer create-project") {
                    let pkg = tmpl.command.replace("composer create-project ", "");
                    create_composer_project(&name, &parent_dir, &pkg, &enriched_path).await
                } else if tmpl.command.starts_with("http") {
                    create_download_project(&name, &parent_dir, &tmpl.command, &enriched_path).await
                } else if !tmpl.command.is_empty() {
                    let output = Command::new("bash")
                        .args(["-c", &format!("{} {}", tmpl.command, name)])
                        .current_dir(&parent_dir)
                        .env("PATH", &enriched_path)
                        .output()
                        .await
                        .map_err(|e| format!("Command failed: {}", e))?;
                    if output.status.success() {
                        Ok(PathBuf::from(&parent_dir).join(&name).to_string_lossy().to_string())
                    } else {
                        Err(String::from_utf8_lossy(&output.stderr).to_string())
                    }
                } else {
                    create_blank(&name, &parent_dir).await
                }
            } else {
                Err(format!("Unknown template: {}", template_id))
            }
        }
    }
}

async fn create_laravel(name: &str, parent_dir: &str, with_filament: bool, path_env: &str) -> Result<String, String> {
    let output = Command::new("composer")
        .args(["create-project", "laravel/laravel", name])
        .current_dir(parent_dir)
        .env("PATH", path_env)
        .output()
        .await
        .map_err(|e| format!("Failed to run composer: {}", e))?;

    if !output.status.success() {
        return Err(format!("Laravel creation failed: {}", String::from_utf8_lossy(&output.stderr)));
    }

    let project_path = PathBuf::from(parent_dir).join(name);

    if with_filament {
        let filament = Command::new("composer")
            .args(["require", "filament/filament"])
            .current_dir(&project_path)
            .env("PATH", path_env)
            .output().await;
        if let Ok(o) = &filament { if !o.status.success() { eprintln!("Filament warning: {}", String::from_utf8_lossy(&o.stderr)); } }

        let _ = Command::new("php")
            .args(["artisan", "filament:install", "--panels"])
            .current_dir(&project_path)
            .env("PATH", path_env)
            .output().await;
    }

    let _ = Command::new("php")
        .args(["artisan", "key:generate"])
        .current_dir(&project_path)
        .env("PATH", path_env)
        .output().await;

    Ok(project_path.to_string_lossy().to_string())
}

async fn create_composer_project(name: &str, parent_dir: &str, package: &str, path_env: &str) -> Result<String, String> {
    let output = Command::new("composer")
        .args(["create-project", package, name])
        .current_dir(parent_dir)
        .env("PATH", path_env)
        .output()
        .await
        .map_err(|e| format!("Failed to run composer: {}", e))?;

    if !output.status.success() {
        return Err(format!("Project creation failed: {}", String::from_utf8_lossy(&output.stderr)));
    }

    Ok(PathBuf::from(parent_dir).join(name).to_string_lossy().to_string())
}

async fn create_download_project(name: &str, parent_dir: &str, url: &str, path_env: &str) -> Result<String, String> {
    let project_path = PathBuf::from(parent_dir).join(name);
    tokio::fs::create_dir_all(&project_path).await.map_err(|e| e.to_string())?;

    let tmp_file = format!("/tmp/unlavarel-{}.download", name);

    let download = Command::new("curl")
        .args(["-L", "-o", &tmp_file, url])
        .env("PATH", path_env)
        .output().await
        .map_err(|e| format!("Download failed: {}", e))?;

    if !download.status.success() {
        return Err("Download failed".into());
    }

    if url.ends_with(".zip") {
        Command::new("unzip").args(["-o", &tmp_file, "-d"]).arg(&project_path)
            .env("PATH", path_env).output().await.map_err(|e| format!("Extract failed: {}", e))?;
    } else {
        Command::new("tar").args(["-xzf", &tmp_file, "--strip-components=1", "-C"]).arg(&project_path)
            .env("PATH", path_env).output().await.map_err(|e| format!("Extract failed: {}", e))?;
    }

    let _ = tokio::fs::remove_file(&tmp_file).await;
    Ok(project_path.to_string_lossy().to_string())
}

async fn create_wordpress(name: &str, parent_dir: &str, path_env: &str) -> Result<String, String> {
    let project_path = PathBuf::from(parent_dir).join(name);
    tokio::fs::create_dir_all(&project_path).await.map_err(|e| e.to_string())?;

    let wp = Command::new("wp").args(["core", "download"]).current_dir(&project_path)
        .env("PATH", path_env).output().await;

    match wp {
        Ok(o) if o.status.success() => return Ok(project_path.to_string_lossy().to_string()),
        _ => {}
    }

    create_download_project(name, parent_dir, "https://wordpress.org/latest.tar.gz", path_env).await
}

async fn create_blank(name: &str, parent_dir: &str) -> Result<String, String> {
    let project_path = PathBuf::from(parent_dir).join(name);
    let public_dir = project_path.join("public");
    tokio::fs::create_dir_all(&public_dir).await.map_err(|e| e.to_string())?;
    tokio::fs::write(public_dir.join("index.php"), "<?php\n\nphpinfo();\n").await.map_err(|e| e.to_string())?;
    Ok(project_path.to_string_lossy().to_string())
}
