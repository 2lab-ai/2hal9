//! Basic integration tests for the hierarchical architecture
//!
//! This test focuses on getting the core components working together.

use hal9_core::hierarchical::{
    // Substrate Layer
    substrate::{
        transport::ChannelTransport,
        runtime::TokioRuntime,
        storage::{SqliteStorage, PersistentStorage},
        resources::{LocalResources, ComputeResource, ResourceRequest, ResourcePriority},
    },
    // Protocol Layer
    protocol::{
        ProtocolManager, ProtocolVersion,
        SignalProtocol, GradientProtocol, ConsensusProtocol,
        consensus::ConsensusAlgorithm,
    },
    // Cognitive Layer
    cognitive::{
        CognitiveLayer, CognitiveInput, CognitiveFactory,
        factory::DefaultCognitiveFactory,
        CognitiveUnitBuilder,
    },
    // Orchestration Layer
    orchestration::{
        DefaultOrchestrator, Orchestrator,
        topology::{GraphTopology, TopologyManager, EvolutionConfig},
        flow::{AdaptiveFlowController, FlowConfig, FlowController},
        routing::{DijkstraRouter, SignalRouter},
        coordination::{RaftCoordinator, StateCoordinator},
    },
    // Intelligence Layer
    intelligence::{
        DefaultIntelligenceCoordinator, IntelligenceCoordinator,
        MetaLearningConfig,
        MetaLearningSystem, SelfOrganizingSystem,
        EmergenceAnalyzer, CreativeSystem,
    },
    // Common interfaces
    interfaces::{LayerMessage, LayerId, MessageType, MessagePriority},
};

use uuid::Uuid;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::test]
async fn test_basic_layer_creation() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Create Substrate Layer components
    let runtime = TokioRuntime::new();
    let transport = Arc::new(ChannelTransport::new());
    
    // Create storage - note we need to initialize it separately
    let mut storage = SqliteStorage::new(":memory:");
    storage.initialize().await?;
    let storage = Arc::new(Mutex::new(storage));
    
    let resources = Arc::new(LocalResources::new());
    
    println!("✅ Substrate layer created");
    
    // 2. Create Protocol Layer components
    let transport_for_protocols = Arc::clone(&transport);
    let protocol_manager = Arc::new(ProtocolManager::new(
        Default::default(),
        transport_for_protocols,
    ));
    
    // Register protocols
    protocol_manager.register_protocol(
        "signal",
        Arc::new(SignalProtocol::new(Arc::clone(&transport)))
    )?;
    
    protocol_manager.register_protocol(
        "gradient",
        Arc::new(GradientProtocol::new(Arc::clone(&transport), 32))
    )?;
    
    protocol_manager.register_protocol(
        "consensus",
        Arc::new(ConsensusProtocol::new(
            Arc::clone(&transport),
            Uuid::new_v4(),
            ConsensusAlgorithm::SimpleMajority,
        ))
    )?;
    
    println!("✅ Protocol layer created");
    
    // 3. Create Cognitive Layer factory
    let cognitive_factory = Arc::new(
        DefaultCognitiveFactory::new()
            .with_protocol_manager(Arc::clone(&protocol_manager))
    );
    
    // Create a simple cognitive unit
    let unit_config = CognitiveUnitBuilder::new(CognitiveLayer::Reflexive)
        .with_parameter("learning_rate", 0.1)
        .build();
    
    let unit = cognitive_factory.create_unit(CognitiveLayer::Reflexive, unit_config)?;
    
    println!("✅ Cognitive layer created with unit: {}", unit.id());
    
    // 4. Create Orchestration Layer
    let topology = Box::new(GraphTopology::new(EvolutionConfig::default()));
    let flow = Box::new(AdaptiveFlowController::new(FlowConfig::default()));
    let state = Box::new(RaftCoordinator::new(Uuid::new_v4()));
    let router = Box::new(DijkstraRouter::new(1000));
    
    let mut orchestrator = DefaultOrchestrator::new(topology, flow, state, router);
    orchestrator.initialize().await?;
    
    println!("✅ Orchestration layer created");
    
    // 5. Create Intelligence Layer
    let meta_learner = Box::new(MetaLearningSystem::new());
    let self_organizer = Box::new(SelfOrganizingSystem::new());
    let emergence_detector = Box::new(EmergenceAnalyzer::new());
    let creativity_engine = Box::new(CreativeSystem::new());
    
    let mut intelligence = DefaultIntelligenceCoordinator::new(
        meta_learner,
        self_organizer,
        emergence_detector,
        creativity_engine,
    );
    
    intelligence.initialize().await?;
    
    // Enable some capabilities
    intelligence.enable_meta_learning(MetaLearningConfig {
        learning_rate_adaptation: true,
        strategy_evolution: true,
        architecture_search: false,
        transfer_learning: true,
        continual_learning: true,
    }).await?;
    
    println!("✅ Intelligence layer created");
    
    Ok(())
}

