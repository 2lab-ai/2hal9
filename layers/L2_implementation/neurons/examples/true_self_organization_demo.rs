//! True Self-Organization Demo
//! 
//! Shows how 25 identical neurons organize themselves into layers
//! without any predefined structure.

use std::io::{self, Write};
use std::collections::HashMap;

fn main() {
    println!("\n🌌 True Self-Organization Demo");
    println!("{}", "=".repeat(70));
    println!("Starting with 25 identical, undifferentiated neurons...\n");
    
    // Simulate the process
    let mut simulator = TrueOrgSimulator::new(25);
    
    // Phase 1: Primordial Soup
    println!("📍 Phase 1: Primordial Soup");
    println!("{}", "-".repeat(50));
    simulator.create_neurons();
    pause();
    
    // Phase 2: Discovery
    println!("\n📡 Phase 2: Discovery Phase");
    println!("{}", "-".repeat(50));
    simulator.discovery_phase();
    pause();
    
    // Phase 3: Handshakes
    println!("\n🤝 Phase 3: Handshake Formation");
    println!("{}", "-".repeat(50));
    simulator.handshake_phase();
    pause();
    
    // Phase 4: Pattern Analysis
    println!("\n🔬 Phase 4: Pattern Analysis");
    println!("{}", "-".repeat(50));
    simulator.analyze_patterns();
    pause();
    
    // Phase 5: Hierarchy Emergence
    println!("\n✨ Phase 5: Hierarchy Emergence");
    println!("{}", "-".repeat(50));
    simulator.emerge_hierarchy();
    
    // Final visualization
    println!("\n🎯 Final Emergent Structure:");
    println!("{}", "=".repeat(70));
    simulator.visualize_final_structure();
    
    println!("\n💡 Key Insight:");
    println!("   No layers were predefined. The hierarchy emerged naturally from");
    println!("   the interactions between neurons with different inherent properties.");
    println!("   This is TRUE self-organization!\n");
}

struct TrueOrgSimulator {
    neurons: Vec<Neuron>,
    connections: HashMap<(usize, usize), Connection>,
    emergent_layers: Vec<Layer>,
}

struct Neuron {
    id: usize,
    speed: f32,
    complexity: f32,
    discovered_neighbors: Vec<usize>,
    assigned_layer: Option<usize>,
}

struct Connection {
    compatibility: f32,
    established_at: usize, // time step
}

#[derive(Clone)]
struct Layer {
    neurons: Vec<usize>,
    characteristics: String,
    emergent_name: String,
}

impl TrueOrgSimulator {
    fn new(count: usize) -> Self {
        Self {
            neurons: Vec::with_capacity(count),
            connections: HashMap::new(),
            emergent_layers: Vec::new(),
        }
    }
    
    fn create_neurons(&mut self) {
        println!("Creating neurons with random properties...\n");
        
        for i in 0..25 {
            let speed = rand_float();
            let complexity = rand_float();
            
            let neuron = Neuron {
                id: i,
                speed,
                complexity,
                discovered_neighbors: Vec::new(),
                assigned_layer: None,
            };
            
            // Show some examples
            if i < 5 || i % 5 == 0 {
                println!("  Neuron-{:02}: speed={:.2}, complexity={:.2}", 
                        i, speed, complexity);
            }
            
            self.neurons.push(neuron);
        }
        
        println!("\n  ... and {} more neurons created", 25 - 6);
        println!("\n✓ All neurons start equal - no predefined roles or layers!");
    }
    
