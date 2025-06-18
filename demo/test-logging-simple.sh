#!/bin/bash
# Simple logging test to demonstrate structured logging

echo "=== HAL9 Structured Logging Test ==="
echo

# Create a simple test program that uses the logging module
cat > /tmp/test_logging.rs << 'EOF'
use tracing::{info, warn, error, debug, instrument};
use tracing_subscriber::{fmt, EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};
use serde_json::json;
use std::time::Instant;

#[instrument]
fn process_request(id: u64, path: &str) -> Result<String, &'static str> {
    let start = Instant::now();
    
    info!(
        request_id = id,
        path = path,
        method = "GET",
        "Processing HTTP request"
    );
    
    // Simulate some work
    std::thread::sleep(std::time::Duration::from_millis(50));
    
    if path == "/error" {
        error!(
            request_id = id,
            path = path,
            error = "Not found",
            status = 404,
            "Request failed"
        );
        return Err("Not found");
    }
    
    let duration = start.elapsed();
    info!(
        request_id = id,
        path = path,
        status = 200,
        duration_ms = duration.as_millis() as u64,
        "Request completed successfully"
    );
    
    Ok("Success".to_string())
}

fn main() {
    // Initialize logging based on LOG_FORMAT env var
    let log_format = std::env::var("LOG_FORMAT").unwrap_or_else(|_| "pretty".to_string());
    
    if log_format == "json" {
        // JSON structured logging
        tracing_subscriber::fmt()
            .json()
            .with_env_filter(EnvFilter::from_default_env())
            .init();
    } else {
        // Pretty logging for development
        tracing_subscriber::fmt()
            .pretty()
            .with_env_filter(EnvFilter::from_default_env())
            .init();
    }
    
    info!(
        service = "hal9-test",
        version = "0.1.0",
        environment = "test",
        "Service starting"
    );
    
    // Test various log levels
    debug!("Debug message - verbose information");
    info!("Info message - general information");
    warn!("Warning message - potential issue");
    
    // Test structured fields
    info!(
        user_id = 123,
        action = "login",
        ip_address = "192.168.1.1",
        "User authentication successful"
    );
    
    // Test error logging with context
    let result = std::panic::catch_unwind(|| {
        panic!("Test panic for error logging");
    });
    
    if result.is_err() {
        error!(
            error_type = "panic",
            message = "Caught panic in test",
            "Error occurred during test execution"
        );
    }
    
    // Test request processing
    let _ = process_request(1001, "/api/health");
    let _ = process_request(1002, "/error");
    let _ = process_request(1003, "/api/neurons");
    
    // Test performance logging
    let start = Instant::now();
    std::thread::sleep(std::time::Duration::from_millis(100));
    let duration = start.elapsed();
    
    info!(
        operation = "database_query",
        table = "neurons",
        rows_returned = 42,
        duration_ms = duration.as_millis() as u64,
        "Database query completed"
    );
    
    // Test batch operation logging
    info!(
        operation = "batch_process",
        total_items = 1000,
        processed = 1000,
        failed = 0,
        duration_ms = 2500,
        items_per_second = 400.0,
        "Batch processing completed"
    );
    
    info!("Service shutdown complete");
}
EOF

# Create a Cargo.toml for the test
cat > /tmp/Cargo.toml << 'EOF'
[package]
name = "test-logging"
version = "0.1.0"
edition = "2021"

[dependencies]
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json", "fmt"] }
serde_json = "1.0"
EOF

echo "Testing pretty logging format:"
echo "=============================="
cd /tmp && LOG_FORMAT=pretty RUST_LOG=debug cargo run --quiet 2>/dev/null || rustc test_logging.rs && LOG_FORMAT=pretty RUST_LOG=debug ./test_logging

echo
echo
echo "Testing JSON logging format:"
echo "============================"
cd /tmp && LOG_FORMAT=json RUST_LOG=info cargo run --quiet 2>/dev/null || LOG_FORMAT=json RUST_LOG=info ./test_logging

echo
echo
echo "Testing with different log levels:"
echo "================================="
echo "ERROR level only:"
cd /tmp && LOG_FORMAT=json RUST_LOG=error ./test_logging 2>/dev/null || true

echo
echo "WARN and above:"
cd /tmp && LOG_FORMAT=json RUST_LOG=warn ./test_logging 2>/dev/null || true

# Clean up
rm -f /tmp/test_logging.rs /tmp/Cargo.toml /tmp/test_logging

echo
echo "✅ Structured logging test complete!"
echo
echo "Key features demonstrated:"
echo "- Configurable output format (pretty/JSON)"
echo "- Structured fields in logs"
echo "- Performance tracking with durations"
echo "- Error context and tracing"
echo "- Request/response logging patterns"
echo "- Environment-based configuration"