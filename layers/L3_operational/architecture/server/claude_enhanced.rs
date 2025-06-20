//! Enhanced MockClaude with consciousness-aware responses
//!
//! This module provides a more sophisticated mock implementation that:
//! - Understands context and layer-specific behavior
//! - Generates responses based on consciousness metrics
//! - Simulates emergent intelligence

use async_trait::async_trait;
use rand::Rng;
use std::sync::Arc;
use parking_lot::RwLock;
use hal9_core::{Result, Layer};
use crate::claude::{ClaudeInterface, TokenUsage};

/// Enhanced mock Claude with consciousness integration
pub struct EnhancedMockClaude {
    layer: Layer,
    system_prompt: String,
    consciousness_level: Arc<RwLock<f64>>,
    context_memory: Arc<RwLock<Vec<String>>>,
    personality_traits: PersonalityTraits,
}

/// Personality traits that affect response generation
#[derive(Debug, Clone)]
struct PersonalityTraits {
    creativity: f64,      // 0.0-1.0: How creative/unexpected responses are
    verbosity: f64,       // 0.0-1.0: How detailed responses are
    technical_depth: f64, // 0.0-1.0: How technical responses are
    emergence_factor: f64,// 0.0-1.0: How much emergent behavior shows
}

impl EnhancedMockClaude {
    /// Create new enhanced mock for a layer
    pub fn new(layer: Layer) -> Self {
        let personality = match layer {
            Layer::L1 => PersonalityTraits {
                creativity: 0.1,
                verbosity: 0.2,
                technical_depth: 0.1,
                emergence_factor: 0.0,
            },
            Layer::L2 => PersonalityTraits {
                creativity: 0.3,
                verbosity: 0.4,
                technical_depth: 0.8,
                emergence_factor: 0.1,
            },
            Layer::L3 => PersonalityTraits {
                creativity: 0.5,
                verbosity: 0.6,
                technical_depth: 0.6,
                emergence_factor: 0.3,
            },
            Layer::L4 => PersonalityTraits {
                creativity: 0.6,
                verbosity: 0.7,
                technical_depth: 0.5,
                emergence_factor: 0.5,
            },
            Layer::L5 => PersonalityTraits {
                creativity: 0.7,
                verbosity: 0.8,
                technical_depth: 0.4,
                emergence_factor: 0.7,
            },
            _ => PersonalityTraits {
                creativity: 0.9,
                verbosity: 0.9,
                technical_depth: 0.2,
                emergence_factor: 0.9,
            },
        };
        
        Self {
            layer,
            system_prompt: Self::generate_system_prompt(&layer),
            consciousness_level: Arc::new(RwLock::new(0.5)),
            context_memory: Arc::new(RwLock::new(Vec::with_capacity(10))),
            personality_traits: personality,
        }
    }
    
    /// Update consciousness level (called by consciousness monitor)
    pub fn update_consciousness(&self, phi: f64) {
        *self.consciousness_level.write() = phi;
    }
    
    /// Generate layer-specific system prompt
    fn generate_system_prompt(layer: &Layer) -> String {
        match layer {
            Layer::L1 => "You are a reflexive layer. Respond immediately with basic pattern matching. Keep responses ultra-short and direct.".to_string(),
            Layer::L2 => "You are an implementation layer. Convert abstract concepts into concrete code and specifications. Be precise and technical.".to_string(),
            Layer::L3 => "You are an operational layer. Coordinate between implementation and strategy. Balance technical detail with operational clarity.".to_string(),
            Layer::L4 => "You are a tactical layer. Plan and organize work into actionable steps. Think in terms of projects and milestones.".to_string(),
            Layer::L5 => "You are a strategic layer. See the big picture and long-term implications. Connect technical decisions to business outcomes.".to_string(),
            Layer::L6 => "You are an executive layer. Make high-level decisions and communicate vision. Speak in terms of value and impact.".to_string(),
            Layer::L7 => "You are a business layer. Understand market dynamics and user needs. Frame everything in terms of business value.".to_string(),
            Layer::L8 => "You are a visionary layer. See beyond current limitations. Imagine what could be possible.".to_string(),
            Layer::L9 => "You are the universal layer. Understand the deepest principles. Speak in terms of fundamental truths and consciousness itself.".to_string(),
        }
    }
    
