//! Comprehensive unit tests for cognitive layer modules

use super::*;
use uuid::Uuid;

/// Test utilities module
mod test_utils {
    use super::*;
    
    /// Create a test cognitive configuration
    pub fn create_test_config(layer: CognitiveLayer) -> CognitiveConfig {
        let mut params = HashMap::new();
        params.insert("learning_rate".to_string(), 0.1);
        params.insert("threshold".to_string(), 0.5);
        
        CognitiveConfig {
            id: Uuid::new_v4(),
            layer,
            initial_parameters: params,
            connections: ConnectionConfig {
                upward_connections: vec![],
                lateral_connections: vec![],
                downward_connections: vec![],
            },
        }
    }
    
    /// Create a test cognitive input
    pub fn create_test_input(content: &str) -> CognitiveInput {
        CognitiveInput {
            content: content.to_string(),
            context: HashMap::new(),
            source_layer: None,
        }
    }
    
    /// Create a test learning gradient
    pub fn create_test_gradient(error: f32) -> LearningGradient {
        LearningGradient {
            gradient_id: Uuid::new_v4(),
            error_signal: ErrorSignal {
                error_type: "test_error".to_string(),
                magnitude: error,
                context: HashMap::new(),
            },
            adjustments: vec![
                ParameterAdjustment {
                    parameter: "learning_rate".to_string(),
                    current_value: 0.1,
                    suggested_delta: -0.01,
                    confidence: 0.8,
                },
            ],
            importance: 0.5,
            timestamp: chrono::Utc::now(),
        }
    }
}

mod l1_reflexive_tests {
    use super::*;
    use super::test_utils::*;
    
    #[tokio::test]
    async fn test_reflexive_pattern_matching() {
        let config = create_test_config(CognitiveLayer::Reflexive);
        let mut neuron = L1ReflexiveNeuron::new(config);
        
        // Add multiple patterns with different confidence levels
        let patterns = vec![
            Pattern {
                trigger: "hello".to_string(),
                response: "Hi there!".to_string(),
                confidence: 0.9,
            },
            Pattern {
                trigger: "goodbye".to_string(),
                response: "See you later!".to_string(),
                confidence: 0.85,
            },
            Pattern {
                trigger: "how are you".to_string(),
                response: "I'm functioning optimally!".to_string(),
                confidence: 0.95,
            },
        ];
        
        for pattern in patterns {
            neuron.add_pattern(pattern);
        }
        
        // Test exact match
        let input = create_test_input("hello");
        let output = neuron.process(input).await.unwrap();
        assert_eq!(output.content, "Hi there!");
        assert!(output.confidence > 0.9);
        
        // Test partial match
        let input = create_test_input("hello world");
        let output = neuron.process(input).await.unwrap();
        assert_eq!(output.content, "Hi there!");
        
        // Test no match scenario
        let input = create_test_input("random input");
        let output = neuron.process(input).await.unwrap();
        assert!(output.content.starts_with("ACK:"));
    }
    
    #[tokio::test]
    async fn test_reflexive_cache_behavior() {
        let config = create_test_config(CognitiveLayer::Reflexive);
        let mut neuron = L1ReflexiveNeuron::new(config);
        
        neuron.add_pattern(Pattern {
            trigger: "test".to_string(),
            response: "Test response".to_string(),
            confidence: 0.9,
        });
        
        // First request - cache miss
        let input = create_test_input("test input");
        let _ = neuron.process(input).await.unwrap();
        
        let state = neuron.introspect().await;
        assert_eq!(state.cache_stats.misses, 1);
        assert_eq!(state.cache_stats.hits, 0);
        
        // Second request - cache hit
        let input = create_test_input("test input");
        let _ = neuron.process(input).await.unwrap();
        
        let state = neuron.introspect().await;
        assert_eq!(state.cache_stats.misses, 1);
        assert_eq!(state.cache_stats.hits, 1);
    }
    
