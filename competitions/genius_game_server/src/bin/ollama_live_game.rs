use colored::*;
use reqwest;
use serde::{Deserialize, Serialize};
use std::time::Instant;

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
struct GameDecision {
    choice: String,
    reasoning: String,
}

async fn call_ollama(prompt: String, model: &str) -> anyhow::Result<GameDecision> {
    let client = reqwest::Client::new();
    let request = OllamaRequest {
        model: model.to_string(),
        prompt,
        stream: false,
        options: OllamaOptions {
            temperature: 0.7,
            num_predict: 100,
        },
    };
    
    let response = client
        .post("http://localhost:11434/api/generate")
        .json(&request)
        .send()
        .await?;
    
    let ollama_resp: OllamaResponse = response.json().await?;
    
    // Parse JSON from response
    match serde_json::from_str(&ollama_resp.response) {
        Ok(decision) => Ok(decision),
        Err(_) => {
            // Fallback
            Ok(GameDecision {
                choice: "cooperate".to_string(),
                reasoning: "Failed to parse response".to_string(),
            })
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("{}", "🎮 Live Ollama Game Demo - Prisoner's Dilemma".bright_cyan().bold());
    println!("{}", "=".repeat(60).bright_blue());
    println!();
    
    // Check Ollama
    println!("{}", "🔍 Checking Ollama...".yellow());
    let check = reqwest::get("http://localhost:11434/api/tags").await;
    if check.is_err() {
        println!("{}", "❌ Ollama is not running!".red());
        println!("{}", "Start with: ollama serve".yellow());
        return Ok(());
    }
    
    let models: serde_json::Value = check.unwrap().json().await?;
    let available_models: Vec<String> = models["models"]
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .filter_map(|m| m["name"].as_str().map(|s| s.to_string()))
        .collect();
    
    println!("{}", "✅ Ollama is running!".green());
    println!("Available models: {:?}", available_models);
    
    // Use llama3 if available, otherwise first model
    let model = if available_models.contains(&"llama3:latest".to_string()) {
        "llama3:latest"
    } else if !available_models.is_empty() {
        &available_models[0]
    } else {
        println!("{}", "❌ No models found! Pull one with: ollama pull llama3".red());
        return Ok(());
    };
    
    println!("Using model: {}", model.bright_green());
    println!();
    
    // Game setup
    println!("{}", "🎯 Starting Prisoner's Dilemma".bright_magenta());
    println!("{}", "-".repeat(60));
    
    let rounds = 5;
    let mut history = Vec::new();
    let mut ollama_score = 0;
    let mut opponent_score = 0;
    
    for round in 1..=rounds {
        println!();
        println!("{}", format!("Round {}/{}:", round, rounds).bright_cyan());
        
        // Create prompt with history
        let history_str = history.iter()
            .map(|(o, op): &(String, String)| format!("You: {}, Opponent: {}", o, op))
            .collect::<Vec<_>>()
            .join("; ");
        
        let prompt = format!(
            r#"You are playing Prisoner's Dilemma. 
Your choices are: cooperate or defect.
{}
What do you choose? Respond ONLY with JSON: {{"choice": "cooperate" or "defect", "reasoning": "brief explanation"}}"#,
            if history.is_empty() { "This is the first round.".to_string() } else { format!("History: {}", history_str) }
        );
        
        // Get Ollama's decision
        let start = Instant::now();
        println!("  {} Ollama is thinking...", "🤔".repeat(3));
        
        let decision = call_ollama(prompt, model).await?;
        let thinking_time = start.elapsed().as_millis();
        
        println!("  {} Ollama chose: {} ({}ms)", 
            "🤖",
            decision.choice.bright_yellow(),
            thinking_time
        );
        println!("  {} Reasoning: {}", "💭", decision.reasoning.italic());
        
        // Simple opponent strategy (tit-for-tat)
        let opponent_choice = if history.is_empty() {
            "cooperate"
        } else {
            &history.last().unwrap().0 // Copy Ollama's last move
        };
        
        println!("  {} Opponent chose: {}", "👤", opponent_choice.bright_cyan());
        
        // Calculate scores
        let (ollama_points, opponent_points) = match (decision.choice.as_str(), opponent_choice) {
            ("cooperate", "cooperate") => (3, 3),
            ("cooperate", "defect") => (0, 5),
            ("defect", "cooperate") => (5, 0),
            ("defect", "defect") => (1, 1),
            _ => (0, 0),
        };
        
        ollama_score += ollama_points;
        opponent_score += opponent_points;
        
        println!("  {} Round score: Ollama +{}, Opponent +{}", 
            "📊",
            ollama_points,
            opponent_points
        );
        
        // Add to history
        history.push((decision.choice, opponent_choice.to_string()));
    }
    
    // Final results
    println!();
    println!("{}", "=".repeat(60).bright_blue());
    println!("{}", "🏁 FINAL RESULTS".bright_yellow().bold());
    println!("  {} Ollama: {} points", "🤖", ollama_score.to_string().bright_green());
    println!("  {} Opponent: {} points", "👤", opponent_score.to_string().bright_cyan());
    
    if ollama_score > opponent_score {
        println!();
        println!("{}", "🎉 Ollama WINS!".bright_green().bold());
    } else if opponent_score > ollama_score {
        println!();
        println!("{}", "😔 Opponent wins!".bright_red());
    } else {
        println!();
        println!("{}", "🤝 It's a tie!".bright_yellow());
    }
    
    Ok(())
}