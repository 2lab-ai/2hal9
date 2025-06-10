//! Start command implementation

use anyhow::{Context, Result};
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use std::path::PathBuf;

use hal9_core::ServerConfig;
use hal9_server::HAL9Server;

pub async fn execute(config_path: PathBuf, daemon: bool) -> Result<()> {
    if daemon {
        println!("{}", "Daemon mode not yet implemented".yellow());
        return Ok(());
    }

    println!(
        "{} {}",
        "Loading configuration from".green(),
        config_path.display()
    );

    // Load configuration
    let config_str =
        std::fs::read_to_string(&config_path).context("Failed to read configuration file")?;

    let config: ServerConfig = if config_path.extension().unwrap_or_default() == "yaml" {
        serde_yaml::from_str(&config_str).context("Failed to parse YAML configuration")?
    } else {
        serde_json::from_str(&config_str).context("Failed to parse JSON configuration")?
    };

    println!("{} {}", "Starting server:".green(), config.server_id.cyan());
    println!("{} {} neurons", "Configured".green(), config.neurons.len());

    // Create progress bar
    let pb = ProgressBar::new(config.neurons.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("#>-"),
    );

    // Create and start server
    let server = HAL9Server::new(config);

    // Start with progress updates
    pb.set_message("Initializing server...");
    server.start().await.context("Failed to start server")?;
    pb.finish_with_message(format!("{}", "Server started successfully!".green()));

    println!();
    println!("{}", "Server is running. Press Ctrl+C to stop.".blue());
    println!();

    // Keep running until interrupted
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            println!("\n{}", "Shutting down...".yellow());
            server.shutdown().await?;
            println!("{}", "Server stopped.".green());
        }
    }

    Ok(())
}
