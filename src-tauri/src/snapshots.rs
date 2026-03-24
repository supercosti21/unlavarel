use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub id: String,
    pub project_name: String,
    pub created_at: String,
    pub files_path: String,
    pub db_dump_path: Option<String>,
    pub size_bytes: u64,
}

fn snapshots_dir() -> PathBuf {
    let dir = dirs_next::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("unlavarel")
        .join("snapshots");
    std::fs::create_dir_all(&dir).ok();
    dir
}

#[tauri::command]
pub async fn create_snapshot(
    project_name: String,
    project_path: String,
    database: Option<String>,
) -> Result<Snapshot, String> {
    let timestamp = chrono_now();
    let id = format!("{}_{}", project_name, timestamp);
    let snap_dir = snapshots_dir().join(&id);
    tokio::fs::create_dir_all(&snap_dir)
        .await
        .map_err(|e| e.to_string())?;

    // 1. Archive project files
    let files_archive = snap_dir.join("files.tar.gz");
    let tar_output = Command::new("tar")
        .args([
            "-czf",
            files_archive.to_str().unwrap_or(""),
            "-C",
            &project_path,
            ".",
        ])
        .output()
        .await
        .map_err(|e| format!("Failed to archive files: {}", e))?;

    if !tar_output.status.success() {
        return Err(format!(
            "tar failed: {}",
            String::from_utf8_lossy(&tar_output.stderr)
        ));
    }

    // 2. Dump database if specified
    let db_dump_path = if let Some(db_name) = &database {
        let dump_file = snap_dir.join("database.sql");
        let dumped = dump_database(db_name, &dump_file).await;
        if dumped {
            Some(dump_file.to_string_lossy().to_string())
        } else {
            None
        }
    } else {
        None
    };

    // 3. Calculate size
    let size = dir_size(&snap_dir).await;

    // 4. Save metadata
    let snapshot = Snapshot {
        id: id.clone(),
        project_name,
        created_at: timestamp,
        files_path: files_archive.to_string_lossy().to_string(),
        db_dump_path,
        size_bytes: size,
    };

    let meta_path = snap_dir.join("snapshot.json");
    let meta_json = serde_json::to_string_pretty(&snapshot).map_err(|e| e.to_string())?;
    tokio::fs::write(&meta_path, meta_json)
        .await
        .map_err(|e| e.to_string())?;

    Ok(snapshot)
}

#[tauri::command]
pub async fn list_snapshots(project_name: Option<String>) -> Result<Vec<Snapshot>, String> {
    let dir = snapshots_dir();
    let mut snapshots = Vec::new();

    let mut entries = tokio::fs::read_dir(&dir)
        .await
        .map_err(|e| e.to_string())?;

    while let Ok(Some(entry)) = entries.next_entry().await {
        let meta_path = entry.path().join("snapshot.json");
        if meta_path.exists() {
            if let Ok(content) = tokio::fs::read_to_string(&meta_path).await {
                if let Ok(snap) = serde_json::from_str::<Snapshot>(&content) {
                    if let Some(ref filter) = project_name {
                        if &snap.project_name == filter {
                            snapshots.push(snap);
                        }
                    } else {
                        snapshots.push(snap);
                    }
                }
            }
        }
    }

    snapshots.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    Ok(snapshots)
}

#[tauri::command]
pub async fn restore_snapshot(
    snapshot_id: String,
    target_path: String,
    restore_db: bool,
) -> Result<String, String> {
    let snap_dir = snapshots_dir().join(&snapshot_id);
    let meta_path = snap_dir.join("snapshot.json");

    let content = tokio::fs::read_to_string(&meta_path)
        .await
        .map_err(|e| format!("Snapshot not found: {}", e))?;
    let snapshot: Snapshot = serde_json::from_str(&content).map_err(|e| e.to_string())?;

    // 1. Restore files
    let files_archive = PathBuf::from(&snapshot.files_path);
    if files_archive.exists() {
        tokio::fs::create_dir_all(&target_path)
            .await
            .map_err(|e| e.to_string())?;

        let tar = Command::new("tar")
            .args(["-xzf", files_archive.to_str().unwrap_or(""), "-C", &target_path])
            .output()
            .await
            .map_err(|e| format!("Failed to extract: {}", e))?;

        if !tar.status.success() {
            return Err(format!(
                "Extract failed: {}",
                String::from_utf8_lossy(&tar.stderr)
            ));
        }
    }

    // 2. Restore database if requested
    if restore_db {
        if let Some(dump_path) = &snapshot.db_dump_path {
            let dump = PathBuf::from(dump_path);
            if dump.exists() {
                restore_database(&snapshot.project_name, &dump).await;
            }
        }
    }

    Ok(format!("Snapshot {} restored to {}", snapshot_id, target_path))
}

