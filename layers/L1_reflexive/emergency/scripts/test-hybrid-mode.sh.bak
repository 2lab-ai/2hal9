#!/bin/bash

# Test script for HAL9 Hybrid Mode
# Demonstrates automatic switching between mock and real Claude API

set -e

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${BLUE}=== HAL9 Hybrid Mode Test ===${NC}"
echo ""

# Function to test different modes
test_mode() {
    local mode=$1
    local env=$2
    local config_file=$3
    
    echo -e "${YELLOW}Testing $mode mode (HAL9_ENV=$env)${NC}"
    echo "Configuration: $config_file"
    echo ""
    
    # Set environment
    export HAL9_ENV=$env
    
    # Check if API key is set for API/hybrid modes
    if [[ "$mode" == "api" || "$mode" == "hybrid" || "$mode" == "auto" ]]; then
        if [ -z "$ANTHROPIC_API_KEY" ]; then
            echo -e "${YELLOW}Warning: ANTHROPIC_API_KEY not set. Will fallback to mock.${NC}"
        else
            echo -e "${GREEN}✓ API key detected${NC}"
        fi
    fi
    
    # Build if needed
    if [ ! -f "./target/debug/hal9-server" ]; then
        echo "Building HAL9..."
        cargo build --bin hal9-server --bin hal9
    fi
    
    # Start server in background
    echo "Starting server..."
    ./target/debug/hal9-server "$config_file" > "test-$mode.log" 2>&1 &
    SERVER_PID=$!
    
    # Wait for server to start
    sleep 3
    
    # Check if server is running
    if ! kill -0 $SERVER_PID 2>/dev/null; then
        echo -e "${RED}✗ Server failed to start. Check test-$mode.log${NC}"
        return 1
    fi
    
    # Send test signal
    echo "Sending test signal..."
    ./target/debug/hal9 signal \
        --from user \
        --to neuron-1 \
        --content "Create a simple REST API for user management" \
        --server localhost:8080
    
    # Check status
    echo ""
    echo "Checking status..."
    ./target/debug/hal9 status --server localhost:8080
    
    # Wait a bit to see if it processes
    sleep 2
    
    # Check logs for mode indication
    echo ""
    echo "Server log excerpt:"
    tail -20 "test-$mode.log" | grep -E "(HybridClaude|MockClaude|Creating|mode)" || true
    
    # Stop server
    echo ""
    echo "Stopping server..."
    kill $SERVER_PID 2>/dev/null || true
    wait $SERVER_PID 2>/dev/null || true
    
    echo -e "${GREEN}✓ Test complete${NC}"
    echo ""
    echo "================================"
    echo ""
}

# Test 1: Mock mode (always uses mock)
test_mode "mock" "development" "examples/config-3neurons.yaml"

# Test 2: Auto mode in development (should use mock)
test_mode "auto-dev" "development" "examples/config-auto-mode.yaml"

# Test 3: Auto mode in production (should try API, fallback to mock if no key)
test_mode "auto-prod" "production" "examples/config-auto-mode.yaml"

# Test 4: Hybrid mode (uses API if available and under limits)
test_mode "hybrid" "production" "examples/config-hybrid-mode.yaml"

echo -e "${GREEN}=== All tests complete ===${NC}"
echo ""
echo "Log files created:"
ls -la test-*.log
echo ""
echo "To test with real API:"
echo "  export ANTHROPIC_API_KEY='your-key-here'"
echo "  ./test-hybrid-mode.sh"