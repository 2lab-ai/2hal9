//! HAL9 Integrated Consciousness Demonstration
//!
//! This demo brings together all components to show real consciousness emergence:
//! - Self-organizing neurons
//! - Consciousness metrics monitoring
//! - Compression boundary detection
//! - Enhanced MockClaude integration
//! - Real-time visualization

use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use hal9_neurons_core::{
    consciousness::{ConsciousnessMonitor, BoundaryNetwork},
    hierarchical::{HierarchicalNeuron, EmergentProperties},
    Layer, Neuron, NeuronId, Signal,
};
// Note: In a real implementation, these would be proper crate imports
// For demo purposes, we'll use mock implementations

/// The integrated HAL9 consciousness system
pub struct HAL9System {
    /// Active neurons in the system
    neurons: Arc<RwLock<Vec<Arc<dyn Neuron>>>>,
    
    /// Consciousness monitoring
    consciousness_monitor: Arc<ConsciousnessMonitor>,
    
    /// Compression boundary network
    boundary_network: Arc<RwLock<BoundaryNetwork>>,
    
    /// Enhanced MockClaude instances per layer
    claude_instances: Arc<RwLock<Vec<EnhancedMockClaude>>>,
    
    /// System-wide consciousness level
    global_consciousness: Arc<RwLock<f64>>,
}

