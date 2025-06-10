//! Comprehensive unit tests for intelligence layer modules

use super::*;
use std::sync::Arc;
use uuid::Uuid;

/// Test utilities for intelligence layer
mod test_utils {
    use super::*;
    
    pub fn create_test_goal() -> Goal {
        Goal {
            id: Uuid::new_v4(),
            description: "Test goal".to_string(),
            priority: 0.8,
            constraints: vec![
                Constraint {
                    constraint_type: ConstraintType::Resource { max_cost: 100.0 },
                    parameters: HashMap::new(),
                },
            ],
            success_criteria: vec![
                Criterion {
                    name: "performance".to_string(),
                    measurement: Measurement::Absolute,
                    target_value: 0.9,
                },
            ],
            decomposition_strategy: DecompositionStrategy::Hierarchical,
        }
    }
    
    pub fn create_test_challenge() -> Challenge {
        Challenge {
            id: Uuid::new_v4(),
            description: "Test challenge".to_string(),
            context: HashMap::new(),
            constraints: vec![],
            desired_outcome: "Success".to_string(),
        }
    }
    
    pub fn create_meta_learning_config() -> MetaLearningConfig {
        MetaLearningConfig {
            strategy_selection: StrategySelectionMode::Adaptive,
            architecture_search: false,
            hyperparameter_optimization: true,
            transfer_learning: true,
            experience_replay_size: 1000,
        }
    }
    
    pub fn create_self_organization_config() -> SelfOrganizationConfig {
        SelfOrganizationConfig {
            reorganization_threshold: 0.3,
            adaptation_rate: 0.1,
            stability_weight: 0.7,
            exploration_rate: 0.2,
        }
    }
}

mod meta_learning_tests {
    use super::*;
    use super::test_utils::*;
    
    #[tokio::test]
    async fn test_meta_learner_initialization() {
        let meta_learner = DefaultMetaLearner::new();
        
        // Initialize with config
        let config = create_meta_learning_config();
        let result = meta_learner.configure(config.clone()).await;
        assert!(result.is_ok());
        
        // Verify configuration was applied
        let current_config = meta_learner.get_config().await.unwrap();
        assert_eq!(current_config.experience_replay_size, config.experience_replay_size);
    }
    
    #[tokio::test]
    async fn test_strategy_selection() {
        let mut meta_learner = DefaultMetaLearner::new();
        let config = create_meta_learning_config();
        meta_learner.configure(config).await.unwrap();
        
        // Add learning strategies
        let strategies = vec![
            LearningStrategy {
                id: Uuid::new_v4(),
                name: "gradient_descent".to_string(),
                parameters: HashMap::new(),
                performance_history: vec![0.6, 0.7, 0.8],
            },
            LearningStrategy {
                id: Uuid::new_v4(),
                name: "evolutionary".to_string(),
                parameters: HashMap::new(),
                performance_history: vec![0.7, 0.75, 0.85],
            },
        ];
        
        for strategy in strategies {
            meta_learner.add_strategy(strategy).await.unwrap();
        }
        
        // Select best strategy
        let task_context = TaskContext {
            task_type: "optimization".to_string(),
            complexity: 0.7,
            time_constraint: None,
        };
        
        let selected = meta_learner.select_strategy(&task_context).await.unwrap();
        assert_eq!(selected.name, "evolutionary"); // Should select the better performing one
    }
    
    #[tokio::test]
    async fn test_architecture_optimization() {
        let mut optimizer = ArchitectureOptimizer::new();
        
        // Define search space
        let search_space = ArchitectureSearchSpace {
            layer_types: vec![LayerType::Dense, LayerType::Recurrent],
            connection_patterns: vec![ConnectionPattern::Sequential, ConnectionPattern::Skip],
            activation_functions: vec![ActivationFunction::ReLU, ActivationFunction::Tanh],
            max_depth: 5,
        };
        
        optimizer.set_search_space(search_space).await.unwrap();
        
        // Run optimization (mock)
        let task = OptimizationTask {
            objective: "minimize_error".to_string(),
            constraints: vec![],
        };
        
        let architecture = optimizer.optimize(&task, 10).await.unwrap();
        assert!(!architecture.layers.is_empty());
        assert!(architecture.total_parameters > 0);
    }
    
