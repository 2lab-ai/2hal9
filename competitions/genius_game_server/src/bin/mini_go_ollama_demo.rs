use colored::*;
use reqwest;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::time::Instant;
use rand::Rng;
use std::fs::File;
use std::io::Write;

const BOARD_SIZE: usize = 9;
const EMPTY: u8 = 0;
const BLACK: u8 = 1;
const WHITE: u8 = 2;

#[derive(Clone, Serialize)]
struct GoBoard {
    board: [[u8; BOARD_SIZE]; BOARD_SIZE],
    current_player: u8,
    captures: [u32; 2], // [black_captures, white_captures]
    move_count: u32,
    ko_point: Option<(usize, usize)>,
    #[serde(skip)]
    move_history: Vec<Move>,
}

#[derive(Clone, Serialize)]
struct Move {
    player: u8,
    row: usize,
    col: usize,
    captures: Vec<(usize, usize)>,
    time_ms: u128,
}

impl GoBoard {
    fn new() -> Self {
        Self {
            board: [[EMPTY; BOARD_SIZE]; BOARD_SIZE],
            current_player: BLACK,
            captures: [0, 0],
            move_count: 0,
            ko_point: None,
            move_history: Vec::new(),
        }
    }

    fn is_valid_move(&self, row: usize, col: usize) -> bool {
        if self.board[row][col] != EMPTY {
            return false;
        }
        
        // Check for ko
        if let Some((ko_row, ko_col)) = self.ko_point {
            if row == ko_row && col == ko_col {
                return false;
            }
        }
        
        // Temporarily place stone
        let mut temp_board = self.board;
        temp_board[row][col] = self.current_player;
        
        // Check if it has liberties or captures something
        let group = self.get_group(&temp_board, row, col);
        let has_liberties = self.count_liberties(&temp_board, &group) > 0;
        
        // Check if it captures opponent stones
        let opponent = if self.current_player == BLACK { WHITE } else { BLACK };
        let mut captures_something = false;
        
        for dr in [-1i32, 0, 1] {
            for dc in [-1i32, 0, 1] {
                if dr.abs() + dc.abs() != 1 { continue; }
                let nr = row as i32 + dr;
                let nc = col as i32 + dc;
                
                if nr >= 0 && nr < BOARD_SIZE as i32 && nc >= 0 && nc < BOARD_SIZE as i32 {
                    let nr = nr as usize;
                    let nc = nc as usize;
                    if temp_board[nr][nc] == opponent {
                        let opp_group = self.get_group(&temp_board, nr, nc);
                        if self.count_liberties(&temp_board, &opp_group) == 0 {
                            captures_something = true;
                        }
                    }
                }
            }
        }
        
        has_liberties || captures_something
    }

    fn make_move(&mut self, row: usize, col: usize) -> bool {
        if !self.is_valid_move(row, col) {
            return false;
        }
        
        let start_time = Instant::now();
        self.board[row][col] = self.current_player;
        self.ko_point = None;
        
        // Check for captures
        let opponent = if self.current_player == BLACK { WHITE } else { BLACK };
        let mut captured_stones = Vec::new();
        
        for dr in [-1i32, 0, 1] {
            for dc in [-1i32, 0, 1] {
                if dr.abs() + dc.abs() != 1 { continue; }
                let nr = row as i32 + dr;
                let nc = col as i32 + dc;
                
                if nr >= 0 && nr < BOARD_SIZE as i32 && nc >= 0 && nc < BOARD_SIZE as i32 {
                    let nr = nr as usize;
                    let nc = nc as usize;
                    if self.board[nr][nc] == opponent {
                        let group = self.get_group(&self.board, nr, nc);
                        if self.count_liberties(&self.board, &group) == 0 {
                            for &(gr, gc) in &group {
                                self.board[gr][gc] = EMPTY;
                                captured_stones.push((gr, gc));
                            }
                        }
                    }
                }
            }
        }
        
        // Update captures
        if self.current_player == BLACK {
            self.captures[0] += captured_stones.len() as u32;
        } else {
            self.captures[1] += captured_stones.len() as u32;
        }
        
        // Check for ko
        if captured_stones.len() == 1 {
            let (cap_row, cap_col) = captured_stones[0];
            let placed_group = self.get_group(&self.board, row, col);
            if placed_group.len() == 1 {
                self.ko_point = Some((cap_row, cap_col));
            }
        }
        
        // Record move
        self.move_history.push(Move {
            player: self.current_player,
            row,
            col,
            captures: captured_stones.clone(),
            time_ms: start_time.elapsed().as_millis(),
        });
        
        self.current_player = opponent;
        self.move_count += 1;
        true
    }

