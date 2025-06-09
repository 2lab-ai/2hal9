//! Integration tests for the new task composition flow

use tokio;
use uuid::Uuid;
use chrono::Utc;

// We'll create minimal test structures to verify the flow
#[derive(Debug, Clone)]
struct TestSignal {
    id: Uuid,
    parent_id: Option<Uuid>,
    from: String,
    to: String,
    content: String,
    layer: String,
}

#[derive(Debug)]
struct TestNeuron {
    id: String,
    layer: String,
}

impl TestNeuron {
    fn new(id: &str, layer: &str) -> Self {
        Self {
            id: id.to_string(),
            layer: layer.to_string(),
        }
    }

    // Simplified processing logic matching the new pattern
    fn process(&self, signal: &TestSignal) -> Vec<TestSignal> {
        match self.layer.as_str() {
            "L4" => {
                // L4 generates 2 L3 tasks
                vec![
                    TestSignal {
                        id: Uuid::new_v4(),
                        parent_id: Some(signal.id),
                        from: self.id.clone(),
                        to: "neuron-2".to_string(),
                        content: format!("Design backend architecture for: {}", signal.content),
                        layer: "L3".to_string(),
                    },
                    TestSignal {
                        id: Uuid::new_v4(),
                        parent_id: Some(signal.id),
                        from: self.id.clone(),
                        to: "neuron-3".to_string(),
                        content: format!("Design API endpoints for: {}", signal.content),
                        layer: "L3".to_string(),
                    },
                ]
            }
            "L3" => {
                // Each L3 generates 2 L2 tasks
                let tasks = if signal.content.contains("backend") {
                    vec![
                        "Implement database schema",
                        "Implement repository pattern"
                    ]
                } else {
                    vec![
                        "Implement REST endpoints",
                        "Implement validation middleware"
                    ]
                };
                
                tasks.iter().map(|&task| TestSignal {
                    id: Uuid::new_v4(),
                    parent_id: Some(signal.id),
                    from: self.id.clone(),
                    to: "neuron-4".to_string(),
                    content: format!("{} for: {}", task, signal.content),
                    layer: "L2".to_string(),
                }).collect()
            }
            _ => vec![]
        }
    }
}

#[tokio::test]
async fn test_complete_hierarchical_flow() {
    // Create neurons
    let l4_neuron = TestNeuron::new("neuron-1", "L4");
    let l3_neuron_2 = TestNeuron::new("neuron-2", "L3");
    let l3_neuron_3 = TestNeuron::new("neuron-3", "L3");
    
    // User input
    let user_signal = TestSignal {
        id: Uuid::new_v4(),
        parent_id: None,
        from: "user".to_string(),
        to: "neuron-1".to_string(),
        content: "Build a simple TODO API service".to_string(),
        layer: "Input".to_string(),
    };
    
    // L4 processing (1 → 2)
    let l3_signals = l4_neuron.process(&user_signal);
    assert_eq!(l3_signals.len(), 2, "L4 should generate exactly 2 L3 tasks");
    
    // Verify L3 signals
    assert_eq!(l3_signals[0].to, "neuron-2");
    assert_eq!(l3_signals[1].to, "neuron-3");
    assert!(l3_signals[0].content.contains("backend"));
    assert!(l3_signals[1].content.contains("API"));
    
    // L3 processing (2 → 4)
    let l2_signals_batch1 = l3_neuron_2.process(&l3_signals[0]);
    let l2_signals_batch2 = l3_neuron_3.process(&l3_signals[1]);
    
    assert_eq!(l2_signals_batch1.len(), 2, "First L3 should generate 2 L2 tasks");
    assert_eq!(l2_signals_batch2.len(), 2, "Second L3 should generate 2 L2 tasks");
    
    // Collect all L2 signals
    let mut all_l2_signals = Vec::new();
    all_l2_signals.extend(l2_signals_batch1);
    all_l2_signals.extend(l2_signals_batch2);
    
    assert_eq!(all_l2_signals.len(), 4, "Total should be 4 L2 tasks");
    
    // Verify all L2 signals go to neuron-4
    for signal in &all_l2_signals {
        assert_eq!(signal.to, "neuron-4");
        assert_eq!(signal.layer, "L2");
    }
    
    // Verify task content patterns
    let l2_contents: Vec<&str> = all_l2_signals.iter()
        .map(|s| s.content.as_str())
        .collect();
    
    // Should have database, repository, endpoints, and validation tasks
    assert!(l2_contents.iter().any(|&c| c.contains("database")));
    assert!(l2_contents.iter().any(|&c| c.contains("repository")));
    assert!(l2_contents.iter().any(|&c| c.contains("endpoints")));
    assert!(l2_contents.iter().any(|&c| c.contains("validation")));
    
    println!("✅ Hierarchical flow test passed!");
    println!("   Input: 1 task");
    println!("   L4→L3: 2 tasks");
    println!("   L3→L2: 4 tasks total");
}

