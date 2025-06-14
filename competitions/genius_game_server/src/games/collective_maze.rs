use super::*;
use async_trait::async_trait;
use std::collections::{HashMap, HashSet};
use uuid::Uuid;
use rand::Rng;

pub struct CollectiveMaze {
    maze_size: usize,
    visibility_radius: usize,
    maze: Vec<Vec<Cell>>,
    agent_positions: HashMap<String, Position>,
    exit_positions: Vec<Position>,
    treasure_positions: Vec<Position>,
}

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
enum Cell {
    Empty,
    Wall,
    Exit,
    Treasure,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
struct Position {
    x: usize,
    y: usize,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct LocalView {
    visible_cells: HashMap<Position, Cell>,
    other_agents: Vec<Position>,
    my_position: Position,
}

impl CollectiveMaze {
    pub fn new() -> Self {
        Self {
            maze_size: 50, // Smaller for testing
            visibility_radius: 5,
            maze: vec![],
            agent_positions: HashMap::new(),
            exit_positions: vec![],
            treasure_positions: vec![],
        }
    }
    
    fn generate_maze(&mut self) {
        let mut rng = rand::thread_rng();
        
        // Initialize empty maze
        self.maze = vec![vec![Cell::Empty; self.maze_size]; self.maze_size];
        
        // Add walls (simple random walls for now)
        for y in 0..self.maze_size {
            for x in 0..self.maze_size {
                if rng.gen_bool(0.3) && !self.is_border(x, y) {
                    self.maze[y][x] = Cell::Wall;
                }
            }
        }
        
        // Add exits (one in each corner)
        let exit_positions = vec![
            Position { x: 1, y: 1 },
            Position { x: self.maze_size - 2, y: 1 },
            Position { x: 1, y: self.maze_size - 2 },
            Position { x: self.maze_size - 2, y: self.maze_size - 2 },
        ];
        
        for pos in &exit_positions {
            self.maze[pos.y][pos.x] = Cell::Exit;
            self.exit_positions.push(*pos);
        }
        
        // Add treasures
        for _ in 0..3 {
            let x = rng.gen_range(10..self.maze_size - 10);
            let y = rng.gen_range(10..self.maze_size - 10);
            let pos = Position { x, y };
            self.maze[y][x] = Cell::Treasure;
            self.treasure_positions.push(pos);
        }
        
        // Ensure borders are walls
        for i in 0..self.maze_size {
            self.maze[0][i] = Cell::Wall;
            self.maze[self.maze_size - 1][i] = Cell::Wall;
            self.maze[i][0] = Cell::Wall;
            self.maze[i][self.maze_size - 1] = Cell::Wall;
        }
    }
    
    fn is_border(&self, x: usize, y: usize) -> bool {
        x == 0 || y == 0 || x == self.maze_size - 1 || y == self.maze_size - 1
    }
    
    fn place_agents(&mut self, players: &[String]) {
        let mut rng = rand::thread_rng();
        
        for player in players {
            // Find empty starting position
            loop {
                let x = rng.gen_range(1..self.maze_size - 1);
                let y = rng.gen_range(1..self.maze_size - 1);
                
                if self.maze[y][x] == Cell::Empty {
                    self.agent_positions.insert(player.clone(), Position { x, y });
                    break;
                }
            }
        }
    }
    
    fn get_local_view(&self, player_id: &str) -> Option<LocalView> {
        let my_pos = self.agent_positions.get(player_id)?;
        let mut visible_cells = HashMap::new();
        let mut other_agents = vec![];
        
        // Get visible cells within radius
        for dy in -(self.visibility_radius as i32)..=(self.visibility_radius as i32) {
            for dx in -(self.visibility_radius as i32)..=(self.visibility_radius as i32) {
                let x = (my_pos.x as i32 + dx) as usize;
                let y = (my_pos.y as i32 + dy) as usize;
                
                if x < self.maze_size && y < self.maze_size {
                    let pos = Position { x, y };
                    visible_cells.insert(pos, self.maze[y][x]);
                    
                    // Check for other agents
                    for (other_id, other_pos) in &self.agent_positions {
                        if other_id != player_id && other_pos == &pos {
                            other_agents.push(pos);
                        }
                    }
                }
            }
        }
        
        Some(LocalView {
            visible_cells,
            other_agents,
            my_position: *my_pos,
        })
    }
    
