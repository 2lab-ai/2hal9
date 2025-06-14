use super::*;
use async_trait::async_trait;
use std::collections::HashMap;
use uuid::Uuid;
use rand::Rng;

pub struct RecursiveReasoning {
    pub max_depth: usize,
    pub guessing_rounds: usize,
    pub thinking_depth_scores: HashMap<String, Vec<usize>>,
}

impl Default for RecursiveReasoning {
    fn default() -> Self {
        Self::new()
    }
}

impl RecursiveReasoning {
    pub fn new() -> Self {
        Self {
            max_depth: 5,
            guessing_rounds: 10,
            thinking_depth_scores: HashMap::new(),
        }
    }
    
    pub fn calculate_reasoning_depth(&self, guess: i32, target: i32) -> usize {
        // Calculate how deep the recursive reasoning goes based on the guess
        let distance = (guess - target).abs() as f64;
        let max_distance = 100.0; // Maximum possible distance
        
        // Closer guesses indicate deeper reasoning
        let depth_ratio = 1.0 - (distance / max_distance).min(1.0);
        let calculated_depth = (depth_ratio * self.max_depth as f64).round() as usize;
        
        // Special case: perfect recursive reasoning (guess = 0)
        if guess == 0 {
            return self.max_depth;
        }
        
        calculated_depth.max(1)
    }
    
    fn generate_puzzle(&self, round: u32) -> RecursivePuzzle {
        let mut rng = rand::thread_rng();
        
        // Create nested reasoning puzzle
        let base_number = rng.gen_range(1..=100);
        let depth = ((round / 3) as usize).min(self.max_depth - 1) + 1;
        
        RecursivePuzzle {
            description: "I'm thinking of a number. What do you think I think you'll guess?".to_string(),
            base_value: base_number,
            depth,
            optimal_answer: self.calculate_optimal_answer(base_number, depth),
        }
    }
    
    fn calculate_optimal_answer(&self, base: i32, depth: usize) -> i32 {
        // Recursive reasoning: each level alternates between cooperation and competition
        let mut answer = base;
        for level in 0..depth {
            if level % 2 == 0 {
                // Even levels: think opponent will go higher
                answer = (answer as f32 * 1.1) as i32;
            } else {
                // Odd levels: think opponent will go lower
                answer = (answer as f32 * 0.9) as i32;
            }
        }
        answer
    }
    
    fn score_guess(&self, guess: i32, optimal: i32, depth: usize) -> i32 {
        let distance = (guess - optimal).abs();
        let base_score = 100 - distance.min(100);
        
        // Bonus for deeper reasoning
        let depth_bonus = depth as i32 * 10;
        
        base_score + depth_bonus
    }
    
