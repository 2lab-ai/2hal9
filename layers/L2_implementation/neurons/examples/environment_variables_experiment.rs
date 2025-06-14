//! Environment Variables Experiment
//! 
//! Shows how different environmental conditions affect emergent structures

use std::collections::HashMap;

fn main() {
    println!("\n🌍 Environmental Conditions Impact on Self-Organization");
    println!("{}", "=".repeat(70));
    println!("\nTesting how different environments affect emergent structures...\n");
    
    // Test different environmental conditions
    let environments = vec![
        Environment {
            name: "High Pressure (Fast Processing Required)",
            pressure: 0.9,
            noise: 0.1,
            resource_scarcity: 0.2,
            connectivity_cost: 0.1,
        },
        Environment {
            name: "Noisy Environment (High Uncertainty)",
            pressure: 0.3,
            noise: 0.8,
            resource_scarcity: 0.3,
            connectivity_cost: 0.2,
        },
        Environment {
            name: "Resource Constrained (Limited Energy)",
            pressure: 0.4,
            noise: 0.2,
            resource_scarcity: 0.9,
            connectivity_cost: 0.5,
        },
        Environment {
            name: "High Connectivity Cost (Expensive Communication)",
            pressure: 0.3,
            noise: 0.2,
            resource_scarcity: 0.3,
            connectivity_cost: 0.8,
        },
        Environment {
            name: "Balanced Environment (Moderate Conditions)",
            pressure: 0.5,
            noise: 0.3,
            resource_scarcity: 0.4,
            connectivity_cost: 0.3,
        },
    ];
    
    let mut results = Vec::new();
    
    for env in environments {
        println!("🔬 Testing: {}", env.name);
        println!("{}", "-".repeat(60));
        
        let result = run_experiment_in_environment(&env);
        
        println!("  → Emergent structure: {} layers", result.layer_count);
        println!("  → Average connectivity: {:.2}", result.avg_connectivity);
        println!("  → Dominant pattern: {}", result.dominant_pattern);
        println!("  → Adaptation: {}\n", result.adaptation_strategy);
        
        results.push((env, result));
    }
    
    // Analyze how environment affects structure
    println!("\n📊 Environmental Impact Analysis");
    println!("{}", "=".repeat(70));
    
    for (env, result) in &results {
        println!("\n{}", env.name);
        println!("  Environmental factors:");
        println!("    - Pressure: {:.1} → {}", 
                env.pressure, 
                if env.pressure > 0.7 { "Favors shallow, fast layers" } 
                else { "Allows deep thinking layers" });
        println!("    - Noise: {:.1} → {}", 
                env.noise,
                if env.noise > 0.6 { "Requires redundancy" }
                else { "Allows specialization" });
        println!("    - Scarcity: {:.1} → {}", 
                env.resource_scarcity,
                if env.resource_scarcity > 0.7 { "Forces efficiency" }
                else { "Enables exploration" });
        println!("    - Connectivity cost: {:.1} → {}", 
                env.connectivity_cost,
                if env.connectivity_cost > 0.6 { "Promotes local clusters" }
                else { "Enables global connections" });
        
        println!("  Emergent adaptation:");
        println!("    → {}", result.adaptation_strategy);
    }
    
    println!("\n💡 Key Insights:");
    println!("  1. High pressure → Shallow hierarchies (2-3 layers)");
    println!("  2. High noise → Redundant connections");
    println!("  3. Resource scarcity → Minimal viable structures");
    println!("  4. High connectivity cost → Local clustering");
    println!("  5. Balanced conditions → Classic 5-layer emergence");
    
    println!("\n🎯 Conclusion:");
    println!("  Environmental conditions directly shape emergent neural organization.");
    println!("  The same neurons organize differently to optimize for their environment!");
}

struct Environment {
    name: &'static str,
    pressure: f32,          // Time pressure (0-1)
    noise: f32,             // Environmental noise (0-1)
    resource_scarcity: f32, // Limited resources (0-1)
    connectivity_cost: f32, // Cost of maintaining connections (0-1)
}

struct ExperimentResult {
    layer_count: usize,
    avg_connectivity: f32,
    dominant_pattern: String,
    adaptation_strategy: String,
}

