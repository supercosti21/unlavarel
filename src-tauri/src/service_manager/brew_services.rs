use async_trait::async_trait;
use std::path::PathBuf;
use tokio::process::Command;

use crate::error::{MacEnvError, Result};
use super::{ServiceInfo, ServiceManager, ServiceStatus};

const MANAGED_SERVICES: &[&str] = &["php", "mysql", "nginx", "redis", "memcached", "dnsmasq", "mailpit"];

pub struct BrewServices {
    brew_path: PathBuf,
}

impl Default for BrewServices {
    fn default() -> Self {
        Self::new()
    }
}

impl BrewServices {
    pub fn new() -> Self {
        let brew_path = if PathBuf::from("/opt/homebrew/bin/brew").exists() {
            PathBuf::from("/opt/homebrew/bin/brew")
        } else {
            PathBuf::from("/usr/local/bin/brew")
        };
        Self { brew_path }
    }

    async fn run_brew_services(&self, args: &[&str]) -> Result<String> {
        let output = Command::new(&self.brew_path)
            .arg("services")
            .args(args)
            .output()
            .await?;

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    fn parse_status_line(&self, line: &str) -> Option<ServiceInfo> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 2 {
            return None;
        }

        let name = parts[0].to_string();
        let status = match parts[1] {
            "started" => ServiceStatus::Running,
            "stopped" | "none" => ServiceStatus::Stopped,
            "error" => ServiceStatus::Errored("service error".into()),
            _ => ServiceStatus::Unknown,
        };

        let pid = parts.get(2).and_then(|p| p.parse().ok());

        Some(ServiceInfo {
            name,
            status,
            version: String::new(),
            pid,
        })
    }

    async fn detect_version(&self, service: &str) -> String {
        let binary = match service {
            "php" => "php",
            "mysql" => "mysql",
            "nginx" => "nginx",
            "redis" => "redis-server",
            _ => return String::new(),
        };

        let output = Command::new(binary)
            .arg("--version")
            .output()
            .await;

        match output {
            Ok(o) if o.status.success() => {
                let stdout = String::from_utf8_lossy(&o.stdout);
                stdout.lines().next().unwrap_or("").to_string()
            }
            _ => String::new(),
        }
    }
}

#[async_trait]
impl ServiceManager for BrewServices {
    fn name(&self) -> &'static str {
        "brew services"
    }

    async fn start(&self, service: &str) -> Result<()> {
        let output = Command::new(&self.brew_path)
            .args(["services", "start", service])
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
        let output = Command::new(&self.brew_path)
            .args(["services", "stop", service])
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
        let output = Command::new(&self.brew_path)
            .args(["services", "restart", service])
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
        let output = self.run_brew_services(&["info", service]).await?;
        let version = self.detect_version(service).await;

        // Parse brew services info output
        let status = if output.contains("running") {
            ServiceStatus::Running
        } else {
            ServiceStatus::Stopped
        };

        Ok(ServiceInfo {
            name: service.to_string(),
            status,
            version,
            pid: None,
        })
    }

    async fn list_managed(&self) -> Result<Vec<ServiceInfo>> {
        let output = self.run_brew_services(&["list"]).await?;
        let mut services = Vec::new();

        for line in output.lines().skip(1) {
            // skip header
            if let Some(mut info) = self.parse_status_line(line) {
                if MANAGED_SERVICES.iter().any(|s| info.name.starts_with(s)) {
                    info.version = self.detect_version(&info.name).await;
                    services.push(info);
                }
            }
        }

        // If no services found via brew, return defaults as stopped
        if services.is_empty() {
            for name in MANAGED_SERVICES {
                services.push(ServiceInfo {
                    name: name.to_string(),
                    status: ServiceStatus::Stopped,
                    version: String::new(),
                    pid: None,
                });
            }
        }

        Ok(services)
    }

    async fn logs(&self, service: &str, lines: usize) -> Result<String> {
        let log_path = format!(
            "{}/var/log/{}.log",
            self.brew_path.parent().unwrap().parent().unwrap().display(),
            service
        );

        match tokio::fs::read_to_string(&log_path).await {
            Ok(content) => {
                let log_lines: Vec<&str> = content.lines().collect();
                let start = log_lines.len().saturating_sub(lines);
                Ok(log_lines[start..].join("\n"))
            }
            Err(_) => Ok(format!("No logs found for {}", service)),
        }
    }
}