    fn get_group(&self, board: &[[u8; BOARD_SIZE]; BOARD_SIZE], row: usize, col: usize) -> Vec<(usize, usize)> {
        let color = board[row][col];
        if color == EMPTY {
            return vec![];
        }
        
        let mut group = vec![];
        let mut visited = [[false; BOARD_SIZE]; BOARD_SIZE];
        let mut stack = vec![(row, col)];
        
        while let Some((r, c)) = stack.pop() {
            if visited[r][c] {
                continue;
            }
            visited[r][c] = true;
            group.push((r, c));
            
            for dr in [-1i32, 0, 1] {
                for dc in [-1i32, 0, 1] {
                    if dr.abs() + dc.abs() != 1 { continue; }
                    let nr = r as i32 + dr;
                    let nc = c as i32 + dc;
                    
                    if nr >= 0 && nr < BOARD_SIZE as i32 && nc >= 0 && nc < BOARD_SIZE as i32 {
                        let nr = nr as usize;
                        let nc = nc as usize;
                        if !visited[nr][nc] && board[nr][nc] == color {
                            stack.push((nr, nc));
                        }
                    }
                }
            }
        }
        
        group
    }

    fn count_liberties(&self, board: &[[u8; BOARD_SIZE]; BOARD_SIZE], group: &[(usize, usize)]) -> usize {
        let mut liberties = std::collections::HashSet::new();
        
        for &(row, col) in group {
            for dr in [-1i32, 0, 1] {
                for dc in [-1i32, 0, 1] {
                    if dr.abs() + dc.abs() != 1 { continue; }
                    let nr = row as i32 + dr;
                    let nc = col as i32 + dc;
                    
                    if nr >= 0 && nr < BOARD_SIZE as i32 && nc >= 0 && nc < BOARD_SIZE as i32 {
                        let nr = nr as usize;
                        let nc = nc as usize;
                        if board[nr][nc] == EMPTY {
                            liberties.insert((nr, nc));
                        }
                    }
                }
            }
        }
        
        liberties.len()
    }

    fn get_valid_moves(&self) -> Vec<(usize, usize)> {
        let mut moves = vec![];
        for row in 0..BOARD_SIZE {
            for col in 0..BOARD_SIZE {
                if self.is_valid_move(row, col) {
                    moves.push((row, col));
                }
            }
        }
        moves
    }

    fn calculate_territory(&self) -> (i32, i32) {
        let mut black_territory = 0;
        let mut white_territory = 0;
        let mut visited = [[false; BOARD_SIZE]; BOARD_SIZE];
        
        for row in 0..BOARD_SIZE {
            for col in 0..BOARD_SIZE {
                if self.board[row][col] == EMPTY && !visited[row][col] {
                    let (territory, owner) = self.flood_fill_territory(row, col, &mut visited);
                    match owner {
                        Some(BLACK) => black_territory += territory,
                        Some(WHITE) => white_territory += territory,
                        Some(_) => {} // Other values (shouldn't happen)
                        None => {} // Neutral territory
                    }
                }
            }
        }
        
        // Add captures to score
        black_territory += self.captures[0] as i32;
        white_territory += self.captures[1] as i32;
        
        (black_territory, white_territory)
    }

