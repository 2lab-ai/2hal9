//! Claude integration abstractions

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info};
use 2hal9_core::{Result, Error, Layer};

/// Claude interface abstraction
#[async_trait]
pub trait ClaudeInterface: Send + Sync {
    /// Send a message and get response
    async fn send_message(&self, message: &str) -> Result<String>;
    
    /// Get the system prompt for this instance
    fn system_prompt(&self) -> &str;
    
    /// Get token usage for the last request
    fn last_token_usage(&self) -> Option<TokenUsage>;
}

/// Token usage tracking
#[derive(Debug, Clone, Default)]
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// Mock Claude implementation for testing
pub struct MockClaude {
    layer: String,
    system_prompt: String,
    responses: HashMap<String, String>,
    delay_ms: u64,
}

impl MockClaude {
    /// Create a new mock Claude for a specific layer
    pub fn new(layer: &str) -> Self {
        let mut responses = HashMap::new();
        
        // Add layer-specific mock responses
        match layer {
            "L4" => {
                responses.insert(
                    "default".to_string(),
                    "FORWARD_TO: neuron-2, neuron-3\nCONTENT: Breaking down into two strategic initiatives:\n1. Design the system architecture\n2. Plan the implementation approach".to_string()
                );
            }
            "L3" => {
                responses.insert(
                    "default".to_string(),
                    "FORWARD_TO: neuron-4, neuron-5\nCONTENT: Design specification:\n- Component A: Handle data processing\n- Component B: Manage user interface".to_string()
                );
            }
            "L2" => {
                responses.insert(
                    "default".to_string(),
                    "RESULT: Implementation complete\n```python\ndef process_data(input):\n    return input.upper()\n```".to_string()
                );
            }
            _ => {}
        }
        
        Self {
            layer: layer.to_string(),
            system_prompt: 2hal9_core::config::get_system_prompt(layer),
            responses,
            delay_ms: 100,
        }
    }
    
    /// Add a custom response for testing
    pub fn add_response(&mut self, trigger: &str, response: &str) {
        self.responses.insert(trigger.to_string(), response.to_string());
    }
    
    /// Set response delay for testing
    pub fn set_delay(&mut self, delay_ms: u64) {
        self.delay_ms = delay_ms;
    }
}

#[async_trait]
impl ClaudeInterface for MockClaude {
    async fn send_message(&self, message: &str) -> Result<String> {
        debug!("MockClaude[{}] received: {}", self.layer, message);
        
        // Simulate processing delay
        tokio::time::sleep(tokio::time::Duration::from_millis(self.delay_ms)).await;
        
        // Check for specific responses
        for (trigger, response) in &self.responses {
            if message.contains(trigger) || trigger == "default" {
                info!("MockClaude[{}] responding with preset response", self.layer);
                return Ok(response.clone());
            }
        }
        
        // Default response based on layer
        Ok(format!("Mock {} response to: {}", self.layer, message))
    }
    
    fn system_prompt(&self) -> &str {
        &self.system_prompt
    }
    
    fn last_token_usage(&self) -> Option<TokenUsage> {
        Some(TokenUsage {
            prompt_tokens: 100,
            completion_tokens: 50,
            total_tokens: 150,
        })
    }
}

/// Claude API client implementation
pub struct ClaudeAPIClient {
    api_key: String,
    model: String,
    system_prompt: String,
    temperature: f32,
    max_tokens: u32,
    last_usage: Option<TokenUsage>,
    client: reqwest::Client,
}

impl ClaudeAPIClient {
    /// Create a new Claude API client
    pub fn new(
        api_key: String,
        model: String,
        layer: &str,
        temperature: f32,
        max_tokens: u32,
    ) -> Self {
        Self {
            api_key,
            model,
            system_prompt: 2hal9_core::config::get_system_prompt(layer),
            temperature,
            max_tokens,
            last_usage: None,
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl ClaudeInterface for ClaudeAPIClient {
    async fn send_message(&self, message: &str) -> Result<String> {
        let request = ClaudeRequest {
            model: self.model.clone(),
            messages: vec![
                Message {
                    role: "system".to_string(),
                    content: self.system_prompt.clone(),
                },
                Message {
                    role: "user".to_string(),
                    content: message.to_string(),
                },
            ],
            max_tokens: self.max_tokens,
            temperature: self.temperature,
        };
        
        let response = self.client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| Error::ClaudeApi(e.to_string()))?;
            
        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(Error::ClaudeApi(format!("API error: {}", error_text)));
        }
        
        let api_response: ClaudeResponse = response.json().await
            .map_err(|e| Error::ClaudeApi(e.to_string()))?;
            
        // Update token usage
        if let Some(usage) = api_response.usage {
            self.last_usage.replace(TokenUsage {
                prompt_tokens: usage.input_tokens,
                completion_tokens: usage.output_tokens,
                total_tokens: usage.input_tokens + usage.output_tokens,
            });
        }
        
        Ok(api_response.content.first()
            .map(|c| c.text.clone())
            .unwrap_or_default())
    }
    
    fn system_prompt(&self) -> &str {
        &self.system_prompt
    }
    
    fn last_token_usage(&self) -> Option<TokenUsage> {
        self.last_usage.clone()
    }
}

// API request/response types
#[derive(Serialize)]
struct ClaudeRequest {
    model: String,
    messages: Vec<Message>,
    max_tokens: u32,
    temperature: f32,
}

#[derive(Serialize, Deserialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ClaudeResponse {
    content: Vec<Content>,
    usage: Option<Usage>,
}

#[derive(Deserialize)]
struct Content {
    text: String,
}

#[derive(Deserialize)]
struct Usage {
    input_tokens: u32,
    output_tokens: u32,
}