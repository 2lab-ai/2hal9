#!/bin/bash

# HAL9 Demo Scenarios Script
# Demonstrates hierarchical AI orchestration through various use cases

set -e

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Configuration
SERVER=${HAL9_SERVER:-"localhost:8080"}
HAL9_BIN="./target/debug/hal9"

# Function to print colored headers
print_header() {
    echo -e "\n${BLUE}===========================================${NC}"
    echo -e "${GREEN}$1${NC}"
    echo -e "${BLUE}===========================================${NC}\n"
}

# Function to print scenario info
print_scenario() {
    echo -e "${YELLOW}Scenario $1:${NC} $2"
    echo -e "${BLUE}-------------------------------------------${NC}"
}

# Function to wait with countdown
wait_countdown() {
    local seconds=$1
    local message=$2
    echo -n "$message"
    for ((i=$seconds; i>0; i--)); do
        echo -n " $i"
        sleep 1
    done
    echo ""
}

# Check if server is running
check_server() {
    echo "Checking server status..."
    if $HAL9_BIN status --server $SERVER >/dev/null 2>&1; then
        echo -e "${GREEN}✓ Server is running${NC}"
        return 0
    else
        echo -e "${RED}✗ Server is not running${NC}"
        echo "Please start the server with: ./run-3neuron-demo.sh"
        exit 1
    fi
}

# Build CLI if needed
build_cli() {
    if [ ! -f "$HAL9_BIN" ]; then
        echo "Building HAL9 CLI..."
        cargo build --bin hal9
    fi
}

# Main demo execution
main() {
    print_header "HAL9 Hierarchical AI Orchestration Demo"
    
    # Build and check prerequisites
    build_cli
    check_server
    
    # Show initial status
    print_header "Initial Server Status"
    $HAL9_BIN status --server $SERVER
    
    wait_countdown 3 "Starting demo scenarios in"
    
    # Scenario 1: Web Application Development
    print_scenario "1" "Web Application Development"
    echo "Task: Create a modern web application for task management"
    echo ""
    $HAL9_BIN signal \
        --from user \
        --to neuron-1 \
        --content "Create a modern web application for task management with React frontend and FastAPI backend" \
        --server $SERVER
    
    wait_countdown 5 "Processing"
    
    # Scenario 2: Data Analysis Pipeline
    print_scenario "2" "Data Analysis Pipeline"
    echo "Task: Design a data analysis system for e-commerce insights"
    echo ""
    $HAL9_BIN signal \
        --from user \
        --to neuron-1 \
        --content "Design and implement a data analysis pipeline for e-commerce customer behavior insights" \
        --server $SERVER
    
    wait_countdown 5 "Processing"
    
    # Scenario 3: API Design
    print_scenario "3" "RESTful API Design"
    echo "Task: Create a secure authentication API"
    echo ""
    $HAL9_BIN signal \
        --from user \
        --to neuron-1 \
        --content "Design a RESTful API for user authentication with JWT tokens and role-based access control" \
        --server $SERVER
    
    wait_countdown 5 "Processing"
    
    # Scenario 4: Machine Learning Pipeline
    print_scenario "4" "Machine Learning Pipeline"
    echo "Task: Build a recommendation system"
    echo ""
    $HAL9_BIN signal \
        --from user \
        --to neuron-1 \
        --content "Create a machine learning pipeline for product recommendation system using collaborative filtering" \
        --server $SERVER
    
    wait_countdown 5 "Processing"
    
    # Scenario 5: Infrastructure Automation
    print_scenario "5" "Infrastructure Automation"
    echo "Task: Set up CI/CD pipeline"
    echo ""
    $HAL9_BIN signal \
        --from user \
        --to neuron-1 \
        --content "Set up a complete CI/CD pipeline with automated testing, Docker containerization, and Kubernetes deployment" \
        --server $SERVER
    
    wait_countdown 5 "Processing"
    
    # Show final status
    print_header "Final Server Status"
    $HAL9_BIN status --server $SERVER
    
    print_header "Demo Complete!"
    echo -e "${GREEN}Successfully demonstrated HAL9's hierarchical AI orchestration capabilities:${NC}"
    echo "• L4 (Strategic): High-level planning and task decomposition"
    echo "• L3 (Design): System architecture and technical design"
    echo "• L2 (Implementation): Concrete code generation and execution"
    echo ""
    echo -e "${BLUE}Each request flowed through the hierarchy:${NC}"
    echo "User → L4 (Strategic Planning) → L3 (System Design) → L2 (Implementation)"
    echo ""
    echo -e "${YELLOW}Check the server logs to see the detailed signal flow!${NC}"
}

# Run the demo
main "$@"