    fn move_agent(&mut self, player_id: &str, direction: Direction) -> MoveResult {
        if let Some(current_pos) = self.agent_positions.get(player_id).cloned() {
            let new_pos = match direction {
                Direction::North => Position { x: current_pos.x, y: current_pos.y.saturating_sub(1) },
                Direction::South => Position { x: current_pos.x, y: (current_pos.y + 1).min(self.maze_size - 1) },
                Direction::East => Position { x: (current_pos.x + 1).min(self.maze_size - 1), y: current_pos.y },
                Direction::West => Position { x: current_pos.x.saturating_sub(1), y: current_pos.y },
            };
            
            // Check if move is valid
            if new_pos.x < self.maze_size && new_pos.y < self.maze_size {
                match self.maze[new_pos.y][new_pos.x] {
                    Cell::Wall => MoveResult::Blocked,
                    Cell::Exit => {
                        self.agent_positions.insert(player_id.to_string(), new_pos);
                        MoveResult::Exit
                    }
                    Cell::Treasure => {
                        self.agent_positions.insert(player_id.to_string(), new_pos);
                        self.maze[new_pos.y][new_pos.x] = Cell::Empty; // Collect treasure
                        MoveResult::Treasure
                    }
                    Cell::Empty => {
                        self.agent_positions.insert(player_id.to_string(), new_pos);
                        MoveResult::Success
                    }
                }
            } else {
                MoveResult::Blocked
            }
        } else {
            MoveResult::Blocked
        }
    }
    
    fn calculate_emergence_score(&self, _shared_map: &HashMap<String, Position>) -> f32 {
        // Measure how well agents are exploring together
        let mut explored_cells = HashSet::new();
        
        for pos in self.agent_positions.values() {
            // Add visible area around each agent
            for dy in -(self.visibility_radius as i32)..=(self.visibility_radius as i32) {
                for dx in -(self.visibility_radius as i32)..=(self.visibility_radius as i32) {
                    let x = (pos.x as i32 + dx) as usize;
                    let y = (pos.y as i32 + dy) as usize;
                    if x < self.maze_size && y < self.maze_size {
                        explored_cells.insert(Position { x, y });
                    }
                }
            }
        }
        
        // Calculate coverage
        let total_cells = self.maze_size * self.maze_size;
        let coverage = explored_cells.len() as f32 / total_cells as f32;
        
        // Check for agent clustering (good for collective)
        let mut cluster_score = 0.0;
        let positions: Vec<_> = self.agent_positions.values().collect();
        
        for i in 0..positions.len() {
            for j in i+1..positions.len() {
                let dist = ((positions[i].x as f32 - positions[j].x as f32).powi(2) +
                           (positions[i].y as f32 - positions[j].y as f32).powi(2)).sqrt();
                if dist < 10.0 {
                    cluster_score += 1.0;
                }
            }
        }
        
        // Normalize cluster score
        let max_clusters = (positions.len() * (positions.len() - 1)) / 2;
        if max_clusters > 0 {
            cluster_score /= max_clusters as f32;
        }
        
        coverage * 0.7 + cluster_score * 0.3
    }
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
enum Direction {
    North,
    South,
    East,
    West,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum MoveResult {
    Success,
    Blocked,
    Exit,
    Treasure,
}

#[async_trait]
impl Game for CollectiveMaze {
    async fn initialize(&mut self, _config: GameConfig) -> anyhow::Result<GameState> {
        self.generate_maze();
        
        Ok(GameState {
            game_id: Uuid::new_v4(),
            game_type: GameType::CollectiveMaze,
            round: 0,
            scores: HashMap::new(),
            history: vec![],
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("maze_size".to_string(), serde_json::json!(self.maze_size));
                meta.insert("visibility_radius".to_string(), serde_json::json!(self.visibility_radius));
                meta.insert("treasures_remaining".to_string(), serde_json::json!(self.treasure_positions.len()));
                meta
            },
        })
    }
    
