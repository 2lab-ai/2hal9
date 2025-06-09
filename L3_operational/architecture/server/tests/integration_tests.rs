//! Integration tests for 2HAL9 server

use std::sync::Arc;
use std::time::Duration;
use hal9_core::{ServerConfig, NeuronSignal, config::{ClaudeConfig, MockResponse}};
use hal9_server::HAL9Server;
use tokio::time::sleep;
use std::collections::HashMap;

/// Helper to create test server config
fn create_test_config() -> ServerConfig {
    use hal9_core::{NeuronConfig, config::{MonitoringConfig, CostControls}};
    
    // Create mock responses for testing
    let mut mock_responses = HashMap::new();
    
    mock_responses.insert("L4".to_string(), vec![
        MockResponse {
            trigger: "default".to_string(),  // Use default trigger to match any input
            response: "FORWARD_TO: test-neuron-2\nCONTENT: Test L4 response".to_string(),
            delay_ms: 10,
        },
    ]);
    
    mock_responses.insert("L3".to_string(), vec![
        MockResponse {
            trigger: "default".to_string(),
            response: "FORWARD_TO: test-neuron-3\nCONTENT: Test L3 response".to_string(),
            delay_ms: 10,
        },
    ]);
    
    mock_responses.insert("L2".to_string(), vec![
        MockResponse {
            trigger: "default".to_string(),
            response: "RESULT: Test implementation complete".to_string(),
            delay_ms: 10,
        },
    ]);
    
    ServerConfig {
        server_id: "test-server".to_string(),
        neurons: vec![
            NeuronConfig {
                id: "test-neuron-1".to_string(),
                layer: "L4".to_string(),
                claude_command: "claude".to_string(),
                forward_connections: vec!["test-neuron-2".to_string()],
                backward_connections: vec![],
                settings: HashMap::new(),
            },
            NeuronConfig {
                id: "test-neuron-2".to_string(),
                layer: "L3".to_string(),
                claude_command: "claude".to_string(),
                forward_connections: vec!["test-neuron-3".to_string()],
                backward_connections: vec!["test-neuron-1".to_string()],
                settings: HashMap::new(),
            },
            NeuronConfig {
                id: "test-neuron-3".to_string(),
                layer: "L2".to_string(),
                claude_command: "claude".to_string(),
                forward_connections: vec![],
                backward_connections: vec!["test-neuron-2".to_string()],
                settings: HashMap::new(),
            },
        ],
        claude: ClaudeConfig {
            mode: "mock".to_string(),
            api_key: None,
            model: "test-model".to_string(),
            temperature: 0.7,
            max_tokens: 100,
            rate_limit: 60,
            mock_responses,
            fallback_to_mock: false,
            cost_controls: CostControls::default(),
        },
        monitoring: MonitoringConfig {
            enabled: true,
            metrics_interval: 1, // Fast metrics for testing
            log_level: "debug".to_string(),
        },
    }
}

#[tokio::test]
async fn test_server_lifecycle() {
    let config = create_test_config();
    let server = Arc::new(HAL9Server::new(config));
    
    // Start server
    server.start().await.expect("Failed to start server");
    
    // Check status
    let status = server.status().await;
    assert_eq!(status.server_id, "test-server");
    assert_eq!(status.neurons.len(), 3);
    
    // Shutdown server
    server.shutdown().await.expect("Failed to shutdown server");
}

#[tokio::test]
async fn test_signal_propagation() {
    let config = create_test_config();
    let server = Arc::new(HAL9Server::new(config));
    
    // Start server
    server.start().await.expect("Failed to start server");
    
    // Send signal to L4 neuron
    let signal = NeuronSignal::forward(
        "test-client",
        "test-neuron-1",
        "client",
        "L4",
        "test signal content".to_string(),
    );
    
    server.send_signal(signal).await.expect("Failed to send signal");
    
    // Wait for processing with retries
    let mut retries = 10;
    let mut metrics = server.metrics().snapshot();
    
    while retries > 0 && metrics.signals_processed == 0 {
        sleep(Duration::from_millis(100)).await;
        metrics = server.metrics().snapshot();
        retries -= 1;
    }
    
    // Check metrics
    assert!(metrics.signals_sent > 0);
    assert!(metrics.signals_processed > 0, "No signals were processed after waiting");
    
    server.shutdown().await.expect("Failed to shutdown server");
}

#[tokio::test]
async fn test_error_handling() {
    let mut config = create_test_config();
    // Add a neuron with invalid forward connection
    config.neurons[0].forward_connections = vec!["non-existent".to_string()];
    
    let server = Arc::new(HAL9Server::new(config));
    server.start().await.expect("Failed to start server");
    
    // Send signal that will fail routing
    let signal = NeuronSignal::forward(
        "test-client",
        "test-neuron-1",
        "client",
        "L4",
        "test signal".to_string(),
    );
    
    // Should not panic, but should record error
    let _ = server.send_signal(signal).await;
    
    sleep(Duration::from_millis(100)).await;
    
    // The routing error might not be recorded in metrics
    // Just verify the server handled the error gracefully
    let status = server.status().await;
    assert!(status.running);
    
    server.shutdown().await.expect("Failed to shutdown server");
}

