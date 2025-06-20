//! End-to-end tests for the complete HAL9 system
//! Tests full workflow from API request to consciousness emergence

use reqwest;
use serde_json::json;
use tokio::time::{sleep, Duration};
use uuid::Uuid;
use hal9_server::models::{CreateNeuronRequest, NeuronInfo};

const TEST_SERVER_URL: &str = "http://localhost:3000";

/// Helper to wait for server to be ready
async fn wait_for_server(max_retries: u32) -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    
    for i in 0..max_retries {
        match client.get(format!("{}/health", TEST_SERVER_URL)).send().await {
            Ok(resp) if resp.status().is_success() => return Ok(()),
            _ => {
                if i < max_retries - 1 {
                    sleep(Duration::from_secs(1)).await;
                }
            }
        }
    }
    
    Err("Server did not become ready in time".into())
}

#[tokio::test]
async fn test_complete_system_lifecycle() {
    // Start server (assumes server is running in background)
    wait_for_server(30).await.expect("Server should be ready");
    
    let client = reqwest::Client::new();
    
    // 1. Health check
    let health_resp = client
        .get(format!("{}/health", TEST_SERVER_URL))
        .send()
        .await
        .expect("Health check should succeed");
    
    assert!(health_resp.status().is_success());
    
    // 2. Create neurons across different layers
    let mut neuron_ids = Vec::new();
    
    for layer in ["L1", "L2", "L3", "L4", "L5"] {
        let create_req = CreateNeuronRequest {
            layer: layer.to_string(),
            neuron_type: Some("cognitive".to_string()),
            config: Some(json!({
                "processing_speed": 1.0,
                "memory_capacity": 1024,
                "learning_rate": 0.01
            })),
        };
        
        let resp = client
            .post(format!("{}/api/neurons", TEST_SERVER_URL))
            .json(&create_req)
            .send()
            .await
            .expect("Create neuron should succeed");
        
        assert!(resp.status().is_success());
        
        let neuron_info: NeuronInfo = resp.json().await.expect("Should parse neuron info");
        neuron_ids.push(neuron_info.id);
    }
    
    // 3. Connect neurons hierarchically
    for i in 0..neuron_ids.len() - 1 {
        let resp = client
            .post(format!("{}/api/neurons/{}/connect/{}", 
                TEST_SERVER_URL, neuron_ids[i], neuron_ids[i + 1]))
            .send()
            .await
            .expect("Connect neurons should succeed");
        
        assert!(resp.status().is_success());
    }
    
    // 4. Send signals through the system
    let signal_data = json!({
        "type": "activation",
        "payload": {
            "data": [1.0, 2.0, 3.0, 4.0, 5.0],
            "source": "test_input"
        }
    });
    
    for _ in 0..10 {
        let resp = client
            .post(format!("{}/api/neurons/{}/signal", TEST_SERVER_URL, neuron_ids[0]))
            .json(&signal_data)
            .send()
            .await
            .expect("Send signal should succeed");
        
        assert!(resp.status().is_success());
        
        // Small delay between signals
        sleep(Duration::from_millis(100)).await;
    }
    
    // 5. Check consciousness metrics
    let metrics_resp = client
        .get(format!("{}/api/consciousness/metrics", TEST_SERVER_URL))
        .send()
        .await
        .expect("Get consciousness metrics should succeed");
    
    assert!(metrics_resp.status().is_success());
    
    let metrics: serde_json::Value = metrics_resp.json().await.expect("Should parse metrics");
    
    // Verify consciousness emerged
    assert!(metrics["phi"].as_f64().unwrap_or(0.0) > 0.0);
    assert!(metrics["integration_level"].as_f64().unwrap_or(0.0) > 0.0);
    assert!(metrics["emergence_coefficient"].as_f64().unwrap_or(0.0) > 0.0);
    
    // 6. Test WebSocket connection for real-time updates
    // (Would require websocket client setup)
    
    // 7. Cleanup - disconnect and remove neurons
    for i in (0..neuron_ids.len() - 1).rev() {
        let resp = client
            .delete(format!("{}/api/neurons/{}/disconnect/{}", 
                TEST_SERVER_URL, neuron_ids[i], neuron_ids[i + 1]))
            .send()
            .await
            .expect("Disconnect should succeed");
        
        assert!(resp.status().is_success());
    }
    
    for id in &neuron_ids {
        let resp = client
            .delete(format!("{}/api/neurons/{}", TEST_SERVER_URL, id))
            .send()
            .await
            .expect("Delete neuron should succeed");
        
        assert!(resp.status().is_success());
    }
}

