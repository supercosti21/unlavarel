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
        Ok(content) => parse_os_release(&content),
        Err(_) => LinuxDistro::Unknown("unknown".to_string()),
    }
}

/// Parse the ID field from /etc/os-release content.
fn parse_os_release(content: &str) -> LinuxDistro {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_current_os() {
        let os = current_os();
        // Should return something valid on any platform
        assert!(matches!(
            os,
            OsType::MacOS | OsType::Linux | OsType::Windows
        ));
    }

    #[test]
    fn test_os_type_serialization() {
        let os = OsType::Linux;
        let json = serde_json::to_string(&os).unwrap();
        assert_eq!(json, "\"Linux\"");
        let parsed: OsType = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, OsType::Linux);
    }

    #[test]
    fn test_linux_distro_equality() {
        assert_eq!(LinuxDistro::Debian, LinuxDistro::Debian);
        assert_ne!(LinuxDistro::Ubuntu, LinuxDistro::Arch);
    }

    #[test]
    fn test_parse_os_release_ubuntu() {
        let content = "NAME=\"Ubuntu\"\nVERSION=\"22.04.3 LTS\"\nID=ubuntu\nID_LIKE=debian\n";
        assert_eq!(parse_os_release(content), LinuxDistro::Ubuntu);
    }

    #[test]
    fn test_parse_os_release_arch() {
        let content = "NAME=\"Arch Linux\"\nID=arch\nBUILD_ID=rolling\n";
        assert_eq!(parse_os_release(content), LinuxDistro::Arch);
    }

    #[test]
    fn test_parse_os_release_manjaro() {
        let content = "NAME=\"Manjaro Linux\"\nID=manjaro\n";
        assert_eq!(parse_os_release(content), LinuxDistro::Arch);
    }

    #[test]
    fn test_parse_os_release_debian() {
        let content = "ID=debian\nVERSION_ID=\"12\"\n";
        assert_eq!(parse_os_release(content), LinuxDistro::Debian);
    }

    #[test]
    fn test_parse_os_release_fedora() {
        let content = "NAME=\"Fedora Linux\"\nID=fedora\n";
        assert_eq!(parse_os_release(content), LinuxDistro::Fedora);
    }

    #[test]
    fn test_parse_os_release_unknown() {
        let content = "ID=opensuse\n";
        assert_eq!(
            parse_os_release(content),
            LinuxDistro::Unknown("opensuse".to_string())
        );
    }

    #[test]
    fn test_parse_os_release_empty() {
        assert_eq!(
            parse_os_release(""),
            LinuxDistro::Unknown("unknown".to_string())
        );
    }

    #[test]
    fn test_parse_os_release_quoted_id() {
        let content = "ID=\"ubuntu\"\n";
        assert_eq!(parse_os_release(content), LinuxDistro::Ubuntu);
    }

    #[tokio::test]
    async fn test_detect_platform_returns_valid() {
        let platform = detect_platform().await;
        assert!(!platform.arch.is_empty());
        assert!(matches!(
            platform.os,
            OsType::MacOS | OsType::Linux | OsType::Windows
        ));
    }
}
