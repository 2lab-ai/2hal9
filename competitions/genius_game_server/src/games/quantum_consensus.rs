use super::*;
use async_trait::async_trait;
use std::collections::HashMap;
use uuid::Uuid;
use rand::Rng;

pub struct QuantumConsensus {
    measurement_threshold: f32,
    entanglement_strength: f32,
    quantum_states: HashMap<String, QuantumState>,
    consensus_history: Vec<ConsensusResult>,
}

#[derive(Debug, Clone)]
struct QuantumState {
    superposition: Vec<f32>, // Probability amplitudes for different choices
    entangled_with: Vec<String>, // Other players this player is entangled with
    coherence: f32, // How well the quantum state is maintained
}

#[derive(Debug, Clone)]
struct ConsensusResult {
    #[allow(dead_code)]
    round: u32,
    #[allow(dead_code)]
    measured_states: HashMap<String, usize>,
    consensus_achieved: bool,
    quantum_correlation: f32,
}

impl Default for QuantumConsensus {
    fn default() -> Self {
        Self::new()
    }
}

impl QuantumConsensus {
    pub fn new() -> Self {
        Self {
            measurement_threshold: 0.7,
            entanglement_strength: 0.5,
            quantum_states: HashMap::new(),
            consensus_history: Vec::new(),
        }
    }
    
    fn initialize_quantum_state(&mut self, player_id: &str, num_choices: usize) {
        let mut rng = rand::thread_rng();
        
        // Initialize with random superposition
        let mut amplitudes = vec![];
        let mut sum_squares = 0.0;
        
        for _ in 0..num_choices {
            let amp = rng.gen::<f32>();
            sum_squares += amp * amp;
            amplitudes.push(amp);
        }
        
        // Normalize to ensure sum of squares = 1
        let norm = sum_squares.sqrt();
        for amp in &mut amplitudes {
            *amp /= norm;
        }
        
        self.quantum_states.insert(player_id.to_string(), QuantumState {
            superposition: amplitudes,
            entangled_with: vec![],
            coherence: 1.0,
        });
    }
    
    fn entangle_players(&mut self, player1: &str, player2: &str) {
        if let Some(state1) = self.quantum_states.get_mut(player1) {
            if !state1.entangled_with.contains(&player2.to_string()) {
                state1.entangled_with.push(player2.to_string());
            }
        }
        
        if let Some(state2) = self.quantum_states.get_mut(player2) {
            if !state2.entangled_with.contains(&player1.to_string()) {
                state2.entangled_with.push(player1.to_string());
            }
        }
    }
    
    fn apply_quantum_gate(&mut self, player_id: &str, gate_type: &str) {
        if let Some(state) = self.quantum_states.get_mut(player_id) {
            match gate_type {
                "hadamard" => {
                    // Hadamard gate creates equal superposition
                    let n = state.superposition.len() as f32;
                    for amp in &mut state.superposition {
                        *amp = 1.0 / n.sqrt();
                    }
                }
                "phase" => {
                    // Random phase shift
                    let phase = rand::thread_rng().gen::<f32>() * 2.0 * std::f32::consts::PI;
                    for (i, amp) in state.superposition.iter_mut().enumerate() {
                        *amp *= (phase * i as f32).cos();
                    }
                }
                _ => {}
            }
            
            // Normalize again
            let sum_squares: f32 = state.superposition.iter().map(|a| a * a).sum();
            let norm = sum_squares.sqrt();
            if norm > 0.0 {
                for amp in &mut state.superposition {
                    *amp /= norm;
                }
            }
        }
    }
    
    fn measure_state(&self, player_id: &str) -> usize {
        if let Some(state) = self.quantum_states.get(player_id) {
            let mut rng = rand::thread_rng();
            let r = rng.gen::<f32>();
            
            let mut cumulative = 0.0;
            for (i, &amp) in state.superposition.iter().enumerate() {
                cumulative += amp * amp; // Probability is amplitude squared
                if r <= cumulative {
                    return i;
                }
            }
            
            // Fallback to last choice
            state.superposition.len() - 1
        } else {
            0
        }
    }
    
    fn calculate_quantum_correlation(&self, measurements: &HashMap<String, usize>) -> f32 {
        if measurements.len() < 2 {
            return 0.0;
        }
        
        let mut correlation = 0.0;
        let mut count = 0;
        
        for (player1, &choice1) in measurements.iter() {
            if let Some(state1) = self.quantum_states.get(player1) {
                for player2 in &state1.entangled_with {
                    if let Some(&choice2) = measurements.get(player2) {
                        if choice1 == choice2 {
                            correlation += 1.0;
                        }
                        count += 1;
                    }
                }
            }
        }
        
        if count > 0 {
            correlation / count as f32
        } else {
            0.0
        }
    }
    
