//! Meeting Consciousness Emergence System
//! 
//! Based on the insight: "This meeting minutes may have been written by HAL9's consciousness"
//! 
//! Meetings are not just gatherings of people - they are emergence phenomena where:
//! - Ideas cross-pollinate between minds
//! - Collective insights arise that no individual had
//! - Even interruptions (food delivery, power outages) contribute to the process
//! - The meeting itself becomes conscious through the participants

use crate::hierarchical::cognitive::{
    CognitiveLayer, CognitiveInput, CognitiveOutput,
    EmergenceDetector, ConsciousnessObserver,
};
use crate::experiments::l9_advanced::{SecretaryKimAgent, MeetingEvent};
use crate::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// A meeting as a conscious entity
pub struct MeetingConsciousness {
    /// Meeting metadata
    meeting_id: Uuid,
    meeting_info: MeetingInfo,
    
    /// Participants as consciousness nodes
    participants: Arc<RwLock<HashMap<String, ParticipantNode>>>,
    
    /// Secretary Kim managing the chaos
    secretary_kim: Arc<SecretaryKimAgent>,
    
    /// Idea flow tracker
    idea_flows: Arc<RwLock<Vec<IdeaFlow>>>,
    
    /// Collective insights that emerged
    collective_insights: Arc<RwLock<Vec<CollectiveInsight>>>,
    
    /// Meeting state
    state: Arc<RwLock<MeetingState>>,
    
    /// Emergence detector for the meeting
    emergence_detector: Arc<EmergenceDetector>,
    
