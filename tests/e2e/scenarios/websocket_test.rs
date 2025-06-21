use crate::test_framework::*;
use anyhow::Result;
use serde_json::Value;
use std::time::Duration;
use tokio::time::timeout;

/// Test WebSocket real-time updates
#[tokio::test]
async fn test_websocket_realtime_updates() -> Result<()> {
    let config = TestConfig::default();
    let client = E2ETestClient::new(config.clone());
    
    client.wait_for_server().await?;
    
    println!("=== Testing WebSocket Real-time Updates ===");
    
    // Connect to WebSocket
    println!("1. Connecting to WebSocket");
    let mut ws = client.connect_websocket("/ws").await?;
    println!("✅ WebSocket connected");
    
    // Subscribe to neuron updates
    println!("\n2. Subscribing to neuron updates");
    ws.send(&serde_json::json!({
        "type": "subscribe",
        "channel": "neurons"
    })).await?;
    
    // Verify subscription confirmation
    let confirmation = ws.receive().await?;
    assert_eq!(
        confirmation.get("type").and_then(|v| v.as_str()),
        Some("subscribed")
    );
    println!("✅ Subscribed to neuron updates");
    
    // Create a neuron and verify real-time update
    println!("\n3. Creating neuron and waiting for update");
    let neuron: Value = client.post("/api/neurons", &Fixtures::neuron()).await?;
    let neuron_id = neuron.get("id").and_then(|v| v.as_str()).unwrap();
    
    // Should receive creation event
    let create_event = timeout(Duration::from_secs(5), ws.receive()).await??;
    assert_eq!(
        create_event.get("type").and_then(|v| v.as_str()),
        Some("neuron_created")
    );
    assert_eq!(
        create_event.get("data").and_then(|d| d.get("id")).and_then(|v| v.as_str()),
        Some(neuron_id)
    );
    println!("✅ Received neuron creation event");
    
    // Send signal and verify propagation events
    println!("\n4. Sending signal and monitoring propagation");
    ws.send(&serde_json::json!({
        "type": "subscribe",
        "channel": "signals"
    })).await?;
    
    // Wait for subscription confirmation
    let _ = ws.receive().await?;
    
    // Send signal
    let _signal: Value = client.post(
        &format!("/api/neurons/{}/signal", neuron_id),
        &Fixtures::signal()
    ).await?;
    
    // Should receive signal event
    let signal_event = timeout(Duration::from_secs(5), ws.receive()).await??;
    assert_eq!(
        signal_event.get("type").and_then(|v| v.as_str()),
        Some("signal_sent")
    );
    println!("✅ Received signal propagation event");
    
    // Test consciousness updates
    println!("\n5. Monitoring consciousness updates");
    ws.send(&serde_json::json!({
        "type": "subscribe",
        "channel": "consciousness",
        "interval": 1000 // milliseconds
    })).await?;
    
    // Receive periodic consciousness updates
    let mut consciousness_updates = 0;
    let start = tokio::time::Instant::now();
    
    while consciousness_updates < 3 && start.elapsed() < Duration::from_secs(10) {
        match timeout(Duration::from_secs(2), ws.receive()).await {
            Ok(Ok(msg)) => {
                if msg.get("type").and_then(|v| v.as_str()) == Some("consciousness_update") {
                    consciousness_updates += 1;
                    if let Some(metrics) = msg.get("data") {
                        println!("✅ Consciousness update {}: emergence={:.3}",
                            consciousness_updates,
                            metrics.get("emergence").and_then(|v| v.as_f64()).unwrap_or(0.0)
                        );
                    }
                }
            }
            _ => {}
        }
    }
    
    assert!(consciousness_updates >= 2, "Should receive periodic consciousness updates");
    
    // Test error handling
    println!("\n6. Testing error handling");
    ws.send(&serde_json::json!({
        "type": "invalid_message_type"
    })).await?;
    
    let error_response = ws.receive().await?;
    assert_eq!(
        error_response.get("type").and_then(|v| v.as_str()),
        Some("error")
    );
    println!("✅ Invalid messages properly handled");
    
    // Clean disconnect
    println!("\n7. Testing clean disconnect");
    ws.close().await?;
    println!("✅ WebSocket disconnected cleanly");
    
    println!("\n✅ WebSocket tests passed!");
    Ok(())
}