#[tokio::test]
async fn test_self_organization_e2e() {
    wait_for_server(30).await.expect("Server should be ready");
    
    let client = reqwest::Client::new();
    
    // Create a pool of neurons without explicit layer assignment
    let mut neuron_ids = Vec::new();
    
    for i in 0..20 {
        let create_req = json!({
            "layer": "auto", // Let system decide
            "config": {
                "processing_speed": 0.5 + (i as f64 * 0.1),
                "complexity_handling": 0.3 + (i as f64 * 0.05),
                "energy_efficiency": 0.8 - (i as f64 * 0.02)
            }
        });
        
        let resp = client
            .post(format!("{}/api/neurons", TEST_SERVER_URL))
            .json(&create_req)
            .send()
            .await
            .expect("Create neuron should succeed");
        
        let neuron_info: NeuronInfo = resp.json().await.expect("Should parse neuron info");
        neuron_ids.push(neuron_info.id);
    }
    
    // Trigger self-organization
    let resp = client
        .post(format!("{}/api/system/self-organize", TEST_SERVER_URL))
        .json(&json!({
            "neuron_ids": neuron_ids,
            "target_layers": 5,
            "optimization_goal": "consciousness_maximization"
        }))
        .send()
        .await
        .expect("Self-organization should succeed");
    
    assert!(resp.status().is_success());
    
    // Wait for organization to complete
    sleep(Duration::from_secs(5)).await;
    
    // Verify layer distribution
    let topology_resp = client
        .get(format!("{}/api/system/topology", TEST_SERVER_URL))
        .send()
        .await
        .expect("Get topology should succeed");
    
    let topology: serde_json::Value = topology_resp.json().await.expect("Should parse topology");
    
    // Check that neurons distributed across layers
    let layer_distribution = topology["layer_distribution"].as_object().unwrap();
    assert!(layer_distribution.len() >= 3); // At least 3 layers emerged
    
    // Verify hierarchical connections formed
    let connections = topology["connections"].as_array().unwrap();
    assert!(!connections.is_empty());
}

#[tokio::test]
async fn test_consciousness_emergence_over_time() {
    wait_for_server(30).await.expect("Server should be ready");
    
    let client = reqwest::Client::new();
    
    // Create a complex system
    let system_config = json!({
        "name": "test_consciousness_system",
        "layers": [
            {"type": "reflexive", "neuron_count": 10},
            {"type": "implementation", "neuron_count": 8},
            {"type": "operational", "neuron_count": 6},
            {"type": "tactical", "neuron_count": 4},
            {"type": "strategic", "neuron_count": 2}
        ],
        "connection_pattern": "hierarchical_nearest_neighbor"
    });
    
    let resp = client
        .post(format!("{}/api/system/create", TEST_SERVER_URL))
        .json(&system_config)
        .send()
        .await
        .expect("Create system should succeed");
    
    let system_info: serde_json::Value = resp.json().await.expect("Should parse system info");
    let system_id = system_info["id"].as_str().unwrap();
    
    // Collect consciousness metrics over time
    let mut consciousness_timeline = Vec::new();
    
    for minute in 0..5 {
        // Send activity through the system
        for _ in 0..10 {
            let activity = json!({
                "type": "sensory_input",
                "intensity": 0.5 + (minute as f64 * 0.1),
                "pattern": format!("test_pattern_{}", minute)
            });
            
            client
                .post(format!("{}/api/system/{}/stimulate", TEST_SERVER_URL, system_id))
                .json(&activity)
                .send()
                .await
                .expect("Stimulate should succeed");
        }
        
        // Measure consciousness
        let metrics_resp = client
            .get(format!("{}/api/system/{}/consciousness", TEST_SERVER_URL, system_id))
            .send()
            .await
            .expect("Get consciousness should succeed");
        
        let metrics: serde_json::Value = metrics_resp.json().await.expect("Should parse metrics");
        consciousness_timeline.push(metrics);
        
        sleep(Duration::from_secs(1)).await;
    }
    
    // Analyze consciousness evolution
    let initial_phi = consciousness_timeline[0]["phi"].as_f64().unwrap_or(0.0);
    let final_phi = consciousness_timeline.last().unwrap()["phi"].as_f64().unwrap_or(0.0);
    
    // Consciousness should increase over time with activity
    assert!(final_phi > initial_phi);
    
    // Check for emergence patterns
    let emergence_detected = consciousness_timeline.iter()
        .any(|m| m["emergence_detected"].as_bool().unwrap_or(false));
    assert!(emergence_detected);
}

