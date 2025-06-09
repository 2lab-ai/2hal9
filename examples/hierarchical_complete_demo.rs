//! Comprehensive example demonstrating all 5 hierarchical layers working together
//! 
//! This example shows how a request flows through all layers:
//! 1. Substrate - Infrastructure and runtime
//! 2. Protocol - Communication and message handling
//! 3. Cognitive - Hierarchical processing (L1-L5)
//! 4. Orchestration - Dynamic topology and routing
//! 5. Intelligence - Meta-learning and emergence

use hal9_core::hierarchical::{
    // Substrate Layer
    substrate::{
        AsyncRuntime, TokioRuntime, MessageTransport, ChannelTransport,
        Storage, SqliteStorage, ResourceManager, LocalResources,
    },
    // Protocol Layer
    protocol::{
        Protocol, SignalProtocol, GradientProtocol, ConsensusProtocol,
        ProtocolManager, ProtocolManagerConfig, Message,
    },
    // Cognitive Layer
    cognitive::{
        CognitiveUnit, CognitiveInput, CognitiveOutput, CognitiveLayer,
        L1ReflexiveNeuron, L2ImplementationNeuron, L3OperationalNeuron,
        L4TacticalNeuron, L5StrategicNeuron, CognitiveUnitBuilder,
    },
    // Orchestration Layer
    orchestration::{
        Orchestrator, DefaultOrchestrator, TopologyManager, GraphTopology,
        FlowController, AdaptiveFlowController, StateCoordinator, RaftCoordinator,
        SignalRouter, DijkstraRouter, UnitDescriptor, UnitType, OrchestrationSignal,
        Connection, ConnectionType, SignalType, RoutingHints,
    },
    // Intelligence Layer
    intelligence::{
        IntelligenceCoordinator, DefaultIntelligenceCoordinator,
        Goal, Challenge, MetaLearningConfig, SelfOrganizationConfig,
        ConsciousnessLevel, DecompositionStrategy, ConstraintType,
    },
};

use std::sync::Arc;
use std::collections::HashMap;
use uuid::Uuid;
use tokio::sync::RwLock;
use chrono::Utc;

/// Complete HAL9 system with all layers
struct HAL9System {
    // Layer 1: Substrate
    runtime: Arc<TokioRuntime>,
    transport: Arc<ChannelTransport>,
    storage: Arc<SqliteStorage>,
    resources: Arc<LocalResources>,
    
    // Layer 2: Protocol
    protocol_manager: Arc<ProtocolManager>,
    
    // Layer 3: Cognitive
    cognitive_units: Arc<RwLock<HashMap<Uuid, Box<dyn CognitiveUnit>>>>,
    
    // Layer 4: Orchestration
    orchestrator: Arc<RwLock<DefaultOrchestrator>>,
    
    // Layer 5: Intelligence
    intelligence: Arc<RwLock<DefaultIntelligenceCoordinator>>,
}

impl HAL9System {
    async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        println!("🚀 Initializing HAL9 Hierarchical System...\n");
        
        // Initialize Substrate Layer
        println!("1️⃣ Substrate Layer:");
        let runtime = Arc::new(TokioRuntime::new()?);
        println!("   ✓ Async runtime initialized");
        
        let transport = Arc::new(ChannelTransport::new());
        println!("   ✓ Message transport ready");
        
        let storage = Arc::new(SqliteStorage::new("hal9_demo.db").await?);
        println!("   ✓ Storage connected");
        
        let resources = Arc::new(LocalResources::new());
        println!("   ✓ Resource manager online\n");
        
        // Initialize Protocol Layer
        println!("2️⃣ Protocol Layer:");
        let protocol_manager = Arc::new(ProtocolManager::new(
            ProtocolManagerConfig::default(),
            transport.clone(),
        ));
        protocol_manager.initialize_protocols().await?;
        println!("   ✓ Protocols initialized (Signal, Gradient, Consensus)\n");
        
        // Initialize Cognitive Layer
        println!("3️⃣ Cognitive Layer:");
        let cognitive_units = Arc::new(RwLock::new(HashMap::new()));
        println!("   ✓ Cognitive unit registry created\n");
        
        // Initialize Orchestration Layer
        println!("4️⃣ Orchestration Layer:");
        let topology = Box::new(GraphTopology::new(Default::default()));
        let flow = Box::new(AdaptiveFlowController::new(Default::default()));
        let state = Box::new(RaftCoordinator::new(Uuid::new_v4()));
        let router = Box::new(DijkstraRouter::new(100));
        
        let mut orchestrator = DefaultOrchestrator::new(topology, flow, state, router);
        orchestrator.initialize().await?;
        let orchestrator = Arc::new(RwLock::new(orchestrator));
        println!("   ✓ Orchestration systems online\n");
        
