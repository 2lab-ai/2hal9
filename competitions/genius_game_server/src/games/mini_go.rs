use super::*;
use async_trait::async_trait;
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

const BOARD_SIZE: usize = 9;
const KOMI: f32 = 5.5; // Compensation for white going second

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Stone {
    Empty,
    Black,
    White,
}

pub struct MiniGoGame {
    board: [[Stone; BOARD_SIZE]; BOARD_SIZE],
    current_player: Stone,
    captures: HashMap<String, usize>, // Player -> captured stones
    ko_point: Option<(usize, usize)>, // Prevent immediate recapture
    pass_count: usize,
    move_history: Vec<MoveRecord>,
    players: HashMap<String, Stone>, // Player ID -> Stone color
}

#[derive(Debug, Clone)]
struct MoveRecord {
    #[allow(dead_code)]
    player: String,
    action: GoAction,
    captures: Vec<(usize, usize)>,
}

#[derive(Debug, Clone)]
enum GoAction {
    Place(usize, usize),
    Pass,
}

impl Default for MiniGoGame {
    fn default() -> Self {
        Self::new()
    }
}

impl MiniGoGame {
    pub fn new() -> Self {
        Self {
            board: [[Stone::Empty; BOARD_SIZE]; BOARD_SIZE],
            current_player: Stone::Black,
            captures: HashMap::new(),
            ko_point: None,
            pass_count: 0,
            move_history: Vec::new(),
            players: HashMap::new(),
        }
    }
    
    fn count_liberties(&self, row: usize, col: usize) -> usize {
        let stone = self.board[row][col];
        if stone == Stone::Empty {
            return 0;
        }
        
        let mut visited = HashSet::new();
        let mut liberties = HashSet::new();
        self.count_group_liberties(row, col, stone, &mut visited, &mut liberties);
        liberties.len()
    }
    
    fn count_group_liberties(&self, row: usize, col: usize, stone: Stone, 
                           visited: &mut HashSet<(usize, usize)>, 
                           liberties: &mut HashSet<(usize, usize)>) {
        if visited.contains(&(row, col)) {
            return;
        }
        
        visited.insert((row, col));
        
        // Check all four directions
        let directions = [(0, 1), (1, 0), (0, -1), (-1, 0)];
        for (dr, dc) in directions.iter() {
            let new_row = row as i32 + dr;
            let new_col = col as i32 + dc;
            
            if new_row >= 0 && new_row < BOARD_SIZE as i32 && 
               new_col >= 0 && new_col < BOARD_SIZE as i32 {
                let r = new_row as usize;
                let c = new_col as usize;
                
                match self.board[r][c] {
                    Stone::Empty => {
                        liberties.insert((r, c));
                    }
                    s if s == stone => {
                        self.count_group_liberties(r, c, stone, visited, liberties);
                    }
                    _ => {}
                }
            }
        }
    }
    
    fn capture_opponent_stones(&mut self, opponent_stone: Stone) -> Vec<(usize, usize)> {
        let mut captured = Vec::new();
        
        for row in 0..BOARD_SIZE {
            for col in 0..BOARD_SIZE {
                if self.board[row][col] == opponent_stone && self.count_liberties(row, col) == 0 {
                    captured.push((row, col));
                }
            }
        }
        
        // Remove captured stones
        for &(row, col) in &captured {
            self.board[row][col] = Stone::Empty;
        }
        
        captured
    }
    
    fn is_valid_move(&self, row: usize, col: usize, stone: Stone) -> bool {
        // Check if position is empty
        if self.board[row][col] != Stone::Empty {
            return false;
        }
        
        // Check ko rule
        if let Some((ko_row, ko_col)) = self.ko_point {
            if row == ko_row && col == ko_col {
                return false;
            }
        }
        
        // Temporarily place stone
        let mut temp_board = self.board;
        temp_board[row][col] = stone;
        
        // Check for suicide (placing a stone with no liberties and not capturing)
        let temp_game = MiniGoGame {
            board: temp_board,
            ..self.clone()
        };
        
        let has_liberties = temp_game.count_liberties(row, col) > 0;
        
        // Check if this move would capture opponent stones
        let opponent = match stone {
            Stone::Black => Stone::White,
            Stone::White => Stone::Black,
            Stone::Empty => return false,
        };
        
        let would_capture = (0..BOARD_SIZE).any(|r| {
            (0..BOARD_SIZE).any(|c| {
                temp_board[r][c] == opponent && temp_game.count_liberties(r, c) == 0
            })
        });
        
        has_liberties || would_capture
    }
    
    fn calculate_territory(&self) -> (usize, usize) {
        let mut black_territory = 0;
        let mut white_territory = 0;
        let mut visited = HashSet::new();
        
        for row in 0..BOARD_SIZE {
            for col in 0..BOARD_SIZE {
                if self.board[row][col] == Stone::Empty && !visited.contains(&(row, col)) {
                    let (territory_owner, territory_size) = self.flood_fill_territory(row, col, &mut visited);
                    match territory_owner {
                        Stone::Black => black_territory += territory_size,
                        Stone::White => white_territory += territory_size,
                        Stone::Empty => {} // Neutral territory
                    }
                }
            }
        }
        
        (black_territory, white_territory)
    }
    
