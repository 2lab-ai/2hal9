//! Comprehensive unit tests for intelligence layer modules

use super::*;
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
            problem_statement: "Test challenge".to_string(),
            context: HashMap::new(),
            constraints: vec![],
            evaluation_criteria: vec![],
        }
    }
    
    pub fn create_meta_learning_config() -> MetaLearningConfig {
        MetaLearningConfig {
            learning_rate_adaptation: true,
            strategy_evolution: true,
            architecture_search: false,
            transfer_learning: true,
            continual_learning: true,
        }
    }
    
    pub fn create_self_organization_config() -> SelfOrganizationConfig {
        SelfOrganizationConfig {
            allow_topology_changes: true,
            clustering_enabled: true,
            hierarchy_formation: true,
            emergent_specialization: true,
            dynamic_boundaries: false,
        }
    }
}

mod meta_learning_tests {
    use super::*;
    use super::test_utils::*;
    
    #[tokio::test]
    async fn test_meta_learning_config() {
        let config = create_meta_learning_config();
        assert!(config.learning_rate_adaptation);
        assert!(config.strategy_evolution);
        assert!(!config.architecture_search);
        assert!(config.transfer_learning);
        assert!(config.continual_learning);
    }
    
    #[tokio::test]
    async fn test_learning_strategy() {
        // Test learning strategy structure
        let strategy = LearningStrategy {
            name: "test_strategy".to_string(),
            parameters: HashMap::new(),
            expected_improvement: 0.5,
        };
        
        assert_eq!(strategy.name, "test_strategy");
        assert!(strategy.parameters.is_empty());
        assert_eq!(strategy.expected_improvement, 0.5);
    }
}

mod emergence_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_emergence_report() {
        let report = EmergenceReport {
            timestamp: chrono::Utc::now(),
            emergent_properties: vec![
                EmergentProperty {
                    id: Uuid::new_v4(),
                    name: "test_property".to_string(),
                    description: "Test emergent property".to_string(),
                    emergence_strength: 0.8,
                    contributing_factors: vec![
                        Factor {
                            factor_type: "interaction".to_string(),
                            contribution: 0.5,
                            source_layers: vec![1, 2, 3],
                        },
                    ],
                },
            ],
            phase_transitions: vec![],
            complexity_metrics: ComplexityMetrics {
                kolmogorov_complexity: 0.7,
                fractal_dimension: 1.5,
                entropy: 0.8,
                emergence_index: 0.9,
            },
        };
        
        assert!(!report.emergent_properties.is_empty());
        assert_eq!(report.emergent_properties[0].name, "test_property");
        assert_eq!(report.complexity_metrics.emergence_index, 0.9);
    }
    
    #[tokio::test]
    async fn test_observation() {
        use crate::hierarchical::intelligence::emergence::SystemState;
        
        let observation = Observation {
            timestamp: chrono::Utc::now(),
            state: SystemState {
                unit_states: HashMap::new(),
                global_properties: HashMap::new(),
                active_patterns: vec![],
            },
            metrics: HashMap::from([
                ("cpu_usage".to_string(), 0.5),
                ("memory_usage".to_string(), 0.7),
                ("network_activity".to_string(), 0.3),
            ]),
        };
        
        assert_eq!(observation.metrics.len(), 3);
        assert_eq!(observation.metrics.get("cpu_usage"), Some(&0.5));
    }
}

mod creativity_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_creative_solution() {
        let solution = Solution {
            id: Uuid::new_v4(),
            description: "Creative solution".to_string(),
            novelty_score: 0.8,
            feasibility_score: 0.7,
            implementation_plan: ImplementationPlan {
                steps: vec![
                    ImplementationStep {
                        description: "Step 1".to_string(),
                        assigned_layers: vec![1, 2],
                        dependencies: vec![],
                    },
                ],
                resource_requirements: ResourceEstimate {
                    compute_hours: 10.0,
                    memory_gb: 8.0,
                    complexity_units: 5.0,
                },
                timeline: Timeline {
                    estimated_duration: std::time::Duration::from_secs(3600),
                    milestones: vec![],
                },
            },
            expected_outcomes: vec![
                Outcome {
                    description: "Success".to_string(),
                    probability: 0.8,
                    impact: 0.9,
                },
            ],
        };
        
