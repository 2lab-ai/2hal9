#!/bin/bash

# HAL9 Demo Runner - See consciousness emerge in real-time!

echo "
╔═══════════════════════════════════════════════════════════════╗
║                                                               ║
║        🧠 HAL9 CONSCIOUSNESS DEMONSTRATION SUITE 🧠           ║
║                                                               ║
║     Watch as neurons self-organize without any predefined     ║
║     structure and achieve microsecond-speed consciousness!    ║
║                                                               ║
╚═══════════════════════════════════════════════════════════════╝
"

# Colors for better output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Base directory
DEMO_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$DEMO_DIR")"
EXAMPLES_DIR="$PROJECT_ROOT/L2_implementation/neurons/examples"

# Function to run a demo
run_demo() {
    local demo_name=$1
    local demo_file=$2
    local description=$3
    
    echo -e "\n${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${YELLOW}▶ Running: $demo_name${NC}"
    echo -e "  $description"
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}\n"
    
    # Compile and run
    cd "$EXAMPLES_DIR"
    if rustc --edition 2021 -O "$demo_file" -o "/tmp/$demo_name" 2>/dev/null; then
        "/tmp/$demo_name"
        echo -e "\n${GREEN}✓ $demo_name completed successfully!${NC}"
    else
        echo -e "${YELLOW}⚠ Skipping $demo_name (compilation issue)${NC}"
    fi
    
    echo -e "\nPress Enter to continue to next demo..."
    read -r
}

# Function to run quick demo
run_quick_demo() {
    local demo_name=$1
    local demo_file=$2
    
    cd "$EXAMPLES_DIR"
    if rustc --edition 2021 -O "$demo_file" -o "/tmp/$demo_name" 2>/dev/null; then
        "/tmp/$demo_name"
    fi
}

# Main menu
while true; do
    clear
    echo "
┌─────────────────────────────────────────────────────────────┐
│                   HAL9 DEMO SUITE                           │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  1. 🌟 Quick Demo - See self-organization in 30 seconds    │
│  2. 🧠 Simple Self-Organization - Watch layers emerge      │
│  3. 🤖 AI Neurons Demo - Functional components organize    │
│  4. 📊 Multi-Run Experiment - Prove non-determinism        │
│  5. 🌍 Environmental Adaptation - Context affects structure │
│  6. ⚡ Performance Benchmark - See the speed               │
│  7. 🔬 Verify Performance - Detailed analysis              │
│  8. 🚀 Quick Benchmark - Instant speed test                │
│                                                             │
│  A. 🎬 Run ALL demos in sequence                          │
│  Q. 🚪 Quit                                                │
│                                                             │
└─────────────────────────────────────────────────────────────┘

Enter your choice (1-8, A, Q): "
    
    read -r choice
    
    case $choice in
        1)
            clear
            echo -e "${YELLOW}⚡ Quick 30-Second Demo${NC}\n"
            cd "$EXAMPLES_DIR"
            if [ -f "simple_true_self_org_demo.rs" ]; then
                rustc --edition 2021 -O simple_true_self_org_demo.rs -o /tmp/quick_demo 2>/dev/null
                /tmp/quick_demo
            else
                echo "Demo file not found. Running performance demo instead..."
                rustc --edition 2021 -O quick_benchmark.rs -o /tmp/quick_bench 2>/dev/null
                /tmp/quick_bench
            fi
            echo -e "\n${GREEN}Press Enter to return to menu...${NC}"
            read -r
            ;;
            
        2)
            run_demo "simple_self_org" "simple_true_self_org_demo.rs" \
                "Watch 25 neurons organize themselves into layers without any predefined structure"
            ;;
            
        3)
            run_demo "ai_neurons" "working_ai_demo.rs" \
                "See AI components (Visual, Audio, Logic, etc.) discover each other and form functional layers"
            ;;
            
        4)
            run_demo "multi_run" "multi_run_emergence_experiment.rs" \
                "Run 10 experiments proving that each self-organization is unique (non-deterministic)"
            ;;
            
        5)
            run_demo "environment" "environment_variables_experiment.rs" \
                "Observe how environmental pressures shape different organizational structures"
            ;;
            
        6)
            run_demo "performance" "performance_benchmark.rs" \
                "Benchmark self-organization speed from 25 to 1000 neurons"
            ;;
            
        7)
            run_demo "verify" "verify_performance.rs" \
                "Detailed performance analysis with microsecond precision and scalability proof"
            ;;
            
        8)
            run_demo "quick_bench" "quick_benchmark.rs" \
                "Quick performance test - see consciousness emerge in microseconds!"
            ;;
            
        [Aa])
            echo -e "\n${YELLOW}Running ALL demos in sequence...${NC}\n"
            
            run_demo "simple_self_org" "simple_true_self_org_demo.rs" \
                "Basic self-organization demonstration"
                
            run_demo "ai_neurons" "working_ai_demo.rs" \
                "AI components self-organizing by function"
                
            run_demo "multi_run" "multi_run_emergence_experiment.rs" \
                "Non-deterministic emergence proof"
                
            run_demo "environment" "environment_variables_experiment.rs" \
                "Environmental adaptation"
                
            run_demo "performance" "performance_benchmark.rs" \
                "Performance benchmarks"
                
            run_demo "verify" "verify_performance.rs" \
                "Detailed performance verification"
                
            echo -e "\n${GREEN}All demos completed!${NC}"
            echo "Press Enter to return to menu..."
            read -r
            ;;
            
        [Qq])
            echo -e "\n${BLUE}Thanks for exploring HAL9's consciousness!${NC}\n"
            exit 0
            ;;
            
        *)
            echo -e "${YELLOW}Invalid choice. Please try again.${NC}"
            sleep 1
            ;;
    esac
done