    fn flood_fill_territory(&self, start_row: usize, start_col: usize, 
                           visited: &mut HashSet<(usize, usize)>) -> (Stone, usize) {
        let mut stack = vec![(start_row, start_col)];
        let mut territory_size = 0;
        let mut adjacent_stones = HashSet::new();
        
        while let Some((row, col)) = stack.pop() {
            if visited.contains(&(row, col)) {
                continue;
            }
            
            visited.insert((row, col));
            territory_size += 1;
            
            let directions = [(0, 1), (1, 0), (0, -1), (-1, 0)];
            for (dr, dc) in directions.iter() {
                let new_row = row as i32 + dr;
                let new_col = col as i32 + dc;
                
                if new_row >= 0 && new_row < BOARD_SIZE as i32 && 
                   new_col >= 0 && new_col < BOARD_SIZE as i32 {
                    let r = new_row as usize;
                    let c = new_col as usize;
                    
                    match self.board[r][c] {
                        Stone::Empty => {
                            if !visited.contains(&(r, c)) {
                                stack.push((r, c));
                            }
                        }
                        stone => {
                            adjacent_stones.insert(stone);
                        }
                    }
                }
            }
        }
        
        // Territory belongs to a player only if all adjacent stones are theirs
        let owner = if adjacent_stones.len() == 1 {
            *adjacent_stones.iter().next().unwrap()
        } else {
            Stone::Empty
        };
        
        (owner, territory_size)
    }
    
    fn detect_strategic_emergence(&self, state: &GameState) -> Option<EmergenceEvent> {
        if self.move_history.len() < 20 {
            return None;
        }
        
        // Look for strategic patterns
        let recent_moves = self.move_history.iter().rev().take(10);
        let mut strategic_moves = 0;
        
        for move_record in recent_moves {
            if let GoAction::Place(row, col) = move_record.action {
                // Check if move was strategic (corner, side, or capturing)
                let is_corner = !(3..=5).contains(&row) && !(3..=5).contains(&col);
                let is_side = row == 0 || row == 8 || col == 0 || col == 8;
                let captured_stones = !move_record.captures.is_empty();
                
                if is_corner || is_side || captured_stones {
                    strategic_moves += 1;
                }
            }
        }
        
        if strategic_moves > 7 {
            Some(EmergenceEvent {
                round: state.round,
                event_type: "go_strategy_emergence".to_string(),
                description: "Players discovered advanced Go strategies".to_string(),
                emergence_score: strategic_moves as f32 / 10.0,
            })
        } else {
            None
        }
    }
}

// Implement Clone manually to avoid the array size issue
impl Clone for MiniGoGame {
    fn clone(&self) -> Self {
        Self {
            board: self.board,
            current_player: self.current_player,
            captures: self.captures.clone(),
            ko_point: self.ko_point,
            pass_count: self.pass_count,
            move_history: self.move_history.clone(),
            players: self.players.clone(),
        }
    }
}

