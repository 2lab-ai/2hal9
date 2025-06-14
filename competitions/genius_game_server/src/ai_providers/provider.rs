use async_trait::async_trait;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use super::GameDecision;

/// AI provider trait for different model backends
#[async_trait]
pub trait AIProvider: Send + Sync {
    /// Make a game decision based on the current state
    async fn make_decision(&self, game_type: &str, game_state: serde_json::Value) -> Result<GameDecision>;
    
    /// Get model information
    fn get_model_info(&self) -> AIModel;
}

/// Configuration for different AI providers
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AIProviderConfig {
    Ollama {
        model: String,
        endpoint: String,
    },
    Bedrock {
        model: String,
        region: String,
    },
    Mock {
        name: String,
    },
}

/// AI model information
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AIModel {
    pub provider: String,
    pub name: String,
    pub max_tokens: usize,
    pub supports_streaming: bool,
}

/// Standard AI response format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIResponse {
    pub text: String,
    pub model: String,
    pub usage: Option<TokenUsage>,
    pub latency_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    pub prompt_tokens: usize,
    pub completion_tokens: usize,
    pub total_tokens: usize,
}