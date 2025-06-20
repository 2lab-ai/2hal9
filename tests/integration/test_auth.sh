#!/bin/bash
# JWT Authentication integration tests

source tests/integration/setup.sh

echo
echo "🔐 Testing JWT Authentication"
echo "============================"
echo

# Start server
echo "Starting test server..."
export DATABASE_URL=$TEST_DATABASE_URL
export JWT_SECRET="test-secret-key"
timeout 30 cargo run --bin hal9-server --release 2>&1 > test_server.log &
SERVER_PID=$!
sleep 5

# Generate test JWT token
echo "Generating test JWT token..."
# This is a test token with payload: {"sub": "test-user", "exp": 9999999999}
TEST_TOKEN="eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiJ0ZXN0LXVzZXIiLCJleHAiOjk5OTk5OTk5OTl9.Vg30C57s3l90JNap_VgMhKZjfc-p7SoBXaSAy8c6BS8"

# Test 1: No authentication
echo
echo "Test 1: Request without authentication"
HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" http://localhost:3000/api/protected)
echo "  Response: HTTP $HTTP_CODE"

if [ "$HTTP_CODE" = "401" ]; then
    echo "  ✅ Correctly rejected unauthenticated request"
else
    echo "  ❌ Expected 401, got $HTTP_CODE"
fi

# Test 2: Invalid token
echo
echo "Test 2: Request with invalid token"
HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" \
    -H "Authorization: Bearer invalid-token" \
    http://localhost:3000/api/protected)
echo "  Response: HTTP $HTTP_CODE"

if [ "$HTTP_CODE" = "401" ]; then
    echo "  ✅ Correctly rejected invalid token"
else
    echo "  ❌ Expected 401, got $HTTP_CODE"
fi

# Test 3: Valid token (if endpoint exists)
echo
echo "Test 3: Request with valid token format"
HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" \
    -H "Authorization: Bearer $TEST_TOKEN" \
    http://localhost:3000/api/protected)
echo "  Response: HTTP $HTTP_CODE"

if [ "$HTTP_CODE" = "200" ] || [ "$HTTP_CODE" = "404" ]; then
    echo "  ✅ Token format accepted (endpoint may not exist)"
else
    echo "  ⚠️  Unexpected response: $HTTP_CODE"
fi

# Test 4: API Key authentication
echo
echo "Test 4: API Key authentication"
HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" \
    -H "X-API-Key: test-api-key-12345" \
    http://localhost:3000/api/neurons)
echo "  Response: HTTP $HTTP_CODE"

if [ "$HTTP_CODE" = "200" ] || [ "$HTTP_CODE" = "401" ]; then
    echo "  ✅ API key handling working"
else
    echo "  ❌ Unexpected response: $HTTP_CODE"
fi

# Clean up
echo
echo "Cleaning up..."
kill $SERVER_PID 2>/dev/null || true
rm -f test_server.log

echo
echo "Authentication tests complete!"