    fn flood_fill_territory(&self, start_row: usize, start_col: usize, visited: &mut [[bool; BOARD_SIZE]; BOARD_SIZE]) -> (i32, Option<u8>) {
        let mut territory = 0;
        let mut borders = std::collections::HashSet::new();
        let mut stack = vec![(start_row, start_col)];
        
        while let Some((row, col)) = stack.pop() {
            if visited[row][col] {
                continue;
            }
            visited[row][col] = true;
            territory += 1;
            
            for dr in [-1i32, 0, 1] {
                for dc in [-1i32, 0, 1] {
                    if dr.abs() + dc.abs() != 1 { continue; }
                    let nr = row as i32 + dr;
                    let nc = col as i32 + dc;
                    
                    if nr >= 0 && nr < BOARD_SIZE as i32 && nc >= 0 && nc < BOARD_SIZE as i32 {
                        let nr = nr as usize;
                        let nc = nc as usize;
                        
                        if self.board[nr][nc] == EMPTY && !visited[nr][nc] {
                            stack.push((nr, nc));
                        } else if self.board[nr][nc] != EMPTY {
                            borders.insert(self.board[nr][nc]);
                        }
                    }
                }
            }
        }
        
        // Territory belongs to a player if only bordered by that player's stones
        let owner = if borders.len() == 1 {
            borders.into_iter().next()
        } else {
            None
        };
        
        (territory, owner)
    }

    fn get_game_state_summary(&self) -> String {
        let valid_moves = self.get_valid_moves();
        let (black_territory, white_territory) = self.calculate_territory();
        
        format!(
            "Move {}: {} to play. Territory estimate: Black {} vs White {}. {} valid moves.",
            self.move_count + 1,
            if self.current_player == BLACK { "Black" } else { "White" },
            black_territory,
            white_territory,
            valid_moves.len()
        )
    }
}

impl fmt::Display for GoBoard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "\n   A B C D E F G H J")?;
        for row in 0..BOARD_SIZE {
            write!(f, "{} ", 9 - row)?;
            for col in 0..BOARD_SIZE {
                let symbol = match self.board[row][col] {
                    BLACK => "● ".black(),
                    WHITE => "○ ".white(),
                    _ => {
                        // Show strategic points with subtle hints
                        if (row == 2 || row == 6) && (col == 2 || col == 6) {
                            "◦ ".bright_black() // Star points
                        } else if row == 4 && col == 4 {
                            "◦ ".bright_black() // Center
                        } else {
                            "· ".bright_black()
                        }
                    },
                };
                write!(f, "{}", symbol)?;
            }
            writeln!(f, " {}", 9 - row)?;
        }
        writeln!(f, "   A B C D E F G H J")?;
        writeln!(f, "\nCaptures - Black: {} | White: {}", self.captures[0], self.captures[1])?;
        Ok(())
    }
}

#[derive(Serialize)]
struct OllamaRequest {
    model: String,
    prompt: String,
    stream: bool,
    options: OllamaOptions,
}

#[derive(Serialize)]
struct OllamaOptions {
    temperature: f32,
    num_predict: usize,
}

#[derive(Deserialize)]
struct OllamaResponse {
    response: String,
}

#[derive(Deserialize)]
struct GoDecision {
    position: String,
    reasoning: String,
}

async fn meditate_on_rules(model: &str, player_name: &str) -> anyhow::Result<()> {
    println!("\n{} {} is preparing to think...", "🧘", player_name.bright_yellow());
    
    let rules = r#"You will play Go on a 9x9 board.

Basic rules:
- Black plays first, then alternate
- Place stones on intersections
- Capture enemy stones by surrounding them
- Winner controls most territory

Now think deeply about this game..."#;

    let client = reqwest::Client::new();
    let request = OllamaRequest {
        model: model.to_string(),
        prompt: rules,
        stream: false,
        options: OllamaOptions {
            temperature: 0.7,
            num_predict: 1,
        },
    };
    
    let _ = client
        .post("http://localhost:11434/api/generate")
        .json(&request)
        .timeout(std::time::Duration::from_secs(30))
        .send()
        .await?;
    
    // Thinking dots
    print!("  ");
    for i in 0..100 {
        print!(".");
        std::io::Write::flush(&mut std::io::stdout()).unwrap();
        tokio::time::sleep(tokio::time::Duration::from_millis(30)).await;
        if i % 20 == 19 {
            println!();
            print!("  ");
        }
    }
    println!("\n  {} {} is ready!", "✨", player_name.bright_green());
    
    Ok(())
}

