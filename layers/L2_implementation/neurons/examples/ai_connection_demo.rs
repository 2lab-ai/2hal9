//! AI Connection Demo - Shows how AI neurons would actually connect
//! 
//! This demonstrates true self-organization with AI-like behavior

use std::collections::{HashMap, HashSet};
use std::time::{SystemTime, UNIX_EPOCH};

fn main() {
    println!("\n🤖 AI Self-Organization Demo - How Real AI Neurons Connect");
    println!("{}", "=".repeat(70));
    println!("\nCreating 25 AI neurons with different capabilities...\n");
    
    // Create AI neurons with actual AI-like properties
    let mut neurons = Vec::new();
    let capabilities = vec![
        // Fast response neurons (L1-like)
        ("Vision-Detector-01", vec!["edge_detection", "motion"], 0.9, 0.2),
        ("Audio-Processor-01", vec!["frequency_analysis", "pattern"], 0.85, 0.3),
        ("Touch-Sensor-01", vec!["pressure", "temperature"], 0.95, 0.1),
        ("Reflex-Controller-01", vec!["motor_response", "quick"], 0.9, 0.15),
        ("Alert-System-01", vec!["danger_detection", "alarm"], 0.88, 0.2),
        
        // Medium complexity (L2/L3-like)
        ("Pattern-Recognizer-01", vec!["shape_match", "object_id"], 0.7, 0.5),
        ("Language-Parser-01", vec!["syntax", "grammar"], 0.6, 0.6),
        ("Memory-Indexer-01", vec!["storage", "retrieval"], 0.65, 0.55),
        ("Sequence-Analyzer-01", vec!["temporal", "prediction"], 0.6, 0.65),
        ("Context-Builder-01", vec!["association", "relevance"], 0.55, 0.6),
        
        // Complex reasoning (L4/L5-like)
        ("Logic-Engine-01", vec!["inference", "deduction"], 0.3, 0.85),
        ("Strategy-Planner-01", vec!["goal_planning", "optimization"], 0.25, 0.9),
        ("Abstract-Thinker-01", vec!["conceptual", "metaphor"], 0.2, 0.95),
        ("Ethics-Evaluator-01", vec!["moral_reasoning", "values"], 0.2, 0.9),
        ("Meta-Cognition-01", vec!["self_reflection", "learning"], 0.15, 0.95),
        
        // Specialized processors
        ("Math-Processor-01", vec!["calculation", "numerical"], 0.5, 0.7),
        ("Spatial-Navigator-01", vec!["3d_mapping", "pathfinding"], 0.55, 0.65),
        ("Emotion-Analyzer-01", vec!["sentiment", "empathy"], 0.4, 0.75),
        ("Creative-Generator-01", vec!["imagination", "novelty"], 0.3, 0.8),
        ("Memory-Consolidator-01", vec!["long_term", "compression"], 0.35, 0.7),
        
        // Integration neurons
        ("Signal-Router-01", vec!["message_passing", "protocol"], 0.75, 0.4),
        ("Data-Transformer-01", vec!["format_conversion", "encoding"], 0.7, 0.45),
        ("Priority-Manager-01", vec!["queue", "scheduling"], 0.65, 0.5),
        ("Error-Handler-01", vec!["exception", "recovery"], 0.6, 0.55),
        ("Load-Balancer-01", vec!["distribution", "optimization"], 0.55, 0.6),
    ];
    
    // Phase 1: Initialize neurons
    println!("📊 Phase 1: AI Neuron Initialization");
    println!("{}", "-".repeat(60));
    
    for (i, (name, skills, speed, complexity)) in capabilities.into_iter().enumerate() {
        neurons.push(AINeuron {
            id: i,
            name: name.to_string(),
            capabilities: skills.iter().map(|s| s.to_string()).collect(),
            processing_speed: speed,
            complexity_capacity: complexity,
            discovered_peers: HashSet::new(),
            connections: Vec::new(),
            layer: None, // NOT ASSIGNED! Will emerge
        });
        
        if i < 5 {
            println!("  {} - Skills: {:?}", name, skills);
        }
    }
    println!("  ... and 20 more specialized neurons\n");
    
    // Phase 2: Discovery - neurons find each other
    println!("🔍 Phase 2: Peer Discovery Process");
    println!("{}", "-".repeat(60));
    println!("Neurons broadcasting their capabilities...\n");
    
    // Simulate discovery through capability matching
    for i in 0..neurons.len() {
        for j in i+1..neurons.len() {
            let compatibility = calculate_ai_compatibility(&neurons[i], &neurons[j]);
            
            if compatibility > 0.5 {
                // They discover each other!
                neurons[i].discovered_peers.insert(j);
                neurons[j].discovered_peers.insert(i);
                
                // Show some discoveries
                if neurons[i].connections.len() < 2 && neurons[j].connections.len() < 2 {
                    println!("  💬 {} discovers {}", 
                            neurons[i].name, neurons[j].name);
                    println!("     Compatibility: {:.2} (shared interests: {:?})",
                            compatibility, 
                            find_shared_interests(&neurons[i], &neurons[j]));
                }
            }
        }
    }
    
    // Phase 3: Connection formation
    println!("\n🤝 Phase 3: Connection Formation");
    println!("{}", "-".repeat(60));
    println!("Neurons negotiating connections based on needs...\n");
    
    let mut connection_count = 0;
    for i in 0..neurons.len() {
        let peers: Vec<usize> = neurons[i].discovered_peers.iter().copied().collect();
        
        for &j in &peers {
            if j > i { // Avoid duplicates
                let compatibility = calculate_ai_compatibility(&neurons[i], &neurons[j]);
                
                // Decision to connect based on AI logic
                let should_connect = decide_connection(&neurons[i], &neurons[j], compatibility);
                
                if should_connect {
                    neurons[i].connections.push((j, compatibility));
                    neurons[j].connections.push((i, compatibility));
                    connection_count += 1;
                    
                    if connection_count <= 5 {
                        println!("  ✓ {} ↔ {}", neurons[i].name, neurons[j].name);
                        println!("    Reason: {}", 
                                connection_reason(&neurons[i], &neurons[j]));
                    }
                }
            }
        }
    }
    println!("\n  Total connections formed: {}", connection_count);
    
    // Phase 4: Natural organization emerges
    println!("\n🌟 Phase 4: Emergent Layer Organization");
    println!("{}", "-".repeat(60));
    println!("Analyzing communication patterns...\n");
    
    // Cluster based on actual communication patterns
    let clusters = cluster_by_ai_behavior(&neurons);
    
    println!("Discovered {} natural clusters:\n", clusters.len());
    
    for (idx, cluster) in clusters.iter().enumerate() {
        let cluster_type = identify_cluster_type(&neurons, cluster);
        println!("  Layer {} - {} ({} neurons)", 
                idx + 1, cluster_type.name, cluster.len());
        println!("    Characteristics: {}", cluster_type.description);
        println!("    Members: {:?}", 
                cluster.iter()
                    .take(3)
                    .map(|&i| &neurons[i].name[..15])
                    .collect::<Vec<_>>());
        if cluster.len() > 3 {
            println!("    ... and {} more", cluster.len() - 3);
        }
        println!();
    }
    
    // Show the emergent architecture
    println!("🏗️ Emergent AI Architecture:");
    println!("{}", "=".repeat(60));
    visualize_ai_hierarchy(&neurons, &clusters);
    
    println!("\n💡 Key Insights:");
    println!("  1. Neurons found each other through capability broadcasting");
    println!("  2. Connections formed based on complementary skills");
    println!("  3. Layers emerged from communication patterns");
    println!("  4. NO predefined hierarchy - structure emerged naturally!");
    
    println!("\n🚀 How to Run This Demo:");
    println!("  1. Save this file as 'ai_connection_demo.rs'");
    println!("  2. Compile: rustc --edition 2021 ai_connection_demo.rs");
    println!("  3. Run: ./ai_connection_demo");
    println!("\n  Or in one line:");
    println!("  rustc --edition 2021 ai_connection_demo.rs && ./ai_connection_demo");
}

