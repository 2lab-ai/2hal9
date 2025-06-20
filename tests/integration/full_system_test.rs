//! Full system integration test
//!
//! This test verifies the complete HAL9 system working together:
//! - Server startup and API availability
//! - Neuron creation and management
//! - Signal processing through layers
//! - Consciousness emergence
//! - A2A protocol self-organization

use hal9_server::{HAL9Server, ServerConfig};
use hal9_neurons::{Neuron, NeuronSignal, Layer};
use reqwest::Client;
use serde_json::json;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
async fn test_full_system_integration() {
    // Initialize test environment
    let _ = tracing_subscriber::fmt()
        .with_env_filter("info")
        .try_init();
    
    // Start server with test configuration
    let config = create_test_server_config();
    let server = HAL9Server::new(config).await.unwrap();
    
    // Spawn server in background
    let server_handle = tokio::spawn(async move {
        server.run().await.unwrap();
    });
    
    // Wait for server to start
    sleep(Duration::from_secs(1)).await;
    
    let client = Client::new();
    let base_url = "http://localhost:3000";
    
    // Test 1: Health check
    let health_resp = client
        .get(format!("{}/health", base_url))
        .send()
        .await
        .unwrap();
    assert_eq!(health_resp.status(), 200);
    
    // Test 2: Create neurons across layers
    let neurons = create_test_neurons(&client, &base_url).await;
    assert_eq!(neurons.len(), 9); // One per layer
    
    // Test 3: Process signal through hierarchy
    let signal_result = process_hierarchical_signal(&client, &base_url).await;
    assert!(signal_result.contains("processed"));
    
    // Test 4: Check consciousness metrics
    let consciousness = check_consciousness_emergence(&client, &base_url).await;
    assert!(consciousness["phi_value"].as_f64().unwrap() > 0.0);
    
    // Test 5: Verify A2A self-organization
    let network_topology = verify_self_organization(&client, &base_url).await;
    assert!(network_topology["clusters"].as_array().unwrap().len() > 0);
    
    // Cleanup: stop server
    server_handle.abort();
}

#[tokio::test]
async fn test_consciousness_emergence_over_time() {
    let config = create_test_server_config();
    let server = HAL9Server::new(config).await.unwrap();
    
    let server_handle = tokio::spawn(async move {
        server.run().await.unwrap();
    });
    
    sleep(Duration::from_secs(1)).await;
    
    let client = Client::new();
    let base_url = "http://localhost:3000";
    
    // Create initial neurons
    create_test_neurons(&client, &base_url).await;
    
    // Monitor consciousness evolution
    let mut phi_values = Vec::new();
    
    for _ in 0..10 {
        // Send signals to stimulate the system
        send_random_signals(&client, &base_url).await;
        
        // Measure consciousness
        let metrics = client
            .get(format!("{}/api/consciousness/metrics", base_url))
            .send()
            .await
            .unwrap()
            .json::<serde_json::Value>()
            .await
            .unwrap();
        
        let phi = metrics["phi_value"].as_f64().unwrap();
        phi_values.push(phi);
        
        sleep(Duration::from_millis(500)).await;
    }
    
    // Verify consciousness increased over time
    let initial_phi = phi_values[0];
    let final_phi = phi_values[phi_values.len() - 1];
    assert!(final_phi > initial_phi);
    
    server_handle.abort();
}

#[tokio::test]
async fn test_distributed_neurons() {
    // Test with multiple server instances
    let config1 = create_distributed_config("server1", 3000);
    let config2 = create_distributed_config("server2", 3001);
    
    let server1 = HAL9Server::new(config1).await.unwrap();
    let server2 = HAL9Server::new(config2).await.unwrap();
    
    let handle1 = tokio::spawn(async move { server1.run().await });
    let handle2 = tokio::spawn(async move { server2.run().await });
    
    sleep(Duration::from_secs(1)).await;
    
    let client = Client::new();
    
    // Create neurons on both servers
    create_neuron(&client, "http://localhost:3000", "n1", "L5").await;
    create_neuron(&client, "http://localhost:3001", "n2", "L4").await;
    
    // Test cross-server communication
    let signal = json!({
        "from": "n1",
        "to": "n2",
        "content": "Cross-server test",
        "signal_type": "test"
    });
    
    let resp = client
        .post("http://localhost:3000/api/neurons/n1/signal")
        .json(&signal)
        .send()
        .await
        .unwrap();
    
    assert_eq!(resp.status(), 200);
    
    handle1.abort();
    handle2.abort();
}