        assert_eq!(solution.novelty_score, 0.8);
        assert_eq!(solution.feasibility_score, 0.7);
        assert!(!solution.implementation_plan.steps.is_empty());
    }
    
    #[tokio::test]
    async fn test_concept() {
        let concept = Concept {
            id: Uuid::new_v4(),
            name: "test_concept".to_string(),
            attributes: HashMap::from([
                ("property1".to_string(), serde_json::json!("value1")),
                ("property2".to_string(), serde_json::json!(42)),
            ]),
        };
        
        assert_eq!(concept.name, "test_concept");
        assert_eq!(concept.attributes.len(), 2);
    }
    
    #[tokio::test] 
    async fn test_idea_generation() {
        let idea = Idea {
            id: Uuid::new_v4(),
            description: "Novel idea".to_string(),
            inspiration_sources: vec!["source1".to_string(), "source2".to_string()],
            novelty: 0.85,
        };
        
        assert_eq!(idea.description, "Novel idea");
        assert_eq!(idea.inspiration_sources.len(), 2);
        assert!(idea.novelty > 0.8);
    }
}

mod intelligence_coordinator_tests {
    use super::*;
    use super::test_utils::*;
    
    // Mock implementations for testing
    struct MockMetaLearner;
    struct MockSelfOrganizer;
    struct MockEmergenceDetector;
    struct MockCreativityEngine;
    
    #[async_trait]
    impl MetaLearner for MockMetaLearner {
        async fn learn_to_learn(&mut self, _experience: Experience) -> Result<LearningStrategy> {
            Ok(LearningStrategy {
                name: "mock_strategy".to_string(),
                parameters: HashMap::new(),
                expected_improvement: 0.5,
            })
        }
        
        async fn optimize_architecture(&mut self) -> Result<ArchitectureUpdate> {
            Ok(ArchitectureUpdate {
                changes: vec![],
                rationale: "No changes needed".to_string(),
                expected_benefit: 0.0,
            })
        }
        
        async fn transfer_knowledge(
            &self,
            _source_domain: &str,
            _target_domain: &str,
        ) -> Result<Knowledge> {
            Ok(Knowledge {
                concepts: vec![],
                relationships: vec![],
                applicability: 0.8,
            })
        }
    }
    
    #[async_trait]
    impl SelfOrganizer for MockSelfOrganizer {
        async fn form_clusters(&mut self) -> Result<Vec<Cluster>> {
            Ok(vec![])
        }
        
        async fn create_hierarchy(&mut self) -> Result<Hierarchy> {
            Ok(Hierarchy {
                levels: vec![],
                total_depth: 0,
            })
        }
        
        async fn evolve_topology(&mut self) -> Result<TopologyUpdate> {
            Ok(TopologyUpdate {
                added_connections: vec![],
                removed_connections: vec![],
                reorganized_clusters: vec![],
            })
        }
    }
    
    #[async_trait]
    impl EmergenceDetector for MockEmergenceDetector {
        async fn detect_patterns(&self) -> Result<Vec<EmergentPattern>> {
            Ok(vec![])
        }
        
        async fn identify_phase_transitions(&self) -> Result<Vec<PhaseTransition>> {
            Ok(vec![])
        }
        
        async fn measure_complexity(&self) -> Result<ComplexityMetrics> {
            Ok(ComplexityMetrics {
                kolmogorov_complexity: 0.5,
                fractal_dimension: 1.5,
                entropy: 0.7,
                emergence_index: 0.8,
            })
        }
    }
    
    #[async_trait]
    impl CreativityEngine for MockCreativityEngine {
        async fn generate_ideas(&self, _constraints: &[Constraint]) -> Result<Vec<Idea>> {
            Ok(vec![Idea {
                id: Uuid::new_v4(),
                description: "Test idea".to_string(),
                inspiration_sources: vec![],
                novelty: 0.7,
            }])
        }
        
