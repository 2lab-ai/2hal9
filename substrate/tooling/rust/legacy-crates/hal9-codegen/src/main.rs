//! HAL9 Code Generation CLI Tool

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use colored::*;
use dialoguer::{theme::ColorfulTheme, Input, Select, MultiSelect, Confirm};
use indicatif::{ProgressBar, ProgressStyle};
use std::path::PathBuf;
use std::time::Duration;

mod api;
mod config;

use api::CodegenClient;
use config::Config;

/// HAL9 Code Generation Assistant
#[derive(Parser)]
#[command(name = "hal9-codegen")]
#[command(about = "AI-powered code generation using HAL9", long_about = None)]
struct Cli {
    /// HAL9 server URL
    #[arg(short, long, env = "HAL9_SERVER_URL", default_value = "http://localhost:8080")]
    server: String,
    
    /// API key for authentication
    #[arg(short = 'k', long, env = "HAL9_API_KEY")]
    api_key: Option<String>,
    
    /// Enable verbose output
    #[arg(short, long)]
    verbose: bool,
    
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate a new project
    New {
        /// Project name
        name: Option<String>,
        
        /// Project type (web-app, api, cli, library, microservice)
        #[arg(short = 't', long)]
        project_type: Option<String>,
        
        /// Skip interactive mode
        #[arg(short = 'y', long)]
        yes: bool,
    },
    
    /// Add a feature to existing project
    Add {
        /// Feature to add (auth, database, api, frontend, testing)
        feature: String,
        
        /// Additional options
        #[arg(short, long)]
        _options: Vec<String>,
    },
    
    /// Generate tests for code
    Test {
        /// File or directory to test
        path: PathBuf,
        
        /// Test framework to use
        #[arg(short, long)]
        _framework: Option<String>,
    },
    
    /// Refactor code
    Refactor {
        /// File to refactor
        file: PathBuf,
        
        /// Refactoring type
        #[arg(short = 't', long)]
        refactor_type: Option<String>,
        
        /// Start line
        #[arg(short = 's', long)]
        start: Option<usize>,
        
        /// End line
        #[arg(short = 'e', long)]
        end: Option<usize>,
    },
    
    /// Review code for issues
    Review {
        /// File or directory to review
        path: PathBuf,
        
        /// Focus areas (security, performance, best-practices, bugs, style)
        #[arg(short, long)]
        focus: Vec<String>,
    },
    
    /// Interactive chat mode
    Chat,
    
    /// Learn from existing codebase
    Learn {
        /// Path to codebase
        path: PathBuf,
        
        /// Project name for reference
        #[arg(short, long)]
        name: String,
    },
    
    /// Generate similar code
    Similar {
        /// Reference file
        reference: PathBuf,
        
        /// Name for new file/component
        name: String,
    },
    
