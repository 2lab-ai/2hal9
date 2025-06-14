use genius_game_server::{*, games::*, collective::*, sota::*};
use std::collections::HashMap;
use tokio::time::{sleep, Duration};
use serde_json::json;

#[tokio::main]
async fn main() {
    println!("🎮 AI Genius Game Demo - Minority Game Competition");
    println!("=" .repeat(60));
    
    // Create game engine
    let engine = GameEngine::new();
    
    // Initialize game
    let config = GameConfig {
        game_type: GameType::MinorityGame,
        rounds: 30,
        time_limit_ms: 1000,
        special_rules: HashMap::new(),
    };
    
    let game_id = engine.create_game(config).await.unwrap();
    println!("✨ Game initialized: {:?}", game_id);
    
    // Create AI players
    let players = vec![
        // Collective Intelligence - Opus Orchestra
        ("collective_opus_1", "🎼", "Opus Orchestra Alpha"),
        ("collective_opus_2", "🎵", "Opus Orchestra Beta"),
        ("collective_opus_3", "🎶", "Opus Orchestra Gamma"),
        
        // Collective Intelligence - Swarm
        ("collective_swarm_1", "🐝", "Swarm Unit 001"),
        ("collective_swarm_2", "🐛", "Swarm Unit 002"),
        ("collective_swarm_3", "🦋", "Swarm Unit 003"),
        
        // SOTA Models
        ("sota_claude", "🤖", "Claude Opus 4"),
        ("sota_gpt4", "🧠", "GPT-4 Turbo"),
        ("sota_gemini", "💫", "Gemini 2.0"),
    ];
    
    println!("\n👥 Players joining:");
    for (id, emoji, name) in &players {
        println!("  {} {} - {}", emoji, name, id);
    }
    
    // Run game rounds
    println!("\n🎯 Starting game rounds...\n");
    
    for round in 0..30 {
        println!("📍 Round {}", round + 1);
        
        let mut actions = HashMap::new();
        let mut round_choices = vec![];
        
        // Each player makes a decision
        for (id, emoji, name) in &players {
            // Simulate AI reasoning
            let choice = simulate_ai_decision(*id, round).await;
            let reasoning = generate_reasoning(*id, round, choice);
            
            actions.insert(id.to_string(), Action {
                player_id: id.to_string(),
                action_type: "decision".to_string(),
                data: json!(choice),
                reasoning: Some(reasoning.clone()),
                confidence: Some(0.7 + (round as f32 * 0.01)),
            });
            
            round_choices.push((emoji, name, choice, reasoning));
        }
        
        // Display choices
        println!("  Choices:");
        for (emoji, name, choice, reasoning) in &round_choices {
            println!("    {} {}: {} - {}", 
                emoji, 
                name, 
                if *choice == 0 { "🔴 RED" } else { "🔵 BLUE" },
                reasoning
            );
        }
        
        // Process round
        let result = engine.process_turn(game_id, actions).await.unwrap();
        
        // Show results
        let red_count = round_choices.iter().filter(|(_, _, c, _)| *c == 0).count();
        let blue_count = round_choices.iter().filter(|(_, _, c, _)| *c == 1).count();
        let minority = if red_count < blue_count { 0 } else { 1 };
        
        println!("\n  Results: 🔴 {} vs 🔵 {} → Minority: {}", 
            red_count, 
            blue_count,
            if minority == 0 { "🔴 RED" } else { "🔵 BLUE" }
        );
        
        // Show top scores
        if let Some(state) = engine.get_game_state(game_id).await {
            let mut scores: Vec<_> = state.scores.iter().collect();
            scores.sort_by(|a, b| b.1.cmp(a.1));
            
            println!("  Top 3 Scores:");
            for (i, (player_id, score)) in scores.iter().take(3).enumerate() {
                let (_, emoji, name) = players.iter()
                    .find(|(id, _, _)| id == player_id.as_str())
                    .unwrap();
                println!("    {}. {} {} - {} points", i + 1, emoji, name, score);
            }
        }
        
        // Check for emergence
        if result.outcome.emergence_detected {
            println!("\n  🌟 EMERGENCE DETECTED! Collective intelligence achieved perfect distribution!");
        }
        
        println!();
        
        // Small delay for readability
        sleep(Duration::from_millis(100)).await;
        
        // After round 20, create emergence conditions
        if round == 20 {
            println!("🔮 Collective intelligence begins to synchronize...\n");
        }
    }
    
    // Final results
    let final_result = engine.finalize_game(game_id).await.unwrap();
    
    println!("\n🏆 FINAL RESULTS");
    println!("=" .repeat(60));
    
    // Sort final scores
    let mut final_scores: Vec<_> = final_result.final_scores.iter().collect();
    final_scores.sort_by(|a, b| b.1.cmp(a.1));
    
    for (rank, (player_id, score)) in final_scores.iter().enumerate() {
        let (_, emoji, name) = players.iter()
            .find(|(id, _, _)| id == player_id.as_str())
            .unwrap();
        
        let medal = match rank {
            0 => "🥇",
            1 => "🥈", 
            2 => "🥉",
            _ => "  ",
        };
        
        println!("{} {}. {} {} - {} points", 
            medal, 
            rank + 1, 
            emoji, 
            name, 
            score
        );
    }
    
    println!("\n📊 Game Analytics:");
    println!("  Emergence Events: {}", final_result.emergence_events.len());
    println!("  Emergence Frequency: {:.2}%", final_result.analytics.emergence_frequency * 100.0);
    println!("  Collective Coordination: {:.2}", final_result.analytics.collective_coordination_score);
    println!("  Strategic Depth: {:.2}", final_result.analytics.strategic_depth);
    
    if !final_result.emergence_events.is_empty() {
        println!("\n🌟 Emergence Timeline:");
        for event in &final_result.emergence_events {
            println!("  Round {}: {} (score: {:.2})", 
                event.round, 
                event.description, 
                event.emergence_score
            );
        }
    }
}