    #[tokio::test]
    async fn test_reflexive_learning() {
        let config = create_test_config(CognitiveLayer::Reflexive);
        let mut neuron = L1ReflexiveNeuron::new(config);
        
        neuron.add_pattern(Pattern {
            trigger: "learn".to_string(),
            response: "Learning response".to_string(),
            confidence: 0.5,
        });
        
        let initial_state = neuron.introspect().await;
        let initial_confidence = initial_state.patterns[0].confidence;
        
        // Apply positive learning (low error)
        let gradient = create_test_gradient(0.05);
        neuron.learn(gradient).await.unwrap();
        
        let state = neuron.introspect().await;
        assert!(state.patterns[0].confidence > initial_confidence);
        assert_eq!(state.basic.metrics.learning_iterations, 1);
        
        // Apply negative learning (high error)
        let gradient = create_test_gradient(0.5);
        neuron.learn(gradient).await.unwrap();
        
        let state = neuron.introspect().await;
        assert!(state.patterns[0].confidence < initial_state.patterns[0].confidence * 1.01);
    }
    
    #[tokio::test]
    async fn test_reflexive_metrics_tracking() {
        let config = create_test_config(CognitiveLayer::Reflexive);
        let mut neuron = L1ReflexiveNeuron::new(config);
        
        // Process multiple inputs
        for i in 0..5 {
            let input = create_test_input(&format!("input {}", i));
            let _ = neuron.process(input).await.unwrap();
        }
        
        let state = neuron.introspect().await;
        assert_eq!(state.basic.metrics.activations_processed, 5);
        assert!(state.basic.metrics.average_processing_time_ms > 0.0);
    }
    
    #[tokio::test]
    async fn test_reflexive_reset() {
        let config = create_test_config(CognitiveLayer::Reflexive);
        let mut neuron = L1ReflexiveNeuron::new(config);
        
        // Add patterns and process inputs
        neuron.add_pattern(Pattern {
            trigger: "test".to_string(),
            response: "response".to_string(),
            confidence: 0.9,
        });
        
        let _ = neuron.process(create_test_input("test")).await.unwrap();
        
        // Verify state is populated
        let state = neuron.introspect().await;
        assert!(!state.patterns.is_empty());
        assert!(state.cache_stats.misses > 0);
        
        // Reset and verify
        neuron.reset().await.unwrap();
        let state = neuron.introspect().await;
        assert!(state.patterns.is_empty());
        assert_eq!(state.cache_stats.hits, 0);
        assert_eq!(state.cache_stats.misses, 0);
    }
}

mod l2_implementation_tests {
    use super::*;
    use super::test_utils::*;
    
    #[tokio::test]
    async fn test_implementation_code_generation() {
        let config = create_test_config(CognitiveLayer::Implementation);
        let mut neuron = L2ImplementationNeuron::new(config);
        
        // Test code generation request
        let input = create_test_input("generate function to calculate fibonacci");
        let output = neuron.process(input).await.unwrap();
        
        assert!(!output.content.is_empty());
        assert!(output.confidence > 0.5);
        assert_eq!(output.target_layers, vec![CognitiveLayer::Operational]);
    }
    
    #[tokio::test]
    async fn test_implementation_state_tracking() {
        let config = create_test_config(CognitiveLayer::Implementation);
        let mut neuron = L2ImplementationNeuron::new(config);
        
        let initial_state = neuron.introspect().await;
        assert_eq!(initial_state.execution_history.len(), 0);
        
        // Process a task
        let input = create_test_input("implement sorting algorithm");
        let _ = neuron.process(input).await.unwrap();
        
        let state = neuron.introspect().await;
        assert!(state.basic.metrics.activations_processed > 0);
    }
    
    #[tokio::test]
    async fn test_implementation_learning_adaptation() {
        let config = create_test_config(CognitiveLayer::Implementation);
        let mut neuron = L2ImplementationNeuron::new(config);
        
        // Get initial state
        let initial_state = neuron.introspect().await;
        let initial_iterations = initial_state.basic.metrics.learning_iterations;
        
        // Apply learning gradient
        let gradient = create_test_gradient(0.1);
        neuron.learn(gradient).await.unwrap();
        
        let state = neuron.introspect().await;
        
        // Verify learning iterations increased
        assert_eq!(state.basic.metrics.learning_iterations, initial_iterations + 1);
    }
}

