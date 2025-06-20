//! Comprehensive logging configuration and utilities for HAL9
//!
//! This module provides structured logging with consistent formats,
//! performance metrics, and request/response tracing.

use std::time::Duration;
use tracing::Span;
use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter,
};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::Serialize;

/// Structured log format for consistent logging across all components
#[derive(Debug, Clone, Serialize)]
pub struct StructuredLog {
    /// Timestamp of the log entry
    pub timestamp: DateTime<Utc>,
    /// Log level (TRACE, DEBUG, INFO, WARN, ERROR)
    pub level: String,
    /// Component or module name
    pub component: String,
    /// Log message
    pub message: String,
    /// Optional trace ID for request correlation
    pub trace_id: Option<String>,
    /// Optional span ID for operation tracking
    pub span_id: Option<String>,
    /// Additional context fields
    #[serde(flatten)]
    pub fields: serde_json::Value,
}

/// Initialize the global logging subscriber with structured JSON output
pub fn init_structured_logging() {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| {
            // Default log levels per module
            "info,hal9=debug,hal9_core=debug,hal9_server=debug,tower_http=debug".into()
        });

    let fmt_layer = fmt::layer()
        .json()
        .with_current_span(true)
        .with_span_list(true)
        .with_target(true)
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_file(true)
        .with_line_number(true)
        .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE);

    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt_layer)
        .init();
}

/// Initialize the global logging subscriber with pretty human-readable output
pub fn init_pretty_logging() {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| {
            "info,hal9=debug,hal9_core=debug,hal9_server=debug,tower_http=debug".into()
        });

    let fmt_layer = fmt::layer()
        .pretty()
        .with_target(true)
        .with_thread_ids(false)
        .with_file(true)
        .with_line_number(true)
        .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE);

    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt_layer)
        .init();
}

/// Generate a new trace ID for request correlation
pub fn generate_trace_id() -> String {
    Uuid::new_v4().to_string()
}

/// Log performance metrics for an operation
#[derive(Debug, Clone, Serialize)]
pub struct PerformanceLog {
    pub operation: String,
    pub duration_ms: f64,
    pub success: bool,
    pub error: Option<String>,
    #[serde(flatten)]
    pub metadata: serde_json::Value,
}

impl PerformanceLog {
    pub fn new(operation: impl Into<String>, duration: Duration, success: bool) -> Self {
        Self {
            operation: operation.into(),
            duration_ms: duration.as_secs_f64() * 1000.0,
            success,
            error: None,
            metadata: serde_json::Value::Object(serde_json::Map::new()),
        }
    }

    pub fn with_error(mut self, error: impl Into<String>) -> Self {
        self.error = Some(error.into());
        self.success = false;
        self
    }

    pub fn with_metadata(mut self, key: &str, value: impl Serialize) -> Self {
        if let serde_json::Value::Object(ref mut map) = self.metadata {
            map.insert(key.to_string(), serde_json::to_value(value).unwrap_or(serde_json::Value::Null));
        }
        self
    }
}

/// Macro for structured logging with consistent format
#[macro_export]
macro_rules! log_structured {
    ($level:expr, $component:expr, $message:expr $(, $key:expr => $value:expr)*) => {{
        use serde_json::json;
        let mut fields = json!({});
        $(
            fields[$key] = json!($value);
        )*
        
        match $level {
            tracing::Level::ERROR => tracing::error!(
                component = $component,
                fields = %fields,
                "{}",
                $message
            ),
            tracing::Level::WARN => tracing::warn!(
                component = $component,
                fields = %fields,
                "{}",
                $message
            ),
            tracing::Level::INFO => tracing::info!(
                component = $component,
                fields = %fields,
                "{}",
                $message
            ),
            tracing::Level::DEBUG => tracing::debug!(
                component = $component,
                fields = %fields,
                "{}",
                $message
            ),
            tracing::Level::TRACE => tracing::trace!(
                component = $component,
                fields = %fields,
                "{}",
                $message
            ),
        }
    }};
}

/// Macro for logging performance metrics
#[macro_export]
macro_rules! log_performance {
    ($operation:expr, $duration:expr, $success:expr $(, $key:expr => $value:expr)*) => {{
        let mut perf_log = $crate::logging::PerformanceLog::new($operation, $duration, $success);
        $(
            perf_log = perf_log.with_metadata($key, $value);
        )*
        
        if $success {
            tracing::info!(
                target: "performance",
                operation = perf_log.operation,
                duration_ms = perf_log.duration_ms,
                success = perf_log.success,
                metadata = %serde_json::to_string(&perf_log.metadata).unwrap_or_default(),
                "Operation completed"
            );
        } else {
            tracing::warn!(
                target: "performance",
                operation = perf_log.operation,
                duration_ms = perf_log.duration_ms,
                success = perf_log.success,
                error = ?perf_log.error,
                metadata = %serde_json::to_string(&perf_log.metadata).unwrap_or_default(),
                "Operation failed"
            );
        }
    }};
}

/// Create a new span for a neuron operation
pub fn neuron_span(neuron_id: &str, layer: &str, operation: &str) -> Span {
    tracing::info_span!(
        "neuron_operation",
        neuron_id = %neuron_id,
        layer = %layer,
        operation = %operation,
        trace_id = %generate_trace_id()
    )
}

/// Create a new span for an API request
pub fn api_span(method: &str, path: &str, trace_id: &str) -> Span {
    tracing::info_span!(
        "api_request",
        method = %method,
        path = %path,
        trace_id = %trace_id
    )
}

/// Create a new span for a database query
pub fn db_span(query_type: &str, table: &str) -> Span {
    tracing::debug_span!(
        "db_query",
        query_type = %query_type,
        table = %table
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    
    

    #[test]
    fn test_performance_log_creation() {
        let perf_log = PerformanceLog::new("test_operation", Duration::from_millis(150), true)
            .with_metadata("user_id", "12345")
            .with_metadata("request_size", 1024);

        assert_eq!(perf_log.operation, "test_operation");
        assert_eq!(perf_log.duration_ms, 150.0);
        assert!(perf_log.success);
        assert!(perf_log.error.is_none());
    }

    #[test]
    fn test_performance_log_with_error() {
        let perf_log = PerformanceLog::new("failing_operation", Duration::from_millis(50), false)
            .with_error("Connection timeout");

        assert!(!perf_log.success);
        assert_eq!(perf_log.error, Some("Connection timeout".to_string()));
    }

    #[test]
    fn test_trace_id_generation() {
        let trace_id1 = generate_trace_id();
        let trace_id2 = generate_trace_id();
        
        assert_ne!(trace_id1, trace_id2);
        assert_eq!(trace_id1.len(), 36); // UUID v4 format
    }
}