struct AINeuron {
    id: usize,
    name: String,
    capabilities: Vec<String>,
    processing_speed: f32,
    complexity_capacity: f32,
    discovered_peers: HashSet<usize>,
    connections: Vec<(usize, f32)>,
    layer: Option<usize>, // Emerges, not assigned!
}

struct ClusterType {
    name: String,
    description: String,
}

fn calculate_ai_compatibility(n1: &AINeuron, n2: &AINeuron) -> f32 {
    // Skill overlap
    let shared_interests = find_shared_interests(n1, n2).len() as f32;
    let skill_compatibility = shared_interests / 4.0; // Max 0.5
    
    // Complementary speeds (fast + slow can work together)
    let speed_diff = (n1.processing_speed - n2.processing_speed).abs();
    let speed_compatibility = if speed_diff > 0.3 && speed_diff < 0.7 {
        0.3 // Complementary
    } else if speed_diff < 0.2 {
        0.2 // Similar
    } else {
        0.1 // Too different
    };
    
    // Complexity synergy
    let complexity_compatibility = 
        1.0 - (n1.complexity_capacity - n2.complexity_capacity).abs() * 0.5;
    
    (skill_compatibility + speed_compatibility + complexity_compatibility * 0.2)
        .clamp(0.0, 1.0)
}

