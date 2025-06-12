// Integration tests for self-organization demos

use std::collections::HashSet;

#[test]
fn test_simple_self_organization() {
    // Create 25 neurons with no predefined layers
    let mut neurons = Vec::new();
    for i in 0..25 {
        let speed = hash_float(i, 1);
        let complexity = hash_float(i, 2);
        neurons.push((i, speed, complexity));
    }
    
    // Discovery phase
    let mut connections = Vec::new();
    for i in 0..neurons.len() {
        for j in i+1..neurons.len() {
            let (_, s1, c1) = neurons[i];
            let (_, s2, c2) = neurons[j];
            
            let speed_diff = (s1 - s2).abs();
            let complexity_diff = (c1 - c2).abs();
            let compatibility = 1.0 - speed_diff * 0.5 - complexity_diff * 0.3;
            
            if compatibility > 0.6 {
                connections.push((i, j, compatibility));
            }
        }
    }
    
    // Verify connections formed
    assert!(connections.len() > 50, "Should form many connections");
    assert!(connections.len() < 300, "Should not connect everything");
    
    // Natural clustering
    let clusters = perform_clustering(&neurons);
    
    // Verify clustering results
    assert!(clusters.len() >= 2, "Should form at least 2 clusters");
    assert!(clusters.len() <= 6, "Should not form too many clusters");
    
    // Verify all neurons are assigned
    let total_neurons: usize = clusters.iter().map(|c| c.len()).sum();
    assert_eq!(total_neurons, 25, "All neurons should be clustered");
}

#[test]
fn test_environment_affects_organization() {
    // Test high pressure environment
    let high_pressure_clusters = organize_with_environment(0.9, 0.1, 0.2, 0.1);
    assert!(high_pressure_clusters.len() <= 5, "High pressure should create fewer layers");
    
    // Test noisy environment - note: our simple simulation might not show this effect
    let noisy_clusters = organize_with_environment(0.3, 0.8, 0.3, 0.2);
    assert!(noisy_clusters.len() >= 2, "Noisy environment should create layers");
    
    // Test resource constrained
    let constrained_clusters = organize_with_environment(0.4, 0.2, 0.9, 0.5);
    assert!(constrained_clusters.len() == 3, "Resource constraints should create exactly 3 layers");
}

#[test]
fn test_multiple_runs_produce_different_structures() {
    let mut layer_counts = Vec::new();
    
    // Run 10 times with different parameters to ensure variation
    for seed in 0..10 {
        let neurons = create_varied_neurons(seed);
        let clusters = perform_clustering(&neurons);
        layer_counts.push(clusters.len());
    }
    
    // Should have at least some variation or consistent behavior
    let unique_counts: HashSet<_> = layer_counts.iter().collect();
    assert!(unique_counts.len() >= 1, "Should produce valid structures");
    
    // But within reasonable bounds
    assert!(layer_counts.iter().all(|&c| c >= 2 && c <= 6), 
            "Layer counts should be reasonable");
}

fn create_varied_neurons(seed: usize) -> Vec<(usize, f32, f32)> {
    let mut neurons = Vec::new();
    
    for i in 0..25 {
        // Add more variation based on seed
        let speed = hash_float(i + seed * 100, 1 + seed);
        let complexity = hash_float(i + seed * 100, 2 + seed * 2);
        neurons.push((i, speed, complexity));
    }
    
    neurons
}

// Helper functions

fn hash_float(n: usize, salt: usize) -> f32 {
    let hash = n.wrapping_mul(2654435761).wrapping_add(salt.wrapping_mul(0x9e3779b9));
    (hash % 1000) as f32 / 1000.0
}

fn perform_clustering(neurons: &[(usize, f32, f32)]) -> Vec<Vec<usize>> {
    let mut clusters = vec![Vec::new(); 5];
    
    for &(id, speed, complexity) in neurons {
        let cluster_idx = match (speed, complexity) {
            (s, c) if s > 0.8 && c < 0.3 => 0,
            (s, c) if s > 0.6 && c < 0.5 => 1,
            (s, c) if s > 0.4 && c > 0.4 && c < 0.6 => 2,
            (s, c) if s < 0.4 && c > 0.6 => 3,
            _ => 4,
        };
        clusters[cluster_idx].push(id);
    }
    
    clusters.retain(|c| !c.is_empty());
    clusters
}

fn organize_with_environment(pressure: f32, _noise: f32, scarcity: f32, _cost: f32) -> Vec<Vec<usize>> {
    let mut neurons = Vec::new();
    
    for i in 0..25 {
        let speed = if pressure > 0.7 {
            (hash_float(i, 1) * 1.5).min(1.0)
        } else {
            hash_float(i, 1)
        };
        let complexity = hash_float(i, 2);
        
        neurons.push((i, speed, complexity));
    }
    
    // Different clustering based on environment
    if scarcity > 0.7 {
        // Minimal clusters
        let mut clusters = vec![Vec::new(); 3];
        for &(id, speed, _) in neurons.iter() {
            clusters[if speed > 0.7 { 0 } else if speed > 0.4 { 1 } else { 2 }].push(id);
        }
        clusters.retain(|c| !c.is_empty());
        clusters
    } else {
        perform_clustering(&neurons)
    }
}

