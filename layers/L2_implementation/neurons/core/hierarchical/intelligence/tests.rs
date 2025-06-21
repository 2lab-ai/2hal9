//! Intelligence layer tests

#[cfg(test)]
use super::*;
use uuid::Uuid;
use std::collections::HashMap;
use chrono::Utc;

// Mock implementations for testing
struct MockMetaLearner {
    learning_efficiency: f32,
}

#[async_trait]
impl MetaLearner for MockMetaLearner {
    async fn learn_to_learn(&mut self, _experience: Experience) -> Result<LearningStrategy> {
        self.learning_efficiency += 0.01;
        Ok(LearningStrategy {
            name: "adaptive_gradient".to_string(),
            parameters: {
                let mut params = HashMap::new();
                params.insert("learning_rate".to_string(), 0.001);
                params.insert("momentum".to_string(), 0.9);
                params
            },
            expected_improvement: 0.1,
        })
    }
    
    async fn optimize_architecture(&mut self) -> Result<ArchitectureUpdate> {
        Ok(ArchitectureUpdate {
            changes: vec![
                ArchitectureChange::AdjustParameters {
                    unit_id: Uuid::new_v4(),
                    params: {
                        let mut params = HashMap::new();
                        params.insert("threshold".to_string(), 0.5);
                        params
                    },
                },
            ],
            rationale: "Improved convergence detected".to_string(),
            expected_benefit: 0.15,
        })
    }
    
    async fn transfer_knowledge(&self, source_domain: &str, _target_domain: &str) -> Result<Knowledge> {
        Ok(Knowledge {
            concepts: vec![
                Concept {
                    id: Uuid::new_v4(),
                    name: format!("{}_concept", source_domain),
                    attributes: HashMap::new(),
                },
            ],
            relationships: vec![],
            applicability: 0.8,
        })
    }
}

struct MockSelfOrganizer {
    organization_level: f32,
}

#[async_trait]
impl SelfOrganizer for MockSelfOrganizer {
    async fn form_clusters(&mut self) -> Result<Vec<Cluster>> {
        Ok(vec![
            Cluster {
                id: Uuid::new_v4(),
                members: vec![Uuid::new_v4(), Uuid::new_v4()],
                purpose: "computation".to_string(),
                cohesion: 0.85,
            },
            Cluster {
                id: Uuid::new_v4(),
                members: vec![Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4()],
                purpose: "memory".to_string(),
                cohesion: 0.90,
            },
        ])
    }
    
    async fn create_hierarchy(&mut self) -> Result<Hierarchy> {
        Ok(Hierarchy {
            levels: vec![
                HierarchyLevel {
                    level: 0,
                    units: vec![Uuid::new_v4(); 5],
                    abstraction_degree: 0.0,
                },
                HierarchyLevel {
                    level: 1,
                    units: vec![Uuid::new_v4(); 3],
                    abstraction_degree: 0.5,
                },
                HierarchyLevel {
                    level: 2,
                    units: vec![Uuid::new_v4()],
                    abstraction_degree: 1.0,
                },
            ],
            total_depth: 3,
        })
    }
    
    async fn evolve_topology(&mut self) -> Result<TopologyUpdate> {
        self.organization_level += 0.05;
        Ok(TopologyUpdate {
            added_connections: vec![(Uuid::new_v4(), Uuid::new_v4())],
            removed_connections: vec![],
            reorganized_clusters: vec![],
        })
    }
}

struct MockEmergenceDetector;

#[async_trait]
impl EmergenceDetector for MockEmergenceDetector {
    async fn detect_patterns(&self) -> Result<Vec<EmergentPattern>> {
        Ok(vec![
            EmergentPattern {
                pattern_id: Uuid::new_v4(),
                description: "Synchronization observed".to_string(),
                frequency: 0.7,
                significance: 0.8,
            },
            EmergentPattern {
                pattern_id: Uuid::new_v4(),
                description: "Hierarchical specialization".to_string(),
                frequency: 0.5,
                significance: 0.9,
            },
        ])
    }
    
    async fn identify_phase_transitions(&self) -> Result<Vec<PhaseTransition>> {
        Ok(vec![
            PhaseTransition {
                from_state: "chaotic".to_string(),
                to_state: "ordered".to_string(),
                transition_point: 0.65,
                hysteresis: 0.1,
            },
        ])
    }
    