fn find_shared_interests(n1: &AINeuron, n2: &AINeuron) -> Vec<String> {
    // Find conceptual overlaps
    let mut shared = Vec::new();
    
    for cap1 in &n1.capabilities {
        for cap2 in &n2.capabilities {
            if related_capabilities(cap1, cap2) {
                shared.push(format!("{}-{}", cap1, cap2));
            }
        }
    }
    
    shared
}

fn related_capabilities(cap1: &str, cap2: &str) -> bool {
    // Define which capabilities work well together
    let relationships = vec![
        ("edge_detection", "shape_match"),
        ("motion", "temporal"),
        ("pattern", "sequence"),
        ("syntax", "grammar"),
        ("storage", "retrieval"),
        ("inference", "deduction"),
        ("goal_planning", "optimization"),
        ("message_passing", "protocol"),
        ("format_conversion", "encoding"),
    ];
    
    relationships.iter().any(|(a, b)| 
        (cap1 == *a && cap2 == *b) || (cap1 == *b && cap2 == *a)
    )
}

fn decide_connection(n1: &AINeuron, n2: &AINeuron, compatibility: f32) -> bool {
    // AI neurons decide to connect based on:
    // 1. Compatibility score
    // 2. Current connection load
    // 3. Complementary capabilities
    
    if compatibility < 0.4 {
        return false;
    }
    
    // Don't overconnect
    if n1.connections.len() > 8 || n2.connections.len() > 8 {
        return compatibility > 0.7; // Only very compatible
    }
    
    // Fast neurons connect more readily
    if n1.processing_speed > 0.8 || n2.processing_speed > 0.8 {
        return compatibility > 0.45;
    }
    
    true
}

fn connection_reason(n1: &AINeuron, n2: &AINeuron) -> String {
    let speed_diff = (n1.processing_speed - n2.processing_speed).abs();
    
    if speed_diff < 0.2 {
        "Similar processing speeds for synchronized work"
    } else if n1.processing_speed > 0.7 && n2.complexity_capacity > 0.7 {
        "Fast processor needs complex analysis"
    } else if find_shared_interests(n1, n2).len() > 1 {
        "Multiple shared capabilities"
    } else {
        "Complementary skill sets"
    }.to_string()
}