#[tauri::command]
pub async fn delete_snapshot(snapshot_id: String) -> Result<(), String> {
    let snap_dir = snapshots_dir().join(&snapshot_id);
    if snap_dir.exists() {
        tokio::fs::remove_dir_all(&snap_dir)
            .await
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

async fn dump_database(db_name: &str, output_path: &PathBuf) -> bool {
    // Try mysqldump first
    let mysql = Command::new("mysqldump")
        .args(["-u", "root", db_name])
        .stdout(std::process::Stdio::piped())
        .output()
        .await;

    if let Ok(o) = mysql {
        if o.status.success() {
            return tokio::fs::write(output_path, &o.stdout).await.is_ok();
        }
    }

    // Try mariadb-dump
    let maria = Command::new("mariadb-dump")
        .args(["-u", "root", db_name])
        .stdout(std::process::Stdio::piped())
        .output()
        .await;

    if let Ok(o) = maria {
        if o.status.success() {
            return tokio::fs::write(output_path, &o.stdout).await.is_ok();
        }
    }

    // Try pg_dump
    let pg = Command::new("pg_dump")
        .arg(db_name)
        .stdout(std::process::Stdio::piped())
        .output()
        .await;

    if let Ok(o) = pg {
        if o.status.success() {
            return tokio::fs::write(output_path, &o.stdout).await.is_ok();
        }
    }

    false
}

async fn restore_database(db_name: &str, dump_path: &PathBuf) {
    let dump = tokio::fs::read_to_string(dump_path).await;
    if let Ok(sql) = dump {
        // Try mysql
        let mut child = Command::new("mysql")
            .args(["-u", "root", db_name])
            .stdin(std::process::Stdio::piped())
            .spawn();

        if let Ok(ref mut c) = child {
            if let Some(stdin) = c.stdin.as_mut() {
                use tokio::io::AsyncWriteExt;
                let _ = stdin.write_all(sql.as_bytes()).await;
                let _ = stdin.shutdown().await;
            }
            let _ = c.wait().await;
            return;
        }

        // Try psql
        let mut child = Command::new("psql")
            .args(["-U", "postgres", "-d", db_name])
            .stdin(std::process::Stdio::piped())
            .spawn();

        if let Ok(ref mut c) = child {
            if let Some(stdin) = c.stdin.as_mut() {
                use tokio::io::AsyncWriteExt;
                let _ = stdin.write_all(sql.as_bytes()).await;
                let _ = stdin.shutdown().await;
            }
            let _ = c.wait().await;
        }
    }
}

fn chrono_now() -> String {
    // Simple timestamp without chrono dependency
    let output = std::process::Command::new("date")
        .arg("+%Y%m%d_%H%M%S")
        .output();

    match output {
        Ok(o) if o.status.success() => {
            String::from_utf8_lossy(&o.stdout).trim().to_string()
        }
        _ => "unknown".to_string(),
    }
}

async fn dir_size(path: &std::path::Path) -> u64 {
    let output = Command::new("du")
        .args(["-sb", path.to_str().unwrap_or(".")])
        .output()
        .await;

    match output {
        Ok(o) if o.status.success() => {
            String::from_utf8_lossy(&o.stdout)
                .split_whitespace()
                .next()
                .and_then(|s| s.parse().ok())
                .unwrap_or(0)
        }
        _ => 0,
    }
}
