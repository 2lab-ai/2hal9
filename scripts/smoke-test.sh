#!/bin/bash
# Smoke test script for HAL9 server

set -e

URL=${1:-http://localhost:8080}
echo "Running smoke tests against: $URL"

# Function to test endpoint
test_endpoint() {
    local endpoint=$1
    local expected_status=$2
    local description=$3
    
    echo -n "Testing $description... "
    
    status=$(curl -s -o /dev/null -w "%{http_code}" "$URL$endpoint")
    
    if [ "$status" = "$expected_status" ]; then
        echo "✓ PASS (HTTP $status)"
        return 0
    else
        echo "✗ FAIL (Expected HTTP $expected_status, got $status)"
        return 1
    fi
}

# Function to test endpoint with auth
test_auth_endpoint() {
    local endpoint=$1
    local expected_status=$2
    local description=$3
    local api_key=$4
    
    echo -n "Testing $description... "
    
    status=$(curl -s -o /dev/null -w "%{http_code}" \
        -H "X-API-Key: $api_key" \
        "$URL$endpoint")
    
    if [ "$status" = "$expected_status" ]; then
        echo "✓ PASS (HTTP $status)"
        return 0
    else
        echo "✗ FAIL (Expected HTTP $expected_status, got $status)"
        return 1
    fi
}

# Track failures
FAILED=0

# Test health endpoints
test_endpoint "/health" "200" "Health check" || ((FAILED++))
test_endpoint "/health/live" "200" "Liveness probe" || ((FAILED++))
test_endpoint "/health/ready" "200" "Readiness probe" || ((FAILED++))

# Test metrics endpoint
test_endpoint "/metrics" "200" "Prometheus metrics" || ((FAILED++))

# Test API endpoints (should require auth)
test_endpoint "/api/v1/neurons" "401" "Neurons API (no auth)" || ((FAILED++))
test_endpoint "/api/v1/layers" "401" "Layers API (no auth)" || ((FAILED++))

# Test with valid API key (if provided)
if [ ! -z "$API_KEY" ]; then
    test_auth_endpoint "/api/v1/neurons" "200" "Neurons API (with auth)" "$API_KEY" || ((FAILED++))
    test_auth_endpoint "/api/v1/layers" "200" "Layers API (with auth)" "$API_KEY" || ((FAILED++))
fi

# Test WebSocket endpoint
echo -n "Testing WebSocket connection... "
if command -v websocat &> /dev/null; then
    if echo "ping" | timeout 5 websocat -n1 "${URL/http/ws}/ws" &> /dev/null; then
        echo "✓ PASS"
    else
        echo "✗ FAIL"
        ((FAILED++))
    fi
else
    echo "⚠ SKIP (websocat not installed)"
fi

# Test rate limiting
echo -n "Testing rate limiting... "
RATE_LIMITED=false
for i in {1..100}; do
    status=$(curl -s -o /dev/null -w "%{http_code}" "$URL/health")
    if [ "$status" = "429" ]; then
        RATE_LIMITED=true
        break
    fi
done

if [ "$RATE_LIMITED" = true ]; then
    echo "✓ PASS (Rate limiting active)"
else
    echo "⚠ WARN (Rate limiting may not be configured)"
fi

# Summary
echo ""
echo "===== SMOKE TEST SUMMARY ====="
if [ $FAILED -eq 0 ]; then
    echo "✓ All tests passed!"
    exit 0
else
    echo "✗ $FAILED tests failed!"
    exit 1
fi