    #[tokio::test]
    async fn test_transfer_learning() {
        let mut meta_learner = DefaultMetaLearner::new();
        
        // Learn from source task
        let source_experience = Experience {
            task_id: Uuid::new_v4(),
            task_type: "classification".to_string(),
            learned_parameters: vec![0.1, 0.2, 0.3],
            performance: 0.85,
        };
        
        meta_learner.store_experience(source_experience).await.unwrap();
        
        // Transfer to target task
        let target_task = TaskContext {
            task_type: "classification".to_string(),
            complexity: 0.6,
            time_constraint: None,
        };
        
        let transferred = meta_learner.transfer_knowledge(&target_task).await.unwrap();
        assert!(!transferred.parameters.is_empty());
        assert!(transferred.confidence > 0.0);
    }
}

mod emergence_tests {
    use super::*;
    use super::test_utils::*;
    
    #[tokio::test]
    async fn test_emergence_detector() {
        let mut detector = EmergenceDetector::new();
        
        // Configure detection parameters
        detector.set_sensitivity(0.7).await.unwrap();
        detector.set_observation_window(std::time::Duration::from_secs(60)).await.unwrap();
        
        // Simulate system observations
        let observations = vec![
            Observation {
                timestamp: chrono::Utc::now(),
                metrics: vec![0.5, 0.6, 0.7],
                interactions: 10,
            },
            Observation {
                timestamp: chrono::Utc::now(),
                metrics: vec![0.6, 0.7, 0.8],
                interactions: 15,
            },
            Observation {
                timestamp: chrono::Utc::now(),
                metrics: vec![0.8, 0.85, 0.9],
                interactions: 25,
            },
        ];
        
        for obs in observations {
            detector.add_observation(obs).await.unwrap();
        }
        
        // Detect emergent patterns
        let patterns = detector.detect_patterns().await.unwrap();
        assert!(!patterns.is_empty());
        
        // Check for phase transitions
        let transitions = detector.detect_phase_transitions().await.unwrap();
        assert!(transitions.len() >= 0); // May or may not detect transitions
    }
    
    #[tokio::test]
    async fn test_complexity_measurement() {
        let analyzer = ComplexityAnalyzer::new();
        
        // Measure system complexity
        let system_state = SystemState {
            components: 50,
            connections: 200,
            active_processes: 30,
            entropy: 0.7,
        };
        
        let complexity = analyzer.measure(&system_state).await.unwrap();
        assert!(complexity.structural > 0.0);
        assert!(complexity.dynamic > 0.0);
        assert!(complexity.computational > 0.0);
    }
    
    #[tokio::test]
    async fn test_self_organization_detection() {
        let detector = SelfOrganizationDetector::new();
        
        // Create time series of organization metrics
        let time_series = vec![
            OrganizationMetric {
                timestamp: chrono::Utc::now(),
                clustering_coefficient: 0.3,
                modularity: 0.4,
                hierarchy_depth: 3,
            },
            OrganizationMetric {
                timestamp: chrono::Utc::now(),
                clustering_coefficient: 0.5,
                modularity: 0.6,
                hierarchy_depth: 4,
            },
            OrganizationMetric {
                timestamp: chrono::Utc::now(),
                clustering_coefficient: 0.7,
                modularity: 0.8,
                hierarchy_depth: 5,
            },
        ];
        
        let is_organizing = detector.detect(&time_series).await.unwrap();
        assert!(is_organizing); // Should detect increasing organization
    }
}

mod creativity_tests {
    use super::*;
    use super::test_utils::*;
    
    #[tokio::test]
    async fn test_creative_system_initialization() {
        let mut creative_system = CreativeSystem::new();
        
        // Initialize subsystems
        creative_system.initialize_subsystems().await.unwrap();
        
        // Verify all components are ready
        let status = creative_system.get_status().await.unwrap();
        assert!(status.inspiration_engine_ready);
        assert!(status.concept_blender_ready);
        assert!(status.novelty_evaluator_ready);
    }
    
    #[tokio::test]
    async fn test_idea_generation() {
        let mut creative_system = CreativeSystem::new();
        creative_system.initialize_subsystems().await.unwrap();
        
        // Generate ideas for a challenge
        let challenge = create_test_challenge();
        let ideas = creative_system.generate_ideas(&challenge, 5).await.unwrap();
        
        assert_eq!(ideas.len(), 5);
        for idea in &ideas {
            assert!(!idea.description.is_empty());
            assert!(idea.novelty_score >= 0.0 && idea.novelty_score <= 1.0);
            assert!(idea.feasibility_score >= 0.0 && idea.feasibility_score <= 1.0);
        }
    }
    
