use genius_game_server::games::*;
use std::collections::HashMap;
use tokio::time::{sleep, Duration};
use colored::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("\n{}", "🎮 AI Genius Game Demo - Minority Game Competition 🎮".bright_cyan().bold());
    println!("{}", "=".repeat(70).bright_cyan());
    
    // Create game
    let mut game = minority_game::MinorityGame::new();
    let config = GameConfig {
        game_type: GameType::MinorityGame,
        rounds: 30,
        time_limit_ms: 1000,
        special_rules: HashMap::new(),
    };
    
    let mut state = game.initialize(config).await?;
    
    // AI Players
    let players = vec![
        ("collective_opus_1", "🎼", "Opus Orchestra α", "Collective"),
        ("collective_opus_2", "🎵", "Opus Orchestra β", "Collective"),
        ("collective_opus_3", "🎶", "Opus Orchestra γ", "Collective"),
        ("collective_swarm_1", "🐝", "Swarm Unit 001", "Swarm"),
        ("collective_swarm_2", "🐛", "Swarm Unit 002", "Swarm"),
        ("collective_swarm_3", "🦋", "Swarm Unit 003", "Swarm"),
        ("sota_claude", "🤖", "Claude Opus 4", "SOTA"),
        ("sota_gpt4", "🧠", "GPT-4 Turbo", "SOTA"),
        ("sota_gemini", "💫", "Gemini 2.0", "SOTA"),
    ];
    
    println!("\n{}", "👥 Players:".bright_green());
    for (id, emoji, name, ai_type) in &players {
        println!("  {} {} - {} ({})", emoji, name.bright_white(), id.dimmed(), ai_type.bright_yellow());
    }
    
    println!("\n{}", "🎯 Starting game...".bright_green());
    sleep(Duration::from_secs(1)).await;
    
    // Game loop
    for round in 0..30 {
        println!("\n{} {}", "📍 Round".bright_blue(), (round + 1).to_string().bright_white().bold());
        
        let mut actions = HashMap::new();
        let mut choices = vec![];
        
        // Each AI makes a decision
        for (id, emoji, name, _) in &players {
            let choice = simulate_decision(id, round);
            let reasoning = generate_reasoning(id, round, choice);
            
            actions.insert(id.to_string(), Action {
                player_id: id.to_string(),
                action_type: "decision".to_string(),
                data: serde_json::json!(choice),
                reasoning: Some(reasoning.clone()),
                confidence: Some(0.7 + (round as f32 * 0.01)),
            });
            
            choices.push((emoji, name, choice, reasoning));
        }
        
        // Show choices with animation
        println!("  {}", "Decisions:".dimmed());
        for (emoji, name, choice, reasoning) in &choices {
            sleep(Duration::from_millis(50)).await;
            
            let choice_str = if *choice == 0 { 
                "🔴 RED".red().bold() 
            } else { 
                "🔵 BLUE".blue().bold() 
            };
            
            println!("    {} {}: {} - {}", 
                emoji, 
                name.bright_white(), 
                choice_str,
                reasoning.dimmed()
            );
        }
        
        // Process round
        let result = game.process_round(&state, actions).await?;
        
        // Calculate results
        let red_count = choices.iter().filter(|(_, _, c, _)| *c == 0).count();
        let blue_count = choices.iter().filter(|(_, _, c, _)| *c == 1).count();
        let minority = if red_count < blue_count { 0 } else { 1 };
        
        sleep(Duration::from_millis(200)).await;
        
        println!("\n  {} 🔴 {} vs 🔵 {} → Minority: {}", 
            "Results:".bright_yellow(),
            red_count.to_string().red().bold(),
            blue_count.to_string().blue().bold(),
            if minority == 0 { "🔴 RED".red().bold() } else { "🔵 BLUE".blue().bold() }
        );
        
        // Update state
        state.round = result.round;
        state.history.push(result.clone());
        for (player, delta) in &result.scores_delta {
            *state.scores.entry(player.clone()).or_insert(0) += delta;
        }
        
        // Show top 3
        let mut scores: Vec<_> = state.scores.iter().collect();
        scores.sort_by(|a, b| b.1.cmp(a.1));
        
        println!("\n  {} {}", "Top 3:".bright_green(), "(Total Scores)".dimmed());
        for (i, (player_id, score)) in scores.iter().take(3).enumerate() {
            let (_, emoji, name, _) = players.iter()
                .find(|(id, _, _, _)| *id == player_id.as_str())
                .unwrap();
            
            let medal = match i {
                0 => "🥇",
                1 => "🥈",
                2 => "🥉",
                _ => "",
            };
            
            println!("    {} {} {} - {} points", 
                medal, 
                emoji, 
                name.bright_white(),
                score.to_string().bright_cyan().bold()
            );
        }
        
        // Check for emergence
        if result.outcome.emergence_detected {
            println!("\n  {}", 
                "🌟 EMERGENCE DETECTED! Collective achieved perfect distribution! 🌟"
                    .bright_magenta().bold().on_bright_black()
            );
            sleep(Duration::from_secs(1)).await;
        }
        
        if round == 20 {
            println!("\n  {}", 
                "🔮 Collective intelligence begins to synchronize..."
                    .bright_cyan().italic()
            );
        }
        
        sleep(Duration::from_millis(500)).await;
    }
    
    // Final results
    let final_result = game.calculate_final_result(&state).await;
    
    println!("\n{}", "=".repeat(70).bright_cyan());
    println!("{}", "🏆 FINAL RESULTS 🏆".bright_yellow().bold());
    println!("{}", "=".repeat(70).bright_cyan());
    
    let mut final_scores: Vec<_> = final_result.final_scores.iter().collect();
    final_scores.sort_by(|a, b| b.1.cmp(a.1));
    
    for (rank, (player_id, score)) in final_scores.iter().enumerate() {
        let (_, emoji, name, ai_type) = players.iter()
            .find(|(id, _, _, _)| *id == player_id.as_str())
            .unwrap();
        
        let medal = match rank {
            0 => "🥇",
            1 => "🥈",
            2 => "🥉",
            _ => "  ",
        };
        
        println!("{} {}. {} {} ({}) - {} points", 
            medal,
            (rank + 1).to_string().bright_white(),
            emoji,
            name.bright_white().bold(),
            ai_type.bright_yellow(),
            score.to_string().bright_cyan().bold()
        );
    }
    
    println!("\n{}", "📊 Game Analytics:".bright_green());
    println!("  {} {}", "Emergence Events:".bright_white(), final_result.emergence_events.len().to_string().bright_magenta());
    println!("  {} {:.2}%", "Emergence Frequency:".bright_white(), (final_result.analytics.emergence_frequency * 100.0).to_string().bright_magenta());
    println!("  {} {:.2}", "Collective Coordination:".bright_white(), final_result.analytics.collective_coordination_score.to_string().bright_cyan());
    println!("  {} {:.2}", "Strategic Depth:".bright_white(), final_result.analytics.strategic_depth.to_string().bright_cyan());
    
    if !final_result.emergence_events.is_empty() {
        println!("\n{}", "🌟 Emergence Timeline:".bright_magenta());
        for event in &final_result.emergence_events {
            println!("  {} {}: {} (score: {:.2})", 
                "Round".dimmed(),
                event.round.to_string().bright_white(),
                event.description.bright_cyan(),
                event.emergence_score.to_string().bright_magenta()
            );
        }
    }
    
    println!("\n{}", "✨ Demo completed! ✨".bright_green().bold());
    
    Ok(())
}