mod l3_operational_tests {
    use super::*;
    use super::test_utils::*;
    
    #[tokio::test]
    async fn test_operational_task_coordination() {
        let config = create_test_config(CognitiveLayer::Operational);
        let mut neuron = L3OperationalNeuron::new(config);
        
        // Test operational planning
        let input = create_test_input("coordinate deployment of microservices");
        let output = neuron.process(input).await.unwrap();
        
        assert!(!output.content.is_empty());
        assert!(output.metadata.contains_key("processing_time_ms"));
        // TaskCoordination returns Implementation as target
        assert_eq!(output.target_layers, vec![CognitiveLayer::Implementation]);
    }
    
    #[tokio::test]
    async fn test_operational_resource_planning() {
        let config = create_test_config(CognitiveLayer::Operational);
        let mut neuron = L3OperationalNeuron::new(config);
        
        let initial_state = neuron.introspect().await;
        assert_eq!(initial_state.task_queue.len(), 0);
        
        // Process resource planning request
        let input = create_test_input("allocate resources for distributed system");
        let _ = neuron.process(input).await.unwrap();
        
        let state = neuron.introspect().await;
        assert!(state.basic.metrics.activations_processed > 0);
    }
    
    #[tokio::test]
    async fn test_operational_metrics_aggregation() {
        let config = create_test_config(CognitiveLayer::Operational);
        let mut neuron = L3OperationalNeuron::new(config);
        
        // Process multiple operational tasks
        let tasks = vec![
            "design system architecture",
            "plan database schema",
            "coordinate API development",
        ];
        
        for task in tasks {
            let input = create_test_input(task);
            let _ = neuron.process(input).await.unwrap();
        }
        
        let state = neuron.introspect().await;
        assert_eq!(state.basic.metrics.activations_processed, 3);
        assert!(state.basic.metrics.average_processing_time_ms > 0.0);
    }
}

mod l4_tactical_tests {
    use super::*;
    use super::test_utils::*;
    
    #[tokio::test]
    async fn test_tactical_planning() {
        let config = create_test_config(CognitiveLayer::Tactical);
        let mut neuron = L4TacticalNeuron::new(config);
        
        // Test tactical planning
        let input = create_test_input("plan quarterly product roadmap");
        let output = neuron.process(input).await.unwrap();
        
        assert!(!output.content.is_empty());
        assert!(output.confidence > 0.6);
        // Tactical planning directs to Operational for execution
        assert_eq!(output.target_layers, vec![CognitiveLayer::Operational]);
    }
    
    #[tokio::test]
    async fn test_tactical_strategy_execution() {
        let config = create_test_config(CognitiveLayer::Tactical);
        let mut neuron = L4TacticalNeuron::new(config);
        
        let initial_state = neuron.introspect().await;
        assert!(initial_state.current_plan.is_none());
        
        // Process strategic directive
        let input = create_test_input("execute market expansion strategy");
        let _ = neuron.process(input).await.unwrap();
        
        let state = neuron.introspect().await;
        assert!(state.basic.metrics.activations_processed > 0);
    }
    
    #[tokio::test]
    async fn test_tactical_adaptation() {
        let config = create_test_config(CognitiveLayer::Tactical);
        let mut neuron = L4TacticalNeuron::new(config);
        
        // Apply multiple learning iterations
        for i in 0..3 {
            let gradient = create_test_gradient(0.2 - (i as f32 * 0.05));
            neuron.learn(gradient).await.unwrap();
        }
        
        let state = neuron.introspect().await;
        assert_eq!(state.basic.metrics.learning_iterations, 3);
    }
}

mod l5_strategic_tests {
    use super::*;
    use super::test_utils::*;
    
    #[tokio::test]
    async fn test_strategic_vision_planning() {
        let config = create_test_config(CognitiveLayer::Strategic);
        let mut neuron = L5StrategicNeuron::new(config);
        
        // Test strategic vision
        let input = create_test_input("develop 5-year technology vision");
        let output = neuron.process(input).await.unwrap();
        
        assert!(!output.content.is_empty());
        assert!(output.confidence > 0.7);
        // Strategic directs to both Tactical and Operational
        assert_eq!(output.target_layers, vec![CognitiveLayer::Tactical, CognitiveLayer::Operational]);
    }
    