async fn get_ollama_move(board: &GoBoard, model: &str, player_name: &str) -> anyhow::Result<(usize, usize)> {
    let valid_moves = board.get_valid_moves();
    if valid_moves.is_empty() {
        return Ok((99, 99)); // Pass
    }
    
    // Analyze board state for better prompt
    let (black_territory, white_territory) = board.calculate_territory();
    let am_black = board.current_player == BLACK;
    let my_territory = if am_black { black_territory } else { white_territory };
    let opp_territory = if am_black { white_territory } else { black_territory };
    
    // Find critical moves
    let mut critical_moves = Vec::new();
    let mut capture_moves = Vec::new();
    
    for &(row, col) in &valid_moves {
        // Simulate move
        let mut test_board = board.clone();
        test_board.board[row][col] = board.current_player;
        
        // Check if this move captures stones
        let opponent = if board.current_player == BLACK { WHITE } else { BLACK };
        let mut captures = 0;
        
        for dr in [-1i32, 0, 1] {
            for dc in [-1i32, 0, 1] {
                if dr.abs() + dc.abs() != 1 { continue; }
                let nr = row as i32 + dr;
                let nc = col as i32 + dc;
                
                if nr >= 0 && nr < BOARD_SIZE as i32 && nc >= 0 && nc < BOARD_SIZE as i32 {
                    let nr = nr as usize;
                    let nc = nc as usize;
                    if test_board.board[nr][nc] == opponent {
                        let group = test_board.get_group(&test_board.board, nr, nc);
                        if test_board.count_liberties(&test_board.board, &group) == 0 {
                            captures += group.len();
                        }
                    }
                }
            }
        }
        
        if captures > 0 {
            capture_moves.push((row, col, captures));
        }
        
        // Check if this move saves our groups
        for dr in [-1i32, 0, 1] {
            for dc in [-1i32, 0, 1] {
                if dr.abs() + dc.abs() != 1 { continue; }
                let nr = row as i32 + dr;
                let nc = col as i32 + dc;
                
                if nr >= 0 && nr < BOARD_SIZE as i32 && nc >= 0 && nc < BOARD_SIZE as i32 {
                    let nr = nr as usize;
                    let nc = nc as usize;
                    if board.board[nr][nc] == board.current_player {
                        let group = board.get_group(&board.board, nr, nc);
                        let liberties = board.count_liberties(&board.board, &group);
                        if liberties == 1 {
                            critical_moves.push((row, col));
                        }
                    }
                }
            }
        }
    }
    
    // Sort capture moves by value
    capture_moves.sort_by(|a, b| b.2.cmp(&a.2));
    
    // Convert moves to notation
    let moves_str: Vec<String> = valid_moves.iter()
        .take(20) // Limit for smaller models
        .map(|(r, c)| {
            let col_char = if *c >= 8 { 
                (b'A' + *c as u8 + 1) as char
            } else { 
                (b'A' + *c as u8) as char 
            };
            let row_num = 9 - r;
            format!("{}{}", col_char, row_num)
        })
        .collect();
    
    // Create situational analysis
    let situation = if my_territory > opp_territory + 10 {
        "You are winning comfortably. Play safe and consolidate territory."
    } else if opp_territory > my_territory + 10 {
        "You are behind. Be aggressive and invade opponent's territory."
    } else {
        "The game is close. Look for key points and efficient moves."
    };
    
    let capture_info = if !capture_moves.is_empty() {
        format!("\nCAPTURE OPPORTUNITIES: {}", 
            capture_moves.iter().take(3).map(|(r, c, n)| {
                let col_char = if *c >= 8 { (b'A' + *c as u8 + 1) as char } else { (b'A' + *c as u8) as char };
                format!("{}{} (captures {})", col_char, 9 - r, n)
            }).collect::<Vec<_>>().join(", ")
        )
    } else {
        String::new()
    };
    
    let critical_info = if !critical_moves.is_empty() {
        "\nCRITICAL: Some of your groups need defending!"
    } else {
        ""
    };
    
    let board_str = format!("{}", board);
    
    let prompt = format!(
        r#"You are {} playing Go on a 9x9 board. You are {}.

CURRENT SITUATION: {}{}{}

BOARD STATE:
{}

Game Summary: {}

STRATEGIC PRIORITIES:
1. Capture opponent stones when possible
2. Save your groups in atari (1 liberty)
3. Build territory in corners and sides
4. Connect your groups for strength
5. Reduce opponent's territory

Top moves to consider: {}

Think step by step:
1. Are there urgent moves (captures/saves)?
2. Where can I build the most territory?
3. How can I reduce opponent's area?

Choose your move and explain why. Format: position (like D5) and your reasoning."#,
        player_name,
        if board.current_player == BLACK { "Black (●)" } else { "White (○)" },
        situation,
        capture_info,
        critical_info,
        board_str,
        board.get_game_state_summary(),
        moves_str.iter().take(10).cloned().collect::<Vec<_>>().join(", ")
    );
    
    let client = reqwest::Client::new();
    let request = OllamaRequest {
        model: model.to_string(),
        prompt,
        stream: false,
        options: OllamaOptions {
            temperature: 0.3,  // Lower temperature for more consistent play
            num_predict: 150,  // More tokens for reasoning
        },
    };
    
    let start = Instant::now();
    let response = client
        .post("http://localhost:11434/api/generate")
        .json(&request)
        .timeout(std::time::Duration::from_secs(30))
        .send()
        .await?;
    
    let elapsed = start.elapsed().as_millis();
    let ollama_resp: OllamaResponse = response.json().await?;
    
    println!("  {} {} thinking... ({}ms)", "🤔", player_name.bright_yellow(), elapsed);
    
    // Parse AI response more flexibly
    let text = &ollama_resp.response;
    println!("  {} {}: {}", "💭", player_name, text.lines().next().unwrap_or("").italic());
    
    // Look for position patterns (more flexible)
    let text_upper = text.to_uppercase();
    
    // Try common position patterns: "D5", "D 5", "Play D5", "Move: D5", etc.
    for (r, c) in &valid_moves {
        let col_char = if *c >= 8 { 
            (b'A' + *c as u8 + 1) as char
        } else { 
            (b'A' + *c as u8) as char 
        };
        let row_num = 9 - r;
        
        // Try different patterns
        let patterns = vec![
            format!("{}{}", col_char, row_num),           // D5
            format!("{} {}", col_char, row_num),          // D 5
            format!("{}:{}", col_char, row_num),          // D:5
            format!("{}-{}", col_char, row_num),          // D-5
            format!("({},{})", col_char, row_num),        // (D,5)
            format!("[{},{}]", col_char, row_num),        // [D,5]
        ];
        
        for pattern in patterns {
            if text_upper.contains(&pattern) {
                println!("  {} AI chooses {}{}", "🎯", col_char, row_num);
                return Ok((*r, *c));
            }
        }
    }
    
    // Smart fallback based on game state
    if !capture_moves.is_empty() {
        println!("  {} {} playing capture move", "⚔️", player_name);
        let (r, c, _) = capture_moves[0];
        return Ok((r, c));
    }
    
    if !critical_moves.is_empty() {
        println!("  {} {} defending critical group", "🛡️", player_name);
        return Ok(critical_moves[0]);
    }
    
    // Strategic fallback
    let strategic_moves = [
        (4, 4), // Center
        (2, 2), (2, 6), (6, 2), (6, 6), // Corners
        (2, 4), (4, 2), (4, 6), (6, 4), // Side star points
    ];
    
    for &pos in &strategic_moves {
        if valid_moves.contains(&pos) {
            println!("  {} {} choosing strategic position", "🎯", player_name);
            return Ok(pos);
        }
    }
    
    // Random fallback
    println!("  {} {} choosing randomly", "🎲", player_name);
    let idx = rand::thread_rng().gen_range(0..valid_moves.len());
    Ok(valid_moves[idx])
}

