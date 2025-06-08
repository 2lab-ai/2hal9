//! 2HAL9 Server main entry point

use std::sync::Arc;
use anyhow::Result;
use tracing::{info, error};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use tokio::signal;

use hal9_core::ServerConfig;
use hal9_server::{HAL9Server, api};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,hal9=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    
    info!("Starting 2HAL9 server v{}", env!("CARGO_PKG_VERSION"));
    
    // Load configuration
    let config = load_config().await?;
    
    // Create server
    let mut server = HAL9Server::new(config.clone());
    
    // Initialize auth if enabled
    if config.auth.enabled {
        let auth_pool = sqlx::SqlitePool::connect(&format!("sqlite:{}?mode=rwc", config.auth.database_path))
            .await
            .map_err(|e| anyhow::anyhow!("Failed to connect to auth database: {}", e))?;
        
        server.initialize_auth(auth_pool).await?;
    }
    
    let server = Arc::new(server);
    
    // Start the server
    server.start().await?;
    
    // Create HTTP API router
    let api_router = api::create_api_router(server.clone());
    
    // Start HTTP server - use env var or default
    let http_port = std::env::var("HTTP_PORT").unwrap_or_else(|_| "8080".to_string());
    let http_addr = format!("127.0.0.1:{}", http_port);
    
    info!("Starting HTTP server on {}", http_addr);
    
    let listener = tokio::net::TcpListener::bind(&http_addr).await?;
    
    // Spawn HTTP server task
    let http_handle = tokio::spawn(async move {
        if let Err(e) = axum::serve(listener, api_router).await {
            error!("HTTP server error: {}", e);
        }
    });
    
    // Wait for shutdown signal
    shutdown_signal().await;
    
    info!("Shutdown signal received, stopping server...");
    
    // Shutdown server
    server.shutdown().await?;
    
    // Wait for HTTP server to stop
    http_handle.abort();
    
    info!("Server stopped");
    Ok(())
}

async fn load_config() -> Result<ServerConfig> {
    // Check for config file argument
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() > 1 {
        // Load from specified file
        let config_path = &args[1];
        info!("Loading configuration from: {}", config_path);
        let config_str = tokio::fs::read_to_string(config_path).await?;
        let config: ServerConfig = serde_yaml::from_str(&config_str)?;
        Ok(config)
    } else {
        // Create default config for testing
        info!("Using default configuration");
        Ok(create_default_config())
    }
}

fn create_default_config() -> ServerConfig {
    use hal9_core::{NeuronConfig, config::{ClaudeConfig, MonitoringConfig, MockResponse, NetworkConfig}};
    use std::collections::HashMap;
    
    // Create mock responses for demo
    let mut mock_responses = HashMap::new();
    
    // L4 responses
    mock_responses.insert("L4".to_string(), vec![
        MockResponse {
            trigger: "default".to_string(),
            response: "FORWARD_TO: neuron-l3-design\nCONTENT: Breaking down the request into design requirements".to_string(),
            delay_ms: 100,
        },
    ]);
    
    // L3 responses
    mock_responses.insert("L3".to_string(), vec![
        MockResponse {
            trigger: "default".to_string(),
            response: "FORWARD_TO: neuron-l2-impl\nCONTENT: Creating implementation plan based on design".to_string(),
            delay_ms: 100,
        },
    ]);
    
    // L2 responses
    mock_responses.insert("L2".to_string(), vec![
        MockResponse {
            trigger: "default".to_string(),
            response: "RESULT: Implementation complete\n```python\n# Generated code\nprint('Hello from 2HAL9!')\n```".to_string(),
            delay_ms: 100,
        },
    ]);
    
    ServerConfig {
        server_id: "hal9-server-1".to_string(),
        neurons: vec![
            NeuronConfig {
                id: "neuron-l4-strategic".to_string(),
                layer: "L4".to_string(),
                claude_command: "claude".to_string(),
                system_prompt: Some("You are a strategic layer neuron".to_string()),
                forward_connections: vec!["neuron-l3-design".to_string()],
                backward_connections: vec![],
                settings: HashMap::new(),
            },
            NeuronConfig {
                id: "neuron-l3-design".to_string(),
                layer: "L3".to_string(),
                claude_command: "claude".to_string(),
                system_prompt: Some("You are a design layer neuron".to_string()),
                forward_connections: vec!["neuron-l2-impl".to_string()],
                backward_connections: vec!["neuron-l4-strategic".to_string()],
                settings: HashMap::new(),
            },
            NeuronConfig {
                id: "neuron-l2-impl".to_string(),
                layer: "L2".to_string(),
                claude_command: "claude".to_string(),
                system_prompt: Some("You are an implementation layer neuron".to_string()),
                forward_connections: vec![],
                backward_connections: vec!["neuron-l3-design".to_string()],
                settings: HashMap::new(),
            },
        ],
        claude: ClaudeConfig {
            mode: "mock".to_string(),
            api_key: None,
            model: "claude-3-sonnet-20240229".to_string(),
            temperature: 0.7,
            max_tokens: 4000,
            rate_limit: 10,
            mock_responses,
            fallback_to_mock: false, // Not needed in mock mode
            cost_controls: Default::default(),
        },
        monitoring: MonitoringConfig::default(),
        network: NetworkConfig::default(),
        memory: Default::default(),
        backward_propagation: Default::default(),
        auth: Default::default(),
    }
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}