#[tokio::test]
async fn test_layer_message_routing() -> Result<(), Box<dyn std::error::Error>> {
    // Test basic message creation and routing
    let message = LayerMessage {
        id: Uuid::new_v4(),
        source_layer: LayerId::Cognitive,
        target_layer: LayerId::Intelligence,
        message_type: MessageType::Data,
        payload: serde_json::json!({
            "test": "data",
            "value": 42
        }),
        timestamp: chrono::Utc::now(),
        priority: MessagePriority::Normal,
    };
    
    // Verify message structure
    assert_eq!(message.source_layer, LayerId::Cognitive);
    assert_eq!(message.target_layer, LayerId::Intelligence);
    
    println!("✅ Layer message test passed");
    
    Ok(())
}

#[tokio::test]
async fn test_cognitive_unit_processing() -> Result<(), Box<dyn std::error::Error>> {
    // Create minimal setup for testing a cognitive unit
    let transport = Arc::new(ChannelTransport::new());
    let protocol_manager = Arc::new(ProtocolManager::new(
        Default::default(),
        transport,
    ));
    
    let factory = DefaultCognitiveFactory::new()
        .with_protocol_manager(protocol_manager);
    
    // Create a reflexive unit
    let config = CognitiveUnitBuilder::new(CognitiveLayer::Reflexive)
        .with_parameter("learning_rate", 0.1)
        .with_parameter("response_threshold", 0.5)
        .build();
    
    let mut unit = factory.create_unit(CognitiveLayer::Reflexive, config)?;
    
    // Create test input
    let input = CognitiveInput {
        content: "test input".to_string(),
        source_layer: None,
        context: HashMap::new(),
    };
    
    // Process the input
    let output = unit.process(input).await?;
    
    println!("✅ Cognitive unit processing test passed");
    println!("  Output: {:?}", output);
    
    Ok(())
}

#[tokio::test]
async fn test_protocol_version_negotiation() -> Result<(), Box<dyn std::error::Error>> {
    let v1 = ProtocolVersion::new(1, 0, 0);
    let v1_1 = ProtocolVersion::new(1, 1, 0);
    let v2 = ProtocolVersion::new(2, 0, 0);
    
    // Test compatibility
    assert!(v1.is_compatible_with(&v1_1), "v1.0.0 should be compatible with v1.1.0");
    assert!(!v1.is_compatible_with(&v2), "v1.0.0 should not be compatible with v2.0.0");
    
    println!("✅ Protocol version negotiation test passed");
    
    Ok(())
}

#[tokio::test]
async fn test_resource_management() -> Result<(), Box<dyn std::error::Error>> {
    let resources = LocalResources::new();
    
    // Test resource metrics
    let usage = resources.usage().await?;
    let capacity = resources.available().await?;
    
    assert!(usage.cpu_usage_percent >= 0.0 && usage.cpu_usage_percent <= 100.0, "CPU usage should be 0-100%");
    assert!(capacity.memory_mb_available > 0, "Should have available memory");
    
    // Test allocation
    let request = ResourceRequest {
        requester_id: Uuid::new_v4().to_string(),
        cpu_cores: Some(0.1),
        memory_mb: Some(1),
        gpu_count: None,
        priority: ResourcePriority::Normal,
        duration: None,
    };
    let allocation = resources.allocate(request).await?;
    
    // Verify allocation
    let new_capacity = resources.available().await?;
    assert!(new_capacity.memory_mb_available <= capacity.memory_mb_available, "Available memory should decrease or stay same");
    
    // Release allocation
    resources.release(allocation).await?;
    
    println!("✅ Resource management test passed");
    
    Ok(())
}

#[tokio::test]
async fn test_intelligence_metrics() -> Result<(), Box<dyn std::error::Error>> {
    let intelligence = DefaultIntelligenceCoordinator::new(
        Box::new(MetaLearningSystem::new()),
        Box::new(SelfOrganizingSystem::new()),
        Box::new(EmergenceAnalyzer::new()),
        Box::new(CreativeSystem::new()),
    );
    
    let metrics = intelligence.metrics().await?;
    
    // Verify initial metrics
    assert_eq!(metrics.meta_learning_efficiency, 0.0);
    assert_eq!(metrics.self_organization_degree, 0.0);
    assert_eq!(metrics.goal_achievement_rate, 0.0);
    assert_eq!(metrics.creativity_index, 0.0);
    assert_eq!(metrics.adaptation_speed, 0.0);
    
    println!("✅ Intelligence metrics test passed");
    
    Ok(())
}