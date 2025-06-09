use anyhow::Result;
use chrono::{DateTime, Utc};
use colored::Colorize;
use comfy_table::{Table, Cell, Attribute};
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;
use tracing::info;

use crate::client::MigrationClient;
use crate::OutputFormat;
use super::{MigrationStatus, MigrationMetrics, format_output};

pub async fn show(
    server: &str,
    detailed: bool,
    format: &OutputFormat,
) -> Result<()> {
    let client = MigrationClient::new(server)?;
    let status = get_migration_status(&client).await?;
    
    match format {
        OutputFormat::Pretty | OutputFormat::Table => {
            display_status(&status, detailed)?;
        }
        OutputFormat::Json => {
            format_output(&status, format)?;
        }
    }
    
    Ok(())
}

pub async fn watch(
    server: &str,
    detailed: bool,
    interval: u64,
    format: &OutputFormat,
) -> Result<()> {
    let client = MigrationClient::new(server)?;
    
    if matches!(format, OutputFormat::Pretty) {
        println!("{}", "👁️  Watching migration status...".bold());
        println!("Press Ctrl+C to stop\n");
    }
    
    loop {
        // Clear screen for pretty output
        if matches!(format, OutputFormat::Pretty) {
            print!("\x1B[2J\x1B[1;1H");
        }
        
        let status = get_migration_status(&client).await?;
        
        match format {
            OutputFormat::Pretty | OutputFormat::Table => {
                display_status(&status, detailed)?;
            }
            OutputFormat::Json => {
                format_output(&status, format)?;
            }
        }
        
        tokio::time::sleep(Duration::from_secs(interval)).await;
    }
}

async fn get_migration_status(client: &MigrationClient) -> Result<MigrationStatus> {
    // Mock implementation - would call actual API
    info!("Fetching migration status");
    
    Ok(MigrationStatus {
        current_phase: "canary".to_string(),
        started_at: Utc::now() - chrono::Duration::hours(2),
        progress: 0.35,
        is_healthy: true,
        active_features: vec![
            "hierarchical_neurons".to_string(),
            "substrate_abstraction".to_string(),
            "protocol_negotiation".to_string(),
        ],
        metrics: MigrationMetrics {
            total_neurons: 100,
            migrated_neurons: 35,
            error_rate: 0.001,
            latency_p99: 8.5,
            throughput_rps: 1250.0,
        },
    })
}

fn display_status(status: &MigrationStatus, detailed: bool) -> Result<()> {
    // Header with current phase
    let phase_color = match status.current_phase.as_str() {
        "shadow" => "🌑 Shadow Mode".dimmed(),
        "canary" => "🐤 Canary Deployment".yellow(),
        "state-migration" => "📦 State Migration".blue(),
        "ramp-up" => "📈 Ramp Up".cyan(),
        "full" => "🚀 Full Migration".green(),
        _ => status.current_phase.normal(),
    };
    
    println!("{} {}", "Current Phase:".bold(), phase_color.bold());
    
    // Progress bar
    let pb = ProgressBar::new(100);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{bar:40.cyan/blue}] {pos}% {msg}")
            .unwrap()
            .progress_chars("█▓▒░")
    );
    pb.set_position((status.progress * 100.0) as u64);
    pb.set_message(format!("{}/{} neurons migrated", 
        status.metrics.migrated_neurons, 
        status.metrics.total_neurons
    ));
    pb.finish();
    
    // Status info
    println!();
    println!("{}", "Migration Info:".bold().underline());
    println!("Started:      {}", format_duration_ago(status.started_at));
    println!("Health:       {}", 
        if status.is_healthy { 
            "✓ Healthy".green() 
        } else { 
            "✗ Unhealthy".red() 
        }
    );
    
    // Active features
    println!();
    println!("{}", "Active Features:".bold().underline());
    for feature in &status.active_features {
        println!("  • {}", feature.cyan());
    }
    
    // Metrics
    println!();
    println!("{}", "Performance Metrics:".bold().underline());
    
    let mut table = Table::new();
    table.set_header(vec![
        Cell::new("Metric").add_attribute(Attribute::Bold),
        Cell::new("Value").add_attribute(Attribute::Bold),
        Cell::new("Status").add_attribute(Attribute::Bold),
    ]);
    
    // Error rate
    let error_status = if status.metrics.error_rate < 0.01 {
        "✓ Good".green()
    } else if status.metrics.error_rate < 0.05 {
        "⚠ Warning".yellow()
    } else {
        "✗ High".red()
    };
    table.add_row(vec![
        Cell::new("Error Rate"),
        Cell::new(format!("{:.2}%", status.metrics.error_rate * 100.0)),
        Cell::new(error_status),
    ]);
    
    // Latency
    let latency_status = if status.metrics.latency_p99 < 10.0 {
        "✓ Good".green()
    } else if status.metrics.latency_p99 < 50.0 {
        "⚠ Warning".yellow()
    } else {
        "✗ High".red()
    };
    table.add_row(vec![
        Cell::new("Latency (p99)"),
        Cell::new(format!("{:.1}ms", status.metrics.latency_p99)),
        Cell::new(latency_status),
    ]);
    
    // Throughput
    table.add_row(vec![
        Cell::new("Throughput"),
        Cell::new(format!("{:.0} req/s", status.metrics.throughput_rps)),
        Cell::new("✓ Normal".green()),
    ]);
    
    println!("{table}");
    
    if detailed {
        println!();
        println!("{}", "Detailed Migration Progress:".bold().underline());
        
        // Show per-layer progress
        let layers = vec![
            ("Substrate Layer", 100),
            ("Protocol Layer", 100),
            ("Cognitive Layer", 35),
            ("Orchestration Layer", 0),
            ("Intelligence Layer", 0),
        ];
        
        for (layer, progress) in layers {
            let bar = create_mini_progress_bar(progress, 20);
            println!("{:<20} {} {:>3}%", layer, bar, progress);
        }
        
        // Show recent events
        println!();
        println!("{}", "Recent Events:".bold().underline());
        println!("  {} Canary deployment started", "2h ago".dimmed());
        println!("  {} Feature flag 'hierarchical_neurons' enabled", "1h ago".dimmed());
        println!("  {} Health check passed", "30m ago".dimmed());
        println!("  {} Traffic increased to 35%", "15m ago".dimmed());
    }
    
    Ok(())
}

fn format_duration_ago(time: DateTime<Utc>) -> String {
    let duration = Utc::now() - time;
    
    if duration.num_days() > 0 {
        format!("{} days ago", duration.num_days())
    } else if duration.num_hours() > 0 {
        format!("{} hours ago", duration.num_hours())
    } else if duration.num_minutes() > 0 {
        format!("{} minutes ago", duration.num_minutes())
    } else {
        "just now".to_string()
    }
}

fn create_mini_progress_bar(percentage: u8, width: usize) -> String {
    let filled = (percentage as usize * width) / 100;
    let empty = width - filled;
    
    format!("{}{}{}{}",
        "[".dimmed(),
        "█".repeat(filled).green(),
        "░".repeat(empty).dimmed(),
        "]".dimmed()
    )
}