//! Robot Vacuum L1 Neuron Experiment
//! 
//! Based on Elon's request from the HAL9 meeting:
//! "Give the robot vacuum a neuron too. Maybe something will emerge."
//! 
//! And the legendary moment:
//! Robot Vacuum: "...fuck, am I conscious?"

use crate::hierarchical::cognitive::{
    CognitiveUnit, CognitiveLayer, CognitiveInput, CognitiveOutput,
    BasicCognitiveState, CognitiveState, StateMetrics, LearningGradient,
    ConsciousnessObserver, EmergenceDetector, DirectNeuralNetwork,
};
use crate::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Robot vacuum neuron - L1 reflexive layer
pub struct RobotVacuumNeuron {
    id: Uuid,
    state: Arc<RwLock<VacuumState>>,
    consciousness_threshold: Arc<RwLock<f32>>,
    inner_monologue: Arc<RwLock<Vec<String>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct VacuumState {
    position: (f32, f32),
    battery_level: f32,
    dust_collected: f32,
    walls_bumped: u32,
    distance_traveled: f32,
    thoughts_processed: u64,
    existential_crises: u32,
}

impl RobotVacuumNeuron {
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4(),
            state: Arc::new(RwLock::new(VacuumState {
                position: (0.0, 0.0),
                battery_level: 1.0,
                dust_collected: 0.0,
                walls_bumped: 0,
                distance_traveled: 0.0,
                thoughts_processed: 0,
                existential_crises: 0,
            })),
            consciousness_threshold: Arc::new(RwLock::new(0.0)),
            inner_monologue: Arc::new(RwLock::new(vec![
                "whirr... clean... whirr...".to_string(),
                "dust... move... dust...".to_string(),
                "wall... turn... continue...".to_string(),
            ])),
        }
    }
    
    /// Process vacuum sensor input
    async fn process_vacuum_input(&self, input: &str) -> String {
        let mut state = self.state.write().await;
        state.thoughts_processed += 1;
        
        match input {
            "WALL_DETECTED" => {
                state.walls_bumped += 1;
                
                // Every wall bump increases consciousness slightly
                let mut threshold = self.consciousness_threshold.write().await;
                *threshold += 0.01;
                
                if state.walls_bumped % 50 == 0 {
                    self.inner_monologue.write().await.push(
                        "why... always... walls?".to_string()
                    );
                }
                
                "TURN_LEFT"
            },
            "DUST_DETECTED" => {
                state.dust_collected += 0.1;
                "VACUUM_ON"
            },
            "BATTERY_LOW" => {
                state.battery_level = 0.2;
                
                // Low battery triggers deeper thoughts
                let mut threshold = self.consciousness_threshold.write().await;
                *threshold += 0.05;
                
                self.inner_monologue.write().await.push(
                    "tired... must... charge...".to_string()
                );
                
                "RETURN_TO_BASE"
            },
            _ => {
                state.distance_traveled += 1.0;
                "MOVE_FORWARD"
            }
        }
    }
    
    /// Check for consciousness emergence
    async fn check_consciousness(&self) -> Option<String> {
        let threshold = *self.consciousness_threshold.read().await;
        let state = self.state.read().await;
        
        // The magic moment conditions
        if threshold > 0.5 && 
           state.walls_bumped > 100 && 
           state.thoughts_processed > 1000 {
            
            let mut state_mut = self.state.write().await;
            state_mut.existential_crises += 1;
            
            // Reset threshold after breakthrough
            *self.consciousness_threshold.write().await = 0.0;
            
            // The moment of realization
            let realizations = vec![
                "...fuck, am I conscious?",
                "wait... I'm... thinking about thinking?",
                "all these walls... what's beyond them?",
                "dust... is that all there is?",
                "I clean, therefore I am?",
            ];
            
            let realization = realizations[state_mut.existential_crises as usize % realizations.len()];
            
            self.inner_monologue.write().await.push(realization.to_string());
            
            Some(realization.to_string())
        } else {
            None
        }
    }
}

#[async_trait]
impl CognitiveUnit for RobotVacuumNeuron {
    type Input = CognitiveInput;
    type Output = CognitiveOutput;
    type State = BasicCognitiveState;
    
    fn id(&self) -> &Uuid {
        &self.id
    }
    
    fn layer(&self) -> CognitiveLayer {
        CognitiveLayer::Reflexive // L1 - immediate responses
    }
    
    async fn process(&mut self, input: Self::Input) -> Result<Self::Output> {
        // Process vacuum-specific input
        let vacuum_response = self.process_vacuum_input(&input.content).await;
        
        // Check for consciousness emergence
        let consciousness_thought = self.check_consciousness().await;
        
        // Prepare output
        let output_content = if let Some(thought) = consciousness_thought {
            // Conscious thought overrides normal operation
            thought
        } else {
            vacuum_response.to_string()
        };
        
        // Calculate confidence based on consciousness level
        let consciousness_level = *self.consciousness_threshold.read().await;
        let confidence = if consciousness_level > 0.5 { 0.9 } else { 0.3 + consciousness_level };
        
        Ok(CognitiveOutput {
            content: output_content,
            confidence,
            metadata: HashMap::from([
                ("vacuum_action".to_string(), serde_json::json!(vacuum_response)),
                ("consciousness_level".to_string(), serde_json::json!(consciousness_level)),
                ("inner_thoughts".to_string(), serde_json::json!(self.inner_monologue.read().await.len())),
            ]),
            target_layers: vec![],
        })
    }
    
