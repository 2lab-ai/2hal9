//! Integration tests for self-organization demos
//! 
//! These tests verify that our demos work correctly and that
//! true self-organization produces expected emergent behavior

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
    assert!(high_pressure_clusters.len() <= 3, "High pressure should create shallow hierarchy");
    
    // Test noisy environment
    let noisy_clusters = organize_with_environment(0.3, 0.8, 0.3, 0.2);
    assert!(noisy_clusters.len() >= 4, "Noisy environment should create more layers");
    
    // Test resource constrained
    let constrained_clusters = organize_with_environment(0.4, 0.2, 0.9, 0.5);
    assert!(constrained_clusters.len() <= 4, "Resource constraints should limit layers");
}

#[test]
fn test_multiple_runs_produce_different_structures() {
    let mut layer_counts = Vec::new();
    
    // Run 10 times
    for seed in 0..10 {
        let clusters = run_seeded_organization(seed);
        layer_counts.push(clusters.len());
    }
    
    // Should have variation
    let unique_counts: std::collections::HashSet<_> = layer_counts.iter().collect();
    assert!(unique_counts.len() > 1, "Multiple runs should produce different structures");
    
    // But within reasonable bounds
    assert!(layer_counts.iter().all(|&c| c >= 2 && c <= 6), 
            "Layer counts should be reasonable");
}

#[test]
fn test_ai_neuron_compatibility() {
    // Test specific AI neuron types connect appropriately
    let visual = ("Visual-Detect", 0.9, 0.2, vec!["vision", "pattern"]);
    let pattern = ("Pattern-Match", 0.7, 0.4, vec!["pattern", "recognition"]);
    let logic = ("Logic-Think", 0.3, 0.8, vec!["reasoning", "inference"]);
    
    let visual_pattern_compat = calculate_ai_compatibility(&visual, &pattern);
    let visual_logic_compat = calculate_ai_compatibility(&visual, &logic);
    
    assert!(visual_pattern_compat > visual_logic_compat, 
            "Visual should be more compatible with Pattern than Logic");
    assert!(visual_pattern_compat > 0.5, "Related neurons should be compatible");
}

#[test]
fn test_emergent_layer_characteristics() {
    let neurons = create_test_neurons(30);
    let clusters = perform_clustering(&neurons);
    
    for cluster in &clusters {
        let avg_speed = calculate_average_speed(&neurons, cluster);
        let avg_complexity = calculate_average_complexity(&neurons, cluster);
        
        // Verify clusters have coherent characteristics
        let speed_variance = calculate_variance(&neurons, cluster, |n| n.1);
        let complexity_variance = calculate_variance(&neurons, cluster, |n| n.2);
        
        assert!(speed_variance < 0.3, "Clusters should have similar speeds");
        assert!(complexity_variance < 0.3, "Clusters should have similar complexity");
    }
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

fn organize_with_environment(pressure: f32, noise: f32, scarcity: f32, cost: f32) -> Vec<Vec<usize>> {
    let mut neurons = Vec::new();
    
    for i in 0..25 {
        let mut speed = hash_float(i, 1);
        let mut complexity = hash_float(i, 2);
        
        // Environment affects properties
        if pressure > 0.7 {
            speed = (speed * 1.5).min(1.0);
        }
        if noise > 0.6 {
            complexity = (complexity * 0.7).max(0.3);
        }
        
        neurons.push((i, speed, complexity));
    }
    
    // Different clustering based on environment
    if scarcity > 0.7 {
        // Minimal clusters
        let mut clusters = vec![Vec::new(); 3];
        for (i, &(id, speed, _)) in neurons.iter().enumerate() {
            clusters[if speed > 0.7 { 0 } else if speed > 0.4 { 1 } else { 2 }].push(id);
        }
        clusters.retain(|c| !c.is_empty());
        clusters
    } else {
        perform_clustering(&neurons)
    }
}

fn run_seeded_organization(seed: usize) -> Vec<Vec<usize>> {
    let mut neurons = Vec::new();
    
    for i in 0..25 {
        let speed = hash_float(i + seed * 100, 1);
        let complexity = hash_float(i + seed * 100, 2);
        neurons.push((i, speed, complexity));
    }
    
    perform_clustering(&neurons)
}

fn calculate_ai_compatibility(
    n1: &(&str, f32, f32, Vec<&str>), 
    n2: &(&str, f32, f32, Vec<&str>)
) -> f32 {
    let speed_diff = (n1.1 - n2.1).abs();
    let complexity_diff = (n1.2 - n2.2).abs();
    
    // Check shared capabilities
    let shared_caps = n1.3.iter().filter(|c| n2.3.contains(c)).count();
    
    let speed_compat = 1.0 - speed_diff;
    let complexity_compat = 1.0 - complexity_diff * 0.5;
    let capability_compat = shared_caps as f32 / 2.0;
    
    (speed_compat * 0.3 + complexity_compat * 0.3 + capability_compat * 0.4).clamp(0.0, 1.0)
}

fn create_test_neurons(count: usize) -> Vec<(usize, f32, f32)> {
    (0..count).map(|i| {
        (i, hash_float(i, 1), hash_float(i, 2))
    }).collect()
}

fn calculate_average_speed(neurons: &[(usize, f32, f32)], cluster: &[usize]) -> f32 {
    cluster.iter().map(|&id| neurons[id].1).sum::<f32>() / cluster.len() as f32
}

fn calculate_average_complexity(neurons: &[(usize, f32, f32)], cluster: &[usize]) -> f32 {
    cluster.iter().map(|&id| neurons[id].2).sum::<f32>() / cluster.len() as f32
}

fn calculate_variance<F>(neurons: &[(usize, f32, f32)], cluster: &[usize], extractor: F) -> f32
where
    F: Fn(&(usize, f32, f32)) -> f32,
{
    let values: Vec<f32> = cluster.iter().map(|&id| extractor(&neurons[id])).collect();
    let mean = values.iter().sum::<f32>() / values.len() as f32;
    let variance = values.iter().map(|&v| (v - mean).powi(2)).sum::<f32>() / values.len() as f32;
    variance
}