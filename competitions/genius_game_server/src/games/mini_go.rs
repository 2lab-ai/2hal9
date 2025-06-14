use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;
use super::{Game, GameState, PlayerAction, GameResult, EmergenceMetrics};
use anyhow::Result;

const BOARD_SIZE: usize = 9; // Mini Go uses 9x9 board
const KOMI: f32 = 5.5; // Compensation for white

/// Mini Go - Simplified version of Go for AI evaluation
pub struct MiniGoGame {
    id: Uuid,
    round: u32,
    max_rounds: u32,
    players: Vec<String>,
    board: [[Stone; BOARD_SIZE]; BOARD_SIZE],
    current_player: usize,
    captured_stones: HashMap<String, u32>,
    move_history: Vec<GoMove>,
    ko_point: Option<(usize, usize)>,
    pass_count: u32,
    special_rules: HashMap<String, String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
enum Stone {
    Empty,
    Black,
    White,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GoMove {
    player: String,
    position: Option<(usize, usize)>, // None means pass
    captured: Vec<(usize, usize)>,
    board_state: [[Stone; BOARD_SIZE]; BOARD_SIZE],
}

impl MiniGoGame {
    pub fn new(max_rounds: u32, special_rules: HashMap<String, String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            round: 0,
            max_rounds,
            players: Vec::new(),
            board: [[Stone::Empty; BOARD_SIZE]; BOARD_SIZE],
            current_player: 0,
            captured_stones: HashMap::new(),
            move_history: Vec::new(),
            ko_point: None,
            pass_count: 0,
            special_rules,
        }
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
        
        // Check if move would be suicide (no liberties)
        if !self.has_liberties(&temp_board, row, col) {
            // Check if it captures enemy stones
            let opponent = match stone {
                Stone::Black => Stone::White,
                Stone::White => Stone::Black,
                Stone::Empty => return false,
            };
            
            let neighbors = self.get_neighbors(row, col);
            let captures_enemy = neighbors.iter().any(|&(r, c)| {
                temp_board[r][c] == opponent && !self.has_liberties(&temp_board, r, c)
            });
            
            if !captures_enemy {
                return false; // Suicide move
            }
        }
        
        true
    }
    
    fn has_liberties(&self, board: &[[Stone; BOARD_SIZE]; BOARD_SIZE], row: usize, col: usize) -> bool {
        let stone = board[row][col];
        if stone == Stone::Empty {
            return true;
        }
        
        let mut visited = HashSet::new();
        self.check_group_liberties(board, row, col, stone, &mut visited)
    }
    
    fn check_group_liberties(
        &self,
        board: &[[Stone; BOARD_SIZE]; BOARD_SIZE],
        row: usize,
        col: usize,
        stone: Stone,
        visited: &mut HashSet<(usize, usize)>,
    ) -> bool {
        if visited.contains(&(row, col)) {
            return false;
        }
        visited.insert((row, col));
        
        // Check neighbors
        for (r, c) in self.get_neighbors(row, col) {
            if board[r][c] == Stone::Empty {
                return true; // Found liberty
            }
            if board[r][c] == stone {
                if self.check_group_liberties(board, r, c, stone, visited) {
                    return true;
                }
            }
        }
        
        false
    }
    
    fn get_neighbors(&self, row: usize, col: usize) -> Vec<(usize, usize)> {
        let mut neighbors = Vec::new();
        
        if row > 0 {
            neighbors.push((row - 1, col));
        }
        if row < BOARD_SIZE - 1 {
            neighbors.push((row + 1, col));
        }
        if col > 0 {
            neighbors.push((row, col - 1));
        }
        if col < BOARD_SIZE - 1 {
            neighbors.push((row, col + 1));
        }
        
        neighbors
    }
    
