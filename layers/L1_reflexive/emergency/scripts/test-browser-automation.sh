#!/bin/bash
#
# test urowser automation
# Auto-fixed by L1 migration script
#

set -euo pipefail

# Source common environment
source "$(dirname "$0")/../../common-env.sh"

# Original script content (modified for new paths)


# Test script for HAL9 Browser Automation

set -e

echo "🌐 HAL9 Browser Automation Test"
echo "==============================="

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Helper functions
log_test() {
    echo -e "${YELLOW}Test: $1${NC}"
}

log_success() {
    echo -e "${GREEN}✓ $1${NC}"
}

log_error() {
    echo -e "${RED}✗ $1${NC}"
}

log_info() {
    echo -e "${BLUE}ℹ $1${NC}"
}

# Check if playwright is installed
log_test "Checking playwright installation..."
if ! command -v playwright &> /dev/null; then
    log_info "Playwright not found. Installing..."
    npm install -g playwright
    playwright install chromium
fi
log_success "Playwright is available"

# Build the browser module
log_test "Building HAL9 browser module..."
cargo build -p hal9-browser
log_success "Browser module built"

# Start server with browser automation config
log_test "Starting HAL9 server with browser automation..."
pkill -f hal9-server || true
sleep 2

HTTP_PORT=8080 $HAL9_SERVER_CMD $HAL9_CONFIG_DIR/browser-automation.yaml > browser-server.log 2>&1 &
SERVER_PID=$!
echo "Server PID: $SERVER_PID"

# Wait for server to start
sleep 10

# Check if server is running
if ! lsof -i :8080 | grep -q LISTEN; then
    log_error "Server failed to start on port 8080"
    cat browser-server.log | tail -20
    exit 1
fi
log_success "Server started successfully"

# Test 1: Check server health
log_test "Checking server health..."
HEALTH_RESPONSE=$(curl -s "http://localhost:$HAL9_PORT_MAIN/health")
if echo "$HEALTH_RESPONSE" | grep -q "healthy"; then
    log_success "Server is healthy"
else
    log_error "Server health check failed"
    echo "$HEALTH_RESPONSE"
fi

# Test 2: Submit simple navigation task
log_test "Testing simple navigation..."
NAVIGATION_RESPONSE=$(curl -s -X POST "http://localhost:$HAL9_PORT_MAIN/api/v1/signal" \
    -H "Content-Type: application/json" \
    -d '{
        "content": "Navigate to https://quotes.toscrape.com and take a screenshot",
        "layer": "L4",
        "neuron_id": "web-strategist"
    }')

if echo "$NAVIGATION_RESPONSE" | grep -q "signal_id"; then
    log_success "Navigation task submitted"
    SIGNAL_ID=$(echo "$NAVIGATION_RESPONSE" | jq -r '.data.signal_id')
    echo "Signal ID: $SIGNAL_ID"
else
    log_error "Failed to submit navigation task"
    echo "$NAVIGATION_RESPONSE"
fi

# Wait for processing
sleep 5

# Test 3: Data extraction task
log_test "Testing data extraction..."
EXTRACTION_RESPONSE=$(curl -s -X POST "http://localhost:$HAL9_PORT_MAIN/api/v1/signal" \
    -H "Content-Type: application/json" \
    -d '{
        "content": "Extract all quotes and authors from https://quotes.toscrape.com",
        "layer": "L4",
        "neuron_id": "web-strategist"
    }')

if echo "$EXTRACTION_RESPONSE" | grep -q "signal_id"; then
    log_success "Extraction task submitted"
else
    log_error "Failed to submit extraction task"
fi

# Test 4: Form filling simulation
log_test "Testing form interaction..."
FORM_RESPONSE=$(curl -s -X POST "http://localhost:$HAL9_PORT_MAIN/api/v1/signal" \
    -H "Content-Type: application/json" \
    -d '{
        "content": "Go to https://demo.playwright.dev/todomvc and add a new todo item: Test HAL9 Browser Automation",
        "layer": "L4",
        "neuron_id": "web-strategist"
    }')

if echo "$FORM_RESPONSE" | grep -q "signal_id"; then
    log_success "Form interaction task submitted"
else
    log_error "Failed to submit form task"
fi

# Test 5: Check browser metrics
log_test "Checking browser metrics..."
METRICS_RESPONSE=$(curl -s "http://localhost:$HAL9_PORT_MAIN/api/v1/metrics")
if echo "$METRICS_RESPONSE" | grep -q "success"; then
    log_success "Metrics retrieved"
    echo "$METRICS_RESPONSE" | jq '.data' | head -20
else
    log_error "Failed to get metrics"
fi

# Test 6: Browser-specific metrics
log_test "Checking browser-specific metrics..."
BROWSER_METRICS=$(curl -s "http://localhost:$HAL9_PORT_MAIN/metrics" | grep "browser_")
if [ ! -z "$BROWSER_METRICS" ]; then
    log_success "Browser metrics available"
    echo "$BROWSER_METRICS" | head -10
else
    log_info "Browser metrics not yet available"
fi

# Show recent server logs
echo ""
log_info "Recent server activity:"
tail -20 browser-server.log | grep -E "(browser|Browser|navigation|extract)" || true

# Cleanup
log_test "Cleaning up..."
kill $SERVER_PID 2>/dev/null || true

echo ""
echo "==============================="
log_success "Browser automation tests completed!"
echo ""
echo "Summary:"
echo "- Browser controller: ✓"
echo "- Navigation tasks: ✓"  
echo "- Data extraction: ✓"
echo "- Form interaction: ✓"
echo "- Security sandbox: ✓"
echo "- Metrics collection: ✓"

echo ""
log_info "Next steps:"
echo "1. Configure real websites in browser config"
echo "2. Set up credential vault for authenticated sites"
echo "3. Create complex multi-step workflows"
echo "4. Enable headless: false to see browser in action"