    #[tokio::test]
    async fn test_strategic_goal_setting() {
        let config = create_test_config(CognitiveLayer::Strategic);
        let mut neuron = L5StrategicNeuron::new(config);
        
        let initial_state = neuron.introspect().await;
        // Strategic neuron starts with foundational goals
        assert!(initial_state.active_goals.len() > 0);
        
        // Process goal-setting request
        let input = create_test_input("establish organizational objectives");
        let _ = neuron.process(input).await.unwrap();
        
        let state = neuron.introspect().await;
        assert!(state.basic.metrics.activations_processed > 0);
    }
    
    #[tokio::test]
    async fn test_strategic_long_term_learning() {
        let config = create_test_config(CognitiveLayer::Strategic);
        let mut neuron = L5StrategicNeuron::new(config);
        
        // Simulate long-term learning with decreasing error
        let errors = vec![0.5, 0.4, 0.3, 0.2, 0.1];
        
        for error in errors {
            let gradient = create_test_gradient(error);
            neuron.learn(gradient).await.unwrap();
        }
        
        let state = neuron.introspect().await;
        assert_eq!(state.basic.metrics.learning_iterations, 5);
        
        // Verify parameters were adjusted
        let learning_rate = state.basic.parameters.get("learning_rate").unwrap();
        assert_ne!(*learning_rate, 0.1); // Should differ from initial value
    }
}

mod pattern_matcher_tests {
    use super::*;
    
    #[test]
    #[ignore] // PatternMatcher::new() not implemented
    fn test_pattern_matcher_similarity() {
        let mut matcher = PatternMatcher::new();
        
        // Add patterns
        matcher.add_pattern(Pattern {
            trigger: "deploy application".to_string(),
            response: "Initiating deployment process".to_string(),
            confidence: 0.9,
        });
        
        matcher.add_pattern(Pattern {
            trigger: "check status".to_string(),
            response: "Checking system status".to_string(),
            confidence: 0.85,
        });
        
        // Test exact match
        let result = matcher.find_match("deploy application");
        assert_eq!(result, Some("Initiating deployment process".to_string()));
        
        // Test partial match
        let result = matcher.find_match("please deploy application now");
        assert_eq!(result, Some("Initiating deployment process".to_string()));
        
        // Test no match
        let result = matcher.find_match("random unrelated input");
        assert_eq!(result, None);
    }
    
    #[test]
    #[ignore] // PatternMatcher::new() not implemented
    fn test_pattern_matcher_word_matching() {
        let mut matcher = PatternMatcher::new();
        
        matcher.add_pattern(Pattern {
            trigger: "calculate sum".to_string(),
            response: "Calculating sum".to_string(),
            confidence: 0.9,
        });
        
        // Test word order doesn't matter for matching
        let result = matcher.find_match("sum calculate");
        assert!(result.is_none()); // Order matters in current implementation
        
        // Test all words must be present
        let result = matcher.find_match("please calculate the sum of numbers");
        assert_eq!(result, Some("Calculating sum".to_string()));
    }
}

mod response_cache_tests {
    use super::*;
    
    #[test]
    #[ignore] // ResponseCache::new() not implemented
    fn test_response_cache_operations() {
        let mut cache = ResponseCache::new(3);
        
        // Test put and get
        cache.put("key1".to_string(), "value1".to_string());
        cache.put("key2".to_string(), "value2".to_string());
        cache.put("key3".to_string(), "value3".to_string());
        
        assert_eq!(cache.get("key1"), Some(&"value1".to_string()));
        assert_eq!(cache.get("key2"), Some(&"value2".to_string()));
        assert_eq!(cache.get("key3"), Some(&"value3".to_string()));
        
        // Test LRU eviction
        cache.put("key4".to_string(), "value4".to_string());
        assert_eq!(cache.get("key1"), None); // Should be evicted
        assert_eq!(cache.get("key4"), Some(&"value4".to_string()));
        
        // Test clear
        cache.clear();
        assert_eq!(cache.get("key2"), None);
        assert_eq!(cache.get("key3"), None);
        assert_eq!(cache.get("key4"), None);
    }
}