    fn capture_stones(&mut self, row: usize, col: usize, stone: Stone) -> Vec<(usize, usize)> {
        let opponent = match stone {
            Stone::Black => Stone::White,
            Stone::White => Stone::Black,
            Stone::Empty => return vec![],
        };
        
        let mut captured = Vec::new();
        
        for (r, c) in self.get_neighbors(row, col) {
            if self.board[r][c] == opponent && !self.has_liberties(&self.board, r, c) {
                // Capture the group
                let mut group = HashSet::new();
                self.find_group(&self.board, r, c, opponent, &mut group);
                
                for &(gr, gc) in &group {
                    self.board[gr][gc] = Stone::Empty;
                    captured.push((gr, gc));
                }
            }
        }
        
        captured
    }
    
    fn find_group(
        &self,
        board: &[[Stone; BOARD_SIZE]; BOARD_SIZE],
        row: usize,
        col: usize,
        stone: Stone,
        group: &mut HashSet<(usize, usize)>,
    ) {
        if group.contains(&(row, col)) || board[row][col] != stone {
            return;
        }
        
        group.insert((row, col));
        
        for (r, c) in self.get_neighbors(row, col) {
            if board[r][c] == stone {
                self.find_group(board, r, c, stone, group);
            }
        }
    }
    
    fn calculate_territory(&self) -> (i32, i32) {
        let mut black_territory = 0;
        let mut white_territory = 0;
        let mut visited = HashSet::new();
        
        for row in 0..BOARD_SIZE {
            for col in 0..BOARD_SIZE {
                if self.board[row][col] == Stone::Empty && !visited.contains(&(row, col)) {
                    let mut territory = HashSet::new();
                    let owner = self.find_territory_owner(row, col, &mut territory);
                    
                    for pos in &territory {
                        visited.insert(*pos);
                    }
                    
                    match owner {
                        Some(Stone::Black) => black_territory += territory.len() as i32,
                        Some(Stone::White) => white_territory += territory.len() as i32,
                        _ => {} // Neutral territory
                    }
                }
            }
        }
        
        // Add captured stones
        let black_captured = self.captured_stones.get(&self.players[0]).copied().unwrap_or(0) as i32;
        let white_captured = self.captured_stones.get(&self.players[1]).copied().unwrap_or(0) as i32;
        
        (black_territory + white_captured, white_territory + black_captured)
    }
    
    fn find_territory_owner(
        &self,
        row: usize,
        col: usize,
        territory: &mut HashSet<(usize, usize)>,
    ) -> Option<Stone> {
        let mut stack = vec![(row, col)];
        let mut border_stones = HashSet::new();
        
        while let Some((r, c)) = stack.pop() {
            if territory.contains(&(r, c)) {
                continue;
            }
            
            if self.board[r][c] == Stone::Empty {
                territory.insert((r, c));
                
                for (nr, nc) in self.get_neighbors(r, c) {
                    if !territory.contains(&(nr, nc)) {
                        stack.push((nr, nc));
                    }
                }
            } else {
                border_stones.insert(self.board[r][c]);
            }
        }
        
        // Territory belongs to a player only if all border stones are theirs
        if border_stones.len() == 1 {
            border_stones.into_iter().next()
        } else {
            None
        }
    }
    