fn parse_position(pos_str: &str, valid_moves: &[(usize, usize)]) -> Option<(usize, usize)> {
    let pos_str = pos_str.trim().to_uppercase();
    
    if pos_str.len() < 2 {
        return None;
    }
    
    let col_char = pos_str.chars().next()?;
    let row_str: String = pos_str.chars().skip(1).collect();
    
    let col = if col_char >= 'J' {
        (col_char as u8 - b'A' - 1) as usize
    } else {
        (col_char as u8 - b'A') as usize
    };
    
    let row = 9_usize.saturating_sub(row_str.parse::<usize>().ok()?);
    
    if row < BOARD_SIZE && col < BOARD_SIZE && valid_moves.contains(&(row, col)) {
        Some((row, col))
    } else {
        None
    }
}

async fn save_game_replay(board: &GoBoard, black_model: &str, white_model: &str) -> anyhow::Result<()> {
    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
    let filename = format!("mini_go_replay_{}.json", timestamp);
    
    let replay = serde_json::json!({
        "timestamp": timestamp.to_string(),
        "black_model": black_model,
        "white_model": white_model,
        "final_captures": board.captures,
        "total_moves": board.move_count,
        "moves": board.move_history,
    });
    
    let mut file = File::create(&filename)?;
    file.write_all(serde_json::to_string_pretty(&replay)?.as_bytes())?;
    
    println!("\n📝 Game saved to: {}", filename.bright_green());
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("{}", "🎮 Mini Go - Ollama vs Ollama (Strategic AI)".bright_cyan().bold());
    println!("{}", "=".repeat(60).bright_blue());
    println!();
    
    // Check Ollama
    println!("{}", "🔍 Checking Ollama...".yellow());
    let check = reqwest::get("http://localhost:11434/api/tags").await?;
    let models: serde_json::Value = check.json().await?;
    
    // Find available models
    let available: Vec<String> = models["models"]
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .filter_map(|m| m["name"].as_str().map(|s| s.to_string()))
        .collect();
    
    // Model selection with preference for smarter models
    let preferred_models = [
        "deepseek-r1:1.5b",
        "llama3.2:latest",
        "llama3:latest",
        "mistral:latest",
        "phi3:latest",
        "gemma:2b",
        "qwen2.5:0.5b",
    ];
    
    let mut selected_models = Vec::new();
    
    // Try to get two different models
    for model in &preferred_models {
        if available.contains(&model.to_string()) && selected_models.len() < 2 {
            selected_models.push(model.to_string());
        }
    }
    
    // Fill with available models if needed
    for model in &available {
        if selected_models.len() < 2 && !selected_models.contains(model) {
            selected_models.push(model.clone());
        }
    }
    
    if selected_models.is_empty() {
        println!("{}", "❌ No models found!".red());
        return Ok(());
    }
    
    // Use two models or same model for both
    let black_model = selected_models[0].clone();
    let white_model = selected_models.get(1).cloned().unwrap_or_else(|| black_model.clone());
    
    println!("{}", format!("✅ Black: {} | White: {}", black_model, white_model).green());
    println!();
    
    // Initialize game
    let mut board = GoBoard::new();
    let max_moves = 40; // Increased for more complete games
    let players = ["AI Black", "AI White"];
    
    // Meditation phase - AI learns the rules
    println!("{}", "📚 Pre-game Meditation Phase".bright_magenta().bold());
    println!("{}", "=".repeat(60).bright_blue());
    
    // Both AIs meditate on the rules
    meditate_on_rules(&black_model, "AI Black").await?;
    meditate_on_rules(&white_model, "AI White").await?;
    
    println!();
    println!("{}", "🏁 Starting Strategic Mini Go Match!".bright_magenta());
    println!("{}", board);
    
    let mut pass_count = 0;
    let game_start = Instant::now();
    
    for move_num in 1..=max_moves {
        if pass_count >= 2 {
            break; // Game ends after 2 passes
        }
        
        let current_player = if board.current_player == BLACK { 0 } else { 1 };
        let current_model = if current_player == 0 { &black_model } else { &white_model };
        
        println!("{}", format!("\nMove {}: {}'s turn", move_num, players[current_player]).bright_cyan());
        println!("{}", board.get_game_state_summary().bright_black());
        
        // Get move from Ollama
        match get_ollama_move(&board, current_model, players[current_player]).await {
            Ok((99, 99)) => {
                println!("  {} {} passes!", "🏳️", players[current_player].bright_yellow());
                pass_count += 1;
                board.current_player = if board.current_player == BLACK { WHITE } else { BLACK };
            }
            Ok((row, col)) => {
                let col_char = if col >= 8 { 
                    (b'A' + col as u8 + 1) as char
                } else { 
                    (b'A' + col as u8) as char 
                };
                let row_num = 9 - row;
                
                if board.make_move(row, col) {
                    let last_move = board.move_history.last().unwrap();
                    let capture_info = if !last_move.captures.is_empty() {
                        format!(" - Captured {} stones!", last_move.captures.len())
                    } else {
                        String::new()
                    };
                    
                    println!("  {} {} plays {}{}{}", 
                        if current_player == 0 { "●" } else { "○" },
                        players[current_player].bright_green(),
                        col_char, row_num,
                        capture_info.bright_red()
                    );
                    pass_count = 0;
                    println!("{}", board);
                } else {
                    println!("  {} Invalid move, passing", "❌".red());
                    pass_count += 1;
                    board.current_player = if board.current_player == BLACK { WHITE } else { BLACK };
                }
            }
            Err(e) => {
                println!("  {} Error: {}, passing", "⚠️", e);
                pass_count += 1;
                board.current_player = if board.current_player == BLACK { WHITE } else { BLACK };
            }
        }
        
        // Brief pause for readability
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
    }
    
    let game_duration = game_start.elapsed();
    
    // Final result with territory calculation
    println!();
    println!("{}", "=".repeat(60).bright_blue());
    println!("{}", "🏁 GAME OVER!".bright_yellow().bold());
    println!();
    println!("{}", "Final Board:".bright_cyan());
    println!("{}", board);
    
    let (black_territory, white_territory) = board.calculate_territory();
    
    println!("{}", "📊 Final Score:".bright_magenta());
    println!("  {} Black: {} points (Territory: {}, Captures: {})", 
        "●", 
        black_territory,
        black_territory - board.captures[0] as i32,
        board.captures[0]
    );
    println!("  {} White: {} points (Territory: {}, Captures: {})", 
        "○", 
        white_territory,
        white_territory - board.captures[1] as i32,
        board.captures[1]
    );
    
    println!();
    if black_territory > white_territory {
        println!("{}", format!("🏆 Black wins by {} points!", black_territory - white_territory).bright_green().bold());
    } else if white_territory > black_territory {
        println!("{}", format!("🏆 White wins by {} points!", white_territory - black_territory).bright_green().bold());
    } else {
        println!("{}", "🤝 It's a tie!".bright_yellow().bold());
    }
    
    println!();
    println!("⏱️  Game duration: {:.1}s", game_duration.as_secs_f32());
    println!("📊 Average move time: {:.1}s", game_duration.as_secs_f32() / board.move_count as f32);
    
    // Save replay
    if let Err(e) = save_game_replay(&board, &black_model, &white_model).await {
        println!("⚠️  Failed to save replay: {}", e);
    }
    
    println!();
    println!("{}", "🎉 Thanks for watching strategic Go AI!".bright_green());
    
    Ok(())
}