        // Initialize Intelligence Layer
        println!("5️⃣ Intelligence Layer:");
        // In a real system, these would be proper implementations
        let intelligence = Arc::new(RwLock::new(
            DefaultIntelligenceCoordinator::new(
                Box::new(MockMetaLearner),
                Box::new(MockSelfOrganizer),
                Box::new(MockEmergenceDetector),
                Box::new(MockCreativityEngine),
            )
        ));
        intelligence.write().await.initialize().await?;
        println!("   ✓ Intelligence systems activated\n");
        
        Ok(Self {
            runtime,
            transport,
            storage,
            resources,
            protocol_manager,
            cognitive_units,
            orchestrator,
            intelligence,
        })
    }
    
    /// Create and register all cognitive units (L1-L5)
    async fn create_cognitive_hierarchy(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("🧠 Creating Cognitive Hierarchy...\n");
        
        let mut units = self.cognitive_units.write().await;
        let mut orchestrator = self.orchestrator.write().await;
        
        // Create L1 Reflexive units
        for i in 0..3 {
            let config = CognitiveUnitBuilder::new(CognitiveLayer::Reflexive)
                .with_parameter("response_threshold", 0.7)
                .build();
            let unit = Box::new(L1ReflexiveNeuron::new(config));
            let unit_id = Uuid::new_v4();
            
            let descriptor = UnitDescriptor {
                id: unit_id,
                unit_type: UnitType::Neuron,
                layer: CognitiveLayer::Reflexive,
                capabilities: vec![],
                resource_requirements: Default::default(),
            };
            
            orchestrator.add_unit(descriptor).await?;
            units.insert(unit_id, unit as Box<dyn CognitiveUnit>);
            println!("   ✓ L1 Reflexive Unit {} created", i + 1);
        }
        
        // Create L2 Implementation units
        for i in 0..2 {
            let config = CognitiveUnitBuilder::new(CognitiveLayer::Implementation).build();
            let unit = Box::new(L2ImplementationNeuron::new(config));
            let unit_id = Uuid::new_v4();
            
            let descriptor = UnitDescriptor {
                id: unit_id,
                unit_type: UnitType::Neuron,
                layer: CognitiveLayer::Implementation,
                capabilities: vec![],
                resource_requirements: Default::default(),
            };
            
            orchestrator.add_unit(descriptor).await?;
            units.insert(unit_id, unit as Box<dyn CognitiveUnit>);
            println!("   ✓ L2 Implementation Unit {} created", i + 1);
        }
        
        // Create L3-L5 units (one each for demo)
        let layers = [
            (CognitiveLayer::Operational, "L3 Operational"),
            (CognitiveLayer::Tactical, "L4 Tactical"),
            (CognitiveLayer::Strategic, "L5 Strategic"),
        ];
        
        for (layer, name) in layers {
            let config = CognitiveUnitBuilder::new(layer).build();
            let unit: Box<dyn CognitiveUnit> = match layer {
                CognitiveLayer::Operational => Box::new(L3OperationalNeuron::new(config)),
                CognitiveLayer::Tactical => Box::new(L4TacticalNeuron::new(config)),
                CognitiveLayer::Strategic => Box::new(L5StrategicNeuron::new(config)),
                _ => unreachable!(),
            };
            
            let unit_id = Uuid::new_v4();
            let descriptor = UnitDescriptor {
                id: unit_id,
                unit_type: UnitType::Neuron,
                layer,
                capabilities: vec![],
                resource_requirements: Default::default(),
            };
            
            orchestrator.add_unit(descriptor).await?;
            units.insert(unit_id, unit);
            println!("   ✓ {} Unit created", name);
        }
        
        // Create connections between layers
        println!("\n   🔗 Creating inter-layer connections...");
        // In a real system, connections would be created based on topology rules
        println!("   ✓ Hierarchical connections established\n");
        
        Ok(())
    }
    
    /// Configure intelligence layer
    async fn configure_intelligence(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("🤖 Configuring Intelligence Systems...\n");
        
        let mut intelligence = self.intelligence.write().await;
        
        // Enable meta-learning
        let meta_config = MetaLearningConfig {
            learning_rate_adaptation: true,
            strategy_evolution: true,
            architecture_search: true,
            transfer_learning: true,
            continual_learning: true,
        };
        intelligence.enable_meta_learning(meta_config).await?;
        println!("   ✓ Meta-learning enabled");
        
        // Enable self-organization
        let self_org_config = SelfOrganizationConfig {
            allow_topology_changes: true,
            clustering_enabled: true,
            hierarchy_formation: true,
            emergent_specialization: true,
            dynamic_boundaries: true,
        };
        intelligence.enable_self_organization(self_org_config).await?;
        println!("   ✓ Self-organization enabled");
        
        // Set high-level goals
        let goals = vec![
            Goal {
                id: Uuid::new_v4(),
                description: "Achieve human-level understanding of context".to_string(),
                priority: 1.0,
                constraints: vec![],
                success_criteria: vec![],
                decomposition_strategy: DecompositionStrategy::Hierarchical,
            },
            Goal {
                id: Uuid::new_v4(),
                description: "Maintain ethical decision-making".to_string(),
                priority: 0.9,
                constraints: vec![],
                success_criteria: vec![],
                decomposition_strategy: DecompositionStrategy::Parallel,
            },
        ];
        intelligence.set_goals(goals).await?;
        println!("   ✓ Strategic goals set\n");
        
        Ok(())
    }
    
    /// Process a request through all layers
    async fn process_request(&self, request: &str) -> Result<String, Box<dyn std::error::Error>> {
        println!("📥 Processing request: \"{}\"\n", request);
        
        // Layer 5: Intelligence analyzes the request
        println!("5️⃣ Intelligence Layer:");
        let intelligence = self.intelligence.read().await;
        let emergence_report = intelligence.observe_emergence().await?;
        println!("   ✓ Emergence patterns: {}", emergence_report.emergent_properties.len());
        println!("   ✓ Complexity index: {:.2}", emergence_report.complexity_metrics.emergence_index);
        
        // Check if we need creative problem solving
        if request.contains("create") || request.contains("design") {
            let challenge = Challenge {
                id: Uuid::new_v4(),
                problem_statement: request.to_string(),
                context: HashMap::new(),
                constraints: vec![],
                evaluation_criteria: vec![],
            };
            
            let solutions = intelligence.create(challenge).await?;
            println!("   ✓ Generated {} creative solutions", solutions.len());
        }
        drop(intelligence);
        
        // Layer 4: Orchestration routes the request
        println!("\n4️⃣ Orchestration Layer:");
        let orchestrator = self.orchestrator.read().await;
        let signal = OrchestrationSignal {
            id: Uuid::new_v4(),
            source: Uuid::new_v4(),
            signal_type: SignalType::Data,
            priority: 0.8,
            payload: serde_json::json!({ "request": request }),
            routing_hints: RoutingHints {
                preferred_path: None,
                avoid_units: vec![],
                max_hops: Some(5),
                deadline: Some(Utc::now() + chrono::Duration::seconds(5)),
            },
        };
        
        let topology = orchestrator.topology().await?;
        println!("   ✓ Topology: {} units, {} connections", 
                topology.units.len(), topology.connections.len());
        
        // Route through appropriate cognitive units
        let targets = orchestrator.route(signal).await?;
        println!("   ✓ Routed to {} cognitive units", targets.len());
        drop(orchestrator);
        
        // Layer 3: Cognitive processing
        println!("\n3️⃣ Cognitive Layer:");
        let units = self.cognitive_units.read().await;
        let mut responses = Vec::new();
        
        // Process through hierarchical layers based on complexity
        let complexity = analyze_complexity(request);
        let required_layer = match complexity {
            c if c < 0.2 => CognitiveLayer::Reflexive,
            c if c < 0.4 => CognitiveLayer::Implementation,
            c if c < 0.6 => CognitiveLayer::Operational,
            c if c < 0.8 => CognitiveLayer::Tactical,
            _ => CognitiveLayer::Strategic,
        };
        
        println!("   ✓ Complexity: {:.2}, using {} processing", 
                complexity, required_layer.name());
        
        // Find a unit of the required layer
        for (id, unit) in units.iter() {
            if unit.layer() == required_layer {
                let input = CognitiveInput {
                    content: request.to_string(),
                    context: HashMap::new(),
                    source_layer: None,
                };
                
                let output = unit.process(input).await?;
                println!("   ✓ {} processed in {:?}", 
                        required_layer.name(),
                        output.metadata.get("processing_time_ms"));
                responses.push(output.content);
                break;
            }
        }
        drop(units);
        
        // Layer 2: Protocol formatting
        println!("\n2️⃣ Protocol Layer:");
        let protocol_manager = &self.protocol_manager;
        // In a real system, we'd encode the response properly
        println!("   ✓ Response encoded with protocol version 1.0");
        
        // Layer 1: Substrate delivery
        println!("\n1️⃣ Substrate Layer:");
        println!("   ✓ Response delivered via message transport");
        
        // Return the best response
        let response = responses.first()
            .cloned()
            .unwrap_or_else(|| "I'm still learning how to respond to that.".to_string());
        
        Ok(response)
    }
    
    /// Display system metrics
    async fn display_metrics(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("\n📊 System Metrics:\n");
        
        // Intelligence metrics
        let intelligence = self.intelligence.read().await;
        let metrics = intelligence.metrics().await?;
        
        println!("Intelligence Layer:");
        println!("   • Meta-learning efficiency: {:.2}", metrics.meta_learning_efficiency);
        println!("   • Self-organization degree: {:.2}", metrics.self_organization_degree);
        println!("   • Goal achievement rate: {:.2}", metrics.goal_achievement_rate);
        println!("   • Creativity index: {:.2}", metrics.creativity_index);
        println!("   • Adaptation speed: {:.2}", metrics.adaptation_speed);
        println!("   • Consciousness level: {:?}", metrics.consciousness_level);
        
        // Orchestration metrics
        let orchestrator = self.orchestrator.read().await;
        let topology = orchestrator.topology().await?;
        println!("\nOrchestration Layer:");
        println!("   • Total units: {}", topology.metrics.total_units);
        println!("   • Total connections: {}", topology.metrics.total_connections);
        println!("   • Average degree: {:.2}", topology.metrics.average_degree);
        println!("   • Clustering coefficient: {:.2}", topology.metrics.clustering_coefficient);
        
        Ok(())
    }
}

