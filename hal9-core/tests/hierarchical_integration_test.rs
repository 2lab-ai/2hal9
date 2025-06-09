//! Integration tests for the hierarchical architecture
//!
//! This test verifies that messages can flow through all 5 layers
//! of the hierarchical architecture correctly.

use hal9_core::hierarchical::{
    substrate::{TokioRuntime, ChannelTransport},
    protocol::{SignalProtocol, ProtocolManager, ProtocolVersion, ProtocolManagerConfig},
    cognitive::{CognitiveFactory, CognitiveConfig, CognitiveLayer, CognitiveInput, ConnectionConfig},
    orchestration::{BasicOrchestrator, UnitDescriptor, UnitType, ResourceRequirements, Capability},
    interfaces::{LayerMessage, LayerId, MessageType},
};
use uuid::Uuid;
use std::collections::HashMap;
use std::sync::Arc;

#[tokio::test]
async fn test_hierarchical_message_flow() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize runtime
    let runtime = TokioRuntime::new();
    
    // Create transport
    let transport = Arc::new(ChannelTransport::new(1024));
    
    // Create protocol manager
    let config = ProtocolManagerConfig {
        max_connections: 100,
        protocol_timeout: std::time::Duration::from_secs(30),
        enable_compression: true,
        enable_encryption: false,
    };
    let mut protocol_manager = ProtocolManager::new(config, transport.clone());
    let signal_protocol = Arc::new(SignalProtocol::new(transport.clone()));
    protocol_manager.register_protocol("signal-protocol", signal_protocol)?;
    
    // Create cognitive factory
    let factory = CognitiveFactory::default();
    
    // Create orchestrator
    let mut orchestrator = BasicOrchestrator::new();
    orchestrator.initialize().await?;
    
    // Create neurons for each layer
    let layers = vec![
        CognitiveLayer::Reflexive,
        CognitiveLayer::Implementation,
        CognitiveLayer::Operational,
        CognitiveLayer::Tactical,
        CognitiveLayer::Strategic,
    ];
    
    let mut unit_ids = Vec::new();
    
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
        
        let unit = factory.create(layer, config).await?;
        let unit_id = orchestrator.add_unit(UnitDescriptor {
            id: Uuid::new_v4(),
            unit_type: UnitType::Cognitive,
            layer,
            capabilities: vec![],
            resource_requirements: ResourceRequirements {
                min_memory_mb: 100,
                min_cpu_cores: 0.1,
                requires_gpu: false,
                network_bandwidth_mbps: None,
            },
        }).await?;
        
        unit_ids.push(unit_id);
    }
    
    // Connect layers in hierarchy
    for i in 0..unit_ids.len() - 1 {
        orchestrator.connect(
            unit_ids[i], 
            unit_ids[i + 1],
            Default::default()
        ).await?;
    }
    
    // Create a test message
    let test_message = LayerMessage {
        id: Uuid::new_v4(),
        source_layer: LayerId::Substrate,
        target_layer: LayerId::Intelligence,
        message_type: MessageType::Data,
        payload: serde_json::json!({
            "content": "Test hierarchical message flow",
            "strength": 0.8
        }),
        timestamp: chrono::Utc::now(),
        priority: 1.0,
    };
    
    // Route the message through the hierarchy
    let routing_signal = hal9_core::hierarchical::orchestration::OrchestrationSignal {
        id: test_message.id,
        source: unit_ids[0],
        signal_type: hal9_core::hierarchical::orchestration::SignalType::Activation,
        priority: 1.0,
        payload: serde_json::to_value(&test_message)?,
        routing_hints: hal9_core::hierarchical::orchestration::RoutingHints {
            preferred_path: None,
            avoid_units: vec![],
            max_hops: None,
            qos_requirements: hal9_core::hierarchical::orchestration::QosRequirements {
                max_latency_ms: Some(1000.0),
                min_reliability: Some(0.9),
                ordered_delivery: false,
            },
        },
    };
    
    let route = orchestrator.route(routing_signal).await?;
    
    // Verify the message was routed through all layers
    assert_eq!(route.len(), unit_ids.len());
    
    // Get topology snapshot
    let topology = orchestrator.topology().await?;
    assert_eq!(topology.total_units, unit_ids.len());
    assert_eq!(topology.total_connections, unit_ids.len() - 1);
    
    println!("✅ Hierarchical message flow test passed!");
    println!("   - Created {} cognitive units", unit_ids.len());
    println!("   - Established {} connections", topology.total_connections);
    println!("   - Message routed through {} layers", route.len());
    
    Ok(())
}

#[tokio::test]
async fn test_protocol_negotiation() -> Result<(), Box<dyn std::error::Error>> {
    use hal9_core::hierarchical::protocol::{
        Protocol, ProtocolCapabilities, CompressionType, EncryptionType
    };
    
    // Create transport
    let transport = Arc::new(ChannelTransport::new(1024));
    
    // Create two protocol instances
    let protocol_v1 = SignalProtocol::new(transport.clone());
    let protocol_v2 = SignalProtocol::new(transport.clone());
    
    // Test version compatibility
    assert!(protocol_v1.version().is_compatible_with(&protocol_v2.version()));
    
    // Test capability negotiation
    let caps = ProtocolCapabilities {
        compression: vec![CompressionType::None, CompressionType::Gzip],
        encryption: vec![EncryptionType::None],
        max_message_size: 1024 * 1024,
        streaming: true,
        bidirectional: true,
        ordered_delivery: true,
    };
    
    let negotiated = protocol_v1.negotiate(&caps).await?;
    assert_eq!(negotiated.compression, CompressionType::None);
    assert_eq!(negotiated.encryption, EncryptionType::None);
    
    println!("✅ Protocol negotiation test passed!");
    
    Ok(())
}

#[tokio::test]
async fn test_cognitive_layer_processing() -> Result<(), Box<dyn std::error::Error>> {
    use hal9_core::hierarchical::cognitive::{CognitiveUnit};
    
    // Create L2 Implementation neuron
    let config = CognitiveConfig {
        id: Uuid::new_v4(),
        layer: CognitiveLayer::Implementation,
        initial_parameters: HashMap::new(),
        connections: ConnectionConfig {
            upward_connections: vec![],
            lateral_connections: vec![],
            downward_connections: vec![],
        },
    };
    
    let factory = CognitiveFactory::default();
    let mut neuron = factory.create(CognitiveLayer::Implementation, config).await?;
    
    // Process a code generation request
    let input = CognitiveInput {
        content: "Generate a function that adds two numbers".to_string(),
        source_layer: Some(CognitiveLayer::Operational),
        context: HashMap::new(),
    };
    
    let output = neuron.process(input).await?;
    
    // Verify output
    assert!(output.content.contains("IMPLEMENTATION"));
    assert!(output.confidence > 0.0);
    assert_eq!(output.target_layers, vec![CognitiveLayer::Reflexive]);
    
    // Test introspection
    let state = neuron.introspect().await;
    assert_eq!(state.unit_id(), neuron.id());
    assert_eq!(state.layer(), CognitiveLayer::Implementation);
    
    println!("✅ Cognitive layer processing test passed!");
    
    Ok(())
}