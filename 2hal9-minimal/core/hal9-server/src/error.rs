//! Error types for 2HAL9 server

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ServerError {
    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Neuron error: {0}")]
    NeuronError(String),

    #[error("Signal routing error: {0}")]
    RoutingError(String),

    #[error("Claude API error: {0}")]
    ClaudeError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

pub type ServerResult<T> = Result<T, ServerError>;
