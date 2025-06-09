//! Example demonstrating the hierarchical architecture in action

use hal9_core::hierarchical::{
    substrate::{
        ChannelTransport, AsyncRuntime, TokioRuntime,
        SqliteStorage, LocalResources,
    },
    protocol::{
        SignalProtocol, GradientProtocol, ConsensusProtocol,
        ProtocolManager, ProtocolManagerConfig,
    },
    cognitive::{
        L1ReflexiveNeuron, L2ImplementationNeuron, L3OperationalNeuron,
        L4TacticalNeuron, L5StrategicNeuron,
        CognitiveUnit, CognitiveInput, CognitiveLayer,
        CognitiveUnitBuilder, DefaultCognitiveFactory,
        Pattern,
    },
};
use std::sync::Arc;
use uuid::Uuid;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    println!("=== HAL9 Hierarchical Architecture Demo ===\n");
    
    // 1. Create Substrate Layer
    println!("1. Initializing Substrate Layer...");
    let runtime = Arc::new(TokioRuntime::new()?);
    let transport = Arc::new(ChannelTransport::new());
    let storage = Arc::new(SqliteStorage::new("demo.db").await?);
    let resources = Arc::new(LocalResources::new());
    
    println!("   ✓ Runtime, Transport, Storage, Resources ready\n");
    
    // 2. Initialize Protocol Layer
    println!("2. Setting up Protocol Layer...");
    let protocol_manager = Arc::new(ProtocolManager::new(
        ProtocolManagerConfig::default(),
        transport.clone(),
    ));
    protocol_manager.initialize_protocols().await?;
    
    println!("   ✓ Signal, Gradient, Consensus protocols ready\n");
    
    // 3. Create Cognitive Layer Neurons
    println!("3. Creating Cognitive Layer Neurons...");
    
    // Create neurons for each layer
    let factory = DefaultCognitiveFactory::new()
        .with_protocol_manager(protocol_manager.clone());
    
    // L1 - Reflexive
    let l1_config = CognitiveUnitBuilder::new(CognitiveLayer::Reflexive)
        .with_parameter("response_threshold", 0.7)
        .build();
    let mut l1 = L1ReflexiveNeuron::new(l1_config);
    
    // Add some patterns
    l1.add_pattern(Pattern {
        trigger: "hello".to_string(),
        response: "Hi! I'm HAL9 with hierarchical architecture!".to_string(),
        confidence: 0.95,
    });
    l1.add_pattern(Pattern {
        trigger: "status".to_string(),
        response: "All systems operational. Hierarchical processing active.".to_string(),
        confidence: 0.9,
    });
    
    // L2 - Implementation
    let l2_config = CognitiveUnitBuilder::new(CognitiveLayer::Implementation)
        .build();
    let mut l2 = L2ImplementationNeuron::new(l2_config);
    
    // L3 - Operational
    let l3_config = CognitiveUnitBuilder::new(CognitiveLayer::Operational)
        .build();
    let mut l3 = L3OperationalNeuron::new(l3_config);
    
    // L4 - Tactical
    let l4_config = CognitiveUnitBuilder::new(CognitiveLayer::Tactical)
        .build();
    let mut l4 = L4TacticalNeuron::new(l4_config);
    
    // L5 - Strategic
    let l5_config = CognitiveUnitBuilder::new(CognitiveLayer::Strategic)
        .build();
    let mut l5 = L5StrategicNeuron::new(l5_config);
    
    println!("   ✓ Created L1-L5 neurons\n");
    
    // 4. Demonstrate Hierarchical Processing
    println!("4. Demonstrating Hierarchical Processing...\n");
    
    // Example 1: Reflexive Response (L1)
    println!("Example 1: Reflexive Response (L1)");
    let input = CognitiveInput {
        content: "hello world".to_string(),
        context: HashMap::new(),
        source_layer: None,
    };
    let output = l1.process(input).await?;
    println!("   Input: 'hello world'");
    println!("   L1 Response: {}", output.content);
    println!("   Processing time: {:?}\n", output.metadata.get("processing_time_ms"));
    
    // Example 2: Implementation Task (L2)
    println!("Example 2: Implementation Task (L2)");
    let input = CognitiveInput {
        content: "Create a function to calculate fibonacci numbers".to_string(),
        context: HashMap::new(),
        source_layer: Some(CognitiveLayer::Operational),
    };
    let output = l2.process(input).await?;
    println!("   Task: Create fibonacci function");
    println!("   L2 Generated Code:");
    println!("{}", output.content.lines()
        .map(|line| format!("   {}", line))
        .collect::<Vec<_>>()
        .join("\n"));
    println!();
    
    // Example 3: System Design (L3)
    println!("Example 3: System Design (L3)");
    let input = CognitiveInput {
        content: "Design a distributed chat application".to_string(),
        context: HashMap::new(),
        source_layer: Some(CognitiveLayer::Tactical),
    };
    let output = l3.process(input).await?;
    println!("   Request: Design distributed chat");
    println!("   L3 Design Output:");
    println!("{}", output.content.lines()
        .take(10)
        .map(|line| format!("   {}", line))
        .collect::<Vec<_>>()
        .join("\n"));
    println!("   ... (truncated)\n");
    
    // Example 4: Strategic Planning (L4)
    println!("Example 4: Strategic Planning (L4)");
    let input = CognitiveInput {
        content: "Create a plan to scale the system to 1000 concurrent users".to_string(),
        context: HashMap::new(),
        source_layer: Some(CognitiveLayer::Strategic),
    };
    let output = l4.process(input).await?;
    println!("   Objective: Scale to 1000 users");
    println!("   L4 Tactical Plan:");
    println!("{}", output.content.lines()
        .take(8)
        .map(|line| format!("   {}", line))
        .collect::<Vec<_>>()
        .join("\n"));
    println!("   ... (truncated)\n");
    
    // Example 5: Vision Setting (L5)
    println!("Example 5: Vision Setting (L5)");
    let input = CognitiveInput {
        content: "Set vision for becoming the leading AI assistant platform".to_string(),
        context: HashMap::new(),
        source_layer: None,
    };
    let output = l5.process(input).await?;
    println!("   Request: Set strategic vision");
    println!("   L5 Strategic Output:");
    println!("{}", output.content.lines()
        .take(6)
        .map(|line| format!("   {}", line))
        .collect::<Vec<_>>()
        .join("\n"));
    println!("   ... (truncated)\n");
    
    // 5. Demonstrate Layer Interaction
    println!("5. Demonstrating Layer Interaction...\n");
    
    // Strategic directive flows down through layers
    println!("   Strategic Goal → Tactical Plan → Operational Design → Implementation");
    
    let strategic_input = CognitiveInput {
        content: "Achieve 99.9% uptime through robust architecture".to_string(),
        context: HashMap::new(),
        source_layer: None,
    };
    
    // L5 creates strategic goal
    let strategic_output = l5.process(strategic_input).await?;
    println!("   L5 Strategic: {}", strategic_output.content.lines().next().unwrap_or(""));
    
    // L4 creates tactical plan from strategic goal
    let tactical_input = CognitiveInput {
        content: format!("Plan: {}", strategic_output.content.lines().next().unwrap_or("")),
        context: HashMap::new(),
        source_layer: Some(CognitiveLayer::Strategic),
    };
    let tactical_output = l4.process(tactical_input).await?;
    println!("   L4 Tactical: Created plan with {} steps", 
           tactical_output.metadata.get("total_steps").unwrap_or(&serde_json::json!(0)));
    
    // L3 creates operational design
    let operational_input = CognitiveInput {
        content: "Design high-availability architecture with redundancy".to_string(),
        context: HashMap::new(),
        source_layer: Some(CognitiveLayer::Tactical),
    };
    let operational_output = l3.process(operational_input).await?;
    println!("   L3 Operational: Designed system with {} tasks",
           operational_output.metadata.get("tasks_queued").unwrap_or(&serde_json::json!(0)));
    
    // L2 implements specific components
    let impl_input = CognitiveInput {
        content: "Implement health check endpoint for monitoring".to_string(),
        context: HashMap::new(),
        source_layer: Some(CognitiveLayer::Operational),
    };
    let impl_output = l2.process(impl_input).await?;
    println!("   L2 Implementation: Generated code for health check endpoint");
    
    // L1 provides immediate response
    let user_input = CognitiveInput {
        content: "status check".to_string(),
        context: HashMap::new(),
        source_layer: None,
    };
    let reflexive_output = l1.process(user_input).await?;
    println!("   L1 Reflexive: {}", reflexive_output.content);
    
    println!("\n=== Demo Complete ===");
    println!("\nThe hierarchical architecture enables:");
    println!("- Different abstraction levels for different types of tasks");
    println!("- Appropriate time horizons for each layer");
    println!("- Clean separation of concerns");
    println!("- Scalable and maintainable AI system");
    
    Ok(())
}

// Example helper to show layer characteristics
fn print_layer_characteristics() {
    use hal9_core::hierarchical::cognitive::CognitiveLayer;
    
    println!("\nLayer Characteristics:");
    for layer in [
        CognitiveLayer::Reflexive,
        CognitiveLayer::Implementation,
        CognitiveLayer::Operational,
        CognitiveLayer::Tactical,
        CognitiveLayer::Strategic,
    ] {
        let chars = layer.characteristics();
        println!("  {}: abstraction={:.1}, horizon={:?}, learning_rate={:.3}",
                layer.name(),
                chars.abstraction_level,
                chars.time_horizon,
                chars.learning_rate);
    }
}