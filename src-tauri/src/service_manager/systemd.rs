use async_trait::async_trait;
use tokio::process::Command;

use crate::error::{MacEnvError, Result};
use super::{ServiceInfo, ServiceManager, ServiceStatus};

const MANAGED_SERVICES: &[(&str, &str)] = &[
    ("php", "php-fpm"),
    ("mysql", "mysqld"),
    ("nginx", "nginx"),
    ("redis", "redis"),
    ("memcached", "memcached"),
    ("dnsmasq", "dnsmasq"),
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
        let output = Command::new("systemctl")
            .args(args)
            .output()
            .await?;

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    async fn detect_version(&self, service: &str) -> String {
        let binary = match service {
            "php" => "php",
            "mysql" => "mysql",
            "nginx" => "nginx",
            "redis" => "redis-server",
            _ => return String::new(),
        };

        let output = Command::new(binary).arg("--version").output().await;
        match output {
            Ok(o) if o.status.success() => {
                String::from_utf8_lossy(&o.stdout)
                    .lines()
                    .next()
                    .unwrap_or("")
                    .to_string()
            }
            _ => String::new(),
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
        let output = Command::new("systemctl")
            .args(["start", unit])
            .output()
            .await?;

        if output.status.success() {
            Ok(())
        } else {
            Err(MacEnvError::ServiceOperationFailed {
                service: service.into(),
                op: "start".into(),
                reason: String::from_utf8_lossy(&output.stderr).into(),
            })
        }
    }

    async fn stop(&self, service: &str) -> Result<()> {
        let unit = self.resolve_unit_name(service);
        let output = Command::new("systemctl")
            .args(["stop", unit])
            .output()
            .await?;

        if output.status.success() {
            Ok(())
        } else {
            Err(MacEnvError::ServiceOperationFailed {
                service: service.into(),
                op: "stop".into(),
                reason: String::from_utf8_lossy(&output.stderr).into(),
            })
        }
    }

    async fn restart(&self, service: &str) -> Result<()> {
        let unit = self.resolve_unit_name(service);
        let output = Command::new("systemctl")
            .args(["restart", unit])
            .output()
            .await?;

        if output.status.success() {
            Ok(())
        } else {
            Err(MacEnvError::ServiceOperationFailed {
                service: service.into(),
                op: "restart".into(),
                reason: String::from_utf8_lossy(&output.stderr).into(),
            })
        }
    }

    async fn status(&self, service: &str) -> Result<ServiceInfo> {
        let unit = self.resolve_unit_name(service);
        let output = self.systemctl(&["is-active", unit]).await?;
        let version = self.detect_version(service).await;

        let status = match output.trim() {
            "active" => ServiceStatus::Running,
            "inactive" | "dead" => ServiceStatus::Stopped,
            "failed" => ServiceStatus::Errored("service failed".into()),
            _ => ServiceStatus::Unknown,
        };

        // Try to get PID
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

        for (canonical, _) in MANAGED_SERVICES {
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
            .args(["-u", unit, "-n", &lines.to_string(), "--no-pager"])
            .output()
            .await?;

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}
