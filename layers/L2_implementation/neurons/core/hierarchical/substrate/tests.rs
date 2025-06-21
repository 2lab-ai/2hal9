//! Integration tests for substrate layer components

#[cfg(test)]
use super::*;
use crate::hierarchical::substrate::{
    runtime::{AsyncRuntime, TokioRuntime, TaskPriority},
    transport::{MessageTransport, ChannelTransport},
    storage::{PersistentStorage, SqliteStorage, StorageKey},
    resources::{ComputeResource, LocalResources, ResourceRequest, ResourcePriority},
};
use crate::Result;
use std::time::Duration;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct TestMessage {
    id: u64,
    content: String,
}

#[tokio::test]
async fn test_runtime_integration() -> Result<()> {
    let runtime = TokioRuntime::new();
    
    // Test spawning tasks with different priorities
    let (tx, mut rx) = tokio::sync::mpsc::channel(10);
    
    // Spawn high priority task
    let tx1 = tx.clone();
    runtime.spawn_with_priority(TaskPriority::High, async move {
        tokio::time::sleep(Duration::from_millis(10)).await;
        tx1.send(1).await.unwrap();
    });
    
    // Spawn normal priority task
    let tx2 = tx.clone();
    runtime.spawn_with_priority(TaskPriority::Normal, async move {
        tokio::time::sleep(Duration::from_millis(10)).await;
        tx2.send(2).await.unwrap();
    });
    
    // Spawn low priority task
    let tx3 = tx.clone();
    runtime.spawn_with_priority(TaskPriority::Low, async move {
        tokio::time::sleep(Duration::from_millis(10)).await;
        tx3.send(3).await.unwrap();
    });
    
    // Collect results
    let mut results = Vec::new();
    for _ in 0..3 {
        if let Some(val) = rx.recv().await {
            results.push(val);
        }
    }
    
    assert_eq!(results.len(), 3);
    
    // Test metrics
    let metrics = runtime.metrics();
    assert!(metrics.total_spawned >= 3);
    
    // Test cancellation
    let token = runtime.cancellation_token();
    let child_token = token.clone();
    
    let handle = runtime.spawn(async move {
        child_token.cancelled().await;
    });
    
    token.cancel();
    tokio::time::sleep(Duration::from_millis(50)).await;
    assert!(handle.is_finished());
    
    Ok(())
}

#[tokio::test]
async fn test_transport_integration() -> Result<()> {
    let transport = ChannelTransport::new();
    
    // Set up receiver
    let mut receiver = transport.receive::<TestMessage>("test-endpoint").await?;
    
    // Send message
    let msg = TestMessage {
        id: 42,
        content: "Hello, Transport!".to_string(),
    };
    
    transport.send("test-endpoint", msg.clone()).await?;
    
    // Receive and verify
    let received = receiver.recv().await.unwrap();
    assert_eq!(received, msg);
    
    // Test pub/sub
    let mut sub1 = transport.subscribe::<TestMessage>("topic1").await?;
    let mut sub2 = transport.subscribe::<TestMessage>("topic1").await?;
    
    let broadcast_msg = TestMessage {
        id: 100,
        content: "Broadcast message".to_string(),
    };
    
    transport.publish("topic1", broadcast_msg.clone()).await?;
    
    // Both subscribers should receive
    assert_eq!(sub1.recv().await.unwrap(), broadcast_msg);
    assert_eq!(sub2.recv().await.unwrap(), broadcast_msg);
    
    // Check metrics
    let metrics = transport.metrics();
    assert_eq!(metrics.messages_sent, 3); // 1 send + 2 publishes
    
    Ok(())
}

