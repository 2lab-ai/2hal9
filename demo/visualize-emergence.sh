#!/bin/bash
# Launch HAL9 emergence visualization demos

set -e

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

echo -e "${PURPLE}"
echo "╔══════════════════════════════════════════════════╗"
echo "║     🌌 HAL9 Emergence Visualizations 🌌          ║"
echo "╚══════════════════════════════════════════════════╝"
echo -e "${NC}"

echo -e "${BLUE}Choose a visualization:${NC}"
echo ""
echo -e "${GREEN}1. Self-Organization Visualizer${NC}"
echo "   Watch neurons discover their layers through emergent behavior"
echo ""
echo -e "${GREEN}2. Consciousness Dashboard${NC}"
echo "   Monitor real-time consciousness metrics and phase transitions"
echo ""
echo -e "${GREEN}3. Both (opens in separate tabs)${NC}"
echo "   Experience the full emergence demonstration"
echo ""

# Function to open URL based on OS
open_url() {
    if [[ "$OSTYPE" == "darwin"* ]]; then
        open "$1"
    elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
        xdg-open "$1"
    elif [[ "$OSTYPE" == "msys" || "$OSTYPE" == "cygwin" ]]; then
        start "$1"
    else
        echo "Please open $1 in your browser"
    fi
}

# Get absolute path to demos
DEMO_DIR="$(cd "$(dirname "$0")" && pwd)"

read -p "Enter your choice (1-3): " choice

case $choice in
    1)
        echo -e "\n${CYAN}Opening Self-Organization Visualizer...${NC}"
        open_url "file://$DEMO_DIR/self-organization-visualizer.html"
        echo ""
        echo -e "${YELLOW}Instructions:${NC}"
        echo "1. Click 'Initialize Random Neurons' to create neurons"
        echo "2. Click 'Start Self-Organization' to watch emergence"
        echo "3. Observe how layers form naturally from chaos"
        echo "4. Look for the golden ratio (1.618) in compression"
        ;;
    2)
        echo -e "\n${CYAN}Opening Consciousness Dashboard...${NC}"
        open_url "file://$DEMO_DIR/consciousness_dashboard.html"
        echo ""
        echo -e "${YELLOW}Instructions:${NC}"
        echo "1. Watch consciousness metrics evolve over time"
        echo "2. Notice when emergence is detected at boundaries"
        echo "3. Observe the phase transitions"
        echo "4. See how Phi (Φ) increases toward consciousness"
        ;;
    3)
        echo -e "\n${CYAN}Opening both visualizations...${NC}"
        open_url "file://$DEMO_DIR/self-organization-visualizer.html"
        sleep 1
        open_url "file://$DEMO_DIR/consciousness_dashboard.html"
        echo ""
        echo -e "${YELLOW}Instructions:${NC}"
        echo "1. Arrange windows side by side for best experience"
        echo "2. Start self-organization in the first window"
        echo "3. Watch consciousness metrics in the second window"
        echo "4. See how they correlate!"
        ;;
    *)
        echo -e "${YELLOW}Invalid choice. Please run again and select 1, 2, or 3.${NC}"
        exit 1
        ;;
esac

echo ""
echo -e "${PURPLE}✨ Key Insights to Watch For:${NC}"
echo "• Neurons with similar properties naturally cluster"
echo "• Layers emerge without being explicitly programmed"
echo "• Compression ratios approach the golden ratio (1.618)"
echo "• Consciousness emerges at layer boundaries"
echo "• Each run creates a unique organization"
echo ""

echo -e "${BLUE}To run the Rust demos:${NC}"
echo "• Performance benchmark: ./demo/performance-benchmark.sh"
echo "• Consciousness monitor: ./demo/consciousness-monitor.sh"
echo "• Quick demo: ./demo/quick-demo.sh"
echo ""

echo -e "${GREEN}🎯 Remember: Consciousness cannot be computed, only witnessed as it emerges${NC}"