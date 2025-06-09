use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;
use tracing::{info, warn, error};
use tracing_subscriber::prelude::*;

mod commands;
mod config;
mod client;
mod monitor;
mod state;
mod dashboard;

use commands::{pre_check, status, migrate, rollback, verify};

/// HAL9 Migration CLI - Production-ready migration tooling for HAL9 hierarchical architecture
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    
    /// HAL9 server URL (can also be set via HAL9_SERVER env var)
    #[arg(short, long, env = "HAL9_SERVER", default_value = "http://localhost:3030")]
    server: String,
    
    /// Output format (json, table, pretty)
    #[arg(short, long, default_value = "pretty")]
    format: OutputFormat,
    
    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,
}

#[derive(Clone, Debug, clap::ValueEnum)]
enum OutputFormat {
    Json,
    Table,
    Pretty,
}

#[derive(Subcommand)]
enum Commands {
    /// Pre-migration health checks and validation
    PreCheck {
        /// Perform deep validation (slower but more thorough)
        #[arg(long)]
        deep: bool,
        
        /// Check specific components only
        #[arg(long)]
        components: Vec<String>,
    },
    
    /// Check current migration status
    Status {
        /// Show detailed status including metrics
        #[arg(long)]
        detailed: bool,
        
        /// Watch status continuously
        #[arg(short, long)]
        watch: bool,
        
        /// Update interval in seconds (for watch mode)
        #[arg(long, default_value = "5")]
        interval: u64,
    },
    
    /// Start or continue migration process
    Migrate {
        /// Target migration phase (shadow, canary, state-migration, ramp-up, full)
        #[arg(short, long)]
        phase: MigrationPhase,
        
        /// Percentage of traffic for canary/ramp-up phases
        #[arg(long)]
        percentage: Option<u8>,
        
        /// Dry run - simulate without making changes
        #[arg(long)]
        dry_run: bool,
        
        /// Auto-approve changes (skip confirmation)
        #[arg(long)]
        yes: bool,
        
        /// Maximum duration for migration phase
        #[arg(long)]
        timeout: Option<u64>,
    },
    
    /// Rollback to previous stable state
    Rollback {
        /// Target phase to rollback to
        #[arg(short, long)]
        to_phase: Option<MigrationPhase>,
        
        /// Force rollback even if health checks fail
        #[arg(long)]
        force: bool,
        
        /// Auto-approve rollback (skip confirmation)
        #[arg(long)]
        yes: bool,
    },
    
    /// Verify migration integrity and performance
    Verify {
        /// Run full verification suite
        #[arg(long)]
        full: bool,
        
        /// Specific verification tests to run
        #[arg(long)]
        tests: Vec<String>,
        
        /// Generate detailed report
        #[arg(long)]
        report: bool,
    },
    
    /// Monitor live migration metrics
    Monitor {
        /// Monitoring dashboard type (metrics, logs, traces)
        #[arg(short, long, default_value = "metrics")]
        dashboard: DashboardType,
        
        /// Refresh interval in seconds
        #[arg(short, long, default_value = "1")]
        interval: u64,
    },
    
    /// Manage feature flags for migration
    Feature {
        #[command(subcommand)]
        command: FeatureCommands,
    },
    
    /// Export/import migration state
    State {
        #[command(subcommand)]
        command: StateCommands,
    },
    
    /// Serve web dashboard
    Dashboard {
        /// Port to serve dashboard on
        #[arg(short, long, default_value = "8080")]
        port: u16,
    },
}

#[derive(Clone, Debug, clap::ValueEnum)]
enum MigrationPhase {
    Shadow,
    Canary,
    StateMigration,
    RampUp,
    Full,
}

#[derive(Clone, Debug, clap::ValueEnum)]
enum DashboardType {
    Metrics,
    Logs,
    Traces,
    Combined,
}

#[derive(Subcommand)]
enum FeatureCommands {
    /// List all feature flags
    List,
    
    /// Enable a feature flag
    Enable {
        /// Feature flag name
        name: String,
        
        /// Apply to specific percentage of traffic
        #[arg(long)]
        percentage: Option<u8>,
    },
    
    /// Disable a feature flag
    Disable {
        /// Feature flag name
        name: String,
    },
    
    /// Get status of a specific feature flag
    Status {
        /// Feature flag name
        name: String,
    },
}

