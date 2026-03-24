use std::path::PathBuf;
use tokio::process::Command;

use crate::error::{MacEnvError, Result};

/// Get the directory where MacEnv stores SSL certificates.
pub fn certs_dir() -> PathBuf {
    let data_dir = dirs_next::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("unlavarel")
        .join("certs");
    std::fs::create_dir_all(&data_dir).ok();
    data_dir
}

/// Find the mkcert binary path.
fn mkcert_bin() -> &'static str {
    if cfg!(target_os = "macos") {
        if cfg!(target_arch = "aarch64") {
            // Try Homebrew path first
            if PathBuf::from("/opt/homebrew/bin/mkcert").exists() {
                return "/opt/homebrew/bin/mkcert";
            }
        } else if PathBuf::from("/usr/local/bin/mkcert").exists() {
            return "/usr/local/bin/mkcert";
        }
    }
    "mkcert"
}

/// Install the local CA into the system trust store.
/// Only needs to be run once per machine.
pub async fn install_ca() -> Result<()> {
    let output = Command::new(mkcert_bin())
        .arg("-install")
        .output()
        .await?;

    if output.status.success() {
        Ok(())
    } else {
        Err(MacEnvError::Other(format!(
            "mkcert -install failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )))
    }
}

/// Check if the local CA is installed.
pub async fn is_ca_installed() -> bool {
    let output = Command::new(mkcert_bin())
        .arg("-CAROOT")
        .output()
        .await;

    match output {
        Ok(o) => {
            let ca_root = String::from_utf8_lossy(&o.stdout).trim().to_string();
            let ca_cert = PathBuf::from(&ca_root).join("rootCA.pem");
            ca_cert.exists()
        }
        Err(_) => false,
    }
}

/// Generate a trusted SSL certificate for a domain.
/// Returns (cert_path, key_path).
pub async fn generate_cert(domain: &str) -> Result<(PathBuf, PathBuf)> {
    let dir = certs_dir();
    let cert_path = dir.join(format!("{}.pem", domain));
    let key_path = dir.join(format!("{}-key.pem", domain));

    // Skip if already exists
    if cert_path.exists() && key_path.exists() {
        return Ok((cert_path, key_path));
    }

    let output = Command::new(mkcert_bin())
        .args([
            "-cert-file",
            cert_path.to_str().unwrap_or(""),
            "-key-file",
            key_path.to_str().unwrap_or(""),
            domain,
            // Also cover wildcard subdomain
            &format!("*.{}", domain),
        ])
        .output()
        .await?;

    if output.status.success() {
        Ok((cert_path, key_path))
    } else {
        Err(MacEnvError::Other(format!(
            "mkcert failed for {}: {}",
            domain,
            String::from_utf8_lossy(&output.stderr)
        )))
    }
}

/// Check if a certificate exists for a domain.
pub fn cert_exists(domain: &str) -> bool {
    let dir = certs_dir();
    dir.join(format!("{}.pem", domain)).exists()
        && dir.join(format!("{}-key.pem", domain)).exists()
}

/// Remove certificates for a domain.
pub async fn remove_cert(domain: &str) -> Result<()> {
    let dir = certs_dir();
    let cert = dir.join(format!("{}.pem", domain));
    let key = dir.join(format!("{}-key.pem", domain));

    if cert.exists() {
        tokio::fs::remove_file(&cert).await?;
    }
    if key.exists() {
        tokio::fs::remove_file(&key).await?;
    }
    Ok(())
}
