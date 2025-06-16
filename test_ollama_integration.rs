use genius_ai::providers::ollama::OllamaProvider;
use genius_ai::AIProvider;
use genius_core::{GameContext, PlayerAction};
use serde_json::json;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("🦙 Testing Ollama Provider...");
    
    // Create Ollama provider
    let provider = OllamaProvider::new("http://localhost:11434", "llama2");
    
    // Create game context
    let context = GameContext {
        game_type: "PrisonersDilemma".to_string(),
        round: 5,
        history: vec![
            json!({"action": "cooperate", "opponent": "defect"}),
            json!({"action": "defect", "opponent": "cooperate"}),
        ],
        game_state: json!({
            "my_score": 10,
            "opponent_score": 12
        }),
        valid_actions: vec!["cooperate".to_string(), "defect".to_string()],
        time_remaining_ms: 5000,
    };
    
    // Get AI decision
    println!("🤔 Asking AI for decision...");
    match provider.get_action(&context).await {
        Ok(action) => {
            println!("✅ AI Decision: {}", action.action_type);
            if let Some(reasoning) = action.reasoning {
                println!("💭 Reasoning: {}", reasoning);
            }
        }
        Err(e) => {
            println!("❌ Error: {}", e);
        }
    }
    
    Ok(())
}