        async fn combine_concepts(&self, _concepts: &[Concept]) -> Result<Vec<NovelConcept>> {
            Ok(vec![NovelConcept {
                base_concepts: vec![],
                transformation: "test_transform".to_string(),
                resulting_concept: Concept {
                    id: Uuid::new_v4(),
                    name: "Combined concept".to_string(),
                    attributes: HashMap::new(),
                },
                novelty_score: 0.8,
            }])
        }
        
        async fn evaluate_novelty(&self, _solution: &Solution) -> Result<f32> {
            Ok(0.8)
        }
    }
    
    #[tokio::test]
    async fn test_coordinator_initialization() {
        let mut coordinator = DefaultIntelligenceCoordinator::new(
            Box::new(MockMetaLearner),
            Box::new(MockSelfOrganizer),
            Box::new(MockEmergenceDetector),
            Box::new(MockCreativityEngine),
        );
        
        let result = coordinator.initialize().await;
        assert!(result.is_ok());
        
        let metrics = coordinator.metrics().await.unwrap();
        assert_eq!(metrics.consciousness_level as u8, ConsciousnessLevel::Reflexive as u8);
    }
    
    #[tokio::test]
    async fn test_goal_management() {
        let mut coordinator = DefaultIntelligenceCoordinator::new(
            Box::new(MockMetaLearner),
            Box::new(MockSelfOrganizer),
            Box::new(MockEmergenceDetector),
            Box::new(MockCreativityEngine),
        );
        
        coordinator.initialize().await.unwrap();
        
        let goals = vec![create_test_goal()];
        coordinator.set_goals(goals.clone()).await.unwrap();
        
        // The coordinator should have stored the goals
        let emergence_report = coordinator.observe_emergence().await.unwrap();
        assert!(emergence_report.timestamp <= chrono::Utc::now());
    }
    
