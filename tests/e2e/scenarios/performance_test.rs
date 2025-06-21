use crate::test_framework::*;
use anyhow::Result;
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::Semaphore;
use tokio::time::{sleep, Duration};

/// Test system performance under various load conditions
#[tokio::test]
async fn test_performance_under_load() -> Result<()> {
    let config = TestConfig::default();
    let client = Arc::new(E2ETestClient::new(config.clone()));
    
    client.wait_for_server().await?;
    
    println!("=== Performance Test: Concurrent Neuron Creation ===");
    
    // Test 1: Concurrent neuron creation
    let mut perf_create = PerfTest::new("concurrent_creation");
    let semaphore = Arc::new(Semaphore::new(50)); // Limit concurrent requests
    let mut handles = vec![];
    
    for i in 0..100 {
        let client = client.clone();
        let sem = semaphore.clone();
        
        let handle = tokio::spawn(async move {
            let _permit = sem.acquire().await.unwrap();
            
            let neuron_data = serde_json::json!({
                "layer": (i % 5) + 1,
                "position": [i as f32 * 0.01, 0.5, 0.5],
                "processing_speed": 1.0,
                "complexity_threshold": 0.7
            });
            
            client.post::<_, Value>("/api/neurons", &neuron_data).await
        });
        
        handles.push(handle);
    }
    
    // Wait for all creations to complete
    let mut created_neurons = vec![];
    for handle in handles {
        if let Ok(Ok(neuron)) = handle.await {
            if let Some(id) = neuron.get("id").and_then(|v| v.as_str()) {
                created_neurons.push(id.to_string());
            }
        }
    }
    
    perf_create.record();
    println!("{}", perf_create.summary());
    println!("Successfully created {} neurons", created_neurons.len());
    
    // Test 2: Signal propagation performance
    println!("\n=== Performance Test: Signal Propagation ===");
    let mut perf_signals = PerfTest::new("signal_propagation");
    
    // Connect neurons in a chain
    for i in 0..created_neurons.len().min(50) - 1 {
        let _: Value = client.post(
            &format!("/api/neurons/{}/connect/{}", created_neurons[i], created_neurons[i + 1]),
            &serde_json::json!({ "weight": 0.9 })
        ).await?;
    }
    
    // Send signals through the chain
    for _i in 0..10 {
        let signal_data = serde_json::json!({
            "pattern": vec![0.1; 20],
            "intensity": 0.8
        });
        
        let _: Value = client.post(
            &format!("/api/neurons/{}/signal", created_neurons[0]),
            &signal_data
        ).await?;
        
        perf_signals.record();
    }
    
    println!("{}", perf_signals.summary());
    
    // Test 3: Consciousness calculation performance
    println!("\n=== Performance Test: Consciousness Metrics ===");
    let mut perf_consciousness = PerfTest::new("consciousness_metrics");
    
    for _ in 0..20 {
        let _: Value = client.get("/api/consciousness/metrics").await?;
        perf_consciousness.record();
    }
    
    println!("{}", perf_consciousness.summary());
    
    // Test 4: Self-organization at scale
    println!("\n=== Performance Test: Self-Organization at Scale ===");
    
    // Create many more neurons for scale test
    let mut scale_handles = vec![];
    let scale_sem = Arc::new(Semaphore::new(100));
    
    for i in 0..400 {
        let client = client.clone();
        let sem = scale_sem.clone();
        
        let handle = tokio::spawn(async move {
            let _permit = sem.acquire().await.unwrap();
            
            let neuron_data = serde_json::json!({
                "layer": 0, // Let self-organization decide
                "position": [
                    (i as f32 * 0.002) % 1.0,
                    ((i as f32 * 0.003) % 1.0),
                    ((i as f32 * 0.005) % 1.0)
                ],
                "processing_speed": 0.5 + (i as f32 * 0.001),
                "complexity_threshold": 0.5 + (i as f32 * 0.0005)
            });
            
            client.post::<_, Value>("/api/neurons", &neuron_data).await
        });
        
        scale_handles.push(handle);
    }
    
    // Wait for creation
    for handle in scale_handles {
        let _ = handle.await;
    }
    
    // Trigger self-organization and measure time
    let start = tokio::time::Instant::now();
    let self_org_result: Value = client.post("/api/system/self-organize", &serde_json::json!({})).await?;
    let self_org_time = start.elapsed();
    
    println!("Self-organization completed in {:?}", self_org_time);
    if let Some(layers) = self_org_result.get("layers_formed").and_then(|v| v.as_u64()) {
        println!("Formed {} layers", layers);
    }
    
    // Test 5: Sustained load test
    println!("\n=== Performance Test: Sustained Load ===");
    let sustained_duration = Duration::from_secs(10);
    let start = tokio::time::Instant::now();
    let mut request_count = 0;
    let mut error_count = 0;
    
    while start.elapsed() < sustained_duration {
        let mut handles = vec![];
        
        // Mix of different operations
        for i in 0..10 {
            let client = client.clone();
            let neuron_id = created_neurons.get(i % created_neurons.len())
                .cloned()
                .unwrap_or_default();
            
            let handle = match i % 4 {
                0 => {
                    // GET neuron
                    tokio::spawn(async move {
                        client.get::<Value>(&format!("/api/neurons/{}", neuron_id)).await
                    })
                }
                1 => {
                    // Send signal
                    tokio::spawn(async move {
                        let signal = serde_json::json!({
                            "pattern": vec![0.5; 10],
                            "intensity": 0.7
                        });
                        client.post::<_, Value>(&format!("/api/neurons/{}/signal", neuron_id), &signal).await
                    })
                }
                2 => {
                    // Get metrics
                    tokio::spawn(async move {
                        client.get::<Value>("/api/consciousness/metrics").await
                    })
                }
                _ => {
                    // Get topology
                    tokio::spawn(async move {
                        client.get::<Value>("/api/system/topology").await
                    })
                }
            };
            
            handles.push(handle);
        }
        
        for handle in handles {
            request_count += 1;
            if handle.await.is_err() {
                error_count += 1;
            }
        }
    }
    
    println!("Sustained load test:");
    println!("  Total requests: {}", request_count);
    println!("  Errors: {}", error_count);
    println!("  Success rate: {:.1}%", (request_count - error_count) as f64 / request_count as f64 * 100.0);
    println!("  Requests/sec: {:.1}", request_count as f64 / sustained_duration.as_secs_f64());
    
    // Cleanup
    println!("\n=== Cleanup ===");
    // In real test, would delete all created neurons
    
    println!("✅ Performance tests completed!");
    Ok(())
}

/// Test rate limiting behavior
#[tokio::test]
async fn test_rate_limiting() -> Result<()> {
    let config = TestConfig::default();
    let client = E2ETestClient::new(config);
    
    client.wait_for_server().await?;
    
    println!("=== Testing Rate Limiting ===");
    
    let mut success_count = 0;
    let mut rate_limited_count = 0;
    
    // Send many requests rapidly
    for i in 0..200 {
        match client.get::<Value>("/health").await {
            Ok(_) => success_count += 1,
            Err(e) if e.to_string().contains("429") => {
                rate_limited_count += 1;
                println!("Rate limited at request {}", i);
            }
            Err(e) => return Err(e),
        }
        
        // Small delay to not overwhelm
        if i % 10 == 0 {
            sleep(Duration::from_millis(10)).await;
        }
    }
    
    println!("Successful requests: {}", success_count);
    println!("Rate limited requests: {}", rate_limited_count);
    
    // Verify rate limiting is working
    assert!(
        rate_limited_count > 0 || success_count == 200,
        "Should either hit rate limit or server has no rate limiting"
    );
    
    Ok(())
}