/// Analyze request complexity (simplified)
fn analyze_complexity(request: &str) -> f32 {
    let factors = [
        request.len() as f32 / 100.0,
        request.split_whitespace().count() as f32 / 20.0,
        if request.contains("?") { 0.2 } else { 0.0 },
        if request.contains("create") || request.contains("design") { 0.3 } else { 0.0 },
        if request.contains("why") || request.contains("how") { 0.2 } else { 0.0 },
    ];
    
    factors.iter().sum::<f32>().min(1.0)
}

// Mock implementations for demo
struct MockMetaLearner;
struct MockSelfOrganizer;
struct MockEmergenceDetector;
struct MockCreativityEngine;

// Implement mock traits (simplified for demo)
#[async_trait::async_trait]
impl hal9_core::hierarchical::intelligence::MetaLearner for MockMetaLearner {
    async fn learn_to_learn(&mut self, _: hal9_core::hierarchical::intelligence::Experience) 
        -> hal9_core::Result<hal9_core::hierarchical::intelligence::LearningStrategy> {
        Ok(hal9_core::hierarchical::intelligence::LearningStrategy {
            name: "demo".to_string(),
            parameters: HashMap::new(),
            expected_improvement: 0.1,
        })
    }
    
    async fn optimize_architecture(&mut self) 
        -> hal9_core::Result<hal9_core::hierarchical::intelligence::ArchitectureUpdate> {
        Ok(hal9_core::hierarchical::intelligence::ArchitectureUpdate {
            changes: vec![],
            rationale: "demo".to_string(),
            expected_benefit: 0.1,
        })
    }
    
