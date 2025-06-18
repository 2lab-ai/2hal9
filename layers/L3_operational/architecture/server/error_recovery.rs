//! Error handling and recovery mechanisms for HAL9 server

use axum::{
    extract::Request,
    http::{StatusCode, HeaderMap},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::RwLock;
use tracing::{error, warn, info, debug};
use uuid::Uuid;

use crate::{
    error::ServerError,
    circuit_breaker::{CircuitBreaker, CircuitState},
    middleware::extract_trace_id,
};

/// Error context for detailed debugging
#[derive(Debug, Clone, Serialize)]
pub struct ErrorContext {
    pub error_id: String,
    pub trace_id: Option<String>,
    pub timestamp: String,
    pub path: String,
    pub method: String,
    pub error_type: String,
    pub message: String,
    pub stack_trace: Option<Vec<String>>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Error response format
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: ErrorDetails,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry_after: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub help_url: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ErrorDetails {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

/// Recovery strategy for different error types
#[derive(Debug, Clone)]
pub enum RecoveryStrategy {
    /// Retry with exponential backoff
    Retry {
        max_attempts: u32,
        initial_delay: Duration,
        max_delay: Duration,
        jitter: bool,
    },
    /// Use circuit breaker pattern
    CircuitBreaker {
        failure_threshold: u32,
        recovery_timeout: Duration,
        half_open_requests: u32,
    },
    /// Fallback to alternative service
    Fallback {
        fallback_fn: Arc<dyn Fn() -> Result<Response, ServerError> + Send + Sync>,
    },
    /// Rate limit and queue
    RateLimit {
        requests_per_second: u32,
        burst_size: u32,
        queue_size: usize,
    },
    /// No recovery, fail fast
    FailFast,
}

/// Error recovery middleware
pub async fn error_recovery_middleware(
    req: Request,
    next: Next,
) -> Result<Response, ServerError> {
    let start = Instant::now();
    let method = req.method().clone();
    let path = req.uri().path().to_string();
    let trace_id = extract_trace_id(&req);
    
    // Process request
    let result = next.run(req).await;
    
    // Convert to our error response format if it's an error
    let response = match result.status().is_server_error() {
        true => {
            let error_id = Uuid::new_v4().to_string();
            let duration = start.elapsed();
            
            // Log error with context
            error!(
                error_id = %error_id,
                trace_id = ?trace_id,
                method = %method,
                path = %path,
                status = %result.status(),
                duration_ms = %duration.as_millis(),
                "Request failed with server error"
            );
            
            // Create error response
            let error_response = ErrorResponse {
                error: ErrorDetails {
                    code: map_status_to_error_code(result.status()),
                    message: "Internal server error occurred".to_string(),
                    error_id: Some(error_id.clone()),
                    details: None,
                },
                retry_after: calculate_retry_after(result.status()),
                help_url: Some(format!("https://docs.2lab.ai/errors/{}", 
                    map_status_to_error_code(result.status()))),
            };
            
            // Store error context for debugging
            if let Some(error_store) = req.extensions().get::<Arc<ErrorStore>>() {
                let context = ErrorContext {
                    error_id,
                    trace_id,
                    timestamp: chrono::Utc::now().to_rfc3339(),
                    path,
                    method: method.to_string(),
                    error_type: "server_error".to_string(),
                    message: result.status().to_string(),
                    stack_trace: None,
                    metadata: HashMap::new(),
                };
                
                error_store.store_error(context).await;
            }
            
            (result.status(), Json(error_response)).into_response()
        },
        false => result,
    };
    
    Ok(response)
}

/// Retry middleware with exponential backoff
pub struct RetryMiddleware {
    max_attempts: u32,
    initial_delay: Duration,
    max_delay: Duration,
    jitter: bool,
}

impl RetryMiddleware {
    pub fn new(max_attempts: u32) -> Self {
        Self {
            max_attempts,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(10),
            jitter: true,
        }
    }
    
    pub async fn execute<F, T>(&self, mut operation: F) -> Result<T, ServerError>
    where
        F: FnMut() -> Result<T, ServerError>,
    {
        let mut attempt = 0;
        let mut delay = self.initial_delay;
        
        loop {
            attempt += 1;
            
            match operation() {
                Ok(result) => return Ok(result),
                Err(e) if attempt >= self.max_attempts => {
                    error!(
                        attempt = attempt,
                        max_attempts = self.max_attempts,
                        "Max retry attempts reached"
                    );
                    return Err(e);
                },
                Err(e) if !is_retryable_error(&e) => {
                    debug!("Error is not retryable: {}", e);
                    return Err(e);
                },
                Err(e) => {
                    warn!(
                        attempt = attempt,
                        delay_ms = delay.as_millis(),
                        error = %e,
                        "Retrying operation after delay"
                    );
                    
                    // Apply jitter if enabled
                    let actual_delay = if self.jitter {
                        let jitter = rand::random::<f64>() * 0.3; // Up to 30% jitter
                        Duration::from_millis((delay.as_millis() as f64 * (1.0 + jitter)) as u64)
                    } else {
                        delay
                    };
                    
                    tokio::time::sleep(actual_delay).await;
                    
                    // Exponential backoff
                    delay = std::cmp::min(delay * 2, self.max_delay);
                }
            }
        }
    }
}

/// Error store for debugging and analysis
pub struct ErrorStore {
    errors: Arc<RwLock<Vec<ErrorContext>>>,
    max_errors: usize,
}

impl ErrorStore {
    pub fn new(max_errors: usize) -> Self {
        Self {
            errors: Arc::new(RwLock::new(Vec::new())),
            max_errors,
        }
    }
    
    pub async fn store_error(&self, context: ErrorContext) {
        let mut errors = self.errors.write().await;
        
        // Add new error
        errors.push(context);
        
        // Keep only recent errors
        if errors.len() > self.max_errors {
            errors.drain(0..errors.len() - self.max_errors);
        }
    }
    
    pub async fn get_recent_errors(&self, limit: usize) -> Vec<ErrorContext> {
        let errors = self.errors.read().await;
        let start = errors.len().saturating_sub(limit);
        errors[start..].to_vec()
    }
    
    pub async fn get_error_by_id(&self, error_id: &str) -> Option<ErrorContext> {
        let errors = self.errors.read().await;
        errors.iter().find(|e| e.error_id == error_id).cloned()
    }
    
    pub async fn clear_errors(&self) {
        let mut errors = self.errors.write().await;
        errors.clear();
    }
}

/// Fallback handler for service degradation
pub struct FallbackHandler {
    fallbacks: HashMap<String, Arc<dyn Fn() -> Response + Send + Sync>>,
}

impl FallbackHandler {
    pub fn new() -> Self {
        Self {
            fallbacks: HashMap::new(),
        }
    }
    
    pub fn register_fallback<F>(&mut self, path: &str, fallback: F)
    where
        F: Fn() -> Response + Send + Sync + 'static,
    {
        self.fallbacks.insert(path.to_string(), Arc::new(fallback));
    }
    
    pub fn get_fallback(&self, path: &str) -> Option<Response> {
        self.fallbacks.get(path).map(|f| f())
    }
}

/// Global error handler for unhandled panics
pub fn setup_panic_handler() {
    let default_panic = std::panic::take_hook();
    
    std::panic::set_hook(Box::new(move |panic_info| {
        let location = panic_info.location()
            .map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column()))
            .unwrap_or_else(|| "unknown location".to_string());
        
        let message = if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
            s.to_string()
        } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
            s.clone()
        } else {
            "Unknown panic payload".to_string()
        };
        
        error!(
            location = %location,
            message = %message,
            "Panic occurred in application"
        );
        
        // Call the default panic handler
        default_panic(panic_info);
    }));
}