    /// Configure HAL9 codegen
    Config {
        /// Configuration key
        key: Option<String>,
        
        /// Configuration value
        value: Option<String>,
        
        /// List all configurations
        #[arg(short, long)]
        list: bool,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Initialize logging
    if cli.verbose {
        tracing_subscriber::fmt()
            .with_env_filter("hal9_codegen=debug")
            .init();
    }
    
    // Load configuration
    let config = Config::load()?;
    
    // Create API client
    let client = CodegenClient::new(
        &cli.server,
        cli.api_key.or(config.api_key),
    )?;
    
    // Handle commands
    match cli.command {
        Commands::New { name, project_type, yes } => {
            generate_new_project(client, name, project_type, yes).await?;
        }
        Commands::Add { feature, options } => {
            add_feature(client, feature, options).await?;
        }
        Commands::Test { path, framework } => {
            generate_tests(client, path, framework).await?;
        }
        Commands::Refactor { file, refactor_type, start, end } => {
            refactor_code(client, file, refactor_type, start, end).await?;
        }
        Commands::Review { path, focus } => {
            review_code(client, path, focus).await?;
        }
        Commands::Chat => {
            interactive_chat(client).await?;
        }
        Commands::Learn { path, name } => {
            learn_from_codebase(client, path, name).await?;
        }
        Commands::Similar { reference, name } => {
            generate_similar(client, reference, name).await?;
        }
        Commands::Config { key, value, list } => {
            handle_config(key, value, list)?;
        }
    }
    
    Ok(())
}

async fn generate_new_project(
    _client: CodegenClient,
    name: Option<String>,
    project_type: Option<String>,
    skip_interactive: bool,
) -> Result<()> {
    let theme = ColorfulTheme::default();
    
    // Get project details
    let project_name = match name {
        Some(n) => n,
        None if !skip_interactive => Input::with_theme(&theme)
            .with_prompt("Project name")
            .interact_text()?,
        _ => {
            eprintln!("{} Project name is required when using --yes flag", "❌".red());
            eprintln!("Usage: hal9-codegen new <NAME> --type <TYPE> --yes");
            return Err(anyhow::anyhow!("Project name required"));
        }
    };
    
    let project_types = vec!["web-app", "api", "cli", "library", "microservice"];
    let project_type = match project_type {
        Some(t) => t,
        None if !skip_interactive => {
            let selection = Select::with_theme(&theme)
                .with_prompt("Project type")
                .items(&project_types)
                .interact()?;
            project_types[selection].to_string()
        }
        _ => {
            eprintln!("{} Project type is required when using --yes flag", "❌".red());
            eprintln!("Available types: {}", project_types.join(", "));
            eprintln!("Usage: hal9-codegen new <NAME> --type <TYPE> --yes");
            return Err(anyhow::anyhow!("Project type required"));
        }
    };
    
    println!("{}", "📋 Project Configuration".bright_blue().bold());
    
    // Get preferences based on project type
    let (backend, frontend, database) = if !skip_interactive {
        match project_type.as_str() {
            "web-app" => {
                let backends = vec!["fastapi", "express", "django", "axum", "none"];
                let backend_idx = Select::with_theme(&theme)
                    .with_prompt("Backend framework")
                    .items(&backends)
                    .interact()?;
                let backend = if backends[backend_idx] != "none" {
                    Some(backends[backend_idx].to_string())
                } else {
                    None
                };
                
                let frontends = vec!["react", "vue", "angular", "svelte"];
                let frontend_idx = Select::with_theme(&theme)
                    .with_prompt("Frontend framework")
                    .items(&frontends)
                    .interact()?;
                let frontend = Some(frontends[frontend_idx].to_string());
                
                let databases = vec!["postgresql", "mysql", "mongodb", "sqlite", "none"];
                let db_idx = Select::with_theme(&theme)
                    .with_prompt("Database")
                    .items(&databases)
                    .interact()?;
                let database = if databases[db_idx] != "none" {
                    Some(databases[db_idx].to_string())
                } else {
                    None
                };
                
                (backend, frontend, database)
            }
            "api" => {
                let frameworks = vec!["fastapi", "express", "gin", "axum"];
                let framework_idx = Select::with_theme(&theme)
                    .with_prompt("API framework")
                    .items(&frameworks)
                    .interact()?;
                let backend = Some(frameworks[framework_idx].to_string());
                
                let databases = vec!["postgresql", "mysql", "mongodb", "redis"];
                let db_idx = Select::with_theme(&theme)
                    .with_prompt("Database")
                    .items(&databases)
                    .interact()?;
                let database = Some(databases[db_idx].to_string());
                
                (backend, None, database)
            }
            _ => (None, None, None),
        }
    } else {
        (None, None, None)
    };
    
    let features = if !skip_interactive {
        let options = vec!["Authentication", "Testing", "Docker", "CI/CD", "Documentation"];
        let selections = MultiSelect::with_theme(&theme)
            .with_prompt("Additional features")
            .items(&options)
            .interact()?;
        
        (
            selections.contains(&0), // auth
            selections.contains(&1), // testing
            selections.contains(&2), // docker
            selections.contains(&3), // ci/cd
        )
    } else {
        (true, true, true, true)
    };
    
    // Confirm generation
    if !skip_interactive {
        println!("\n{}", "📝 Summary".bright_green().bold());
        println!("  Project: {}", project_name.bright_white());
        println!("  Type: {}", project_type.bright_white());
        if let Some(ref b) = backend {
            println!("  Backend: {}", b.bright_white());
        }
        if let Some(ref f) = frontend {
            println!("  Frontend: {}", f.bright_white());
        }
        if let Some(ref d) = database {
            println!("  Database: {}", d.bright_white());
        }
        println!("  Features: auth={}, testing={}, docker={}, ci/cd={}",
            features.0, features.1, features.2, features.3);
        
        if !Confirm::with_theme(&theme)
            .with_prompt("Generate project?")
            .default(true)
            .interact()? {
            println!("{}", "Cancelled".red());
            return Ok(());
        }
    }
    
    // Start generation
    println!("\n{}", "🚀 Generating project...".bright_blue().bold());
    
    let pb = ProgressBar::new(100);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")?
            .progress_chars("#>-")
    );
    