    async fn measure_complexity(&self) -> Result<ComplexityMetrics> {
        Ok(ComplexityMetrics {
            kolmogorov_complexity: 0.7,
            fractal_dimension: 1.4,
            entropy: 0.6,
            emergence_index: 0.75,
        })
    }
}

struct MockCreativityEngine;

#[async_trait]
impl CreativityEngine for MockCreativityEngine {
    async fn generate_ideas(&self, _constraints: &[Constraint]) -> Result<Vec<Idea>> {
        Ok(vec![
            Idea {
                id: Uuid::new_v4(),
                description: "Combine pattern matching with gradient flow".to_string(),
                inspiration_sources: vec!["neural_networks".to_string(), "fluid_dynamics".to_string()],
                novelty: 0.8,
            },
            Idea {
                id: Uuid::new_v4(),
                description: "Use quantum superposition for parallel exploration".to_string(),
                inspiration_sources: vec!["quantum_computing".to_string()],
                novelty: 0.95,
            },
        ])
    }
    
    async fn combine_concepts(&self, concepts: &[Concept]) -> Result<Vec<NovelConcept>> {
        if concepts.len() < 2 {
            return Ok(vec![]);
        }
        
        Ok(vec![
            NovelConcept {
                base_concepts: concepts.iter().map(|c| c.id).collect(),
                transformation: "intersection".to_string(),
                resulting_concept: Concept {
                    id: Uuid::new_v4(),
                    name: "hybrid_concept".to_string(),
                    attributes: HashMap::new(),
                },
                novelty_score: 0.7,
            },
        ])
    }
    
    async fn evaluate_novelty(&self, solution: &Solution) -> Result<f32> {
        Ok(solution.novelty_score * 0.9) // Slightly discount self-reported novelty
    }
}

#[tokio::test]
async fn test_default_intelligence_coordinator() {
    let meta_learner = Box::new(MockMetaLearner { learning_efficiency: 0.5 });
    let self_organizer = Box::new(MockSelfOrganizer { organization_level: 0.3 });
    let emergence_detector = Box::new(MockEmergenceDetector);
    let creativity_engine = Box::new(MockCreativityEngine);
    
    let mut coordinator = DefaultIntelligenceCoordinator::new(
        meta_learner,
        self_organizer,
        emergence_detector,
        creativity_engine,
    );
    
    // Test initialization
    assert!(coordinator.initialize().await.is_ok());
    
    // Test goal setting
    let goals = vec![
        Goal {
            id: Uuid::new_v4(),
            description: "Achieve 95% accuracy on pattern recognition".to_string(),
            priority: 1.0,
            constraints: vec![
                Constraint {
                    constraint_type: ConstraintType::Time {
                        deadline: Utc::now() + chrono::Duration::days(7),
                    },
                    parameters: HashMap::new(),
                },
            ],
            success_criteria: vec![
                Criterion {
                    name: "accuracy".to_string(),
                    measurement: Measurement::Absolute,
                    target_value: 0.95,
                },
            ],
            decomposition_strategy: DecompositionStrategy::Hierarchical,
        },
    ];
    
    assert!(coordinator.set_goals(goals).await.is_ok());
    
    // Test meta-learning configuration
    let meta_config = MetaLearningConfig {
        learning_rate_adaptation: true,
        strategy_evolution: true,
        architecture_search: true,
        transfer_learning: true,
        continual_learning: true,
    };
    
    assert!(coordinator.enable_meta_learning(meta_config).await.is_ok());
    
    // Test self-organization configuration
    let self_org_config = SelfOrganizationConfig {
        allow_topology_changes: true,
        clustering_enabled: true,
        hierarchy_formation: true,
        emergent_specialization: true,
        dynamic_boundaries: true,
    };
    
    assert!(coordinator.enable_self_organization(self_org_config).await.is_ok());
}

#[tokio::test]
async fn test_emergence_observation() {
    let meta_learner = Box::new(MockMetaLearner { learning_efficiency: 0.5 });
    let self_organizer = Box::new(MockSelfOrganizer { organization_level: 0.3 });
    let emergence_detector = Box::new(MockEmergenceDetector);
    let creativity_engine = Box::new(MockCreativityEngine);
    
    let mut coordinator = DefaultIntelligenceCoordinator::new(
        meta_learner,
        self_organizer,
        emergence_detector,
        creativity_engine,
    );
    
    coordinator.initialize().await.unwrap();
    
    // Test emergence observation
    let report = coordinator.observe_emergence().await.unwrap();
    
    assert!(!report.emergent_properties.is_empty());
    assert!(!report.phase_transitions.is_empty());
    assert!(report.complexity_metrics.emergence_index > 0.0);
}

