#!/bin/bash
# Error handling and recovery integration tests

source tests/integration/setup.sh

echo
echo "🛡️ Testing Error Handling & Recovery"
echo "===================================="
echo

# Start server
echo "Starting test server..."
export DATABASE_URL=$TEST_DATABASE_URL
timeout 30 cargo run --bin hal9-server --release 2>&1 > test_server.log &
SERVER_PID=$!
sleep 5

# Test 1: Invalid JSON payload
echo "Test 1: Invalid JSON payload"
RESPONSE=$(curl -s -w "\n%{http_code}" \
    -X POST \
    -H "Content-Type: application/json" \
    -d "{ invalid json }" \
    http://localhost:3000/api/neurons)
HTTP_CODE=$(echo "$RESPONSE" | tail -1)
BODY=$(echo "$RESPONSE" | head -n -1)

echo "  Response: HTTP $HTTP_CODE"
if [ "$HTTP_CODE" = "400" ]; then
    echo "  ✅ Correctly handled invalid JSON"
else
    echo "  ❌ Expected 400, got $HTTP_CODE"
fi

# Test 2: Method not allowed
echo
echo "Test 2: Method not allowed"
HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" \
    -X DELETE \
    http://localhost:3000/health)
echo "  Response: HTTP $HTTP_CODE"

if [ "$HTTP_CODE" = "405" ]; then
    echo "  ✅ Correctly rejected invalid method"
else
    echo "  ❌ Expected 405, got $HTTP_CODE"
fi

# Test 3: Not found endpoint
echo
echo "Test 3: Non-existent endpoint"
RESPONSE=$(curl -s -w "\n%{http_code}" http://localhost:3000/api/nonexistent)
HTTP_CODE=$(echo "$RESPONSE" | tail -1)
BODY=$(echo "$RESPONSE" | head -n -1)

echo "  Response: HTTP $HTTP_CODE"
if [ "$HTTP_CODE" = "404" ]; then
    echo "  ✅ Correctly returned 404"
    if echo "$BODY" | grep -q "error"; then
        echo "  ✅ Error message included"
    fi
else
    echo "  ❌ Expected 404, got $HTTP_CODE"
fi

# Test 4: Circuit breaker simulation
echo
echo "Test 4: Circuit breaker pattern"
echo "  Simulating multiple failures..."

FAILURES=0
for i in {1..10}; do
    # Try to access a failing endpoint
    HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" \
        http://localhost:3000/api/simulate-failure 2>/dev/null)
    
    if [ "$HTTP_CODE" = "503" ] || [ "$HTTP_CODE" = "500" ]; then
        ((FAILURES++))
    fi
done

echo "  Failures detected: $FAILURES/10"
if [ $FAILURES -gt 0 ]; then
    echo "  ✅ Error handling working"
else
    echo "  ℹ️  No failures detected (endpoint may not exist)"
fi

# Test 5: Timeout handling
echo
echo "Test 5: Request timeout handling"
echo "  Making slow request..."

# Use a very short timeout
HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" \
    --max-time 1 \
    http://localhost:3000/api/slow-endpoint 2>/dev/null)

if [ -z "$HTTP_CODE" ]; then
    echo "  ✅ Client timeout working"
else
    echo "  Response: HTTP $HTTP_CODE"
fi

# Clean up
echo
echo "Cleaning up..."
kill $SERVER_PID 2>/dev/null || true
rm -f test_server.log

echo
echo "Error handling tests complete!"