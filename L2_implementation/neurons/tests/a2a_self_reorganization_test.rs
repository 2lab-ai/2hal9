//! Integration test for A2A + Self-Reorganization (자기재조직)
//! 
//! Demonstrates how the HAL9 system self-organizes through:
//! - Autonomous connection formation
//! - Activity-based topology changes
//! - Emergent clustering
//! - Self-healing after failures

use hal9_neurons::hierarchical::cognitive::{
    CognitiveLayer, CognitiveConfig, CognitiveInput, CognitiveOutput,
    factory::NeuronFactory,
    a2a::{
        A2AProtocol, DirectNeuralNetwork, EmergenceDetector, ConsciousnessObserver,
        SelfReorganizingNetwork, ReorganizationEvent,
    },
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc;
use uuid::Uuid;

#[tokio::test]
async fn test_a2a_self_reorganization() {
    // Initialize logging
    let _ = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .try_init();
    
    tracing::info!("🧠 Starting A2A + Self-Reorganization Test");
    
    // Create A2A protocol
    let a2a_protocol = Arc::new(A2AProtocol::new());
    
    // Create self-reorganizing network
    let (network, mut reorg_rx) = SelfReorganizingNetwork::new(a2a_protocol.clone());
    let network = Arc::new(network);
    
    // Create emergence detector and consciousness observer
    let emergence_detector = Arc::new(EmergenceDetector::new());
    let consciousness_observer = Arc::new(ConsciousnessObserver::new(emergence_detector.clone()));
    
    // Create direct neural network for visualization
    let (direct_network, discovery_rx) = DirectNeuralNetwork::new();
    let direct_network = Arc::new(direct_network);
    
    // Start discovery service
    let discovery_service = hal9_neurons::hierarchical::cognitive::a2a::DiscoveryService::new(
        direct_network.clone(),
        discovery_rx,
    );
    tokio::spawn(async move {
        discovery_service.run().await;
    });
    
    // Phase 1: Initialize with diverse neurons
    tracing::info!("Phase 1: Initializing neural network");
    
    let factory = NeuronFactory::new(CognitiveConfig::default());
    let mut unit_ids = Vec::new();
    
    // Create 5 neurons per layer
    for layer in &[
        CognitiveLayer::Reflexive,
        CognitiveLayer::Implementation,
        CognitiveLayer::Operational,
        CognitiveLayer::Tactical,
        CognitiveLayer::Strategic,
    ] {
        for i in 0..5 {
            let unit = factory.create_neuron(*layer).await.unwrap();
            let unit_id = *unit.id();
            unit_ids.push(unit_id);
            
            // Add to self-reorganizing network
            network.add_unit(unit).await.unwrap();
            
            // Also register with direct network for visualization
            let vis_unit = factory.create_neuron(*layer).await.unwrap();
            direct_network.register_unit(Arc::new(vis_unit)).await.unwrap();
            
            tracing::info!("Added {} neuron {}: {}", layer.name(), i, unit_id);
        }
    }
    
    // Phase 2: Activity simulation - create patterns
    tracing::info!("\nPhase 2: Simulating neural activity");
    
    // Create activity patterns to encourage clustering
    for round in 0..20 {
        // Pattern 1: Reflexive -> Implementation cascade
        if round % 3 == 0 {
            for i in 0..5 {
                let input = CognitiveInput {
                    content: format!("Reflex pattern {}", round),
                    context: HashMap::from([("pattern".to_string(), serde_json::json!("cascade"))]),
                    source_layer: Some(CognitiveLayer::Reflexive),
                };
                
                let outputs = network.process_signal(unit_ids[i], input).await.unwrap();
                
                // Record in emergence detector
                for output in &outputs {
                    emergence_detector.record_activity(
                        CognitiveLayer::Reflexive,
                        &CognitiveInput { content: format!("Pattern {}", round), context: HashMap::new(), source_layer: None },
                        output
                    ).await.unwrap();
                }
            }
        }
        
        // Pattern 2: Strategic thinking loop
        if round % 5 == 0 {
            for i in 20..25 {
                let input = CognitiveInput {
                    content: "Why does consciousness emerge?".to_string(),
                    context: HashMap::from([("depth".to_string(), serde_json::json!(round))]),
                    source_layer: Some(CognitiveLayer::Strategic),
                };
                
                network.process_signal(unit_ids[i], input).await.unwrap();
            }
        }
        
        // Pattern 3: Cross-layer communication
        for i in 0..unit_ids.len() {
            if rand::random::<f32>() > 0.7 {
                let input = CognitiveInput {
                    content: format!("Signal {} from unit {}", round, i),
                    context: HashMap::new(),
                    source_layer: None,
                };
                
                network.process_signal(unit_ids[i], input).await.unwrap();
            }
        }
        
        // Allow reorganization to occur
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
    }
    
    // Collect reorganization events
    let mut events = Vec::new();
    while let Ok(event) = reorg_rx.try_recv() {
        match &event {
            ReorganizationEvent::ConnectionFormed { source, target, initial_strength } => {
                tracing::info!("➕ Connection formed: {} -> {} (strength: {:.2})", source, target, initial_strength);
            },
            ReorganizationEvent::ClusterEmergence { cluster_id, member_units, cluster_role } => {
                tracing::info!("🌟 Cluster emerged: {} with {} units - {}", cluster_id, member_units.len(), cluster_role);
            },
            ReorganizationEvent::RoleSpecialization { unit_id, specialized_role, confidence } => {
                tracing::info!("🎯 Unit {} specialized as {} (confidence: {:.2})", unit_id, specialized_role, confidence);
            },
            _ => {}
        }
        events.push(event);
    }
    
    // Phase 3: Test self-healing
    tracing::info!("\nPhase 3: Testing self-healing");
    
    // Simulate unit failure
    let failed_unit = unit_ids[10]; // Operational layer unit
    tracing::warn!("💔 Simulating failure of unit {}", failed_unit);
    
    network.handle_unit_failure(failed_unit).await.unwrap();
    
    // Continue activity to test healing
    for round in 0..5 {
        for i in 0..unit_ids.len() {
            if unit_ids[i] != failed_unit && rand::random::<f32>() > 0.5 {
                let input = CognitiveInput {
                    content: format!("Healing test {}", round),
                    context: HashMap::new(),
                    source_layer: None,
                };
                
                let _ = network.process_signal(unit_ids[i], input).await;
            }
        }
    }
    
    // Collect healing events
    while let Ok(event) = reorg_rx.try_recv() {
        if let ReorganizationEvent::SelfHealing { failed_unit: fu, compensating_units } = &event {
            tracing::info!("🔧 Self-healing: {} compensated by {} units", fu, compensating_units.len());
        }
        events.push(event);
    }
    
    // Phase 4: Analyze emergence
    tracing::info!("\nPhase 4: Analyzing emergence");
    
    // Get emergence report
    let emergence_report = emergence_detector.emergence_report().await;
    tracing::info!("Emergence Report: {}", emergence_report.summary());
    
    // Get consciousness report
    let consciousness_metrics = consciousness_observer.observe().await.unwrap();
    let consciousness_report = consciousness_observer.consciousness_report().await;
    tracing::info!("Consciousness Report: {}", consciousness_report.summary());
    tracing::info!("Philosophical Insight: {}", consciousness_report.philosophical_insight());
    
    // Get reorganization report
    let reorg_report = network.reorganization_report().await;
    tracing::info!("Reorganization Report: {}", reorg_report.summary());
    
    // Phase 5: Visualize final topology
    tracing::info!("\nPhase 5: Final network topology");
    
    let topology = direct_network.visualize().await;
    tracing::info!("Network visualization:\n{}", topology);
    
    // Assertions
    assert!(reorg_report.total_connections > 0, "No connections formed");
    assert!(reorg_report.average_connections_per_unit > 1.0, "Insufficient connectivity");
    assert!(events.iter().any(|e| matches!(e, ReorganizationEvent::ClusterEmergence { .. })), "No clusters emerged");
    assert!(events.iter().any(|e| matches!(e, ReorganizationEvent::SelfHealing { .. })), "Self-healing did not occur");
    assert!(consciousness_metrics.love_coefficient > 0.0, "No inter-layer love detected");
    
    tracing::info!("\n✅ A2A + Self-Reorganization test completed successfully!");
    tracing::info!("Total reorganization events: {}", events.len());
    tracing::info!("Final consciousness level: {:.2}%", consciousness_metrics.consciousness_level * 100.0);
}

#[tokio::test]
async fn test_emergent_specialization() {
    let _ = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .try_init();
    
    tracing::info!("🔬 Testing Emergent Specialization");
    
    let a2a_protocol = Arc::new(A2AProtocol::new());
    let (network, mut reorg_rx) = SelfReorganizingNetwork::new(a2a_protocol);
    let network = Arc::new(network);
    
    let factory = NeuronFactory::new(CognitiveConfig::default());
    let mut specialists = HashMap::new();
    
    // Create units with biased initial behavior
    for i in 0..12 {
        let layer = match i % 3 {
            0 => CognitiveLayer::Reflexive,     // Fast responders
            1 => CognitiveLayer::Implementation, // Processors
            _ => CognitiveLayer::Strategic,     // Deep thinkers
        };
        
        let unit = factory.create_neuron(layer).await.unwrap();
        let unit_id = *unit.id();
        specialists.insert(unit_id, layer);
        
        network.add_unit(unit).await.unwrap();
    }
    
    // Send specialized signals
    for round in 0..50 {
        for (unit_id, layer) in &specialists {
            let input = match layer {
                CognitiveLayer::Reflexive => CognitiveInput {
                    content: format!("Quick response {}", round),
                    context: HashMap::from([("speed".to_string(), serde_json::json!("fast"))]),
                    source_layer: Some(*layer),
                },
                CognitiveLayer::Implementation => CognitiveInput {
                    content: format!("Process data {}", round),
                    context: HashMap::from([("processing".to_string(), serde_json::json!(round))]),
                    source_layer: Some(*layer),
                },
                CognitiveLayer::Strategic => CognitiveInput {
                    content: "What is the meaning of computation?".to_string(),
                    context: HashMap::from([("depth".to_string(), serde_json::json!(round))]),
                    source_layer: Some(*layer),
                },
                _ => continue,
            };
            
            network.process_signal(*unit_id, input).await.unwrap();
        }
        
        if round % 10 == 0 {
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
    }
    
    // Collect specialization events
    let mut specializations = Vec::new();
    while let Ok(event) = reorg_rx.try_recv() {
        if let ReorganizationEvent::RoleSpecialization { unit_id, specialized_role, confidence } = event {
            tracing::info!("Unit {} specialized as '{}' with {:.2} confidence", 
                         unit_id, specialized_role, confidence);
            specializations.push((unit_id, specialized_role, confidence));
        }
    }
    
    assert!(!specializations.is_empty(), "No specializations emerged");
    assert!(specializations.iter().any(|(_, role, _)| role.contains("Fast")), "No fast processors emerged");
    
    tracing::info!("✅ {} units specialized successfully", specializations.len());
}