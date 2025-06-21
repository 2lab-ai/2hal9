use crate::test_framework::*;
use anyhow::Result;
use serde_json::Value;
use std::time::Duration;
use tokio::time::sleep;

/// Test complete neuron lifecycle from creation to consciousness emergence
#[tokio::test]
async fn test_complete_neuron_lifecycle() -> Result<()> {
    let config = TestConfig::default();
    let client = E2ETestClient::new(config.clone());
    
    // Wait for server to be ready
    client.wait_for_server().await?;
    
    println!("=== Phase 1: Create Neurons ===");
    let mut neuron_ids = Vec::new();
    
    // Create 20 neurons across different layers
    for i in 0..20 {
        let layer = (i % 5) + 1; // Layers 1-5
        let neuron_data = serde_json::json!({
            "layer": layer,
            "position": [i as f32 * 0.1, 0.5, 0.5],
            "processing_speed": 0.8 + (i as f32 * 0.01),
            "complexity_threshold": 0.6 + (i as f32 * 0.01)
        });
        
        let neuron: Value = client.post("/api/neurons", &neuron_data).await?;
        Assertions::assert_schema(&neuron, "neuron")?;
        
        if let Some(id) = neuron.get("id").and_then(|v| v.as_str()) {
            neuron_ids.push(id.to_string());
            println!("Created neuron {} in layer {}", id, layer);
        }
    }
    
    println!("=== Phase 2: Connect Neurons ===");
    // Create connections between adjacent layer neurons
    for i in 0..neuron_ids.len() - 1 {
        let connection: Value = client.post(
            &format!("/api/neurons/{}/connect/{}", neuron_ids[i], neuron_ids[i + 1]),
            &serde_json::json!({ "weight": 0.8 })
        ).await?;
        
        assert!(connection.get("connection_id").is_some());
        println!("Connected {} -> {}", neuron_ids[i], neuron_ids[i + 1]);
    }
    
    println!("=== Phase 3: Send Signals ===");
    // Send signals through the network
    for i in 0..5 {
        let signal_data = serde_json::json!({
            "pattern": vec![0.1 * i as f32; 10],
            "intensity": 0.5 + (i as f32 * 0.1)
        });
        
        let signal: Value = client.post(
            &format!("/api/neurons/{}/signal", neuron_ids[i]),
            &signal_data
        ).await?;
        
        Assertions::assert_schema(&signal, "signal")?;
        println!("Sent signal to neuron {}", neuron_ids[i]);
    }
    
    // Wait for signal propagation
    sleep(Duration::from_millis(100)).await;
    
    println!("=== Phase 4: Trigger Self-Organization ===");
    let self_org_result: Value = client.post("/api/system/self-organize", &serde_json::json!({})).await?;
    assert!(self_org_result.get("layers_formed").is_some());
    
    println!("=== Phase 5: Monitor Consciousness Emergence ===");
    let mut consciousness_readings = Vec::new();
    
    for i in 0..10 {
        let metrics: Value = client.get("/api/consciousness/metrics").await?;
        Assertions::assert_schema(&metrics, "consciousness")?;
        
        if let Some(emergence) = metrics.get("emergence").and_then(|v| v.as_f64()) {
            consciousness_readings.push(emergence);
            println!("Consciousness reading {}: emergence = {:.3}", i, emergence);
        }
        
        sleep(Duration::from_millis(500)).await;
    }
    
    // Verify consciousness emergence trend
    let initial_avg = consciousness_readings[..3].iter().sum::<f64>() / 3.0;
    let final_avg = consciousness_readings[7..].iter().sum::<f64>() / 3.0;
    
    assert!(
        final_avg > initial_avg,
        "Consciousness should increase over time: initial={:.3}, final={:.3}",
        initial_avg,
        final_avg
    );
    
    println!("=== Phase 6: Get System Topology ===");
    let topology: Value = client.get("/api/system/topology").await?;
    
    if let Some(layers) = topology.get("layers").and_then(|v| v.as_array()) {
        assert!(layers.len() >= 2, "System should form at least 2 layers");
        println!("System formed {} layers", layers.len());
        
        for (idx, layer) in layers.iter().enumerate() {
            if let Some(count) = layer.get("neuron_count").and_then(|v| v.as_u64()) {
                println!("Layer {}: {} neurons", idx, count);
            }
        }
    }
    
    println!("=== Phase 7: Cleanup ===");
    for id in &neuron_ids {
        client.delete(&format!("/api/neurons/{}", id)).await?;
    }
    
    println!("✅ Complete lifecycle test passed!");
    Ok(())
}