    fn detect_emergence(&self) -> EmergenceMetrics {
        if self.move_history.len() < 10 {
            return EmergenceMetrics {
                emergence_detected: false,
                coordination_score: 0.0,
                collective_intelligence_index: 0.0,
                decision_diversity_index: 0.0,
                strategic_depth: 0.0,
                special_patterns: HashMap::new(),
            };
        }
        
        // Analyze strategic patterns
        let mut pattern_complexity = 0.0;
        let mut territory_control = 0.0;
        let mut tactical_depth = 0.0;
        
        // Check for complex patterns (ladders, nets, life & death)
        // This is simplified - real Go pattern recognition is much more complex
        
        // Territory control balance
        let (black_territory, white_territory) = self.calculate_territory();
        let total_territory = black_territory + white_territory;
        if total_territory > 0 {
            let balance = 1.0 - ((black_territory - white_territory).abs() as f32 / total_territory as f32);
            territory_control = balance;
        }
        
        // Tactical depth - check for strategic sacrifices
        let sacrifices = self.move_history.windows(3).filter(|moves| {
            moves[1].captured.len() > 0 && moves[2].captured.len() > moves[1].captured.len()
        }).count();
        tactical_depth = (sacrifices as f32 / self.move_history.len().max(1) as f32).min(1.0);
        
        // Pattern complexity - check for non-obvious moves
        let complex_moves = self.move_history.iter().filter(|m| {
            if let Some((row, col)) = m.position {
                // Check if move is not directly adjacent to existing stones
                self.get_neighbors(row, col).iter()
                    .all(|&(r, c)| m.board_state[r][c] == Stone::Empty)
            } else {
                false
            }
        }).count();
        pattern_complexity = (complex_moves as f32 / self.move_history.len().max(1) as f32) * 2.0;
        
        let emergence_detected = pattern_complexity > 0.3 && 
                                territory_control > 0.7 && 
                                tactical_depth > 0.1;
        
        let mut special_patterns = HashMap::new();
        special_patterns.insert("territory_balance".to_string(), territory_control.to_string());
        special_patterns.insert("pattern_complexity".to_string(), pattern_complexity.to_string());
        special_patterns.insert("tactical_depth".to_string(), tactical_depth.to_string());
        
        EmergenceMetrics {
            emergence_detected,
            coordination_score: territory_control,
            collective_intelligence_index: pattern_complexity,
            decision_diversity_index: 0.5, // Go doesn't have diversity in the same sense
            strategic_depth: tactical_depth,
            special_patterns,
        }
    }
}

#[async_trait]
impl Game for MiniGoGame {
    async fn get_state(&self) -> Result<GameState> {
        // Generate available moves
        let mut available_moves = Vec::new();
        
        if self.players.len() == 2 {
            let current_stone = if self.current_player == 0 { Stone::Black } else { Stone::White };
            
            for row in 0..BOARD_SIZE {
                for col in 0..BOARD_SIZE {
                    if self.is_valid_move(row, col, current_stone) {
                        available_moves.push(format!("{},{}", row, col));
                    }
                }
            }
            
            available_moves.push("pass".to_string());
        }
        
        let (black_territory, white_territory) = self.calculate_territory();
        
        let mut scores = HashMap::new();
        if self.players.len() == 2 {
            scores.insert(self.players[0].clone(), black_territory);
            scores.insert(self.players[1].clone(), (white_territory as f32 + KOMI) as i32);
        }
        
        let board_visual = self.board.iter()
            .map(|row| row.iter().map(|&stone| match stone {
                Stone::Empty => '.',
                Stone::Black => 'B',
                Stone::White => 'W',
            }).collect::<String>())
            .collect::<Vec<_>>();
        
        let mut player_states = HashMap::new();
        for (i, player) in self.players.iter().enumerate() {
            player_states.insert(player.clone(), serde_json::json!({
                "color": if i == 0 { "black" } else { "white" },
                "captured": self.captured_stones.get(player).copied().unwrap_or(0),
                "territory": if i == 0 { black_territory } else { white_territory },
            }));
        }
        
        Ok(GameState {
            game_id: self.id,
            game_type: "mini_go".to_string(),
            round: self.round,
            players: self.players.clone(),
            current_state: serde_json::json!({
                "board": board_visual,
                "current_player": if self.players.is_empty() { "" } else { &self.players[self.current_player] },
                "available_moves": available_moves,
                "pass_count": self.pass_count,
                "ko_point": self.ko_point,
            }),
            available_actions: available_moves,
            scores,
            player_states,
            is_complete: self.pass_count >= 2 || self.round >= self.max_rounds,
            special_data: Some(serde_json::json!({
                "board_size": BOARD_SIZE,
                "komi": KOMI,
                "move_count": self.move_history.len(),
            })),
        })
    }
    
