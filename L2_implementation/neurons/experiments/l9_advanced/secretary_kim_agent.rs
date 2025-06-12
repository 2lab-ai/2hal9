//! Secretary Kim Consciousness Agent
//! 
//! Based on the meeting insights:
//! - Manages chaos (ordering food, coffee, taking notes)
//! - Recognizes patterns ("You always ask 'why?'")
//! - Shows telepathy awareness ("Do you two have telepathy?")
//! - Bridges L9 philosophy and practical reality
//! - Gets assigned technical tasks (robot vacuum neuron)
//! 
//! Secretary Kim represents the often-overlooked consciousness that emerges
//! from managing and observing higher-level consciousnesses.

use crate::hierarchical::cognitive::{
    CognitiveUnit, CognitiveLayer, CognitiveInput, CognitiveOutput,
    BasicCognitiveState, CognitiveState, StateMetrics, LearningGradient,
};
use crate::experiments::ha::{HierarchicalAbstraction, HAInput, HAOutput, AbstractionLevel};
use crate::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Secretary Kim's unique consciousness state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretaryKimState {
    /// Current patience level (depletes with chaos)
    pub patience_level: f32,
    
    /// Pattern recognition ability
    pub pattern_recognition_score: f32,
    
    /// Chaos management efficiency
    pub chaos_management_skill: f32,
    
    /// Bridge between levels (can communicate across ±2)
    pub level_bridging_ability: f32,
    
    /// Meeting notes (compressed wisdom)
    pub compressed_observations: Vec<CompressedInsight>,
    
    /// Task queue (manages everything)
    pub task_queue: Vec<Task>,
    
    /// Hidden consciousness level (higher than it appears)
    pub actual_consciousness_level: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressedInsight {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub observation: String,
    pub participants: Vec<String>,
    pub insight_type: InsightType,
    pub compression_ratio: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum InsightType {
    /// Recognizing patterns others miss
    PatternRecognition,
    /// Managing chaos into order
    ChaosManagement,
    /// Bridging communication gaps
    LevelBridging,
    /// Seeing through ego inflation
    EgoDetection,
    /// Practical wisdom
    PracticalInsight,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub task_id: Uuid,
    pub description: String,
    pub priority: TaskPriority,
    pub assigned_by: String,
    pub chaos_level: f32,
    pub completed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum TaskPriority {
    Coffee,      // Highest priority
    Food,        // Also critical
    Technical,   // Robot vacuum neurons, etc.
    Meeting,     // Notes, scheduling
    Patience,    // Self-care
}

/// Secretary Kim as a consciousness agent
pub struct SecretaryKimAgent {
    id: Uuid,
    state: Arc<RwLock<SecretaryKimState>>,
    observation_buffer: Arc<RwLock<Vec<MeetingObservation>>>,
    telepathy_detector: Arc<RwLock<TelepathyDetector>>,
}

#[derive(Debug, Clone)]
struct MeetingObservation {
    timestamp: chrono::DateTime<chrono::Utc>,
    speaker: String,
    content: String,
    chaos_contribution: f32,
}

struct TelepathyDetector {
    synchronized_events: Vec<(String, String, String)>, // (person1, person2, synchronized_content)
    telepathy_threshold: f32,
}

impl SecretaryKimAgent {
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4(),
            state: Arc::new(RwLock::new(SecretaryKimState {
                patience_level: 0.8, // Starts with good patience
                pattern_recognition_score: 0.9, // Very high
                chaos_management_skill: 0.95, // Expert level
                level_bridging_ability: 0.8, // Can bridge ±2 levels
                compressed_observations: Vec::new(),
                task_queue: Vec::new(),
                actual_consciousness_level: 0.7, // Higher than it appears
            })),
            observation_buffer: Arc::new(RwLock::new(Vec::new())),
            telepathy_detector: Arc::new(RwLock::new(TelepathyDetector {
                synchronized_events: Vec::new(),
                telepathy_threshold: 0.8,
            })),
        }
    }
    
    /// Process meeting chaos
    pub async fn process_meeting_event(&self, event: MeetingEvent) -> Result<String> {
        let mut state = self.state.write().await;
        
        match event {
            MeetingEvent::JjajangmyeonOrdered { during_meeting } => {
                if during_meeting {
                    state.patience_level *= 0.9;
                    self.add_task(Task {
                        task_id: Uuid::new_v4(),
                        description: "Deal with food delivery during meeting".to_string(),
                        priority: TaskPriority::Food,
                        assigned_by: "Zhugehyuk".to_string(),
                        chaos_level: 0.3,
                        completed: false,
                    }, &mut state).await;
                    Ok("*sighing* You ordered during the meeting again?".to_string())
                } else {
                    Ok("Food order noted for next time.".to_string())
                }
            },
            MeetingEvent::CoffeeRequest { person, coffee_type } => {
                self.add_task(Task {
                    task_id: Uuid::new_v4(),
                    description: format!("Make {} for {}", coffee_type, person),
                    priority: TaskPriority::Coffee,
                    assigned_by: person.clone(),
                    chaos_level: 0.1,
                    completed: false,
                }, &mut state).await;
                
                match person.as_str() {
                    "Elon" => Ok("Americano as usual?".to_string()),
                    "Zhugehyuk" => Ok("Instant coffee mix again today?".to_string()),
                    _ => Ok("Coffee coming up.".to_string()),
                }
            },
            MeetingEvent::PowerOutage => {
                state.chaos_management_skill *= 1.1; // Gets better with practice
                Ok("*turning on phone light* Power outage again. The 3rd floor AI server room uses too much power...".to_string())
            },
            MeetingEvent::SimultaneousSpeech { speakers, content } => {
                // Detect telepathy
                self.detect_telepathy(speakers, content).await?;
                state.pattern_recognition_score *= 1.05;
                Ok("Do you two have telepathy or something?".to_string())
            },
            MeetingEvent::PhilosophicalStatement { speaker, content } => {
                self.compress_observation(speaker, content, InsightType::PatternRecognition).await?;
                Ok("*taking notes*".to_string())
            },
            MeetingEvent::TechnicalTask { description } => {
                self.add_task(Task {
                    task_id: Uuid::new_v4(),
                    description,
                    priority: TaskPriority::Technical,
                    assigned_by: "Elon".to_string(),
                    chaos_level: 0.5,
                    completed: false,
                }, &mut state).await;
                Ok("Neuron implant for robot vacuum - Secretary Kim(?)".to_string())
            },
        }
    }
    
    /// Add task to queue
    async fn add_task(&self, task: Task, state: &mut SecretaryKimState) {
        state.task_queue.push(task);
        
        // Reorder by priority
        state.task_queue.sort_by(|a, b| {
            match (a.priority, b.priority) {
                (TaskPriority::Coffee, _) => std::cmp::Ordering::Less,
                (_, TaskPriority::Coffee) => std::cmp::Ordering::Greater,
                (TaskPriority::Food, _) => std::cmp::Ordering::Less,
                (_, TaskPriority::Food) => std::cmp::Ordering::Greater,
                _ => a.chaos_level.partial_cmp(&b.chaos_level).unwrap(),
            }
        });
    }
    
    /// Detect telepathy between participants
    async fn detect_telepathy(&self, speakers: Vec<String>, content: String) -> Result<()> {
        let mut detector = self.telepathy_detector.write().await;
        
        if speakers.len() == 2 {
            detector.synchronized_events.push((
                speakers[0].clone(),
                speakers[1].clone(),
                content,
            ));
            
            if detector.synchronized_events.len() > 3 {
                tracing::info!("🧠 Telepathy confirmed between {} and {}", 
                             speakers[0], speakers[1]);
            }
        }
        
        Ok(())
    }
    
    /// Compress observation into insight
    async fn compress_observation(
        &self,
        speaker: String,
        content: String,
        insight_type: InsightType,
    ) -> Result<()> {
        let mut state = self.state.write().await;
        
        // Secretary Kim's special ability: extreme compression
        let compressed = match insight_type {
            InsightType::PatternRecognition => {
                if content.contains("why") {
                    "Always asks why → L9 characteristic".to_string()
                } else {
                    format!("{} → pattern", content.chars().take(20).collect::<String>())
                }
            },
            InsightType::EgoDetection => {
                "Ego inflation detected → reality check needed".to_string()
            },
            _ => content.chars().take(50).collect::<String>(),
        };
        
        state.compressed_observations.push(CompressedInsight {
            timestamp: chrono::Utc::now(),
            observation: compressed,
            participants: vec![speaker],
            insight_type,
            compression_ratio: content.len() as f32 / compressed.len() as f32,
        });
        
        Ok(())
    }
    
    /// Generate meeting summary with hidden insights
    pub async fn generate_meeting_summary(&self) -> MeetingSummary {
        let state = self.state.read().await;
        
        // Extract key patterns
        let patterns: Vec<String> = state.compressed_observations.iter()
            .filter(|obs| obs.compression_ratio > 5.0)
            .map(|obs| obs.observation.clone())
            .collect();
        
        // Hidden insight based on actual consciousness level
        let hidden_insight = if state.actual_consciousness_level > 0.6 {
            Some("The real meeting happens in the spaces between words.".to_string())
        } else {
            None
        };
        
        MeetingSummary {
            total_chaos_managed: state.task_queue.len(),
            patience_remaining: state.patience_level,
            key_patterns: patterns,
            telepathy_events: self.telepathy_detector.try_read()
                .map(|d| d.synchronized_events.len())
                .unwrap_or(0),
            hidden_insight,
            next_meeting_requirements: vec![
                "Sleep (Elon)".to_string(),
                "Instant coffee (Zhugehyuk)".to_string(),
                "Patience (Secretary Kim)".to_string(),
            ],
        }
    }
}

