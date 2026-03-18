use std::path::PathBuf;
use tokio::process::Command;

use crate::error::Result;

/// Default Mailpit web UI URL.
pub const MAILPIT_URL: &str = "http://localhost:8025";

/// Default Mailpit SMTP port.
pub const MAILPIT_SMTP_PORT: u16 = 1025;

/// Find the mailpit binary path.
fn mailpit_bin() -> &'static str {
    if cfg!(target_os = "macos") {
        if cfg!(target_arch = "aarch64") {
            if PathBuf::from("/opt/homebrew/bin/mailpit").exists() {
                return "/opt/homebrew/bin/mailpit";
            }
        } else if PathBuf::from("/usr/local/bin/mailpit").exists() {
            return "/usr/local/bin/mailpit";
        }
    }
    "mailpit"
}

/// Get the sendmail_path value for PHP configuration.
pub fn php_sendmail_path() -> String {
    format!("{} sendmail", mailpit_bin())
}

/// Get PHP ini paths for the current platform (all installed versions).
fn php_ini_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();

    if cfg!(target_os = "macos") {
        let prefix = if cfg!(target_arch = "aarch64") {
            "/opt/homebrew"
        } else {
            "/usr/local"
        };
        for version in &["8.1", "8.2", "8.3", "8.4"] {
            let ini = PathBuf::from(prefix)
                .join(format!("etc/php/{}/php.ini", version));
            if ini.exists() {
                paths.push(ini);
            }
        }
    } else if cfg!(target_os = "linux") {
        // Debian/Ubuntu: /etc/php/{version}/fpm/php.ini
        for version in &["8.1", "8.2", "8.3", "8.4"] {
            let ini = PathBuf::from(format!("/etc/php/{}/fpm/php.ini", version));
            if ini.exists() {
                paths.push(ini);
            }
            let cli_ini = PathBuf::from(format!("/etc/php/{}/cli/php.ini", version));
            if cli_ini.exists() {
                paths.push(cli_ini);
            }
        }
        // Arch: /etc/php/php.ini
        let arch_ini = PathBuf::from("/etc/php/php.ini");
        if arch_ini.exists() {
            paths.push(arch_ini);
        }
    }

    paths
}

/// Configure PHP to use Mailpit for mail sending.
/// Updates sendmail_path in all found php.ini files.
pub async fn configure_php_mail() -> Result<Vec<String>> {
    let sendmail = php_sendmail_path();
    let ini_paths = php_ini_paths();
    let mut results = Vec::new();

    for ini_path in ini_paths {
        let content = tokio::fs::read_to_string(&ini_path).await;
        match content {
            Ok(text) => {
                let needs_sudo = cfg!(target_os = "linux");
                let new_line = format!("sendmail_path = {}", sendmail);

                if text.contains(&new_line) {
                    results.push(format!("{}: already configured", ini_path.display()));
                    continue;
                }

                // Replace existing sendmail_path or append
                let updated = if text.contains("sendmail_path") {
                    // Replace the line
                    text.lines()
                        .map(|line| {
                            if line.trim_start().starts_with("sendmail_path")
                                || line.trim_start().starts_with(";sendmail_path")
                            {
                                new_line.as_str()
                            } else {
                                line
                            }
                        })
                        .collect::<Vec<_>>()
                        .join("\n")
                } else {
                    format!("{}\n{}\n", text, new_line)
                };

                if needs_sudo {
                    // Write via sudo tee
                    let output = Command::new("sudo")
                        .args(["tee", ini_path.to_str().unwrap_or("")])
                        .kill_on_drop(true)
                        .stdin(std::process::Stdio::piped())
                        .stdout(std::process::Stdio::null())
                        .spawn();

                    match output {
                        Ok(mut child) => {
                            if let Some(stdin) = child.stdin.as_mut() {
                                use tokio::io::AsyncWriteExt;
                                stdin.write_all(updated.as_bytes()).await?;
                                stdin.shutdown().await?;
                            }
                            child.wait().await?;
                            results.push(format!("{}: configured (sudo)", ini_path.display()));
                        }
                        Err(e) => {
                            results.push(format!("{}: failed ({})", ini_path.display(), e));
                        }
                    }
                } else {
                    tokio::fs::write(&ini_path, updated).await?;
                    results.push(format!("{}: configured", ini_path.display()));
                }
            }
            Err(e) => {
                results.push(format!("{}: skipped ({})", ini_path.display(), e));
            }
        }
    }

    Ok(results)
}

/// Check if Mailpit is currently running.
pub async fn is_mailpit_running() -> bool {
    // Try connecting to the web UI
    let output = Command::new("curl")
        .args(["-s", "-o", "/dev/null", "-w", "%{http_code}", MAILPIT_URL])
        .output()
        .await;

    match output {
        Ok(o) => {
            let code = String::from_utf8_lossy(&o.stdout).trim().to_string();
            code == "200"
        }
        Err(_) => false,
    }
}
