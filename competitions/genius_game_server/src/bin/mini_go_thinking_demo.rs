use colored::*;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::time::Instant;
use rand::Rng;

const BOARD_SIZE: usize = 9;
const EMPTY: u8 = 0;
const BLACK: u8 = 1;
const WHITE: u8 = 2;

#[derive(Clone)]
struct GoBoard {
    board: [[u8; BOARD_SIZE]; BOARD_SIZE],
    current_player: u8,
    captures: [u32; 2], // [black_captures, white_captures]
    move_count: u32,
    ko_point: Option<(usize, usize)>,
}

impl GoBoard {
    fn new() -> Self {
        Self {
            board: [[EMPTY; BOARD_SIZE]; BOARD_SIZE],
            current_player: BLACK,
            captures: [0, 0],
            move_count: 0,
            ko_point: None,
        }
    }

    fn is_valid_move(&self, row: usize, col: usize) -> bool {
        if self.board[row][col] != EMPTY {
            return false;
        }
        
        if let Some((ko_row, ko_col)) = self.ko_point {
            if row == ko_row && col == ko_col {
                return false;
            }
        }
        
        let mut temp_board = self.board;
        temp_board[row][col] = self.current_player;
        
        let group = self.get_group(&temp_board, row, col);
        let has_liberties = self.count_liberties(&temp_board, &group) > 0;
        
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
        
        self.board[row][col] = self.current_player;
        self.ko_point = None;
        
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
        
        if self.current_player == BLACK {
            self.captures[0] += captured_stones.len() as u32;
        } else {
            self.captures[1] += captured_stones.len() as u32;
        }
        
        if captured_stones.len() == 1 {
            let (cap_row, cap_col) = captured_stones[0];
            let placed_group = self.get_group(&self.board, row, col);
            if placed_group.len() == 1 {
                self.ko_point = Some((cap_row, cap_col));
            }
        }
        
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
                    _ => "· ".bright_black(),
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

async fn teach_ai_go_rules(model: &str, player_name: &str, is_black: bool) -> anyhow::Result<()> {
    println!("\n{}", format!("📚 Teaching {} the rules of Go...", player_name).bright_cyan());
    
    let teaching_prompt = format!(
        r#"You are about to play Go (바둑/Baduk/Weiqi) on a 9x9 board. You will be playing as {}.

COMPLETE GO RULES AND STRATEGY:

1. OBJECTIVE:
   - Control more territory than your opponent
   - Territory = empty points surrounded by your stones + captured opponent stones

2. BASIC RULES:
   - Black plays first
   - Players alternate placing one stone
   - Stones cannot move once placed
   - {} = Black stones, {} = White stones

3. CAPTURING:
   - A group of stones is captured when it has NO liberties (empty adjacent points)
   - Captured stones are removed from the board
   - You CANNOT play a move that results in your own group having no liberties (suicide)
   - UNLESS that move captures opponent stones first

4. KO RULE:
   - You cannot immediately recapture a single stone that just captured your single stone
   - This prevents infinite loops

5. BOARD NOTATION:
   - Columns: A B C D E F G H J (note: no 'I' to avoid confusion with '1')
   - Rows: 1-9 (bottom to top)
   - Example: D5 is center, A1 is bottom-left corner

6. STRATEGY FUNDAMENTALS:
   a) Opening (序盤):
      - Corners first (easiest to make territory)
      - Then sides
      - Center last
      - Good opening moves: corners (C3, C7, G3, G7), center (D5/E5)
   
   b) Life and Death (死活):
      - Groups need TWO eyes to live permanently
      - An eye = empty point surrounded by your stones that opponent cannot fill
      - One-eyed groups can be captured
   
   c) Connection and Cutting:
      - Connected stones are stronger
      - Cut opponent's stones to make them weaker
      - Diagonal connections can be cut
   
   d) Territory:
      - Build walls to surround empty space
      - Invade opponent's potential territory
      - Reduce opponent's territory from outside
   
   e) Tactics:
      - Atari (アタリ) = threatening to capture on next move (1 liberty left)
      - Net (ゲタ) = surrounding stones so they cannot escape
      - Ladder (シチョウ) = chasing stones in a diagonal pattern
      - Snapback = sacrifice stone to recapture immediately