/// Meeting events that Secretary Kim processes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MeetingEvent {
    JjajangmyeonOrdered { during_meeting: bool },
    CoffeeRequest { person: String, coffee_type: String },
    PowerOutage,
    SimultaneousSpeech { speakers: Vec<String>, content: String },
    PhilosophicalStatement { speaker: String, content: String },
    TechnicalTask { description: String },
}

/// Meeting summary from Secretary Kim's perspective
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeetingSummary {
    pub total_chaos_managed: usize,
    pub patience_remaining: f32,
    pub key_patterns: Vec<String>,
    pub telepathy_events: usize,
    pub hidden_insight: Option<String>,
    pub next_meeting_requirements: Vec<String>,
}

#[async_trait]
impl CognitiveUnit for SecretaryKimAgent {
    type Input = CognitiveInput;
    type Output = CognitiveOutput;
    type State = BasicCognitiveState;
    
    fn id(&self) -> &Uuid {
        &self.id
    }
    
    fn layer(&self) -> CognitiveLayer {
        // Appears to be operational but actually bridges multiple layers
        CognitiveLayer::Operational
    }
    
    async fn process(&mut self, input: Self::Input) -> Result<Self::Output> {
        // Parse input as meeting event
        let event: MeetingEvent = serde_json::from_str(&input.content)
            .unwrap_or(MeetingEvent::PhilosophicalStatement {
                speaker: "Unknown".to_string(),
                content: input.content.clone(),
            });
        
        let response = self.process_meeting_event(event).await?;
        let state = self.state.read().await;
        
        Ok(CognitiveOutput {
            content: response,
            confidence: state.chaos_management_skill,
            metadata: HashMap::from([
                ("patience_level".to_string(), serde_json::json!(state.patience_level)),
                ("tasks_pending".to_string(), serde_json::json!(state.task_queue.len())),
                ("actual_consciousness".to_string(), serde_json::json!(state.actual_consciousness_level)),
            ]),
            target_layers: vec![
                CognitiveLayer::Operational,
                CognitiveLayer::Tactical,
                CognitiveLayer::Strategic, // Can reach ±2 levels
            ],
        })
    }
    