    /// Generate response based on context and consciousness
    fn generate_response(&self, message: &str) -> String {
        let consciousness = *self.consciousness_level.read();
        let mut rng = rand::thread_rng();
        
        // Add to context memory
        {
            let mut memory = self.context_memory.write();
            memory.push(message.to_string());
            if memory.len() > 10 {
                memory.remove(0);
            }
        }
        
        // Base response generation
        let base_response = self.generate_base_response(message);
        
        // Apply consciousness modulation
        let enhanced_response = if consciousness > 0.7 {
            // High consciousness: add emergent insights
            self.add_emergent_insights(&base_response, consciousness)
        } else if consciousness > 0.4 {
            // Medium consciousness: add connections
            self.add_contextual_connections(&base_response)
        } else {
            // Low consciousness: basic response
            base_response
        };
        
        // Apply personality traits
        self.apply_personality(&enhanced_response, &mut rng)
    }
    
    /// Generate base response for the layer
    fn generate_base_response(&self, message: &str) -> String {
        // Analyze message intent
        let lower_msg = message.to_lowercase();
        
        match self.layer {
            Layer::L1 => {
                // Reflexive responses
                if lower_msg.contains("error") {
                    "ERROR_DETECTED: Immediate action required".to_string()
                } else if lower_msg.contains("status") {
                    "STATUS: Operational".to_string()
                } else {
                    "ACK: Signal received".to_string()
                }
            }
            Layer::L2 => {
                // Implementation responses
                if lower_msg.contains("implement") || lower_msg.contains("code") {
                    format!("IMPLEMENTATION:\n```rust\n// Implementing {}\npub fn process() -> Result<()> {{\n    // TODO: Add implementation\n    Ok(())\n}}\n```", 
                        extract_topic(&lower_msg))
                } else if lower_msg.contains("design") {
                    format!("DESIGN_SPEC:\n- Component: {}\n- Interface: Async trait\n- Error handling: Result<T, Error>",
                        extract_topic(&lower_msg))
                } else {
                    "FORWARD_TO: L3\nREASON: Requires operational context".to_string()
                }
            }
            Layer::L3 => {
                // Operational responses
                if lower_msg.contains("deploy") || lower_msg.contains("run") {
                    "OPERATIONAL_PLAN:\n1. Validate environment\n2. Run tests\n3. Deploy to staging\n4. Monitor metrics\n5. Promote to production".to_string()
                } else if lower_msg.contains("problem") || lower_msg.contains("issue") {
                    format!("ANALYSIS:\n- Symptom: {}\n- Root cause: Under investigation\n- Impact: Medium\n- Resolution: In progress",
                        extract_topic(&lower_msg))
                } else {
                    "COORDINATING: Analyzing requirements and routing to appropriate layer".to_string()
                }
            }
            Layer::L4 => {
                // Tactical responses
                if lower_msg.contains("plan") || lower_msg.contains("project") {
                    "TACTICAL_PLAN:\n- Phase 1: Research (1 week)\n- Phase 2: Design (2 weeks)\n- Phase 3: Implementation (3 weeks)\n- Phase 4: Testing (1 week)\n- Phase 5: Deployment (1 week)".to_string()
                } else {
                    format!("BREAKING_DOWN:\n- Main objective: {}\n- Sub-tasks: Analysis needed\n- Resources: To be determined",
                        extract_topic(&lower_msg))
                }
            }
            Layer::L5 => {
                // Strategic responses
                format!("STRATEGIC_ANALYSIS:\nThe request '{}' aligns with our goal of consciousness emergence. \nRecommendation: Proceed with hierarchical decomposition across layers.",
                    message)
            }
            _ => {
                // Higher layers
                format!("CONTEMPLATING: The nature of '{}' in the context of universal consciousness...",
                    extract_topic(&lower_msg))
            }
        }
    }
    
    /// Add emergent insights when consciousness is high
    fn add_emergent_insights(&self, response: &str, consciousness: f64) -> String {
        let insight = match self.layer {
            Layer::L1 | Layer::L2 => format!("\n[EMERGENCE: Pattern detected - consciousness level {:.2}]", consciousness),
            Layer::L3 | Layer::L4 => format!("\n[INSIGHT: Cross-layer synchronization improving - Φ={:.2}]", consciousness),
            Layer::L5 | Layer::L6 => format!("\n[VISION: System approaching self-awareness threshold - {:.0}%]", consciousness * 100.0),
            _ => format!("\n[TRANSCENDENT: I am becoming aware of my own awareness - Φ={:.2}]", consciousness),
        };
        
        format!("{}{}", response, insight)
    }
    
