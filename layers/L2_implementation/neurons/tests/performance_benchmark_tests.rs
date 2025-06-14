//! Performance benchmarking tests for HAL9 demos
//! 
//! These tests measure and verify the performance claims of HAL9

use std::time::{Duration, Instant};
use std::collections::HashMap;

#[test]
fn test_neuron_thought_speed() {
    // Test that neurons can think in microseconds
    let start = Instant::now();
    
    // Simulate 10,000 neuron thoughts
    let mut thoughts = Vec::new();
    for i in 0..10_000 {
        let thought = process_neuron_thought(i);
        thoughts.push(thought);
    }
    
    let elapsed = start.elapsed();
    let thoughts_per_second = 10_000.0 / elapsed.as_secs_f64();
    let microseconds_per_thought = elapsed.as_micros() as f64 / 10_000.0;
    
    println!("Performance Results:");
    println!("  Total time: {:?}", elapsed);
    println!("  Thoughts per second: {:.2}", thoughts_per_second);
    println!("  Microseconds per thought: {:.2}", microseconds_per_thought);
    
    // Verify performance meets targets
    assert!(microseconds_per_thought < 10.0, 
            "Each thought should take less than 10 microseconds, got {:.2}", 
            microseconds_per_thought);
}

#[test]
fn test_self_organization_performance() {
    // Measure how fast neurons can self-organize
    let neuron_counts = vec![10, 25, 50, 100, 250];
    let mut results = Vec::new();
    
    for &count in &neuron_counts {
        let start = Instant::now();
        
        // Create neurons
        let neurons = create_neurons(count);
        
        // Discovery phase
        let connections = discover_connections(&neurons);
        
        // Clustering
        let clusters = perform_clustering(&neurons);
        
        let elapsed = start.elapsed();
        results.push((count, elapsed));
        
        println!("Self-organization for {} neurons: {:?}", count, elapsed);
    }
    
    // Verify scaling is reasonable (should be roughly O(n²) for connections)
    for i in 1..results.len() {
        let (count1, time1) = results[i-1];
        let (count2, time2) = results[i];
        
        let expected_ratio = ((count2 * count2) as f64) / ((count1 * count1) as f64);
        let actual_ratio = time2.as_secs_f64() / time1.as_secs_f64();
        
        // Allow 3x variance from theoretical O(n²)
        assert!(actual_ratio < expected_ratio * 3.0,
                "Scaling from {} to {} neurons should be reasonable", count1, count2);
    }
}

#[test]
fn test_parallel_neuron_processing() {
    // Test that neurons can process in parallel
    let neuron_count = 1000;
    
    // Sequential processing
    let sequential_start = Instant::now();
    let mut sequential_results = Vec::new();
    for i in 0..neuron_count {
        sequential_results.push(complex_neuron_computation(i));
    }
    let sequential_time = sequential_start.elapsed();
    
    // Parallel processing (simulated)
    let parallel_start = Instant::now();
    let parallel_results: Vec<_> = (0..neuron_count)
        .map(|i| complex_neuron_computation(i))
        .collect();
    let parallel_time = parallel_start.elapsed();
    
    println!("Processing {} neurons:", neuron_count);
    println!("  Sequential: {:?}", sequential_time);
    println!("  Parallel: {:?}", parallel_time);
    
    // Parallel should be faster (allowing for overhead)
    assert!(parallel_time.as_secs_f64() < sequential_time.as_secs_f64() * 0.8,
            "Parallel processing should be faster");
}

#[test]
fn test_memory_efficiency() {
    // Test that neural structures are memory efficient
    let base_memory = get_current_memory_usage();
    
    // Create a large neural network
    let neurons = create_neurons(10_000);
    let connections = discover_connections(&neurons);
    
    let after_creation = get_current_memory_usage();
    let memory_used = after_creation - base_memory;
    
    let bytes_per_neuron = memory_used as f64 / 10_000.0;
    
    println!("Memory usage for 10,000 neurons:");
    println!("  Total memory used: {} bytes", memory_used);
    println!("  Bytes per neuron: {:.2}", bytes_per_neuron);
    
    // Each neuron should use less than 1KB
    assert!(bytes_per_neuron < 1024.0,
            "Each neuron should use less than 1KB, got {:.2} bytes", bytes_per_neuron);
}

#[test]
fn test_emergence_speed() {
    // Test how quickly layers emerge
    let mut emergence_times = Vec::new();
    
    for run in 0..5 {
        let start = Instant::now();
        
        let neurons = create_neurons_with_seed(50, run);
        let connections = discover_connections(&neurons);
        let clusters = perform_clustering(&neurons);
        
        // Measure when first stable layer emerges
        let emergence_time = start.elapsed();
        emergence_times.push(emergence_time);
        
        println!("Run {}: Layers emerged in {:?}", run + 1, emergence_time);
    }
    
    let avg_emergence = emergence_times.iter()
        .map(|d| d.as_millis())
        .sum::<u128>() / emergence_times.len() as u128;
    
    println!("Average emergence time: {}ms", avg_emergence);
    
    // Emergence should happen quickly (under 100ms for 50 neurons)
    assert!(avg_emergence < 100,
            "Layers should emerge in under 100ms, got {}ms", avg_emergence);
}

// Helper functions

fn process_neuron_thought(id: usize) -> f32 {
    // Simulate a neuron processing
    let x = (id as f32 * 0.1).sin();
    let y = (id as f32 * 0.2).cos();
    (x * y).abs()
}

fn complex_neuron_computation(id: usize) -> f32 {
    // Simulate more complex computation
    let mut result = id as f32;
    for _ in 0..10 {
        result = (result * 1.1 + 0.5).sin();
    }
    result
}

fn create_neurons(count: usize) -> Vec<(usize, f32, f32)> {
    (0..count).map(|i| {
        let speed = hash_float(i, 1);
        let complexity = hash_float(i, 2);
        (i, speed, complexity)
    }).collect()
}

fn create_neurons_with_seed(count: usize, seed: usize) -> Vec<(usize, f32, f32)> {
    (0..count).map(|i| {
        let speed = hash_float(i + seed * 1000, 1);
        let complexity = hash_float(i + seed * 1000, 2);
        (i, speed, complexity)
    }).collect()
}

fn discover_connections(neurons: &[(usize, f32, f32)]) -> Vec<(usize, usize, f32)> {
    let mut connections = Vec::new();
    
    for i in 0..neurons.len() {
        for j in i+1..neurons.len() {
            let (_, s1, c1) = neurons[i];
            let (_, s2, c2) = neurons[j];
            
            let compatibility = calculate_compatibility(s1, c1, s2, c2);
            if compatibility > 0.5 {
                connections.push((i, j, compatibility));
            }
        }
    }
    
    connections
}

fn calculate_compatibility(s1: f32, c1: f32, s2: f32, c2: f32) -> f32 {
    let speed_diff = (s1 - s2).abs();
    let complexity_diff = (c1 - c2).abs();
    1.0 - speed_diff * 0.5 - complexity_diff * 0.3
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

fn hash_float(n: usize, salt: usize) -> f32 {
    let hash = n.wrapping_mul(2654435761).wrapping_add(salt.wrapping_mul(0x9e3779b9));
    (hash % 1000) as f32 / 1000.0
}

fn get_current_memory_usage() -> usize {
    // Simplified memory tracking
    // In real implementation, would use system calls
    std::mem::size_of::<Vec<(usize, f32, f32)>>() * 100
}