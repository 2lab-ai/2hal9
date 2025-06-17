//! 2HAL9 CLI - Command line interface for 2HAL9 neural network

use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::Colorize;
use std::path::PathBuf;
use tracing::error;

mod commands;
use commands::{signal, start, status, stop};

#[derive(Parser)]
#[command(
    name = "hal9",
    about = "2HAL9 - Hierarchical AI Neural Network",
    version,
    author,
    long_about = "A distributed AI consciousness system implementing hierarchical abstraction through networked AI neurons"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Verbose output
    #[arg(short, long, global = true)]
    verbose: bool,

    /// Quiet mode (errors only)
    #[arg(short, long, global = true)]
    quiet: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Start a 2HAL9 server
    Start {
        /// Configuration file path
        #[arg(short, long, default_value = "config.yaml")]
        config: PathBuf,

        /// Run in background
        #[arg(short, long)]
        daemon: bool,
    },

    /// Show server status
    Status {
        /// Server address
        #[arg(short, long, default_value = "localhost:8080")]
        server: String,

        /// Output format (text, json)
        #[arg(short, long, default_value = "text")]
        format: String,
    },

    /// Send a signal to a neuron
    Signal {
        /// Source neuron (use "user" for external input)
        #[arg(short, long, default_value = "user")]
        from: String,

        /// Target neuron ID
        #[arg(short, long)]
        to: String,

        /// Signal content
        #[arg(short, long)]
        content: String,

        /// Server address
        #[arg(short, long, default_value = "localhost:8080")]
        server: String,
    },

    /// Stop a running server
    Stop {
        /// Server address
        #[arg(short, long, default_value = "localhost:8080")]
        server: String,

        /// Force shutdown without grace period
        #[arg(short, long)]
        force: bool,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    let log_level = if cli.quiet {
        "error"
    } else if cli.verbose {
        "debug"
    } else {
        "info"
    };

    tracing_subscriber::fmt()
        .with_env_filter(log_level)
        .with_target(false)
        .with_thread_ids(false)
        .with_thread_names(false)
        .init();

    // Print banner
    if !cli.quiet {
        print_banner();
    }

    // Execute command
    let result = match cli.command {
        Commands::Start { config, daemon } => start::execute(config, daemon).await,
        Commands::Status { server, format } => status::execute(server, format).await,
        Commands::Signal {
            from,
            to,
            content,
            server,
        } => signal::execute(from, to, content, server).await,
        Commands::Stop { server, force } => stop::execute(server, force).await,
    };

    if let Err(e) = result {
        error!("{}", e.to_string().red());
        std::process::exit(1);
    }

    Ok(())
}

fn print_banner() {
    println!(
        "{}",
        r#"
     ____  _   _    _    _     ___  
    |___ \| | | |  / \  | |   / _ \ 
      __) | |_| | / _ \ | |  | (_) |
     / __/|  _  |/ ___ \| |___\__, |
    |_____|_| |_/_/   \_\_____|  /_/ 
    "#
        .cyan()
    );
    println!("{}", "Hierarchical AI Neural Network".blue());
    println!();
}
