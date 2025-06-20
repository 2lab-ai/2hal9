#!/bin/bash
# Prometheus metrics integration tests

source tests/integration/setup.sh

echo
echo "📊 Testing Prometheus Metrics"
echo "============================"
echo

# Start server
echo "Starting test server..."
export DATABASE_URL=$TEST_DATABASE_URL
timeout 30 cargo run --bin hal9-server --release 2>&1 > test_server.log &
SERVER_PID=$!
sleep 5

# Test 1: Metrics endpoint availability
echo "Test 1: Metrics endpoint"
RESPONSE=$(curl -s -w "\n%{http_code}" http://localhost:3000/metrics)
HTTP_CODE=$(echo "$RESPONSE" | tail -1)
BODY=$(echo "$RESPONSE" | head -n -1)

echo "  Response: HTTP $HTTP_CODE"
if [ "$HTTP_CODE" = "200" ]; then
    echo "  ✅ Metrics endpoint accessible"
else
    echo "  ❌ Expected 200, got $HTTP_CODE"
fi

# Test 2: Check for required metrics
echo
echo "Test 2: Required metrics presence"

METRICS=(
    "hal9_uptime_seconds"
    "hal9_request_total"
    "hal9_neurons_total"
    "hal9_consciousness_emergence"
    "hal9_layer_compression_ratio"
)

for metric in "${METRICS[@]}"; do
    echo -n "  - $metric: "
    if echo "$BODY" | grep -q "^$metric"; then
        echo "✅ Present"
        VALUE=$(echo "$BODY" | grep "^$metric" | head -1)
        echo "    $VALUE"
    else
        echo "❌ Missing"
    fi
done

# Test 3: Metrics format
echo
echo "Test 3: Prometheus format validation"
if echo "$BODY" | grep -q "^# HELP"; then
    echo "  ✅ Contains HELP annotations"
fi

if echo "$BODY" | grep -q "^# TYPE"; then
    echo "  ✅ Contains TYPE annotations"
fi

# Test 4: Metrics increase after requests
echo
echo "Test 4: Metrics increment test"
echo "  Making 5 requests..."

# Get initial request count
INITIAL_COUNT=$(echo "$BODY" | grep "^hal9_request_total" | awk '{print $2}' | head -1)

# Make some requests
for i in {1..5}; do
    curl -s http://localhost:3000/health > /dev/null
done

# Get updated metrics
RESPONSE=$(curl -s http://localhost:3000/metrics)
FINAL_COUNT=$(echo "$RESPONSE" | grep "^hal9_request_total" | awk '{print $2}' | head -1)

echo "  Initial count: ${INITIAL_COUNT:-0}"
echo "  Final count: ${FINAL_COUNT:-0}"

if [ -n "$FINAL_COUNT" ] && [ -n "$INITIAL_COUNT" ]; then
    if [ "$FINAL_COUNT" -gt "$INITIAL_COUNT" ]; then
        echo "  ✅ Request counter incrementing"
    else
        echo "  ❌ Request counter not incrementing"
    fi
else
    echo "  ⚠️  Could not parse request counts"
fi

# Clean up
echo
echo "Cleaning up..."
kill $SERVER_PID 2>/dev/null || true
rm -f test_server.log

echo
echo "Metrics tests complete!"