    async fn process_round(&mut self, state: &GameState, actions: HashMap<String, Action>) -> anyhow::Result<RoundResult> {
        // Place agents on first round
        if state.round == 0 && self.agent_positions.is_empty() {
            let players: Vec<String> = actions.keys().cloned().collect();
            self.place_agents(&players);
        }
        
        let mut scores_delta = HashMap::new();
        let mut special_events = vec![];
        let mut escaped_agents = vec![];
        let mut _treasures_found = 0;
        
        // Process each agent's move
        for (player_id, action) in &actions {
            // Get local view for the agent
            let _local_view = self.get_local_view(player_id);
            
            // Parse move direction
            if let Some(dir_str) = action.data.get("move").and_then(|m| m.as_str()) {
                let direction = match dir_str {
                    "north" | "up" => Direction::North,
                    "south" | "down" => Direction::South,
                    "east" | "right" => Direction::East,
                    "west" | "left" => Direction::West,
                    _ => continue,
                };
                
                // Execute move
                match self.move_agent(player_id, direction) {
                    MoveResult::Exit => {
                        escaped_agents.push(player_id.clone());
                        scores_delta.insert(player_id.clone(), 50);
                        special_events.push(format!("{} found an exit!", player_id));
                    }
                    MoveResult::Treasure => {
                        _treasures_found += 1;
                        scores_delta.insert(player_id.clone(), 30);
                        special_events.push(format!("{} found treasure!", player_id));
                    }
                    MoveResult::Success => {
                        scores_delta.insert(player_id.clone(), 1); // Small reward for exploration
                    }
                    MoveResult::Blocked => {
                        // No penalty for hitting walls
                    }
                }
            }
        }
        
        // Calculate emergence score
        let shared_map = HashMap::new(); // TODO: Implement shared mapping
        let emergence_score = self.calculate_emergence_score(&shared_map);
        let emergence_detected = emergence_score > 0.6 && actions.len() > 5;
        
        if emergence_detected {
            special_events.push("Collective exploration pattern emerged!".to_string());
            
            // Bonus for all collective agents
            for player_id in actions.keys() {
                if player_id.starts_with("collective_") {
                    *scores_delta.entry(player_id.clone()).or_insert(0) += 5;
                }
            }
        }
        
        // Determine winners/losers for this round
        let winners = escaped_agents.clone();
        let losers = vec![]; // No losers in exploration game
        
        Ok(RoundResult {
            round: state.round + 1,
            actions: actions.clone(),
            outcome: Outcome {
                winners,
                losers,
                special_events,
                emergence_detected,
            },
            scores_delta,
            timestamp: chrono::Utc::now(),
        })
    }
    
    async fn is_game_over(&self, state: &GameState) -> bool {
        // Game ends when all agents escape or max rounds reached
        let all_escaped = self.agent_positions.values()
            .all(|pos| self.exit_positions.contains(pos));
        
        state.round >= 100 || all_escaped || 
        state.scores.values().any(|&score| score >= 200)
    }
    
    async fn calculate_final_result(&self, state: &GameState) -> GameResult {
        // Winner is highest scoring player
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
                        event_type: "collective_exploration".to_string(),
                        description: "Agents coordinated exploration efficiently".to_string(),
                        emergence_score: 0.8,
                    })
                } else {
                    None
                }
            })
            .collect();
        
        // Calculate collective vs individual performance
        let collective_total: i32 = state.scores.iter()
            .filter(|(id, _)| id.starts_with("collective_"))
            .map(|(_, &score)| score)
            .sum();
        
        let sota_total: i32 = state.scores.iter()
            .filter(|(id, _)| id.starts_with("sota_"))
            .map(|(_, &score)| score)
            .sum();
        
        let collective_count = state.scores.keys().filter(|k| k.starts_with("collective_")).count().max(1);
        let sota_count = state.scores.keys().filter(|k| k.starts_with("sota_")).count().max(1);
        
        // Calculate emergence frequency before moving emergence_events
        let emergence_frequency = emergence_events.len() as f32 / state.round.max(1) as f32;
        
        GameResult {
            game_id: state.game_id,
            winner,
            final_scores: state.scores.clone(),
            total_rounds: state.round,
            emergence_events,
            analytics: GameAnalytics {
                collective_coordination_score: emergence_frequency,
                decision_diversity_index: 0.0, // TODO
                strategic_depth: state.round as f32 / 100.0,
                emergence_frequency,
                performance_differential: (collective_total / collective_count as i32) as f32 - 
                                         (sota_total / sota_count as i32) as f32,
            },
        }
    }
}