7. IMPORTANT CONCEPTS:
   - Sente (先手) = moves opponent must respond to
   - Gote (後手) = moves that lose initiative
   - Influence = potential to make territory in the future
   - Thickness = strong walls facing the center

8. COMMON MISTAKES TO AVOID:
   - Don't play too close to opponent's strong groups
   - Don't make empty triangles (inefficient shape)
   - Don't fill your own eyes
   - Don't ignore opponent's atari threats

Now I want you to think deeply about this game. Consider:
- What would be good opening moves?
- How can you build territory efficiently?
- When should you attack vs defend?
- How to balance territory vs influence?

Take your time to internalize these rules and strategies. When you're ready to play, you'll need to analyze the board position and make strategic decisions.

Think about all of this{}I'm giving you time to process and strategize{}"#,
        if is_black { "Black (●)" } else { "White (○)" },
        "●", "○",
        "...", "..."
    );
    
    let client = reqwest::Client::new();
    let request = OllamaRequest {
        model: model.to_string(),
        prompt: teaching_prompt,
        stream: false,
        options: OllamaOptions {
            temperature: 0.1,  // Very low for learning
            num_predict: 500,  // Allow comprehensive response
        },
    };
    
    println!("  📤 Sending rules to {}...", player_name);
    let start = Instant::now();
    
    let response = client
        .post("http://localhost:11434/api/generate")
        .json(&request)
        .timeout(std::time::Duration::from_secs(60))
        .send()
        .await?;
    
    let ollama_resp: OllamaResponse = response.json().await?;
    let elapsed = start.elapsed();
    
    println!("  ✅ {} learned the rules! ({}s)", player_name, elapsed.as_secs_f32());
    
    // Show a snippet of AI's understanding
    let understanding = ollama_resp.response.chars().take(200).collect::<String>();
    println!("  💭 {}: {}{}", player_name, understanding.trim().italic(), "...".bright_black());
    
    Ok(())
}