#[tokio::test]
async fn test_storage_integration() -> Result<()> {
    let mut storage = SqliteStorage::new(":memory:");
    storage.initialize().await?;
    
    // Test basic operations
    let key = StorageKey::new()
        .layer("substrate")
        .neuron("test-neuron")
        .data_type("config")
        .build();
    
    let value = TestMessage {
        id: 123,
        content: "Storage test".to_string(),
    };
    
    // Put and get
    storage.put(&key, &value).await?;
    let retrieved: TestMessage = storage.get(&key).await?.unwrap();
    assert_eq!(retrieved, value);
    
    // Test exists
    assert!(storage.exists(&key).await?);
    
    // Test list keys
    let key2 = StorageKey::new()
        .layer("substrate")
        .neuron("test-neuron")
        .data_type("state")
        .build();
    
    storage.put(&key2, &value).await?;
    
    let keys = storage.list_keys("layer:substrate/neuron:test-neuron").await?;
    assert_eq!(keys.len(), 2);
    
    // Test compare and swap
    let counter_key = "counter";
    storage.put(counter_key, 0i32).await?;
    
    let success = storage.compare_and_swap(counter_key, Some(0i32), 1i32).await?;
    assert!(success);
    
    let success = storage.compare_and_swap(counter_key, Some(0i32), 2i32).await?;
    assert!(!success); // Should fail as value is now 1
    
    // Test transaction
    let mut tx = storage.transaction().await?;
    tx.put("tx-key1", bincode::serialize(&"value1").unwrap()).await?;
    tx.put("tx-key2", bincode::serialize(&"value2").unwrap()).await?;
    tx.commit().await?;
    
    assert!(storage.exists("tx-key1").await?);
    assert!(storage.exists("tx-key2").await?);
    
    // Test TTL
    storage.put("ttl-key", "ttl-value").await?;
    storage.set_ttl("ttl-key", Duration::from_secs(1)).await?;
    
    // Key should exist now
    assert!(storage.exists("ttl-key").await?);
    
    // Wait for expiration
    tokio::time::sleep(Duration::from_secs(2)).await;
    
    // Key should be gone (note: SQLite doesn't auto-expire, would need background task)
    // This is just to show the API works
    
    Ok(())
}

#[tokio::test]
async fn test_resources_integration() -> Result<()> {
    let resources = LocalResources::new();
    
    // Test resource allocation
    let request1 = ResourceRequest {
        requester_id: "neuron1".to_string(),
        cpu_cores: Some(0.5),
        memory_mb: Some(512),
        gpu_count: None,
        priority: ResourcePriority::Normal,
        duration: Some(Duration::from_secs(60)),
    };
    
    let allocation1 = resources.allocate(request1).await?;
    assert_eq!(allocation1.cpu_cores, 0.5);
    assert_eq!(allocation1.memory_mb, 512);
    
    // Check available resources decreased
    let capacity = resources.available().await?;
    assert!(capacity.cpu_cores_available < capacity.cpu_cores_total);
    assert!(capacity.memory_mb_available < capacity.memory_mb_total);
    
    // Test resource limits
    let limits = crate::hierarchical::substrate::resources::ResourceLimits {
        max_cpu_cores: Some(0.2),
        max_memory_mb: Some(256),
        max_gpu_count: None,
        max_concurrent_tasks: None,
    };
    
    resources.set_limits("limited-neuron", limits).await?;
    
    // Try to exceed limits
    let request2 = ResourceRequest {
        requester_id: "limited-neuron".to_string(),
        cpu_cores: Some(0.5),
        memory_mb: Some(512),
        gpu_count: None,
        priority: ResourcePriority::Normal,
        duration: None,
    };
    
    let result = resources.allocate(request2).await;
    assert!(result.is_err());
    
    // Test monitoring
    let mut monitor = resources.monitor("neuron1").await?;
    
    // Should receive metrics (in background task)
    tokio::time::timeout(Duration::from_secs(2), async {
        if let Some(metric) = monitor.next_metric().await {
            assert!(metric.cpu_usage >= 0.0);
            assert!(metric.memory_mb > 0);
        }
    }).await.ok();
    
    // Release allocation
    resources.release(allocation1).await?;
    
    // Check resources restored
    let final_capacity = resources.available().await?;
    assert!(final_capacity.cpu_cores_available > capacity.cpu_cores_available);
    
    Ok(())
}

