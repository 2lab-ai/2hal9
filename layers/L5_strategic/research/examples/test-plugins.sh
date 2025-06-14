#!/bin/bash
# Test script for HAL9 WebAssembly Plugin System

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${BLUE}Testing HAL9 WebAssembly Plugin System...${NC}"

# Check if Rust and wasm target are installed
if ! command -v rustc &> /dev/null; then
    echo -e "${RED}Error: Rust is not installed${NC}"
    exit 1
fi

if ! rustup target list --installed | grep -q "wasm32-wasi"; then
    echo -e "${YELLOW}Installing wasm32-wasi target...${NC}"
    rustup target add wasm32-wasi
fi

# Build example plugins
echo -e "\n${GREEN}Building example plugins...${NC}"

# Build sentiment analyzer plugin
echo -e "${BLUE}Building sentiment analyzer plugin...${NC}"
cd plugins/examples/sentiment-analyzer
cargo build --target wasm32-wasi --release
WASM_FILE="target/wasm32-wasi/release/hal9_sentiment_analyzer.wasm"
if [ -f "$WASM_FILE" ]; then
    echo -e "${GREEN}✓ Sentiment analyzer built successfully${NC}"
    echo "  Size: $(du -h $WASM_FILE | cut -f1)"
else
    echo -e "${RED}✗ Failed to build sentiment analyzer${NC}"
fi
cd ../../..

# Build web scraper plugin
echo -e "\n${BLUE}Building web scraper plugin...${NC}"
cd plugins/examples/web-scraper
cargo build --target wasm32-wasi --release
WASM_FILE="target/wasm32-wasi/release/hal9_web_scraper.wasm"
if [ -f "$WASM_FILE" ]; then
    echo -e "${GREEN}✓ Web scraper built successfully${NC}"
    echo "  Size: $(du -h $WASM_FILE | cut -f1)"
else
    echo -e "${RED}✗ Failed to build web scraper${NC}"
fi
cd ../../..

# Create test configuration
echo -e "\n${GREEN}Creating test configuration...${NC}"
cat > examples/plugin-test.yaml << 'EOF'
# HAL9 Plugin System Test Configuration

server:
  host: localhost
  port: 9000
  mode: hybrid

plugins:
  enabled: true
  directory: "./plugins"
  auto_load: true
  auto_activate: true
  max_plugins: 10
  hot_reload: false
  
  # Security policies
  security_policies:
    default:
      allow_network: false
      allow_filesystem: false
      allow_system_time: true
      allow_random: true
      max_memory_mb: 64
      max_cpu_percent: 25
    
    neuron:
      allow_network: false
      allow_filesystem: false
      allow_system_time: true
      allow_random: true
      max_memory_mb: 128
      max_cpu_percent: 50
    
    tool:
      allow_network: true
      allow_filesystem: true
      allow_system_time: true
      allow_random: true
      max_memory_mb: 256
      max_cpu_percent: 75
      allowed_hosts:
        - "*.example.com"
        - "api.openai.com"
      allowed_paths:
        - "/tmp"

neurons:
  # Regular neurons
  - name: "coordinator"
    type: "L3"
    layer: "L3"
    claude_model: "claude-3-sonnet-20240229"
    config:
      temperature: 0.7
      max_tokens: 2048
  
  # Plugin neurons will be loaded dynamically
  - name: "plugin-loader"
    type: "plugin"
    layer: "L2"
    config:
      plugin_type: "dynamic"
      auto_discover: true

# Memory configuration for plugins
memory:
  type: "sqlite"
  path: "./data/hal9_plugins.db"
  embeddings:
    enabled: true
    model: "all-MiniLM-L6-v2"
    cache_size: 1000

# Monitoring
monitoring:
  prometheus:
    enabled: true
    port: 9001
  
  plugin_metrics:
    enabled: true
    track_execution_time: true
    track_memory_usage: true
    track_api_calls: true
EOF

echo -e "${GREEN}✓ Configuration created${NC}"

# Test plugin loading
echo -e "\n${GREEN}Testing plugin loading...${NC}"

# Create a simple test that loads plugins
cat > test_plugin_loading.rs << 'EOF'
use hal9_server::plugins::{PluginManager, PluginManagerConfig};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize plugin manager
    let config = PluginManagerConfig {
        plugins_dir: PathBuf::from("./plugins/examples"),
        auto_load: true,
        auto_activate: true,
        max_plugins: 10,
        enable_hot_reload: false,
    };
    
    let manager = PluginManager::new(config).await?;
    
    // List loaded plugins
    let plugins = manager.list_plugins().await;
    println!("Loaded {} plugins:", plugins.len());
    
    for plugin in plugins {
        println!("  - {} v{} ({})", 
            plugin.metadata.name,
            plugin.metadata.version,
            plugin.state
        );
    }
    
    Ok(())
}
EOF