async fn get_ollama_move(board: &GoBoard, model: &str, player_name: &str) -> anyhow::Result<(usize, usize)> {
    let valid_moves = board.get_valid_moves();
    if valid_moves.is_empty() {
        return Ok((99, 99)); // Pass
    }
    
    let moves_str: Vec<String> = valid_moves.iter()
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
    
    let board_str = format!("{}", board);
    
    let prompt = format!(
        r#"You are {} playing Go. Current board:

{}

Move {}: Your turn as {}.

Legal moves: {}

Based on your understanding of Go strategy, choose your move.
Reply with JSON: {{"position": "D5", "reasoning": "strategic explanation"}}"#,
        player_name,
        board_str,
        board.move_count + 1,
        if board.current_player == BLACK { "Black (●)" } else { "White (○)" },
        moves_str.join(", ")
    );
    
    let client = reqwest::Client::new();
    let request = OllamaRequest {
        model: model.to_string(),
        prompt,
        stream: false,
        options: OllamaOptions {
            temperature: 0.5,
            num_predict: 150,
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
    
    println!("  🤔 {} thinking... ({}ms)", player_name.bright_yellow(), elapsed);
    
    // Try to parse JSON response
    let text = &ollama_resp.response;
    
    // Try to extract JSON
    if let Some(start) = text.find('{') {
        if let Some(end) = text.rfind('}') {
            if let Ok(decision) = serde_json::from_str::<GoDecision>(&text[start..=end]) {
                println!("  💭 {}: {}", player_name, decision.reasoning.italic());
                
                // Parse position
                let pos_str = decision.position.trim().to_uppercase();
                if pos_str.len() >= 2 {
                    if let Some(col_char) = pos_str.chars().next() {
                        if let Some(row_char) = pos_str.chars().nth(1) {
                            let col = if col_char >= 'J' {
                                (col_char as u8 - b'A' - 1) as usize
                            } else {
                                (col_char as u8 - b'A') as usize
                            };
                            if let Some(row_digit) = row_char.to_digit(10) {
                                let row = 9_usize.saturating_sub(row_digit as usize);
                                
                                if row < BOARD_SIZE && col < BOARD_SIZE && valid_moves.contains(&(row, col)) {
                                    return Ok((row, col));
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    // Fallback to strategic positions
    let strategic_moves = [
        (4, 4), // Center
        (2, 2), (2, 6), (6, 2), (6, 6), // Corners
        (2, 4), (4, 2), (4, 6), (6, 4), // Side points
    ];
    
    for &pos in &strategic_moves {
        if valid_moves.contains(&pos) {
            println!("  🎯 {} choosing strategic position", player_name);
            return Ok(pos);
        }
    }
    
    // Random fallback
    println!("  🎲 {} choosing randomly", player_name);
    let idx = rand::thread_rng().gen_range(0..valid_moves.len());
    Ok(valid_moves[idx])
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("{}", "🎮 Mini Go - AI with Deep Thinking Time".bright_cyan().bold());
    println!("{}", "=".repeat(60).bright_blue());
    println!();
    
    // Check Ollama
    println!("{}", "🔍 Checking Ollama...".yellow());
    let check = reqwest::get("http://localhost:11434/api/tags").await?;
    let models: serde_json::Value = check.json().await?;
    
    let available: Vec<String> = models["models"]
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .filter_map(|m| m["name"].as_str().map(|s| s.to_string()))
        .collect();
    
    if available.is_empty() {
        println!("{}", "❌ No models found!".red());
        return Ok(());
    }
    
    // Select model
    let model = available.iter()
        .find(|m| m.contains("deepseek") || m.contains("llama") || m.contains("mistral"))
        .cloned()
        .unwrap_or_else(|| available[0].clone());
    
    println!("{}", format!("✅ Using model: {}", model).green());
    println!();
    
    // Teaching phase
    println!("{}", "🎓 TEACHING PHASE".bright_magenta().bold());
    println!("{}", "-".repeat(60));
    
    // Teach both AIs the rules
    teach_ai_go_rules(&model, "AI Black", true).await?;
    teach_ai_go_rules(&model, "AI White", false).await?;
    
    println!();
    println!("{}", "🧠 Giving AIs time to think deeply about Go strategy...".bright_yellow());
    
    // Visual thinking time
    print!("  ");
    for i in 0..100 {
        print!("{}", ".".bright_black());
        std::io::Write::flush(&mut std::io::stdout())?;
        tokio::time::sleep(tokio::time::Duration::from_millis(20)).await;
        
        if i % 25 == 24 {
            println!();
            print!("  ");
        }
    }
    println!();
    println!();
    
    println!("{}", "✅ AIs have finished contemplating Go strategy!".bright_green());
    println!();
    
    // Game phase
    println!("{}", "🏁 STARTING GAME".bright_magenta().bold());
    println!("{}", "-".repeat(60));
    
    let mut board = GoBoard::new();
    let max_moves = 30;
    let players = ["AI Black", "AI White"];
    
    println!("{}", board);
    
    let mut pass_count = 0;
    
    for move_num in 1..=max_moves {
        if pass_count >= 2 {
            break;
        }
        
        let current_player = if board.current_player == BLACK { 0 } else { 1 };
        println!("{}", format!("\nMove {}: {}'s turn", move_num, players[current_player]).bright_cyan());
        
        match get_ollama_move(&board, &model, players[current_player]).await {
            Ok((99, 99)) => {
                println!("  🏳️ {} passes!", players[current_player].bright_yellow());
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
                    println!("  {} {} plays {}{}", 
                        if current_player == 0 { "●" } else { "○" },
                        players[current_player].bright_green(),
                        col_char, row_num
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
                println!("  ⚠️ Error: {}, passing", e);
                pass_count += 1;
                board.current_player = if board.current_player == BLACK { WHITE } else { BLACK };
            }
        }
        
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    }
    
    // Final result
    println!();
    println!("{}", "=".repeat(60).bright_blue());
    println!("{}", "🏁 GAME OVER!".bright_yellow().bold());
    println!();
    println!("{}", "Final Board:".bright_cyan());
    println!("{}", board);
    
    println!("{}", "📊 Final Score:".bright_magenta());
    println!("  ● Black captures: {}", board.captures[0]);
    println!("  ○ White captures: {}", board.captures[1]);
    println!();
    
    println!("{}", "🎉 Thanks for watching AI Go with thinking time!".bright_green());
    
    Ok(())
}