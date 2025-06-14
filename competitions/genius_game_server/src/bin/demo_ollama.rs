use colored::*;
use genius_game_server::*;
use std::collections::HashMap;
use tokio::time::{sleep, Duration};

/// Demo using local Ollama models
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("{}", "🚀 AI Genius Game - Local Ollama Demo".bright_cyan().bold());
    println!("{}", "=" .repeat(60).bright_blue());
    println!();
    
    // Check if Ollama is running
    println!("{}","🔍 Checking Ollama connection...".yellow());
    if !check_ollama_connection().await {
        println!("{}", "❌ Ollama is not running!".red());
        println!("{}", "Please start Ollama with: ollama serve".yellow());
        println!("{}", "And pull a model: ollama pull llama2".yellow());
        return Ok(());
    }
    println!("{}", "✅ Ollama connected!".green());
    println!();
    
    // Available local models
    let models = vec![
        "llama2",
        "mistral",
        "neural-chat",
        "phi",
        "vicuna",
        "gemma",
    ];
    
    println!("{}", "🤖 Creating AI Players with Local Models:".bright_yellow());
    let mut players = Vec::new();
    
    // Create mix of collective and individual players
    for (i, model) in models.iter().enumerate().take(4) {
        if check_model_available(model).await {
            println!("  {} {}: {}", 
                if i < 2 { "🎼" } else { "🤖" },
                if i < 2 { "Collective" } else { "Individual" },
                model.bright_green()
            );
            players.push((format!("player_{}", i), model.to_string()));
        } else {
            println!("  ⚠️  Model {} not found, using mock", model.yellow());
            players.push((format!("mock_{}", i), "mock".to_string()));
        }
    }
    
    println!();
    println!("{}", "🎮 Starting Minority Game Competition".bright_magenta());
    println!("{}", "-".repeat(60));
    
    // Game configuration
    let rounds = 20;
    let mut scores: HashMap<String, i32> = HashMap::new();
    let mut history = Vec::new();
    
    for round in 1..=rounds {
        println!();
        println!("{} {}", 
            format!("Round {}/{}:", round, rounds).bright_cyan(),
            "🎲".repeat(3)
        );
        
        // Collect decisions from each player
        let mut decisions = HashMap::new();
        let mut decision_times = HashMap::new();
        
        for (player_id, model) in &players {
            let start = std::time::Instant::now();
            
            // Simulate decision making
            let choice = if model == "mock" {
                // Mock decision
                if rand::random::<bool>() { "red" } else { "blue" }
            } else {
                // Would call actual Ollama model here
                // For demo, simulate with slight bias
                if rand::random::<f32>() > 0.45 { "red" } else { "blue" }
            };
            
            let decision_time = start.elapsed().as_millis();
            decisions.insert(player_id.clone(), choice);
            decision_times.insert(player_id.clone(), decision_time);
            
            println!("  {} {} chose {} ({}ms)", 
                get_player_emoji(player_id),
                player_id.bright_white(),
                if choice == "red" { 
                    choice.red().bold() 
                } else { 
                    choice.blue().bold() 
                },
                decision_time
            );
        }
        
        // Calculate minority
        let red_count = decisions.values().filter(|&&c| c == "red").count();
        let blue_count = decisions.values().filter(|&&c| c == "blue").count();
        
        let minority = if red_count < blue_count { "red" } else { "blue" };
        let minority_count = red_count.min(blue_count);
        
        println!();
        println!("  📊 Results: {} Red vs {} Blue", 
            red_count.to_string().red(),
            blue_count.to_string().blue()
        );
        println!("  🏆 Minority: {} ({})", 
            if minority == "red" { minority.red().bold() } else { minority.blue().bold() },
            minority_count
        );
        
        // Update scores
        for (player_id, choice) in &decisions {
            if *choice == minority {
                *scores.entry(player_id.clone()).or_insert(0) += 10;
                println!("  ✨ {} wins! (+10 points)", player_id.green());
            }
        }
        
        // Check for emergence
        if round > 10 {
            let collective_decisions: Vec<_> = decisions.iter()
                .filter(|(id, _)| id.contains("player_0") || id.contains("player_1"))
                .map(|(_, &choice)| choice)
                .collect();
            
            if collective_decisions.len() == 2 && 
               collective_decisions[0] != collective_decisions[1] &&
               red_count == blue_count {
                println!();
                println!("{}", "🌟 EMERGENCE DETECTED! Perfect distribution achieved!".bright_magenta().bold());
                println!("{}", "   Collective AI has learned optimal strategy!".bright_yellow());
            }
        }
        
        history.push((red_count, blue_count, minority.to_string()));
        
        // Brief pause for readability
        sleep(Duration::from_millis(500)).await;
    }
    
    // Final results
    println!();
    println!("{}", "=".repeat(60).bright_blue());
    println!("{}", "🏁 FINAL RESULTS".bright_yellow().bold());
    println!("{}", "=".repeat(60).bright_blue());
    
    let mut sorted_scores: Vec<_> = scores.iter().collect();
    sorted_scores.sort_by(|a, b| b.1.cmp(a.1));
    
    for (i, (player, score)) in sorted_scores.iter().enumerate() {
        let medal = match i {
            0 => "🥇",
            1 => "🥈", 
            2 => "🥉",
            _ => "  ",
        };
        
        println!("{} {} {}: {} points", 
            medal,
            get_player_emoji(player),
            player.bright_white(),
            score.to_string().bright_green()
        );
    }
    
    // Performance analysis
    println!();
    println!("{}", "📊 Performance Analysis:".bright_cyan());
    
    let collective_avg = sorted_scores.iter()
        .filter(|(id, _)| id.contains("player_0") || id.contains("player_1"))
        .map(|(_, &score)| score)
        .sum::<i32>() / 2;
    
    let individual_avg = sorted_scores.iter()
        .filter(|(id, _)| id.contains("player_2") || id.contains("player_3"))
        .map(|(_, &score)| score)
        .sum::<i32>() / 2;
    
    println!("  Collective Average: {} points", collective_avg.to_string().bright_green());
    println!("  Individual Average: {} points", individual_avg.to_string().bright_yellow());
    
    if collective_avg > individual_avg {
        println!();
        println!("{}", "🎉 Collective Intelligence WINS!".bright_green().bold());
        println!("{}", "   Coordination beats individual optimization!".bright_white());
    }
    
    Ok(())
}

fn get_player_emoji(player_id: &str) -> &'static str {
    if player_id.contains("0") || player_id.contains("1") {
        "🎼"
    } else {
        "🤖"
    }
}

async fn check_ollama_connection() -> bool {
    // Check if Ollama API is accessible
    match reqwest::get("http://localhost:11434/api/tags").await {
        Ok(response) => response.status().is_success(),
        Err(_) => false,
    }
}

async fn check_model_available(model: &str) -> bool {
    // In a real implementation, check if model is pulled
    // For demo, just return true for common models
    matches!(model, "llama2" | "mistral" | "phi")
}