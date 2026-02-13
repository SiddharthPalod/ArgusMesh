/// Unified error handling system for the entire backend.
/// Follows the Error trait pattern for consistent error handling across modules.
use thiserror::Error;

/// Main error type for the Argus Mesh backend.
#[derive(Error, Debug)]
pub enum MeshError {
    #[error("Storage error: {0}")]
    Storage(String),

    #[error("Transport error: {0}")]
    Transport(String),

    #[error("Crypto error: {0}")]
    Crypto(String),

    #[error("Routing error: {0}")]
    Routing(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("Runtime error: {0}")]
    Runtime(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),
}

/// Result type alias for operations that can fail.
pub type MeshResult<T> = Result<T, MeshError>;

/// Convenience conversions from common error types.
impl From<sled::Error> for MeshError {
    fn from(err: sled::Error) -> Self {
        MeshError::Storage(err.to_string())
    }
}

impl From<bincode::Error> for MeshError {
    fn from(err: bincode::Error) -> Self {
        MeshError::Serialization(err.to_string())
    }
}

impl From<serde_json::Error> for MeshError {
    fn from(err: serde_json::Error) -> Self {
        MeshError::Serialization(err.to_string())
    }
}

impl From<crate::transport::error::TransportError> for MeshError {
    fn from(err: crate::transport::error::TransportError) -> Self {
        MeshError::Transport(format!("{:?}", err))
    }
}
