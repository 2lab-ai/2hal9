//! Status command implementation

use anyhow::Result;
use colored::Colorize;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct ServerStatus {
    running: bool,
    uptime_seconds: u64,
    neurons: Vec<NeuronStatus>,
    metrics: MetricsSummary,
}

#[derive(Debug, Serialize, Deserialize)]
struct NeuronStatus {
    id: String,
    layer: String,
    state: String,
    health: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct MetricsSummary {
    signals_sent: u64,
    signals_processed: u64,
    signals_failed: u64,
    average_latency_ms: f64,
}

#[derive(Debug, Serialize, Deserialize)]
struct ApiResponse<T> {
    success: bool,
    data: Option<T>,
    error: Option<String>,
}

pub async fn execute(server: String, format: String) -> Result<()> {
    // Create HTTP client
    let client = reqwest::Client::new();
    
    // Query server status
    let url = format!("http://{}/api/v1/status", server);
    
    match client.get(&url).send().await {
        Ok(response) => {
            if response.status().is_success() {
                let api_response: ApiResponse<ServerStatus> = response.json().await?;
                
                if let Some(status) = api_response.data {
                    if format == "json" {
                        // JSON format
                        println!("{}", serde_json::to_string_pretty(&status)?);
                    } else {
                        // Text format
                        print_status_text(&status);
                    }
                } else {
                    println!("{} Server returned no data", "✗".red());
                }
            } else {
                println!("{} Failed to get status: {}", "✗".red(), response.status());
            }
        }
        Err(e) => {
            println!("{} Failed to connect to server: {}", "✗".red(), e);
            println!("Is the server running at {}?", server.cyan());
        }
    }
    
    Ok(())
}

fn print_status_text(status: &ServerStatus) {
    println!("\n{}", "HAL9 Server Status".bold().underline());
    println!("{}: {}", "Status".bold(), if status.running { "Running".green() } else { "Stopped".red() });
    println!("{}: {}", "Uptime".bold(), format_duration(status.uptime_seconds));
    
    if !status.neurons.is_empty() {
        println!("\n{}", "Neurons".bold().underline());
        println!("{:<20} {:<10} {:<15} {:<10}", "ID", "Layer", "State", "Health");
        println!("{}", "-".repeat(55));
        
        for neuron in &status.neurons {
            let state_colored = match neuron.state.as_str() {
                "Running" => neuron.state.green(),
                "Stopped" => neuron.state.red(),
                _ => neuron.state.yellow(),
            };
            
            let health_colored = match neuron.health.as_str() {
                "Healthy" => neuron.health.green(),
                "Unhealthy" => neuron.health.red(),
                _ => neuron.health.yellow(),
            };
            
            println!("{:<20} {:<10} {:<15} {:<10}", 
                neuron.id.cyan(), 
                neuron.layer, 
                state_colored, 
                health_colored
            );
        }
    }
    
    println!("\n{}", "Performance Metrics".bold().underline());
    println!("{}: {}", "Signals sent".bold(), status.metrics.signals_sent);
    println!("{}: {}", "Signals processed".bold(), status.metrics.signals_processed);
    println!("{}: {}", "Signals failed".bold(), status.metrics.signals_failed.to_string().red());
    println!("{}: {:.2}ms", "Average latency".bold(), status.metrics.average_latency_ms);
}

fn format_duration(seconds: u64) -> String {
    if seconds < 60 {
        format!("{} seconds", seconds)
    } else if seconds < 3600 {
        format!("{} minutes", seconds / 60)
    } else if seconds < 86400 {
        format!("{:.1} hours", seconds as f64 / 3600.0)
    } else {
        format!("{:.1} days", seconds as f64 / 86400.0)
    }
}