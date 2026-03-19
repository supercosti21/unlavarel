use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Privilege {
    User,
    Elevated,
}

/// Check if the current process is running with elevated privileges.
pub fn is_elevated() -> bool {
    #[cfg(unix)]
    {
        unsafe { libc::geteuid() == 0 }
    }
    #[cfg(not(unix))]
    {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_privilege_serialization() {
        let user = Privilege::User;
        let json = serde_json::to_string(&user).unwrap();
        assert_eq!(json, "\"User\"");

        let elevated = Privilege::Elevated;
        let json = serde_json::to_string(&elevated).unwrap();
        assert_eq!(json, "\"Elevated\"");
    }

    #[test]
    fn test_privilege_equality() {
        assert_eq!(Privilege::User, Privilege::User);
        assert_eq!(Privilege::Elevated, Privilege::Elevated);
        assert_ne!(Privilege::User, Privilege::Elevated);
    }

    #[test]
    fn test_is_elevated_not_root() {
        // Tests typically don't run as root
        if std::env::var("USER").unwrap_or_default() != "root" {
            assert!(!is_elevated());
        }
    }
}
