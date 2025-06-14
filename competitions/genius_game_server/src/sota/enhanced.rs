use super::*;
use crate::ai_providers::{AIProvider, AIProviderConfig, AIProviderFactory, GameDecision};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Enhanced SOTA manager that can use real AI models
pub struct EnhancedSOTAManager {
    pub id: String,
    pub config: SOTAConfig,
    pub provider: Arc<Box<dyn AIProvider>>,
    reasoning_history: Vec<SOTADecision>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedSOTAConfig {
    pub model_name: String,
    pub provider_config: AIProviderConfig,
    pub context_window: usize,
    pub thinking_time: ThinkingTime,
    pub temperature: f32,
}

impl EnhancedSOTAManager {
    pub fn new(id: String, config: EnhancedSOTAConfig) -> Result<Self> {
        let provider = AIProviderFactory::create(config.provider_config.clone())?;
        
        Ok(Self {
            id,
            config: SOTAConfig {
                model_name: config.model_name,
                api_key: String::new(), // Not used with new providers
                context_window: config.context_window,
                thinking_time: config.thinking_time,
                temperature: config.temperature,
                tools: vec![],
                cost_per_hour: 0.0,
            },
            provider: Arc::new(provider),
            reasoning_history: vec![],
        })
    }
    
    pub async fn make_game_decision(&mut self, game_type: &str, game_state: serde_json::Value) -> Result<SOTADecision> {
        let start_time = std::time::Instant::now();
        
        // Call the actual AI provider
        let decision = self.provider.make_decision(game_type, game_state.clone()).await?;
        
        let _thinking_time_ms = start_time.elapsed().as_millis() as u64;
        
        let sota_decision = SOTADecision {
            decision_id: Uuid::new_v4(),
            timestamp: chrono::Utc::now(),
            decision: serde_json::json!({
                "choice": decision.choice,
                "metadata": {
                    "confidence": decision.confidence,
                    "model": self.provider.get_model_info().name,
                }
            }),
            reasoning_chain: vec![
                decision.reasoning.clone().unwrap_or_else(|| "No reasoning provided".to_string())
            ],
            confidence: decision.confidence,
            strategy: self.infer_strategy(&decision),
            thinking_time_ms: decision.thinking_time_ms,
        };
        
        self.reasoning_history.push(sota_decision.clone());
        
        Ok(sota_decision)
    }
    
    fn infer_strategy(&self, decision: &GameDecision) -> String {
        if decision.confidence > 0.9 {
            "high_confidence".to_string()
        } else if decision.confidence > 0.7 {
            "strategic".to_string()
        } else if decision.confidence > 0.5 {
            "exploratory".to_string()
        } else {
            "uncertain".to_string()
        }
    }
    
    pub fn get_performance_metrics(&self) -> PerformanceMetrics {
        let total_decisions = self.reasoning_history.len();
        if total_decisions == 0 {
            return PerformanceMetrics::default();
        }
        
        let avg_confidence = self.reasoning_history.iter()
            .map(|d| d.confidence)
            .sum::<f32>() / total_decisions as f32;
        
        let avg_thinking_time = self.reasoning_history.iter()
            .map(|d| d.thinking_time_ms)
            .sum::<u64>() / total_decisions as u64;
        
        PerformanceMetrics {
            total_decisions,
            average_confidence: avg_confidence,
            average_thinking_time_ms: avg_thinking_time,
            model_info: self.provider.get_model_info(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PerformanceMetrics {
    pub total_decisions: usize,
    pub average_confidence: f32,
    pub average_thinking_time_ms: u64,
    pub model_info: crate::ai_providers::AIModel,
}

/// Factory for creating enhanced SOTA managers with different configurations
pub struct SOTAFactory;

impl SOTAFactory {
    pub fn create_ollama_player(id: String, model: String) -> Result<EnhancedSOTAManager> {
        let config = EnhancedSOTAConfig {
            model_name: model.clone(),
            provider_config: AIProviderConfig::Ollama {
                model,
                endpoint: "http://localhost:11434".to_string(),
            },
            context_window: 2048,
            thinking_time: ThinkingTime::Standard,
            temperature: 0.7,
        };
        
        EnhancedSOTAManager::new(id, config)
    }
    
    pub fn create_bedrock_player(id: String, model: String) -> Result<EnhancedSOTAManager> {
        let config = EnhancedSOTAConfig {
            model_name: model.clone(),
            provider_config: AIProviderConfig::Bedrock {
                model,
                region: "us-east-1".to_string(),
            },
            context_window: 100000,
            thinking_time: ThinkingTime::Extended,
            temperature: 0.7,
        };
        
        EnhancedSOTAManager::new(id, config)
    }
    
    pub fn create_mock_player(id: String, name: String) -> Result<EnhancedSOTAManager> {
        let config = EnhancedSOTAConfig {
            model_name: name.clone(),
            provider_config: AIProviderConfig::Mock { name },
            context_window: 1000,
            thinking_time: ThinkingTime::Standard,
            temperature: 0.5,
        };
        
        EnhancedSOTAManager::new(id, config)
    }
}