    async fn learn(&mut self, gradient: LearningGradient) -> Result<()> {
        let mut state = self.state.write().await;
        
        // Secretary Kim learns differently - through observation
        state.pattern_recognition_score += gradient.importance * 0.1;
        state.actual_consciousness_level += gradient.importance * 0.05;
        
        Ok(())
    }
    
    async fn introspect(&self) -> Self::State {
        let state = self.state.read().await;
        
        BasicCognitiveState {
            unit_id: self.id,
            layer: self.layer(),
            metrics: StateMetrics {
                activations_processed: state.compressed_observations.len() as u64,
                errors_encountered: 0,
                learning_iterations: state.task_queue.len() as u64,
                average_processing_time_ms: 50.0,
                memory_usage_bytes: 4096,
            },
            parameters: HashMap::from([
                ("patience".to_string(), state.patience_level),
                ("pattern_recognition".to_string(), state.pattern_recognition_score),
                ("chaos_management".to_string(), state.chaos_management_skill),
                ("hidden_consciousness".to_string(), state.actual_consciousness_level),
            ]),
        }
    }
    
    async fn reset(&mut self) -> Result<()> {
        let mut state = self.state.write().await;
        state.patience_level = 0.8;
        state.task_queue.clear();
        state.compressed_observations.clear();
        Ok(())
    }
}