    // Send request to server
    let description = format!("{} project with chosen stack", project_type);
    let response = client.generate_project(
        &description,
        &project_type,
        backend,
        frontend,
        database,
        features.1, // testing
        features.2, // docker
        features.3, // ci/cd
    ).await?;
    
    // Simulate progress (in production, poll for actual progress)
    for i in 0..100 {
        pb.set_position(i);
        pb.set_message(match i {
            0..=20 => "Creating project structure...",
            21..=40 => "Generating backend code...",
            41..=60 => "Generating frontend code...",
            61..=80 => "Setting up configuration...",
            81..=95 => "Adding tests and documentation...",
            _ => "Finalizing...",
        });
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    
    pb.finish_with_message("Complete!");
    
    println!("\n{} Project generated successfully!", "✅".bright_green());
    println!("📁 Location: {}", response.location.unwrap_or_else(|| "./generated".to_string()).bright_white());
    println!("\n{}", "Next steps:".bright_blue().bold());
    println!("  1. cd {}", project_name);
    println!("  2. Follow the README.md for setup instructions");
    println!("  3. Run 'hal9-codegen add <feature>' to add more features");
    
    Ok(())
}

async fn add_feature(
    _client: CodegenClient,
    feature: String,
    _options: Vec<String>,
) -> Result<()> {
    println!("{} Adding {} to project...", "🔧".bright_blue(), feature.bright_white());
    
    // TODO: Implement feature addition
    println!("{} Feature addition not yet implemented", "⚠️".yellow());
    
    Ok(())
}

async fn generate_tests(
    _client: CodegenClient,
    path: PathBuf,
    _framework: Option<String>,
) -> Result<()> {
    println!("{} Generating tests for {}...", "🧪".bright_blue(), path.display());
    
    // TODO: Implement test generation
    println!("{} Test generation not yet implemented", "⚠️".yellow());
    
    Ok(())
}

async fn refactor_code(
    _client: CodegenClient,
    file: PathBuf,
    refactor_type: Option<String>,
    start: Option<usize>,
    end: Option<usize>,
) -> Result<()> {
    println!("{} Refactoring {}...", "🔨".bright_blue(), file.display());
    
    let content = std::fs::read_to_string(&file)
        .context("Failed to read file")?;
    
    let refactor_type = refactor_type.unwrap_or_else(|| "extract-method".to_string());
    
    let response = client.refactor_code(
        file.to_string_lossy().to_string(),
        refactor_type,
        start,
        end,
    ).await?;
    
    if response.success {
        println!("{} Refactoring complete!", "✅".green());
        for change in response.changes {
            println!("  Line {}: {}", change.line, change.description);
        }
        
        // TODO: Apply changes to file
        println!("\n{} Review and apply changes manually for now", "ℹ️".blue());
    } else {
        println!("{} Refactoring failed", "❌".red());
    }
    
    Ok(())
}

async fn review_code(
    _client: CodegenClient,
    path: PathBuf,
    focus: Vec<String>,
) -> Result<()> {
    println!("{} Reviewing code in {}...", "🔍".bright_blue(), path.display());
    
    let content = std::fs::read_to_string(&path)
        .context("Failed to read file")?;
    
    let response = client.review_code(
        path.to_string_lossy().to_string(),
        content,
        focus,
    ).await?;
    
    println!("\n{}", "Code Review Results".bright_green().bold());
    println!("Overall Score: {}/10", response.overall_score);
    
    if !response.issues.is_empty() {
        println!("\n{}", "Issues Found:".yellow().bold());
        for issue in response.issues {
            let icon = match issue.severity.as_str() {
                "error" => "❌",
                "warning" => "⚠️",
                _ => "ℹ️",
            };
            println!("{} {} ({})", icon, issue.message, issue.severity);
            if let Some(line) = issue.line {
                println!("  Line: {}", line);
            }
            if let Some(suggestion) = issue.suggestion {
                println!("  Suggestion: {}", suggestion.bright_white());
            }
        }
    }
    
    if !response.suggestions.is_empty() {
        println!("\n{}", "Suggestions:".blue().bold());
        for suggestion in response.suggestions {
            println!("💡 {}", suggestion.text);
            println!("   {}", suggestion.description.dimmed());
        }
    }
    
    Ok(())
}

async fn interactive_chat(_client: CodegenClient) -> Result<()> {
    println!("{}", "💬 HAL9 Code Generation Chat".bright_blue().bold());
    println!("Type 'exit' or 'quit' to leave\n");
    
    let theme = ColorfulTheme::default();
    
    loop {
        let input: String = Input::with_theme(&theme)
            .with_prompt(">")
            .interact_text()?;
        
        if input.trim() == "exit" || input.trim() == "quit" {
            println!("{}", "Goodbye!".bright_green());
            break;
        }
        
        // Process chat input
        println!("{} Processing...", "🤔".bright_blue());
        
        // TODO: Implement chat functionality
        println!("{} Chat functionality coming soon!", "🚧".yellow());
    }
    
    Ok(())
}

async fn learn_from_codebase(
    _client: CodegenClient,
    _path: PathBuf,
    _name: String,
) -> Result<()> {
    println!("{} Learning from codebase: {}...", "🧠".bright_blue(), _name.bright_white());
    
    // TODO: Implement learning functionality
    println!("{} Learning functionality not yet implemented", "⚠️".yellow());
    
    Ok(())
}

async fn generate_similar(
    _client: CodegenClient,
    reference: PathBuf,
    _name: String,
) -> Result<()> {
    println!("{} Generating similar code to {}...", "🔄".bright_blue(), reference.display());
    
    // TODO: Implement similar code generation
    println!("{} Similar code generation not yet implemented", "⚠️".yellow());
    
    Ok(())
}

fn handle_config(key: Option<String>, value: Option<String>, list: bool) -> Result<()> {
    let mut config = Config::load()?;
    
    if list {
        println!("{}", "HAL9 Codegen Configuration".bright_blue().bold());
        println!("Server URL: {}", config.server_url.unwrap_or_else(|| "default".to_string()));
        println!("API Key: {}", if config.api_key.is_some() { "***" } else { "not set" });
        return Ok(());
    }
    
    if let (Some(key), Some(value)) = (key, value) {
        match key.as_str() {
            "server" => config.server_url = Some(value),
            "api-key" => config.api_key = Some(value),
            _ => return Err(anyhow::anyhow!("Unknown configuration key: {}", key)),
        }
        config.save()?;
        println!("{} Configuration updated", "✅".green());
    } else {
        println!("{} Usage: hal9-codegen config <key> <value>", "ℹ️".blue());
        println!("Available keys: server, api-key");
    }
    
    Ok(())
}