#[tokio::test]
async fn test_creative_problem_solving() {
    let meta_learner = Box::new(MockMetaLearner { learning_efficiency: 0.5 });
    let self_organizer = Box::new(MockSelfOrganizer { organization_level: 0.3 });
    let emergence_detector = Box::new(MockEmergenceDetector);
    let creativity_engine = Box::new(MockCreativityEngine);
    
    let mut coordinator = DefaultIntelligenceCoordinator::new(
        meta_learner,
        self_organizer,
        emergence_detector,
        creativity_engine,
    );
    
    coordinator.initialize().await.unwrap();
    
    // Test creative problem solving
    let challenge = Challenge {
        id: Uuid::new_v4(),
        problem_statement: "Design a system that can learn from minimal examples".to_string(),
        context: HashMap::new(),
        constraints: vec![
            Constraint {
                constraint_type: ConstraintType::Resource { max_cost: 1000.0 },
                parameters: HashMap::new(),
            },
        ],
        evaluation_criteria: vec![
            Criterion {
                name: "sample_efficiency".to_string(),
                measurement: Measurement::Relative { baseline: 100.0 },
                target_value: 10.0,
            },
        ],
    };
    
    let solutions = coordinator.create(challenge).await.unwrap();
    
    assert!(!solutions.is_empty());
    assert!(solutions[0].novelty_score > 0.0);
    assert!(solutions[0].feasibility_score > 0.0);
}

#[tokio::test]
async fn test_intelligence_metrics() {
    let meta_learner = Box::new(MockMetaLearner { learning_efficiency: 0.5 });
    let self_organizer = Box::new(MockSelfOrganizer { organization_level: 0.3 });
    let emergence_detector = Box::new(MockEmergenceDetector);
    let creativity_engine = Box::new(MockCreativityEngine);
    
    let mut coordinator = DefaultIntelligenceCoordinator::new(
        meta_learner,
        self_organizer,
        emergence_detector,
        creativity_engine,
    );
    
    coordinator.initialize().await.unwrap();
    
    // Enable all features
    let meta_config = MetaLearningConfig {
        learning_rate_adaptation: true,
        strategy_evolution: true,
        architecture_search: true,
        transfer_learning: true,
        continual_learning: true,
    };
    coordinator.enable_meta_learning(meta_config).await.unwrap();
    
    let self_org_config = SelfOrganizationConfig {
        allow_topology_changes: true,
        clustering_enabled: true,
        hierarchy_formation: true,
        emergent_specialization: true,
        dynamic_boundaries: true,
    };
    coordinator.enable_self_organization(self_org_config).await.unwrap();
    
    // Get metrics
    let metrics = coordinator.metrics().await.unwrap();
    
    assert!(metrics.meta_learning_efficiency >= 0.0);
    assert!(metrics.self_organization_degree >= 0.0);
    assert!(metrics.goal_achievement_rate >= 0.0);
    assert!(metrics.creativity_index >= 0.0);
    assert!(metrics.adaptation_speed >= 0.0);
    
    match metrics.consciousness_level {
        ConsciousnessLevel::Reflexive => (),
        _ => panic!("Expected reflexive consciousness level for initial state"),
    }
}

#[tokio::test]
async fn test_consciousness_evolution() {
    // This test demonstrates how consciousness levels might evolve
    let levels = [
        ConsciousnessLevel::Reflexive,
        ConsciousnessLevel::Aware,
        ConsciousnessLevel::SelfAware,
        ConsciousnessLevel::MetaAware,
        ConsciousnessLevel::Transcendent,
    ];
    
    for (i, level) in levels.iter().enumerate() {
        match level {
            ConsciousnessLevel::Reflexive => {
                assert_eq!(i, 0);
            },
            ConsciousnessLevel::Aware => {
                assert_eq!(i, 1);
            },
            ConsciousnessLevel::SelfAware => {
                assert_eq!(i, 2);
            },
            ConsciousnessLevel::MetaAware => {
                assert_eq!(i, 3);
            },
            ConsciousnessLevel::Transcendent => {
                assert_eq!(i, 4);
            },
        }
    }
}
}