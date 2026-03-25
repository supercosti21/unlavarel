use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::io::AsyncWriteExt;

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadProgress {
    pub downloaded: u64,
    pub total: Option<u64>,
    pub phase: String,  // "downloading", "installing", "done"
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
fn pick_asset(assets: &[GitHubAsset]) -> Option<(String, String)> {
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
        .map(|a| (a.browser_download_url.clone(), a.name.clone()))
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

fn build_client() -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .user_agent("Unlavarel-Updater")
        .build()
        .map_err(|e| format!("HTTP client error: {}", e))
}

#[tauri::command]
pub async fn check_for_updates() -> Result<UpdateInfo, String> {
    let url = format!(
        "https://api.github.com/repos/{}/releases",
        GITHUB_REPO
    );

    let client = build_client()?;

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
    let (download_url, _) = pick_asset(&release.assets)
        .unwrap_or_else(|| (release.html_url.clone(), String::new()));

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

/// Download the update asset and install it automatically.
/// Returns a message telling the user to restart.
#[tauri::command]
pub async fn download_and_install_update(
    app: tauri::AppHandle,
    download_url: String,
) -> Result<String, String> {
    use tauri::Emitter;

    let client = build_client()?;

    // Emit progress: downloading
    let _ = app.emit("update-progress", DownloadProgress {
        downloaded: 0,
        total: None,
        phase: "downloading".into(),
    });

    let response = client
        .get(&download_url)
        .send()
        .await
        .map_err(|e| format!("Download failed: {}", e))?;

    let total_size = response.content_length();
    let file_name = download_url
        .split('/')
        .last()
        .unwrap_or("update")
        .to_string();

    // Download to temp directory
    let tmp_dir = std::env::temp_dir().join("unlavarel-update");
    tokio::fs::create_dir_all(&tmp_dir)
        .await
        .map_err(|e| format!("Failed to create temp dir: {}", e))?;
    let tmp_file = tmp_dir.join(&file_name);

    let mut file = tokio::fs::File::create(&tmp_file)
        .await
        .map_err(|e| format!("Failed to create temp file: {}", e))?;

    let mut downloaded: u64 = 0;
    let mut stream = response.bytes_stream();
    use futures_util::StreamExt;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| format!("Download error: {}", e))?;
        file.write_all(&chunk)
            .await
            .map_err(|e| format!("Write error: {}", e))?;
        downloaded += chunk.len() as u64;

        // Emit progress every ~100KB
        if downloaded % 102400 < chunk.len() as u64 {
            let _ = app.emit("update-progress", DownloadProgress {
                downloaded,
                total: total_size,
                phase: "downloading".into(),
            });
        }
    }

    file.flush().await.map_err(|e| format!("Flush error: {}", e))?;
    drop(file);

    // Emit progress: installing
    let _ = app.emit("update-progress", DownloadProgress {
        downloaded,
        total: total_size,
        phase: "installing".into(),
    });

    // Platform-specific installation
    install_update(&tmp_file).await?;

    // Emit progress: done
    let _ = app.emit("update-progress", DownloadProgress {
        downloaded,
        total: total_size,
        phase: "done".into(),
    });

    Ok("Update installed. Please restart the app to use the new version.".into())
}

/// Platform-specific update installation
async fn install_update(file_path: &PathBuf) -> Result<(), String> {
    let file_str = file_path.to_string_lossy();

    if cfg!(target_os = "macos") {
        install_update_macos(file_path).await
    } else if cfg!(target_os = "windows") {
        install_update_windows(&file_str).await
    } else {
        install_update_linux(file_path).await
    }
}

/// macOS: mount DMG, copy .app to /Applications, unmount
async fn install_update_macos(dmg_path: &PathBuf) -> Result<(), String> {
    use tokio::process::Command;

    // Mount the DMG
    let mount_output = Command::new("hdiutil")
        .args(["attach", &dmg_path.to_string_lossy(), "-nobrowse", "-quiet"])
        .output()
        .await
        .map_err(|e| format!("Failed to mount DMG: {}", e))?;

    if !mount_output.status.success() {
        return Err(format!(
            "Failed to mount DMG: {}",
            String::from_utf8_lossy(&mount_output.stderr)
        ));
    }

    // Find the mount point (parse hdiutil output or use known name)
    let stdout = String::from_utf8_lossy(&mount_output.stdout);
    let mount_point = stdout
        .lines()
        .last()
        .and_then(|line| line.split('\t').last())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "/Volumes/Unlavarel".to_string());

    // Find the .app bundle in the mounted volume
    let app_source = format!("{}/Unlavarel.app", mount_point);
    let app_dest = "/Applications/Unlavarel.app";

    // Remove old app and copy new one
    let cp_result = Command::new("cp")
        .args(["-Rf", &app_source, app_dest])
        .output()
        .await
        .map_err(|e| format!("Failed to copy app: {}", e))?;

    // Always try to unmount, even if copy failed
    let _ = Command::new("hdiutil")
        .args(["detach", &mount_point, "-quiet"])
        .output()
        .await;

    if !cp_result.status.success() {
        return Err(format!(
            "Failed to install update: {}",
            String::from_utf8_lossy(&cp_result.stderr)
        ));
    }

    // Remove quarantine attribute from the new app
    let _ = Command::new("xattr")
        .args(["-rd", "com.apple.quarantine", app_dest])
        .output()
        .await;

    Ok(())
}

/// Windows: launch the MSI installer silently
async fn install_update_windows(msi_path: &str) -> Result<(), String> {
    use tokio::process::Command;

    // Launch MSI with passive mode (progress bar, no interaction)
    Command::new("msiexec")
        .args(["/i", msi_path, "/passive", "/norestart"])
        .spawn()
        .map_err(|e| format!("Failed to launch installer: {}", e))?;

    Ok(())
}

/// Linux: replace current AppImage with downloaded one
async fn install_update_linux(appimage_path: &PathBuf) -> Result<(), String> {
    // Find current executable path
    let current_exe = std::env::current_exe()
        .map_err(|e| format!("Failed to get current executable: {}", e))?;

    // If running as AppImage, APPIMAGE env var has the real path
    let target_path = std::env::var("APPIMAGE")
        .map(PathBuf::from)
        .unwrap_or(current_exe);

    // Copy new AppImage over current one
    tokio::fs::copy(appimage_path, &target_path)
        .await
        .map_err(|e| format!("Failed to replace AppImage: {}", e))?;

    // Make executable
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::Permissions::from_mode(0o755);
        std::fs::set_permissions(&target_path, perms)
            .map_err(|e| format!("Failed to set permissions: {}", e))?;
    }

    Ok(())
}

/// Restart the application
#[tauri::command]
pub async fn restart_app(app: tauri::AppHandle) -> Result<(), String> {
    // Re-launch the app then exit current instance
    let current_exe = std::env::current_exe()
        .map_err(|e| format!("Failed to get executable path: {}", e))?;

    // On Linux, prefer APPIMAGE path
    let exe_path = std::env::var("APPIMAGE")
        .map(PathBuf::from)
        .unwrap_or(current_exe);

    std::process::Command::new(&exe_path)
        .spawn()
        .map_err(|e| format!("Failed to restart: {}", e))?;

    app.exit(0);
    Ok(())
}