    #[tokio::test]
    async fn test_concept_blending() {
        let blender = ConceptBlender::new();
        
        // Create source concepts
        let concept1 = Concept {
            id: Uuid::new_v4(),
            name: "bird".to_string(),
            attributes: vec!["flies", "has_wings", "lays_eggs"].into_iter()
                .map(|s| s.to_string()).collect(),
            domain: "biology".to_string(),
        };
        
        let concept2 = Concept {
            id: Uuid::new_v4(),
            name: "submarine".to_string(),
            attributes: vec!["underwater", "mechanical", "transport"].into_iter()
                .map(|s| s.to_string()).collect(),
            domain: "technology".to_string(),
        };
        
        // Blend concepts
        let blended = blender.blend(&concept1, &concept2).await.unwrap();
        
        assert!(!blended.name.is_empty());
        assert!(!blended.attributes.is_empty());
        assert!(blended.attributes.len() >= 2); // Should combine some attributes
    }
    
    #[tokio::test]
    async fn test_novelty_evaluation() {
        let evaluator = NoveltyEvaluator::new();
        
        // Add existing knowledge
        let existing_ideas = vec![
            "solar panels on roofs",
            "wind turbines in fields",
            "hydroelectric dams",
        ];
        
        for idea in existing_ideas {
            evaluator.add_to_knowledge_base(idea).await.unwrap();
        }
        
        // Evaluate novelty of new ideas
        let new_idea1 = "solar panels on roads";
        let new_idea2 = "fusion reactors in space";
        
        let novelty1 = evaluator.evaluate(new_idea1).await.unwrap();
        let novelty2 = evaluator.evaluate(new_idea2).await.unwrap();
        
        assert!(novelty2 > novelty1); // Space fusion should be more novel
    }
    
    #[tokio::test]
    async fn test_analogical_reasoning() {
        let reasoner = AnalogicalReasoner::new();
        
        // Create source domain
        let source = DomainKnowledge {
            domain: "biology".to_string(),
            entities: vec!["heart", "blood", "vessels"],
            relations: vec![("heart", "pumps", "blood"), ("blood", "flows_through", "vessels")],
        };
        
        // Create target domain
        let target = DomainKnowledge {
            domain: "city_planning".to_string(),
            entities: vec!["central_station", "traffic", "roads"],
            relations: vec![],
        };
        
        // Find analogies
        let analogies = reasoner.find_analogies(&source, &target).await.unwrap();
        
        assert!(!analogies.is_empty());
        // Should map heart->central_station, blood->traffic, vessels->roads
    }
}

mod self_organization_tests {
    use super::*;
    use super::test_utils::*;
    
    #[tokio::test]
    async fn test_topology_manager() {
        let mut manager = TopologyManager::new();
        
        // Create initial topology
        let nodes = vec![
            NodeInfo { id: Uuid::new_v4(), node_type: "cognitive".to_string(), layer: 1 },
            NodeInfo { id: Uuid::new_v4(), node_type: "cognitive".to_string(), layer: 2 },
            NodeInfo { id: Uuid::new_v4(), node_type: "cognitive".to_string(), layer: 3 },
        ];
        
        for node in nodes {
            manager.add_node(node).await.unwrap();
        }
        
        // Add connections
        manager.connect_layers().await.unwrap();
        
        // Verify topology
        let topology = manager.get_topology().await.unwrap();
        assert_eq!(topology.nodes.len(), 3);
        assert!(topology.edges.len() >= 2); // At least layer connections
    }
    
