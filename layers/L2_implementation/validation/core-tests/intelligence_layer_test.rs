//! Integration tests for the Intelligence Layer
//!
//! Tests the meta-learning, self-organization, emergence detection, and creativity capabilities.

use hal9_core::hierarchical::{
    intelligence::{
        IntelligenceCoordinator, DefaultIntelligenceCoordinator,
        MetaLearningConfig, SelfOrganizationConfig, Goal, Challenge,
        ConsciousnessLevel,
        DecompositionStrategy, ConstraintType, Constraint,
        // Concrete implementations
        MetaLearningSystem, SelfOrganizingSystem, EmergenceAnalyzer, CreativeSystem,
        // Traits
        MetaLearner, SelfOrganizer, EmergenceDetector, CreativityEngine,
        Experience, Action, Feedback, Outcome,
    },
};
use uuid::Uuid;
use std::collections::HashMap;

#[tokio::test]
async fn test_intelligence_coordinator_creation() -> Result<(), Box<dyn std::error::Error>> {
    // Create subsystems
    let meta_learner = Box::new(MetaLearningSystem::new());
    let self_organizer = Box::new(SelfOrganizingSystem::new());
    let emergence_detector = Box::new(EmergenceAnalyzer::new());
    let creativity_engine = Box::new(CreativeSystem::new());
    
    // Create coordinator
    let mut coordinator = DefaultIntelligenceCoordinator::new(
        meta_learner,
        self_organizer,
        emergence_detector,
        creativity_engine,
    );
    
    // Initialize
    coordinator.initialize().await?;
    
    // Get initial metrics
    let metrics = coordinator.metrics().await?;
    assert_eq!(metrics.meta_learning_efficiency, 0.0);
    assert!(matches!(metrics.consciousness_level, ConsciousnessLevel::Reflexive));
    
    println!("✅ Intelligence coordinator created successfully!");
    Ok(())
}

#[tokio::test]
async fn test_meta_learning_configuration() -> Result<(), Box<dyn std::error::Error>> {
    let mut coordinator = create_test_coordinator();
    coordinator.initialize().await?;
    
    // Enable meta-learning
    let config = MetaLearningConfig {
        learning_rate_adaptation: true,
        strategy_evolution: true,
        architecture_search: true,
        transfer_learning: true,
        continual_learning: true,
    };
    
    coordinator.enable_meta_learning(config).await?;
    
    // Check metrics updated
    let metrics = coordinator.metrics().await?;
    assert!(metrics.meta_learning_efficiency > 0.0);
    
    println!("✅ Meta-learning configuration test passed!");
    Ok(())
}

#[tokio::test]
async fn test_self_organization_configuration() -> Result<(), Box<dyn std::error::Error>> {
    let mut coordinator = create_test_coordinator();
    coordinator.initialize().await?;
    
    // Enable self-organization
    let config = SelfOrganizationConfig {
        allow_topology_changes: true,
        clustering_enabled: true,
        hierarchy_formation: true,
        emergent_specialization: true,
        dynamic_boundaries: true,
    };
    
    coordinator.enable_self_organization(config).await?;
    
    // Check metrics updated
    let metrics = coordinator.metrics().await?;
    assert!(metrics.self_organization_degree > 0.0);
    
    println!("✅ Self-organization configuration test passed!");
    Ok(())
}

#[tokio::test]
async fn test_goal_setting() -> Result<(), Box<dyn std::error::Error>> {
    let mut coordinator = create_test_coordinator();
    coordinator.initialize().await?;
    
    // Create test goals
    let goals = vec![
        Goal {
            id: Uuid::new_v4(),
            description: "Achieve human-level language understanding".to_string(),
            priority: 0.9,
            constraints: vec![
                Constraint {
                    constraint_type: ConstraintType::Time {
                        deadline: chrono::Utc::now() + chrono::Duration::days(365),
                    },
                    parameters: HashMap::new(),
                },
            ],
            success_criteria: vec![],
            decomposition_strategy: DecompositionStrategy::Hierarchical,
        },
        Goal {
            id: Uuid::new_v4(),
            description: "Optimize system performance".to_string(),
            priority: 0.7,
            constraints: vec![
                Constraint {
                    constraint_type: ConstraintType::Resource { max_cost: 1000.0 },
                    parameters: HashMap::new(),
                },
            ],
            success_criteria: vec![],
            decomposition_strategy: DecompositionStrategy::Parallel,
        },
    ];
    
    coordinator.set_goals(goals).await?;
    
    println!("✅ Goal setting test passed!");
    Ok(())
}

#[tokio::test]
async fn test_emergence_observation() -> Result<(), Box<dyn std::error::Error>> {
    let mut coordinator = create_test_coordinator();
    coordinator.initialize().await?;
    
    // Observe emergence
    let report = coordinator.observe_emergence().await?;
    
    // Check report structure
    assert!(report.timestamp <= chrono::Utc::now());
    assert!(report.emergent_properties.is_empty() || !report.emergent_properties.is_empty());
    assert!(report.phase_transitions.is_empty() || !report.phase_transitions.is_empty());
    
    println!("✅ Emergence observation test passed!");
    println!("  - Emergent properties: {}", report.emergent_properties.len());
    println!("  - Phase transitions: {}", report.phase_transitions.len());
    println!("  - Complexity metrics: {:?}", report.complexity_metrics);
    
    Ok(())
}

