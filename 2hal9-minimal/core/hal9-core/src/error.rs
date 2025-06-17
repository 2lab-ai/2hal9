//! Error types for 2HAL9

use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Neuron {id} error: {message}")]
    Neuron { id: String, message: String },

    #[error("Signal routing error: {0}")]
    Routing(String),

    #[error("Process management error: {0}")]
    Process(String),

    #[error("Communication error: {0}")]
    Communication(String),

    #[error("Claude API error: {0}")]
    ClaudeApi(String),

    #[error("Rate limit exceeded")]
    RateLimit,

    #[error("Timeout after {0} seconds")]
    Timeout(u64),

    #[error("Cost limit exceeded: {reason}")]
    CostLimit { reason: String },

    #[error("Circuit breaker open for {service}")]
    CircuitBreakerOpen { service: String },

    #[error("Invalid state: {0}")]
    InvalidState(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Tool execution error: {0}")]
    ToolExecution(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Protocol error: {0}")]
    Protocol(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Resource exhausted: {0}")]
    ResourceExhausted(String),

    #[error("Resource not found: {0}")]
    ResourceNotFound(String),

    #[error("Transport error: {0}")]
    Transport(String),

    #[error("Storage error: {0}")]
    Storage(String),

    #[error("Deserialization error: {0}")]
    Deserialization(String),

    #[error("Processing error: {0}")]
    Processing(String),

    #[error("Runtime error: {0}")]
    Runtime(String),

    #[error("Migration error: {0}")]
    Migration(String),

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Other error: {0}")]
    Other(#[from] anyhow::Error),
}

impl Error {
    /// Check if error is recoverable
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            Error::RateLimit
                | Error::Timeout(_)
                | Error::Communication(_)
                | Error::CircuitBreakerOpen { .. }
        )
    }

    /// Check if error is fatal
    pub fn is_fatal(&self) -> bool {
        matches!(self, Error::Config(_) | Error::InvalidState(_))
    }
}
