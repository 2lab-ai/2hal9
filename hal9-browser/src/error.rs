//! Browser automation error types

use thiserror::Error;

pub type Result<T> = std::result::Result<T, BrowserError>;

#[derive(Error, Debug)]
pub enum BrowserError {
    #[error("Playwright error: {0}")]
    Playwright(String),
    
    #[error("Context pool exhausted")]
    PoolExhausted,
    
    #[error("Resource limit exceeded: {0}")]
    ResourceLimitExceeded(String),
    
    #[error("Security violation: {0}")]
    SecurityViolation(String),
    
    #[error("Invalid selector: {0}")]
    InvalidSelector(String),
    
    #[error("Element not found: {0}")]
    ElementNotFound(String),
    
    #[error("Timeout waiting for: {0}")]
    Timeout(String),
    
    #[error("Navigation failed: {0}")]
    NavigationFailed(String),
    
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
    
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),
    
    #[error("Credential not found for site: {0}")]
    CredentialNotFound(String),
    
    #[error("Action not allowed: {0}")]
    ActionNotAllowed(String),
    
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl BrowserError {
    /// Check if error is retriable
    pub fn is_retriable(&self) -> bool {
        matches!(self, 
            Self::Playwright(_) | 
            Self::Timeout(_) | 
            Self::NavigationFailed(_) |
            Self::ElementNotFound(_)
        )
    }
}

// Convert from playwright errors
impl From<playwright::Error> for BrowserError {
    fn from(err: playwright::Error) -> Self {
        BrowserError::Playwright(err.to_string())
    }
}