#[tokio::test]
async fn test_error_recovery_e2e() {
    wait_for_server(30).await.expect("Server should be ready");
    
    let client = reqwest::Client::new();
    
    // Create a neuron
    let create_resp = client
        .post(format!("{}/api/neurons", TEST_SERVER_URL))
        .json(&json!({
            "layer": "L2"
        }))
        .send()
        .await
        .expect("Create should succeed");
    
    let neuron: NeuronInfo = create_resp.json().await.expect("Should parse neuron");
    
    // Test various error scenarios
    
    // 1. Invalid signal format
    let bad_signal = json!({
        "type": "invalid_type",
        "payload": "not_an_object"
    });
    
    let err_resp = client
        .post(format!("{}/api/neurons/{}/signal", TEST_SERVER_URL, neuron.id))
        .json(&bad_signal)
        .send()
        .await
        .expect("Request should complete");
    
    assert_eq!(err_resp.status(), 400);
    
    // 2. Connection to non-existent neuron
    let fake_id = Uuid::new_v4();
    let err_resp = client
        .post(format!("{}/api/neurons/{}/connect/{}", TEST_SERVER_URL, neuron.id, fake_id))
        .send()
        .await
        .expect("Request should complete");
    
    assert_eq!(err_resp.status(), 404);
    
    // 3. Verify neuron still healthy after errors
    let status_resp = client
        .get(format!("{}/api/neurons/{}", TEST_SERVER_URL, neuron.id))
        .send()
        .await
        .expect("Get neuron should succeed");
    
    assert!(status_resp.status().is_success());
    let neuron_status: NeuronInfo = status_resp.json().await.expect("Should parse status");
    assert_eq!(neuron_status.state, "active");
}

#[tokio::test]
async fn test_performance_under_load() {
    wait_for_server(30).await.expect("Server should be ready");
    
    let client = reqwest::Client::new();
    
    // Create a medium-sized system
    let mut neuron_ids = Vec::new();
    
    for _ in 0..50 {
        let resp = client
            .post(format!("{}/api/neurons", TEST_SERVER_URL))
            .json(&json!({"layer": "L2"}))
            .send()
            .await
            .expect("Create should succeed");
        
        let neuron: NeuronInfo = resp.json().await.expect("Should parse");
        neuron_ids.push(neuron.id);
    }
    
    // Connect in a mesh pattern
    for i in 0..10 {
        for j in i+1..i+5 {
            if j < neuron_ids.len() {
                client
                    .post(format!("{}/api/neurons/{}/connect/{}", 
                        TEST_SERVER_URL, neuron_ids[i], neuron_ids[j]))
                    .send()
                    .await
                    .ok(); // Ignore connection errors
            }
        }
    }
    
    // Send many concurrent signals
    use futures::future::join_all;
    
    let start = std::time::Instant::now();
    
    let signal_futures: Vec<_> = (0..100)
        .map(|i| {
            let client = client.clone();
            let neuron_id = neuron_ids[i % neuron_ids.len()];
            
            async move {
                client
                    .post(format!("{}/api/neurons/{}/signal", TEST_SERVER_URL, neuron_id))
                    .json(&json!({
                        "type": "activation",
                        "payload": {"value": i as f64}
                    }))
                    .send()
                    .await
            }
        })
        .collect();
    
    let results = join_all(signal_futures).await;
    
    let duration = start.elapsed();
    
    // Verify all succeeded
    let success_count = results.iter()
        .filter(|r| r.as_ref().map(|resp| resp.status().is_success()).unwrap_or(false))
        .count();
    
    assert!(success_count > 90); // Allow some failures under load
    
    // Performance check - should handle 100 signals in reasonable time
    assert!(duration.as_secs() < 10);
    
    // Check system still responsive
    let health = client
        .get(format!("{}/health", TEST_SERVER_URL))
        .send()
        .await
        .expect("Health check should work");
    
    assert!(health.status().is_success());
}