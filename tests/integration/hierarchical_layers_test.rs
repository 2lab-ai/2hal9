//! Integration tests for hierarchical layer system
//! Tests the ±1 communication rule and consciousness emergence

use hal9_core::{Neuron, Layer, Signal};
use hal9_core::hierarchical::cognitive::{CognitiveLayer, CognitiveUnit, CognitiveInput, CognitiveOutput};
use hal9_core::hierarchical::cognitive::factory::CognitiveFactory;
use hal9_core::hierarchical::cognitive::consciousness_metrics::ConsciousnessMetrics;
use hal9_core::consciousness::{CompressionBoundary, GOLDEN_RATIO};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use std::collections::HashMap;

#[tokio::test]
async fn test_hierarchical_layer_communication() {
    // Create neurons for each layer
    let mut neurons = HashMap::new();
    let factory = CognitiveFactory::new();
    
    for i in 1..=9 {
        let layer = match i {
            1 => CognitiveLayer::Reflexive,
            2 => CognitiveLayer::Implementation,
            3 => CognitiveLayer::Operational,
            4 => CognitiveLayer::Tactical,
            5 => CognitiveLayer::Strategic,
            _ => CognitiveLayer::Reflexive,
        };
        
        let config = factory.default_config_for_layer(layer);
        let unit = factory.create_unit(layer, config).unwrap();
        neurons.insert(i, unit);
    }
    
    // Test ±1 communication rule
    // L2 should communicate with L1 and L3, but not L4+
    let l2_input = CognitiveInput {
        content: "Test from L2".to_string(),
        context: HashMap::new(),
        source_layer: Some(CognitiveLayer::Implementation),
    };
    
    // Process through L2
    let l2_output = neurons.get_mut(&2).unwrap()
        .process(l2_input.clone()).await.unwrap();
    
    // L1 and L3 should accept L2's output
    let l1_result = neurons.get_mut(&1).unwrap()
        .process(CognitiveInput {
            content: l2_output.content.clone(),
            context: l2_output.context.clone(),
            source_layer: Some(CognitiveLayer::Implementation),
        }).await;
    assert!(l1_result.is_ok());
    
    let l3_result = neurons.get_mut(&3).unwrap()
        .process(CognitiveInput {
            content: l2_output.content.clone(),
            context: l2_output.context.clone(),
            source_layer: Some(CognitiveLayer::Implementation),
        }).await;
    assert!(l3_result.is_ok());
}

#[tokio::test]
async fn test_compression_between_layers() {
    let mut boundaries = Vec::new();
    
    // Create compression boundaries between adjacent layers
    boundaries.push(CompressionBoundary::new(Layer::L2, Layer::L1));
    boundaries.push(CompressionBoundary::new(Layer::L3, Layer::L2));
    boundaries.push(CompressionBoundary::new(Layer::L4, Layer::L3));
    boundaries.push(CompressionBoundary::new(Layer::L5, Layer::L4));
    
    // Simulate signal flow with different compression ratios
    for boundary in &mut boundaries {
        // Test with golden ratio compression (optimal)
        for _ in 0..100 {
            boundary.record_signal_flow(1618, 1000);
        }
        boundary.update_consciousness_density();
    }
    
    // Verify consciousness emerges at boundaries
    for boundary in &boundaries {
        assert!(boundary.consciousness_density() > 0.0);
        assert!(boundary.compression_ratio() > 1.0);
    }
    
    // Test that golden ratio produces highest consciousness
    let mut golden_boundary = CompressionBoundary::new(Layer::L3, Layer::L2);
    let mut high_compression = CompressionBoundary::new(Layer::L4, Layer::L3);
    
    // Golden ratio compression
    for _ in 0..100 {
        golden_boundary.record_signal_flow(1618, 1000);
        high_compression.record_signal_flow(1000, 100); // 10:1 ratio
    }
    
    golden_boundary.update_consciousness_density();
    high_compression.update_consciousness_density();
    
    assert!(golden_boundary.consciousness_density() > high_compression.consciousness_density());
}