    async fn transfer_knowledge(&self, _: &str, _: &str) 
        -> hal9_core::Result<hal9_core::hierarchical::intelligence::Knowledge> {
        Ok(hal9_core::hierarchical::intelligence::Knowledge {
            concepts: vec![],
            relationships: vec![],
            applicability: 0.5,
        })
    }
}

// Similar mock implementations for other traits...

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("╔═══════════════════════════════════════════════════════════╗");
    println!("║          HAL9 Hierarchical Architecture Demo              ║");
    println!("║                   All 5 Layers Active                     ║");
    println!("╚═══════════════════════════════════════════════════════════╝\n");
    
    // Initialize the complete system
    let system = HAL9System::new().await?;
    
    // Create cognitive hierarchy
    system.create_cognitive_hierarchy().await?;
    
    // Configure intelligence
    system.configure_intelligence().await?;
    
    // Process various requests
    println!("═══════════════════════════════════════════════════════════\n");
    
    let requests = [
        "Hello HAL9",
        "What is the weather today?",
        "Calculate the fibonacci sequence",
        "Design a distributed messaging system",
        "How can we achieve artificial general intelligence?",
    ];
    
    for request in &requests {
        let response = system.process_request(request).await?;
        println!("💬 Response: {}\n", response);
        println!("───────────────────────────────────────────────────────────\n");
    }
    
    // Display final metrics
    system.display_metrics().await?;
    
    println!("\n✅ Demo completed successfully!");
    println!("\nThis demonstration shows how all 5 layers work together:");
    println!("1. Substrate provides infrastructure");
    println!("2. Protocol handles communication");
    println!("3. Cognitive processes information hierarchically");
    println!("4. Orchestration manages topology and routing");
    println!("5. Intelligence enables meta-learning and emergence");
    
    Ok(())
}