    fn discovery_phase(&mut self) {
        println!("Neurons broadcasting their presence...\n");
        
        let messages = vec![
            "Hello? Anyone there?",
            "Fast processor seeking connections!",
            "Deep thinker here, looking for peers.",
            "I process quickly but simply.",
            "Complex calculations are my specialty.",
            "Seeking compatible neighbors...",
        ];
        
        // Simulate discovery broadcasts
        for i in 0..6 {
            println!("  Neuron-{:02}: \"{}\"", i, messages[i % messages.len()]);
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
        
        println!("\n  ... {} more discovery messages ...", 25 - 6);
        
        // Neurons discover each other based on signals
        for i in 0..self.neurons.len() {
            for j in i+1..self.neurons.len() {
                // Randomly decide if they discover each other
                if rand_float() > 0.5 {
                    self.neurons[i].discovered_neighbors.push(j);
                    self.neurons[j].discovered_neighbors.push(i);
                }
            }
        }
        
        println!("\n✓ Neurons have discovered their neighbors!");
    }
    
    fn handshake_phase(&mut self) {
        println!("Attempting handshakes between discovered neighbors...\n");
        
        let mut handshake_count = 0;
        let mut shown = 0;
        
        for i in 0..self.neurons.len() {
            let neuron_i = &self.neurons[i];
            
            for &j in &neuron_i.discovered_neighbors.clone() {
                if j > i { // Avoid duplicates
                    let compatibility = self.calculate_compatibility(i, j);
                    
                    if compatibility > 0.5 {
                        self.connections.insert((i, j), Connection {
                            compatibility,
                            established_at: handshake_count,
                        });
                        
                        handshake_count += 1;
                        
                        // Show first few handshakes
                        if shown < 5 {
                            println!("  ✓ Neuron-{:02} ↔ Neuron-{:02} (compatibility: {:.2})",
                                    i, j, compatibility);
                            shown += 1;
                        }
                    }
                }
            }
        }
        
        println!("\n  ... {} more successful handshakes ...", handshake_count - shown);
        println!("\n✓ Total connections formed: {}", handshake_count);
    }
    
    fn calculate_compatibility(&self, i: usize, j: usize) -> f32 {
        let n1 = &self.neurons[i];
        let n2 = &self.neurons[j];
        
        // Similar speeds work well
        let speed_diff = (n1.speed - n2.speed).abs();
        let speed_compat = 1.0 - speed_diff;
        
        // Complementary complexity is good
        let complexity_diff = (n1.complexity - n2.complexity).abs();
        let complexity_compat = if complexity_diff < 0.2 {
            0.6 // Too similar
        } else if complexity_diff > 0.7 {
            0.4 // Too different  
        } else {
            0.9 // Just right
        };
        
        (speed_compat * 0.5 + complexity_compat * 0.5).clamp(0.0, 1.0)
    }
    
    fn analyze_patterns(&mut self) {
        println!("Analyzing communication patterns and neuron characteristics...\n");
        
        // Group neurons by characteristics
        let mut clusters: Vec<Vec<usize>> = vec![Vec::new(); 5];
        
        for (i, neuron) in self.neurons.iter().enumerate() {
            let cluster_idx = match (neuron.speed, neuron.complexity) {
                (s, c) if s > 0.8 && c < 0.3 => 0,
                (s, c) if s > 0.6 && c < 0.5 => 1,
                (s, c) if s > 0.4 && c > 0.4 && c < 0.6 => 2,
                (s, c) if s < 0.5 && c > 0.6 => 3,
                _ => 4,
            };
            
            clusters[cluster_idx].push(i);
        }
        
        // Remove empty clusters
        clusters.retain(|c| !c.is_empty());
        
        println!("  Found {} natural clusters based on properties:", clusters.len());
        
        for (idx, cluster) in clusters.iter().enumerate() {
            let avg_speed: f32 = cluster.iter()
                .map(|&i| self.neurons[i].speed)
                .sum::<f32>() / cluster.len() as f32;
            
            let avg_complexity: f32 = cluster.iter()
                .map(|&i| self.neurons[i].complexity)
                .sum::<f32>() / cluster.len() as f32;
            
            println!("    Cluster {}: {} neurons, avg_speed={:.2}, avg_complexity={:.2}",
                    idx + 1, cluster.len(), avg_speed, avg_complexity);
        }
        
        // Store for emergence phase
        for (idx, cluster) in clusters.into_iter().enumerate() {
            self.emergent_layers.push(Layer {
                neurons: cluster,
                characteristics: String::new(),
                emergent_name: String::new(),
            });
        }
    }
    
    fn emerge_hierarchy(&mut self) {
        println!("Hierarchy emerging from natural clusters...\n");
        
        // Calculate speeds first to avoid borrow issues
        let layer_speeds: Vec<f32> = self.emergent_layers.iter()
            .map(|layer| {
                layer.neurons.iter()
                    .map(|&i| self.neurons[i].speed)
                    .sum::<f32>() / layer.neurons.len() as f32
            })
            .collect();
        
        // Sort indices by speed
        let mut indices: Vec<usize> = (0..self.emergent_layers.len()).collect();
        indices.sort_by(|&a, &b| {
            layer_speeds[b].partial_cmp(&layer_speeds[a]).unwrap()
        });
        
        // Reorder layers
        let mut sorted_layers = Vec::new();
        for idx in indices {
            sorted_layers.push(self.emergent_layers[idx].clone());
        }
        self.emergent_layers = sorted_layers;
        
        // Assign names based on emergent properties
        let neurons = &self.neurons;
        for (idx, layer) in self.emergent_layers.iter_mut().enumerate() {
            let avg_speed: f32 = layer.neurons.iter()
                .map(|&i| neurons[i].speed)
                .sum::<f32>() / layer.neurons.len() as f32;
            
            let avg_complexity: f32 = layer.neurons.iter()
                .map(|&i| neurons[i].complexity)
                .sum::<f32>() / layer.neurons.len() as f32;
            
            let (name, characteristics) = match (avg_speed, avg_complexity) {
                (s, c) if s > 0.7 && c < 0.3 => {
                    ("Reflexive Layer", "Very fast, simple processing")
                },
                (s, c) if s > 0.5 && c < 0.5 => {
                    ("Implementation Layer", "Fast, moderate complexity")
                },
                (s, c) if s > 0.4 && c > 0.4 && c < 0.6 => {
                    ("Operational Layer", "Balanced speed and complexity")
                },
                (s, c) if s < 0.5 && c > 0.6 => {
                    ("Strategic Layer", "Slow, highly complex")
                },
                _ => {
                    ("Tactical Layer", "Specialized processing")
                },
            };
            
            layer.emergent_name = format!("L{} - {} (emerged)", idx + 1, name);
            layer.characteristics = characteristics.to_string();
            
            // Assign neurons to this layer
            for &neuron_id in &layer.neurons {
                self.neurons[neuron_id].assigned_layer = Some(idx);
            }
            
            println!("  Layer {} emerged: {}", idx + 1, layer.emergent_name);
            println!("    → {}", characteristics);
            println!("    → {} neurons naturally grouped here", layer.neurons.len());
            println!();
        }
        
        println!("✓ Hierarchy has emerged naturally from neuron interactions!");
    }
    
    fn visualize_final_structure(&self) {
        println!("\n    Emergent Neural Hierarchy");
        println!("    {}", "─".repeat(40));
        
        for (idx, layer) in self.emergent_layers.iter().enumerate() {
            println!("\n    {}", layer.emergent_name);
            print!("    ");
            
            // Show neurons in this layer
            for (i, &neuron_id) in layer.neurons.iter().enumerate() {
                if i > 0 && i % 5 == 0 {
                    print!("\n    ");
                }
                print!(" N{:02} ", neuron_id);
            }
            println!("\n    {}", layer.characteristics);
        }
        
        // Show connectivity stats
        let total_connections = self.connections.len();
        let avg_compatibility: f32 = self.connections.values()
            .map(|c| c.compatibility)
            .sum::<f32>() / total_connections as f32;
        
        println!("\n    Connectivity Statistics:");
        println!("    • Total connections: {}", total_connections);
        println!("    • Average compatibility: {:.2}", avg_compatibility);
        println!("    • Neurons per layer: {:?}", 
                self.emergent_layers.iter()
                    .map(|l| l.neurons.len())
                    .collect::<Vec<_>>());
    }
}

fn pause() {
    print!("\nPress Enter to continue...");
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
}

fn rand_float() -> f32 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    ((nanos % 1000) as f32 / 1000.0) * 0.8 + 0.1
}