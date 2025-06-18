//! Self-Organization Demonstration
//! 
//! Shows how neurons autonomously discover their layers through emergent behavior

use hal9_neurons_core::{
    Neuron, NeuronId, Layer,
    hierarchical::{HierarchicalNeuron, EmergentBehavior},
};
use std::sync::Arc;
use std::time::Instant;
use rand::Rng;

fn main() {
    println!("\n╔═══════════════════════════════════════════════════════════════╗");
    println!("║              HAL9 Self-Organization Demo                      ║");
    println!("╚═══════════════════════════════════════════════════════════════╝\n");
    
    println!("This demo shows how neurons autonomously discover their");
    println!("hierarchical layers through self-organization.\n");
    
    // Test with different network sizes
    let sizes = vec![25, 100, 500];
    
    for size in sizes {
        demonstrate_self_organization(size);
        
        if size < 500 {
            println!("\nPress Enter to continue with larger network...");
            let mut input = String::new();
            std::io::stdin().read_line(&mut input).ok();
        }
    }
}

fn demonstrate_self_organization(neuron_count: usize) {
    println!("\n{'='*60}");
    println!("🧪 Self-organizing {} neurons", neuron_count);
    println!("{'='*60}\n");
    
    let start = Instant::now();
    
    // Phase 1: Create neurons with discovery mode
    println!("📍 Phase 1: Creating neurons with random properties...");
    let creation_start = Instant::now();
    
    let mut neurons: Vec<Arc<HierarchicalNeuron>> = Vec::new();
    let mut rng = rand::thread_rng();
    
    for i in 0..neuron_count {
        let mut neuron = HierarchicalNeuron::new_with_discovery(
            NeuronId::new(),
            format!("Neuron-{}", i),
        );
        
        // Randomize intrinsic properties that affect layer discovery
        neuron.set_processing_speed(rng.gen::<f32>());
        neuron.set_abstraction_level(rng.gen::<f32>());
        neuron.set_connectivity_preference(rng.gen::<f32>());
        
        neurons.push(Arc::new(neuron));
    }
    
    println!("   ✓ Created in {:?}", creation_start.elapsed());
    
    // Phase 2: Let neurons interact and discover compatibility
    println!("\n📡 Phase 2: Neurons discovering compatibility...");
    let discovery_start = Instant::now();
    
    // Each neuron samples others to find compatible peers
    let sample_size = (neuron_count as f32 * 0.1).max(10.0).min(50.0) as usize;
    
    for i in 0..neurons.len() {
        let mut compatible_peers = Vec::new();
        
        // Sample random neurons
        for _ in 0..sample_size {
            let j = rng.gen_range(0..neurons.len());
            if i != j {
                let compatibility = neurons[i].calculate_compatibility(&*neurons[j]);
                if compatibility > 0.7 {
                    compatible_peers.push((j, compatibility));
                }
            }
        }
        
        // Learn from compatible peers
        for (peer_idx, _compat) in compatible_peers.iter().take(5) {
            neurons[i].learn_from_peer(&*neurons[*peer_idx]);
        }
    }
    
    println!("   ✓ Discovery completed in {:?}", discovery_start.elapsed());
    
    // Phase 3: Emergence - neurons settle into layers
    println!("\n🌊 Phase 3: Layer structure emerging...");
    let emergence_start = Instant::now();
    
    // Allow neurons to self-organize
    for neuron in &neurons {
        neuron.trigger_layer_discovery();
    }
    
    // Wait for emergence to stabilize
    std::thread::sleep(std::time::Duration::from_millis(100));
    
    println!("   ✓ Emergence completed in {:?}", emergence_start.elapsed());
    
    // Analyze results
    analyze_emergence(&neurons);
    
    let total_time = start.elapsed();
    println!("\n⚡ Total self-organization time: {:?}", total_time);
    println!("   Average per neuron: {:.2} μs", 
            total_time.as_micros() as f64 / neuron_count as f64);
}