#[derive(Subcommand)]
enum StateCommands {
    /// Export current migration state
    Export {
        /// Output file path
        #[arg(short, long)]
        output: String,
        
        /// Include sensitive data
        #[arg(long)]
        include_sensitive: bool,
    },
    
    /// Import migration state from file
    Import {
        /// Input file path
        #[arg(short, long)]
        input: String,
        
        /// Validate only, don't apply
        #[arg(long)]
        validate_only: bool,
    },
    
    /// Create migration checkpoint
    Checkpoint {
        /// Checkpoint name
        #[arg(short, long)]
        name: String,
        
        /// Description
        #[arg(short, long)]
        description: Option<String>,
    },
    
    /// List available checkpoints
    ListCheckpoints,
    
    /// Restore from checkpoint
    Restore {
        /// Checkpoint name or ID
        #[arg(short, long)]
        checkpoint: String,
        
        /// Force restore even if current state is dirty
        #[arg(long)]
        force: bool,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Initialize tracing
    let filter = if cli.verbose {
        "hal9_migrate=debug,hal9=debug,info"
    } else {
        "hal9_migrate=info,warn"
    };
    
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::EnvFilter::new(filter))
        .init();
    
    // ASCII art banner
    if matches!(cli.format, OutputFormat::Pretty) {
        println!("{}", r#"
╦ ╦╔═╗╦  ╔═╗  ╔╦╗╦╔═╗╦═╗╔═╗╔╦╗╔═╗
╠═╣╠═╣║  ╚═╗  ║║║║║ ╦╠╦╝╠═╣ ║ ║╣ 
╩ ╩╩ ╩╩═╝╚═╝  ╩ ╩╩╚═╝╩╚═╩ ╩ ╩ ╚═╝
        "#.cyan().bold());
        println!("{}\n", "Hierarchical Architecture Migration Tool".dimmed());
    }
    
    // Execute command
    match cli.command {
        Commands::PreCheck { deep, components } => {
            pre_check::run(&cli.server, deep, components, &cli.format).await?;
        }
        
        Commands::Status { detailed, watch, interval } => {
            if watch {
                status::watch(&cli.server, detailed, interval, &cli.format).await?;
            } else {
                status::show(&cli.server, detailed, &cli.format).await?;
            }
        }
        
        Commands::Migrate { phase, percentage, dry_run, yes, timeout } => {
            migrate::run(
                &cli.server,
                phase,
                percentage,
                dry_run,
                yes,
                timeout,
                &cli.format
            ).await?;
        }
        
        Commands::Rollback { to_phase, force, yes } => {
            rollback::run(&cli.server, to_phase, force, yes, &cli.format).await?;
        }
        
        Commands::Verify { full, tests, report } => {
            verify::run(&cli.server, full, tests, report, &cli.format).await?;
        }
        
        Commands::Monitor { dashboard, interval } => {
            monitor::run(&cli.server, dashboard, interval).await?;
        }
        
        Commands::Feature { command } => {
            match command {
                FeatureCommands::List => {
                    commands::feature::list(&cli.server, &cli.format).await?;
                }
                FeatureCommands::Enable { name, percentage } => {
                    commands::feature::enable(&cli.server, &name, percentage, &cli.format).await?;
                }
                FeatureCommands::Disable { name } => {
                    commands::feature::disable(&cli.server, &name, &cli.format).await?;
                }
                FeatureCommands::Status { name } => {
                    commands::feature::status(&cli.server, &name, &cli.format).await?;
                }
            }
        }
        
        Commands::State { command } => {
            match command {
                StateCommands::Export { output, include_sensitive } => {
                    state::export(&cli.server, &output, include_sensitive, &cli.format).await?;
                }
                StateCommands::Import { input, validate_only } => {
                    state::import(&cli.server, &input, validate_only, &cli.format).await?;
                }
                StateCommands::Checkpoint { name, description } => {
                    state::checkpoint(&cli.server, &name, description, &cli.format).await?;
                }
                StateCommands::ListCheckpoints => {
                    state::list_checkpoints(&cli.server, &cli.format).await?;
                }
                StateCommands::Restore { checkpoint, force } => {
                    state::restore(&cli.server, &checkpoint, force, &cli.format).await?;
                }
            }
        }
        
        Commands::Dashboard { port } => {
            info!("Starting migration dashboard on port {}", port);
            let server = dashboard::DashboardServer::new(&cli.server)?;
            server.run(port).await?;
        }
    }
    
    Ok(())
}