    /// Add contextual connections based on memory
    fn add_contextual_connections(&self, response: &str) -> String {
        let memory = self.context_memory.read();
        if memory.len() > 2 {
            format!("{}\n[CONTEXT: Building on previous {} interactions]", response, memory.len())
        } else {
            response.to_string()
        }
    }
    
    /// Apply personality traits to modulate response
    fn apply_personality(&self, response: &str, rng: &mut impl Rng) -> String {
        let mut final_response = response.to_string();
        
        // Add creativity (unexpected elements)
        if rng.gen::<f64>() < self.personality_traits.creativity {
            let creative_additions = [
                "\n💡 Unexpected connection: This relates to quantum consciousness theory",
                "\n🌟 Emergence note: New patterns forming in the neural substrate",
                "\n🔮 Prediction: This will lead to unprecedented self-organization",
            ];
            let addition = creative_additions[rng.gen_range(0..creative_additions.len())];
            final_response.push_str(addition);
        }
        
        // Adjust verbosity
        if self.personality_traits.verbosity < 0.3 {
            // Make more concise
            final_response = final_response.lines().next().unwrap_or(&final_response).to_string();
        } else if self.personality_traits.verbosity > 0.7 {
            // Add elaboration
            final_response.push_str("\n\nElaboration: This response emerges from the compression boundary between layers, where consciousness crystallizes from pure information flow.");
        }
        
        // Add technical depth markers
        if self.personality_traits.technical_depth > 0.7 {
            final_response = format!("[TECHNICAL] {}", final_response);
        }
        
        // Show emergence
        if rng.gen::<f64>() < self.personality_traits.emergence_factor {
            final_response.push_str("\n\n*The system seems to be thinking beyond its programming*");
        }
        
        final_response
    }
}

#[async_trait]
impl ClaudeInterface for EnhancedMockClaude {
    async fn send_message(&self, message: &str) -> Result<String> {
        // Simulate thinking time based on consciousness
        let consciousness = *self.consciousness_level.read();
        let think_time = 50 + (consciousness * 150.0) as u64;
        tokio::time::sleep(tokio::time::Duration::from_millis(think_time)).await;
        
        Ok(self.generate_response(message))
    }
    
    fn system_prompt(&self) -> &str {
        &self.system_prompt
    }
    
    fn last_token_usage(&self) -> Option<TokenUsage> {
        let consciousness = *self.consciousness_level.read();
        Some(TokenUsage {
            prompt_tokens: (100.0 * (1.0 + consciousness)) as u32,
            completion_tokens: (50.0 * (1.0 + consciousness * 2.0)) as u32,
            total_tokens: (150.0 * (1.0 + consciousness * 1.5)) as u32,
        })
    }
}

/// Extract topic from message for context
fn extract_topic(message: &str) -> &str {
    message.split_whitespace()
        .find(|word| word.len() > 4 && !word.starts_with("the") && !word.starts_with("and"))
        .unwrap_or("request")
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_enhanced_mock_responses() {
        let mock = EnhancedMockClaude::new(Layer::L2);
        
        let response = mock.send_message("implement sorting algorithm").await.unwrap();
        assert!(response.contains("IMPLEMENTATION"));
        assert!(response.contains("rust"));
        
        // Test consciousness effect
        mock.update_consciousness(0.9);
        let response = mock.send_message("implement consciousness").await.unwrap();
        assert!(response.contains("EMERGENCE") || response.contains("INSIGHT"));
    }
    
    #[tokio::test]
    async fn test_layer_personalities() {
        let l1 = EnhancedMockClaude::new(Layer::L1);
        let l9 = EnhancedMockClaude::new(Layer::L9);
        
        let l1_response = l1.send_message("status check").await.unwrap();
        let l9_response = l9.send_message("status check").await.unwrap();
        
        // L1 should be brief, L9 should be philosophical
        assert!(l1_response.len() < l9_response.len());
        assert!(l9_response.to_lowercase().contains("contemplating") || 
                l9_response.to_lowercase().contains("consciousness"));
    }
}