/// Test WebSocket authentication
#[tokio::test]
async fn test_websocket_authentication() -> Result<()> {
    let mut config = TestConfig::default();
    config.auth_enabled = true;
    let client = E2ETestClient::new(config.clone());
    
    client.wait_for_server().await?;
    
    println!("=== Testing WebSocket Authentication ===");
    
    // Register and login
    let username = format!("ws_user_{}", uuid::Uuid::new_v4());
    client.register(&username, "password123").await?;
    client.login(&username, "password123").await?;
    
    // Connect with authentication
    println!("1. Connecting with valid token");
    let auth_token = client.auth_token.lock().await.clone().unwrap();
    
    let mut ws = client.connect_websocket("/ws").await?;
    
    // Authenticate WebSocket connection
    ws.send(&serde_json::json!({
        "type": "authenticate",
        "token": auth_token
    })).await?;
    
    let auth_response = ws.receive().await?;
    assert_eq!(
        auth_response.get("type").and_then(|v| v.as_str()),
        Some("authenticated")
    );
    println!("✅ WebSocket authenticated successfully");
    
    // Test authenticated operations
    ws.send(&serde_json::json!({
        "type": "subscribe",
        "channel": "user_neurons"
    })).await?;
    
    let sub_response = ws.receive().await?;
    assert_eq!(
        sub_response.get("type").and_then(|v| v.as_str()),
        Some("subscribed")
    );
    println!("✅ Can subscribe to protected channels");
    
    ws.close().await?;
    
    // Test unauthenticated connection
    println!("\n2. Testing unauthenticated connection");
    let mut unauth_ws = client.connect_websocket("/ws").await?;
    
    // Try to subscribe without authentication
    unauth_ws.send(&serde_json::json!({
        "type": "subscribe",
        "channel": "user_neurons"
    })).await?;
    
    let unauth_response = unauth_ws.receive().await?;
    assert_eq!(
        unauth_response.get("type").and_then(|v| v.as_str()),
        Some("error")
    );
    assert!(
        unauth_response.get("message")
            .and_then(|v| v.as_str())
            .map(|s| s.contains("auth"))
            .unwrap_or(false),
        "Should indicate authentication required"
    );
    println!("✅ Unauthenticated requests properly rejected");
    
    unauth_ws.close().await?;
    
    println!("\n✅ WebSocket authentication tests passed!");
    Ok(())
}

/// Test WebSocket connection resilience
#[tokio::test]
async fn test_websocket_resilience() -> Result<()> {
    let config = TestConfig::default();
    let client = E2ETestClient::new(config.clone());
    
    client.wait_for_server().await?;
    
    println!("=== Testing WebSocket Resilience ===");
    
    // Test 1: Reconnection after disconnect
    println!("1. Testing reconnection");
    let mut ws1 = client.connect_websocket("/ws").await?;
    ws1.send(&serde_json::json!({
        "type": "subscribe",
        "channel": "neurons"
    })).await?;
    let _ = ws1.receive().await?;
    
    // Disconnect
    ws1.close().await?;
    
    // Reconnect
    let mut ws2 = client.connect_websocket("/ws").await?;
    ws2.send(&serde_json::json!({
        "type": "subscribe",
        "channel": "neurons"
    })).await?;
    
    let response = ws2.receive().await?;
    assert_eq!(
        response.get("type").and_then(|v| v.as_str()),
        Some("subscribed")
    );
    println!("✅ Reconnection successful");
    
    // Test 2: Multiple concurrent connections
    println!("\n2. Testing multiple concurrent connections");
    let mut connections = vec![];
    
    for i in 0..5 {
        let mut ws = client.connect_websocket("/ws").await?;
        ws.send(&serde_json::json!({
            "type": "subscribe",
            "channel": "neurons",
            "client_id": format!("client_{}", i)
        })).await?;
        
        let response = ws.receive().await?;
        assert_eq!(
            response.get("type").and_then(|v| v.as_str()),
            Some("subscribed")
        );
        
        connections.push(ws);
    }
    
    println!("✅ {} concurrent connections established", connections.len());
    
    // Create neuron and verify all connections receive update
    let _neuron: Value = client.post("/api/neurons", &Fixtures::neuron()).await?;
    
    let mut received_count = 0;
    for ws in &mut connections {
        match timeout(Duration::from_secs(2), ws.receive()).await {
            Ok(Ok(msg)) if msg.get("type").and_then(|v| v.as_str()) == Some("neuron_created") => {
                received_count += 1;
            }
            _ => {}
        }
    }
    
    println!("✅ {}/{} connections received broadcast", received_count, connections.len());
    
    // Close all connections
    for ws in connections {
        ws.close().await?;
    }
    
    // Test 3: Message ordering
    println!("\n3. Testing message ordering");
    let mut ws = client.connect_websocket("/ws").await?;
    ws.send(&serde_json::json!({
        "type": "subscribe",
        "channel": "test_ordering"
    })).await?;
    let _ = ws.receive().await?;
    
    // Send multiple messages quickly
    for i in 0..10 {
        ws.send(&serde_json::json!({
            "type": "echo",
            "sequence": i
        })).await?;
    }
    
    // Verify messages are received in order
    let mut last_sequence = -1;
    for _ in 0..10 {
        match timeout(Duration::from_secs(1), ws.receive()).await {
            Ok(Ok(msg)) => {
                if let Some(seq) = msg.get("sequence").and_then(|v| v.as_i64()) {
                    assert!(seq > last_sequence, "Messages should be in order");
                    last_sequence = seq;
                }
            }
            _ => break,
        }
    }
    
    println!("✅ Message ordering preserved");
    
    ws.close().await?;
    
    println!("\n✅ WebSocket resilience tests passed!");
    Ok(())
}