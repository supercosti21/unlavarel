use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tauri::Emitter;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

/// Active log watchers — one per service.
type Watchers = Arc<Mutex<HashMap<String, tokio::task::JoinHandle<()>>>>;

static WATCHERS: std::sync::LazyLock<Watchers> =
    std::sync::LazyLock::new(|| Arc::new(Mutex::new(HashMap::new())));

/// Start streaming logs for a service via Tauri events.
#[tauri::command]
pub async fn watch_service_logs(
    app: tauri::AppHandle,
    service: String,
) -> Result<(), String> {
    // Stop existing watcher if any
    stop_watch_inner(&service);

    let svc = service.clone();
    let handle = tokio::spawn(async move {
        let _ = stream_logs(app, &svc).await;
    });

    WATCHERS.lock().unwrap().insert(service, handle);
    Ok(())
}

/// Stop streaming logs for a service.
#[tauri::command]
pub async fn unwatch_service_logs(service: String) -> Result<(), String> {
    stop_watch_inner(&service);
    Ok(())
}

fn stop_watch_inner(service: &str) {
    if let Some(handle) = WATCHERS.lock().unwrap().remove(service) {
        handle.abort();
    }
}

async fn stream_logs(app: tauri::AppHandle, service: &str) -> Result<(), Box<dyn std::error::Error>> {
    if cfg!(target_os = "macos") {
        stream_logs_macos(app, service).await
    } else if cfg!(target_os = "linux") {
        stream_logs_linux(app, service).await
    } else {
        Ok(()) // Windows: not yet
    }
}

async fn stream_logs_macos(app: tauri::AppHandle, service: &str) -> Result<(), Box<dyn std::error::Error>> {
    let brew = if cfg!(target_arch = "aarch64") {
        "/opt/homebrew"
    } else {
        "/usr/local"
    };
    let log_path = format!("{}/var/log/{}.log", brew, service);

    // Use tail -f to follow the log
    let mut child = Command::new("tail")
        .args(["-f", "-n", "50", &log_path])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()?;

    if let Some(stdout) = child.stdout.take() {
        let reader = BufReader::new(stdout);
        let mut lines = reader.lines();
        let event_name = format!("log:{}", service);

        while let Ok(Some(line)) = lines.next_line().await {
            if app.emit(&event_name, &line).is_err() {
                break; // Window closed
            }
        }
    }

    child.kill().await.ok();
    Ok(())
}

async fn stream_logs_linux(app: tauri::AppHandle, service: &str) -> Result<(), Box<dyn std::error::Error>> {
    let unit = if service.starts_with("php") {
        "php-fpm"
    } else {
        match service {
            "mysql" => "mysqld",
            "mariadb" => "mariadb",
            "nginx" => "nginx",
            "redis" => "redis",
            "memcached" => "memcached",
            "dnsmasq" => "dnsmasq",
            "mailpit" => "mailpit",
            other => other,
        }
    };

    let mut child = Command::new("journalctl")
        .args(["-u", unit, "-f", "-n", "50", "--no-pager", "-o", "short-iso"])
        .stdout(std::process::Stdio::piped())
        .spawn()?;

    if let Some(stdout) = child.stdout.take() {
        let reader = BufReader::new(stdout);
        let mut lines = reader.lines();
        let event_name = format!("log:{}", service);

        while let Ok(Some(line)) = lines.next_line().await {
            if app.emit(&event_name, &line).is_err() {
                break;
            }
        }
    }

    child.kill().await.ok();
    Ok(())
}
