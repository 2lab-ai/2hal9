//! Signal command implementation

use anyhow::Result;
use colored::Colorize;
use serde_json::json;

pub async fn execute(from: String, to: String, content: String, server: String) -> Result<()> {
    println!("{} signal:", "Sending".green());
    println!("  {}: {}", "From".bold(), from.cyan());
    println!("  {}: {}", "To".bold(), to.cyan());
    println!("  {}: {}", "Content".bold(), content);
    
    // Create HTTP client
    let client = reqwest::Client::new();
    
    // Build request payload
    let payload = json!({
        "content": content,
        "layer": "L4",  // Default to L4 for user input
        "neuron_id": if to.is_empty() { None } else { Some(to.clone()) }
    });
    
    // Send request
    let url = format!("http://{}/api/v1/signal", server);
    println!("\n{} to {}", "Sending".yellow(), url.cyan());
    
    let response = client
        .post(&url)
        .json(&payload)
        .send()
        .await?;
    
    // Check response
    if response.status().is_success() {
        let result: serde_json::Value = response.json().await?;
        
        if let Some(signal_id) = result.get("data").and_then(|d| d.get("signal_id")) {
            println!("\n{} Signal sent successfully!", "✓".green());
            println!("{}: {}", "Signal ID".bold(), signal_id.as_str().unwrap_or("unknown").yellow());
        } else {
            println!("\n{} Signal sent!", "✓".green());
        }
        
        // Show full response in debug mode
        if std::env::var("DEBUG").is_ok() {
            println!("\n{}", "Response:".bold());
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
    } else {
        let status = response.status();
        let error_text = response.text().await?;
        println!("\n{} Failed to send signal!", "✗".red());
        println!("{}: {}", "Status".bold(), status.to_string().red());
        println!("{}: {}", "Error".bold(), error_text);
    }
    
    Ok(())
}