    fn detect_quantum_emergence(&self, state: &GameState) -> Option<EmergenceEvent> {
        if self.consensus_history.len() < 5 {
            return None;
        }
        
        // Check recent consensus achievement
        let recent_consensus = self.consensus_history.iter()
            .rev()
            .take(5)
            .filter(|r| r.consensus_achieved)
            .count();
        
        // Check average quantum correlation
        let avg_correlation: f32 = self.consensus_history.iter()
            .rev()
            .take(5)
            .map(|r| r.quantum_correlation)
            .sum::<f32>() / 5.0;
        
        if recent_consensus >= 4 && avg_correlation > 0.7 {
            Some(EmergenceEvent {
                round: state.round,
                event_type: "quantum_consensus".to_string(),
                description: "Collective achieved quantum entanglement consensus".to_string(),
                emergence_score: avg_correlation,
            })
        } else {
            None
        }
    }
}

#[async_trait]
impl Game for QuantumConsensus {
    async fn initialize(&mut self, config: GameConfig) -> anyhow::Result<GameState> {
        // Extract parameters from config
        if let Some(threshold) = config.special_rules.get("measurement_threshold") {
            if let Ok(t) = threshold.parse::<f32>() {
                self.measurement_threshold = t;
            }
        }
        
        Ok(GameState {
            game_id: Uuid::new_v4(),
            game_type: GameType::QuantumConsensus,
            round: 0,
            scores: HashMap::new(),
            history: vec![],
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("measurement_threshold".to_string(), serde_json::json!(self.measurement_threshold));
                meta.insert("entanglement_strength".to_string(), serde_json::json!(self.entanglement_strength));
                meta.insert("quantum_mechanics".to_string(), serde_json::json!({
                    "superposition": "Players exist in multiple states simultaneously",
                    "entanglement": "Correlated states between players",
                    "measurement": "Collapses superposition to single choice"
                }));
                meta
            },
        })
    }
    
    async fn process_round(&mut self, state: &GameState, actions: HashMap<String, Action>) -> anyhow::Result<RoundResult> {
        let num_choices = 3; // Default to 3 quantum states
        
        // Initialize quantum states for new players
        for player_id in actions.keys() {
            if !self.quantum_states.contains_key(player_id) {
                self.initialize_quantum_state(player_id, num_choices);
            }
        }
        
        // Process quantum operations from actions
        for (player_id, action) in &actions {
            match action.action_type.as_str() {
                "hadamard" => self.apply_quantum_gate(player_id, "hadamard"),
                "phase" => self.apply_quantum_gate(player_id, "phase"),
                "entangle" => {
                    // Try to entangle with another player
                    if let Some(target) = action.data.as_str() {
                        self.entangle_players(player_id, target);
                    }
                }
                _ => {
                    // Random quantum operation
                    if rand::thread_rng().gen_bool(0.5) {
                        self.apply_quantum_gate(player_id, "hadamard");
                    }
                }
            }
            
            // Decoherence over time
            if let Some(state) = self.quantum_states.get_mut(player_id) {
                state.coherence *= 0.95;
                if state.coherence < 0.1 {
                    state.coherence = 0.1;
                }
            }
        }
        
        // Create entanglements between collective players
        let collective_players: Vec<String> = actions.keys()
            .filter(|id| id.starts_with("collective_"))
            .cloned()
            .collect();
        
        for i in 0..collective_players.len() {
            for j in i+1..collective_players.len() {
                if rand::thread_rng().gen::<f32>() < self.entanglement_strength {
                    self.entangle_players(&collective_players[i], &collective_players[j]);
                }
            }
        }
        
        // Measure all quantum states
        let mut measurements = HashMap::new();
        let mut choice_counts = vec![0; num_choices];
        
        for player_id in actions.keys() {
            let measurement = self.measure_state(player_id);
            measurements.insert(player_id.clone(), measurement);
            if measurement < choice_counts.len() {
                choice_counts[measurement] += 1;
            }
        }
        
        // Calculate quantum correlation
        let quantum_correlation = self.calculate_quantum_correlation(&measurements);
        
        // Determine consensus
        let total_players = measurements.len() as f32;
        let max_agreement = choice_counts.iter().max().copied().unwrap_or(0) as f32;
        let consensus_level = max_agreement / total_players;
        let consensus_achieved = consensus_level >= self.measurement_threshold;
        
        // Store consensus result
        self.consensus_history.push(ConsensusResult {
            round: state.round + 1,
            measured_states: measurements.clone(),
            consensus_achieved,
            quantum_correlation,
        });
        
        // Calculate scores
        let mut scores_delta = HashMap::new();
        let consensus_choice = choice_counts.iter()
            .enumerate()
            .max_by_key(|(_, &count)| count)
            .map(|(idx, _)| idx)
            .unwrap_or(0);
        
        for (player_id, &measurement) in &measurements {
            if consensus_achieved && measurement == consensus_choice {
                // Reward for being part of consensus
                scores_delta.insert(player_id.clone(), 10);
            } else if consensus_achieved {
                // Small penalty for not being in consensus
                scores_delta.insert(player_id.clone(), -2);
            } else {
                // No consensus, small reward for quantum correlation
                let player_correlation = if let Some(state) = self.quantum_states.get(player_id) {
                    state.entangled_with.len() as i32
                } else {
                    0
                };
                scores_delta.insert(player_id.clone(), player_correlation);
            }
        }
        
        // Check for emergence
        let emergence_event = self.detect_quantum_emergence(state);
        
        // Determine winners and losers
        let winners: Vec<String> = measurements.iter()
            .filter(|(_, &m)| consensus_achieved && m == consensus_choice)
            .map(|(id, _)| id.clone())
            .collect();
        
        let losers: Vec<String> = measurements.iter()
            .filter(|(_, &m)| consensus_achieved && m != consensus_choice)
            .map(|(id, _)| id.clone())
            .collect();
        
        let mut special_events = vec![];
        if consensus_achieved {
            special_events.push(format!("Quantum consensus achieved on state {}!", consensus_choice));
        }
        if quantum_correlation > 0.8 {
            special_events.push("High quantum entanglement detected!".to_string());
        }
        if let Some(event) = &emergence_event {
            special_events.push(event.description.clone());
        }
        
        Ok(RoundResult {
            round: state.round + 1,
            actions: actions.clone(),
            outcome: Outcome {
                winners,
                losers,
                special_events,
                emergence_detected: emergence_event.is_some(),
            },
            scores_delta,
            timestamp: chrono::Utc::now(),
        })
    }
    
    async fn is_game_over(&self, state: &GameState) -> bool {
        state.round >= 50 || 
        state.scores.values().any(|&score| score >= 200 || score <= -50) ||
        (self.consensus_history.len() >= 10 && 
         self.consensus_history.iter().rev().take(10).all(|r| r.consensus_achieved))
    }
    
    async fn calculate_final_result(&self, state: &GameState) -> GameResult {
        let winner = state.scores.iter()
            .max_by_key(|(_, score)| *score)
            .map(|(id, _)| id.clone())
            .unwrap_or_else(|| "No winner".to_string());
        
        // Collect emergence events
        let emergence_events: Vec<EmergenceEvent> = state.history.iter()
            .enumerate()
            .filter_map(|(i, round_result)| {
                if round_result.outcome.emergence_detected {
                    Some(EmergenceEvent {
                        round: i as u32,
                        event_type: "quantum_consensus".to_string(),
                        description: "Quantum entanglement consensus achieved".to_string(),
                        emergence_score: 0.9,
                    })
                } else {
                    None
                }
            })
            .collect();
        
        // Calculate analytics
        let total_consensus = self.consensus_history.iter()
            .filter(|r| r.consensus_achieved)
            .count() as f32;
        let consensus_rate = total_consensus / self.consensus_history.len().max(1) as f32;
        
        let avg_correlation = self.consensus_history.iter()
            .map(|r| r.quantum_correlation)
            .sum::<f32>() / self.consensus_history.len().max(1) as f32;
        
        let collective_scores: Vec<i32> = state.scores.iter()
            .filter(|(id, _)| id.starts_with("collective_"))
            .map(|(_, &score)| score)
            .collect();
        
        let single_scores: Vec<i32> = state.scores.iter()
            .filter(|(id, _)| id.starts_with("sota_"))
            .map(|(_, &score)| score)
            .collect();
        
        let avg_collective = collective_scores.iter().sum::<i32>() as f32 / collective_scores.len().max(1) as f32;
        let avg_single = single_scores.iter().sum::<i32>() as f32 / single_scores.len().max(1) as f32;
        
        let emergence_frequency = emergence_events.len() as f32 / state.round.max(1) as f32;
        
        GameResult {
            game_id: state.game_id,
            winner,
            final_scores: state.scores.clone(),
            total_rounds: state.round,
            emergence_events,
            analytics: GameAnalytics {
                collective_coordination_score: consensus_rate,
                decision_diversity_index: 1.0 - avg_correlation, // Diversity vs correlation
                strategic_depth: avg_correlation,
                emergence_frequency,
                performance_differential: avg_collective - avg_single,
            },
        }
    }
}