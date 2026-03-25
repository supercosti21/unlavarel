use serde::{Deserialize, Serialize};

const GITHUB_REPO: &str = "supercosti21/unlavarel";
const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateInfo {
    pub current_version: String,
    pub latest_version: String,
    pub update_available: bool,
    pub release_notes: String,
    pub download_url: String,
    pub published_at: String,
}

#[derive(Debug, Deserialize)]
struct GitHubRelease {
    tag_name: String,
    body: Option<String>,
    html_url: String,
    published_at: Option<String>,
    prerelease: bool,
    assets: Vec<GitHubAsset>,
}

#[derive(Debug, Deserialize)]
struct GitHubAsset {
    name: String,
    browser_download_url: String,
}

/// Pick the right asset for the current platform
fn pick_asset(assets: &[GitHubAsset]) -> Option<String> {
    let ext = if cfg!(target_os = "macos") {
        ".dmg"
    } else if cfg!(target_os = "windows") {
        ".msi"
    } else {
        ".AppImage"
    };

    assets
        .iter()
        .find(|a| a.name.ends_with(ext))
        .map(|a| a.browser_download_url.clone())
}

/// Compare semver-ish version strings (e.g. "0.1.0" vs "0.2.0")
fn is_newer(latest: &str, current: &str) -> bool {
    let parse = |v: &str| -> Vec<u64> {
        v.trim_start_matches('v')
            .split('.')
            .filter_map(|s| s.parse().ok())
            .collect()
    };
    let l = parse(latest);
    let c = parse(current);
    l > c
}

#[tauri::command]
pub async fn check_for_updates() -> Result<UpdateInfo, String> {
    let url = format!(
        "https://api.github.com/repos/{}/releases",
        GITHUB_REPO
    );

    let client = reqwest::Client::builder()
        .user_agent("Unlavarel-Updater")
        .build()
        .map_err(|e| format!("HTTP client error: {}", e))?;

    let releases: Vec<GitHubRelease> = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Failed to check for updates: {}", e))?
        .json()
        .await
        .map_err(|e| format!("Failed to parse release data: {}", e))?;

    // Find latest non-prerelease with a real version tag, fall back to any release
    let release = releases
        .iter()
        .find(|r| !r.prerelease && r.tag_name.starts_with('v'))
        .or_else(|| releases.first())
        .ok_or("No releases found")?;

    let latest = release.tag_name.trim_start_matches('v').to_string();
    let download_url = pick_asset(&release.assets)
        .unwrap_or_else(|| release.html_url.clone());

    Ok(UpdateInfo {
        current_version: CURRENT_VERSION.to_string(),
        latest_version: latest.clone(),
        update_available: is_newer(&latest, CURRENT_VERSION),
        release_notes: release.body.clone().unwrap_or_default(),
        download_url,
        published_at: release.published_at.clone().unwrap_or_default(),
    })
}

#[tauri::command]
pub async fn get_current_version() -> Result<String, String> {
    Ok(CURRENT_VERSION.to_string())
}