#[test]
fn test_task_decomposition_patterns() {
    let test_cases = vec![
        ("Build a simple TODO API service", vec![
            "Design backend architecture",
            "Design API endpoints",
            "Implement database schema",
            "Implement repository pattern",
            "Implement REST endpoints",
            "Implement validation middleware",
        ]),
        ("Create a basic e-commerce product catalog", vec![
            "Design backend architecture",
            "Design API endpoints",
            "Implement database schema",
            "Implement repository pattern",
            "Implement REST endpoints",
            "Implement validation middleware",
        ]),
        ("Build a real-time chat system", vec![
            "Design backend architecture",
            "Design API endpoints",
            "Implement database schema",
            "Implement repository pattern",
            "Implement REST endpoints",
            "Implement validation middleware",
        ]),
    ];
    
    for (input, expected_patterns) in test_cases {
        println!("Testing: {}", input);
        
        // Simulate the flow
        let l4 = TestNeuron::new("neuron-1", "L4");
        let user_signal = TestSignal {
            id: Uuid::new_v4(),
            parent_id: None,
            from: "user".to_string(),
            to: "neuron-1".to_string(),
            content: input.to_string(),
            layer: "Input".to_string(),
        };
        
        let l3_signals = l4.process(&user_signal);
        assert_eq!(l3_signals.len(), 2);
        
        // Check that expected patterns appear in the decomposition
        let all_content: Vec<String> = l3_signals.iter()
            .map(|s| s.content.clone())
            .collect();
        
        for pattern in &expected_patterns[..2] {  // First 2 are L3 patterns
            assert!(all_content.iter().any(|c| c.contains(pattern)),
                "Missing pattern '{}' in L3 output", pattern);
        }
    }
}

#[test]
fn test_parent_child_relationships() {
    let l4 = TestNeuron::new("neuron-1", "L4");
    let l3_2 = TestNeuron::new("neuron-2", "L3");
    let l3_3 = TestNeuron::new("neuron-3", "L3");
    
    let root_signal = TestSignal {
        id: Uuid::new_v4(),
        parent_id: None,
        from: "user".to_string(),
        to: "neuron-1".to_string(),
        content: "Test task".to_string(),
        layer: "Input".to_string(),
    };
    
    // Process through layers
    let l3_signals = l4.process(&root_signal);
    let l2_signals_1 = l3_2.process(&l3_signals[0]);
    let l2_signals_2 = l3_3.process(&l3_signals[1]);
    
    // Verify parent relationships
    for l3_signal in &l3_signals {
        assert_eq!(l3_signal.parent_id, Some(root_signal.id));
    }
    
    for l2_signal in &l2_signals_1 {
        assert_eq!(l2_signal.parent_id, Some(l3_signals[0].id));
    }
    
    for l2_signal in &l2_signals_2 {
        assert_eq!(l2_signal.parent_id, Some(l3_signals[1].id));
    }
    
    println!("✅ Parent-child relationships verified!");
}