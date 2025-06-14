// Cargo.toml
/*
[package]
name = "hal9-neuron-network"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = { version = "1", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
anyhow = "1.0"
tracing = "0.1"
tracing-subscriber = "0.3"
uuid = { version = "1.0", features = ["v4", "serde"] }
async-trait = "0.1"
clap = { version = "4.0", features = ["derive"] }
config = "0.13"
chrono = { version = "0.4", features = ["serde"] }
dashmap = "5.5"
*/

use anyhow::{Context, Result};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::Stdio;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::process::{Child, Command};
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

// ===== Core Types =====

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeuronSignal {
    pub signal_id: Uuid,
    pub from_neuron: String,
    pub to_neuron: String,
    pub layer_from: String,
    pub layer_to: String,
    pub propagation_type: PropagationType,
    pub batch_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub payload: SignalPayload,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PropagationType {
    Forward,
    Backward,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalPayload {
    pub activation: Activation,
    pub gradient: Option<Gradient>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Activation {
    pub content: String,
    pub strength: f32,
    pub features: HashMap<String, f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Gradient {
    pub error_type: String,
    pub magnitude: f32,
    pub adjustments: Vec<String>,
    pub loss: f32,
}

// ===== Configuration =====

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub server_id: String,
    pub listen_addr: String,
    pub listen_port: u16,
    pub remote_servers: Vec<RemoteServer>,
    pub neurons: Vec<NeuronConfig>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RemoteServer {
    pub id: String,
    pub addr: String,
    pub port: u16,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NeuronConfig {
    pub id: String,
    pub layer: String,
    pub claude_command: String,
    pub forward_connections: Vec<String>,
    pub backward_connections: Vec<String>,
}

// ===== Claude Code Neuron Wrapper =====

pub struct ClaudeNeuron {
    pub id: String,
    pub layer: String,
    pub process: Child,
    pub stdin_tx: mpsc::Sender<String>,
    pub stdout_rx: mpsc::Receiver<String>,
}

impl ClaudeNeuron {
    pub async fn spawn(config: &NeuronConfig) -> Result<Self> {
        info!("Spawning Claude neuron: {}", config.id);
        
        let mut cmd = Command::new(&config.claude_command);
        cmd.stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
            
        // Add neuron-specific environment variables
        cmd.env("NEURON_ID", &config.id)
            .env("NEURON_LAYER", &config.layer);
            
        let mut process = cmd.spawn()
            .context("Failed to spawn Claude process")?;
            
        let stdin = process.stdin.take().expect("Failed to get stdin");
        let stdout = process.stdout.take().expect("Failed to get stdout");
        
        // Create channels for async communication
        let (stdin_tx, mut stdin_rx) = mpsc::channel::<String>(100);
        let (stdout_tx, stdout_rx) = mpsc::channel::<String>(100);
        
        // Stdin writer task
        tokio::spawn(async move {
            let mut stdin = stdin;
            while let Some(msg) = stdin_rx.recv().await {
                if let Err(e) = stdin.write_all(msg.as_bytes()).await {
                    error!("Failed to write to stdin: {}", e);
                    break;
                }
                if let Err(e) = stdin.write_all(b"\n").await {
                    error!("Failed to write newline: {}", e);
                    break;
                }
                if let Err(e) = stdin.flush().await {
                    error!("Failed to flush stdin: {}", e);
                    break;
                }
            }
        });
        
        // Stdout reader task
        tokio::spawn(async move {
            let mut reader = BufReader::new(stdout);
            let mut line = String::new();
            
            loop {
                line.clear();
                match reader.read_line(&mut line).await {
                    Ok(0) => break, // EOF
                    Ok(_) => {
                        if let Err(e) = stdout_tx.send(line.trim().to_string()).await {
                            error!("Failed to send stdout: {}", e);
                            break;
                        }
                    }
                    Err(e) => {
                        error!("Failed to read stdout: {}", e);
                        break;
                    }
                }
            }
        });
        
        Ok(ClaudeNeuron {
            id: config.id.clone(),
            layer: config.layer.clone(),
            process,
            stdin_tx,
            stdout_rx,
        })
    }
    
    pub async fn send_prompt(&self, prompt: &str) -> Result<()> {
        self.stdin_tx.send(prompt.to_string()).await
            .context("Failed to send prompt to neuron")
    }
    
    pub async fn read_response(&mut self) -> Result<Option<String>> {
        Ok(self.stdout_rx.recv().await)
    }
    
    pub async fn process_signal(&mut self, signal: &NeuronSignal) -> Result<String> {
        // Format signal as prompt for Claude
        let prompt = self.format_prompt(signal);
        
        // Send to Claude process
        self.send_prompt(&prompt).await?;
        
        // Collect response (might be multi-line)
        let mut response = String::new();
        let timeout = tokio::time::Duration::from_secs(30);
        
        loop {
            match tokio::time::timeout(timeout, self.read_response()).await {
                Ok(Some(line)) => {
                    if line == "END_RESPONSE" {
                        break;
                    }
                    response.push_str(&line);
                    response.push('\n');
                }
                Ok(None) => break,
                Err(_) => {
                    warn!("Timeout waiting for response from neuron {}", self.id);
                    break;
                }
            }
        }
        
        Ok(response.trim().to_string())
    }
    
    fn format_prompt(&self, signal: &NeuronSignal) -> String {
        match signal.propagation_type {
            PropagationType::Forward => {
                format!(
                    "FORWARD_SIGNAL\nFrom: {}\nLayer: {}\nStrength: {}\nContent: {}\nFeatures: {:?}\n",
                    signal.from_neuron,
                    signal.layer_from,
                    signal.payload.activation.strength,
                    signal.payload.activation.content,
                    signal.payload.activation.features
                )
            }
            PropagationType::Backward => {
                let gradient = signal.payload.gradient.as_ref().unwrap();
                format!(
                    "BACKWARD_SIGNAL\nFrom: {}\nError: {}\nMagnitude: {}\nLoss: {}\nAdjustments: {:?}\n",
                    signal.from_neuron,
                    gradient.error_type,
                    gradient.magnitude,
                    gradient.loss,
                    gradient.adjustments
                )
            }
        }
    }
}

// ===== HAL9 Server =====

pub struct HAL9Server {
    config: ServerConfig,
    neurons: Arc<DashMap<String, Arc<RwLock<ClaudeNeuron>>>>,
    routing_table: Arc<DashMap<String, RoutingEntry>>,
    remote_connections: Arc<DashMap<String, TcpStream>>,
    signal_queue: mpsc::Sender<NeuronSignal>,
}

#[derive(Debug, Clone)]
pub enum RoutingEntry {
    Local,
    Remote(String), // server_id
}

impl HAL9Server {
    pub async fn new(config: ServerConfig) -> Result<Self> {
        let (signal_tx, signal_rx) = mpsc::channel(1000);
        
        let server = Self {
            config,
            neurons: Arc::new(DashMap::new()),
            routing_table: Arc::new(DashMap::new()),
            remote_connections: Arc::new(DashMap::new()),
            signal_queue: signal_tx,
        };
        
        // Start signal processor
        server.start_signal_processor(signal_rx);
        
        Ok(server)
    }
    
    pub async fn start(&self) -> Result<()> {
        // 1. Spawn local neurons
        for neuron_config in &self.config.neurons {
            let neuron = ClaudeNeuron::spawn(neuron_config).await?;
            self.neurons.insert(
                neuron_config.id.clone(),
                Arc::new(RwLock::new(neuron))
            );
            self.routing_table.insert(
                neuron_config.id.clone(),
                RoutingEntry::Local
            );
            
            info!("Started neuron: {} on layer {}", neuron_config.id, neuron_config.layer);
        }
        
        // 2. Connect to remote servers
        for remote in &self.config.remote_servers {
            self.connect_to_remote(remote).await?;
        }
        
        // 3. Start TCP listener
        self.start_tcp_listener().await?;
        
        info!("HAL9 Server {} started", self.config.server_id);
        
        Ok(())
    }
    
    async fn connect_to_remote(&self, remote: &RemoteServer) -> Result<()> {
        let addr = format!("{}:{}", remote.addr, remote.port);
        let stream = TcpStream::connect(&addr).await
            .context(format!("Failed to connect to remote server {}", remote.id))?;
            
        self.remote_connections.insert(remote.id.clone(), stream);
        
        info!("Connected to remote server: {}", remote.id);
        
        Ok(())
    }
    
    async fn start_tcp_listener(&self) -> Result<()> {
        let addr = format!("{}:{}", self.config.listen_addr, self.config.listen_port);
        let listener = TcpListener::bind(&addr).await?;
        
        info!("TCP listener started on {}", addr);
        
        let signal_queue = self.signal_queue.clone();
        
        tokio::spawn(async move {
            loop {
                match listener.accept().await {
                    Ok((stream, addr)) => {
                        info!("New connection from {}", addr);
                        
                        let queue = signal_queue.clone();
                        tokio::spawn(async move {
                            if let Err(e) = handle_tcp_connection(stream, queue).await {
                                error!("TCP connection error: {}", e);
                            }
                        });
                    }
                    Err(e) => {
                        error!("Failed to accept connection: {}", e);
                    }
                }
            }
        });
        
        Ok(())
    }
    
    fn start_signal_processor(&self, mut rx: mpsc::Receiver<NeuronSignal>) {
        let neurons = self.neurons.clone();
        let routing_table = self.routing_table.clone();
        let remote_connections = self.remote_connections.clone();
        let signal_queue = self.signal_queue.clone();
        
        tokio::spawn(async move {
            while let Some(signal) = rx.recv().await {
                debug!("Processing signal: {} -> {}", signal.from_neuron, signal.to_neuron);
                
                // Route signal to appropriate handler
                if let Some(routing) = routing_table.get(&signal.to_neuron) {
                    match routing.value() {
                        RoutingEntry::Local => {
                            // Process locally
                            if let Some(neuron_lock) = neurons.get(&signal.to_neuron) {
                                let neuron_lock = neuron_lock.clone();
                                let signal_clone = signal.clone();
                                let queue = signal_queue.clone();
                                
                                tokio::spawn(async move {
                                    let mut neuron = neuron_lock.write().await;
                                    match neuron.process_signal(&signal_clone).await {
                                        Ok(response) => {
                                            // Parse response and generate new signals
                                            if let Ok(new_signals) = parse_neuron_response(
                                                &neuron.id,
                                                &neuron.layer,
                                                &response,
                                                &signal_clone
                                            ) {
                                                for new_signal in new_signals {
                                                    if let Err(e) = queue.send(new_signal).await {
                                                        error!("Failed to queue signal: {}", e);
                                                    }
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            error!("Neuron {} processing error: {}", neuron.id, e);
                                        }
                                    }
                                });
                            }
                        }
                        RoutingEntry::Remote(server_id) => {
                            // Send to remote server
                            if let Some(stream) = remote_connections.get(server_id) {
                                let signal_json = serde_json::to_string(&signal).unwrap();
                                // TODO: Implement proper TCP write
                                debug!("Sending signal to remote server {}", server_id);
                            }
                        }
                    }
                }
            }
        });
    }
    
    pub async fn send_signal(&self, signal: NeuronSignal) -> Result<()> {
        self.signal_queue.send(signal).await
            .context("Failed to queue signal")
    }
}

// ===== Helper Functions =====

async fn handle_tcp_connection(
    mut stream: TcpStream,
    signal_queue: mpsc::Sender<NeuronSignal>
) -> Result<()> {
    let mut reader = BufReader::new(&mut stream);
    let mut line = String::new();
    
    loop {
        line.clear();
        match reader.read_line(&mut line).await {
            Ok(0) => break, // Connection closed
            Ok(_) => {
                if let Ok(signal) = serde_json::from_str::<NeuronSignal>(&line) {
                    signal_queue.send(signal).await?;
                }
            }
            Err(e) => {
                error!("Failed to read from TCP: {}", e);
                break;
            }
        }
    }
    
    Ok(())
}

fn parse_neuron_response(
    neuron_id: &str,
    neuron_layer: &str,
    response: &str,
    original_signal: &NeuronSignal
) -> Result<Vec<NeuronSignal>> {
    // TODO: Implement proper response parsing
    // For now, just create a simple forward signal
    
    let mut signals = Vec::new();
    
    // Extract target neurons and content from response
    // This is a simplified version - real implementation would parse Claude's structured output
    
    if response.contains("FORWARD_TO:") {
        // Parse forward targets
        // Example: "FORWARD_TO: neuron-3, neuron-4\nCONTENT: ..."
    } else if response.contains("BACKWARD_TO:") {
        // Parse backward propagation
    }
    
    Ok(signals)
}

// ===== Main Entry Point =====

use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Configuration file path
    #[arg(short, long)]
    config: String,
    
    /// Server ID (overrides config)
    #[arg(short, long)]
    server_id: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    let args = Args::parse();
    
    // Load configuration
    let config_str = std::fs::read_to_string(&args.config)?;
    let mut config: ServerConfig = serde_json::from_str(&config_str)?;
    
    if let Some(server_id) = args.server_id {
        config.server_id = server_id;
    }
    
    // Start server
    let server = HAL9Server::new(config).await?;
    server.start().await?;
    
    // Keep running
    tokio::signal::ctrl_c().await?;
    
    info!("Shutting down HAL9 server");
    
    Ok(())
}

// ===== Topology Configuration =====

#[derive(Debug, Clone, Deserialize)]
pub struct TopologyConfig {
    pub version: String,
    pub description: String,
    pub servers: Vec<ServerConfig>,
    pub global_routing: HashMap<String, String>, // neuron_id -> server_id
}

// ===== Enhanced Routing Table Builder =====

impl HAL9Server {
    pub async fn build_routing_table(&self, topology: &TopologyConfig) -> Result<()> {
        // Build global routing table from topology config
        for (neuron_id, server_id) in &topology.global_routing {
            if server_id == &self.config.server_id {
                self.routing_table.insert(neuron_id.clone(), RoutingEntry::Local);
            } else {
                self.routing_table.insert(neuron_id.clone(), RoutingEntry::Remote(server_id.clone()));
            }
        }
        
        info!("Built routing table with {} entries", self.routing_table.len());
        Ok(())
    }
}

// ===== Example Configurations =====
/*
// topology.json - 유연한 네트워크 토폴로지 정의
{
    "version": "1.0",
    "description": "Flexible 7-neuron network",
    "servers": [
        {
            "server_id": "hal9-0",
            "listen_addr": "0.0.0.0",
            "listen_port": 8081
        },
        {
            "server_id": "hal9-1", 
            "listen_addr": "0.0.0.0",
            "listen_port": 9080
        }
    ],
    "global_routing": {
        "neuron-1": "hal9-0",
        "neuron-2": "hal9-0",
        "neuron-3": "hal9-1",
        "neuron-4": "hal9-0",
        "neuron-5": "hal9-0",
        "neuron-6": "hal9-1",
        "neuron-7": "hal9-1"
    }
}

// server0.json - 서버별 상세 설정
{
    "server_id": "hal9-0",
    "listen_addr": "0.0.0.0",
    "listen_port": 8081,
    "remote_servers": [
        {
            "id": "hal9-1",
            "addr": "192.168.1.101",
            "port": 9080
        }
    ],
    "neurons": [
        {
            "id": "neuron-1",
            "layer": "L4",
            "claude_command": "claude-code",
            "forward_connections": ["neuron-2", "neuron-3"],
            "backward_connections": []
        },
        {
            "id": "neuron-2",
            "layer": "L3",
            "claude_command": "claude-code",
            "forward_connections": ["neuron-4", "neuron-5", "neuron-6", "neuron-7"],
            "backward_connections": ["neuron-1"]
        },
        {
            "id": "neuron-4",
            "layer": "L2",
            "claude_command": "claude-code",
            "forward_connections": [],
            "backward_connections": ["neuron-2", "neuron-3"]
        },
        {
            "id": "neuron-5",
            "layer": "L2",
            "claude_command": "claude-code",
            "forward_connections": [],
            "backward_connections": ["neuron-2", "neuron-3"]
        }
    ]
}

// server1.json
{
    "server_id": "hal9-1",
    "listen_addr": "0.0.0.0",
    "listen_port": 9080,
    "remote_servers": [
        {
            "id": "hal9-0",
            "addr": "192.168.1.100",
            "port": 8081
        }
    ],
    "neurons": [
        {
            "id": "neuron-3",
            "layer": "L3",
            "claude_command": "claude-code",
            "forward_connections": ["neuron-4", "neuron-5", "neuron-6", "neuron-7"],
            "backward_connections": ["neuron-1"]
        },
        {
            "id": "neuron-6",
            "layer": "L2",
            "claude_command": "claude-code",
            "forward_connections": [],
            "backward_connections": ["neuron-2", "neuron-3"]
        },
        {
            "id": "neuron-7",
            "layer": "L2",
            "claude_command": "claude-code",
            "forward_connections": [],
            "backward_connections": ["neuron-2", "neuron-3"]
        }
    ]
}

// Alternative topology examples:

// topology-layered.json - 레이어별 분리
{
    "version": "1.0",
    "description": "Layer-based separation",
    "global_routing": {
        "neuron-1": "hal9-0",  // L4 layer on server 0
        "neuron-2": "hal9-0",  // L3 layer on server 0  
        "neuron-3": "hal9-0",  // L3 layer on server 0
        "neuron-4": "hal9-1",  // L2 layer on server 1
        "neuron-5": "hal9-1",  // L2 layer on server 1
        "neuron-6": "hal9-1",  // L2 layer on server 1
        "neuron-7": "hal9-1"   // L2 layer on server 1
    }
}

// topology-balanced.json - 부하 분산
{
    "version": "1.0", 
    "description": "Load balanced distribution",
    "global_routing": {
        "neuron-1": "hal9-0",
        "neuron-2": "hal9-0",
        "neuron-3": "hal9-1",
        "neuron-4": "hal9-0",
        "neuron-5": "hal9-1",
        "neuron-6": "hal9-0",
        "neuron-7": "hal9-1"
    }
}
*/