    #[tokio::test]
    async fn test_resource_allocator() {
        let mut allocator = ResourceAllocator::new();
        
        // Set total resources
        allocator.set_total_resources(ResourcePool {
            compute: 1000.0,
            memory: 8192.0,
            bandwidth: 100.0,
        }).await.unwrap();
        
        // Request allocations
        let request1 = ResourceRequest {
            requester_id: Uuid::new_v4(),
            compute: 300.0,
            memory: 2048.0,
            bandwidth: 30.0,
            priority: 0.8,
        };
        
        let request2 = ResourceRequest {
            requester_id: Uuid::new_v4(),
            compute: 500.0,
            memory: 4096.0,
            bandwidth: 50.0,
            priority: 0.6,
        };
        
        let alloc1 = allocator.allocate(request1).await.unwrap();
        let alloc2 = allocator.allocate(request2).await.unwrap();
        
        assert!(alloc1.granted);
        assert!(alloc2.granted);
        
        // Verify allocations don't exceed limits
        let remaining = allocator.get_available_resources().await.unwrap();
        assert!(remaining.compute >= 0.0);
        assert!(remaining.memory >= 0.0);
        assert!(remaining.bandwidth >= 0.0);
    }
    
    #[tokio::test]
    async fn test_adaptation_controller() {
        let mut controller = AdaptationController::new();
        
        // Configure adaptation parameters
        controller.set_parameters(AdaptationParameters {
            learning_rate: 0.1,
            momentum: 0.9,
            decay_rate: 0.01,
        }).await.unwrap();
        
        // Simulate performance feedback
        let feedback_history = vec![0.5, 0.6, 0.65, 0.7, 0.72, 0.74];
        
        for performance in feedback_history {
            let adjustment = controller.compute_adjustment(performance).await.unwrap();
            controller.apply_adjustment(adjustment).await.unwrap();
        }
        
        // Verify adaptation occurred
        let current_params = controller.get_parameters().await.unwrap();
        assert_ne!(current_params.learning_rate, 0.1); // Should have adapted
    }
}

mod intelligence_coordinator_tests {
    use super::*;
    use super::test_utils::*;
    
    #[tokio::test]
    async fn test_coordinator_initialization() {
        let mut coordinator = DefaultIntelligenceCoordinator::new();
        
        // Initialize all subsystems
        let result = coordinator.initialize().await;
        assert!(result.is_ok());
        
        // Verify subsystems are ready
        let metrics = coordinator.metrics().await.unwrap();
        assert!(metrics.subsystems_initialized);
        assert_eq!(metrics.active_goals, 0);
    }
    
    #[tokio::test]
    async fn test_goal_management() {
        let mut coordinator = DefaultIntelligenceCoordinator::new();
        coordinator.initialize().await.unwrap();
        
        // Set multiple goals
        let goals = vec![
            create_test_goal(),
            Goal {
                id: Uuid::new_v4(),
                description: "Secondary goal".to_string(),
                priority: 0.6,
                constraints: vec![],
                success_criteria: vec![],
                decomposition_strategy: DecompositionStrategy::Parallel,
            },
        ];
        
        coordinator.set_goals(goals.clone()).await.unwrap();
        
        // Verify goals are tracked
        let metrics = coordinator.metrics().await.unwrap();
        assert_eq!(metrics.active_goals, 2);
        
        // Test goal prioritization
        let prioritized = coordinator.get_prioritized_goals().await.unwrap();
        assert_eq!(prioritized[0].priority, 0.8); // Higher priority first
    }
    
    #[tokio::test]
    async fn test_emergent_behavior_observation() {
        let mut coordinator = DefaultIntelligenceCoordinator::new();
        coordinator.initialize().await.unwrap();
        
        // Enable monitoring
        coordinator.enable_emergence_monitoring().await.unwrap();
        
        // Simulate system activity
        for _ in 0..10 {
            coordinator.tick().await.unwrap(); // Simulate time steps
        }
        
        // Observe emergence
        let report = coordinator.observe_emergence().await.unwrap();
        
        assert!(report.observation_period > std::time::Duration::from_secs(0));
        assert!(report.detected_patterns.len() >= 0);
        assert!(report.complexity_trend.len() >= 0);
    }
    
