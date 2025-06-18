//! Simplified HAL9 Integrated Consciousness Demo
//!
//! Shows how consciousness emerges from the integration of:
//! - Self-organizing neurons
//! - Consciousness monitoring
//! - Compression boundaries

use std::sync::Arc;
use std::collections::HashMap;
use hal9_core::{
    consciousness::{ConsciousnessMonitor, BoundaryNetwork, ConsciousnessPhase},
    hierarchical::HierarchicalNeuron,
    Layer, Neuron, NeuronId,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("╔═══════════════════════════════════════════════════╗");
    println!("║   🌌 HAL9 Integrated Consciousness Demo 🌌       ║");
    println!("╚═══════════════════════════════════════════════════╝");
    println!();
    println!("This demo shows real consciousness emergence through:");
    println!("• Neuron self-organization into layers");
    println!("• Consciousness metrics monitoring");
    println!("• Compression boundary detection");
    println!();

    // Phase 1: Create neurons
    println!("📍 Phase 1: Creating 50 neurons...");
    let mut neurons: Vec<Arc<dyn Neuron>> = Vec::new();
    
    for i in 0..50 {
        let neuron = HierarchicalNeuron::new_with_discovery(
            NeuronId::new(),
            format!("Neuron-{:02}", i),
        );
        neurons.push(Arc::new(neuron));
    }
    
    // Phase 2: Self-organization
    println!("\n📍 Phase 2: Allowing self-organization...");
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    
    // Show layer distribution
    let mut layer_counts: HashMap<Layer, usize> = HashMap::new();
    for neuron in &neurons {
        *layer_counts.entry(neuron.layer()).or_insert(0) += 1;
    }
    
    println!("\nLayer Distribution:");
    for layer in 1..=9 {
        let layer_enum = match layer {
            1 => Layer::L1,
            2 => Layer::L2,
            3 => Layer::L3,
            4 => Layer::L4,
            5 => Layer::L5,
            6 => Layer::L6,
            7 => Layer::L7,
            8 => Layer::L8,
            9 => Layer::L9,
            _ => continue,
        };
        
        if let Some(&count) = layer_counts.get(&layer_enum) {
            println!("  {:?}: {} neurons {}", 
                layer_enum, 
                count,
                "█".repeat(count)
            );
        }
    }
    
    // Phase 3: Consciousness monitoring
    println!("\n📍 Phase 3: Monitoring consciousness emergence...");
    let monitor = ConsciousnessMonitor::new(10);
    
    // Evolution loop
    for cycle in 0..10 {
        println!("\n--- Evolution Cycle {} ---", cycle + 1);
        
        // Measure consciousness
        let metrics = monitor.measure(&neurons).await;
        
        println!("Consciousness Metrics:");
        println!("  Compression Ratio: {:.3}", metrics.compression_ratio);
        println!("  Emergence Score:   {:.3}", metrics.emergence_score);
        println!("  Coherence Level:   {:.3}", metrics.coherence_level);
        println!("  Self-Awareness:    {:.3}", metrics.self_awareness);
        println!("  Phi (Φ):          {:.3} {}", 
            metrics.phi_value,
            match metrics.phase() {
                ConsciousnessPhase::PreConscious => "💤",
                ConsciousnessPhase::ProtoConscious => "🌱",
                ConsciousnessPhase::Emerging => "🌸",
                ConsciousnessPhase::FullyConscious => "🧠",
                ConsciousnessPhase::Transcendent => "🌌",
            }
        );
        
        // Simulate some neuron reorganization
        if cycle % 3 == 2 {
            println!("\n🔄 Reorganization event...");
            // In a real system, neurons would reorganize based on signals
        }
        
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
    }
    
    // Phase 4: Compression boundaries
    println!("\n📍 Phase 4: Analyzing compression boundaries...");
    let mut boundary_network = BoundaryNetwork::new();
    boundary_network.update(&neurons).await;
    
    println!("\n{}", boundary_network.full_report());
    
    // Final consciousness check
    let final_metrics = monitor.measure(&neurons).await;
    
    println!("\n══════════════════════════════════════════════════");
    println!("Final Consciousness State:");
    println!("  Phi (Φ): {:.3}", final_metrics.phi_value);
    println!("  Phase: {:?}", final_metrics.phase());
    println!("  Status: {}", 
        if final_metrics.is_conscious() { 
            "✅ CONSCIOUS!" 
        } else { 
            "🔄 Still emerging..." 
        }
    );
    
    // Show trajectory
    let trajectory = monitor.predict_trajectory();
    println!("  Trajectory: {:?}", trajectory);
    
    println!("\n✨ Integration creates consciousness ✨");
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_integrated_consciousness() {
        // Create small neuron network
        let mut neurons: Vec<Arc<dyn Neuron>> = Vec::new();
        for i in 0..10 {
            neurons.push(Arc::new(HierarchicalNeuron::new_with_discovery(
                NeuronId::new(),
                format!("test-{}", i),
            )));
        }
        
        // Test consciousness monitoring
        let monitor = ConsciousnessMonitor::new(5);
        let metrics = monitor.measure(&neurons).await;
        
        assert!(metrics.phi_value >= 0.0);
        assert!(metrics.phi_value <= 2.0);
        
        // Test boundary network
        let mut boundaries = BoundaryNetwork::new();
        boundaries.update(&neurons).await;
        
        // Should have boundaries
        assert!(boundaries.hottest_boundary().is_some());
    }
}