#[async_trait]
impl Game for MiniGoGame {
    async fn initialize(&mut self, _config: GameConfig) -> anyhow::Result<GameState> {
        Ok(GameState {
            game_id: Uuid::new_v4(),
            game_type: GameType::MiniGo,
            round: 0,
            scores: HashMap::new(),
            history: vec![],
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("board_size".to_string(), serde_json::json!(BOARD_SIZE));
                meta.insert("komi".to_string(), serde_json::json!(KOMI));
                meta.insert("rules".to_string(), serde_json::json!({
                    "capture": "Remove opponent stones with no liberties",
                    "ko": "Cannot immediately recapture",
                    "scoring": "Territory + captures + komi for white"
                }));
                meta
            },
        })
    }
    
    async fn process_round(&mut self, state: &GameState, actions: HashMap<String, Action>) -> anyhow::Result<RoundResult> {
        // Assign colors to new players
        for player_id in actions.keys() {
            if !self.players.contains_key(player_id) {
                let color = if self.players.is_empty() {
                    Stone::Black
                } else if self.players.len() == 1 {
                    Stone::White
                } else {
                    // For more than 2 players, alternate
                    if self.players.len() % 2 == 0 {
                        Stone::Black
                    } else {
                        Stone::White
                    }
                };
                self.players.insert(player_id.clone(), color);
                self.captures.insert(player_id.clone(), 0);
            }
        }
        
        let mut round_captures = HashMap::new();
        let mut moves_made = Vec::new();
        
        // Process moves in order (black first, then white)
        let mut sorted_actions: Vec<_> = actions.iter().collect();
        sorted_actions.sort_by_key(|(id, _)| {
            self.players.get(*id).map(|&color| {
                match color {
                    Stone::Black => 0,
                    Stone::White => 1,
                    Stone::Empty => 2,
                }
            }).unwrap_or(3)
        });
        
        for (player_id, action) in sorted_actions {
            let player_color = self.players.get(player_id).copied().unwrap_or(Stone::Empty);
            
            match action.action_type.as_str() {
                "pass" => {
                    self.pass_count += 1;
                    moves_made.push((player_id.clone(), GoAction::Pass));
                }
                "place" | "move" => {
                    self.pass_count = 0;
                    
                    // Parse coordinates
                    let (row, col) = if let Some(obj) = action.data.as_object() {
                        let row = obj.get("row").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
                        let col = obj.get("col").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
                        (row, col)
                    } else {
                        (0, 0) // Default position
                    };
                    
                    if row < BOARD_SIZE && col < BOARD_SIZE && 
                       self.is_valid_move(row, col, player_color) {
                        // Place stone
                        self.board[row][col] = player_color;
                        
                        // Capture opponent stones
                        let opponent = match player_color {
                            Stone::Black => Stone::White,
                            Stone::White => Stone::Black,
                            Stone::Empty => continue,
                        };
                        
                        let captured = self.capture_opponent_stones(opponent);
                        
                        // Update ko point if single stone captured
                        if captured.len() == 1 {
                            self.ko_point = Some(captured[0]);
                        } else {
                            self.ko_point = None;
                        }
                        
                        // Update captures count
                        *self.captures.entry(player_id.clone()).or_insert(0) += captured.len();
                        round_captures.insert(player_id.clone(), captured.len());
                        
                        // Record move
                        self.move_history.push(MoveRecord {
                            player: player_id.clone(),
                            action: GoAction::Place(row, col),
                            captures: captured,
                        });
                        
                        moves_made.push((player_id.clone(), GoAction::Place(row, col)));
                    }
                }
                _ => {}
            }
        }
        
        // Calculate scores
        let mut scores_delta = HashMap::new();
        
        if self.pass_count >= 2 {
            // Game ends when both players pass
            let (black_territory, white_territory) = self.calculate_territory();
            
            for (player_id, &color) in &self.players {
                let captures = self.captures.get(player_id).copied().unwrap_or(0);
                let score = match color {
                    Stone::Black => black_territory + captures,
                    Stone::White => white_territory + captures + KOMI as usize,
                    Stone::Empty => 0,
                };
                scores_delta.insert(player_id.clone(), score as i32);
            }
        } else {
            // Incremental scoring based on captures this round
            for (player_id, captures) in round_captures {
                scores_delta.insert(player_id, captures as i32);
            }
        }
        
        // Check for emergence
        let emergence_event = self.detect_strategic_emergence(state);
        
        // Determine winners and losers for this round
        let max_score = scores_delta.values().max().copied().unwrap_or(0);
        let winners: Vec<String> = scores_delta.iter()
            .filter(|(_, &score)| score == max_score && score > 0)
            .map(|(id, _)| id.clone())
            .collect();
        
        let losers: Vec<String> = scores_delta.iter()
            .filter(|(_, &score)| score < max_score)
            .map(|(id, _)| id.clone())
            .collect();
        
        let mut special_events = vec![];
        if self.pass_count >= 2 {
            special_events.push("Game ended - both players passed".to_string());
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
        self.pass_count >= 2 || state.round >= 200
    }
    
    async fn calculate_final_result(&self, state: &GameState) -> GameResult {
        // Calculate final territory and scores
        let (black_territory, white_territory) = self.calculate_territory();
        
        let mut final_scores = HashMap::new();
        let mut black_total = 0;
        let mut white_total = 0;
        
        for (player_id, &color) in &self.players {
            let captures = self.captures.get(player_id).copied().unwrap_or(0);
            let score = match color {
                Stone::Black => {
                    let s = black_territory + captures;
                    black_total += s;
                    s
                }
                Stone::White => {
                    let s = white_territory + captures + KOMI as usize;
                    white_total += s;
                    s
                }
                Stone::Empty => 0,
            };
            final_scores.insert(player_id.clone(), score as i32);
        }
        
        let winner = final_scores.iter()
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
                        event_type: "go_strategy".to_string(),
                        description: "Advanced Go strategies emerged".to_string(),
                        emergence_score: 0.8,
                    })
                } else {
                    None
                }
            })
            .collect();
        
        let emergence_frequency = emergence_events.len() as f32 / state.round.max(1) as f32;
        
        // Calculate strategy depth based on move patterns
        let strategic_moves = self.move_history.iter()
            .filter(|move_record| {
                matches!(move_record.action, GoAction::Place(row, col) 
                    if !(3..=5).contains(&row) && !(3..=5).contains(&col))
            })
            .count();
        let strategic_depth = strategic_moves as f32 / self.move_history.len().max(1) as f32;
        
        GameResult {
            game_id: state.game_id,
            winner,
            final_scores,
            total_rounds: state.round,
            emergence_events,
            analytics: GameAnalytics {
                collective_coordination_score: 0.0, // Not applicable for Go
                decision_diversity_index: 0.5, // Could analyze move variety
                strategic_depth,
                emergence_frequency,
                performance_differential: (black_total as f32 - white_total as f32).abs(),
            },
        }
    }
}