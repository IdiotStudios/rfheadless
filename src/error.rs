//! Error types for the headless engine

use thiserror::Error;

/// Result type alias for engine operations
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur in the headless engine
#[derive(Error, Debug)]
pub enum Error {
    /// Failed to initialize the engine
    #[error("Engine initialization failed: {0}")]
    InitializationError(String),

    /// Failed to load a URL
    #[error("Failed to load URL: {0}")]
    LoadError(String),

    /// Failed to render content
    #[error("Rendering failed: {0}")]
    RenderError(String),

    /// Failed to execute JavaScript
    #[error("Script execution failed: {0}")]
    ScriptError(String),

    /// Operation timed out
    #[error("Operation timed out after {0}ms")]
    Timeout(u64),

    /// Invalid configuration
    #[error("Invalid configuration: {0}")]
    ConfigError(String),

    /// Network error
    #[error("Network error: {0}")]
    NetworkError(String),

    /// CDP-specific error
    #[cfg(feature = "cdp")]
    #[error("CDP error: {0}")]
    CdpError(String),

    /// Generic error
    #[error("{0}")]
    Other(String),
}

#[cfg(feature = "cdp")]
impl From<anyhow::Error> for Error {
    fn from(err: anyhow::Error) -> Self {
        Error::CdpError(err.to_string())
    }
}
