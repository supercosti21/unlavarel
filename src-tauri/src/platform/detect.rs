use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OsType {
    MacOS,
    Linux,
    Windows,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LinuxDistro {
    Debian,
    Ubuntu,
    Arch,
    Fedora,
    Unknown(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedPlatform {
    pub os: OsType,
    pub linux_distro: Option<LinuxDistro>,
    pub arch: String,
}

pub fn current_os() -> OsType {
    if cfg!(target_os = "macos") {
        OsType::MacOS
    } else if cfg!(target_os = "linux") {
        OsType::Linux
    } else if cfg!(target_os = "windows") {
        OsType::Windows
    } else {
        OsType::Linux // fallback
    }
}

pub async fn detect_platform() -> DetectedPlatform {
    let os = current_os();
    let arch = std::env::consts::ARCH.to_string();

    let linux_distro = if os == OsType::Linux {
        Some(detect_linux_distro().await)
    } else {
        None
    };

    DetectedPlatform {
        os,
        linux_distro,
        arch,
    }
}

async fn detect_linux_distro() -> LinuxDistro {
    let os_release = tokio::fs::read_to_string("/etc/os-release").await;
    match os_release {
        Ok(content) => {
            let id = content
                .lines()
                .find(|l| l.starts_with("ID="))
                .map(|l| l.trim_start_matches("ID=").trim_matches('"').to_lowercase());

            match id.as_deref() {
                Some("debian") => LinuxDistro::Debian,
                Some("ubuntu") => LinuxDistro::Ubuntu,
                Some("arch") | Some("manjaro") | Some("endeavouros") => LinuxDistro::Arch,
                Some("fedora") => LinuxDistro::Fedora,
                Some(other) => LinuxDistro::Unknown(other.to_string()),
                None => LinuxDistro::Unknown("unknown".to_string()),
            }
        }
        Err(_) => LinuxDistro::Unknown("unknown".to_string()),
    }
}
