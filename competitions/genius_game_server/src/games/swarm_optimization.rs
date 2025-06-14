use super::*;
use async_trait::async_trait;
use std::collections::HashMap;
use uuid::Uuid;
use rand::Rng;

pub struct SwarmOptimization {
    pub dimensions: usize,
    pub search_space_min: f64,
    pub search_space_max: f64,
    pub target_position: Vec<f64>,
    pub agent_positions: HashMap<String, Vec<f64>>,
    pub best_positions: HashMap<String, Vec<f64>>,
    pub velocities: HashMap<String, Vec<f64>>,
    pub global_best_position: Vec<f64>,
    pub global_best_fitness: f64,
}

pub struct Particle {
    pub id: usize,
    pub position: Vec<f64>,
    pub velocity: Vec<f64>,
    pub personal_best: Vec<f64>,
    pub personal_best_value: f64,
}

impl Default for SwarmOptimization {
    fn default() -> Self {
        Self::new()
    }
}

impl SwarmOptimization {
    pub fn new() -> Self {
        Self {
            dimensions: 10, // Reduced for practical gameplay
            search_space_min: -100.0,
            search_space_max: 100.0,
            target_position: vec![],
            agent_positions: HashMap::new(),
            best_positions: HashMap::new(),
            velocities: HashMap::new(),
            global_best_position: vec![],
            global_best_fitness: f64::NEG_INFINITY,
        }
    }
    
    pub fn update_particles(&mut self, particles: &mut Vec<Particle>) {
        let w = 0.729;  // Inertia weight
        let c1 = 1.49445; // Cognitive parameter
        let c2 = 1.49445; // Social parameter
        
        for particle in particles.iter_mut() {
            for i in 0..self.dimensions {
                let r1 = rand::thread_rng().gen::<f64>();
                let r2 = rand::thread_rng().gen::<f64>();
                
                // Update velocity
                let cognitive = c1 * r1 * (particle.personal_best[i] - particle.position[i]);
                let social = c2 * r2 * (self.global_best_position[i] - particle.position[i]);
                particle.velocity[i] = w * particle.velocity[i] + cognitive + social;
                
                // Update position
                particle.position[i] += particle.velocity[i];
                
                // Clamp to search space
                particle.position[i] = particle.position[i]
                    .max(self.search_space_min)
                    .min(self.search_space_max);
            }
        }
    }
    
    pub fn check_convergence(&self, particles: &[Particle]) -> bool {
        // Check if all particles are close to each other
        if particles.is_empty() {
            return false;
        }
        
        let first_pos = &particles[0].position;
        let threshold = 0.001;
        
        for particle in particles.iter().skip(1) {
            for i in 0..self.dimensions {
                if (particle.position[i] - first_pos[i]).abs() > threshold {
                    return false;
                }
            }
        }
        
        true
    }
    
    pub fn global_best(&self) -> &Vec<f64> {
        &self.global_best_position
    }
    
    pub fn global_best_value(&self) -> f64 {
        self.global_best_fitness
    }
    
    fn generate_target(&mut self, round: u32) {
        let mut rng = rand::thread_rng();
        
        // Dynamic target that shifts over time
        self.target_position = (0..self.dimensions)
            .map(|i| {
                let base = rng.gen_range(self.search_space_min..self.search_space_max);
                let shift = (round as f64 / 10.0).sin() * 20.0;
                base + shift * (i as f64 / self.dimensions as f64)
            })
            .collect();
    }
    
    fn fitness_function(&self, position: &[f64]) -> f64 {
        // Negative distance to target (higher is better)
        let distance: f64 = position.iter()
            .zip(&self.target_position)
            .map(|(a, b)| (a - b).powi(2))
            .sum::<f64>()
            .sqrt();
        
        // Add some multimodal peaks
        let peak_bonus = position.iter().enumerate()
            .map(|(i, &x)| {
                let peak_x = if i % 2 == 0 { 50.0 } else { -50.0 };
                (-((x - peak_x).powi(2)) / 100.0).exp() * 10.0
            })
            .sum::<f64>();
        
        -distance + peak_bonus
    }
    