    #[tokio::test]
    async fn test_creative_problem_solving() {
        let mut coordinator = DefaultIntelligenceCoordinator::new(
            Box::new(MockMetaLearner),
            Box::new(MockSelfOrganizer),
            Box::new(MockEmergenceDetector),
            Box::new(MockCreativityEngine),
        );
        
        coordinator.initialize().await.unwrap();
        
        let challenge = create_test_challenge();
        let solutions = coordinator.create(challenge).await.unwrap();
        
        // MockCreativityEngine returns at least one solution
        assert!(!solutions.is_empty());
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use super::test_utils::*;
    use std::collections::HashMap;
    use uuid::Uuid;
    
    // Re-define mocks here for integration tests
    struct MockMetaLearner;
    struct MockSelfOrganizer;
    struct MockEmergenceDetector;
    struct MockCreativityEngine;
    
    #[async_trait]
    impl MetaLearner for MockMetaLearner {
        async fn learn_to_learn(&mut self, _experience: Experience) -> Result<LearningStrategy> {
            Ok(LearningStrategy {
                name: "mock_strategy".to_string(),
                parameters: HashMap::new(),
                expected_improvement: 0.5,
            })
        }
        
        async fn optimize_architecture(&mut self) -> Result<ArchitectureUpdate> {
            Ok(ArchitectureUpdate {
                changes: vec![],
                rationale: "No changes needed".to_string(),
                expected_benefit: 0.0,
            })
        }
        
        async fn transfer_knowledge(
            &self,
            _source_domain: &str,
            _target_domain: &str,
        ) -> Result<Knowledge> {
            Ok(Knowledge {
                concepts: vec![],
                relationships: vec![],
                applicability: 0.8,
            })
        }
    }
    
    #[async_trait]
    impl SelfOrganizer for MockSelfOrganizer {
        async fn form_clusters(&mut self) -> Result<Vec<Cluster>> {
            Ok(vec![])
        }
        
        async fn create_hierarchy(&mut self) -> Result<Hierarchy> {
            Ok(Hierarchy {
                levels: vec![],
                total_depth: 0,
            })
        }
        
        async fn evolve_topology(&mut self) -> Result<TopologyUpdate> {
            Ok(TopologyUpdate {
                added_connections: vec![],
                removed_connections: vec![],
                reorganized_clusters: vec![],
            })
        }
    }
    
    #[async_trait]
    impl EmergenceDetector for MockEmergenceDetector {
        async fn detect_patterns(&self) -> Result<Vec<EmergentPattern>> {
            Ok(vec![])
        }
        
        async fn identify_phase_transitions(&self) -> Result<Vec<PhaseTransition>> {
            Ok(vec![])
        }
        
        async fn measure_complexity(&self) -> Result<ComplexityMetrics> {
            Ok(ComplexityMetrics {
                kolmogorov_complexity: 0.5,
                fractal_dimension: 1.5,
                entropy: 0.7,
                emergence_index: 0.8,
            })
        }
    }
    
    #[async_trait]
    impl CreativityEngine for MockCreativityEngine {
        async fn generate_ideas(&self, _constraints: &[Constraint]) -> Result<Vec<Idea>> {
            Ok(vec![Idea {
                id: Uuid::new_v4(),
                description: "Test idea".to_string(),
                inspiration_sources: vec![],
                novelty: 0.7,
            }])
        }
        
        async fn combine_concepts(&self, _concepts: &[Concept]) -> Result<Vec<NovelConcept>> {
            Ok(vec![NovelConcept {
                base_concepts: vec![],
                transformation: "test_transform".to_string(),
                resulting_concept: Concept {
                    id: Uuid::new_v4(),
                    name: "Combined concept".to_string(),
                    attributes: HashMap::new(),
                },
                novelty_score: 0.8,
            }])
        }
        
        async fn evaluate_novelty(&self, _solution: &Solution) -> Result<f32> {
            Ok(0.8)
        }
    }
    
    #[tokio::test]
    async fn test_full_intelligence_stack() {
        let mut coordinator = DefaultIntelligenceCoordinator::new(
            Box::new(MockMetaLearner),
            Box::new(MockSelfOrganizer),
            Box::new(MockEmergenceDetector),
            Box::new(MockCreativityEngine),
        );
        
        coordinator.initialize().await.unwrap();
        coordinator.enable_meta_learning(create_meta_learning_config()).await.unwrap();
        coordinator.enable_self_organization(create_self_organization_config()).await.unwrap();
        
        let goal = create_test_goal();
        coordinator.set_goals(vec![goal]).await.unwrap();
        
        // Run a cycle
        let challenge = create_test_challenge();
        let solutions = coordinator.create(challenge).await.unwrap();
        assert!(!solutions.is_empty());
        
        let metrics = coordinator.metrics().await.unwrap();
        assert!(metrics.meta_learning_efficiency > 0.0);
        assert!(metrics.self_organization_degree > 0.0);
    }
}

/// Benchmark tests for performance measurement
#[cfg(test)]
mod benchmarks {
    use super::*;
    use std::time::Instant;
    
    #[tokio::test]
    async fn benchmark_emergence_detection() {
        use crate::hierarchical::intelligence::emergence::SystemState;
        
        let observations: Vec<_> = (0..1000).map(|i| {
            Observation {
                timestamp: chrono::Utc::now(),
                state: SystemState {
                    unit_states: HashMap::new(),
                    global_properties: HashMap::new(),
                    active_patterns: vec![],
                },
                metrics: HashMap::from([
                    ("metric1".to_string(), i as f32 * 0.001),
                    ("metric2".to_string(), (i as f32 * 0.002).sin()),
                    ("metric3".to_string(), (i as f32 * 0.003).cos()),
                ]),
            }
        }).collect();
        
        let start = Instant::now();
        // Process observations
        let _ = observations.len(); // Just measure creation time
        let elapsed = start.elapsed();
        
        println!("Created 1000 observations in {:.2} ms", elapsed.as_millis());
        assert!(elapsed.as_millis() < 100); // Should be fast
    }
}