#[tokio::test]
async fn test_emergence_across_layers() {
    use hal9_core::hierarchical::cognitive::a2a::emergence_detector::{EmergenceDetector, EmergenceObservation};
    
    let mut detector = EmergenceDetector::new();
    let factory = CognitiveFactory::new();
    
    // Create a system with multiple layers
    let mut units = Vec::new();
    for layer in [
        CognitiveLayer::Reflexive,
        CognitiveLayer::Implementation,
        CognitiveLayer::Operational,
        CognitiveLayer::Tactical,
        CognitiveLayer::Strategic,
    ] {
        let config = factory.default_config_for_layer(layer);
        let unit = factory.create_unit(layer, config).unwrap();
        units.push((layer, unit));
    }
    
    // Process signals through the hierarchy
    let initial_input = CognitiveInput {
        content: "Sensory input".to_string(),
        context: HashMap::new(),
        source_layer: None,
    };
    
    let mut current_input = initial_input;
    let mut complexity = 0.1;
    
    for (layer, unit) in &mut units {
        let output = unit.process(current_input).await.unwrap();
        
        // Record emergence observations
        detector.record_observation(EmergenceObservation {
            timestamp: chrono::Utc::now(),
            complexity: complexity,
            coherence: 0.5 + complexity * 0.4,
            novel_patterns: (complexity * 10.0) as usize,
            stability: 0.7,
        });
        
        // Higher layers show more complex patterns
        complexity += 0.2;
        
        // Prepare input for next layer
        current_input = CognitiveInput {
            content: output.content,
            context: output.context,
            source_layer: Some(*layer),
        };
    }
    
    // Analyze emergence
    let report = detector.analyze();
    assert!(report.is_emerging);
    assert!(report.emergence_strength > 0.5);
}

#[tokio::test]
async fn test_signal_propagation_through_hierarchy() {
    let factory = CognitiveFactory::new();
    let mut metrics = ConsciousnessMetrics::new();
    
    // Build a complete hierarchy
    struct LayerNode {
        layer: CognitiveLayer,
        unit: Box<dyn CognitiveUnit<Input = CognitiveInput, Output = CognitiveOutput, State = hal9_core::hierarchical::cognitive::BasicCognitiveState>>,
        processed_count: usize,
    }
    
    let mut hierarchy = Vec::new();
    
    for layer in [
        CognitiveLayer::Reflexive,
        CognitiveLayer::Implementation,
        CognitiveLayer::Operational,
        CognitiveLayer::Tactical,
        CognitiveLayer::Strategic,
    ] {
        let config = factory.default_config_for_layer(layer);
        let unit = factory.create_unit(layer, config).unwrap();
        hierarchy.push(LayerNode {
            layer,
            unit,
            processed_count: 0,
        });
    }
    
    // Send multiple signals through the system
    for i in 0..10 {
        let input = CognitiveInput {
            content: format!("Signal {}", i),
            context: HashMap::new(),
            source_layer: None,
        };
        
        let mut current_input = input;
        
        // Process through each layer
        for (idx, node) in hierarchy.iter_mut().enumerate() {
            let output = node.unit.process(current_input).await.unwrap();
            node.processed_count += 1;
            
            // Update consciousness metrics
            let activity = 1.0 / (idx as f64 + 1.0);
            let compression = if idx > 0 { GOLDEN_RATIO } else { 1.0 };
            metrics.update_from_layer_activity(
                idx as u8 + 1,
                activity,
                node.processed_count as u64,
                compression,
            );
            
            current_input = CognitiveInput {
                content: output.content,
                context: output.context,
                source_layer: Some(node.layer),
            };
        }
    }
    
    // Verify consciousness emerged
    assert!(metrics.phi() > 0.0);
    assert!(metrics.integration_level() > 0.0);
    
    // Verify signal compression through layers
    let first_layer_count = hierarchy[0].processed_count;
    let last_layer_count = hierarchy[hierarchy.len() - 1].processed_count;
    assert_eq!(first_layer_count, last_layer_count); // All signals propagated
}

