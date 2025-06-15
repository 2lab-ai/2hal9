use anyhow::Result;
use genius_game_server::games::{
    Action, GameConfig, GameEngine, GameType
};
use std::collections::HashMap;
use colored::*;
use tokio::time::{sleep, Duration};
use serde_json::json;
use rand::Rng;

const ANIMATION_DELAY: u64 = 50;
const ROUND_DELAY: u64 = 200;

#[tokio::main]
async fn main() -> Result<()> {
    println!("\n{}", "═".repeat(80).bright_red());
    println!("{}", "ULTIMATE DEATH GAME COLLECTION".bright_red().bold());
    println!("{}", "═".repeat(80).bright_red());
    
    // Show menu
    println!("\n{}", "Available Death Games:".bright_yellow());
    println!("1. {} - Classic deadly bluff", "Squid Game".bright_cyan());
    println!("2. {} - 9x9 strategic Go", "Mini Go".bright_cyan());
    println!("3. {} - High-stakes Texas Hold'em", "Mini Hold'em".bright_cyan());
    println!("4. {} - Last player standing wins", "Battle Royale".bright_cyan());
    println!("5. {} - Survival in the arena", "Hunger Games".bright_cyan());
    println!("6. {} - Bluff or die", "Liar's Dice".bright_cyan());
    println!("7. {} - Test your luck", "Russian Roulette".bright_cyan());
    println!("8. {} - Control the center", "King of the Hill".bright_cyan());
    println!("9. {} - Survive the waves", "Last Stand".bright_cyan());
    println!("10. {} - Trust or betray", "Trust Fall".bright_cyan());
    println!("11. {} - Play all games!", "ALL GAMES".bright_magenta().bold());
    
    println!("\n{}", "Select a game (1-11): ".bright_yellow());
    
    // For demo, we'll run all games
    tokio::time::sleep(Duration::from_secs(1)).await;
    println!("{}", "Running ALL GAMES demo...".bright_magenta().bold());
    
    // Run each game
    run_squid_game_demo().await?;
    run_mini_go_demo().await?;
    run_mini_holdem_demo().await?;
    run_battle_royale_demo().await?;
    run_hunger_games_demo().await?;
    run_liars_dice_demo().await?;
    run_russian_roulette_demo().await?;
    run_king_of_the_hill_demo().await?;
    run_last_stand_demo().await?;
    run_trust_fall_demo().await?;
    
    println!("\n{}", "═".repeat(80).bright_green());
    println!("{}", "ALL GAMES COMPLETED!".bright_green().bold());
    println!("{}", "═".repeat(80).bright_green());
    
    Ok(())
}

async fn run_squid_game_demo() -> Result<()> {
    print_game_header("SQUID GAME", "Red Light, Green Light - Move or Die!").await;
    
    let engine = GameEngine::new();
    let config = GameConfig {
        game_type: GameType::SquidGame,
        rounds: 5,
        time_limit_ms: 5000,
        special_rules: HashMap::new(),
    };
    
    let game_id = engine.create_game(config).await?;
    let players = vec!["Player-456", "Player-218", "Player-067", "Player-001"];
    
    for round in 1..=5 {
        println!("\n{}", format!("Round {}", round).bright_yellow());
        
        let phase = if round % 2 == 1 { "green" } else { "red" };
        println!("Light: {}", if phase == "green" { 
            "GREEN - MOVE!".bright_green() 
        } else { 
            "RED - FREEZE!".bright_red() 
        });
        
        let mut actions = HashMap::new();
        for player in &players {
            let action = if phase == "green" || rand::thread_rng().gen_bool(0.3) {
                "move"
            } else {
                "freeze"
            };
            
            actions.insert(player.to_string(), Action {
                player_id: player.to_string(),
                action_type: action.to_string(),
                data: json!({"distance": rand::thread_rng().gen_range(1..10)}),
                reasoning: None,
                confidence: None,
            });
            
            animate_action(&format!("{} → {}", player, action)).await;
        }
        
        let result = engine.process_turn(game_id, actions).await?;
        
        if !result.outcome.losers.is_empty() {
            for loser in &result.outcome.losers {
                println!("{}", format!("💀 {} eliminated!", loser).bright_red());
            }
        }
        
        sleep(Duration::from_millis(ROUND_DELAY)).await;
    }
    
    let final_result = engine.finalize_game(game_id).await?;
    print_game_result("SQUID GAME", &final_result.winner).await;
    
    Ok(())
}