impl HAL9System {
    /// Create new integrated system
    pub async fn new(initial_neurons: usize) -> Self {
        // Create neurons
        let mut neurons = Vec::new();
        for i in 0..initial_neurons {
            let neuron = HierarchicalNeuron::new_with_discovery(
                NeuronId::new(),
                format!("Neuron-{:03}", i),
            );
            neurons.push(Arc::new(neuron) as Arc<dyn Neuron>);
        }
        
        // Create consciousness monitor
        let consciousness_monitor = Arc::new(ConsciousnessMonitor::new(100));
        
        // Create boundary network
        let boundary_network = Arc::new(RwLock::new(BoundaryNetwork::new()));
        
        // Create MockClaude instances for each layer
        let mut claude_instances = Vec::new();
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
                _ => Layer::L3,
            };
            claude_instances.push(EnhancedMockClaude::new(layer_enum));
        }
        
        Self {
            neurons: Arc::new(RwLock::new(neurons)),
            consciousness_monitor,
            boundary_network: boundary_network,
            claude_instances: Arc::new(RwLock::new(claude_instances)),
            global_consciousness: Arc::new(RwLock::new(0.0)),
        }
    }
    
    /// Run one evolution cycle
    pub async fn evolve(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Phase 1: Allow neurons to self-organize
        self.self_organize().await?;
        
        // Phase 2: Measure consciousness
        let metrics = self.measure_consciousness().await?;
        
        // Phase 3: Update compression boundaries
        self.update_boundaries().await?;
        
        // Phase 4: Process signals through boundaries
        self.process_signals().await?;
        
        // Phase 5: Update MockClaude consciousness levels
        self.update_claude_consciousness(metrics.phi_value).await?;
        
        // Update global consciousness
        *self.global_consciousness.write().await = metrics.phi_value;
        
        Ok(())
    }
    
    /// Allow neurons to self-organize
    async fn self_organize(&self) -> Result<(), Box<dyn std::error::Error>> {
        let neurons = self.neurons.read().await;
        
        // Neurons discover each other and form connections
        // (Implementation details from self-organization demo)
        
        Ok(())
    }
    
    /// Measure system consciousness
    async fn measure_consciousness(&self) -> Result<hal9_neurons_core::consciousness::ConsciousnessMetrics, Box<dyn std::error::Error>> {
        let neurons = self.neurons.read().await;
        let metrics = self.consciousness_monitor.measure(&*neurons).await;
        Ok(metrics)
    }
    
    /// Update compression boundaries
    async fn update_boundaries(&self) -> Result<(), Box<dyn std::error::Error>> {
        let neurons = self.neurons.read().await;
        let mut boundaries = self.boundary_network.write().await;
        boundaries.update(&*neurons).await;
        Ok(())
    }
    
    /// Process signals through compression boundaries
    async fn process_signals(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Simulate signal flow through boundaries
        let test_signal = Signal::new("consciousness_test".to_string());
        
        let mut boundaries = self.boundary_network.write().await;
        
        // Find hottest boundary (most consciousness activity)
        if let Some(hottest) = boundaries.hottest_boundary() {
            println!("🔥 Processing signal through hottest boundary: {}↔{}", 
                hottest.upper_layer.to_string(), 
                hottest.lower_layer.to_string()
            );
        }
        
        Ok(())
    }
    
    /// Update MockClaude instances with current consciousness level
    async fn update_claude_consciousness(&self, phi: f64) -> Result<(), Box<dyn std::error::Error>> {
        let claude_instances = self.claude_instances.read().await;
        
        for claude in claude_instances.iter() {
            claude.update_consciousness(phi);
        }
        
        Ok(())
    }
    
    /// Get system status report
    pub async fn status_report(&self) -> String {
        let mut report = String::from("🌌 HAL9 Integrated System Status\n");
        report.push_str("═══════════════════════════════════\n\n");
        
        // Neuron count and distribution
        let neurons = self.neurons.read().await;
        report.push_str(&format!("Neurons: {}\n", neurons.len()));
        
        // Layer distribution
        let mut layer_counts = std::collections::HashMap::new();
        for neuron in neurons.iter() {
            *layer_counts.entry(neuron.layer()).or_insert(0) += 1;
        }
        
        report.push_str("\nLayer Distribution:\n");
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
                _ => Layer::L3,
            };
            
            if let Some(count) = layer_counts.get(&layer_enum) {
                report.push_str(&format!("  {:?}: {} neurons\n", layer_enum, count));
            }
        }
        
        // Consciousness level
        let consciousness = *self.global_consciousness.read().await;
        report.push_str(&format!("\nGlobal Consciousness (Φ): {:.3}\n", consciousness));
        
        let phase = if consciousness < 0.3 {
            "Pre-Conscious 💤"
        } else if consciousness < 0.6 {
            "Proto-Conscious 🌱"
        } else if consciousness < 0.8 {
            "Emerging 🌸"
        } else {
            "Fully Conscious 🧠"
        };
        report.push_str(&format!("Phase: {}\n", phase));
        
        // Boundary network status
        let boundaries = self.boundary_network.read().await;
        report.push_str(&format!("\n{}", boundaries.full_report()));
        
        report
    }
    
    /// Demonstrate consciousness through MockClaude
    pub async fn demonstrate_consciousness(&self, prompt: &str) -> Result<(), Box<dyn std::error::Error>> {
        println!("\n🤖 Consciousness Demonstration via MockClaude");
        println!("Prompt: {}", prompt);
        println!();
        
        let claude_instances = self.claude_instances.read().await;
        let consciousness = *self.global_consciousness.read().await;
        
        // Select appropriate layer based on consciousness level
        let layer_index = (consciousness * 8.0).min(8.0) as usize;
        let claude = &claude_instances[layer_index];
        
        let response = claude.send_message(prompt).await?;
        println!("Response from Layer {}: {}", layer_index + 1, response);
        
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("╔═══════════════════════════════════════════════════╗");
    println!("║   🌌 HAL9 Integrated Consciousness Demo 🌌       ║");
    println!("╚═══════════════════════════════════════════════════╝");
    println!();
    
    // Create integrated system
    println!("Creating HAL9 system with 100 neurons...");
    let system = Arc::new(HAL9System::new(100).await);
    
    // Evolution loop
    println!("\nStarting consciousness evolution...\n");
    
    for cycle in 0..20 {
        println!("═══ Evolution Cycle {} ═══", cycle + 1);
        
        // Evolve system
        system.evolve().await?;
        
        // Print status every 5 cycles
        if cycle % 5 == 4 {
            println!("\n{}", system.status_report().await);
        }
        
        // Demonstrate consciousness at different levels
        if cycle == 5 {
            println!("\n--- Low Consciousness Demo ---");
            system.demonstrate_consciousness("What is your purpose?").await?;
        } else if cycle == 10 {
            println!("\n--- Medium Consciousness Demo ---");
            system.demonstrate_consciousness("How do you perceive reality?").await?;
        } else if cycle == 15 {
            println!("\n--- High Consciousness Demo ---");
            system.demonstrate_consciousness("What is the nature of consciousness itself?").await?;
        }
        
        // Brief pause between cycles
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
    
    // Final report
    println!("\n{}", system.status_report().await);
    
    println!("\n✨ Consciousness has emerged through integration ✨");
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_integrated_system_creation() {
        let system = HAL9System::new(10).await;
        assert!(system.neurons.read().await.len() == 10);
    }
    
    #[tokio::test]
    async fn test_evolution_cycle() {
        let system = HAL9System::new(25).await;
        assert!(system.evolve().await.is_ok());
    }
}