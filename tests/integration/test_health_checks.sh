#!/bin/bash
# Health check integration tests

source tests/integration/setup.sh

echo
echo "🏥 Testing Health Check Endpoints"
echo "================================"
echo

# Start server in background
echo "Starting test server..."
export DATABASE_URL=$TEST_DATABASE_URL
timeout 30 cargo run --bin hal9-server --release 2>&1 > test_server.log &
SERVER_PID=$!

# Wait for server to start
echo "Waiting for server to start..."
sleep 5

# Test simple health check
echo -n "Test 1: Simple health check... "
RESPONSE=$(curl -s -w "\n%{http_code}" http://localhost:3000/health)
HTTP_CODE=$(echo "$RESPONSE" | tail -1)
BODY=$(echo "$RESPONSE" | head -n -1)

if [ "$HTTP_CODE" = "200" ] && echo "$BODY" | grep -q "healthy"; then
    echo "✅ PASSED"
else
    echo "❌ FAILED (HTTP $HTTP_CODE)"
    echo "Response: $BODY"
fi

# Test detailed health check
echo -n "Test 2: Detailed health check... "
RESPONSE=$(curl -s -w "\n%{http_code}" http://localhost:3000/health/detailed)
HTTP_CODE=$(echo "$RESPONSE" | tail -1)
BODY=$(echo "$RESPONSE" | head -n -1)

if [ "$HTTP_CODE" = "200" ] && echo "$BODY" | grep -q "components"; then
    echo "✅ PASSED"
    echo "  Components checked:"
    echo "$BODY" | jq -r '.components[].name' 2>/dev/null | sed 's/^/    - /'
else
    echo "❌ FAILED (HTTP $HTTP_CODE)"
fi

# Test liveness probe
echo -n "Test 3: Kubernetes liveness probe... "
RESPONSE=$(curl -s -w "\n%{http_code}" http://localhost:3000/liveness)
HTTP_CODE=$(echo "$RESPONSE" | tail -1)

if [ "$HTTP_CODE" = "200" ]; then
    echo "✅ PASSED"
else
    echo "❌ FAILED (HTTP $HTTP_CODE)"
fi

# Test readiness probe
echo -n "Test 4: Kubernetes readiness probe... "
RESPONSE=$(curl -s -w "\n%{http_code}" http://localhost:3000/readiness)
HTTP_CODE=$(echo "$RESPONSE" | tail -1)

if [ "$HTTP_CODE" = "200" ] || [ "$HTTP_CODE" = "503" ]; then
    echo "✅ PASSED (HTTP $HTTP_CODE)"
else
    echo "❌ FAILED (HTTP $HTTP_CODE)"
fi

# Clean up
echo
echo "Cleaning up..."
kill $SERVER_PID 2>/dev/null || true
rm -f test_server.log

echo
echo "Health check tests complete!"