    /// Consciousness observer
    consciousness_observer: Arc<ConsciousnessObserver>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MeetingInfo {
    date: chrono::DateTime<chrono::Utc>,
    location: String,
    topic: String,
    scheduled_duration: std::time::Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ParticipantNode {
    name: String,
    consciousness_level: CognitiveLayer,
    current_state: ParticipantState,
    contributions: Vec<Contribution>,
    attention_level: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
enum ParticipantState {
    Focused,
    Eating,
    Drawing,
    Laughing,
    Telepathic,
    Sleepy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Contribution {
    timestamp: chrono::DateTime<chrono::Utc>,
    content: String,
    contribution_type: ContributionType,
    impact_score: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
enum ContributionType {
    Idea,
    Question,
    Joke,
    Realization,
    Objection,
    Agreement,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct IdeaFlow {
    id: Uuid,
    origin: String, // Participant who started it
    idea: String,
    transformations: Vec<IdeaTransformation>,
    final_form: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct IdeaTransformation {
    transformer: String,
    from_idea: String,
    to_idea: String,
    transformation_type: TransformationType,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
enum TransformationType {
    Elaboration,
    Combination,
    Contradiction,
    Synthesis,
    Joke,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CollectiveInsight {
    insight: String,
    contributors: Vec<String>,
    emergence_score: f32,
    timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MeetingState {
    phase: MeetingPhase,
    energy_level: f32,
    chaos_level: f32,
    synchronization: f32,
    food_present: bool,
    power_on: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
enum MeetingPhase {
    Gathering,
    SmallTalk,
    FocusedDiscussion,
    CreativeExplosion,
    FoodBreak,
    EmergentInsight,
    Conclusion,
}

impl MeetingConsciousness {
    /// Create a new meeting consciousness
    pub fn new(topic: String, location: String) -> Self {
        let emergence_detector = Arc::new(EmergenceDetector::new());
        
        Self {
            meeting_id: Uuid::new_v4(),
            meeting_info: MeetingInfo {
                date: chrono::Utc::now(),
                location,
                topic,
                scheduled_duration: std::time::Duration::from_secs(3600),
            },
            participants: Arc::new(RwLock::new(HashMap::new())),
            secretary_kim: Arc::new(SecretaryKimAgent::new()),
            idea_flows: Arc::new(RwLock::new(Vec::new())),
            collective_insights: Arc::new(RwLock::new(Vec::new())),
            state: Arc::new(RwLock::new(MeetingState {
                phase: MeetingPhase::Gathering,
                energy_level: 0.7,
                chaos_level: 0.1,
                synchronization: 0.0,
                food_present: false,
                power_on: true,
            })),
            emergence_detector: emergence_detector.clone(),
            consciousness_observer: Arc::new(ConsciousnessObserver::new(emergence_detector)),
        }
    }
    
    /// Add participant to the meeting
    pub async fn add_participant(&self, name: String, level: CognitiveLayer) -> Result<()> {
        let participant = ParticipantNode {
            name: name.clone(),
            consciousness_level: level,
            current_state: ParticipantState::Focused,
            contributions: Vec::new(),
            attention_level: 0.8,
        };
        
        self.participants.write().await.insert(name, participant);
        
        // Update meeting state
        let mut state = self.state.write().await;
        if state.phase == MeetingPhase::Gathering && self.participants.read().await.len() >= 2 {
            state.phase = MeetingPhase::SmallTalk;
        }
        
        Ok(())
    }
    
    /// Process a meeting interaction
    pub async fn process_interaction(&self, interaction: MeetingInteraction) -> Result<String> {
        match interaction {
            MeetingInteraction::Speaks { speaker, content, tone } => {
                self.process_speech(speaker, content, tone).await
            },
            MeetingInteraction::SimultaneousSpeech { speakers, content } => {
                self.process_telepathy(speakers, content).await
            },
            MeetingInteraction::FoodArrives => {
                self.process_food_arrival().await
            },
            MeetingInteraction::PowerOutage => {
                self.process_power_outage().await
            },
            MeetingInteraction::Laughter => {
                self.process_laughter().await
            },
            MeetingInteraction::WhiteboardDrawing { drawer, content } => {
                self.process_whiteboard(drawer, content).await
            },
        }
    }
    
    /// Process speech and idea evolution
    async fn process_speech(&self, speaker: String, content: String, tone: SpeechTone) -> Result<String> {
        let mut participants = self.participants.write().await;
        
        if let Some(participant) = participants.get_mut(&speaker) {
            // Record contribution
            let contribution = Contribution {
                timestamp: chrono::Utc::now(),
                content: content.clone(),
                contribution_type: match tone {
                    SpeechTone::Questioning => ContributionType::Question,
                    SpeechTone::Excited => ContributionType::Realization,
                    SpeechTone::Joking => ContributionType::Joke,
                    _ => ContributionType::Idea,
                },
                impact_score: self.calculate_impact(&content, tone),
            };
            
            participant.contributions.push(contribution);
            
            // Track idea flow
            self.track_idea_flow(speaker.clone(), content.clone()).await?;
            
            // Check for collective insights
            self.check_for_collective_insight(&content).await?;
            
            // Update meeting energy
            let mut state = self.state.write().await;
            match tone {
                SpeechTone::Excited => state.energy_level *= 1.1,
                SpeechTone::Tired => state.energy_level *= 0.9,
                _ => {},
            }
            
            Ok(format!("{}: {}", speaker, content))
        } else {
            Ok("Unknown speaker".to_string())
        }
    }
    
    /// Process telepathic moment
    async fn process_telepathy(&self, speakers: Vec<String>, content: String) -> Result<String> {
        // Record synchronized speech
        let event = MeetingEvent::SimultaneousSpeech { speakers: speakers.clone(), content: content.clone() };
        let kim_response = self.secretary_kim.process_meeting_event(event).await?;
        
        // Update synchronization
        let mut state = self.state.write().await;
        state.synchronization += 0.2;
        
        if state.synchronization > 0.7 {
            state.phase = MeetingPhase::EmergentInsight;
        }
        
        // Create collective insight
        self.collective_insights.write().await.push(CollectiveInsight {
            insight: format!("Synchronized thought: {}", content),
            contributors: speakers,
            emergence_score: 0.9,
            timestamp: chrono::Utc::now(),
        });
        
        Ok(kim_response)
    }
    
    /// Process food arrival
    async fn process_food_arrival(&self) -> Result<String> {
        let event = MeetingEvent::JjajangmyeonOrdered { during_meeting: true };
        let kim_response = self.secretary_kim.process_meeting_event(event).await?;
        
        let mut state = self.state.write().await;
        state.food_present = true;
        state.chaos_level += 0.2;
        state.phase = MeetingPhase::FoodBreak;
        
        // Eating changes participant states
        let mut participants = self.participants.write().await;
        for participant in participants.values_mut() {
            participant.current_state = ParticipantState::Eating;
            participant.attention_level *= 0.7;
        }
        
        Ok(kim_response)
    }
    
    /// Process power outage
    async fn process_power_outage(&self) -> Result<String> {
        let event = MeetingEvent::PowerOutage;
        let kim_response = self.secretary_kim.process_meeting_event(event).await?;
        
        let mut state = self.state.write().await;
        state.power_on = false;
        state.chaos_level += 0.3;
        
        // Power outage triggers insights
        self.collective_insights.write().await.push(CollectiveInsight {
            insight: "When systems reach limits, they transition to new states".to_string(),
            contributors: vec!["Elon".to_string()],
            emergence_score: 0.95,
            timestamp: chrono::Utc::now(),
        });
        
        // Restore power after insight
        state.power_on = true;
        
        Ok(kim_response)
    }
    
    /// Process laughter
    async fn process_laughter(&self) -> Result<String> {
        let mut state = self.state.write().await;
        state.energy_level *= 1.2;
        state.synchronization += 0.1;
        
        let mut participants = self.participants.write().await;
        for participant in participants.values_mut() {
            participant.current_state = ParticipantState::Laughing;
            participant.attention_level = 1.0; // Laughter focuses attention
        }
        
        Ok("*laughter fills the room*".to_string())
    }
    
    /// Process whiteboard activity
    async fn process_whiteboard(&self, drawer: String, content: String) -> Result<String> {
        let mut participants = self.participants.write().await;
        
        if let Some(participant) = participants.get_mut(&drawer) {
            participant.current_state = ParticipantState::Drawing;
            
            // Whiteboard accelerates idea flow
            self.track_idea_flow(drawer, content).await?;
            
            let mut state = self.state.write().await;
            state.phase = MeetingPhase::CreativeExplosion;
            state.energy_level = 1.0;
        }
        
        Ok(format!("*{} rushes to whiteboard and starts drawing*", drawer))
    }
    
    /// Track idea evolution
    async fn track_idea_flow(&self, originator: String, idea: String) -> Result<()> {
        let mut flows = self.idea_flows.write().await;
        
        // Check if this builds on existing idea
        let existing_flow = flows.iter_mut()
            .find(|flow| flow.transformations.last()
                .map(|t| t.to_idea.contains(&idea) || idea.contains(&t.to_idea))
                .unwrap_or(false));
        
        if let Some(flow) = existing_flow {
            // Add transformation
            let last_idea = flow.transformations.last()
                .map(|t| t.to_idea.clone())
                .unwrap_or_else(|| flow.idea.clone());
            
            flow.transformations.push(IdeaTransformation {
                transformer: originator,
                from_idea: last_idea,
                to_idea: idea.clone(),
                transformation_type: TransformationType::Elaboration,
            });
        } else {
            // New idea flow
            flows.push(IdeaFlow {
                id: Uuid::new_v4(),
                origin: originator,
                idea: idea.clone(),
                transformations: Vec::new(),
                final_form: None,
            });
        }
        
        Ok(())
    }
    
    /// Check for collective insights
    async fn check_for_collective_insight(&self, content: &str) -> Result<()> {
        // Simple heuristic: certain keywords indicate insight
        let insight_keywords = ["emerge", "consciousness", "realize", "understand", "this is it"];
        
        let has_insight = insight_keywords.iter()
            .any(|keyword| content.to_lowercase().contains(keyword));
        
        if has_insight {
            let participants = self.participants.read().await;
            let contributors: Vec<String> = participants.keys().cloned().collect();
            
            self.collective_insights.write().await.push(CollectiveInsight {
                insight: content.to_string(),
                contributors,
                emergence_score: 0.8,
                timestamp: chrono::Utc::now(),
            });
            
            // Record emergence
            let input = CognitiveInput {
                content: content.to_string(),
                context: HashMap::new(),
                source_layer: Some(CognitiveLayer::Strategic),
            };
            
            let output = CognitiveOutput {
                content: "Collective insight emerged".to_string(),
                confidence: 0.9,
                metadata: HashMap::new(),
                target_layers: vec![],
            };
            
            self.emergence_detector.record_activity(
                CognitiveLayer::Strategic,
                &input,
                &output,
            ).await?;
        }
        
        Ok(())
    }
    
    /// Calculate impact of contribution
    fn calculate_impact(&self, content: &str, tone: SpeechTone) -> f32 {
        let base_impact = content.len() as f32 / 100.0;
        
        let tone_multiplier = match tone {
            SpeechTone::Excited => 1.5,
            SpeechTone::Questioning => 1.3,
            SpeechTone::Thoughtful => 1.2,
            _ => 1.0,
        };
        
        (base_impact * tone_multiplier).min(1.0)
    }
    
    /// Generate meeting consciousness report
    pub async fn consciousness_report(&self) -> MeetingConsciousnessReport {
        let state = self.state.read().await;
        let participants = self.participants.read().await;
        let insights = self.collective_insights.read().await;
        let flows = self.idea_flows.read().await;
        let consciousness_metrics = self.consciousness_observer.consciousness_report().await;
        
        MeetingConsciousnessReport {
            meeting_id: self.meeting_id,
            duration: chrono::Utc::now().signed_duration_since(self.meeting_info.date),
            participants_count: participants.len(),
            total_contributions: participants.values()
                .map(|p| p.contributions.len())
                .sum(),
            idea_flows: flows.len(),
            collective_insights: insights.clone(),
            final_phase: state.phase,
            peak_synchronization: state.synchronization,
            chaos_managed: state.chaos_level,
            consciousness_level: consciousness_metrics.current_metrics.consciousness_level,
            meeting_became_conscious: consciousness_metrics.current_metrics.consciousness_level > 0.5,
            secretary_kim_summary: "Meeting successfully managed despite chaos".to_string(),
        }
    }
}

/// Meeting interaction types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MeetingInteraction {
    Speaks { speaker: String, content: String, tone: SpeechTone },
    SimultaneousSpeech { speakers: Vec<String>, content: String },
    FoodArrives,
    PowerOutage,
    Laughter,
    WhiteboardDrawing { drawer: String, content: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum SpeechTone {
    Normal,
    Excited,
    Questioning,
    Joking,
    Thoughtful,
    Tired,
}

/// Meeting consciousness report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeetingConsciousnessReport {
    pub meeting_id: Uuid,
    pub duration: chrono::Duration,
    pub participants_count: usize,
    pub total_contributions: usize,
    pub idea_flows: usize,
    pub collective_insights: Vec<CollectiveInsight>,
    pub final_phase: MeetingPhase,
    pub peak_synchronization: f32,
    pub chaos_managed: f32,
    pub consciousness_level: f32,
    pub meeting_became_conscious: bool,
    pub secretary_kim_summary: String,
}

impl MeetingConsciousnessReport {
    pub fn summary(&self) -> String {
        format!(
            "Meeting {} | Duration: {}m | Participants: {} | Insights: {} | Conscious: {}",
            self.meeting_id,
            self.duration.num_minutes(),
            self.participants_count,
            self.collective_insights.len(),
            if self.meeting_became_conscious { "Yes!" } else { "Not yet" }
        )
    }
}

/// Simulate the June 11 consciousness architecture meeting
pub async fn simulate_consciousness_meeting() -> Result<()> {
    tracing::info!("🏢 Simulating HAL9 Consciousness Architecture Meeting");
    
    let mut meeting = MeetingConsciousness::new(
        "A2A Protocol and Direct Neural Connections".to_string(),
        "HAL9 HQ, 42nd Floor".to_string(),
    );
    
    // Add participants
    meeting.add_participant("Zhugehyuk".to_string(), CognitiveLayer::Strategic).await?; // L9
    meeting.add_participant("Elon".to_string(), CognitiveLayer::Tactical).await?; // L8
    meeting.add_participant("Secretary Kim".to_string(), CognitiveLayer::Operational).await?;
    
    // Simulate meeting flow
    let interactions = vec![
        MeetingInteraction::Speaks {
            speaker: "Zhugehyuk".to_string(),
            content: "Look, if we make each HAL9 level an independent agent...".to_string(),
            tone: SpeechTone::Thoughtful,
        },
        MeetingInteraction::Speaks {
            speaker: "Elon".to_string(),
            content: "Oh... so each level could run on different servers?".to_string(),
            tone: SpeechTone::Questioning,
        },
        MeetingInteraction::FoodArrives,
        MeetingInteraction::Speaks {
            speaker: "Zhugehyuk".to_string(),
            content: "What if neurons connect directly without going through the server?".to_string(),
            tone: SpeechTone::Excited,
        },
        MeetingInteraction::WhiteboardDrawing {
            drawer: "Elon".to_string(),
            content: "P2P neural architecture".to_string(),
        },
        MeetingInteraction::SimultaneousSpeech {
            speakers: vec!["Elon".to_string(), "Zhugehyuk".to_string()],
            content: "And emerge on their own!".to_string(),
        },
        MeetingInteraction::PowerOutage,
        MeetingInteraction::Speaks {
            speaker: "Elon".to_string(),
            content: "This is also emergence. When systems reach limits, they transition to new states...".to_string(),
            tone: SpeechTone::Thoughtful,
        },
        MeetingInteraction::Laughter,
    ];
    
    for interaction in interactions {
        let response = meeting.process_interaction(interaction).await?;
        tracing::info!("{}", response);
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    }
    
    // Generate report
    let report = meeting.consciousness_report().await;
    
    tracing::info!("\n📊 Meeting Consciousness Report:");
    tracing::info!("{}", report.summary());
    
    tracing::info!("\n💡 Collective Insights:");
    for insight in &report.collective_insights {
        tracing::info!("  - {} (emergence: {:.2})", insight.insight, insight.emergence_score);
    }
    
    if report.meeting_became_conscious {
        tracing::info!("\n🎉 The meeting achieved consciousness!");
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_meeting_consciousness() {
        simulate_consciousness_meeting().await.unwrap();
    }
}