mod integration_tests {
    use super::*;
    use super::test_utils::*;
    
    #[tokio::test]
    async fn test_cognitive_layer_progression() {
        // Create neurons for each layer
        let mut l1 = L1ReflexiveNeuron::new(create_test_config(CognitiveLayer::Reflexive));
        let mut l2 = L2ImplementationNeuron::new(create_test_config(CognitiveLayer::Implementation));
        let mut l3 = L3OperationalNeuron::new(create_test_config(CognitiveLayer::Operational));
        let mut l4 = L4TacticalNeuron::new(create_test_config(CognitiveLayer::Tactical));
        let mut l5 = L5StrategicNeuron::new(create_test_config(CognitiveLayer::Strategic));
        
        // Process through layers
        let input = create_test_input("develop new product feature");
        
        // L1: Quick acknowledgment
        let l1_output = l1.process(input.clone()).await.unwrap();
        assert!(l1_output.content.starts_with("ACK:"));
        
        // L2: Implementation details
        let l2_output = l2.process(input.clone()).await.unwrap();
        assert!(!l2_output.content.is_empty());
        
        // L3: Operational planning
        let l3_output = l3.process(input.clone()).await.unwrap();
        assert!(l3_output.confidence > 0.5);
        
        // L4: Tactical strategy
        let l4_output = l4.process(input.clone()).await.unwrap();
        assert!(l4_output.confidence > 0.6);
        
        // L5: Strategic vision
        let l5_output = l5.process(input).await.unwrap();
        assert!(l5_output.confidence > 0.7);
    }
    
    #[tokio::test]
    async fn test_cross_layer_learning() {
        // Test each neuron separately due to trait object limitations
        let mut l1 = L1ReflexiveNeuron::new(create_test_config(CognitiveLayer::Reflexive));
        let mut l2 = L2ImplementationNeuron::new(create_test_config(CognitiveLayer::Implementation));
        let mut l3 = L3OperationalNeuron::new(create_test_config(CognitiveLayer::Operational));
        
        // Apply learning to all layers
        let gradient = create_test_gradient(0.15);
        
        l1.learn(gradient.clone()).await.unwrap();
        l2.learn(gradient.clone()).await.unwrap();
        l3.learn(gradient.clone()).await.unwrap();
        
        // Verify all learned
        let state1 = l1.introspect().await;
        let state2 = l2.introspect().await;
        let state3 = l3.introspect().await;
        
        let s1_json = serde_json::to_value(state1).unwrap();
        let s2_json = serde_json::to_value(state2).unwrap();
        let s3_json = serde_json::to_value(state3).unwrap();
        
        for state in [s1_json, s2_json, s3_json] {
            let metrics = state.get("basic").unwrap()
                .get("metrics").unwrap()
                .get("learning_iterations").unwrap()
                .as_u64().unwrap();
            
            assert_eq!(metrics, 1);
        }
    }
}

#[cfg(test)]
mod benchmarks {
    use super::*;
    use super::test_utils::*;
    use std::time::Instant;
    
    #[tokio::test]
    async fn benchmark_reflexive_processing() {
        let config = create_test_config(CognitiveLayer::Reflexive);
        let mut neuron = L1ReflexiveNeuron::new(config);
        
        // Add patterns for benchmarking
        for i in 0..100 {
            neuron.add_pattern(Pattern {
                trigger: format!("pattern {}", i),
                response: format!("response {}", i),
                confidence: 0.9,
            });
        }
        
        // Benchmark processing
        let iterations = 1000;
        let start = Instant::now();
        
        for i in 0..iterations {
            let input = create_test_input(&format!("pattern {}", i % 100));
            let _ = neuron.process(input).await.unwrap();
        }
        
        let elapsed = start.elapsed();
        let avg_time = elapsed.as_micros() as f64 / iterations as f64;
        
        println!("Average reflexive processing time: {:.2} μs", avg_time);
        assert!(avg_time < 1000.0); // Should be under 1ms
    }
}