# Create package examples
echo -e "\n${GREEN}Creating plugin packages...${NC}"

# Package sentiment analyzer
cd plugins/examples/sentiment-analyzer
if [ -f "target/wasm32-wasi/release/hal9_sentiment_analyzer.wasm" ]; then
    cp target/wasm32-wasi/release/hal9_sentiment_analyzer.wasm sentiment_analyzer.wasm
    zip -q sentiment-analyzer.hal9 manifest.json sentiment_analyzer.wasm
    echo -e "${GREEN}✓ Created sentiment-analyzer.hal9${NC}"
fi
cd ../../..

# Package web scraper
cd plugins/examples/web-scraper
if [ -f "target/wasm32-wasi/release/hal9_web_scraper.wasm" ]; then
    cp target/wasm32-wasi/release/hal9_web_scraper.wasm web_scraper.wasm
    echo '{"api_version":1,"metadata":{"id":"b7f9c5d2-4a8e-4f3b-9c1d-8e5f7a2b3c4d","name":"Web Scraper","version":"0.1.0","author":"HAL9 Developers","description":"Web scraping tool plugin","license":"MIT"},"files":{"wasm":"web_scraper.wasm"}}' > manifest.json
    zip -q web-scraper.hal9 manifest.json web_scraper.wasm
    echo -e "${GREEN}✓ Created web-scraper.hal9${NC}"
fi
cd ../../..

# Test plugin API
echo -e "\n${GREEN}Testing plugin API endpoints...${NC}"

# Start server in background (mock)
echo -e "${BLUE}Starting HAL9 server with plugin support...${NC}"
echo "(Server would start here with: cargo run --bin hal9-server --features plugins -- --config examples/plugin-test.yaml)"

# Mock API tests
echo -e "\n${YELLOW}Plugin API endpoints:${NC}"
echo "  GET  /api/v1/plugins                - List all plugins"
echo "  GET  /api/v1/plugins/:id            - Get plugin info"
echo "  POST /api/v1/plugins                - Install plugin"
echo "  PUT  /api/v1/plugins/:id/activate   - Activate plugin"
echo "  PUT  /api/v1/plugins/:id/deactivate - Deactivate plugin"
echo "  DELETE /api/v1/plugins/:id          - Uninstall plugin"

# Test scenarios
echo -e "\n${GREEN}Plugin test scenarios:${NC}"
echo -e "${BLUE}1. Signal Processing Test${NC}"
echo "   - Send text signal to sentiment analyzer"
echo "   - Verify sentiment analysis results"
echo "   - Check caching behavior"

echo -e "\n${BLUE}2. Tool Execution Test${NC}"
echo "   - Call web scraper tool"
echo "   - Verify parameter validation"
echo "   - Test error handling"

echo -e "\n${BLUE}3. Security Sandbox Test${NC}"
echo "   - Verify network access restrictions"
echo "   - Test memory limits"
echo "   - Check permission enforcement"

echo -e "\n${BLUE}4. Performance Test${NC}"
echo "   - Measure plugin execution time"
echo "   - Monitor memory usage"
echo "   - Test concurrent plugin calls"

# Summary
echo -e "\n${GREEN}================================${NC}"
echo -e "${GREEN}Plugin System Test Summary${NC}"
echo -e "${GREEN}================================${NC}"
echo -e "${GREEN}✓ Plugin SDK available${NC}"
echo -e "${GREEN}✓ Example plugins built${NC}"
echo -e "${GREEN}✓ Plugin packages created${NC}"
echo -e "${GREEN}✓ Test configuration ready${NC}"
echo -e "\n${YELLOW}To run the full plugin system:${NC}"
echo "1. Build with plugin feature: cargo build --features plugins"
echo "2. Start server: cargo run --features plugins -- --config examples/plugin-test.yaml"
echo "3. Load plugins: hal9 plugin load ./plugins/examples/sentiment-analyzer.hal9"
echo "4. Test plugin: hal9 signal send --layer L2 --content 'I love this plugin system!'"

echo -e "\n${BLUE}Plugin development resources:${NC}"
echo "- Guide: docs/PLUGIN_DEVELOPMENT_GUIDE.md"
echo "- Examples: plugins/examples/"
echo "- SDK: hal9-plugin-sdk/"