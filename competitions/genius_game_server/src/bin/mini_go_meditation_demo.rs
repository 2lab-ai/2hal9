use colored::*;
use reqwest;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::time::Instant;
use rand::Rng;
use std::io::{self, Write as IoWrite};

const BOARD_SIZE: usize = 9;
const EMPTY: u8 = 0;
const BLACK: u8 = 1;
const WHITE: u8 = 2;

#[derive(Clone)]
struct GoBoard {
    board: [[u8; BOARD_SIZE]; BOARD_SIZE],
    current_player: u8,
    captures: [u32; 2],
    move_count: u32,
    ko_point: Option<(usize, usize)>,
    move_history: Vec<GameMove>,
}

#[derive(Clone)]
struct GameMove {
    player: u8,
    position: String,
    reasoning: String,
    #[allow(dead_code)]
    board_after: [[u8; BOARD_SIZE]; BOARD_SIZE],
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

    fn make_move(&mut self, row: usize, col: usize, reasoning: &str) -> bool {
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
        
        // Save move history
        let col_char = if col >= 8 { 
            (b'A' + col as u8 + 1) as char
        } else { 
            (b'A' + col as u8) as char 
        };
        let position = format!("{}{}", col_char, 9 - row);
        
        self.move_history.push(GameMove {
            player: self.current_player,
            position,
            reasoning: reasoning.to_string(),
            board_after: self.board,
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

async fn meditation_phase(model: &str, player_name: &str, is_black: bool) -> anyhow::Result<()> {
    println!("\n{}", format!("🧘 {} 명상 시작...", player_name).bright_cyan());
    
    let meditation_prompt = format!(
        r#"당신은 곧 바둑을 둘 것입니다.
돌을 놓고, 영역을 만들고, 상대를 포위하는 게임입니다.

당신은 {} 돌을 가지고 플레이합니다.
● = 흑돌 (선공)
○ = 백돌 (후공)

바둑판은 9x9 크기입니다.
좌표는 A-J (I 제외), 1-9로 표시됩니다.

중요한 위치들:
- 중앙: E5
- 코너: C3, C7, G3, G7
- 변: 가장자리의 중간 지점들

이제 이 게임에 대해 깊이 생각해보세요...

{}"#,
        if is_black { "흑" } else { "백" },
        ".".repeat(100)
    );
    
    let client = reqwest::Client::new();
    let request = OllamaRequest {
        model: model.to_string(),
        prompt: meditation_prompt,
        stream: false,
        options: OllamaOptions {
            temperature: 0.3,
            num_predict: 200,
        },
    };
    
    print!("  ");
    for i in 0..50 {
        print!("{}", ".".bright_black());
        io::stdout().flush()?;
        tokio::time::sleep(tokio::time::Duration::from_millis(40)).await;
        if i % 25 == 24 {
            println!();
            print!("  ");
        }
    }
    
    let start = Instant::now();
    let response = client
        .post("http://localhost:11434/api/generate")
        .json(&request)
        .timeout(std::time::Duration::from_secs(60))
        .send()
        .await?;
    
    let _ollama_resp: OllamaResponse = response.json().await?;
    let elapsed = start.elapsed();
    
    println!();
    println!("  {} {} 명상 완료! ({}s)", "✅", player_name, elapsed.as_secs_f32());
    
    Ok(())
}

async fn get_ollama_move(board: &GoBoard, model: &str, player_name: &str) -> anyhow::Result<(usize, usize, String)> {
    let valid_moves = board.get_valid_moves();
    if valid_moves.is_empty() {
        return Ok((99, 99, "No valid moves".to_string()));
    }
    
    // Build context from move history
    let mut context = String::new();
    if !board.move_history.is_empty() {
        context.push_str("\n최근 수들:\n");
        for (i, m) in board.move_history.iter().rev().take(5).enumerate() {
            context.push_str(&format!(
                "- {} 전: {} {}\n", 
                i + 1,
                if m.player == BLACK { "흑" } else { "백" },
                m.position
            ));
        }
    }
    
    let board_str = format!("{}", board);
    
    let prompt = format!(
        r#"현재 바둑판 상태:
{}{}

당신은 {} {}입니다.
어디에 두시겠습니까? (예: D5, E7, C3)

간단히 이유도 말씀해주세요."#,
        board_str,
        context,
        player_name,
        if board.current_player == BLACK { "(흑)" } else { "(백)" }
    );
    
    let client = reqwest::Client::new();
    let request = OllamaRequest {
        model: model.to_string(),
        prompt,
        stream: false,
        options: OllamaOptions {
            temperature: 0.5,
            num_predict: 100,
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
    
    println!("  {} {} 생각 중... ({}ms)", "🤔", player_name.bright_yellow(), elapsed);
    
    let text = &ollama_resp.response;
    
    // Extract position from natural language
    let position = extract_position(text, &valid_moves);
    
    // Extract reasoning
    let reasoning = text.lines()
        .skip_while(|line| line.trim().is_empty())
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .take(100)
        .collect::<String>();
    
    if let Some((row, col)) = position {
        println!("  {} {}: {}", "💭", player_name, reasoning.italic());
        Ok((row, col, reasoning))
    } else {
        // Strategic fallback
        let strategic_moves = [
            (4, 4), // Center
            (2, 2), (2, 6), (6, 2), (6, 6), // Corners
            (2, 4), (4, 2), (4, 6), (6, 4), // Side points
        ];
        
        for &pos in &strategic_moves {
            if valid_moves.contains(&pos) {
                let reason = "전략적 요충지 선택".to_string();
                println!("  {} {} {}", "🎯", player_name, reason.italic());
                return Ok((pos.0, pos.1, reason));
            }
        }
        
        // Random fallback
        let idx = rand::thread_rng().gen_range(0..valid_moves.len());
        let pos = valid_moves[idx];
        let reason = "가능한 수 중 선택".to_string();
        println!("  {} {} {}", "🎲", player_name, reason.italic());
        Ok((pos.0, pos.1, reason))
    }
}

fn extract_position(text: &str, valid_moves: &[(usize, usize)]) -> Option<(usize, usize)> {
    let text_upper = text.to_uppercase();
    
    // Try various patterns
    let _patterns = [
        // English notation: D5, E7, etc.
        r"([A-HJ])(\d)",
        // Korean: 4열 5행
        r"(\d)열\s*(\d)행",
        // Korean: 디5, 이7
        r"([가-힣])(\d)",
    ];
    
    // Check for standard notation (A-J, 1-9)
    for row in 0..BOARD_SIZE {
        for col in 0..BOARD_SIZE {
            let col_char = if col >= 8 { 
                (b'A' + col as u8 + 1) as char
            } else { 
                (b'A' + col as u8) as char 
            };
            let row_num = 9 - row;
            
            let pos_str = format!("{}{}", col_char, row_num);
            if text_upper.contains(&pos_str) && valid_moves.contains(&(row, col)) {
                return Some((row, col));
            }
            
            // Also check with space: "D 5"
            let pos_str_space = format!("{} {}", col_char, row_num);
            if text_upper.contains(&pos_str_space) && valid_moves.contains(&(row, col)) {
                return Some((row, col));
            }
        }
    }
    
    None
}

async fn ask_why(model: &str, player_name: &str, last_move: &GameMove) -> anyhow::Result<()> {
    println!("\n{}", format!("🤷 {}에게 물어봅니다...", player_name).bright_cyan());
    
    let prompt = format!(
        "방금 {}에 두셨는데, 왜 그곳을 선택하셨나요? 짧게 설명해주세요.",
        last_move.position
    );
    
    let client = reqwest::Client::new();
    let request = OllamaRequest {
        model: model.to_string(),
        prompt,
        stream: false,
        options: OllamaOptions {
            temperature: 0.7,
            num_predict: 50,
        },
    };
    
    let response = client
        .post("http://localhost:11434/api/generate")
        .json(&request)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await?;
    
    let ollama_resp: OllamaResponse = response.json().await?;
    println!("  {} {}: {}", "💬", player_name, ollama_resp.response.trim().italic());
    
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("{}", "🎮 Mini Go - 명상하는 AI (Meditation AI)".bright_cyan().bold());
    println!("{}", "=".repeat(60).bright_blue());
    println!();
    
    // Check Ollama
    println!("{}", "🔍 Ollama 확인 중...".yellow());
    let check = reqwest::get("http://localhost:11434/api/tags").await?;
    let models: serde_json::Value = check.json().await?;
    
    let available: Vec<String> = models["models"]
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .filter_map(|m| m["name"].as_str().map(|s| s.to_string()))
        .collect();
    
    if available.is_empty() {
        println!("{}", "❌ 모델을 찾을 수 없습니다!".red());
        return Ok(());
    }
    
    let model = available.iter()
        .find(|m| m.contains("deepseek") || m.contains("llama") || m.contains("qwen"))
        .cloned()
        .unwrap_or_else(|| available[0].clone());
    
    println!("{}", format!("✅ 사용 모델: {}", model).green());
    println!();
    
    // Meditation phase
    println!("{}", "🧘 명상 단계 (Meditation Phase)".bright_magenta().bold());
    println!("{}", "-".repeat(60));
    
    meditation_phase(&model, "AI 흑", true).await?;
    meditation_phase(&model, "AI 백", false).await?;
    
    println!();
    println!("{}", "✨ AI들이 바둑에 대한 이해를 깊게 했습니다!".bright_green());
    println!();
    
    // Game phase
    println!("{}", "🏁 게임 시작".bright_magenta().bold());
    println!("{}", "-".repeat(60));
    
    let mut board = GoBoard::new();
    let max_moves = 20;
    let players = ["AI 흑", "AI 백"];
    
    println!("{}", board);
    
    let mut pass_count = 0;
    
    for move_num in 1..=max_moves {
        if pass_count >= 2 {
            break;
        }
        
        let current_player = if board.current_player == BLACK { 0 } else { 1 };
        println!("{}", format!("\n수 {}: {}의 차례", move_num, players[current_player]).bright_cyan());
        
        match get_ollama_move(&board, &model, players[current_player]).await {
            Ok((99, 99, _)) => {
                println!("  {} {} 패스!", "🏳️", players[current_player].bright_yellow());
                pass_count += 1;
                board.current_player = if board.current_player == BLACK { WHITE } else { BLACK };
            }
            Ok((row, col, reasoning)) => {
                let col_char = if col >= 8 { 
                    (b'A' + col as u8 + 1) as char
                } else { 
                    (b'A' + col as u8) as char 
                };
                let row_num = 9 - row;
                
                if board.make_move(row, col, &reasoning) {
                    println!("  {} {} → {}{}", 
                        if current_player == 0 { "●" } else { "○" },
                        players[current_player].bright_green(),
                        col_char, row_num
                    );
                    pass_count = 0;
                    println!("{}", board);
                    
                    // Sometimes ask why
                    if move_num > 3 && rand::thread_rng().gen_bool(0.3) {
                        if let Some(last_move) = board.move_history.last() {
                            let _ = ask_why(&model, players[current_player], last_move).await;
                        }
                    }
                } else {
                    println!("  {} 잘못된 수, 패스", "❌".red());
                    pass_count += 1;
                    board.current_player = if board.current_player == BLACK { WHITE } else { BLACK };
                }
            }
            Err(e) => {
                println!("  {} 에러: {}, 패스", "⚠️", e);
                pass_count += 1;
                board.current_player = if board.current_player == BLACK { WHITE } else { BLACK };
            }
        }
        
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    }
    
    // Final conversation
    println!();
    println!("{}", "=".repeat(60).bright_blue());
    println!("{}", "🏁 게임 종료!".bright_yellow().bold());
    println!();
    println!("{}", "최종 보드:".bright_cyan());
    println!("{}", board);
    
    println!("{}", "📊 최종 점수:".bright_magenta());
    println!("  {} 흑 포획: {}", "●", board.captures[0]);
    println!("  {} 백 포획: {}", "○", board.captures[1]);
    println!();
    
    println!("{}", "💬 게임 대화록:".bright_cyan());
    for (i, m) in board.move_history.iter().enumerate() {
        println!("  {}. {} {}: {}", 
            i + 1,
            if m.player == BLACK { "흑" } else { "백" },
            m.position,
            m.reasoning.chars().take(50).collect::<String>()
        );
    }
    
    println!();
    println!("{}", "🎉 명상하는 AI 바둑을 시청해주셔서 감사합니다!".bright_green());
    
    Ok(())
}