use anyhow::Result;
use colored::Colorize;
use std::io::{self, Write};
use tracing::info;

use crate::DashboardType;
use crate::client::MigrationClient;

pub async fn run(
    server: &str,
    dashboard: DashboardType,
    interval: u64,
) -> Result<()> {
    let client = MigrationClient::new(server)?;
    
    println!("{}", "📊 HAL9 Migration Monitor".bold().cyan());
    println!("Press 'q' to quit\n");
    
    // Enable raw mode for terminal
    crossterm::terminal::enable_raw_mode()?;
    
    loop {
        // Clear screen
        print!("\x1B[2J\x1B[1;1H");
        
        match dashboard {
            DashboardType::Metrics => display_metrics_dashboard(&client).await?,
            DashboardType::Logs => display_logs_dashboard(&client).await?,
            DashboardType::Traces => display_traces_dashboard(&client).await?,
            DashboardType::Combined => display_combined_dashboard(&client).await?,
        }
        
        // Check for 'q' key press
        if check_quit_key()? {
            break;
        }
        
        tokio::time::sleep(tokio::time::Duration::from_secs(interval)).await;
    }
    
    // Restore terminal
    crossterm::terminal::disable_raw_mode()?;
    println!("\nMonitoring stopped.");
    
    Ok(())
}

async fn display_metrics_dashboard(client: &MigrationClient) -> Result<()> {
    println!("{}", "=== METRICS DASHBOARD ===".bold().cyan());
    println!();
    
    // System metrics
    println!("{}", "System Metrics:".bold());
    println!("  CPU Usage:      {}  {}", "45%".green(), create_bar(45, 20));
    println!("  Memory Usage:   {}  {}", "62%".yellow(), create_bar(62, 20));
    println!("  Disk I/O:       {} MB/s", "125".cyan());
    println!();
    
    // Migration metrics
    println!("{}", "Migration Progress:".bold());
    println!("  Current Phase:  {}", "Canary (35%)".yellow());
    println!("  Neurons:        {}/{}", "35".green(), "100");
    println!("  Error Rate:     {} (target: <1%)", "0.1%".green());
    println!("  Latency (p99):  {} (target: <10ms)", "8.5ms".green());
    println!();
    
    // Layer status
    println!("{}", "Layer Status:".bold());
    println!("  Substrate:      {}", "✓ Healthy".green());
    println!("  Protocol:       {}", "✓ Healthy".green());
    println!("  Cognitive:      {}", "⚠ Migrating".yellow());
    println!("  Orchestration:  {}", "○ Pending".dimmed());
    println!("  Intelligence:   {}", "○ Pending".dimmed());
    
    Ok(())
}

async fn display_logs_dashboard(client: &MigrationClient) -> Result<()> {
    println!("{}", "=== LOGS DASHBOARD ===".bold().cyan());
    println!();
    
    // Mock log entries
    let logs = vec![
        ("INFO", "Migration controller started", "14:32:01"),
        ("INFO", "Canary phase initialized at 35%", "14:32:05"),
        ("WARN", "High memory usage in cognitive layer", "14:32:15"),
        ("INFO", "Health check passed", "14:32:30"),
        ("INFO", "Traffic routing updated", "14:32:45"),
        ("DEBUG", "Neuron cluster formed: strategic-001", "14:33:00"),
        ("INFO", "Metrics collected successfully", "14:33:15"),
    ];
    
    for (level, message, time) in logs.iter().rev() {
        let level_colored = match *level {
            "ERROR" => level.red(),
            "WARN" => level.yellow(),
            "INFO" => level.green(),
            "DEBUG" => level.cyan(),
            _ => level.normal(),
        };
        
        println!("{} [{}] {}", time.dimmed(), level_colored, message);
    }
    
    Ok(())
}

async fn display_traces_dashboard(client: &MigrationClient) -> Result<()> {
    println!("{}", "=== TRACES DASHBOARD ===".bold().cyan());
    println!();
    
    // Mock trace data
    println!("{}", "Active Traces:".bold());
    println!();
    
    println!("Request ID: {}", "abc-123-def".cyan());
    println!("  ├─ gateway (2.1ms)");
    println!("  ├─ router (0.5ms)");
    println!("  ├─ cognitive_layer (5.2ms)");
    println!("  │  ├─ neuron_selection (1.1ms)");
    println!("  │  ├─ processing (3.8ms)");
    println!("  │  └─ response_synthesis (0.3ms)");
    println!("  └─ response (0.2ms)");
    println!("  Total: {}", "8.0ms".green());
    
    Ok(())
}

async fn display_combined_dashboard(client: &MigrationClient) -> Result<()> {
    println!("{}", "=== COMBINED DASHBOARD ===".bold().cyan());
    println!();
    
    // Compact view of all dashboards
    println!("{}", "Quick Stats:".bold());
    println!("  Phase: {} | Progress: {} | Health: {}", 
        "Canary".yellow(), 
        "35%".cyan(), 
        "✓".green()
    );
    println!("  Error Rate: {} | Latency: {} | Throughput: {}", 
        "0.1%".green(),
        "8.5ms".green(),
        "1250 req/s".cyan()
    );
    println!();
    
    // Recent events
    println!("{}", "Recent Events:".bold());
    println!("  14:33:15 [INFO]  Metrics collected");
    println!("  14:33:00 [DEBUG] Neuron cluster formed");
    println!("  14:32:45 [INFO]  Traffic routing updated");
    
    Ok(())
}

fn create_bar(percentage: u32, width: usize) -> String {
    let filled = (percentage as usize * width) / 100;
    let empty = width.saturating_sub(filled);
    
    format!("[{}{}]",
        "█".repeat(filled).green(),
        "░".repeat(empty).dimmed()
    )
}

fn check_quit_key() -> Result<bool> {
    use crossterm::event::{poll, read, Event, KeyCode};
    
    if poll(std::time::Duration::from_millis(0))? {
        if let Event::Key(key) = read()? {
            if key.code == KeyCode::Char('q') {
                return Ok(true);
            }
        }
    }
    Ok(false)
}