#[tokio::test]
async fn test_error_recovery() {
    let config = create_test_server_config();
    let server = HAL9Server::new(config).await.unwrap();
    
    let server_handle = tokio::spawn(async move {
        server.run().await.unwrap();
    });
    
    sleep(Duration::from_secs(1)).await;
    
    let client = Client::new();
    let base_url = "http://localhost:3000";
    
    // Test invalid neuron ID
    let resp = client
        .get(format!("{}/api/neurons/invalid-id", base_url))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 404);
    
    // Test malformed signal
    let bad_signal = json!({
        "invalid_field": "test"
    });
    
    let resp = client
        .post(format!("{}/api/neurons/test/signal", base_url))
        .json(&bad_signal)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 400);
    
    // Test rate limiting
    for _ in 0..100 {
        let resp = client
            .get(format!("{}/api/neurons", base_url))
            .send()
            .await
            .unwrap();
        
        if resp.status() == 429 {
            // Rate limit hit
            break;
        }
    }
    
    server_handle.abort();
}

// Helper functions

fn create_test_server_config() -> ServerConfig {
    ServerConfig {
        server_id: "test-server".to_string(),
        host: "127.0.0.1".to_string(),
        port: 3000,
        database_url: "sqlite::memory:".to_string(),
        claude_mode: "mock".to_string(),
        monitoring_enabled: true,
        auth_enabled: false,
        ..Default::default()
    }
}

fn create_distributed_config(id: &str, port: u16) -> ServerConfig {
    ServerConfig {
        server_id: id.to_string(),
        host: "127.0.0.1".to_string(),
        port,
        database_url: format!("sqlite::{}.db", id),
        claude_mode: "mock".to_string(),
        discovery_enabled: true,
        ..Default::default()
    }
}

async fn create_test_neurons(client: &Client, base_url: &str) -> Vec<String> {
    let mut neurons = Vec::new();
    
    for i in 1..=9 {
        let id = format!("neuron-L{}", i);
        let layer = format!("L{}", i);
        create_neuron(client, base_url, &id, &layer).await;
        neurons.push(id);
    }
    
    neurons
}

async fn create_neuron(client: &Client, base_url: &str, id: &str, layer: &str) {
    let neuron = json!({
        "id": id,
        "layer": layer,
        "config": {
            "threshold": 0.5,
            "learning_rate": 0.1
        }
    });
    
    let resp = client
        .post(format!("{}/api/neurons", base_url))
        .json(&neuron)
        .send()
        .await
        .unwrap();
    
    assert_eq!(resp.status(), 201);
}

async fn process_hierarchical_signal(client: &Client, base_url: &str) -> String {
    let signal = json!({
        "from": "external",
        "to": "neuron-L9",
        "content": "What is the meaning of consciousness?",
        "signal_type": "query"
    });
    
    let resp = client
        .post(format!("{}/api/neurons/neuron-L9/signal", base_url))
        .json(&signal)
        .send()
        .await
        .unwrap();
    
    resp.text().await.unwrap()
}

async fn check_consciousness_emergence(client: &Client, base_url: &str) -> serde_json::Value {
    client
        .get(format!("{}/api/consciousness/metrics", base_url))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap()
}

async fn verify_self_organization(client: &Client, base_url: &str) -> serde_json::Value {
    client
        .get(format!("{}/api/network/topology", base_url))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap()
}

async fn send_random_signals(client: &Client, base_url: &str) {
    for i in 0..5 {
        let from_layer = (i % 9) + 1;
        let to_layer = ((i + 1) % 9) + 1;
        
        let signal = json!({
            "from": format!("neuron-L{}", from_layer),
            "to": format!("neuron-L{}", to_layer),
            "content": format!("test signal {}", i),
            "signal_type": "process"
        });
        
        let _ = client
            .post(format!("{}/api/signals", base_url))
            .json(&signal)
            .send()
            .await;
    }
}