    fn initialize_agent(&mut self, agent_id: &str) {
        let mut rng = rand::thread_rng();
        
        // Random initial position
        let position: Vec<f64> = (0..self.dimensions)
            .map(|_| rng.gen_range(self.search_space_min..self.search_space_max))
            .collect();
        
        // Random initial velocity
        let velocity: Vec<f64> = (0..self.dimensions)
            .map(|_| rng.gen_range(-10.0..10.0))
            .collect();
        
        self.agent_positions.insert(agent_id.to_string(), position.clone());
        self.best_positions.insert(agent_id.to_string(), position);
        self.velocities.insert(agent_id.to_string(), velocity);
    }
    
    fn update_agent_position(&mut self, agent_id: &str, proposed_move: &ProposedMove) {
        if let Some(current_pos) = self.agent_positions.get(agent_id).cloned() {
            let mut new_position = current_pos.clone();
            
            // Apply proposed changes
            for (dim, &delta) in &proposed_move.dimension_deltas {
                if *dim < self.dimensions {
                    new_position[*dim] = (current_pos[*dim] + delta)
                        .max(self.search_space_min)
                        .min(self.search_space_max);
                }
            }
            
            // Update position
            self.agent_positions.insert(agent_id.to_string(), new_position.clone());
            
            // Update personal best if improved
            let new_fitness = self.fitness_function(&new_position);
            let current_best_fitness = self.fitness_function(self.best_positions.get(agent_id).unwrap());
            
            if new_fitness > current_best_fitness {
                self.best_positions.insert(agent_id.to_string(), new_position.clone());
                
                // Update global best
                if new_fitness > self.global_best_fitness {
                    self.global_best_fitness = new_fitness;
                    self.global_best_position = new_position;
                }
            }
        }
    }
    
