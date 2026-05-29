use thiserror::Error;

/// Errors that can occur during theme operations.
#[derive(Error, Debug)]
pub enum ThemeError {
    /// Failed to parse JSON input.
    #[error("failed to parse JSON: {0}")]
    Json(#[from] serde_json::Error),

    /// Failed to parse TOML input.
    #[error("failed to parse TOML: {0}")]
    Toml(#[from] toml::de::Error),

    /// Failed to serialize a value to TOML.
    #[error("failed to serialize TOML: {0}")]
    TomlSerialize(#[from] toml::ser::Error),

    /// Failed to parse XML/plist input.
    #[error("failed to parse XML: {0}")]
    XmlParse(String),

    /// A theme failed validation checks.
    #[error("invalid theme: {0}")]
    Validation(String),

    /// The requested theme name was not found.
    #[error("unknown theme: {0}")]
    UnknownTheme(String),

    /// An I/O error occurred.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// The `download` feature gate is not enabled.
    #[error("download feature not enabled")]
    DownloadDisabled,
}
