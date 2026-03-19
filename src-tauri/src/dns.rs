use std::path::PathBuf;
use tokio::process::Command;

use crate::error::{MacEnvError, Result};

/// Get the dnsmasq config file path for the current platform.
pub fn dnsmasq_config_path() -> PathBuf {
    if cfg!(target_os = "macos") {
        let prefix = if cfg!(target_arch = "aarch64") {
            "/opt/homebrew"
        } else {
            "/usr/local"
        };
        PathBuf::from(prefix).join("etc/dnsmasq.conf")
    } else {
        PathBuf::from("/etc/dnsmasq.conf")
    }
}

const TEST_TLD_ENTRY: &str = "address=/.test/127.0.0.1";

/// Check if dnsmasq is already configured for .test TLD.
pub async fn is_configured() -> Result<bool> {
    let config_path = dnsmasq_config_path();
    if !config_path.exists() {
        return Ok(false);
    }

    let content = tokio::fs::read_to_string(&config_path).await?;
    Ok(content.contains(TEST_TLD_ENTRY))
}

/// Configure dnsmasq for .test TLD resolution.
/// On macOS, also creates /etc/resolver/test (requires sudo for that step only).
/// On Linux, just adds to dnsmasq.conf.
pub async fn setup_dnsmasq() -> Result<()> {
    let config_path = dnsmasq_config_path();

    // Add .test resolution to dnsmasq.conf if not already there
    if !is_configured().await? {
        let existing = if config_path.exists() {
            tokio::fs::read_to_string(&config_path)
                .await
                .unwrap_or_default()
        } else {
            String::new()
        };

        let new_content = format!(
            "{}\n# MacEnv: resolve *.test to localhost\n{}\n",
            existing.trim_end(),
            TEST_TLD_ENTRY
        );

        // On Linux, dnsmasq.conf is owned by root, need to write via tee
        if cfg!(target_os = "linux") {
            let output = Command::new("sudo")
                .args(["tee", "-a", config_path.to_str().unwrap_or("")])
                .kill_on_drop(true)
                .stdin(std::process::Stdio::piped())
                .stdout(std::process::Stdio::null())
                .spawn();

            match output {
                Ok(mut child) => {
                    if let Some(stdin) = child.stdin.as_mut() {
                        use tokio::io::AsyncWriteExt;
                        let line = format!(
                            "\n# MacEnv: resolve *.test to localhost\n{}\n",
                            TEST_TLD_ENTRY
                        );
                        stdin.write_all(line.as_bytes()).await?;
                        stdin.shutdown().await?;
                    }
                    child.wait().await?;
                }
                Err(e) => {
                    return Err(MacEnvError::Other(format!(
                        "Failed to write dnsmasq config: {}. Try running: echo '{}' | sudo tee -a {}",
                        e, TEST_TLD_ENTRY, config_path.display()
                    )));
                }
            }
        } else {
            // macOS: Homebrew dnsmasq.conf is user-writable
            tokio::fs::write(&config_path, new_content).await?;
        }
    }

    // macOS: create /etc/resolver/test
    if cfg!(target_os = "macos") {
        setup_macos_resolver().await?;
    }

    // Linux: configure systemd-resolved if present
    if cfg!(target_os = "linux") {
        setup_linux_resolved().await.ok(); // Best effort
    }

    // Restart dnsmasq
    restart_dnsmasq().await?;

    Ok(())
}

/// macOS: create /etc/resolver/test so the system uses dnsmasq for .test domains.
async fn setup_macos_resolver() -> Result<()> {
    let resolver_dir = PathBuf::from("/etc/resolver");
    let resolver_file = resolver_dir.join("test");

    if resolver_file.exists() {
        return Ok(());
    }

    // These commands need sudo
    let mkdir = Command::new("sudo")
        .args(["mkdir", "-p", "/etc/resolver"])
        .output()
        .await?;

    if !mkdir.status.success() {
        return Err(MacEnvError::ElevationRequired {
            context: "Create /etc/resolver directory".into(),
        });
    }

    let write = Command::new("sudo")
        .args([
            "bash",
            "-c",
            "echo 'nameserver 127.0.0.1' > /etc/resolver/test",
        ])
        .output()
        .await?;

    if !write.status.success() {
        return Err(MacEnvError::ElevationRequired {
            context: "Write /etc/resolver/test".into(),
        });
    }

    Ok(())
}

/// Linux: configure systemd-resolved to forward .test queries to dnsmasq.
async fn setup_linux_resolved() -> Result<()> {
    // Check if systemd-resolved is running
    let status = Command::new("systemctl")
        .args(["is-active", "systemd-resolved"])
        .output()
        .await?;

    if !status.status.success() {
        return Ok(()); // Not using systemd-resolved, skip
    }

    // Add dnsmasq as a DNS server for .test via resolvectl
    let _ = Command::new("sudo")
        .args(["resolvectl", "dns", "lo", "127.0.0.1"])
        .output()
        .await;

    let _ = Command::new("sudo")
        .args(["resolvectl", "domain", "lo", "~test"])
        .output()
        .await;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dnsmasq_config_path_not_empty() {
        let path = dnsmasq_config_path();
        assert!(!path.to_string_lossy().is_empty());
        assert!(path.to_string_lossy().contains("dnsmasq"));
    }

    #[test]
    fn test_test_tld_entry_constant() {
        assert_eq!(TEST_TLD_ENTRY, "address=/.test/127.0.0.1");
    }
}

/// Restart the dnsmasq service.
pub async fn restart_dnsmasq() -> Result<()> {
    let output = if cfg!(target_os = "macos") {
        let brew = if cfg!(target_arch = "aarch64") {
            "/opt/homebrew/bin/brew"
        } else {
            "/usr/local/bin/brew"
        };
        Command::new(brew)
            .args(["services", "restart", "dnsmasq"])
            .output()
            .await?
    } else if cfg!(target_os = "linux") {
        Command::new("sudo")
            .args(["systemctl", "restart", "dnsmasq"])
            .output()
            .await?
    } else {
        return Ok(()); // Windows: not supported yet
    };

    if output.status.success() {
        Ok(())
    } else {
        Err(MacEnvError::ServiceOperationFailed {
            service: "dnsmasq".into(),
            op: "restart".into(),
            reason: String::from_utf8_lossy(&output.stderr).into(),
        })
    }
}