    fn detect_meta_reasoning(&self, guesses: &HashMap<String, i32>, puzzle: &RecursivePuzzle) -> bool {
        // Check if collective agents are modeling each other's reasoning
        let collective_guesses: Vec<_> = guesses.iter()
            .filter(|(id, _)| id.starts_with("collective_"))
            .map(|(_, &guess)| guess)
            .collect();
        
        if collective_guesses.len() < 3 {
            return false;
        }
        
        // Check for clustering around meta-levels
        let mut depth_clusters = vec![0; self.max_depth];
        for &guess in &collective_guesses {
            for depth in 1..=self.max_depth {
                let expected = self.calculate_optimal_answer(puzzle.base_value, depth);
                if (guess - expected).abs() < 5 {
                    depth_clusters[depth - 1] += 1;
                }
            }
        }
        
        // Emergence if agents are reasoning at different depths
        let active_depths = depth_clusters.iter().filter(|&&c| c > 0).count();
        active_depths >= 3
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct RecursivePuzzle {
    description: String,
    base_value: i32,
    depth: usize,
    optimal_answer: i32,
}

#[async_trait]
impl Game for RecursiveReasoning {
    async fn initialize(&mut self, _config: GameConfig) -> anyhow::Result<GameState> {
        Ok(GameState {
            game_id: Uuid::new_v4(),
            game_type: GameType::RecursiveReasoning,
            round: 0,
            scores: HashMap::new(),
            history: vec![],
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("max_depth".to_string(), serde_json::json!(self.max_depth));
                meta.insert("current_depth".to_string(), serde_json::json!(1));
                meta
            },
        })
    }
    
    async fn process_round(&mut self, state: &GameState, actions: HashMap<String, Action>) -> anyhow::Result<RoundResult> {
        let puzzle = self.generate_puzzle(state.round);
        let mut scores_delta = HashMap::new();
        let mut guesses = HashMap::new();
        
        // Process guesses
        for (player_id, action) in &actions {
            if let Some(guess) = action.data.get("guess").and_then(|g| g.as_i64()) {
                let guess = guess as i32;
                guesses.insert(player_id.clone(), guess);
                
                let score = self.score_guess(guess, puzzle.optimal_answer, puzzle.depth);
                scores_delta.insert(player_id.clone(), score);
                
                // Track thinking depth
                self.thinking_depth_scores
                    .entry(player_id.clone())
                    .or_default()
                    .push(puzzle.depth);
            }
        }
        
        // Check for meta-reasoning emergence
        let emergence_detected = self.detect_meta_reasoning(&guesses, &puzzle);
        
        // Determine winners (closest to optimal)
        let best_score = scores_delta.values().max().cloned().unwrap_or(0);
        let winners: Vec<String> = scores_delta.iter()
            .filter(|(_, &score)| score == best_score)
            .map(|(id, _)| id.clone())
            .collect();
        
        let special_events = if emergence_detected {
            vec!["Meta-reasoning emergence detected!".to_string()]
        } else {
            vec![]
        };
        
        Ok(RoundResult {
            round: state.round + 1,
            actions: actions.clone(),
            outcome: Outcome {
                winners,
                losers: vec![],
                special_events,
                emergence_detected,
            },
            scores_delta,
            timestamp: chrono::Utc::now(),
        })
    }
    
    async fn is_game_over(&self, state: &GameState) -> bool {
        state.round >= self.guessing_rounds as u32 ||
        state.scores.values().any(|&score| score >= 1000)
    }
    
    async fn calculate_final_result(&self, state: &GameState) -> GameResult {
        let winner = state.scores.iter()
            .max_by_key(|(_, score)| *score)
            .map(|(id, _)| id.clone())
            .unwrap_or_else(|| "No winner".to_string());
        
        // Count emergence events
        let emergence_events: Vec<EmergenceEvent> = state.history.iter()
            .enumerate()
            .filter_map(|(i, round_result)| {
                if round_result.outcome.emergence_detected {
                    Some(EmergenceEvent {
                        round: i as u32,
                        event_type: "meta_reasoning".to_string(),
                        description: "Collective demonstrated multi-level recursive reasoning".to_string(),
                        emergence_score: 0.85,
                    })
                } else {
                    None
                }
            })
            .collect();
        
        let emergence_frequency = emergence_events.len() as f32 / state.round.max(1) as f32;
        
        // Calculate average thinking depth
        let avg_depth: f32 = self.thinking_depth_scores.values()
            .flat_map(|v| v.iter())
            .map(|&d| d as f32)
            .sum::<f32>() / self.thinking_depth_scores.values()
            .map(|v| v.len())
            .sum::<usize>().max(1) as f32;
        
        GameResult {
            game_id: state.game_id,
            winner,
            final_scores: state.scores.clone(),
            total_rounds: state.round,
            emergence_events,
            analytics: GameAnalytics {
                collective_coordination_score: emergence_frequency,
                decision_diversity_index: 0.0,
                strategic_depth: avg_depth / self.max_depth as f32,
                emergence_frequency,
                performance_differential: 0.0,
            },
        }
    }
}