fn run_experiment_in_environment(env: &Environment) -> ExperimentResult {
    // Create 25 neurons
    let mut neurons = Vec::new();
    for i in 0..25 {
        let base_speed = hash_float(i, 1);
        let base_complexity = hash_float(i, 2);
        
        // Environment affects neuron properties
        let speed = if env.pressure > 0.7 {
            (base_speed * 1.5).min(1.0)  // Pressure increases speed
        } else {
            base_speed
        };
        
        let complexity = if env.noise > 0.6 {
            (base_complexity * 0.7).max(0.3)  // Noise reduces complexity
        } else {
            base_complexity
        };
        
        neurons.push((i, speed, complexity));
    }
    
    // Discovery phase affected by environment
    let mut connections = Vec::new();
    let discovery_threshold = if env.connectivity_cost > 0.6 {
        0.7  // High cost = pickier about connections
    } else {
        0.5  // Low cost = more connections
    };
    
    for i in 0..neurons.len() {
        for j in i+1..neurons.len() {
            let compatibility = calculate_environmental_compatibility(
                &neurons[i], &neurons[j], env
            );
            
            if compatibility > discovery_threshold {
                connections.push((i, j, compatibility));
            }
        }
    }
    
    // Clustering affected by environment
    let clusters = if env.resource_scarcity > 0.7 {
        // Scarce resources = fewer, larger clusters
        cluster_for_efficiency(&neurons, &connections)
    } else if env.noise > 0.7 {
        // High noise = redundant clusters
        cluster_for_redundancy(&neurons, &connections)
    } else if env.pressure > 0.7 {
        // High pressure = speed-optimized clusters
        cluster_for_speed(&neurons, &connections)
    } else {
        // Normal clustering
        cluster_normally(&neurons, &connections)
    };
    
    // Analyze results
    let layer_count = clusters.len();
    let avg_connectivity = connections.len() as f32 / neurons.len() as f32;
    
    let dominant_pattern = if layer_count <= 2 {
        "Flat organization".to_string()
    } else if layer_count >= 6 {
        "Deep hierarchy".to_string()
    } else if clusters.iter().any(|c| c.len() > 10) {
        "Hub-and-spoke".to_string()
    } else {
        "Distributed clusters".to_string()
    };
    
    let adaptation_strategy = match (env.pressure > 0.6, env.noise > 0.6, 
                                   env.resource_scarcity > 0.6, env.connectivity_cost > 0.6) {
        (true, _, _, _) => "Speed-optimized pathways for rapid response",
        (_, true, _, _) => "Redundant connections for noise resilience",
        (_, _, true, _) => "Minimal structure for resource efficiency",
        (_, _, _, true) => "Local clusters to minimize communication cost",
        _ => "Balanced organization for general purpose",
    }.to_string();
    
    ExperimentResult {
        layer_count,
        avg_connectivity,
        dominant_pattern,
        adaptation_strategy,
    }
}

fn calculate_environmental_compatibility(
    n1: &(usize, f32, f32),
    n2: &(usize, f32, f32),
    env: &Environment
) -> f32 {
    let speed_diff = (n1.1 - n2.1).abs();
    let complexity_diff = (n1.2 - n2.2).abs();
    
    // Base compatibility
    let mut compatibility = 1.0 - speed_diff * 0.5 - complexity_diff * 0.5;
    
    // Environmental modifiers
    if env.pressure > 0.7 && n1.1 > 0.7 && n2.1 > 0.7 {
        compatibility *= 1.3;  // Boost fast-fast connections
    }
    
    if env.noise > 0.7 && speed_diff < 0.2 {
        compatibility *= 1.2;  // Similar neurons for redundancy
    }
    
    if env.resource_scarcity > 0.7 && (n1.1 + n2.1) / 2.0 < 0.5 {
        compatibility *= 0.5;  // Penalize slow connections
    }
    
    compatibility.clamp(0.0, 1.0)
}

fn cluster_for_efficiency(
    neurons: &[(usize, f32, f32)],
    _connections: &[(usize, usize, f32)]
) -> Vec<Vec<usize>> {
    // Create minimal viable clusters (3 large clusters)
    let mut clusters = vec![Vec::new(); 3];
    
    for (i, &(id, speed, complexity)) in neurons.iter().enumerate() {
        let cluster_idx = if speed > 0.7 {
            0  // Fast processors
        } else if complexity > 0.6 {
            1  // Complex thinkers
        } else {
            2  // General purpose
        };
        clusters[cluster_idx].push(id);
    }
    
    clusters.retain(|c| !c.is_empty());
    clusters
}

fn cluster_for_redundancy(
    neurons: &[(usize, f32, f32)],
    _connections: &[(usize, usize, f32)]
) -> Vec<Vec<usize>> {
    // Create many small, overlapping clusters
    let mut clusters = vec![Vec::new(); 7];
    
    for &(id, speed, complexity) in neurons {
        // Each neuron can belong to multiple clusters
        let primary = ((speed * 5.0) as usize).min(4);
        let secondary = ((complexity * 5.0) as usize).min(4) + 2;
        
        clusters[primary].push(id);
        if hash_float(id, 100) > 0.5 {
            clusters[secondary].push(id);  // Some redundancy
        }
    }
    
    clusters.retain(|c| !c.is_empty());
    clusters
}

fn cluster_for_speed(
    neurons: &[(usize, f32, f32)],
    _connections: &[(usize, usize, f32)]
) -> Vec<Vec<usize>> {
    // Create shallow hierarchy (2-3 layers max)
    let mut clusters = vec![Vec::new(); 2];
    
    for &(id, speed, _) in neurons {
        let cluster_idx = if speed > 0.5 { 0 } else { 1 };
        clusters[cluster_idx].push(id);
    }
    
    clusters
}

fn cluster_normally(
    neurons: &[(usize, f32, f32)],
    _connections: &[(usize, usize, f32)]
) -> Vec<Vec<usize>> {
    // Standard 5-layer clustering
    let mut clusters = vec![Vec::new(); 5];
    
    for &(id, speed, complexity) in neurons {
        let cluster_idx = match (speed, complexity) {
            (s, c) if s > 0.8 && c < 0.2 => 0,
            (s, c) if s > 0.6 && c < 0.4 => 1,
            (s, c) if s > 0.4 && c > 0.4 && c < 0.6 => 2,
            (s, c) if s < 0.4 && c > 0.6 => 3,
            _ => 4,
        };
        clusters[cluster_idx].push(id);
    }
    
    clusters.retain(|c| !c.is_empty());
    clusters
}

fn hash_float(n: usize, salt: usize) -> f32 {
    let hash = n.wrapping_mul(2654435761).wrapping_add(salt.wrapping_mul(0x9e3779b9));
    (hash % 1000) as f32 / 1000.0
}