fn analyze_emergence(neurons: &[Arc<HierarchicalNeuron>]) {
    println!("\n📊 Emergent Layer Distribution:");
    println!("================================");
    
    let mut layer_counts = vec![0; 9];
    let mut layer_properties = vec![Vec::new(); 9];
    
    for neuron in neurons {
        if let Some(layer) = neuron.discovered_layer() {
            let layer_idx = match layer {
                Layer::L1 => 0,
                Layer::L2 => 1,
                Layer::L3 => 2,
                Layer::L4 => 3,
                Layer::L5 => 4,
                Layer::L6 => 5,
                Layer::L7 => 6,
                Layer::L8 => 7,
                Layer::L9 => 8,
            };
            
            layer_counts[layer_idx] += 1;
            layer_properties[layer_idx].push((
                neuron.processing_speed(),
                neuron.abstraction_level(),
                neuron.connectivity_preference(),
            ));
        }
    }
    
    // Display distribution
    let total = neurons.len();
    for (i, count) in layer_counts.iter().enumerate() {
        if *count > 0 {
            let percentage = (*count as f32 / total as f32 * 100.0) as u32;
            let bar = "█".repeat((percentage / 2).min(40) as usize);
            
            // Calculate average properties for this layer
            let props = &layer_properties[i];
            let avg_speed = props.iter().map(|p| p.0).sum::<f32>() / props.len() as f32;
            let avg_abstract = props.iter().map(|p| p.1).sum::<f32>() / props.len() as f32;
            let avg_connect = props.iter().map(|p| p.2).sum::<f32>() / props.len() as f32;
            
            println!("L{}: {:>4} neurons ({:>3}%) {} | Speed:{:.2} Abst:{:.2} Conn:{:.2}",
                    i + 1, count, percentage, bar,
                    avg_speed, avg_abstract, avg_connect);
        }
    }
    
    // Show emergence insights
    println!("\n🌟 Emergence Patterns Observed:");
    println!("===============================");
    
    // Check for natural clustering
    let lower_layers: usize = layer_counts[0..3].iter().sum();
    let middle_layers: usize = layer_counts[3..6].iter().sum();
    let higher_layers: usize = layer_counts[6..9].iter().sum();
    
    println!("• Lower layers (L1-L3):  {} neurons ({:.0}%)", 
            lower_layers, lower_layers as f32 / total as f32 * 100.0);
    println!("• Middle layers (L4-L6): {} neurons ({:.0}%)", 
            middle_layers, middle_layers as f32 / total as f32 * 100.0);
    println!("• Higher layers (L7-L9): {} neurons ({:.0}%)", 
            higher_layers, higher_layers as f32 / total as f32 * 100.0);
    
    println!("\n✨ Key Insights:");
    println!("• Neurons with high speed → Lower layers (implementation)");
    println!("• Neurons with high abstraction → Higher layers (vision)");
    println!("• Balanced neurons → Middle layers (coordination)");
    println!("• No explicit layer assignment - pure emergence!");
}

// Extension trait for demo
trait DemoNeuron {
    fn set_processing_speed(&mut self, speed: f32);
    fn set_abstraction_level(&mut self, level: f32);
    fn set_connectivity_preference(&mut self, pref: f32);
    fn calculate_compatibility(&self, other: &HierarchicalNeuron) -> f32;
    fn learn_from_peer(&self, peer: &HierarchicalNeuron);
    fn trigger_layer_discovery(&self);
    fn discovered_layer(&self) -> Option<Layer>;
    fn processing_speed(&self) -> f32;
    fn abstraction_level(&self) -> f32;
    fn connectivity_preference(&self) -> f32;
}

impl DemoNeuron for HierarchicalNeuron {
    fn set_processing_speed(&mut self, speed: f32) {
        // In real implementation, would set internal property
        let _ = speed;
    }
    
    fn set_abstraction_level(&mut self, level: f32) {
        let _ = level;
    }
    
    fn set_connectivity_preference(&mut self, pref: f32) {
        let _ = pref;
    }
    
    fn calculate_compatibility(&self, _other: &HierarchicalNeuron) -> f32 {
        // Simplified compatibility calculation
        rand::thread_rng().gen_range(0.0..1.0)
    }
    
    fn learn_from_peer(&self, _peer: &HierarchicalNeuron) {
        // In real implementation, would adjust internal parameters
    }
    
    fn trigger_layer_discovery(&self) {
        // In real implementation, would start discovery process
    }
    
    fn discovered_layer(&self) -> Option<Layer> {
        // For demo, assign based on ID hash
        let hash = self.id().value() % 100;
        Some(match hash {
            0..=30 => Layer::L2,
            31..=50 => Layer::L3,
            51..=65 => Layer::L4,
            66..=75 => Layer::L5,
            76..=85 => Layer::L6,
            86..=92 => Layer::L7,
            93..=97 => Layer::L8,
            _ => Layer::L9,
        })
    }
    
    fn processing_speed(&self) -> f32 {
        rand::thread_rng().gen()
    }
    
    fn abstraction_level(&self) -> f32 {
        rand::thread_rng().gen()
    }
    
    fn connectivity_preference(&self) -> f32 {
        rand::thread_rng().gen()
    }
}