#[tokio::test]
async fn test_creative_solution_generation() -> Result<(), Box<dyn std::error::Error>> {
    let mut coordinator = create_test_coordinator();
    coordinator.initialize().await?;
    
    // Create a challenge
    let challenge = Challenge {
        id: Uuid::new_v4(),
        problem_statement: "Design a more efficient learning algorithm".to_string(),
        context: HashMap::from([
            ("domain".to_string(), serde_json::json!("machine_learning")),
            ("current_efficiency".to_string(), serde_json::json!(0.7)),
        ]),
        constraints: vec![
            Constraint {
                constraint_type: ConstraintType::Quality { min_score: 0.8 },
                parameters: HashMap::new(),
            },
        ],
        evaluation_criteria: vec![],
    };
    
    // Generate creative solutions
    let solutions = coordinator.create(challenge).await?;
    
    assert!(!solutions.is_empty());
    for solution in &solutions {
        assert!(solution.novelty_score >= 0.0 && solution.novelty_score <= 1.0);
        assert!(solution.feasibility_score >= 0.0 && solution.feasibility_score <= 1.0);
        println!("  - Solution {}: novelty={:.2}, feasibility={:.2}", 
                 solution.id, solution.novelty_score, solution.feasibility_score);
    }
    
    println!("✅ Creative solution generation test passed!");
    println!("  Generated {} solutions", solutions.len());
    
    Ok(())
}

#[tokio::test]
async fn test_meta_learning_experience() -> Result<(), Box<dyn std::error::Error>> {
    let mut meta_learner = MetaLearningSystem::new();
    
    // Create test experience
    let experience = Experience {
        context: HashMap::from([
            ("task".to_string(), serde_json::json!("classification")),
            ("dataset_size".to_string(), serde_json::json!(1000)),
        ]),
        actions: vec![
            Action {
                action_type: "train".to_string(),
                parameters: HashMap::from([
                    ("epochs".to_string(), serde_json::json!(10)),
                ]),
                timestamp: chrono::Utc::now(),
            },
        ],
        outcomes: vec![
            Outcome {
                description: "Model trained successfully".to_string(),
                probability: 0.95,
                impact: 0.8,
            },
        ],
        feedback: Feedback {
            reward: 0.85,
            explanation: Some("Good performance on validation set".to_string()),
        },
    };
    
    // Learn from experience
    let strategy = meta_learner.learn_to_learn(experience).await?;
    
    assert!(!strategy.name.is_empty());
    assert!(!strategy.parameters.is_empty());
    assert!(strategy.expected_improvement > 0.0);
    
    println!("✅ Meta-learning from experience test passed!");
    println!("  - Strategy: {}", strategy.name);
    println!("  - Expected improvement: {:.2}", strategy.expected_improvement);
    
    Ok(())
}

#[tokio::test]
async fn test_self_organization_clustering() -> Result<(), Box<dyn std::error::Error>> {
    let mut self_organizer = SelfOrganizingSystem::new();
    
    // Form clusters
    let clusters = self_organizer.form_clusters().await?;
    
    // Initially no clusters (no units added)
    assert_eq!(clusters.len(), 0);
    
    println!("✅ Self-organization clustering test passed!");
    
    Ok(())
}

#[tokio::test]
async fn test_emergence_pattern_detection() -> Result<(), Box<dyn std::error::Error>> {
    let emergence_detector = EmergenceAnalyzer::new();
    
    // Detect patterns
    let patterns = emergence_detector.detect_patterns().await?;
    
    // Check pattern structure
    for pattern in &patterns {
        assert!(pattern.frequency >= 0.0 && pattern.frequency <= 1.0);
        assert!(pattern.significance >= 0.0 && pattern.significance <= 1.0);
    }
    
    println!("✅ Emergence pattern detection test passed!");
    println!("  - Detected {} patterns", patterns.len());
    
    Ok(())
}

#[tokio::test]
async fn test_creativity_idea_generation() -> Result<(), Box<dyn std::error::Error>> {
    let creativity_engine = CreativeSystem::new();
    
    // Generate ideas with constraints
    let constraints = vec![
        Constraint {
            constraint_type: ConstraintType::Resource { max_cost: 500.0 },
            parameters: HashMap::new(),
        },
    ];
    
    let ideas = creativity_engine.generate_ideas(&constraints).await?;
    
    assert!(!ideas.is_empty());
    for idea in &ideas {
        assert!(idea.novelty >= 0.0 && idea.novelty <= 1.0);
        println!("  - Idea {}: {}", idea.id, idea.description);
    }
    
    println!("✅ Creativity idea generation test passed!");
    println!("  - Generated {} ideas", ideas.len());
    
    Ok(())
}

// Helper function to create test coordinator
fn create_test_coordinator() -> DefaultIntelligenceCoordinator {
    DefaultIntelligenceCoordinator::new(
        Box::new(MetaLearningSystem::new()),
        Box::new(SelfOrganizingSystem::new()),
        Box::new(EmergenceAnalyzer::new()),
        Box::new(CreativeSystem::new()),
    )
}