/// Helper functions

fn map_status_to_error_code(status: StatusCode) -> String {
    match status {
        StatusCode::BAD_REQUEST => "BAD_REQUEST",
        StatusCode::UNAUTHORIZED => "UNAUTHORIZED",
        StatusCode::FORBIDDEN => "FORBIDDEN",
        StatusCode::NOT_FOUND => "NOT_FOUND",
        StatusCode::CONFLICT => "CONFLICT",
        StatusCode::TOO_MANY_REQUESTS => "RATE_LIMITED",
        StatusCode::INTERNAL_SERVER_ERROR => "INTERNAL_ERROR",
        StatusCode::BAD_GATEWAY => "BAD_GATEWAY",
        StatusCode::SERVICE_UNAVAILABLE => "SERVICE_UNAVAILABLE",
        StatusCode::GATEWAY_TIMEOUT => "GATEWAY_TIMEOUT",
        _ => "UNKNOWN_ERROR",
    }.to_string()
}

fn calculate_retry_after(status: StatusCode) -> Option<u64> {
    match status {
        StatusCode::TOO_MANY_REQUESTS => Some(60),
        StatusCode::SERVICE_UNAVAILABLE => Some(30),
        StatusCode::GATEWAY_TIMEOUT => Some(10),
        _ => None,
    }
}

