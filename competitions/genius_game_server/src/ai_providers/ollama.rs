use async_trait::async_trait;
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use super::{AIProvider, AIModel, GameDecision};
use std::time::Instant;

/// Ollama provider for local models
pub struct OllamaProvider {
    model: String,
    endpoint: String,
    client: reqwest::Client,
}

impl OllamaProvider {
    pub fn new(model: String, endpoint: String) -> Result<Self> {
        Ok(Self {
            model,
            endpoint,
            client: reqwest::Client::new(),
        })
    }
    
    async fn call_ollama(&self, prompt: String) -> Result<OllamaResponse> {
        let request = OllamaRequest {
            model: self.model.clone(),
            prompt,
            stream: false,
            options: Some(OllamaOptions {
                temperature: 0.7,
                top_p: 0.9,
                num_predict: 200,
            }),
        };
        
        let response = self.client
            .post(&format!("{}/api/generate", self.endpoint))
            .json(&request)
            .send()
            .await?;
        
        if !response.status().is_success() {
            return Err(anyhow!("Ollama API error: {}", response.status()));
        }
        
        let ollama_response: OllamaResponse = response.json().await?;
        Ok(ollama_response)
    }
}

#[async_trait]
impl AIProvider for OllamaProvider {
    async fn make_decision(&self, game_type: &str, game_state: serde_json::Value) -> Result<GameDecision> {
        let start = Instant::now();
        
        // Create a focused prompt for the game
        let prompt = format!(
            r#"You are playing a {} game. Current state:
{}

Available choices: {}

Respond with ONLY a JSON object in this exact format:
{{"choice": "your_choice", "reasoning": "brief explanation", "confidence": 0.0-1.0}}

Your response:"#,
            game_type,
            serde_json::to_string_pretty(&game_state)?,
            game_state["available_choices"].as_array()
                .unwrap_or(&vec![])
                .iter()
                .filter_map(|v| v.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        );
        
        let response = self.call_ollama(prompt).await?;
        
        // Try to parse the response as JSON
        let decision: DecisionResponse = match serde_json::from_str(&response.response) {
            Ok(d) => d,
            Err(_) => {
                // Fallback: try to extract from the text
                let empty_vec = vec![];
                let choices_array = game_state["available_choices"]
                    .as_array()
                    .unwrap_or(&empty_vec);
                
                let choices: Vec<_> = choices_array
                    .iter()
                    .filter_map(|v| v.as_str())
                    .collect();
                
                let choice = choices.first().unwrap_or(&"default").to_string();
                
                DecisionResponse {
                    choice,
                    reasoning: Some("Failed to parse response".to_string()),
                    confidence: 0.5,
                }
            }
        };
        
        Ok(GameDecision {
            choice: decision.choice,
            reasoning: decision.reasoning,
            confidence: decision.confidence,
            thinking_time_ms: start.elapsed().as_millis() as u64,
        })
    }
    
    fn get_model_info(&self) -> AIModel {
        AIModel {
            provider: "ollama".to_string(),
            name: self.model.clone(),
            max_tokens: 2048,
            supports_streaming: true,
        }
    }
}

#[derive(Debug, Serialize)]
struct OllamaRequest {
    model: String,
    prompt: String,
    stream: bool,
    options: Option<OllamaOptions>,
}

#[derive(Debug, Serialize)]
struct OllamaOptions {
    temperature: f32,
    top_p: f32,
    num_predict: usize,
}

#[derive(Debug, Deserialize)]
struct OllamaResponse {
    response: String,
    #[allow(dead_code)]
    done: bool,
    #[allow(dead_code)]
    context: Option<Vec<u32>>,
    #[allow(dead_code)]
    total_duration: Option<u64>,
    #[allow(dead_code)]
    prompt_eval_count: Option<usize>,
    #[allow(dead_code)]
    eval_count: Option<usize>,
}

#[derive(Debug, Deserialize)]
struct DecisionResponse {
    choice: String,
    reasoning: Option<String>,
    confidence: f32,
}