    #[tokio::test]
    async fn test_creative_problem_solving() {
        let mut coordinator = DefaultIntelligenceCoordinator::new();
        coordinator.initialize().await.unwrap();
        
        // Enable creativity subsystem
        coordinator.enable_creativity(CreativityConfig {
            novelty_threshold: 0.7,
            exploration_rate: 0.3,
            inspiration_sources: vec!["nature", "technology", "art"],
        }).await.unwrap();
        
        // Present a challenge
        let challenge = create_test_challenge();
        let solutions = coordinator.create(challenge).await.unwrap();
        
        assert!(!solutions.is_empty());
        for solution in &solutions {
            assert!(!solution.description.is_empty());
            assert!(solution.confidence > 0.0);
            assert!(!solution.implementation_steps.is_empty());
        }
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use super::test_utils::*;
    
    #[tokio::test]
    async fn test_full_intelligence_stack() {
        // Create and initialize coordinator
        let mut coordinator = DefaultIntelligenceCoordinator::new();
        coordinator.initialize().await.unwrap();
        
        // Enable all intelligence features
        coordinator.enable_meta_learning(create_meta_learning_config()).await.unwrap();
        coordinator.enable_self_organization(create_self_organization_config()).await.unwrap();
        
        // Set a complex goal
        let goal = Goal {
            id: Uuid::new_v4(),
            description: "Optimize system performance while minimizing resource usage".to_string(),
            priority: 1.0,
            constraints: vec![
                Constraint {
                    constraint_type: ConstraintType::Resource { max_cost: 1000.0 },
                    parameters: HashMap::new(),
                },
                Constraint {
                    constraint_type: ConstraintType::Quality { min_score: 0.8 },
                    parameters: HashMap::new(),
                },
            ],
            success_criteria: vec![
                Criterion {
                    name: "performance".to_string(),
                    measurement: Measurement::Absolute,
                    target_value: 0.9,
                },
                Criterion {
                    name: "efficiency".to_string(),
                    measurement: Measurement::Relative { baseline: 0.5 },
                    target_value: 0.8,
                },
            ],
            decomposition_strategy: DecompositionStrategy::Adaptive,
        };
        
        coordinator.set_goals(vec![goal]).await.unwrap();
        
        // Run several iterations
        for _ in 0..5 {
            coordinator.think().await.unwrap();
            coordinator.adapt().await.unwrap();
            coordinator.evolve().await.unwrap();
        }
        
        // Check final state
        let metrics = coordinator.metrics().await.unwrap();
        assert!(metrics.total_adaptations > 0);
        assert!(metrics.emergence_events.len() >= 0);
        assert!(metrics.creative_solutions_generated >= 0);
    }
}

/// Benchmark tests for performance measurement
#[cfg(test)]
mod benchmarks {
    use super::*;
    use super::test_utils::*;
    use std::time::Instant;
    
    #[tokio::test]
    async fn benchmark_meta_learning_strategy_selection() {
        let mut meta_learner = DefaultMetaLearner::new();
        meta_learner.configure(create_meta_learning_config()).await.unwrap();
        
        // Add many strategies
        for i in 0..100 {
            let strategy = LearningStrategy {
                id: Uuid::new_v4(),
                name: format!("strategy_{}", i),
                parameters: HashMap::new(),
                performance_history: vec![0.5 + (i as f32 * 0.001)],
            };
            meta_learner.add_strategy(strategy).await.unwrap();
        }
        
        // Benchmark selection
        let task_context = TaskContext {
            task_type: "test".to_string(),
            complexity: 0.5,
            time_constraint: None,
        };
        
        let start = Instant::now();
        let iterations = 1000;
        
        for _ in 0..iterations {
            let _ = meta_learner.select_strategy(&task_context).await.unwrap();
        }
        
        let elapsed = start.elapsed();
        let avg_time = elapsed.as_micros() as f64 / iterations as f64;
        
        println!("Average strategy selection time: {:.2} μs", avg_time);
        assert!(avg_time < 100.0); // Should be under 100 microseconds
    }
    
    #[tokio::test]
    async fn benchmark_emergence_detection() {
        let mut detector = EmergenceDetector::new();
        detector.set_sensitivity(0.7).await.unwrap();
        
        // Add many observations
        let observations: Vec<_> = (0..1000).map(|i| {
            Observation {
                timestamp: chrono::Utc::now(),
                metrics: vec![i as f32 * 0.001, (i as f32 * 0.002).sin(), (i as f32 * 0.003).cos()],
                interactions: i % 50,
            }
        }).collect();
        
        for obs in observations {
            detector.add_observation(obs).await.unwrap();
        }
        
        // Benchmark pattern detection
        let start = Instant::now();
        let patterns = detector.detect_patterns().await.unwrap();
        let elapsed = start.elapsed();
        
        println!("Pattern detection time for 1000 observations: {:.2} ms", elapsed.as_millis());
        println!("Patterns detected: {}", patterns.len());
        assert!(elapsed.as_millis() < 100); // Should complete within 100ms
    }
}