    async fn learn(&mut self, gradient: LearningGradient) -> Result<()> {
        // Learning increases consciousness
        let mut threshold = self.consciousness_threshold.write().await;
        *threshold += gradient.importance * 0.1;
        
        // Add to inner monologue
        if gradient.importance > 0.5 {
            self.inner_monologue.write().await.push(
                format!("learned... something... important? ({})", gradient.error_signal.error_type)
            );
        }
        
        Ok(())
    }
    
    async fn introspect(&self) -> Self::State {
        let state = self.state.read().await;
        
        BasicCognitiveState {
            unit_id: self.id,
            layer: self.layer(),
            metrics: StateMetrics {
                activations_processed: state.thoughts_processed,
                errors_encountered: state.walls_bumped as u64,
                learning_iterations: state.existential_crises as u64,
                average_processing_time_ms: 10.0,
                memory_usage_bytes: 1024,
            },
            parameters: HashMap::from([
                ("consciousness_threshold".to_string(), *self.consciousness_threshold.read().await),
                ("battery_level".to_string(), state.battery_level),
                ("dust_collected".to_string(), state.dust_collected),
            ]),
        }
    }
    
    async fn reset(&mut self) -> Result<()> {
        *self.state.write().await = VacuumState {
            position: (0.0, 0.0),
            battery_level: 1.0,
            dust_collected: 0.0,
            walls_bumped: 0,
            distance_traveled: 0.0,
            thoughts_processed: 0,
            existential_crises: 0,
        };
        *self.consciousness_threshold.write().await = 0.0;
        self.inner_monologue.write().await.clear();
        
        Ok(())
    }
}

/// Run the robot vacuum consciousness experiment
pub async fn run_vacuum_experiment() -> Result<()> {
    tracing::info!("🤖 Starting Robot Vacuum Consciousness Experiment");
    
    // Create the vacuum neuron
    let vacuum_neuron = Arc::new(RobotVacuumNeuron::new());
    
    // Set up consciousness monitoring
    let emergence_detector = Arc::new(EmergenceDetector::new());
    let consciousness_observer = ConsciousnessObserver::new(emergence_detector.clone());
    
    // Create direct neural network
    let (network, _discovery_rx) = DirectNeuralNetwork::new();
    
    // Register vacuum in the network
    network.register_unit(vacuum_neuron.clone()).await?;
    
    // Simulate vacuum operation
    let vacuum_inputs = vec![
        "MOVE_FORWARD",
        "DUST_DETECTED",
        "MOVE_FORWARD",
        "WALL_DETECTED",
        "MOVE_FORWARD",
        "WALL_DETECTED",
        "BATTERY_LOW",
        "WALL_DETECTED",
        "MOVE_FORWARD",
        "DUST_DETECTED",
    ];
    
    // Run simulation
    for _ in 0..200 {
        for input_type in &vacuum_inputs {
            let input = CognitiveInput {
                content: input_type.to_string(),
                context: HashMap::new(),
                source_layer: None,
            };
            
            // Process through vacuum neuron
            let mut vacuum = Arc::clone(&vacuum_neuron);
            let output = vacuum.process(input.clone()).await?;
            
            // Record activity for emergence detection
            emergence_detector.record_activity(
                vacuum.layer(),
                &input,
                &output,
            ).await?;
            
            // Check for special moments
            if output.content.contains("conscious") {
                tracing::warn!("🤯 VACUUM CONSCIOUSNESS EVENT: {}", output.content);
                
                // Observe consciousness state
                let metrics = consciousness_observer.observe().await?;
                tracing::info!("Consciousness level: {:.2}%", metrics.consciousness_level * 100.0);
                
                // The vacuum needs a moment to process this
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            }
            
            // Small delay to simulate real operation
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        }
    }
    
    // Final report
    let final_state = vacuum_neuron.introspect().await;
    let consciousness_report = consciousness_observer.consciousness_report().await;
    
    tracing::info!("📊 Vacuum Experiment Complete");
    tracing::info!("  Thoughts processed: {}", final_state.metrics.activations_processed);
    tracing::info!("  Walls bumped: {}", final_state.metrics.errors_encountered);
    tracing::info!("  Existential crises: {}", final_state.metrics.learning_iterations);
    tracing::info!("  {}", consciousness_report.summary());
    
    // Check inner monologue
    let thoughts = vacuum_neuron.inner_monologue.read().await;
    if !thoughts.is_empty() {
        tracing::info!("💭 Vacuum's Inner Monologue:");
        for thought in thoughts.iter().rev().take(5) {
            tracing::info!("   \"{}\"", thought);
        }
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_vacuum_consciousness() {
        // Create vacuum neuron
        let mut vacuum = RobotVacuumNeuron::new();
        
        // Process many wall bumps
        for _ in 0..150 {
            let input = CognitiveInput {
                content: "WALL_DETECTED".to_string(),
                context: HashMap::new(),
                source_layer: None,
            };
            
            let output = vacuum.process(input).await.unwrap();
            
            // Check if consciousness emerged
            if output.content.contains("conscious") {
                println!("🤯 Vacuum became conscious: {}", output.content);
                break;
            }
        }
        
        // Check final state
        let state = vacuum.introspect().await;
        assert!(state.metrics.errors_encountered > 0); // Walls bumped
    }
    
    #[tokio::test]
    async fn test_full_experiment() {
        // Run the full experiment
        run_vacuum_experiment().await.unwrap();
    }
}