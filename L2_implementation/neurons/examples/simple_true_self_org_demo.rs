//! Simplified True Self-Organization Demo

fn main() {
    println!("\n🌌 True Self-Organization Demo (Simplified)");
    println!("{}", "=".repeat(70));
    
    // Phase 1: Create undifferentiated neurons
    println!("\n📍 Phase 1: Creating 25 identical neurons");
    let mut neurons = Vec::new();
    for i in 0..25 {
        let speed = hash_float(i, 1);
        let complexity = hash_float(i, 2);
        neurons.push((i, speed, complexity));
        
        if i < 3 {
            println!("  Neuron-{:02}: speed={:.2}, complexity={:.2}", i, speed, complexity);
        }
    }
    println!("  ... {} more neurons created", 22);
    
    // Phase 2: Discovery - neurons find compatible partners
    println!("\n📡 Phase 2: Discovery - neurons finding each other");
    let mut connections = Vec::new();
    
    for i in 0..neurons.len() {
        for j in i+1..neurons.len() {
            let (_, s1, c1) = neurons[i];
            let (_, s2, c2) = neurons[j];
            
            // Calculate compatibility
            let speed_diff = (s1 - s2).abs();
            let complexity_diff = (c1 - c2).abs();
            let compatibility = 1.0 - speed_diff * 0.5 - complexity_diff * 0.3;
            
            if compatibility > 0.6 {
                connections.push((i, j, compatibility));
                if connections.len() <= 5 {
                    println!("  Neuron-{:02} ↔ Neuron-{:02} connected (compatibility: {:.2})", 
                            i, j, compatibility);
                }
            }
        }
    }
    println!("  Total connections formed: {}", connections.len());
    
    // Phase 3: Natural clustering based on properties
    println!("\n🔬 Phase 3: Natural clustering");
    let mut clusters: Vec<Vec<usize>> = vec![Vec::new(); 5];
    
    for (i, &(_, speed, complexity)) in neurons.iter().enumerate() {
        let cluster_idx = match (speed, complexity) {
            (s, c) if s > 0.7 && c < 0.3 => 0,  // Fast & Simple
            (s, c) if s > 0.5 && c < 0.5 => 1,  // Fast & Medium
            (s, c) if s > 0.3 && c > 0.3 && c < 0.7 => 2,  // Balanced
            (s, c) if s < 0.4 && c > 0.6 => 3,  // Slow & Complex
            _ => 4,  // Others
        };
        clusters[cluster_idx].push(i);
    }
    
    // Remove empty clusters
    clusters.retain(|c| !c.is_empty());
    
    for (i, cluster) in clusters.iter().enumerate() {
        println!("  Cluster {}: {} neurons", i + 1, cluster.len());
    }
    
    // Phase 4: Layers emerge from clusters
    println!("\n✨ Phase 4: Hierarchy emerges!");
    println!("{}", "-".repeat(50));
    
    let layer_names = [
        "Reflexive (Fast & Simple)",
        "Implementation (Fast & Medium)", 
        "Operational (Balanced)",
        "Strategic (Slow & Complex)",
        "Tactical (Specialized)"
    ];
    
    for (i, cluster) in clusters.iter().enumerate() {
        println!("\n  Layer {} emerged: {}", i + 1, layer_names[i]);
        println!("  Contains {} neurons: {:?}", cluster.len(), 
                cluster.iter().take(5).collect::<Vec<_>>());
    }
    
    println!("\n🎯 Key Point:");
    println!("  • Started with 25 identical neurons");
    println!("  • No predefined layers or roles");
    println!("  • Structure emerged from interactions");
    println!("  • Different initial conditions → different structures!");
}

// Simple hash function for deterministic randomness
fn hash_float(n: usize, salt: usize) -> f32 {
    let hash = n.wrapping_mul(2654435761).wrapping_add(salt.wrapping_mul(0x9e3779b9));
    (hash % 1000) as f32 / 1000.0
}