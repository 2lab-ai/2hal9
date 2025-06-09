//! Performance benchmarks for the HAL9 hierarchical architecture
//!
//! Validates that each layer meets its design performance targets:
//! - L1 Reflexive: < 10ms response time
//! - L2 Implementation: 50-200ms for code generation
//! - L3 Operational: 100-500ms for design creation
//! - L4 Tactical: 200-1000ms for plan development
//! - L5 Strategic: 500-2000ms for strategic decisions

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use hal9_core::hierarchical::{
    cognitive::{
        CognitiveLayer, CognitiveInput,
        l1_reflexive::L1ReflexiveNeuron,
        l2_implementation::L2ImplementationNeuron,
        l3_operational::L3OperationalNeuron,
        l4_tactical::L4TacticalNeuron,
        l5_strategic::L5StrategicNeuron,
        factory::{CognitiveUnitBuilder, CognitiveConfig},
        ConnectionConfig,
    },
    intelligence::{
        MetaLearningSystem, SelfOrganizingSystem,
        EmergenceAnalyzer, CreativeSystem,
        Experience, Action, Feedback, Outcome,
    },
};
use std::collections::HashMap;
use uuid::Uuid;
use tokio::runtime::Runtime;

/// Benchmark L1 Reflexive layer performance
fn benchmark_l1_reflexive(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let config = CognitiveConfig {
        id: Uuid::new_v4(),
        layer: CognitiveLayer::Reflexive,
        initial_parameters: HashMap::from([
            ("learning_rate".to_string(), 0.1),
            ("response_threshold".to_string(), 0.5),
        ]),
        connections: ConnectionConfig {
            upward_connections: vec![],
            lateral_connections: vec![],
            downward_connections: vec![],
        },
    };
    
    let neuron = L1ReflexiveNeuron::new(config);
    
    // Add some patterns for testing
    neuron.add_pattern(hal9_core::hierarchical::cognitive::l1_reflexive::Pattern {
        trigger: "hello".to_string(),
        response: "Hi there!".to_string(),
        confidence: 0.9,
    });
    
    let input = CognitiveInput {
        content: "hello".to_string(),
        source_layer: None,
        context: HashMap::new(),
    };
    
    c.bench_function("L1 Reflexive Response", |b| {
        b.to_async(&rt).iter(|| async {
            let mut neuron_clone = neuron.clone();
            black_box(neuron_clone.process(input.clone()).await.unwrap())
        });
    });
}

/// Benchmark L2 Implementation layer performance
fn benchmark_l2_implementation(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let config = CognitiveConfig {
        id: Uuid::new_v4(),
        layer: CognitiveLayer::Implementation,
        initial_parameters: HashMap::from([
            ("learning_rate".to_string(), 0.05),
            ("complexity_threshold".to_string(), 0.7),
        ]),
        connections: ConnectionConfig::default(),
    };
    
    let neuron = L2ImplementationNeuron::new(config);
    
    let inputs = vec![
        ("simple", CognitiveInput {
            content: "Create a function to add two numbers".to_string(),
            source_layer: Some(CognitiveLayer::Implementation),
            context: HashMap::from([
                ("language".to_string(), serde_json::json!("rust")),
            ]),
        }),
        ("complex", CognitiveInput {
            content: "Implement a binary search tree with insert and search methods".to_string(),
            source_layer: Some(CognitiveLayer::Implementation),
            context: HashMap::from([
                ("language".to_string(), serde_json::json!("rust")),
                ("requirements".to_string(), serde_json::json!(["balanced", "generic"])),
            ]),
        }),
    ];
    
    let mut group = c.benchmark_group("L2 Implementation");
    for (name, input) in inputs {
        group.bench_with_input(BenchmarkId::from_parameter(name), &input, |b, input| {
            b.to_async(&rt).iter(|| async {
                let mut neuron_clone = neuron.clone();
                black_box(neuron_clone.process(input.clone()).await.unwrap())
            });
        });
    }
    group.finish();
}

/// Benchmark L3 Operational layer performance
fn benchmark_l3_operational(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let config = CognitiveConfig {
        id: Uuid::new_v4(),
        layer: CognitiveLayer::Operational,
        initial_parameters: HashMap::from([
            ("learning_rate".to_string(), 0.02),
        ]),
        connections: ConnectionConfig::default(),
    };
    
    let neuron = L3OperationalNeuron::new(config);
    
    let input = CognitiveInput {
        content: "Design a microservices architecture for an e-commerce platform".to_string(),
        source_layer: Some(CognitiveLayer::Operational),
        context: HashMap::from([
            ("scale".to_string(), serde_json::json!("large")),
            ("requirements".to_string(), serde_json::json!(["scalable", "resilient"])),
        ]),
    };
    
    c.bench_function("L3 Operational Design", |b| {
        b.to_async(&rt).iter(|| async {
            let mut neuron_clone = neuron.clone();
            black_box(neuron_clone.process(input.clone()).await.unwrap())
        });
    });
}

/// Benchmark L4 Tactical layer performance
fn benchmark_l4_tactical(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let config = CognitiveConfig {
        id: Uuid::new_v4(),
        layer: CognitiveLayer::Tactical,
        initial_parameters: HashMap::from([
            ("learning_rate".to_string(), 0.01),
            ("planning_horizon".to_string(), 300.0), // 5 minutes
        ]),
        connections: ConnectionConfig::default(),
    };
    
    let neuron = L4TacticalNeuron::new(config);
    
    let input = CognitiveInput {
        content: "Create a 3-month development plan for launching a new AI product".to_string(),
        source_layer: Some(CognitiveLayer::Tactical),
        context: HashMap::from([
            ("team_size".to_string(), serde_json::json!(10)),
            ("budget".to_string(), serde_json::json!(500000)),
        ]),
    };
    
    c.bench_function("L4 Tactical Planning", |b| {
        b.to_async(&rt).iter(|| async {
            let mut neuron_clone = neuron.clone();
            black_box(neuron_clone.process(input.clone()).await.unwrap())
        });
    });
}

