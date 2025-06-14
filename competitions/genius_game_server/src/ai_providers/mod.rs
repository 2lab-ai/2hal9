pub mod ollama;
pub mod bedrock;
pub mod provider;

pub use provider::{AIProvider, AIProviderConfig, AIResponse, AIModel};
pub use ollama::OllamaProvider;
pub use bedrock::BedrockProvider;

use async_trait::async_trait;
use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Standard AI decision for game moves
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameDecision {
    pub choice: String,
    pub reasoning: Option<String>,
    pub confidence: f32,
    pub thinking_time_ms: u64,
}

/// Factory for creating AI providers
pub struct AIProviderFactory;

impl AIProviderFactory {
    pub fn create(config: AIProviderConfig) -> Result<Box<dyn AIProvider>> {
        match config {
            AIProviderConfig::Ollama { model, endpoint } => {
                Ok(Box::new(OllamaProvider::new(model, endpoint)?))
            }
            AIProviderConfig::Bedrock { model, region } => {
                Ok(Box::new(BedrockProvider::new(model, region)?))
            }
            AIProviderConfig::Mock { name } => {
                Ok(Box::new(MockProvider { name }))
            }
        }
    }
}

/// Mock provider for testing
pub struct MockProvider {
    name: String,
}

#[async_trait]
impl AIProvider for MockProvider {
    async fn make_decision(&self, _game_type: &str, game_state: serde_json::Value) -> Result<GameDecision> {
        // Simple mock logic
        let empty_vec = vec![];
        let choices_array = game_state["available_choices"]
            .as_array()
            .unwrap_or(&empty_vec);
        
        let choices: Vec<_> = choices_array
            .iter()
            .filter_map(|v| v.as_str())
            .collect();
        
        let choice = if choices.is_empty() {
            "default".to_string()
        } else {
            choices[rand::random::<usize>() % choices.len()].to_string()
        };
        
        Ok(GameDecision {
            choice,
            reasoning: Some(format!("{} mock decision", self.name)),
            confidence: 0.7,
            thinking_time_ms: 10,
        })
    }
    
    fn get_model_info(&self) -> AIModel {
        AIModel {
            provider: "mock".to_string(),
            name: self.name.clone(),
            max_tokens: 1000,
            supports_streaming: false,
        }
    }
}