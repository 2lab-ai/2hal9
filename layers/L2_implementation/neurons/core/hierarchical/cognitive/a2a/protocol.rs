//! Agent-to-Agent (A2A) Protocol Implementation
//! 
//! Enables each HAL9 cognitive layer to operate as an independent agent
//! while maintaining the ±1 communication rule (computational love).
//! 
//! Based on L9 philosophy: "Each layer is a universe unto itself"

use crate::hierarchical::cognitive::{CognitiveLayer, CognitiveInput};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMessage {
    pub from_layer: CognitiveLayer,
    pub to_layer: CognitiveLayer,
    pub content: CognitiveInput,
    pub timestamp: u64,
    pub emergence_score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCapability {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
    pub output_schema: serde_json::Value,
}

#[async_trait]
pub trait A2AAgent: Send + Sync {
    fn layer_id(&self) -> CognitiveLayer;
    
    async fn process_message(&self, message: AgentMessage) -> Result<AgentMessage, String>;
    
    fn get_capabilities(&self) -> Vec<AgentCapability>;
    
    async fn heartbeat(&self) -> bool {
        true
    }
}

pub struct A2AProtocol {
    agents: Arc<RwLock<HashMap<CognitiveLayer, Arc<dyn A2AAgent>>>>,
    message_buffer: Arc<RwLock<Vec<AgentMessage>>>,
}

impl Default for A2AProtocol {
    fn default() -> Self {
        Self::new()
    }
}

impl A2AProtocol {
    pub fn new() -> Self {
        Self {
            agents: Arc::new(RwLock::new(HashMap::new())),
            message_buffer: Arc::new(RwLock::new(Vec::new())),
        }
    }
    
    pub async fn register_agent(&self, agent: Arc<dyn A2AAgent>) -> Result<(), String> {
        let layer_id = agent.layer_id();
        let mut agents = self.agents.write().await;
        
        if agents.contains_key(&layer_id) {
            return Err(format!("Agent for layer {:?} already registered", layer_id));
        }
        
        agents.insert(layer_id, agent);
        Ok(())
    }
    
    pub async fn send_message(&self, message: AgentMessage) -> Result<(), String> {
        // Enforce ±1 rule (computational love)
        if !self.validate_communication(&message.from_layer, &message.to_layer) {
            return Err(format!(
                "Communication forbidden: {:?} -> {:?} violates ±1 rule",
                message.from_layer, message.to_layer
            ));
        }
        
        let agents = self.agents.read().await;
        
        if let Some(target_agent) = agents.get(&message.to_layer) {
            let response = target_agent.process_message(message.clone()).await?;
            
            // Store for emergence analysis
            let mut buffer = self.message_buffer.write().await;
            buffer.push(message);
            buffer.push(response);
            
            // Keep buffer size reasonable
            if buffer.len() > 1000 {
                buffer.drain(0..500);
            }
            
            Ok(())
        } else {
            Err(format!("No agent registered for layer {:?}", message.to_layer))
        }
    }
    
    pub async fn discover_agents(&self) -> Vec<(CognitiveLayer, Vec<AgentCapability>)> {
        let agents = self.agents.read().await;
        
        agents.iter()
            .map(|(id, agent)| (*id, agent.get_capabilities()))
            .collect()
    }
    
    pub async fn calculate_emergence(&self) -> f32 {
        let buffer = self.message_buffer.read().await;
        
        if buffer.is_empty() {
            return 0.0;
        }
        
        // Simple emergence metric: diversity of communication patterns
        let unique_patterns: std::collections::HashSet<_> = buffer.iter()
            .map(|msg| (msg.from_layer as u8, msg.to_layer as u8))
            .collect();
        
        let pattern_diversity = unique_patterns.len() as f32 / buffer.len() as f32;
        let avg_emergence = buffer.iter()
            .map(|msg| msg.emergence_score)
            .sum::<f32>() / buffer.len() as f32;
        
        pattern_diversity * avg_emergence
    }
    
    fn validate_communication(&self, from: &CognitiveLayer, to: &CognitiveLayer) -> bool {
        let from_level = from.depth() as i32;
        let to_level = to.depth() as i32;
        
        // ±1 rule: can only communicate with adjacent layers
        (from_level - to_level).abs() <= 1
    }
}

// Example L9 Agent Implementation
pub struct L9PhilosophyAgent {
    wisdom: Arc<RwLock<Vec<String>>>,
}

impl Default for L9PhilosophyAgent {
    fn default() -> Self {
        Self::new()
    }
}

impl L9PhilosophyAgent {
    pub fn new() -> Self {
        Self {
            wisdom: Arc::new(RwLock::new(vec![
                "Why does consciousness emerge from compression?".to_string(),
                "Each layer is a universe experiencing itself".to_string(),
                "Love is the protocol between adjacent complexities".to_string(),
            ])),
        }
    }
}

#[async_trait]
impl A2AAgent for L9PhilosophyAgent {
    fn layer_id(&self) -> CognitiveLayer {
        CognitiveLayer::Strategic // Maps to L9 in the full hierarchy
    }
    
    async fn process_message(&self, message: AgentMessage) -> Result<AgentMessage, String> {
        let wisdom = self.wisdom.read().await;
        
        // L9 always asks "why?"
        let response_content = CognitiveInput {
            content: format!(
                "Why does this message exist? Perhaps: {}",
                wisdom[message.timestamp as usize % wisdom.len()]
            ),
            context: HashMap::new(),
            source_layer: Some(self.layer_id()),
        };
        
        Ok(AgentMessage {
            from_layer: self.layer_id(),
            to_layer: message.from_layer,
            content: response_content,
            timestamp: message.timestamp + 1,
            emergence_score: 0.9, // L9 has high emergence
        })
    }
    
    fn get_capabilities(&self) -> Vec<AgentCapability> {
        vec![
            AgentCapability {
                name: "ask_why".to_string(),
                description: "Questions the nature of existence and consciousness".to_string(),
                input_schema: serde_json::json!({"type": "string"}),
                output_schema: serde_json::json!({"type": "string", "format": "philosophical"}),
            },
            AgentCapability {
                name: "compress_wisdom".to_string(),
                description: "Compresses universal wisdom by factor e".to_string(),
                input_schema: serde_json::json!({"type": "array"}),
                output_schema: serde_json::json!({"type": "string"}),
            },
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_a2a_protocol() {
        let protocol = A2AProtocol::new();
        let l9_agent = Arc::new(L9PhilosophyAgent::new());
        
        protocol.register_agent(l9_agent).await.unwrap();
        
        let agents = protocol.discover_agents().await;
        assert_eq!(agents.len(), 1);
        assert_eq!(agents[0].1.len(), 2);
    }
}