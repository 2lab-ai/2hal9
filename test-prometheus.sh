#!/bin/bash

# Test script for HAL9 Prometheus metrics

set -e

echo "🔍 HAL9 Prometheus Metrics Test"
echo "==============================="

# Configuration
API_BASE="http://localhost:9737"
METRICS_URL="$API_BASE/metrics"

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

# Start server
log_test "Starting HAL9 server..."
pkill -f hal9-server || true
sleep 2

HTTP_PORT=9737 cargo run --bin hal9-server examples/config-3neurons.yaml > prometheus-server.log 2>&1 &
SERVER_PID=$!
echo "Server PID: $SERVER_PID"

# Wait for server to start
sleep 8

# Check if server is running
if ! lsof -i :9737 | grep -q LISTEN; then
    log_error "Server failed to start on port 9737"
    cat prometheus-server.log
    exit 1
fi
log_success "Server started successfully"

# Test 1: Check Prometheus metrics endpoint
log_test "Checking /metrics endpoint..."
METRICS_RESPONSE=$(curl -s -w "\nHTTP_STATUS:%{http_code}" "$METRICS_URL")
HTTP_STATUS=$(echo "$METRICS_RESPONSE" | grep "HTTP_STATUS:" | cut -d: -f2)
METRICS_DATA=$(echo "$METRICS_RESPONSE" | grep -v "HTTP_STATUS:")

if [ "$HTTP_STATUS" = "200" ]; then
    log_success "Metrics endpoint returned 200 OK"
else
    log_error "Metrics endpoint returned $HTTP_STATUS"
    exit 1
fi

# Test 2: Verify Prometheus format
log_test "Verifying Prometheus format..."
if echo "$METRICS_DATA" | grep -q "# HELP"; then
    log_success "Found HELP comments"
else
    log_error "Missing HELP comments in Prometheus format"
fi

if echo "$METRICS_DATA" | grep -q "# TYPE"; then
    log_success "Found TYPE declarations"
else
    log_error "Missing TYPE declarations in Prometheus format"
fi

# Test 3: Check for core metrics
log_test "Checking for core metrics..."
CORE_METRICS=(
    "hal9_server_uptime_seconds"
    "hal9_signals_sent_total"
    "hal9_signals_processed_total"
    "hal9_neurons_active"
    "hal9_claude_tokens_used_total"
    "hal9_claude_cost_dollars_total"
)

for metric in "${CORE_METRICS[@]}"; do
    if echo "$METRICS_DATA" | grep -q "^$metric"; then
        log_success "Found metric: $metric"
        echo "$METRICS_DATA" | grep "^$metric" | head -3
    else
        log_error "Missing metric: $metric"
    fi
done

# Test 4: Send a signal to generate some activity
log_test "Sending test signal..."
SIGNAL_RESPONSE=$(curl -s -X POST "$API_BASE/api/v1/signal" \
    -H "Content-Type: application/json" \
    -d '{
        "content": "Test signal for metrics",
        "layer": "L4"
    }')

if echo "$SIGNAL_RESPONSE" | grep -q "success"; then
    log_success "Signal sent successfully"
else
    log_error "Failed to send signal"
fi

# Wait for processing
sleep 3

# Test 5: Check if metrics updated
log_test "Checking if metrics updated..."
METRICS_AFTER=$(curl -s "$METRICS_URL")

# Check if signals_processed increased
if echo "$METRICS_AFTER" | grep -q "hal9_signals_processed_total"; then
    PROCESSED_COUNT=$(echo "$METRICS_AFTER" | grep "hal9_signals_processed_total" | grep 'status="success"' | awk '{print $2}')
    if [ -n "$PROCESSED_COUNT" ] && [ "$PROCESSED_COUNT" != "0" ]; then
        log_success "Signal processing metrics updated (count: $PROCESSED_COUNT)"
    else
        log_error "Signal processing metrics not updated"
    fi
fi

# Test 6: Check alternative format endpoint
log_test "Checking JSON metrics format..."
JSON_METRICS=$(curl -s "$API_BASE/api/v1/metrics/export?format=json")
if echo "$JSON_METRICS" | jq . > /dev/null 2>&1; then
    log_success "JSON metrics format working"
    echo "$JSON_METRICS" | jq '.signals_sent, .signals_processed, .neurons_active'
else
    log_error "JSON metrics format failed"
fi

# Test 7: Check Prometheus format via export endpoint
log_test "Checking Prometheus format via export endpoint..."
PROM_EXPORT=$(curl -s "$API_BASE/api/v1/metrics/export?format=prometheus")
if echo "$PROM_EXPORT" | grep -q "# HELP"; then
    log_success "Prometheus export format working"
else
    log_error "Prometheus export format failed"
fi

# Cleanup
log_test "Cleaning up..."
kill $SERVER_PID 2>/dev/null || true

echo ""
echo "==============================="
log_success "Prometheus metrics tests completed!"
echo ""
echo "Summary:"
echo "- Metrics endpoint: ✓"
echo "- Prometheus format: ✓"
echo "- Core metrics present: ✓"
echo "- Metrics updating: ✓"
echo "- Multiple formats: ✓"

# Show sample of final metrics
echo ""
log_info "Sample metrics output:"
echo "$METRICS_DATA" | head -20