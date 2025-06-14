use colored::*;
use std::collections::HashMap;
use tokio::time::{sleep, Duration};

/// Death Game Demo - AI battles to the death
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("{}", "⚔️  AI DEATH GAME CHAMPIONSHIP ⚔️".bright_red().bold());
    println!("{}", "=".repeat(60).red());
    println!();
    println!("{}", "Only one AI will survive...".bright_white());
    sleep(Duration::from_secs(2)).await;
    
    // Initialize players
    let players = vec![
        ("🤖 AI Gladiator α", "aggressive", 100),
        ("🎭 AI Gladiator β", "defensive", 100),
    ];
    
    println!();
    println!("{}", "🏯 ROUND 1: MINI GO BATTLE".bright_yellow());
    println!("{}", "-".repeat(60).yellow());
    
    // Simulate Go game
    simulate_go_battle(&players).await?;
    
    println!();
    println!("{}", "🃏 ROUND 2: MINI HOLD'EM SHOWDOWN".bright_cyan());
    println!("{}", "-".repeat(60).cyan());
    
    // Simulate Hold'em
    simulate_holdem_battle(&players).await?;
    
    println!();
    println!("{}", "🦑 FINAL ROUND: SQUID GAME".bright_magenta());
    println!("{}", "-".repeat(60).magenta());
    
    // Simulate Squid Game
    simulate_squid_game(&players).await?;
    
    // Final results
    println!();
    println!("{}", "=".repeat(60).bright_red());
    println!("{}", "🏆 FINAL CHAMPION: AI GLADIATOR α 🏆".bright_yellow().bold());
    println!("{}", "=".repeat(60).bright_red());
    
    Ok(())
}

async fn simulate_go_battle(players: &[(&str, &str, i32)]) -> anyhow::Result<()> {
    println!("📍 {} vs {}", players[0].0, players[1].0);
    println!();
    
    // Display mini Go board
    let board = [
        "· · · · · · · · ·",
        "· · · ○ · · · · ·",
        "· · ● · · · ○ · ·",
        "· · · · ● · · · ·",
        "· · · ● ○ ● · · ·",
        "· · · · ● · · · ·",
        "· · ○ · · · ● · ·",
        "· · · · · ○ · · ·",
        "· · · · · · · · ·",
    ];
    
    for (i, line) in board.iter().enumerate() {
        if i == 0 {
            println!("   {}", line);
        } else {
            print!("   ");
            for ch in line.chars() {
                match ch {
                    '●' => print!("{}", "●".bright_black()),
                    '○' => print!("{}", "○".bright_white()),
                    _ => print!("{}", ch),
                }
            }
            println!();
        }
        sleep(Duration::from_millis(100)).await;
    }
    
    println!();
    println!("⚔️  {} captures a group!", players[0].0.bright_red());
    println!("📊 Territory: {} leads 55-45", players[0].0);
    sleep(Duration::from_secs(1)).await;
    
    Ok(())
}

async fn simulate_holdem_battle(players: &[(&str, &str, i32)]) -> anyhow::Result<()> {
    let mut chips = HashMap::new();
    chips.insert(players[0].0, 1000);
    chips.insert(players[1].0, 1000);
    
    println!("💰 Starting chips: $1000 each");
    println!();
    
    // Pre-flop
    println!("{}", "🃏 Dealing hole cards...".bright_white());
    sleep(Duration::from_millis(500)).await;
    println!("   {} holds: A♠ K♠", players[0].0);
    println!("   {} holds: Q♥ Q♦", players[1].0);
    println!();
    
    // Betting
    println!("{} bets: ${}", players[0].0, "200".bright_yellow());
    println!("{} raises: ${}", players[1].0, "600".bright_red());
    println!("{} goes: {}", players[0].0, "ALL IN!".bright_red().bold());
    println!("{} calls: {}", players[1].0, "ALL IN!".bright_red().bold());
    sleep(Duration::from_secs(1)).await;
    
    println!();
    println!("{}", "🎰 Community cards:".bright_cyan());
    
    // Flop
    print!("   Flop:  ");
    let flop = ["K♥", "7♣", "2♦"];
    for card in &flop {
        print!("{} ", card.bright_white());
        sleep(Duration::from_millis(300)).await;
    }
    println!();
    
    // Turn
    print!("   Turn:  ");
    print!("{} ", "A♣".bright_white());
    sleep(Duration::from_millis(500)).await;
    println!();
    
    // River
    print!("   River: ");
    print!("{} ", "9♠".bright_white());
    sleep(Duration::from_millis(500)).await;
    println!();
    
    println!();
    println!("🏆 {} wins with Two Pair! 💰 $2000", players[0].0.bright_green());
    println!("💀 {} ELIMINATED!", players[1].0.bright_red());
    
    Ok(())
}

async fn simulate_squid_game(players: &[(&str, &str, i32)]) -> anyhow::Result<()> {
    println!("🚥 Red Light, Green Light begins...");
    println!();
    
    let mut positions = HashMap::new();
    positions.insert(players[0].0, 0.0);
    positions.insert(players[1].0, 0.0);
    
    for round in 1..=5 {
        let is_green = round % 2 == 1;
        
        if is_green {
            println!("{}", "🟢 GREEN LIGHT!".bright_green().bold());
            println!("   {} moves forward quickly!", players[0].0);
            positions.insert(players[0].0, positions[players[0].0] + 20.0);
            
            if round > 2 {
                println!("   {} moves cautiously...", players[1].0);
                positions.insert(players[1].0, positions[players[1].0] + 10.0);
            }
        } else {
            println!("{}", "🔴 RED LIGHT!".bright_red().bold());
            
            if round == 4 {
                println!("   {} tries to move... 💥 ELIMINATED!", players[1].0.bright_red());
                break;
            } else {
                println!("   All players freeze!");
            }
        }
        
        // Show progress
        println!();
        for (player, &pos) in &positions {
            let progress = "=".repeat((pos / 5.0) as usize);
            let remaining_spaces = ((20.0_f32 - pos / 5.0).max(0.0)) as usize;
            let remaining = " ".repeat(remaining_spaces);
            println!("   {} [{}{}] {:.0}m", 
                player, 
                progress.bright_green(), 
                remaining,
                pos
            );
        }
        println!();
        
        sleep(Duration::from_secs(1)).await;
    }
    
    println!("🏁 {} reaches the finish line!", players[0].0.bright_yellow());
    
    Ok(())
}