use thiserror::Error;

/// Errors that can occur during scope matching and font resolution.
#[derive(Error, Debug)]
pub enum PolyfontError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Font not found: {0}")]
    FontNotFound(String),

    #[error("Invalid scope pattern: {0}")]
    InvalidScope(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Parse error: {0}")]
    Parse(String),
}