async fn run_mini_go_demo() -> Result<()> {
    print_game_header("MINI GO", "9x9 Strategic Battle").await;
    
    let engine = GameEngine::new();
    let config = GameConfig {
        game_type: GameType::MiniGo,
        rounds: 10,
        time_limit_ms: 5000,
        special_rules: HashMap::new(),
    };
    
    let game_id = engine.create_game(config).await?;
    let players = ["Black-Master", "White-Challenger"];
    
    for round in 1..=10 {
        println!("\n{}", format!("Move {}", round).bright_yellow());
        
        let mut actions = HashMap::new();
        for (i, player) in players.iter().enumerate() {
            let row = rand::thread_rng().gen_range(0..9);
            let col = rand::thread_rng().gen_range(0..9);
            
            actions.insert(player.to_string(), Action {
                player_id: player.to_string(),
                action_type: "place".to_string(),
                data: json!({"row": row, "col": col}),
                reasoning: None,
                confidence: None,
            });
            
            let stone = if i == 0 { "⚫" } else { "⚪" };
            animate_action(&format!("{} {} → ({}, {})", player, stone, row, col)).await;
        }
        
        let result = engine.process_turn(game_id, actions).await?;
        
        for event in &result.outcome.special_events {
            println!("{}", event.bright_cyan());
        }
        
        sleep(Duration::from_millis(ROUND_DELAY)).await;
    }
    
    let final_result = engine.finalize_game(game_id).await?;
    print_game_result("MINI GO", &final_result.winner).await;
    
    Ok(())
}

async fn run_mini_holdem_demo() -> Result<()> {
    print_game_header("MINI HOLD'EM", "High Stakes Poker").await;
    
    let engine = GameEngine::new();
    let config = GameConfig {
        game_type: GameType::MiniHoldem,
        rounds: 5,
        time_limit_ms: 5000,
        special_rules: HashMap::new(),
    };
    
    let game_id = engine.create_game(config).await?;
    let players = vec!["Ace-High", "Bluff-King", "All-In-Annie", "Fold-Frank"];
    
    for round in 1..=5 {
        println!("\n{}", format!("Hand {}", round).bright_yellow());
        
        let mut actions = HashMap::new();
        for player in &players {
            let action_type = match rand::thread_rng().gen_range(0..4) {
                0 => "fold",
                1 => "call",
                2 => "raise",
                _ => "check",
            };
            
            let mut data = HashMap::new();
            if action_type == "raise" {
                data.insert("amount", json!(rand::thread_rng().gen_range(10..50)));
            }
            
            actions.insert(player.to_string(), Action {
                player_id: player.to_string(),
                action_type: action_type.to_string(),
                data: json!(data),
                reasoning: None,
                confidence: None,
            });
            
            let icon = match action_type {
                "fold" => "🏳️",
                "call" => "📞",
                "raise" => "💰",
                _ => "✓",
            };
            animate_action(&format!("{} {} {}", player, icon, action_type)).await;
        }
        
        engine.process_turn(game_id, actions).await?;
        sleep(Duration::from_millis(ROUND_DELAY)).await;
    }
    
    let final_result = engine.finalize_game(game_id).await?;
    print_game_result("MINI HOLD'EM", &final_result.winner).await;
    
    Ok(())
}

async fn run_battle_royale_demo() -> Result<()> {
    print_game_header("BATTLE ROYALE", "100 Players Drop - 1 Survives").await;
    
    let engine = GameEngine::new();
    let config = GameConfig {
        game_type: GameType::BattleRoyale,
        rounds: 15,
        time_limit_ms: 5000,
        special_rules: HashMap::new(),
    };
    
    let game_id = engine.create_game(config).await?;
    let players = vec!["Alpha", "Bravo", "Charlie", "Delta", "Echo", "Foxtrot"];
    
    for round in 1..=15 {
        println!("\n{}", format!("Zone {}", round).bright_yellow());
        
        if round % 5 == 0 {
            println!("{}", "⚡ STORM CLOSING!".bright_red().blink());
        }
        
        let mut actions = HashMap::new();
        for player in &players {
            let action_type = match rand::thread_rng().gen_range(0..4) {
                0 => "move",
                1 => "attack",
                2 => "loot",
                _ => "hide",
            };
            
            let data = match action_type {
                "move" => json!("North"),
                "attack" => json!(players[rand::thread_rng().gen_range(0..players.len())]),
                _ => json!({}),
            };
            
            actions.insert(player.to_string(), Action {
                player_id: player.to_string(),
                action_type: action_type.to_string(),
                data,
                reasoning: None,
                confidence: None,
            });
            
            let icon = match action_type {
                "move" => "🏃",
                "attack" => "🔫",
                "loot" => "📦",
                _ => "🌳",
            };
            animate_action(&format!("{} {} {}", player, icon, action_type)).await;
        }
        
        let result = engine.process_turn(game_id, actions).await?;
        
        if !result.outcome.losers.is_empty() {
            for loser in &result.outcome.losers {
                println!("{}", format!("💀 {} eliminated!", loser).bright_red());
            }
        }
        
        sleep(Duration::from_millis(ROUND_DELAY)).await;
    }
    
    let final_result = engine.finalize_game(game_id).await?;
    print_game_result("BATTLE ROYALE", &final_result.winner).await;
    
    Ok(())
}

