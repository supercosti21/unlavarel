use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum MacEnvError {
    #[error("Package manager `{manager}` is not installed")]
    PackageManagerNotFound { manager: String },

    #[error("Package manager `{manager}` bootstrap failed: {reason}")]
    BootstrapFailed { manager: String, reason: String },

    #[error("Package `{package}` not found in registry for manager `{manager}`")]
    PackageNotInRegistry { package: String, manager: String },

    #[error("Package `{package}` installation failed (exit code {code}): {stderr}")]
    InstallFailed {
        package: String,
        code: i32,
        stderr: String,
    },

    #[error("Package `{package}` version `{version}` is not available")]
    VersionNotAvailable { package: String, version: String },

    #[error("Service `{service}` not found")]
    ServiceNotFound { service: String },

    #[error("Service operation `{op}` failed for `{service}`: {reason}")]
    ServiceOperationFailed {
        service: String,
        op: String,
        reason: String,
    },

    #[error("Operation requires elevated privileges: {context}")]
    ElevationRequired { context: String },

    #[error("Unsupported platform: {os}")]
    UnsupportedPlatform { os: String },

    #[error("Command execution failed: {0}")]
    CommandExec(#[from] std::io::Error),

    #[error("Path `{0}` does not exist")]
    PathNotFound(PathBuf),

    #[error("Command timed out after {seconds}s")]
    Timeout { seconds: u64 },

    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, MacEnvError>;

impl serde::Serialize for MacEnvError {
    fn serialize<S: serde::Serializer>(
        &self,
        serializer: S,
    ) -> std::result::Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display_messages() {
        let err = MacEnvError::PackageManagerNotFound {
            manager: "homebrew".into(),
        };
        assert_eq!(
            err.to_string(),
            "Package manager `homebrew` is not installed"
        );

        let err = MacEnvError::PackageNotInRegistry {
            package: "foo".into(),
            manager: "apt".into(),
        };
        assert_eq!(
            err.to_string(),
            "Package `foo` not found in registry for manager `apt`"
        );

        let err = MacEnvError::ServiceNotFound {
            service: "nginx".into(),
        };
        assert_eq!(err.to_string(), "Service `nginx` not found");

        let err = MacEnvError::ElevationRequired {
            context: "install php".into(),
        };
        assert_eq!(
            err.to_string(),
            "Operation requires elevated privileges: install php"
        );

        let err = MacEnvError::UnsupportedPlatform { os: "haiku".into() };
        assert_eq!(err.to_string(), "Unsupported platform: haiku");

        let err = MacEnvError::Timeout { seconds: 30 };
        assert_eq!(err.to_string(), "Command timed out after 30s");

        let err = MacEnvError::Other("custom error".into());
        assert_eq!(err.to_string(), "custom error");
    }

    #[test]
    fn test_error_serialization() {
        let err = MacEnvError::ServiceNotFound {
            service: "redis".into(),
        };
        let json = serde_json::to_string(&err).unwrap();
        assert_eq!(json, "\"Service `redis` not found\"");
    }

    #[test]
    fn test_io_error_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file missing");
        let err: MacEnvError = io_err.into();
        assert!(err.to_string().contains("file missing"));
    }

    #[test]
    fn test_result_type_alias() {
        let ok: Result<i32> = Ok(42);
        assert_eq!(ok.unwrap(), 42);

        let err: Result<i32> = Err(MacEnvError::Other("fail".into()));
        assert!(err.is_err());
    }
}
