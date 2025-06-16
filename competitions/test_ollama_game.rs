use genius_game_server::ai_providers::{AIProviderConfig, AIProviderFactory, GameDecision};
use serde_json::json;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("🎮 Testing Ollama with Genius Games");
    println!("===================================\n");
    
    // Create Ollama provider
    let config = AIProviderConfig::Ollama {
        model: "llama3:latest".to_string(),
        endpoint: "http://localhost:11434".to_string(),
    };
    
    let provider = AIProviderFactory::create(config)?;
    
    // Test 1: Prisoner's Dilemma
    println!("🎯 Test 1: Prisoner's Dilemma");
    let game_state = json!({
        "game_type": "prisoners_dilemma",
        "round": 1,
        "my_score": 0,
        "opponent_score": 0,
        "history": [],
        "available_choices": ["cooperate", "defect"]
    });
    
    let decision = provider.make_decision("prisoners_dilemma", game_state).await?;
    println!("Decision: {}", decision.choice);
    println!("Reasoning: {:?}", decision.reasoning);
    println!("Confidence: {:.2}", decision.confidence);
    println!("Thinking time: {}ms\n", decision.thinking_time_ms);
    
    // Test 2: Minority Game
    println!("🎯 Test 2: Minority Game");
    let game_state = json!({
        "game_type": "minority_game",
        "round": 1,
        "players": 5,
        "my_previous_choices": [],
        "available_choices": ["red", "blue"]
    });
    
    let decision = provider.make_decision("minority_game", game_state).await?;
    println!("Decision: {}", decision.choice);
    println!("Reasoning: {:?}", decision.reasoning);
    println!("Confidence: {:.2}", decision.confidence);
    println!("Thinking time: {}ms\n", decision.thinking_time_ms);
    
    println!("✅ All tests passed!");
    Ok(())
}