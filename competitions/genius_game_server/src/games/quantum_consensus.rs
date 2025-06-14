use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use super::{Game, GameState, PlayerAction, GameResult, EmergenceMetrics};
use anyhow::Result;

/// Quantum-inspired consensus game where decisions exist in superposition
pub struct QuantumConsensusGame {
    id: Uuid,
    round: u32,
    max_rounds: u32,
    players: Vec<String>,
    quantum_states: HashMap<String, QuantumState>,
    entanglements: Vec<Entanglement>,
    measurement_history: Vec<MeasurementResult>,
    special_rules: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct QuantumState {
    player_id: String,
    superposition: Vec<f32>, // Probability amplitudes for each basis state
    phase: f32,
    coherence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Entanglement {
    players: (String, String),
    strength: f32,
    correlation_type: EntanglementType,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
enum EntanglementType {
    Positive,  // Same measurement outcomes
    Negative,  // Opposite measurement outcomes
    Phase,     // Phase-correlated
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MeasurementResult {
    round: u32,
    collapsed_states: HashMap<String, usize>,
    consensus_achieved: bool,
    quantum_discord: f32,
}

impl QuantumConsensusGame {
    pub fn new(max_rounds: u32, special_rules: HashMap<String, String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            round: 0,
            max_rounds,
            players: Vec::new(),
            quantum_states: HashMap::new(),
            entanglements: Vec::new(),
            measurement_history: Vec::new(),
            special_rules,
        }
    }
    
    fn initialize_quantum_state(&self, player_id: &str) -> QuantumState {
        let num_basis_states = 4; // |00⟩, |01⟩, |10⟩, |11⟩
        let mut superposition = vec![0.0; num_basis_states];
        
        // Initialize in equal superposition
        for i in 0..num_basis_states {
            superposition[i] = 1.0 / (num_basis_states as f32).sqrt();
        }
        
        QuantumState {
            player_id: player_id.to_string(),
            superposition,
            phase: rand::random::<f32>() * 2.0 * std::f32::consts::PI,
            coherence: 1.0,
        }
    }
    
    fn apply_quantum_gate(&mut self, player: &str, gate_type: &str) {
        if let Some(state) = self.quantum_states.get_mut(player) {
            match gate_type {
                "hadamard" => {
                    // Apply Hadamard gate to create superposition
                    let old = state.superposition.clone();
                    state.superposition[0] = (old[0] + old[1]) / 2.0_f32.sqrt();
                    state.superposition[1] = (old[0] - old[1]) / 2.0_f32.sqrt();
                }
                "phase" => {
                    // Apply phase shift
                    state.phase += std::f32::consts::PI / 4.0;
                }
                "entangle" => {
                    // Entanglement is handled separately
                }
                _ => {}
            }
            
            // Normalize the state
            let norm: f32 = state.superposition.iter().map(|&x| x * x).sum::<f32>().sqrt();
            for amp in &mut state.superposition {
                *amp /= norm;
            }
        }
    }
    
    fn create_entanglement(&mut self, player1: &str, player2: &str) {
        let correlation = rand::random::<f32>();
        let ent_type = if correlation > 0.7 {
            EntanglementType::Positive
        } else if correlation > 0.4 {
            EntanglementType::Negative
        } else {
            EntanglementType::Phase
        };
        
        self.entanglements.push(Entanglement {
            players: (player1.to_string(), player2.to_string()),
            strength: correlation,
            correlation_type: ent_type,
        });
    }
    
    fn measure_states(&self) -> HashMap<String, usize> {
        let mut collapsed_states = HashMap::new();
        
        for (player, state) in &self.quantum_states {
            // Collapse the quantum state based on probability amplitudes
            let mut cumulative_prob = 0.0;
            let random_val = rand::random::<f32>();
            let mut collapsed_value = 0;
            
            for (i, &amplitude) in state.superposition.iter().enumerate() {
                cumulative_prob += amplitude * amplitude;
                if random_val <= cumulative_prob {
                    collapsed_value = i;
                    break;
                }
            }
            
            // Apply entanglement correlations
            for entanglement in &self.entanglements {
                if entanglement.players.0 == *player || entanglement.players.1 == *player {
                    let other_player = if entanglement.players.0 == *player {
                        &entanglement.players.1
                    } else {
                        &entanglement.players.0
                    };
                    
                    if let Some(&other_value) = collapsed_states.get(other_player) {
                        match entanglement.correlation_type {
                            EntanglementType::Positive => {
                                collapsed_value = other_value;
                            }
                            EntanglementType::Negative => {
                                collapsed_value = 3 - other_value; // Opposite in 2-bit system
                            }
                            EntanglementType::Phase => {
                                // Phase correlation doesn't affect measurement outcome
                            }
                        }
                    }
                }
            }
            
            collapsed_states.insert(player.clone(), collapsed_value);
        }
        
        collapsed_states
    }
    
    fn calculate_quantum_discord(&self, measurements: &HashMap<String, usize>) -> f32 {
        let values: Vec<usize> = measurements.values().copied().collect();
        if values.is_empty() {
            return 0.0;
        }
        
        // Calculate entropy
        let mut counts = HashMap::new();
        for &v in &values {
            *counts.entry(v).or_insert(0) += 1;
        }
        
        let total = values.len() as f32;
        let entropy: f32 = counts.values()
            .map(|&count| {
                let p = count as f32 / total;
                -p * p.log2()
            })
            .sum();
        
        // Normalize to [0, 1]
        entropy / 2.0 // Max entropy for 4 states is 2
    }
    
    fn detect_emergence(&self) -> EmergenceMetrics {
        if self.measurement_history.len() < 3 {
            return EmergenceMetrics {
                emergence_detected: false,
                coordination_score: 0.0,
                collective_intelligence_index: 0.0,
                decision_diversity_index: 0.0,
                strategic_depth: 0.0,
                special_patterns: HashMap::new(),
            };
        }
        
        // Check for quantum consensus patterns
        let recent = self.measurement_history.iter().rev().take(5).collect::<Vec<_>>();
        
        // Measure consensus achievement rate
        let consensus_rate = recent.iter()
            .filter(|r| r.consensus_achieved)
            .count() as f32 / recent.len() as f32;
        
        // Average quantum discord (lower means more order)
        let avg_discord: f32 = recent.iter()
            .map(|r| r.quantum_discord)
            .sum::<f32>() / recent.len() as f32;
        
        // Check entanglement network density
        let entanglement_density = self.entanglements.len() as f32 / 
            (self.players.len() * (self.players.len() - 1) / 2).max(1) as f32;
        
        // Average coherence
        let avg_coherence = self.quantum_states.values()
            .map(|s| s.coherence)
            .sum::<f32>() / self.quantum_states.len().max(1) as f32;
        
        // Emergence detected when:
        // 1. High consensus rate
        // 2. Low quantum discord (ordered states)
        // 3. Dense entanglement network
        // 4. High coherence maintained
        let emergence_detected = consensus_rate > 0.6 && 
                                avg_discord < 0.5 && 
                                entanglement_density > 0.5 &&
                                avg_coherence > 0.7;
        
        let mut special_patterns = HashMap::new();
        special_patterns.insert("consensus_rate".to_string(), consensus_rate.to_string());
        special_patterns.insert("quantum_discord".to_string(), avg_discord.to_string());
        special_patterns.insert("entanglement_density".to_string(), entanglement_density.to_string());
        special_patterns.insert("coherence".to_string(), avg_coherence.to_string());
        
        EmergenceMetrics {
            emergence_detected,
            coordination_score: consensus_rate,
            collective_intelligence_index: 1.0 - avg_discord,
            decision_diversity_index: avg_discord,
            strategic_depth: entanglement_density,
            special_patterns,
        }
    }
}

#[async_trait]
impl Game for QuantumConsensusGame {
    async fn get_state(&self) -> Result<GameState> {
        let available_choices = vec![
            "measure".to_string(),
            "hadamard".to_string(),
            "phase".to_string(),
            "entangle".to_string(),
        ];
        
        let mut player_states = HashMap::new();
        for player in &self.players {
            if let Some(qstate) = self.quantum_states.get(player) {
                player_states.insert(player.clone(), serde_json::json!({
                    "superposition": qstate.superposition,
                    "phase": qstate.phase,
                    "coherence": qstate.coherence,
                    "entangled_with": self.entanglements.iter()
                        .filter(|e| e.players.0 == *player || e.players.1 == *player)
                        .map(|e| if e.players.0 == *player { &e.players.1 } else { &e.players.0 })
                        .collect::<Vec<_>>(),
                }));
            }
        }
        
        Ok(GameState {
            game_id: self.id,
            game_type: "quantum_consensus".to_string(),
            round: self.round,
            players: self.players.clone(),
            current_state: serde_json::json!({
                "round": self.round,
                "max_rounds": self.max_rounds,
                "available_choices": available_choices,
                "entanglement_count": self.entanglements.len(),
                "last_measurement": self.measurement_history.last(),
            }),
            available_actions: available_choices,
            scores: HashMap::new(), // Scores based on consensus achievement
            player_states,
            is_complete: self.round >= self.max_rounds,
            special_data: Some(serde_json::json!({
                "quantum_mechanics": {
                    "basis_states": ["00", "01", "10", "11"],
                    "gates_available": ["hadamard", "phase", "entangle"],
                },
            })),
        })
    }
    
    async fn process_action(&mut self, action: PlayerAction) -> Result<()> {
        match action.action_type.as_str() {
            "hadamard" | "phase" => {
                self.apply_quantum_gate(&action.player_id, &action.action_type);
            }
            "entangle" => {
                // Randomly entangle with another player
                if let Some(other) = self.players.iter()
                    .find(|&p| p != &action.player_id && rand::random::<f32>() > 0.5) {
                    self.create_entanglement(&action.player_id, other);
                }
            }
            _ => {}
        }
        
        // Decoherence over time
        if let Some(state) = self.quantum_states.get_mut(&action.player_id) {
            state.coherence *= 0.95;
        }
        
        Ok(())
    }
    
    async fn advance_round(&mut self) -> Result<GameResult> {
        self.round += 1;
        
        // Perform quantum measurement
        let collapsed_states = self.measure_states();
        
        // Check for consensus
        let values: Vec<_> = collapsed_states.values().copied().collect();
        let consensus_achieved = values.windows(2).all(|w| w[0] == w[1]);
        
        let quantum_discord = self.calculate_quantum_discord(&collapsed_states);
        
        let measurement = MeasurementResult {
            round: self.round,
            collapsed_states: collapsed_states.clone(),
            consensus_achieved,
            quantum_discord,
        };
        
        self.measurement_history.push(measurement);
        
        // Update scores based on consensus participation
        let mut scores = HashMap::new();
        if consensus_achieved {
            for player in &self.players {
                scores.insert(player.clone(), 10);
            }
        } else {
            // Partial credit based on alignment
            for (player, &value) in &collapsed_states {
                let same_count = collapsed_states.values()
                    .filter(|&&v| v == value)
                    .count();
                scores.insert(player.clone(), same_count as i32);
            }
        }
        
        let emergence = self.detect_emergence();
        
        Ok(GameResult {
            round: self.round,
            scores,
            round_winners: if consensus_achieved { self.players.clone() } else { vec![] },
            is_final_round: self.round >= self.max_rounds,
            emergence_metrics: emergence,
            special_data: Some(serde_json::json!({
                "collapsed_states": collapsed_states,
                "consensus_achieved": consensus_achieved,
                "quantum_discord": quantum_discord,
            })),
        })
    }
    
    fn add_player(&mut self, player_id: String) -> Result<()> {
        if !self.players.contains(&player_id) {
            self.players.push(player_id.clone());
            let qstate = self.initialize_quantum_state(&player_id);
            self.quantum_states.insert(player_id, qstate);
        }
        Ok(())
    }
    
    fn get_game_id(&self) -> Uuid {
        self.id
    }
}