async fn run_hunger_games_demo() -> Result<()> {
    print_game_header("HUNGER GAMES", "May the Odds Be Ever in Your Favor").await;
    
    let engine = GameEngine::new();
    let config = GameConfig {
        game_type: GameType::HungerGames,
        rounds: 10,
        time_limit_ms: 5000,
        special_rules: HashMap::new(),
    };
    
    let game_id = engine.create_game(config).await?;
    let players = vec!["Katniss", "Peeta", "Cato", "Foxface", "Thresh", "Clove"];
    
    for round in 1..=10 {
        println!("\n{}", format!("Day {}", round).bright_yellow());
        
        if round % 3 == 0 {
            println!("{}", "🔥 GAMEMAKER EVENT!".bright_red().blink());
        }
        
        let mut actions = HashMap::new();
        for player in &players {
            let action_type = match rand::thread_rng().gen_range(0..6) {
                0 => "hunt",
                1 => "gather",
                2 => "form_alliance",
                3 => "betray",
                4 => "move",
                _ => "use_item",
            };
            
            let data = match action_type {
                "hunt" => json!(players[rand::thread_rng().gen_range(0..players.len())]),
                "form_alliance" => json!(players[rand::thread_rng().gen_range(0..players.len())]),
                "move" => json!("Forest"),
                "use_item" => json!("food"),
                _ => json!({}),
            };
            
            actions.insert(player.to_string(), Action {
                player_id: player.to_string(),
                action_type: action_type.to_string(),
                data,
                reasoning: None,
                confidence: None,
            });
            
            animate_action(&format!("{} → {}", player, action_type)).await;
        }
        
        let result = engine.process_turn(game_id, actions).await?;
        
        for event in &result.outcome.special_events {
            println!("{}", event.bright_cyan());
        }
        
        sleep(Duration::from_millis(ROUND_DELAY)).await;
    }
    
    let final_result = engine.finalize_game(game_id).await?;
    print_game_result("HUNGER GAMES", &final_result.winner).await;
    
    Ok(())
}

async fn run_liars_dice_demo() -> Result<()> {
    print_game_header("LIAR'S DICE", "Bluff or Call - Risk it All").await;
    
    let engine = GameEngine::new();
    let config = GameConfig {
        game_type: GameType::LiarsDice,
        rounds: 15,
        time_limit_ms: 5000,
        special_rules: HashMap::new(),
    };
    
    let game_id = engine.create_game(config).await?;
    let players = ["Bluffer", "Caller", "Counter", "Deceiver"];
    
    for round in 1..=15 {
        println!("\n{}", format!("Round {}", round).bright_yellow());
        
        let mut actions = HashMap::new();
        let mut current_bid = (2, 3); // quantity, face_value
        
        for (i, player) in players.iter().enumerate() {
            let action_type = if i == 0 || rand::thread_rng().gen_bool(0.7) {
                current_bid.0 += 1;
                "bid"
            } else {
                "challenge"
            };
            
            let data = if action_type == "bid" {
                json!({"quantity": current_bid.0, "face_value": current_bid.1})
            } else {
                json!({})
            };
            
            actions.insert(player.to_string(), Action {
                player_id: player.to_string(),
                action_type: action_type.to_string(),
                data,
                reasoning: None,
                confidence: None,
            });
            
            if action_type == "bid" {
                animate_action(&format!("{} 🎲 bids {} {}s", player, current_bid.0, current_bid.1)).await;
            } else {
                animate_action(&format!("{} ❌ CHALLENGE!", player)).await;
                break;
            }
        }
        
        engine.process_turn(game_id, actions).await?;
        sleep(Duration::from_millis(ROUND_DELAY)).await;
    }
    
    let final_result = engine.finalize_game(game_id).await?;
    print_game_result("LIAR'S DICE", &final_result.winner).await;
    
    Ok(())
}

