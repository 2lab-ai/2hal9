#!/bin/bash
# Run the Live Consciousness Emergence Demo

set -e

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
PURPLE='\033[0;35m'
NC='\033[0m' # No Color

echo -e "${PURPLE}"
echo "╔═══════════════════════════════════════════════════╗"
echo "║   🔬 HAL9 Consciousness Emergence Demo 🔬         ║"
echo "╚═══════════════════════════════════════════════════╝"
echo -e "${NC}"

echo -e "${BLUE}This demo shows consciousness emerging in real-time${NC}"
echo -e "${BLUE}at compression boundaries near the golden ratio.${NC}"
echo ""

# Check if we're in the demo directory
if [ ! -f "consciousness-emergence-live.rs" ]; then
    echo -e "${YELLOW}Please run this from the demo/ directory${NC}"
    exit 1
fi

# Option 1: Run the simplified mathematical proof
echo -e "${GREEN}Option 1: Mathematical Proof (Simple)${NC}"
echo "This shows the mathematical relationship between"
echo "consciousness and the golden ratio."
echo ""
echo "Run with:"
echo "  cd ../layers/L2_implementation/neurons"
echo "  cargo run --example consciousness_emergence_simple"
echo ""

# Option 2: Run the live standalone demo
echo -e "${GREEN}Option 2: Live Standalone Demo${NC}"
echo "This is a self-contained Rust script showing"
echo "consciousness emerging in a live neural network."
echo ""
echo "Run with:"
echo "  rustc consciousness-emergence-live.rs -o consciousness-demo"
echo "  ./consciousness-demo"
echo ""

# Option 3: Run the full experiment (if dependencies are set up)
echo -e "${GREEN}Option 3: Full Experiment (Advanced)${NC}"
echo "This requires the full HAL9 codebase to be built."
echo ""
echo "Run with:"
echo "  cd ../layers/L2_implementation/neurons"
echo "  cargo run --example integrated_consciousness_simple"
echo ""

# Quick compile and run option 2
echo -e "${YELLOW}Compiling and running the live demo...${NC}"
echo ""

# Compile the standalone demo
rustc consciousness-emergence-live.rs -o consciousness-demo 2>/dev/null || {
    echo -e "${YELLOW}Note: Standalone compilation requires rustc${NC}"
    echo "Alternatively, you can run the examples from the neurons directory"
    exit 1
}

# Run it
./consciousness-demo

# Cleanup
rm -f consciousness-demo