use thiserror::Error;

#[derive(Error, Debug)]
pub enum ThemeError {
    #[error("failed to parse JSON: {0}")]
    Json(#[from] serde_json::Error),

    #[error("failed to parse TOML: {0}")]
    Toml(#[from] toml::de::Error),

    #[error("failed to serialize TOML: {0}")]
    TomlSerialize(#[from] toml::ser::Error),

    #[error("failed to parse XML: {0}")]
    XmlParse(String),

    #[error("invalid theme: {0}")]
    Validation(String),

    #[error("unknown theme: {0}")]
    UnknownTheme(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("download feature not enabled")]
    DownloadDisabled,
}