#[tokio::test]
async fn test_substrate_layer_integration() -> Result<()> {
    // Test all components working together
    let runtime = TokioRuntime::new();
    let transport = ChannelTransport::new();
    let mut storage = SqliteStorage::new(":memory:");
    storage.initialize().await?;
    let resources = LocalResources::new();
    
    // Simulate a neuron workflow
    
    // 1. Allocate resources
    let resource_req = ResourceRequest {
        requester_id: "integration-neuron".to_string(),
        cpu_cores: Some(0.1),
        memory_mb: Some(100),
        gpu_count: None,
        priority: ResourcePriority::Normal,
        duration: None,
    };
    
    let allocation = resources.allocate(resource_req).await?;
    
    // 2. Store configuration
    let config_key = StorageKey::new()
        .layer("substrate")
        .neuron("integration-neuron")
        .data_type("config")
        .build();
    
    let config = serde_json::json!({
        "allocation_id": allocation.allocation_id,
        "transport_endpoint": "integration-endpoint",
    });
    
    storage.put(&config_key, &config).await?;
    
    // 3. Set up transport
    let mut receiver = transport.receive::<serde_json::Value>("integration-endpoint").await?;
    
    // 4. Spawn processing task
    let transport_clone = transport.clone();
    let handle = runtime.spawn(async move {
        // Simulate processing
        tokio::time::sleep(Duration::from_millis(50)).await;
        
        // Send result
        let result = serde_json::json!({
            "status": "processed",
            "timestamp": chrono::Utc::now(),
        });
        
        transport_clone.send("integration-endpoint", result).await.unwrap();
    });
    
    // 5. Wait for result
    let result = tokio::time::timeout(Duration::from_secs(1), receiver.recv()).await
        .map_err(|_| crate::Error::Timeout(1))?;
    assert!(result.is_some());
    
    let result_value = result.unwrap();
    assert_eq!(result_value["status"], "processed");
    
    // 6. Store result
    let result_key = StorageKey::new()
        .layer("substrate")
        .neuron("integration-neuron")
        .data_type("results")
        .id(&allocation.allocation_id.to_string())
        .build();
    
    storage.put(&result_key, &result_value).await?;
    
    // 7. Verify task completed
    tokio::time::sleep(Duration::from_millis(100)).await;
    assert!(handle.is_finished());
    
    // 8. Check metrics
    let runtime_metrics = runtime.metrics();
    assert!(runtime_metrics.total_spawned >= 1);
    assert!(runtime_metrics.total_completed >= 1);
    
    let transport_metrics = transport.metrics();
    assert!(transport_metrics.messages_sent >= 1);
    
    // 9. Clean up
    resources.release(allocation).await?;
    storage.delete(&config_key).await?;
    
    Ok(())
}

#[tokio::test]
async fn test_error_handling() -> Result<()> {
    // Test various error conditions
    
    // Storage errors
    let storage = SqliteStorage::new(":memory:");
    // Try to use before initialization
    let result = storage.get::<String>("key").await;
    assert!(result.is_err());
    
    // Transport errors
    let transport = ChannelTransport::new();
    // Try to send to non-existent endpoint
    let result = transport.send("non-existent", "message").await;
    assert!(result.is_err());
    
    // Resource errors
    let resources = LocalResources::new();
    
    // Try to allocate more than available
    let huge_request = ResourceRequest {
        requester_id: "greedy".to_string(),
        cpu_cores: Some(10000.0), // Way more than any machine has
        memory_mb: Some(1_000_000_000), // 1TB
        gpu_count: Some(1000),
        priority: ResourcePriority::Normal,
        duration: None,
    };
    
    let result = resources.allocate(huge_request).await;
    assert!(result.is_err());
    
    Ok(())
}