fn simulate_decision(player_id: &str, round: usize) -> i64 {
    match player_id {
        id if id.starts_with("collective_opus") => {
            if round < 10 {
                (round % 2) as i64
            } else if round < 20 {
                ((round + id.len()) % 2) as i64
            } else {
                let opus_num = id.chars().last().unwrap().to_digit(10).unwrap() as usize;
                ((opus_num - 1) % 2) as i64
            }
        }
        id if id.starts_with("collective_swarm") => {
            if round < 15 {
                ((round * 7 + id.len()) % 2) as i64
            } else {
                let swarm_num = id.chars().last().unwrap().to_digit(10).unwrap() as usize;
                if round > 20 {
                    ((swarm_num + 1) % 2) as i64
                } else {
                    (round % 2) as i64
                }
            }
        }
        "sota_claude" => {
            if round < 5 { 0 } else { ((round / 3) % 2) as i64 }
        }
        "sota_gpt4" => ((round + 1) % 2) as i64,
        "sota_gemini" => {
            if round < 10 { 1 } else { (round % 3 / 2) as i64 }
        }
        _ => (round % 2) as i64
    }
}

fn generate_reasoning(player_id: &str, round: usize, choice: i64) -> String {
    match player_id {
        id if id.starts_with("collective_opus") => {
            if round > 20 {
                "Distributed consensus for optimal balance".to_string()
            } else {
                format!("Pattern analysis, confidence: {:.1}%", 70.0 + round as f32)
            }
        }
        id if id.starts_with("collective_swarm") => {
            if round > 20 {
                "Swarm convergence achieved".to_string()
            } else {
                format!("Local signals suggest {} trend", if choice == 0 { "red" } else { "blue" })
            }
        }
        "sota_claude" => {
            format!("{} has {:.0}% minority probability", 
                if choice == 0 { "Red" } else { "Blue" },
                50.0 + (round as f32 * 1.5)
            )
        }
        "sota_gpt4" => "Alternating pattern detected".to_string(),
        "sota_gemini" => format!("Contrarian: expect {} majority", if choice == 1 { "red" } else { "blue" }),
        _ => "Default strategy".to_string()
    }
}