/// Benchmark L5 Strategic layer performance
fn benchmark_l5_strategic(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let config = CognitiveConfig {
        id: Uuid::new_v4(),
        layer: CognitiveLayer::Strategic,
        initial_parameters: HashMap::from([
            ("learning_rate".to_string(), 0.005),
            ("vision_horizon".to_string(), 3600.0), // 1 hour
        ]),
        connections: ConnectionConfig::default(),
    };
    
    let neuron = L5StrategicNeuron::new(config);
    
    let input = CognitiveInput {
        content: "Define a 5-year vision for achieving artificial general intelligence".to_string(),
        source_layer: Some(CognitiveLayer::Strategic),
        context: HashMap::from([
            ("current_capabilities".to_string(), serde_json::json!("narrow AI")),
            ("resources".to_string(), serde_json::json!("unlimited")),
        ]),
    };
    
    c.bench_function("L5 Strategic Vision", |b| {
        b.to_async(&rt).iter(|| async {
            let mut neuron_clone = neuron.clone();
            black_box(neuron_clone.process(input.clone()).await.unwrap())
        });
    });
}

/// Benchmark meta-learning system
fn benchmark_meta_learning(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut meta_learner = MetaLearningSystem::new();
    
    let experience = Experience {
        context: HashMap::from([
            ("task".to_string(), serde_json::json!("classification")),
            ("dataset_size".to_string(), serde_json::json!(10000)),
        ]),
        actions: vec![
            Action {
                action_type: "train".to_string(),
                parameters: HashMap::from([
                    ("epochs".to_string(), serde_json::json!(50)),
                    ("batch_size".to_string(), serde_json::json!(32)),
                ]),
                timestamp: chrono::Utc::now(),
            },
        ],
        outcomes: vec![
            Outcome {
                description: "Model achieved 95% accuracy".to_string(),
                probability: 0.9,
                impact: 0.85,
            },
        ],
        feedback: Feedback {
            reward: 0.9,
            explanation: Some("Excellent performance".to_string()),
        },
    };
    
    c.bench_function("Meta-Learning Strategy Selection", |b| {
        b.to_async(&rt).iter(|| async {
            let mut ml_clone = meta_learner.clone();
            black_box(ml_clone.learn_to_learn(experience.clone()).await.unwrap())
        });
    });
}

/// Benchmark self-organization clustering
fn benchmark_self_organization(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut self_organizer = SelfOrganizingSystem::new();
    
    // Add some units for clustering (in a real test, we'd add actual units)
    // For now, we benchmark the empty case
    
    c.bench_function("Self-Organization Clustering", |b| {
        b.to_async(&rt).iter(|| async {
            let mut so_clone = self_organizer.clone();
            black_box(so_clone.form_clusters().await.unwrap())
        });
    });
}

/// Benchmark emergence detection
fn benchmark_emergence_detection(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let emergence_detector = EmergenceAnalyzer::new();
    
    c.bench_function("Emergence Pattern Detection", |b| {
        b.to_async(&rt).iter(|| async {
            let ed_clone = emergence_detector.clone();
            black_box(ed_clone.detect_patterns().await.unwrap())
        });
    });
}

/// Benchmark creative idea generation
fn benchmark_creativity(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let creativity_engine = CreativeSystem::new();
    
    let constraints = vec![
        hal9_core::hierarchical::intelligence::Constraint {
            constraint_type: hal9_core::hierarchical::intelligence::ConstraintType::Resource { 
                max_cost: 1000.0 
            },
            parameters: HashMap::new(),
        },
    ];
    
    c.bench_function("Creative Idea Generation", |b| {
        b.to_async(&rt).iter(|| async {
            let ce_clone = creativity_engine.clone();
            black_box(ce_clone.generate_ideas(&constraints).await.unwrap())
        });
    });
}

/// Benchmark inter-layer communication
fn benchmark_layer_communication(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    use hal9_core::hierarchical::interfaces::{LayerMessage, LayerId, MessageType, MessagePriority};
    
    let message = LayerMessage {
        id: Uuid::new_v4(),
        source_layer: LayerId::Cognitive,
        target_layer: LayerId::Intelligence,
        message_type: MessageType::Data,
        payload: serde_json::json!({
            "content": "Test message",
            "data": vec![1, 2, 3, 4, 5],
        }),
        timestamp: chrono::Utc::now(),
        priority: MessagePriority::Normal,
    };
    
    c.bench_function("Layer Message Serialization", |b| {
        b.iter(|| {
            let serialized = serde_json::to_string(&message).unwrap();
            let _deserialized: LayerMessage = serde_json::from_str(&serialized).unwrap();
            black_box(serialized)
        });
    });
}

/// Main benchmark groups
criterion_group!(
    cognitive_benches,
    benchmark_l1_reflexive,
    benchmark_l2_implementation,
    benchmark_l3_operational,
    benchmark_l4_tactical,
    benchmark_l5_strategic
);

criterion_group!(
    intelligence_benches,
    benchmark_meta_learning,
    benchmark_self_organization,
    benchmark_emergence_detection,
    benchmark_creativity
);

criterion_group!(
    communication_benches,
    benchmark_layer_communication
);

criterion_main!(cognitive_benches, intelligence_benches, communication_benches);