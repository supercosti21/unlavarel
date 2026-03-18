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