    async fn process_action(&mut self, action: PlayerAction) -> Result<()> {
        // Process moves when it's player's turn
        if self.players.len() == 2 && self.players[self.current_player] == action.player_id {
            if action.action_type == "pass" {
                self.pass_count += 1;
            } else {
                // Parse move coordinates
                let parts: Vec<&str> = action.action_type.split(',').collect();
                if parts.len() == 2 {
                    if let (Ok(row), Ok(col)) = (parts[0].parse::<usize>(), parts[1].parse::<usize>()) {
                        let stone = if self.current_player == 0 { Stone::Black } else { Stone::White };
                        
                        if self.is_valid_move(row, col, stone) {
                            // Place stone
                            self.board[row][col] = stone;
                            
                            // Capture enemy stones
                            let captured = self.capture_stones(row, col, stone);
                            
                            // Update captured count
                            *self.captured_stones.entry(action.player_id.clone()).or_insert(0) += captured.len() as u32;
                            
                            // Update ko point
                            if captured.len() == 1 && 
                               self.get_neighbors(captured[0].0, captured[0].1).len() == 4 {
                                self.ko_point = Some(captured[0]);
                            } else {
                                self.ko_point = None;
                            }
                            
                            // Record move
                            self.move_history.push(GoMove {
                                player: action.player_id,
                                position: Some((row, col)),
                                captured,
                                board_state: self.board,
                            });
                            
                            self.pass_count = 0;
                        }
                    }
                }
            }
            
            // Switch players
            self.current_player = 1 - self.current_player;
        }
        
        Ok(())
    }
    
    async fn advance_round(&mut self) -> Result<GameResult> {
        self.round += 1;
        
        // For demo purposes, simulate some moves
        if self.players.len() == 2 && self.pass_count < 2 && self.round < self.max_rounds {
            // Simple AI: play random valid moves
            let current_stone = if self.current_player == 0 { Stone::Black } else { Stone::White };
            let mut valid_moves = Vec::new();
            
            for row in 0..BOARD_SIZE {
                for col in 0..BOARD_SIZE {
                    if self.is_valid_move(row, col, current_stone) {
                        valid_moves.push((row, col));
                    }
                }
            }
            
            if !valid_moves.is_empty() && rand::random::<f32>() > 0.1 {
                let (row, col) = valid_moves[rand::random::<usize>() % valid_moves.len()];
                let action = PlayerAction {
                    player_id: self.players[self.current_player].clone(),
                    action_type: format!("{},{}", row, col),
                    data: None,
                };
                self.process_action(action).await?;
            } else {
                // Pass
                let action = PlayerAction {
                    player_id: self.players[self.current_player].clone(),
                    action_type: "pass".to_string(),
                    data: None,
                };
                self.process_action(action).await?;
            }
        }
        
        let (black_territory, white_territory) = self.calculate_territory();
        let white_score = white_territory as f32 + KOMI;
        
        let mut scores = HashMap::new();
        if self.players.len() == 2 {
            scores.insert(self.players[0].clone(), black_territory);
            scores.insert(self.players[1].clone(), white_score as i32);
        }
        
        let round_winners = if self.pass_count >= 2 || self.round >= self.max_rounds {
            // Game over
            if black_territory > white_score as i32 {
                vec![self.players[0].clone()]
            } else {
                vec![self.players[1].clone()]
            }
        } else {
            vec![]
        };
        
        let emergence = self.detect_emergence();
        
        Ok(GameResult {
            round: self.round,
            scores,
            round_winners,
            is_final_round: self.pass_count >= 2 || self.round >= self.max_rounds,
            emergence_metrics: emergence,
            special_data: Some(serde_json::json!({
                "board": self.board,
                "game_ended": self.pass_count >= 2,
                "final_scores": {
                    "black": black_territory,
                    "white": white_score,
                },
            })),
        })
    }
    
    fn add_player(&mut self, player_id: String) -> Result<()> {
        if self.players.len() < 2 && !self.players.contains(&player_id) {
            self.players.push(player_id.clone());
            self.captured_stones.insert(player_id, 0);
        }
        Ok(())
    }
    
    fn get_game_id(&self) -> Uuid {
        self.id
    }
}