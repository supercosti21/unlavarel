use async_trait::async_trait;
use tokio::process::Command;

use super::{ServiceInfo, ServiceManager, ServiceStatus};
use crate::error::{MacEnvError, Result};

const MANAGED_SERVICES: &[(&str, &str)] = &[
    ("php", "php-fpm"),
    ("mysql", "mariadb"), // Arch uses mariadb
    ("mariadb", "mariadb"),
    ("nginx", "nginx"),
    ("redis", "redis"),
    ("memcached", "memcached"),
    ("dnsmasq", "dnsmasq"),
    ("mailpit", "mailpit"),
    ("postgresql", "postgresql"),
];

pub struct Systemd;

impl Default for Systemd {
    fn default() -> Self {
        Self::new()
    }
}

impl Systemd {
    pub fn new() -> Self {
        Self
    }

    fn resolve_unit_name<'a>(&self, canonical: &'a str) -> &'a str {
        MANAGED_SERVICES
            .iter()
            .find(|(c, _)| *c == canonical)
            .map(|(_, u)| *u)
            .unwrap_or(canonical)
    }

    async fn systemctl(&self, args: &[&str]) -> Result<String> {
        let output = Command::new("systemctl").args(args).output().await?;

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// Run systemctl with pkexec for operations that need root
    async fn systemctl_elevated(&self, args: &[&str]) -> Result<String> {
        let mut full_args = vec!["systemctl"];
        full_args.extend_from_slice(args);
        let script = full_args.join(" ");

        let output = crate::elevated::run_script_elevated(&script)
            .await
            .map_err(|e| MacEnvError::ServiceOperationFailed {
                service: args.last().unwrap_or(&"unknown").to_string(),
                op: args.first().unwrap_or(&"unknown").to_string(),
                reason: e,
            })?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            Err(MacEnvError::ServiceOperationFailed {
                service: args.last().unwrap_or(&"unknown").to_string(),
                op: args.first().unwrap_or(&"unknown").to_string(),
                reason: stderr,
            })
        }
    }

    async fn detect_version(&self, service: &str) -> String {
        let binary = match service {
            "php" => "php",
            "mysql" | "mariadb" => "mysql",
            "nginx" => "nginx",
            "redis" => "redis-server",
            "postgresql" => "psql",
            _ => return String::new(),
        };

        let output = Command::new(binary).arg("--version").output().await;
        match output {
            Ok(o) => {
                let text = if o.status.success() {
                    String::from_utf8_lossy(&o.stdout).to_string()
                } else {
                    String::from_utf8_lossy(&o.stderr).to_string()
                };
                text.lines().next().unwrap_or("").trim().to_string()
            }
            _ => String::new(),
        }
    }

    /// Initialize MariaDB data directory if it hasn't been done yet
    async fn ensure_mariadb_initialized(&self) -> Result<()> {
        let data_dir = std::path::PathBuf::from("/var/lib/mysql");

        // If the data dir exists and has files, it's already initialized
        if data_dir.exists() {
            if let Ok(mut entries) = tokio::fs::read_dir(&data_dir).await {
                if entries.next_entry().await.ok().flatten().is_some() {
                    return Ok(()); // Already has data
                }
            }
        }

        // Run mariadb-install-db
        let output = crate::elevated::run_elevated(
            "mariadb-install-db",
            &["--user=mysql", "--basedir=/usr", "--datadir=/var/lib/mysql"],
        )
        .await
        .map_err(MacEnvError::Other)?;

        if output.status.success() {
            Ok(())
        } else {
            Err(MacEnvError::ServiceOperationFailed {
                service: "mariadb".into(),
                op: "initialize".into(),
                reason: String::from_utf8_lossy(&output.stderr).into(),
            })
        }
    }
}

#[async_trait]
impl ServiceManager for Systemd {
    fn name(&self) -> &'static str {
        "systemd"
    }

    async fn start(&self, service: &str) -> Result<()> {
        let unit = self.resolve_unit_name(service);

        // Special handling: initialize MariaDB on first start
        if unit == "mariadb" {
            self.ensure_mariadb_initialized().await?;
        }

        self.systemctl_elevated(&["start", unit]).await?;
        Ok(())
    }

    async fn stop(&self, service: &str) -> Result<()> {
        let unit = self.resolve_unit_name(service);
        self.systemctl_elevated(&["stop", unit]).await?;
        Ok(())
    }

    async fn restart(&self, service: &str) -> Result<()> {
        let unit = self.resolve_unit_name(service);

        if unit == "mariadb" {
            self.ensure_mariadb_initialized().await?;
        }

        self.systemctl_elevated(&["restart", unit]).await?;
        Ok(())
    }

    async fn status(&self, service: &str) -> Result<ServiceInfo> {
        let unit = self.resolve_unit_name(service);
        // status check doesn't need root
        let output = self.systemctl(&["is-active", unit]).await?;
        let version = self.detect_version(service).await;

        let status = match output.trim() {
            "active" => ServiceStatus::Running,
            "inactive" | "dead" => ServiceStatus::Stopped,
            "failed" => ServiceStatus::Errored("service failed".into()),
            _ => ServiceStatus::Unknown,
        };

        let pid_output = self.systemctl(&["show", "-p", "MainPID", unit]).await?;
        let pid = pid_output
            .trim()
            .strip_prefix("MainPID=")
            .and_then(|p| p.parse().ok())
            .filter(|&p: &u32| p > 0);

        Ok(ServiceInfo {
            name: service.to_string(),
            status,
            version,
            pid,
        })
    }

    async fn list_managed(&self) -> Result<Vec<ServiceInfo>> {
        let mut services = Vec::new();

        for (canonical, unit) in MANAGED_SERVICES {
            // Skip if the unit doesn't exist on this system
            let exists = Command::new("systemctl")
                .args(["cat", unit])
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .output()
                .await
                .map(|o| o.status.success())
                .unwrap_or(false);

            // Also check if the binary exists
            let binary_exists = match *canonical {
                "php" => which("php").await,
                "mysql" | "mariadb" => which("mysql").await || which("mariadb").await,
                "nginx" => which("nginx").await,
                "redis" => which("redis-server").await,
                "memcached" => which("memcached").await,
                "dnsmasq" => which("dnsmasq").await,
                "mailpit" => which("mailpit").await,
                "postgresql" => which("psql").await,
                _ => false,
            };

            if !exists && !binary_exists {
                continue; // Skip services that aren't installed
            }

            let info = self.status(canonical).await.unwrap_or(ServiceInfo {
                name: canonical.to_string(),
                status: ServiceStatus::Stopped,
                version: String::new(),
                pid: None,
            });
            services.push(info);
        }

        Ok(services)
    }

    async fn logs(&self, service: &str, lines: usize) -> Result<String> {
        let unit = self.resolve_unit_name(service);
        let output = Command::new("journalctl")
            .args([
                "-u",
                unit,
                "-n",
                &lines.to_string(),
                "--no-pager",
                "-o",
                "short-iso",
            ])
            .output()
            .await?;

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}

async fn which(cmd: &str) -> bool {
    Command::new("which")
        .arg(cmd)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .output()
        .await
        .map(|o| o.status.success())
        .unwrap_or(false)
}