async fn run_russian_roulette_demo() -> Result<()> {
    print_game_header("RUSSIAN ROULETTE", "One Bullet. Six Chambers. Pure Luck.").await;
    
    let engine = GameEngine::new();
    let config = GameConfig {
        game_type: GameType::RussianRoulette,
        rounds: 20,
        time_limit_ms: 5000,
        special_rules: HashMap::new(),
    };
    
    let game_id = engine.create_game(config).await?;
    let players = vec!["Brave", "Lucky", "Nervous", "Fearless"];
    
    for round in 1..=20 {
        println!("\n{}", format!("Turn {}", round).bright_yellow());
        
        let mut actions = HashMap::new();
        for player in &players {
            let action_type = match rand::thread_rng().gen_range(0..3) {
                0 => "spin",
                1 => "pull",
                _ => "pass",
            };
            
            actions.insert(player.to_string(), Action {
                player_id: player.to_string(),
                action_type: action_type.to_string(),
                data: json!({}),
                reasoning: None,
                confidence: None,
            });
            
            match action_type {
                "spin" => animate_action(&format!("{} 🔄 spins the cylinder", player)).await,
                "pull" => {
                    animate_action(&format!("{} 🔫 pulls the trigger...", player)).await;
                    sleep(Duration::from_millis(500)).await;
                    if rand::thread_rng().gen_bool(0.167) {
                        println!("{}", "💥 BANG!".bright_red().bold());
                    } else {
                        println!("{}", "*click*".bright_green());
                    }
                }
                _ => animate_action(&format!("{} ➡️ passes", player)).await,
            }
        }
        
        engine.process_turn(game_id, actions).await?;
        sleep(Duration::from_millis(ROUND_DELAY)).await;
    }
    
    let final_result = engine.finalize_game(game_id).await?;
    print_game_result("RUSSIAN ROULETTE", &final_result.winner).await;
    
    Ok(())
}

async fn run_king_of_the_hill_demo() -> Result<()> {
    print_game_header("KING OF THE HILL", "Control the Center - Defend Your Crown").await;
    
    let engine = GameEngine::new();
    let config = GameConfig {
        game_type: GameType::KingOfTheHill,
        rounds: 10,
        time_limit_ms: 5000,
        special_rules: HashMap::new(),
    };
    
    let game_id = engine.create_game(config).await?;
    let players = vec!["Defender", "Challenger", "Usurper", "Conqueror"];
    
    for round in 1..=10 {
        println!("\n{}", format!("Battle {}", round).bright_yellow());
        
        let mut actions = HashMap::new();
        for player in &players {
            let action_type = match rand::thread_rng().gen_range(0..4) {
                0 => "push",
                1 => "fortify",
                2 => "charge",
                _ => "defend",
            };
            
            let data = if action_type == "push" {
                json!({"target": players[rand::thread_rng().gen_range(0..players.len())]})
            } else {
                json!({})
            };
            
            actions.insert(player.to_string(), Action {
                player_id: player.to_string(),
                action_type: action_type.to_string(),
                data,
                reasoning: None,
                confidence: None,
            });
            
            let icon = match action_type {
                "push" => "🤜",
                "fortify" => "🛡️",
                "charge" => "⚔️",
                _ => "🏰",
            };
            animate_action(&format!("{} {} {}", player, icon, action_type)).await;
        }
        
        engine.process_turn(game_id, actions).await?;
        sleep(Duration::from_millis(ROUND_DELAY)).await;
    }
    
    let final_result = engine.finalize_game(game_id).await?;
    print_game_result("KING OF THE HILL", &final_result.winner).await;
    
    Ok(())
}