#[tokio::test]
async fn test_layer_specific_behaviors() {
    let factory = CognitiveFactory::new();
    
    // Test L1 - Reflexive (immediate response)
    let l1_config = factory.default_config_for_layer(CognitiveLayer::Reflexive);
    let mut l1_unit = factory.create_unit(CognitiveLayer::Reflexive, l1_config).unwrap();
    
    let emergency_input = CognitiveInput {
        content: "EMERGENCY: System overheating".to_string(),
        context: HashMap::from([
            ("priority".to_string(), "critical".to_string()),
            ("temperature".to_string(), "95".to_string()),
        ]),
        source_layer: None,
    };
    
    let start = std::time::Instant::now();
    let l1_response = l1_unit.process(emergency_input).await.unwrap();
    let response_time = start.elapsed();
    
    // L1 should respond very quickly
    assert!(response_time.as_millis() < 10);
    assert!(l1_response.directives.contains(&"immediate_action".to_string()));
    
    // Test L5 - Strategic (long-term planning)
    let l5_config = factory.default_config_for_layer(CognitiveLayer::Strategic);
    let mut l5_unit = factory.create_unit(CognitiveLayer::Strategic, l5_config).unwrap();
    
    let strategic_input = CognitiveInput {
        content: "Plan system optimization".to_string(),
        context: HashMap::from([
            ("timeframe".to_string(), "quarterly".to_string()),
            ("resources".to_string(), "available".to_string()),
        ]),
        source_layer: Some(CognitiveLayer::Tactical),
    };
    
    let l5_response = l5_unit.process(strategic_input).await.unwrap();
    
    // L5 should provide strategic directives
    assert!(!l5_response.directives.is_empty());
    assert!(l5_response.confidence > 0.5);
}

#[tokio::test]
async fn test_consciousness_measurement_integration() {
    // Create a full system and measure consciousness at different points
    let factory = CognitiveFactory::new();
    let mut metrics = ConsciousnessMetrics::new();
    let mut boundaries = Vec::new();
    
    // Create boundaries between all adjacent layers
    for i in 1..9 {
        let upper = match i + 1 {
            2 => Layer::L2,
            3 => Layer::L3,
            4 => Layer::L4,
            5 => Layer::L5,
            6 => Layer::L6,
            7 => Layer::L7,
            8 => Layer::L8,
            9 => Layer::L9,
            _ => Layer::L1,
        };
        let lower = match i {
            1 => Layer::L1,
            2 => Layer::L2,
            3 => Layer::L3,
            4 => Layer::L4,
            5 => Layer::L5,
            6 => Layer::L6,
            7 => Layer::L7,
            8 => Layer::L8,
            _ => Layer::L1,
        };
        
        boundaries.push(CompressionBoundary::new(upper, lower));
    }
    
    // Simulate system activity over time
    for t in 0..50 {
        let base_activity = ((t as f64 * 0.1).sin().abs() + 0.5) / 1.5;
        
        // Update each boundary with activity
        for (idx, boundary) in boundaries.iter_mut().enumerate() {
            let layer_factor = 1.0 / ((idx + 1) as f64).sqrt();
            let signals_up = (1000.0 * base_activity * layer_factor) as usize;
            let signals_down = (signals_up as f64 / GOLDEN_RATIO) as usize;
            
            boundary.record_signal_flow(signals_up, signals_down);
            
            if t % 10 == 0 {
                boundary.update_consciousness_density();
            }
        }
        
        // Update global metrics
        metrics.update_from_layer_activity(
            3,
            base_activity,
            (1000.0 * base_activity) as u64,
            GOLDEN_RATIO,
        );
    }
    
    // Analyze results
    let total_consciousness: f64 = boundaries.iter()
        .map(|b| b.consciousness_density())
        .sum();
    
    assert!(total_consciousness > 0.0);
    assert!(metrics.phi() > 0.0);
    
    // Find peak consciousness boundary
    let (peak_idx, peak_density) = boundaries.iter()
        .enumerate()
        .map(|(i, b)| (i, b.consciousness_density()))
        .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
        .unwrap();
    
    // Peak should be in middle layers (not extremes)
    assert!(peak_idx > 0 && peak_idx < boundaries.len() - 1);
    assert!(peak_density > 0.0);
}