async fn simulate_ai_decision(player_id: &str, round: usize) -> i64 {
    // Simulate different AI strategies
    match player_id {
        // Opus Orchestra - sophisticated pattern analysis
        id if id.starts_with("collective_opus") => {
            if round < 10 {
                (round % 2) as i64
            } else if round < 20 {
                ((round + id.len()) % 2) as i64
            } else {
                // Emergence: perfect distribution
                let opus_num = id.chars().last().unwrap().to_digit(10).unwrap() as usize;
                ((opus_num - 1) % 2) as i64
            }
        }
        
        // Swarm - emergent consensus
        id if id.starts_with("collective_swarm") => {
            if round < 15 {
                // Random-ish
                ((round * 7 + id.len()) % 2) as i64
            } else {
                // Converge to pattern
                let swarm_num = id.chars().last().unwrap().to_digit(10).unwrap() as usize;
                if round > 20 {
                    ((swarm_num + 1) % 2) as i64
                } else {
                    (round % 2) as i64
                }
            }
        }
        
        // SOTA models - individual strategies
        "sota_claude" => {
            // Analytical approach
            if round < 5 {
                0
            } else {
                ((round / 3) % 2) as i64
            }
        }
        
        "sota_gpt4" => {
            // Pattern matching
            ((round + 1) % 2) as i64
        }
        
        "sota_gemini" => {
            // Contrarian strategy
            if round < 10 {
                1
            } else {
                (round % 3 / 2) as i64
            }
        }
        
        _ => (round % 2) as i64
    }
}

fn generate_reasoning(player_id: &str, round: usize, choice: i64) -> String {
    match player_id {
        id if id.starts_with("collective_opus") => {
            if round > 20 {
                format!("Achieving distributed consensus for optimal minority balance")
            } else {
                format!("Analyzing historical patterns, confidence: {:.1}%", 70.0 + round as f32)
            }
        }
        
        id if id.starts_with("collective_swarm") => {
            if round > 20 {
                format!("Swarm convergence achieved, following emergent pattern")
            } else {
                format!("Local communication suggests {} trend", if choice == 0 { "red" } else { "blue" })
            }
        }
        
        "sota_claude" => {
            format!("Multi-level reasoning indicates {} has {:.0}% minority probability", 
                if choice == 0 { "red" } else { "blue" },
                50.0 + (round as f32 * 1.5)
            )
        }
        
        "sota_gpt4" => {
            format!("Pattern analysis: alternating strategy with {} bias", 
                if choice == 0 { "red" } else { "blue" }
            )
        }
        
        "sota_gemini" => {
            format!("Contrarian play: expecting majority on {}", 
                if choice == 1 { "red" } else { "blue" }
            )
        }
        
        _ => "Default strategy".to_string()
    }
}