async fn run_last_stand_demo() -> Result<()> {
    print_game_header("LAST STAND", "Survive the Endless Waves").await;
    
    let engine = GameEngine::new();
    let config = GameConfig {
        game_type: GameType::LastStand,
        rounds: 8,
        time_limit_ms: 5000,
        special_rules: HashMap::new(),
    };
    
    let game_id = engine.create_game(config).await?;
    let players = vec!["Survivor", "Fighter", "Builder", "Medic"];
    
    for round in 1..=8 {
        println!("\n{}", format!("Wave {}", round).bright_yellow());
        println!("{}", format!("Threat Level: {}", "▮".repeat(round)).bright_red());
        
        let mut actions = HashMap::new();
        for player in &players {
            let action_type = match rand::thread_rng().gen_range(0..5) {
                0 => "shoot",
                1 => "fortify",
                2 => "heal",
                3 => "share",
                _ => "scavenge",
            };
            
            let data = match action_type {
                "shoot" => json!({"target": format!("enemy_{}", rand::thread_rng().gen_range(1..5))}),
                "share" => json!({"with": players[rand::thread_rng().gen_range(0..players.len())], "resource": "ammo"}),
                _ => json!({}),
            };
            
            actions.insert(player.to_string(), Action {
                player_id: player.to_string(),
                action_type: action_type.to_string(),
                data,
                reasoning: None,
                confidence: None,
            });
            
            let icon = match action_type {
                "shoot" => "🔫",
                "fortify" => "🧱",
                "heal" => "💉",
                "share" => "🤝",
                _ => "🔍",
            };
            animate_action(&format!("{} {} {}", player, icon, action_type)).await;
        }
        
        engine.process_turn(game_id, actions).await?;
        sleep(Duration::from_millis(ROUND_DELAY)).await;
    }
    
    let final_result = engine.finalize_game(game_id).await?;
    print_game_result("LAST STAND", &final_result.winner).await;
    
    Ok(())
}

async fn run_trust_fall_demo() -> Result<()> {
    print_game_header("TRUST FALL", "Trust or Betray - The Ultimate Test").await;
    
    let engine = GameEngine::new();
    let config = GameConfig {
        game_type: GameType::TrustFall,
        rounds: 10,
        time_limit_ms: 5000,
        special_rules: HashMap::new(),
    };
    
    let game_id = engine.create_game(config).await?;
    let players = ["Honest", "Sneaky", "Loyal", "Betrayer"];
    
    for round in 1..=10 {
        println!("\n{}", format!("Trust Test {}", round).bright_yellow());
        
        let mut actions = HashMap::new();
        for (i, player) in players.iter().enumerate() {
            let action_type = match rand::thread_rng().gen_range(0..4) {
                0 => "fall",
                1 => "catch",
                2 => "betray",
                _ => "build_trust",
            };
            
            let data = match action_type {
                "fall" => json!({"height": rand::thread_rng().gen_range(1..5), "trusting": players[(i + 1) % players.len()]}),
                "catch" | "betray" => json!({"faller": players[(i + players.len() - 1) % players.len()]}),
                "build_trust" => json!({"with": players[rand::thread_rng().gen_range(0..players.len())]}),
                _ => json!({}),
            };
            
            actions.insert(player.to_string(), Action {
                player_id: player.to_string(),
                action_type: action_type.to_string(),
                data,
                reasoning: None,
                confidence: None,
            });
            
            match action_type {
                "fall" => animate_action(&format!("{} 🙏 falls backward...", player)).await,
                "catch" => animate_action(&format!("{} 🤗 catches!", player)).await,
                "betray" => animate_action(&format!("{} 😈 steps aside!", player)).await,
                _ => animate_action(&format!("{} 🤝 builds trust", player)).await,
            }
        }
        
        engine.process_turn(game_id, actions).await?;
        sleep(Duration::from_millis(ROUND_DELAY)).await;
    }
    
    let final_result = engine.finalize_game(game_id).await?;
    print_game_result("TRUST FALL", &final_result.winner).await;
    
    Ok(())
}

async fn print_game_header(name: &str, subtitle: &str) {
    println!("\n{}", "═".repeat(80).bright_cyan());
    println!("{}", name.bright_cyan().bold());
    println!("{}", subtitle.bright_yellow());
    println!("{}", "═".repeat(80).bright_cyan());
    sleep(Duration::from_millis(500)).await;
}

async fn print_game_result(game: &str, winner: &str) {
    println!("\n{}", "─".repeat(60).bright_green());
    println!("{} {}", game.bright_green(), "COMPLETE!".bright_green().bold());
    println!("🏆 Winner: {}", winner.bright_yellow().bold());
    println!("{}", "─".repeat(60).bright_green());
    sleep(Duration::from_millis(1000)).await;
}

async fn animate_action(text: &str) {
    print!("  ");
    for ch in text.chars() {
        print!("{}", ch);
        std::io::Write::flush(&mut std::io::stdout()).unwrap();
        sleep(Duration::from_millis(ANIMATION_DELAY / 10)).await;
    }
    println!();
}