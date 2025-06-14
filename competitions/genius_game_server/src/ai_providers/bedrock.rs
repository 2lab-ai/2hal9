use async_trait::async_trait;
use anyhow::{Result, anyhow};
use aws_sdk_bedrockruntime::types::{ContentBlock, Message, ConversationRole};
use aws_sdk_bedrockruntime::Client;
use serde::{Deserialize, Serialize};
use super::{AIProvider, AIModel, GameDecision};
use std::time::Instant;

/// AWS Bedrock provider for Claude and other models
pub struct BedrockProvider {
    model: String,
    client: Client,
}

impl BedrockProvider {
    pub fn new(_model: String, _region: String) -> Result<Self> {
        // For now, create a mock client - in production, use proper AWS config
        // This would normally use aws_config::from_env()
        Err(anyhow!("Bedrock provider requires AWS credentials setup"))
    }
    
    pub async fn new_with_config(model: String) -> Result<Self> {
        let config = aws_config::load_from_env().await;
        let client = Client::new(&config);
        
        Ok(Self {
            model,
            client,
        })
    }
}

#[async_trait]
impl AIProvider for BedrockProvider {
    async fn make_decision(&self, game_type: &str, game_state: serde_json::Value) -> Result<GameDecision> {
        let start = Instant::now();
        
        let prompt = format!(
            r#"You are playing a {} game. Current state:
{}

Available choices: {}

Respond with ONLY a JSON object: {{"choice": "your_choice", "reasoning": "brief explanation", "confidence": 0.0-1.0}}"#,
            game_type,
            serde_json::to_string_pretty(&game_state)?,
            game_state["available_choices"].as_array()
                .unwrap_or(&vec![])
                .iter()
                .filter_map(|v| v.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        );
        
        let message = Message::builder()
            .role(ConversationRole::User)
            .content(ContentBlock::Text(prompt))
            .build()
            .map_err(|e| anyhow!("Failed to build message: {}", e))?;
        
        let request = self.client
            .converse()
            .model_id(&self.model)
            .messages(message);
        
        let response = request.send().await
            .map_err(|e| anyhow!("Bedrock API error: {}", e))?;
        
        let output = response.output()
            .ok_or_else(|| anyhow!("No output from Bedrock"))?;
        
        let message = match output {
            aws_sdk_bedrockruntime::types::ConverseOutput::Message(msg) => msg,
            _ => return Err(anyhow!("Invalid output format")),
        };
        
        let content = message.content()
            .first()
            .ok_or_else(|| anyhow!("No content in response"))?;
        
        let text = match content {
            aws_sdk_bedrockruntime::types::ContentBlock::Text(t) => t.as_str(),
            _ => return Err(anyhow!("Content is not text")),
        };
        
        // Parse the response
        let decision: DecisionResponse = serde_json::from_str(text)
            .unwrap_or_else(|_| DecisionResponse {
                choice: game_state["available_choices"][0].as_str().unwrap_or("default").to_string(),
                reasoning: Some("Failed to parse response".to_string()),
                confidence: 0.5,
            });
        
        Ok(GameDecision {
            choice: decision.choice,
            reasoning: decision.reasoning,
            confidence: decision.confidence,
            thinking_time_ms: start.elapsed().as_millis() as u64,
        })
    }
    
    fn get_model_info(&self) -> AIModel {
        AIModel {
            provider: "bedrock".to_string(),
            name: self.model.clone(),
            max_tokens: 100000,
            supports_streaming: true,
        }
    }
}

#[derive(Debug, Deserialize)]
struct DecisionResponse {
    choice: String,
    reasoning: Option<String>,
    confidence: f32,
}