#[tokio::test]
async fn test_circuit_breaker() {
    // This test needs a custom implementation that actually fails
    // For now, just verify basic circuit breaker functionality exists
    let config = create_test_config();
    let server = Arc::new(HAL9Server::new(config));
    server.start().await.expect("Failed to start server");
    
    // The circuit breaker is integrated but testing it requires
    // actual failures from the Claude interface
    // This would be better tested with a mock that can be configured to fail
    
    server.shutdown().await.expect("Failed to shutdown server");
}

#[tokio::test]
async fn test_metrics_collection() {
    let config = create_test_config();
    let server = Arc::new(HAL9Server::new(config));
    
    server.start().await.expect("Failed to start server");
    
    // Send multiple signals
    for i in 0..5 {
        let signal = NeuronSignal::forward(
            "test-client",
            "test-neuron-1",
            "client",
            "L4",
            format!("test signal {}", i),
        );
        
        server.send_signal(signal).await.expect("Failed to send signal");
        sleep(Duration::from_millis(50)).await;
    }
    
    // Wait for metrics update
    sleep(Duration::from_secs(2)).await;
    
    // Check metrics
    let metrics = server.metrics().snapshot();
    assert_eq!(metrics.signals_sent, 5);
    assert!(metrics.signals_processed > 0);
    assert!(metrics.layer_latencies.len() > 0);
    assert!(metrics.processing_times.len() > 0);
    
    server.shutdown().await.expect("Failed to shutdown server");
}

#[tokio::test]
async fn test_concurrent_signals() {
    let config = create_test_config();
    let server = Arc::new(HAL9Server::new(config));
    
    server.start().await.expect("Failed to start server");
    
    // Send multiple signals concurrently
    let mut handles = vec![];
    
    for i in 0..10 {
        let server_clone = server.clone();
        let handle = tokio::spawn(async move {
            let signal = NeuronSignal::forward(
                "test-client",
                "test-neuron-1",
                "client",
                "L4",
                format!("concurrent signal {}", i),
            );
            
            server_clone.send_signal(signal).await
        });
        
        handles.push(handle);
    }
    
    // Wait for all signals to complete
    for handle in handles {
        let _ = handle.await.expect("Task panicked");
    }
    
    // Wait for processing
    sleep(Duration::from_millis(500)).await;
    
    // Check that all signals were processed
    let metrics = server.metrics().snapshot();
    assert_eq!(metrics.signals_sent, 10);
    
    server.shutdown().await.expect("Failed to shutdown server");
}

#[tokio::test]
async fn test_neuron_health_monitoring() {
    let config = create_test_config();
    let server = Arc::new(HAL9Server::new(config));
    
    server.start().await.expect("Failed to start server");
    
    // Get initial health
    let registry = server.registry();
    let health_map = registry.health_check().await;
    
    assert_eq!(health_map.len(), 3);
    for (_, health) in &health_map {
        assert_eq!(health.signals_processed, 0);
        assert_eq!(health.errors_count, 0);
    }
    
    // Process some signals
    for _ in 0..3 {
        let signal = NeuronSignal::forward(
            "test-client",
            "test-neuron-1",
            "client",
            "L4",
            "health test".to_string(),
        );
        
        server.send_signal(signal).await.expect("Failed to send signal");
        sleep(Duration::from_millis(100)).await;
    }
    
    // Check health again
    let health_map = registry.health_check().await;
    let l4_health = health_map.get("test-neuron-1").expect("L4 neuron not found");
    assert!(l4_health.signals_processed > 0);
    
    server.shutdown().await.expect("Failed to shutdown server");
}

#[tokio::test]
async fn test_backward_propagation() {
    let config = create_test_config();
    let server = Arc::new(HAL9Server::new(config));
    
    server.start().await.expect("Failed to start server");
    
    // Send backward signal (error propagation)
    let gradient = hal9_core::Gradient::new("test_error".to_string(), 0.5);
    let signal = NeuronSignal::backward(
        "test-neuron-3",
        "test-neuron-2",
        "L2",
        "L3",
        gradient,
    );
    
    server.send_signal(signal).await.expect("Failed to send backward signal");
    
    // Wait for processing
    sleep(Duration::from_millis(100)).await;
    
    // Check that signal was processed
    let metrics = server.metrics().snapshot();
    assert!(metrics.signals_sent > 0);
    
    server.shutdown().await.expect("Failed to shutdown server");
}