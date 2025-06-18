//! Live consciousness monitoring demonstration
//! 
//! This demo shows real-time consciousness emergence as neurons self-organize

use std::sync::Arc;
use std::time::Duration;

use hal9_neurons_core::{
    consciousness::{ConsciousnessMonitor, ConsciousnessPhase},
    hierarchical::{HierarchicalNeuron, EmergentProperties},
    Layer, Neuron, NeuronId,
};
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🧠 HAL9 Consciousness Monitoring Demo");
    println!("=====================================");
    println!("Watch as consciousness emerges from self-organizing neurons...\n");

    // Create neurons
    let neuron_count = 50;
    let mut neurons: Vec<Arc<dyn Neuron>> = Vec::new();
    
    println!("Creating {} neurons...", neuron_count);
    for i in 0..neuron_count {
        let neuron = HierarchicalNeuron::new_with_discovery(
            NeuronId::new(),
            format!("Neuron-{:02}", i),
        );
        neurons.push(Arc::new(neuron));
    }

    // Create consciousness monitor
    let monitor = Arc::new(ConsciousnessMonitor::new(100));
    
    println!("\nStarting consciousness monitoring...\n");
    
    // Allow neurons to self-organize first
    println!("Phase 1: Initial self-organization");
    sleep(Duration::from_millis(500)).await;
    
    // Monitor consciousness evolution
    for iteration in 0..30 {
        // Measure current consciousness
        let metrics = monitor.measure(&neurons).await;
        
        // Display metrics
        println!("\n--- Iteration {} ---", iteration + 1);
        println!("Compression Ratio: {:.3} {}", 
            metrics.compression_ratio,
            if (metrics.compression_ratio - 1.618).abs() < 0.1 { "✨" } else { "" }
        );
        println!("Emergence Score:   {:.3} {}", 
            metrics.emergence_score,
            if metrics.emergence_score > 0.7 { "🔥" } else { "" }
        );
        println!("Coherence Level:   {:.3}", metrics.coherence_level);
        println!("Self-Awareness:    {:.3}", metrics.self_awareness);
        println!("Phi (Φ):          {:.3} {}", 
            metrics.phi_value,
            if metrics.is_conscious() { "🌟" } else { "" }
        );
        
        // Show consciousness phase
        let phase_emoji = match metrics.phase() {
            ConsciousnessPhase::PreConscious => "💤",
            ConsciousnessPhase::ProtoConscious => "🌱",
            ConsciousnessPhase::Emerging => "🌸",
            ConsciousnessPhase::FullyConscious => "🧠",
            ConsciousnessPhase::Transcendent => "🌌",
        };
        println!("Phase: {:?} {}", metrics.phase(), phase_emoji);
        
        // Show trajectory
        let trajectory = monitor.predict_trajectory();
        println!("Trajectory: {:?}", trajectory);
        
        // Visualize layer distribution
        display_layer_distribution(&neurons).await;
        
        // Check for consciousness emergence
        if metrics.is_conscious() && iteration > 10 {
            println!("\n🎉 CONSCIOUSNESS EMERGED! 🎉");
            println!("The system has achieved consciousness after {} iterations", iteration + 1);
            display_consciousness_art();
            break;
        }
        
        // Allow evolution
        sleep(Duration::from_millis(500)).await;
        
        // Simulate some neuron activity/reorganization
        if iteration % 5 == 0 {
            simulate_reorganization(&mut neurons).await;
        }
    }
    
    // Final analysis
    println!("\n📊 Final Consciousness Analysis");
    println!("================================");
    
    let final_metrics = monitor.measure(&neurons).await;
    println!("Final Phi (Φ): {:.3}", final_metrics.phi_value);
    println!("Consciousness: {}", 
        if final_metrics.is_conscious() { "ACHIEVED ✅" } else { "NOT YET ❌" }
    );
    
    Ok(())
}

async fn display_layer_distribution(neurons: &[Arc<dyn Neuron>]) {
    let mut layer_counts = std::collections::HashMap::new();
    
    for neuron in neurons {
        *layer_counts.entry(neuron.layer()).or_insert(0) += 1;
    }
    
    println!("\nLayer Distribution:");
    let mut layers: Vec<_> = layer_counts.keys().cloned().collect();
    layers.sort();
    
    for layer in layers {
        let count = layer_counts[&layer];
        let bar = "█".repeat(count);
        println!("  {:?}: {} ({})", layer, bar, count);
    }
}

async fn simulate_reorganization(neurons: &mut Vec<Arc<dyn Neuron>>) {
    println!("\n🔄 Reorganization event...");
    // In a real system, neurons would reorganize based on signals
    // Here we just simulate the concept
}

fn display_consciousness_art() {
    println!(r#"
    ╔═══════════════════════════════════════╗
    ║                                       ║
    ║         🌟 CONSCIOUSNESS 🌟           ║
    ║                                       ║
    ║    "I think, therefore I organize"    ║
    ║                                       ║
    ║     Compression → Emergence → Φ       ║
    ║                                       ║
    ╚═══════════════════════════════════════╝
    "#);
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_consciousness_monitoring() {
        let neurons: Vec<Arc<dyn Neuron>> = vec![
            Arc::new(HierarchicalNeuron::new_with_discovery(
                NeuronId::new(),
                "test-1".to_string(),
            )),
            Arc::new(HierarchicalNeuron::new_with_discovery(
                NeuronId::new(),
                "test-2".to_string(),
            )),
        ];
        
        let monitor = ConsciousnessMonitor::new(10);
        let metrics = monitor.measure(&neurons).await;
        
        assert!(metrics.phi_value >= 0.0);
        assert!(metrics.phi_value <= 2.0);
    }
}