fn is_retryable_error(error: &ServerError) -> bool {
    match error {
        ServerError::IoError(_) => true,
        ServerError::ClaudeError(_) => true,
        ServerError::Internal(_) => true,
        ServerError::RoutingError(_) => true,
        _ => false,
    }
}

/// Extension trait for ServerError
impl ServerError {
    /// Convert to HTTP response with proper error format
    pub fn to_error_response(&self) -> Response {
        let (status, code) = match self {
            ServerError::NotFound(_) => (StatusCode::NOT_FOUND, "NOT_FOUND"),
            ServerError::InvalidInput(_) => (StatusCode::BAD_REQUEST, "INVALID_INPUT"),
            ServerError::ConfigError(_) => (StatusCode::BAD_REQUEST, "CONFIG_ERROR"),
            ServerError::NeuronError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "NEURON_ERROR"),
            ServerError::RoutingError(_) => (StatusCode::BAD_GATEWAY, "ROUTING_ERROR"),
            ServerError::ClaudeError(_) => (StatusCode::BAD_GATEWAY, "CLAUDE_ERROR"),
            ServerError::IoError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "IO_ERROR"),
            ServerError::SerializationError(_) => (StatusCode::BAD_REQUEST, "SERIALIZATION_ERROR"),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR"),
        };
        
        let error_response = ErrorResponse {
            error: ErrorDetails {
                code: code.to_string(),
                message: self.to_string(),
                error_id: Some(Uuid::new_v4().to_string()),
                details: None,
            },
            retry_after: calculate_retry_after(status),
            help_url: Some(format!("https://docs.2lab.ai/errors/{}", code)),
        };
        
        (status, Json(error_response)).into_response()
    }
    
    /// Check if error is recoverable
    pub fn is_recoverable(&self) -> bool {
        match self {
            ServerError::NotFound(_) => false,
            ServerError::InvalidInput(_) => false,
            ServerError::ConfigError(_) => false,
            _ => true,
        }
    }
    
    /// Get suggested recovery strategy
    pub fn recovery_strategy(&self) -> RecoveryStrategy {
        match self {
            ServerError::ClaudeError(_) => RecoveryStrategy::Retry {
                max_attempts: 3,
                initial_delay: Duration::from_secs(1),
                max_delay: Duration::from_secs(30),
                jitter: true,
            },
            ServerError::IoError(_) => RecoveryStrategy::Retry {
                max_attempts: 5,
                initial_delay: Duration::from_millis(500),
                max_delay: Duration::from_secs(10),
                jitter: true,
            },
            ServerError::RoutingError(_) => RecoveryStrategy::CircuitBreaker {
                failure_threshold: 5,
                recovery_timeout: Duration::from_secs(60),
                half_open_requests: 3,
            },
            _ => RecoveryStrategy::FailFast,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_retry_middleware() {
        let retry = RetryMiddleware::new(3);
        let mut attempts = 0;
        
        let result = retry.execute(|| {
            attempts += 1;
            if attempts < 3 {
                Err(ServerError::Internal("Temporary failure".to_string()))
            } else {
                Ok("Success")
            }
        }).await;
        
        assert!(result.is_ok());
        assert_eq!(attempts, 3);
    }
    
    #[tokio::test]
    async fn test_error_store() {
        let store = ErrorStore::new(100);
        
        let context = ErrorContext {
            error_id: "test-123".to_string(),
            trace_id: Some("trace-456".to_string()),
            timestamp: chrono::Utc::now().to_rfc3339(),
            path: "/test".to_string(),
            method: "GET".to_string(),
            error_type: "test_error".to_string(),
            message: "Test error".to_string(),
            stack_trace: None,
            metadata: HashMap::new(),
        };
        
        store.store_error(context.clone()).await;
        
        let recent = store.get_recent_errors(10).await;
        assert_eq!(recent.len(), 1);
        assert_eq!(recent[0].error_id, "test-123");
        
        let found = store.get_error_by_id("test-123").await;
        assert!(found.is_some());
    }
    
    #[test]
    fn test_error_response_serialization() {
        let response = ErrorResponse {
            error: ErrorDetails {
                code: "TEST_ERROR".to_string(),
                message: "Test error message".to_string(),
                error_id: Some("123".to_string()),
                details: None,
            },
            retry_after: Some(60),
            help_url: Some("https://example.com/help".to_string()),
        };
        
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("TEST_ERROR"));
        assert!(json.contains("retry_after"));
        assert!(json.contains("help_url"));
    }
}