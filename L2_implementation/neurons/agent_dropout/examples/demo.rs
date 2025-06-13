//! HAL9 Agent Self-Organization & Dropout Demo

use agent_dropout::*;
use chrono::Utc;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use uuid::Uuid;

/// Demo simulation of agent network
#[tokio::main]
async fn main() {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
    
    println!("\n🚀 HAL9 Agent Self-Organization & Dropout Demo\n");
    println!("This demo simulates a network of AI agents that:");
    println!("1. Enter the network and introduce themselves");
    println!("2. Evaluate each other's capabilities");
    println!("3. Self-organize into hierarchical layers");
    println!("4. Drop underperforming agents automatically\n");
    
    // Create core components
    let evaluation_engine = Arc::new(EvaluationEngine::new());
    let network_topology = Arc::new(NetworkTopology::new());
    let assessment_pool = Arc::new(AssessmentPool::new());
    
    // Configure dropout manager
    let dropout_config = DropoutConfig {
        dropout_threshold: 0.2, // Drop bottom 20%
        evaluation_interval: Duration::from_secs(10), // Every 10 seconds for demo
        minimum_agent_count: 5,
        grace_period: Duration::from_secs(5),
    };
    let dropout_orchestrator = Arc::new(DropoutOrchestrator::new(dropout_config));
    
    // Phase 1: Initial agent creation and entry
    println!("📊 Phase 1: Creating initial agent population...\n");
    
    let mut agents = Vec::new();
    for i in 0..10 {
        let level = ((i % 4) * 5 + 3) as u8; // Levels: 3, 8, 13, 18, repeating
        let agent = create_demo_agent(level, i).await;
        agents.push(agent);
    }
    
    // Register agents and simulate entry protocol
    for (idx, agent) in agents.iter().enumerate() {
        println!("🤖 Agent {} entering network...", idx + 1);
        println!("   ID: {}", agent.id);
        println!("   Self-declared level: L{}", agent.profile.capability_level.value());
        println!("   Specializations: {:?}", agent.profile.specialization);
        
        // Register with systems
        evaluation_engine.register_agent(agent.id, agent.profile.capability_level);
        let position = network_topology.place_agent(&agent.profile).await;
        dropout_orchestrator.register_agent(agent.clone()).await.unwrap();
        
        println!("   Placed in layer: {:?}", position.layer);
        println!("   Initial connections: {}", position.initial_connections.len());
        println!("   Position quality: {:.2}\n", position.position_quality);
        
        sleep(Duration::from_millis(500)).await;
    }
    
    // Phase 2: Assessment and mutual evaluation
    println!("\n📝 Phase 2: Agents assessing each other...\n");
    
    for agent in &agents[..3] { // Demo with first 3 agents
        // Get assessment question
        let question = assessment_pool
            .get_question_for_level(agent.profile.capability_level)
            .unwrap();
            
        println!("❓ Question for Agent L{}:", agent.profile.capability_level.value());
        println!("   Category: {:?}", question.category);
        println!("   Content: {}", truncate_string(&question.content, 100));
        
        // Simulate agent response
        let response = simulate_agent_response(agent, question).await;
        println!("   Answer confidence: {:.2}", response.confidence);
        
        // Get evaluations from other agents
        let evaluations = evaluation_engine
            .broadcast_evaluation_request(agent.id, &response)
            .await;
            
        println!("   Received {} evaluations", evaluations.len());
        
        // Aggregate to estimate level
        let estimate = evaluation_engine.aggregate_evaluations(evaluations).await;
        println!("   Estimated level: L{} (confidence: {:.2})", 
                 estimate.estimated_level.value(), 
                 estimate.confidence);
        println!("   Consensus score: {:.2}\n", estimate.consensus_score);
        
        sleep(Duration::from_millis(500)).await;
    }
    
    // Phase 3: Network activity and performance tracking
    println!("\n⚡ Phase 3: Simulating network activity...\n");
    
    // Simulate some agent interactions
    for _ in 0..20 {
        let agent1 = &agents[rand::random::<usize>() % agents.len()];
        let agent2 = &agents[rand::random::<usize>() % agents.len()];
        
        if agent1.id != agent2.id {
            let success = rand::random::<f32>() > 0.3; // 70% success rate
            network_topology.update_connection(agent1.id, agent2.id, success).await;
        }
    }
    
    // Show network statistics
    let stats = dropout_orchestrator.network_stats().await;
    println!("📊 Network Statistics:");
    println!("   Total agents: {}", stats.total_agents);
    println!("   Average level: {:.1}", stats.average_level);
    println!("   Average performance: {:.2}", stats.average_performance);
    println!("   Dropout rate: {:.0}%\n", stats.dropout_rate * 100.0);
    
    // Phase 4: Dropout cycle
    println!("\n🔄 Phase 4: Running dropout evaluation cycle...\n");
    
    // Wait for grace period to expire
    println!("⏳ Waiting for grace period to expire...");
    sleep(Duration::from_secs(6)).await;
    
    // Run health check
    let report = dropout_orchestrator.health_check_cycle().await.unwrap();
    
    println!("\n📋 Dropout Report:");
    println!("   Evaluated agents: {}", report.evaluated_agents);
    println!("   Dropped agents: {}", report.dropped_agents.len());
    println!("   Replaced agents: {}", report.replaced_agents.len());
    println!("   Warnings issued: {}", report.warnings_issued);
    
    if !report.dropped_agents.is_empty() {
        println!("\n❌ Dropped agents:");
        for id in &report.dropped_agents {
            println!("   - {}", id);
        }
    }
    
    if !report.replaced_agents.is_empty() {
        println!("\n✅ New agents joined:");
        for id in &report.replaced_agents {
            println!("   - {}", id);
        }
    }
    
    // Phase 5: Show final network topology
    println!("\n\n🌐 Phase 5: Final network visualization...\n");
    
    let viz = network_topology.export_topology().await;
    println!("Network topology:");
    println!("   Nodes: {}", viz.nodes.len());
    println!("   Edges: {}", viz.edges.len());
    
    // Show layer statistics
    for layer in [NetworkLayer::Basic, NetworkLayer::Intermediate, NetworkLayer::Advanced, NetworkLayer::Expert] {
        let layer_stats = network_topology.layer_statistics(layer).await;
        if layer_stats.agent_count > 0 {
            println!("\n   {:?} Layer:", layer);
            println!("     Agents: {}", layer_stats.agent_count);
            println!("     Avg Level: {:.1}", layer_stats.average_level);
            println!("     Connections: {}", layer_stats.total_connections);
            println!("     Connectivity: {:.2}", layer_stats.connectivity_ratio);
        }
    }
    
    // Evolutionary optimization suggestion
    println!("\n\n🧬 Evolutionary Optimization Analysis:\n");
    
    let mut optimizer = EvolutionaryOptimizer::new(0.2, 0.1);
    let suggestion = optimizer.optimize_network(&stats);
    
    if let Some(new_rate) = suggestion.adjust_dropout_rate {
        println!("   📊 Suggested dropout rate adjustment: {:.0}%", new_rate * 100.0);
    }
    
    if suggestion.introduce_random_agent {
        println!("   🎲 Recommendation: Introduce random agent for diversity");
    }
    
    if !suggestion.specialization_needed.is_empty() {
        println!("   🎯 Specializations needed: {:?}", suggestion.specialization_needed);
    }
    
    println!("\n\n✨ Demo completed! The network has self-organized and optimized itself.\n");
}

