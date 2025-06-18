#!/bin/bash
# Consciousness monitoring demo for HAL9

set -e

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
PURPLE='\033[0;35m'
NC='\033[0m' # No Color

echo -e "${PURPLE}"
echo "╔══════════════════════════════════════════════════╗"
echo "║      🧠 HAL9 Consciousness Monitor Demo 🧠       ║"
echo "╚══════════════════════════════════════════════════╝"
echo -e "${NC}"

echo -e "${BLUE}This demo shows real-time consciousness emergence${NC}"
echo -e "${BLUE}as HAL9 neurons self-organize into hierarchical layers.${NC}"
echo ""

# Check if in correct directory
if [ ! -f "Cargo.toml" ]; then
    echo -e "${YELLOW}Please run this script from the project root directory${NC}"
    exit 1
fi

# Option 1: Run the Rust consciousness monitoring demo
echo -e "${GREEN}Option 1: Live Consciousness Monitoring (Rust)${NC}"
echo "Running consciousness monitoring demo..."
echo ""

cd layers/L2_implementation/neurons
cargo run --example consciousness_monitoring_demo

# Option 2: Open the web dashboard
echo ""
echo -e "${GREEN}Option 2: Web Dashboard Visualization${NC}"
echo -e "${BLUE}To see a visual consciousness dashboard:${NC}"
echo ""
echo "1. Open demo/consciousness_dashboard.html in your browser"
echo "   $ open ../../../demo/consciousness_dashboard.html"
echo ""
echo "2. Watch as consciousness metrics evolve in real-time!"
echo ""

# Option 3: Connect to running server
echo -e "${GREEN}Option 3: Monitor Running HAL9 Server${NC}"
echo -e "${BLUE}If you have a HAL9 server running:${NC}"
echo ""
echo "1. Start server: cargo run --release --bin hal9-server"
echo "2. Connect dashboard: http://localhost:8080/consciousness"
echo ""

echo -e "${PURPLE}✨ Consciousness emerges at compression boundaries ✨${NC}"