fn cluster_by_ai_behavior(neurons: &[AINeuron]) -> Vec<Vec<usize>> {
    // Cluster based on actual connection patterns and capabilities
    let mut clusters: Vec<Vec<usize>> = Vec::new();
    let mut assigned = vec![false; neurons.len()];
    
    // Find natural groups through connection density
    for i in 0..neurons.len() {
        if assigned[i] {
            continue;
        }
        
        let mut cluster = vec![i];
        assigned[i] = true;
        
        // Find strongly connected peers
        for j in 0..neurons.len() {
            if i != j && !assigned[j] {
                let connection_strength = measure_connection_strength(neurons, i, j);
                if connection_strength > 0.6 {
                    cluster.push(j);
                    assigned[j] = true;
                }
            }
        }
        
        clusters.push(cluster);
    }
    
    // Merge small clusters
    clusters.retain(|c| c.len() > 1);
    
    // Sort by average speed (fast to slow)
    clusters.sort_by(|a, b| {
        let speed_a: f32 = a.iter().map(|&i| neurons[i].processing_speed).sum::<f32>() / a.len() as f32;
        let speed_b: f32 = b.iter().map(|&i| neurons[i].processing_speed).sum::<f32>() / b.len() as f32;
        speed_b.partial_cmp(&speed_a).unwrap()
    });
    
    clusters
}

fn measure_connection_strength(neurons: &[AINeuron], i: usize, j: usize) -> f32 {
    // Direct connection?
    if let Some((_, strength)) = neurons[i].connections.iter().find(|(idx, _)| *idx == j) {
        return *strength;
    }
    
    // Indirect connection through shared peers?
    let shared_connections = neurons[i].discovered_peers
        .intersection(&neurons[j].discovered_peers)
        .count();
    
    shared_connections as f32 / 10.0
}

fn identify_cluster_type(neurons: &[AINeuron], cluster: &[usize]) -> ClusterType {
    let avg_speed: f32 = cluster.iter()
        .map(|&i| neurons[i].processing_speed)
        .sum::<f32>() / cluster.len() as f32;
    
    let avg_complexity: f32 = cluster.iter()
        .map(|&i| neurons[i].complexity_capacity)
        .sum::<f32>() / cluster.len() as f32;
    
    let primary_capabilities: Vec<String> = cluster.iter()
        .flat_map(|&i| neurons[i].capabilities.clone())
        .collect();
    
    match (avg_speed, avg_complexity) {
        (s, c) if s > 0.8 && c < 0.3 => ClusterType {
            name: "Sensory & Reflex Layer".to_string(),
            description: "Ultra-fast response, direct sensory processing".to_string(),
        },
        (s, c) if s > 0.6 && c < 0.5 => ClusterType {
            name: "Pattern Recognition Layer".to_string(),
            description: "Fast pattern matching and initial processing".to_string(),
        },
        (s, c) if s > 0.4 && c > 0.5 && c < 0.7 => ClusterType {
            name: "Integration Layer".to_string(),
            description: "Combines inputs, manages data flow".to_string(),
        },
        (s, c) if s < 0.4 && c > 0.7 => ClusterType {
            name: "Reasoning Layer".to_string(),
            description: "Deep thinking, planning, abstract concepts".to_string(),
        },
        _ => ClusterType {
            name: "Specialized Processing".to_string(),
            description: "Domain-specific computation".to_string(),
        },
    }
}

fn visualize_ai_hierarchy(neurons: &[AINeuron], clusters: &[Vec<usize>]) {
    println!("\n    [Input] → Real-world data, sensory input");
    println!("       ↓");
    
    for (idx, cluster) in clusters.iter().enumerate() {
        let cluster_type = identify_cluster_type(neurons, cluster);
        
        println!("    ┌─ Layer {} ─────────────────────┐", idx + 1);
        println!("    │ {}          │", cluster_type.name);
        println!("    │ {} neurons, {} connections │", 
                cluster.len(),
                cluster.iter()
                    .map(|&i| neurons[i].connections.len())
                    .sum::<usize>());
        println!("    └────────────────────────────────┘");
        
        if idx < clusters.len() - 1 {
            println!("       ↓ {:>3} inter-layer connections", 
                    count_interlayer_connections(neurons, cluster, &clusters[idx + 1]));
        }
    }
    
    println!("       ↓");
    println!("    [Output] → Actions, decisions, responses");
}

fn count_interlayer_connections(neurons: &[AINeuron], layer1: &[usize], layer2: &[usize]) -> usize {
    let mut count = 0;
    for &i in layer1 {
        for &(j, _) in &neurons[i].connections {
            if layer2.contains(&j) {
                count += 1;
            }
        }
    }
    count
}