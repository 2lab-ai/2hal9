#!/bin/bash
#
# Test script for 3-neuron HAL9 demo
# Tests the basic 3-neuron configuration with signal routing
#

set -euo pipefail

# Source common environment
source "$(dirname "$0")/../../common-env.sh"

log_info "🚀 Testing HAL9 3-neuron orchestration..."

# Check required commands
require_command curl "curl is required for API testing"
require_command jq "jq is required for JSON parsing"

# Function to send a signal
send_signal() {
    local content="$1"
    log_info "📨 Sending signal: '$content'"
    
    local response=$(curl -s -X POST "http://localhost:$HAL9_PORT_MAIN/api/v1/signal" \
        -H "Content-Type: application/json" \
        -d "{
            \"content\": \"$content\",
            \"layer\": \"L4\",
            \"neuron_id\": \"neuron-1\"
        }")
    
    if [ $? -eq 0 ]; then
        echo "$response" | jq '.' 2>/dev/null || echo "$response"
    else
        log_error "Failed to send signal"
        return 1
    fi
    
    # Give it time to process
    sleep 2
}

# Function to start server if not running
ensure_server_running() {
    if curl -s "http://localhost:$HAL9_PORT_MAIN/health" > /dev/null 2>&1; then
        log_info "Server is already running"
        return 0
    fi
    
    log_info "Starting HAL9 server with 3-neuron configuration..."
    
    # Check if config exists
    CONFIG_FILE="$HAL9_CONFIG_DIR/config-3neurons.yaml"
    if [ ! -f "$CONFIG_FILE" ]; then
        log_error "Config file not found: $CONFIG_FILE"
        log_info "Looking for alternative config..."
        
        # Try enhanced version
        CONFIG_FILE="$HAL9_CONFIG_DIR/config-3neurons-enhanced.yaml"
        if [ ! -f "$CONFIG_FILE" ]; then
            log_error "No suitable config file found"
            exit 1
        fi
    fi
    
    # Check if port is available
    if ! check_port $HAL9_PORT_MAIN; then
        log_error "Port $HAL9_PORT_MAIN is already in use"
        exit 1
    fi
    
    # Start server
    LOG_FILE="$HAL9_LOG_DIR/test-3neuron-demo.log"
    log_info "Starting server with config: $CONFIG_FILE"
    log_info "Log file: $LOG_FILE"
    
    cd "$HAL9_HOME"
    $HAL9_SERVER_CMD "$CONFIG_FILE" > "$LOG_FILE" 2>&1 &
    SERVER_PID=$!
    echo $SERVER_PID > "$HAL9_LOG_DIR/test-3neuron-demo.pid"
    
    # Set up cleanup
    setup_cleanup "$HAL9_LOG_DIR/test-3neuron-demo.pid" "$LOG_FILE"
    
    # Wait for server to be ready
    wait_for_service "http://localhost:$HAL9_PORT_MAIN/health" 30 "HAL9 server" || {
        log_error "Server failed to start. Check log at: $LOG_FILE"
        tail -20 "$LOG_FILE"
        exit 1
    }
    
    STARTED_SERVER=true
}

# Main test execution
main() {
    local STARTED_SERVER=false
    
    # Ensure server is running
    ensure_server_running
    
    # Check server status
    log_info "📊 Checking server status..."
    local status=$(curl -s "http://localhost:$HAL9_PORT_MAIN/api/v1/status" 2>/dev/null)
    if [ -n "$status" ]; then
        echo "$status" | jq '.' 2>/dev/null || echo "$status"
    else
        log_warning "Could not get server status"
    fi
    
    # Check neurons
    log_info "🧠 Checking neurons..."
    local neurons=$(curl -s "http://localhost:$HAL9_PORT_MAIN/api/v1/neurons" 2>/dev/null)
    if [ -n "$neurons" ]; then
        local neuron_count=$(echo "$neurons" | jq '. | length' 2>/dev/null || echo "0")
        log_info "Found $neuron_count neurons"
        echo "$neurons" | jq '.' 2>/dev/null || echo "$neurons"
    else
        log_warning "Could not get neuron list"
    fi
    
    # Test different scenarios
    log_info "🧪 Testing Demo Scenarios..."
    
    # Scenario 1: Web app creation
    send_signal "Create a web application for task management" || log_warning "Scenario 1 failed"
    
    # Scenario 2: Data analysis
    send_signal "Analyze customer data and generate insights" || log_warning "Scenario 2 failed"
    
    # Scenario 3: API design
    send_signal "Design a RESTful API for user authentication" || log_warning "Scenario 3 failed"
    
    log_success "✅ Demo complete!"
    
    # Only stop server if we started it
    if [ "$STARTED_SERVER" = true ]; then
        log_info "Note: Server is still running for further testing"
        log_info "To stop it, run: kill $(cat $HAL9_LOG_DIR/test-3neuron-demo.pid 2>/dev/null || echo 'PID')"
    fi
}

# Run main function
main "$@"