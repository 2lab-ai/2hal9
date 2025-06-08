//! Test cases for minimum task composition flow (L4→L3→L2)
//! 
//! This module tests the hierarchical task decomposition:
//! - L4 (Strategic): 1 task
//! - L3 (Design): 2 tasks (from L4)
//! - L2 (Implementation): 4 tasks total (2 from each L3)

use chrono::Utc;
use uuid::Uuid;
use std::collections::HashMap;
use tokio::sync::mpsc;

// Import from main module
use hal9_mvp::{Signal, MockNeuron};

/// Test data structure for task composition
#[derive(Debug, Clone)]
pub struct TaskCompositionTest {
    pub user_prompt: String,
    pub expected_l4_output: String,
    pub expected_l3_outputs: Vec<String>,
    pub expected_l2_outputs: Vec<String>,
}

/// Get the standard test case for TODO API
pub fn get_todo_api_test_case() -> TaskCompositionTest {
    TaskCompositionTest {
        user_prompt: "Build a simple TODO API service".to_string(),
        expected_l4_output: "Design and implement a TODO API service with CRUD operations".to_string(),
        expected_l3_outputs: vec![
            "Design backend architecture and data model for TODO service".to_string(),
            "Design API endpoints and request/response schemas".to_string(),
        ],
        expected_l2_outputs: vec![
            // From L3 Task 1
            "Implement database schema with id, title, description, completed, created_at fields".to_string(),
            "Implement repository pattern with create, read, update, delete methods".to_string(),
            // From L3 Task 2
            "Implement REST endpoints: POST /todos, GET /todos, PUT /todos/:id, DELETE /todos/:id".to_string(),
            "Implement validation middleware and error handling for API requests".to_string(),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::mpsc;

    #[tokio::test]
    async fn test_l4_to_l3_decomposition() {
        let test_case = get_todo_api_test_case();
        
        // Create L4 neuron
        let l4_neuron = MockNeuron::new("neuron-1", "L4");
        
        // Create input signal
        let input_signal = Signal {
            id: Uuid::new_v4(),
            parent_id: None,
            from: "user".to_string(),
            to: "neuron-1".to_string(),
            content: test_case.user_prompt.clone(),
            layer: "Input".to_string(),
            timestamp: Utc::now(),
        };
        
        // Process signal
        let l3_signals = l4_neuron.process(&input_signal).await;
        
        // Verify we get exactly 2 L3 signals
        assert_eq!(l3_signals.len(), 2, "L4 should decompose into exactly 2 L3 tasks");
        
        // Verify parent-child relationships
        for signal in &l3_signals {
            assert_eq!(signal.parent_id, Some(input_signal.id));
            assert_eq!(signal.from, "neuron-1");
            assert_eq!(signal.layer, "L3");
        }
        
        // Verify the signals go to different L3 neurons
        assert_eq!(l3_signals[0].to, "neuron-2");
        assert_eq!(l3_signals[1].to, "neuron-3");
    }

    #[tokio::test]
    async fn test_l3_to_l2_decomposition() {
        let test_case = get_todo_api_test_case();
        
        // Create L3 neurons
        let l3_neuron_2 = MockNeuron::new("neuron-2", "L3");
        let l3_neuron_3 = MockNeuron::new("neuron-3", "L3");
        
        // Create L3 input signals
        let parent_id = Uuid::new_v4();
        let l3_signal_1 = Signal {
            id: Uuid::new_v4(),
            parent_id: Some(parent_id),
            from: "neuron-1".to_string(),
            to: "neuron-2".to_string(),
            content: "Design backend architecture and data model for TODO service".to_string(),
            layer: "L3".to_string(),
            timestamp: Utc::now(),
        };
        
        let l3_signal_2 = Signal {
            id: Uuid::new_v4(),
            parent_id: Some(parent_id),
            from: "neuron-1".to_string(),
            to: "neuron-3".to_string(),
            content: "Design API endpoints and request/response schemas".to_string(),
            layer: "L3".to_string(),
            timestamp: Utc::now(),
        };
        
        // Process both L3 signals
        let l2_signals_1 = l3_neuron_2.process(&l3_signal_1).await;
        let l2_signals_2 = l3_neuron_3.process(&l3_signal_2).await;
        
        // Each L3 should generate 2 L2 tasks
        assert_eq!(l2_signals_1.len(), 2, "First L3 should generate 2 L2 tasks");
        assert_eq!(l2_signals_2.len(), 2, "Second L3 should generate 2 L2 tasks");
        
        // Verify all go to L2 (neuron-4)
        for signal in l2_signals_1.iter().chain(l2_signals_2.iter()) {
            assert_eq!(signal.to, "neuron-4");
            assert_eq!(signal.layer, "L2");
        }
    }

    #[tokio::test]
    async fn test_complete_task_composition_flow() {
        let test_case = get_todo_api_test_case();
        let (tx, mut rx) = mpsc::channel::<Signal>(100);
        
        // Track all signals
        let mut all_signals = Vec::new();
        
        // Create neurons
        let l4_neuron = MockNeuron::new("neuron-1", "L4");
        let l3_neuron_2 = MockNeuron::new("neuron-2", "L3");
        let l3_neuron_3 = MockNeuron::new("neuron-3", "L3");
        let l2_neuron = MockNeuron::new("neuron-4", "L2");
        
        // Start with user input
        let user_signal = Signal {
            id: Uuid::new_v4(),
            parent_id: None,
            from: "user".to_string(),
            to: "neuron-1".to_string(),
            content: test_case.user_prompt.clone(),
            layer: "Input".to_string(),
            timestamp: Utc::now(),
        };
        all_signals.push(user_signal.clone());
        
        // L4 processing
        let l3_signals = l4_neuron.process(&user_signal).await;
        assert_eq!(l3_signals.len(), 2);
        all_signals.extend(l3_signals.clone());
        
        // L3 processing (sequential)
        let mut l2_signals = Vec::new();
        
        // Process first L3
        let l2_batch_1 = l3_neuron_2.process(&l3_signals[0]).await;
        assert_eq!(l2_batch_1.len(), 2);
        l2_signals.extend(l2_batch_1);
        
        // Process second L3
        let l2_batch_2 = l3_neuron_3.process(&l3_signals[1]).await;
        assert_eq!(l2_batch_2.len(), 2);
        l2_signals.extend(l2_batch_2);
        
        // Total should be 4 L2 tasks
        assert_eq!(l2_signals.len(), 4);
        all_signals.extend(l2_signals);
        
        // Verify the complete hierarchy
        let input_count = all_signals.iter().filter(|s| s.layer == "Input").count();
        let l3_count = all_signals.iter().filter(|s| s.layer == "L3").count();
        let l2_count = all_signals.iter().filter(|s| s.layer == "L2").count();
        
        assert_eq!(input_count, 1);
        assert_eq!(l3_count, 2);
        assert_eq!(l2_count, 4);
        
        // Verify parent-child relationships
        verify_signal_hierarchy(&all_signals);
    }

    #[test]
    fn test_task_content_validation() {
        let test_case = get_todo_api_test_case();
        
        // Verify expected outputs match the decomposition pattern
        assert_eq!(test_case.expected_l3_outputs.len(), 2);
        assert_eq!(test_case.expected_l2_outputs.len(), 4);
        
        // Verify L3 outputs are design-focused
        for l3_output in &test_case.expected_l3_outputs {
            assert!(l3_output.contains("Design") || l3_output.contains("design"));
        }
        
        // Verify L2 outputs are implementation-focused
        for l2_output in &test_case.expected_l2_outputs {
            assert!(l2_output.contains("Implement") || l2_output.contains("implement"));
        }
    }

    fn verify_signal_hierarchy(signals: &[Signal]) {
        // Build parent-child map
        let mut children_by_parent: HashMap<Option<Uuid>, Vec<&Signal>> = HashMap::new();
        
        for signal in signals {
            children_by_parent
                .entry(signal.parent_id)
                .or_insert_with(Vec::new)
                .push(signal);
        }
        
        // Verify root has no parent
        let roots = children_by_parent.get(&None).expect("Should have root signal");
        assert_eq!(roots.len(), 1);
        
        // Verify each non-root has exactly one parent
        for signal in signals {
            if signal.parent_id.is_some() {
                let parent_exists = signals.iter().any(|s| Some(s.id) == signal.parent_id);
                assert!(parent_exists, "Signal {:?} has invalid parent", signal.id);
            }
        }
        
        // Verify L4 has 2 children (L3)
        let l4_signal = signals.iter().find(|s| s.layer == "L4").expect("Should have L4 signal");
        let l4_children = children_by_parent.get(&Some(l4_signal.id)).expect("L4 should have children");
        assert_eq!(l4_children.len(), 2);
        
        // Verify each L3 has 2 children (L2)
        for l3_signal in signals.iter().filter(|s| s.layer == "L3") {
            let l3_children = children_by_parent.get(&Some(l3_signal.id));
            if let Some(children) = l3_children {
                assert_eq!(children.len(), 2, "Each L3 should have exactly 2 L2 children");
            }
        }
    }
}

/// Test cases for different scenarios
#[cfg(test)]
mod scenario_tests {
    use super::*;

    fn get_ecommerce_test_case() -> TaskCompositionTest {
        TaskCompositionTest {
            user_prompt: "Create a basic e-commerce product catalog".to_string(),
            expected_l4_output: "Design and implement e-commerce product catalog system".to_string(),
            expected_l3_outputs: vec![
                "Design product data model and inventory management".to_string(),
                "Design product browsing and search interface".to_string(),
            ],
            expected_l2_outputs: vec![
                "Implement product schema with SKU, name, price, inventory fields".to_string(),
                "Implement product service with CRUD and inventory tracking".to_string(),
                "Implement product listing API with pagination and filters".to_string(),
                "Implement product search with full-text and faceted search".to_string(),
            ],
        }
    }

    fn get_chat_test_case() -> TaskCompositionTest {
        TaskCompositionTest {
            user_prompt: "Build a real-time chat system".to_string(),
            expected_l4_output: "Design and implement real-time chat communication system".to_string(),
            expected_l3_outputs: vec![
                "Design message delivery and persistence architecture".to_string(),
                "Design real-time communication and presence system".to_string(),
            ],
            expected_l2_outputs: vec![
                "Implement message schema with sender, content, timestamp fields".to_string(),
                "Implement message storage with Redis pub/sub for delivery".to_string(),
                "Implement WebSocket server for real-time connections".to_string(),
                "Implement presence tracking and typing indicators".to_string(),
            ],
        }
    }

    #[test]
    fn test_multiple_scenarios() {
        let scenarios = vec![
            get_todo_api_test_case(),
            get_ecommerce_test_case(),
            get_chat_test_case(),
        ];
        
        for scenario in scenarios {
            // Verify 1→2→4 decomposition pattern
            assert_eq!(scenario.expected_l3_outputs.len(), 2);
            assert_eq!(scenario.expected_l2_outputs.len(), 4);
        }
    }
}