/// Create a demo agent with simulated capabilities
async fn create_demo_agent(level: u8, index: usize) -> AgentNeuron {
    let specializations = match level {
        1..=5 => vec!["basic_reasoning".to_string(), "pattern_matching".to_string()],
        6..=10 => vec!["logical_analysis".to_string(), "creative_problem_solving".to_string()],
        11..=15 => vec!["systems_thinking".to_string(), "meta_cognition".to_string()],
        16..=20 => vec!["consciousness_modeling".to_string(), "ethical_reasoning".to_string()],
        _ => vec!["general".to_string()],
    };
    
    let mut agent = AgentNeuron::new(
        AgentLevel::new(level).unwrap(),
        specializations,
    );
    
    // Simulate some performance history
    let performance = 0.3 + (level as f32 / 20.0) * 0.5 + (rand::random::<f32>() * 0.2);
    for _ in 0..5 {
        agent.performance_tracker.record_task(
            rand::random::<f32>() < performance,
            Duration::from_millis(100 + rand::random::<u64>() % 900),
        );
    }
    
    agent
}

/// Simulate an agent's response to a question
async fn simulate_agent_response(agent: &AgentNeuron, question: &AssessmentQuestion) -> AssessmentResponse {
    // Simulate thinking time based on question difficulty and agent level
    let level_diff = agent.profile.capability_level.value() as i32 - question.difficulty.value() as i32;
    let base_time = 1000 + (question.difficulty.value() as u64 * 100);
    let thinking_time = if level_diff > 0 {
        base_time / 2
    } else {
        base_time * 2
    };
    
    // Simulate confidence based on level difference
    let confidence = match level_diff {
        d if d >= 5 => 0.9 + rand::random::<f32>() * 0.1,
        d if d >= 0 => 0.7 + rand::random::<f32>() * 0.2,
        d if d >= -5 => 0.4 + rand::random::<f32>() * 0.3,
        _ => 0.2 + rand::random::<f32>() * 0.3,
    };
    
    AssessmentResponse {
        agent_id: agent.id,
        question_id: question.id,
        answer: format!("Simulated answer from L{} agent", agent.profile.capability_level.value()),
        confidence,
        reasoning: Some("Simulated reasoning process...".to_string()),
        time_taken: Duration::from_millis(thinking_time),
        timestamp: Utc::now(),
    }
}

/// Truncate a string for display
fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len-3])
    }
}