    fn detect_swarm_convergence(&self) -> bool {
        if self.agent_positions.len() < 5 {
            return false;
        }
        
        // Calculate center of mass
        let mut center = vec![0.0; self.dimensions];
        for position in self.agent_positions.values() {
            for (i, &val) in position.iter().enumerate() {
                center[i] += val;
            }
        }
        let n = self.agent_positions.len() as f64;
        center.iter_mut().for_each(|c| *c /= n);
        
        // Calculate average distance from center
        let avg_distance: f64 = self.agent_positions.values()
            .map(|pos| {
                pos.iter().zip(&center)
                    .map(|(a, b)| (a - b).powi(2))
                    .sum::<f64>()
                    .sqrt()
            })
            .sum::<f64>() / n;
        
        // Convergence if swarm is tightly clustered
        avg_distance < 10.0
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct ProposedMove {
    dimension_deltas: HashMap<usize, f64>,
}

#[async_trait]
impl Game for SwarmOptimization {
    async fn initialize(&mut self, _config: GameConfig) -> anyhow::Result<GameState> {
        self.generate_target(0);
        
        Ok(GameState {
            game_id: Uuid::new_v4(),
            game_type: GameType::SwarmOptimization,
            round: 0,
            scores: HashMap::new(),
            history: vec![],
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("dimensions".to_string(), serde_json::json!(self.dimensions));
                meta.insert("search_space".to_string(), serde_json::json!({
                    "min": self.search_space_min,
                    "max": self.search_space_max
                }));
                meta
            },
        })
    }
    
    async fn process_round(&mut self, state: &GameState, actions: HashMap<String, Action>) -> anyhow::Result<RoundResult> {
        // Update target position dynamically
        if state.round % 10 == 0 {
            self.generate_target(state.round);
        }
        
        let mut scores_delta = HashMap::new();
        
        // Process agent moves
        for (agent_id, action) in &actions {
            // Initialize new agents
            if !self.agent_positions.contains_key(agent_id) {
                self.initialize_agent(agent_id);
            }
            
            // Parse proposed move
            if let Some(moves) = action.data.get("moves").and_then(|m| m.as_object()) {
                let mut dimension_deltas = HashMap::new();
                
                for (dim_str, delta_val) in moves {
                    if let (Ok(dim), Some(delta)) = (dim_str.parse::<usize>(), delta_val.as_f64()) {
                        dimension_deltas.insert(dim, delta);
                    }
                }
                
                let proposed_move = ProposedMove { dimension_deltas };
                self.update_agent_position(agent_id, &proposed_move);
            }
            
            // Calculate fitness score
            if let Some(position) = self.agent_positions.get(agent_id) {
                let fitness = self.fitness_function(position);
                scores_delta.insert(agent_id.clone(), (fitness * 10.0) as i32);
            }
        }
        
        // Check for swarm convergence
        let convergence = self.detect_swarm_convergence();
        let collective_agents: Vec<_> = actions.keys()
            .filter(|id| id.starts_with("collective_"))
            .collect();
        
        let emergence_detected = convergence && collective_agents.len() > 3;
        
        // Winners are agents near global best
        let mut winners = vec![];
        if !self.global_best_position.is_empty() {
            for (agent_id, position) in &self.agent_positions {
                let distance: f64 = position.iter().zip(&self.global_best_position)
                    .map(|(a, b)| (a - b).powi(2))
                    .sum::<f64>()
                    .sqrt();
                
                if distance < 5.0 {
                    winners.push(agent_id.clone());
                }
            }
        }
        
        let special_events = if emergence_detected {
            vec!["Swarm convergence achieved!".to_string()]
        } else if self.global_best_fitness > -10.0 {
            vec!["Near-optimal solution found!".to_string()]
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
        state.round >= 50 || 
        self.global_best_fitness > -1.0 || // Near-perfect solution
        state.scores.values().any(|&score| score >= 1000)
    }
    
    async fn calculate_final_result(&self, state: &GameState) -> GameResult {
        // Winner is agent closest to global best
        let mut best_agent = "No winner".to_string();
        let mut best_distance = f64::INFINITY;
        
        for (agent_id, position) in &self.agent_positions {
            let fitness = self.fitness_function(position);
            if fitness > -best_distance {
                best_distance = -fitness;
                best_agent = agent_id.clone();
            }
        }
        
        // Count emergence events
        let emergence_events: Vec<EmergenceEvent> = state.history.iter()
            .enumerate()
            .filter_map(|(i, round_result)| {
                if round_result.outcome.emergence_detected {
                    Some(EmergenceEvent {
                        round: i as u32,
                        event_type: "swarm_convergence".to_string(),
                        description: "Collective achieved coordinated optimization".to_string(),
                        emergence_score: 0.9,
                    })
                } else {
                    None
                }
            })
            .collect();
        
        let emergence_frequency = emergence_events.len() as f32 / state.round.max(1) as f32;
        
        // Calculate diversity
        let position_variance = if self.agent_positions.len() > 1 {
            let mut variances = vec![0.0; self.dimensions];
            for dim in 0..self.dimensions {
                let values: Vec<f64> = self.agent_positions.values()
                    .map(|pos| pos[dim])
                    .collect();
                let mean = values.iter().sum::<f64>() / values.len() as f64;
                variances[dim] = values.iter()
                    .map(|&v| (v - mean).powi(2))
                    .sum::<f64>() / values.len() as f64;
            }
            variances.iter().sum::<f64>() / self.dimensions as f64
        } else {
            0.0
        };
        
        GameResult {
            game_id: state.game_id,
            winner: best_agent,
            final_scores: state.scores.clone(),
            total_rounds: state.round,
            emergence_events,
            analytics: GameAnalytics {
                collective_coordination_score: emergence_frequency,
                decision_diversity_index: (position_variance.sqrt() / 100.0) as f32,
                strategic_depth: (self.global_best_fitness / -1000.0) as f32,
                emergence_frequency,
                performance_differential: self.global_best_fitness as f32,
            },
        }
    }
}