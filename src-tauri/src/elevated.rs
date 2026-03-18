use std::sync::Mutex;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

/// Session-cached sudo password.
/// Stored in memory only — never written to disk. Cleared on app exit.
static CACHED_PASSWORD: std::sync::LazyLock<Mutex<Option<String>>> =
    std::sync::LazyLock::new(|| Mutex::new(None));

static PASSWORD_CACHED: std::sync::LazyLock<Mutex<bool>> =
    std::sync::LazyLock::new(|| Mutex::new(false));

/// Check if we have a cached password
pub fn has_cached_password() -> bool {
    *PASSWORD_CACHED.lock().unwrap()
}

/// Store password for the session
fn cache_password(password: &str) {
    *CACHED_PASSWORD.lock().unwrap() = Some(password.to_string());
    *PASSWORD_CACHED.lock().unwrap() = true;
}

/// Clear cached password
pub fn clear_password() {
    *CACHED_PASSWORD.lock().unwrap() = None;
    *PASSWORD_CACHED.lock().unwrap() = false;
}

/// Get the cached password
fn get_password() -> Option<String> {
    CACHED_PASSWORD.lock().unwrap().clone()
}

/// Run a command with elevated privileges.
/// Uses cached password if available, otherwise falls back to pkexec.
pub async fn run_elevated(program: &str, args: &[&str]) -> Result<std::process::Output, String> {
    // If on macOS, Homebrew doesn't need sudo for most things
    if cfg!(target_os = "macos") && program != "sudo" {
        return run_direct(program, args).await;
    }

    // Try with cached password first
    if let Some(password) = get_password() {
        match run_with_sudo(&password, program, args).await {
            Ok(output) if output.status.success() => return Ok(output),
            Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr);
                // If auth failed, clear the cache
                if stderr.contains("incorrect password") || stderr.contains("Sorry") {
                    clear_password();
                } else {
                    // Command failed for other reasons, still return it
                    return Ok(output);
                }
            }
            Err(_) => {
                clear_password();
            }
        }
    }

    // No cached password — fall back to pkexec (graphical prompt)
    let output = Command::new("pkexec")
        .arg(program)
        .args(args)
        .output()
        .await
        .map_err(|e| format!("Failed to run pkexec: {}", e))?;

    Ok(output)
}

/// Run a command with sudo, piping the password via stdin
async fn run_with_sudo(password: &str, program: &str, args: &[&str]) -> Result<std::process::Output, String> {
    let mut child = Command::new("sudo")
        .arg("-S") // read password from stdin
        .arg("-k") // don't use cached credentials (we manage our own)
        .arg("--")
        .arg(program)
        .args(args)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn sudo: {}", e))?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(format!("{}\n", password).as_bytes())
            .await
            .map_err(|e| e.to_string())?;
        stdin.shutdown().await.ok();
    }

    child.wait_with_output().await.map_err(|e| e.to_string())
}

/// Run a command directly without elevation
async fn run_direct(program: &str, args: &[&str]) -> Result<std::process::Output, String> {
    Command::new(program)
        .args(args)
        .output()
        .await
        .map_err(|e| format!("Failed to run {}: {}", program, e))
}

/// Run a bash script with elevation (single call for multiple commands)
pub async fn run_script_elevated(script: &str) -> Result<std::process::Output, String> {
    // Try with cached password
    if let Some(password) = get_password() {
        match run_with_sudo(&password, "bash", &["-c", script]).await {
            Ok(output) if output.status.success() => return Ok(output),
            Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr);
                if stderr.contains("incorrect password") || stderr.contains("Sorry") {
                    clear_password();
                } else {
                    return Ok(output);
                }
            }
            Err(_) => { clear_password(); }
        }
    }

    // Write script to temp file and run with pkexec
    let script_path = "/tmp/macenv_elevated.sh";
    tokio::fs::write(script_path, script)
        .await
        .map_err(|e| e.to_string())?;

    Command::new("chmod")
        .args(["+x", script_path])
        .output()
        .await
        .ok();

    let output = Command::new("pkexec")
        .args(["bash", script_path])
        .output()
        .await
        .map_err(|e| format!("pkexec failed: {}", e))?;

    tokio::fs::remove_file(script_path).await.ok();
    Ok(output)
}

// --- Tauri commands ---

#[tauri::command]
pub async fn save_session_password(password: String) -> Result<(), String> {
    // Verify password is correct first
    let output = run_with_sudo(&password, "true", &[]).await?;
    if output.status.success() {
        cache_password(&password);
        Ok(())
    } else {
        Err("Incorrect password".into())
    }
}

#[tauri::command]
pub async fn has_session_password() -> Result<bool, String> {
    Ok(has_cached_password())
}

#[tauri::command]
pub async fn clear_session_password() -> Result<(), String> {
    clear_password();
    Ok(())
}
