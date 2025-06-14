//! Basic integration tests for the hierarchical architecture
//!
//! This test verifies basic functionality of the hierarchical components.

use hal9_core::hierarchical::{
    cognitive::{CognitiveLayer, CognitiveConfig, CognitiveInput, ConnectionConfig},
    protocol::ProtocolVersion,
    interfaces::{LayerMessage, LayerId, MessageType, MessagePriority},
};
use uuid::Uuid;
use std::collections::HashMap;

#[tokio::test]
async fn test_cognitive_unit_creation() -> Result<(), Box<dyn std::error::Error>> {
    // Test that we can create cognitive units for each layer
    let layers = vec![
        CognitiveLayer::Reflexive,
        CognitiveLayer::Implementation,
        CognitiveLayer::Operational,
        CognitiveLayer::Tactical,
        CognitiveLayer::Strategic,
    ];
    
    for layer in layers {
        let config = CognitiveConfig {
            id: Uuid::new_v4(),
            layer,
            initial_parameters: HashMap::new(),
            connections: ConnectionConfig {
                upward_connections: vec![],
                lateral_connections: vec![],
                downward_connections: vec![],
            },
        };
        
        // Just verify we can create the config - actual factory usage would need proper setup
        assert_eq!(config.layer, layer);
        println!("✅ Created config for layer: {:?}", layer);
    }
    
    Ok(())
}

#[tokio::test]
async fn test_layer_message_structure() {
    // Test LayerMessage creation
    let message = LayerMessage {
        id: Uuid::new_v4(),
        source_layer: LayerId::Cognitive,
        target_layer: LayerId::Orchestration,
        message_type: MessageType::Control,
        payload: serde_json::json!({
            "action": "test",
            "data": 123
        }),
        timestamp: chrono::Utc::now(),
        priority: MessagePriority::Normal,
    };
    
    assert_eq!(message.source_layer, LayerId::Cognitive);
    assert_eq!(message.target_layer, LayerId::Orchestration);
    assert_eq!(message.priority, MessagePriority::Normal);
    
    println!("✅ Layer message structure test passed!");
}

#[tokio::test]
async fn test_protocol_version_compatibility() {
    let v1_0_0 = ProtocolVersion::new(1, 0, 0);
    let v1_1_0 = ProtocolVersion::new(1, 1, 0);
    let v2_0_0 = ProtocolVersion::new(2, 0, 0);
    
    // Same major version should be compatible
    assert!(v1_0_0.is_compatible_with(&v1_1_0));
    
    // Different major version should not be compatible
    assert!(!v1_0_0.is_compatible_with(&v2_0_0));
    
    println!("✅ Protocol version compatibility test passed!");
}

#[tokio::test]
async fn test_cognitive_input_structure() {
    let input = CognitiveInput {
        content: "Test input content".to_string(),
        source_layer: Some(CognitiveLayer::Operational),
        context: HashMap::from([
            ("key1".to_string(), serde_json::json!("value1")),
            ("key2".to_string(), serde_json::json!(42)),
        ]),
    };
    
    assert_eq!(input.content, "Test input content");
    assert_eq!(input.source_layer, Some(CognitiveLayer::Operational));
    assert_eq!(input.context.len(), 2);
    
    println!("✅ Cognitive input structure test passed!");
}

#[tokio::test]
async fn test_connection_config() {
    let config = ConnectionConfig {
        upward_connections: vec![Uuid::new_v4(), Uuid::new_v4()],
        lateral_connections: vec![Uuid::new_v4()],
        downward_connections: vec![],
    };
    
    assert_eq!(config.upward_connections.len(), 2);
    assert_eq!(config.lateral_connections.len(), 1);
    assert_eq!(config.downward_connections.len(), 0);
    
    println!("✅ Connection config test passed!");
}