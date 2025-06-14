//! Simple demo to verify agent dropout functionality

use agent_dropout::{
    AgentLevel, AgentProfile, AssessmentPool, DropoutController, 
    EvaluationEngine, NetworkTopology,
};
use std::time::Duration;
use uuid::Uuid;

#[tokio::test]
async fn test_agent_dropout_demo() {
    println!("\n=== Agent Dropout Demo ===\n");
    
    // 1. Create network components
    let topology = NetworkTopology::new();
    let assessment_pool = AssessmentPool::new();
    let _evaluation_engine = EvaluationEngine::new();
    let dropout_controller = DropoutController::new(
        1024 * 1024 * 10, // 10MB memory limit
        Duration::from_secs(300), // 5 minute idle timeout
        0.3, // 30% quality threshold
    );
    
    // 2. Add agents to the network
    println!("Adding agents to network...");
    let mut agents = Vec::new();
    
    for i in 0..5 {
        let level = match i {
            0 => AgentLevel::L3,
            1 => AgentLevel::L5,
            2 => AgentLevel::L8,
            3 => AgentLevel::L10,
            4 => AgentLevel::L15,
            _ => AgentLevel::L5,
        };
        
        let agent = AgentProfile::new(Uuid::new_v4(), level);
        println!("  - Agent {} (Level {})", i + 1, level.value());
        
        let position = topology.place_agent(&agent).await;
        println!("    Placed in {:?} layer with {} initial connections", 
                 position.layer, position.initial_connections.len());
        
        agents.push(agent);
    }
    
    // 3. Get network statistics
    let stats = topology.get_network_stats().await;
    println!("\nNetwork Statistics:");
    println!("  Total agents: {}", stats.total_agents);
    println!("  Total connections: {}", stats.total_connections);
    println!("  Average connectivity: {:.2}", stats.average_connectivity);
    
    // 4. Test assessment questions
    println!("\nTesting assessment questions...");
    for level in [AgentLevel::L5, AgentLevel::L10, AgentLevel::L15] {
        if let Some(question) = assessment_pool.get_question_for_level(level) {
            println!("  Level {}: {}", level.value(), &question.content[..50.min(question.content.len())]);
        }
    }
    
    // 5. Test dropout decision
    println!("\nTesting dropout decisions...");
    for (i, agent) in agents.iter().enumerate() {
        let quality_score = 0.2 + (i as f32 * 0.2); // Varying quality scores
        let should_drop = dropout_controller.should_dropout(agent, quality_score).await;
        println!("  Agent {} (L{}, quality {:.2}): {}",
                 i + 1, agent.capability_level.value(), quality_score,
                 if should_drop { "DROP" } else { "KEEP" });
    }
    
    // 6. Test connections
    println!("\nTesting agent connections...");
    if agents.len() >= 2 {
        let connected = topology.are_connected(agents[0].id, agents[1].id).await;
        println!("  Agent 1 <-> Agent 2: {}", if connected { "Connected" } else { "Not connected" });
    }
    
    println!("\n=== Demo Complete ===");
}