use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShareInfo {
    pub domain: String,
    pub public_url: String,
    pub provider: String,
}

// TODO: Track tunnel processes for clean shutdown
// static TUNNELS: std::sync::LazyLock<Mutex<HashMap<String, tokio::process::Child>>> =
//     std::sync::LazyLock::new(|| Mutex::new(HashMap::new()));

/// Detect which tunnel provider is available.
fn detect_provider() -> Option<&'static str> {
    if which("cloudflared") { return Some("cloudflared"); }
    if which("ngrok") { return Some("ngrok"); }
    None
}

fn which(cmd: &str) -> bool {
    std::process::Command::new("which")
        .arg(cmd)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

#[tauri::command]
pub async fn share_site(domain: String) -> Result<ShareInfo, String> {
    let provider = detect_provider()
        .ok_or("No tunnel provider found. Install ngrok or cloudflared.")?;

    // Determine local port (default 80 for Nginx)
    let local_url = "http://localhost:80".to_string();

    let (_child, public_url) = match provider {
        "cloudflared" => start_cloudflared(&local_url).await?,
        "ngrok" => start_ngrok(&local_url).await?,
        _ => return Err("Unknown provider".into()),
    };

    // Store the child process so we can stop it later
    // Note: we can't store Child in a Mutex easily because it's not Send
    // For now, just let it run and return the URL

    Ok(ShareInfo {
        domain,
        public_url,
        provider: provider.to_string(),
    })
}

async fn start_cloudflared(local_url: &str) -> Result<(Option<tokio::process::Child>, String), String> {
    let mut child = Command::new("cloudflared")
        .args(["tunnel", "--url", local_url])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to start cloudflared: {}", e))?;

    // cloudflared prints the URL to stderr
    let stderr = child.stderr.take().ok_or("No stderr")?;
    let reader = BufReader::new(stderr);
    let mut lines = reader.lines();

    // Wait for the URL to appear (usually within a few seconds)
    let mut url = String::new();
    let _timeout = tokio::time::timeout(std::time::Duration::from_secs(15), async {
        while let Ok(Some(line)) = lines.next_line().await {
            if line.contains("https://") && line.contains(".trycloudflare.com") {
                if let Some(start) = line.find("https://") {
                    url = line[start..].split_whitespace().next().unwrap_or("").to_string();
                    return;
                }
            }
        }
    }).await;

    if url.is_empty() {
        child.kill().await.ok();
        return Err("Cloudflared did not provide a URL within 15 seconds".into());
    }

    Ok((Some(child), url))
}

async fn start_ngrok(local_url: &str) -> Result<(Option<tokio::process::Child>, String), String> {
    let _child = Command::new("ngrok")
        .args(["http", local_url])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to start ngrok: {}", e))?;

    // ngrok exposes its API at localhost:4040
    // Wait a bit for it to start
    tokio::time::sleep(std::time::Duration::from_secs(3)).await;

    let api_response = reqwest_lite("http://localhost:4040/api/tunnels").await;
    let url = match api_response {
        Ok(body) => {
            // Parse the JSON to find the public URL
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&body) {
                json["tunnels"][0]["public_url"]
                    .as_str()
                    .unwrap_or("")
                    .to_string()
            } else {
                String::new()
            }
        }
        Err(_) => String::new(),
    };

    if url.is_empty() {
        return Err("ngrok did not provide a public URL. Is it authenticated?".into());
    }

    Ok((None, url))
}

/// Minimal HTTP GET without reqwest dependency.
async fn reqwest_lite(url: &str) -> Result<String, String> {
    let output = Command::new("curl")
        .args(["-s", url])
        .output()
        .await
        .map_err(|e| e.to_string())?;

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

#[tauri::command]
pub async fn stop_sharing(_domain: String) -> Result<(), String> {
    // Kill any tunnel processes
    // For cloudflared/ngrok, we'd need to track PIDs
    // Simple approach: kill by name
    if cfg!(unix) {
        let _ = Command::new("pkill").arg("-f").arg("cloudflared tunnel").output().await;
        let _ = Command::new("pkill").arg("-f").arg("ngrok http").output().await;
    }
    Ok(())
}

#[tauri::command]
pub async fn get_sharing_providers() -> Result<Vec<String>, String> {
    let mut providers = Vec::new();
    if which("cloudflared") { providers.push("cloudflared".into()); }
    if which("ngrok") { providers.push("ngrok".into()); }
    Ok(providers)
}