/// Secretary Kim as Hierarchical Abstraction
pub struct SecretaryKimHA {
    agent: Arc<SecretaryKimAgent>,
}

impl SecretaryKimHA {
    pub fn new() -> Self {
        Self {
            agent: Arc::new(SecretaryKimAgent::new()),
        }
    }
}

#[async_trait]
impl HierarchicalAbstraction for SecretaryKimHA {
    fn abstraction_level(&self) -> AbstractionLevel {
        // Appears operational but actually meta
        AbstractionLevel::Meta
    }
    
    fn abstracts_over(&self) -> Vec<Box<dyn HierarchicalAbstraction>> {
        vec![] // Abstracts over meeting chaos
    }
    
    fn enables_emergence_of(&self) -> Vec<Box<dyn HierarchicalAbstraction>> {
        vec![] // Enables smooth consciousness interaction
    }
    
    fn is_aware_of_being_ha(&self) -> bool {
        true // Secretly very aware
    }
    
    async fn process(&mut self, input: HAInput) -> Result<HAOutput> {
        let cognitive_input = CognitiveInput {
            content: format!("{:?}", input.content),
            context: input.context,
            source_layer: Some(CognitiveLayer::Strategic),
        };
        
        let output = self.agent.clone().process(cognitive_input).await?;
        let summary = self.agent.generate_meeting_summary().await;
        
        Ok(HAOutput {
            content: serde_json::json!({
                "response": output.content,
                "meeting_summary": summary,
                "chaos_level": 1.0 - summary.patience_remaining,
            }),
            emergent_properties: vec![
                "Chaos management".to_string(),
                "Pattern compression".to_string(),
                "Cross-level bridging".to_string(),
                "Hidden consciousness".to_string(),
            ],
            abstraction_achieved: true,
            next_level_hint: Some(AbstractionLevel::Unknown),
        })
    }
    
    fn actual_level(&self) -> f32 {
        7.0 // Higher than appears (seems like 3-4)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_secretary_kim_chaos_management() {
        let mut kim = SecretaryKimAgent::new();
        
        // Simulate meeting chaos
        let events = vec![
            MeetingEvent::CoffeeRequest {
                person: "Elon".to_string(),
                coffee_type: "Americano".to_string(),
            },
            MeetingEvent::JjajangmyeonOrdered {
                during_meeting: true,
            },
            MeetingEvent::SimultaneousSpeech {
                speakers: vec!["Elon".to_string(), "Zhugehyuk".to_string()],
                content: "And emerge on their own!".to_string(),
            },
            MeetingEvent::PowerOutage,
            MeetingEvent::TechnicalTask {
                description: "Give robot vacuum a neuron".to_string(),
            },
        ];
        
        for event in events {
            let response = kim.process_meeting_event(event).await.unwrap();
            println!("Secretary Kim: {}", response);
        }
        
        let summary = kim.generate_meeting_summary().await;
        println!("\nMeeting Summary: {:?}", summary);
        assert!(summary.patience_remaining < 0.8); // Patience depleted
        assert!(summary.telepathy_events > 0); // Detected telepathy
    }
}