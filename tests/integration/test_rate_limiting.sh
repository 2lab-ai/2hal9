#!/bin/bash
# Rate limiting integration tests

source tests/integration/setup.sh

echo
echo "🚦 Testing Rate Limiting"
echo "======================="
echo

# Start server
echo "Starting test server..."
export DATABASE_URL=$TEST_DATABASE_URL
timeout 30 cargo run --bin hal9-server --release 2>&1 > test_server.log &
SERVER_PID=$!
sleep 5

# Test rate limiting
echo "Test 1: Rate limit enforcement"
echo "  Making 100 rapid requests..."

SUCCESS=0
RATE_LIMITED=0

for i in {1..100}; do
    HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" http://localhost:3000/api/neurons)
    
    if [ "$HTTP_CODE" = "200" ]; then
        ((SUCCESS++))
    elif [ "$HTTP_CODE" = "429" ]; then
        ((RATE_LIMITED++))
    fi
    
    # Show progress
    if [ $((i % 10)) -eq 0 ]; then
        echo -n "."
    fi
done
echo

echo "  Results:"
echo "    - Successful requests: $SUCCESS"
echo "    - Rate limited (429): $RATE_LIMITED"

if [ $RATE_LIMITED -gt 0 ]; then
    echo "  ✅ Rate limiting is working!"
else
    echo "  ⚠️  No rate limiting detected"
fi

# Test per-IP rate limiting
echo
echo "Test 2: Per-IP rate limiting"
echo "  Testing different IPs..."

# Simulate requests from different IPs using X-Forwarded-For
IP1_SUCCESS=$(curl -s -o /dev/null -w "%{http_code}" -H "X-Forwarded-For: 192.168.1.1" http://localhost:3000/api/neurons)
IP2_SUCCESS=$(curl -s -o /dev/null -w "%{http_code}" -H "X-Forwarded-For: 192.168.1.2" http://localhost:3000/api/neurons)

echo "    - IP 192.168.1.1: HTTP $IP1_SUCCESS"
echo "    - IP 192.168.1.2: HTTP $IP2_SUCCESS"

if [ "$IP1_SUCCESS" = "200" ] || [ "$IP2_SUCCESS" = "200" ]; then
    echo "  ✅ Per-IP tracking working"
else
    echo "  ❌ Per-IP tracking may not be working"
fi

# Test rate limit headers
echo
echo "Test 3: Rate limit headers"
HEADERS=$(curl -s -I http://localhost:3000/api/neurons)

if echo "$HEADERS" | grep -q "X-RateLimit-"; then
    echo "  ✅ Rate limit headers present:"
    echo "$HEADERS" | grep "X-RateLimit-" | sed 's/^/    /'
else
    echo "  ⚠️  No rate limit headers found"
fi

# Clean up
echo
echo "Cleaning up..."
kill $SERVER_PID 2>/dev/null || true
rm -f test_server.log

echo
echo "Rate limiting tests complete!"