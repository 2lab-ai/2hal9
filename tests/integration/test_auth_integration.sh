#!/bin/bash
# Integration test for authentication

set -e

echo "🔐 Testing JWT Authentication Integration..."

# Start server with auth enabled
export AUTH_ENABLED=true
export JWT_SECRET="test-secret-key-12345"
export AUTH_DATABASE_PATH="test_auth.db"

# Clean up previous test database
rm -f test_auth.db

# Start server in background
timeout 30s cargo run --bin hal9-server -- &
SERVER_PID=$!

# Wait for server to start
sleep 5

# Test 1: Register a new user
echo "Testing user registration..."
REGISTER_RESPONSE=$(curl -s -X POST http://localhost:8080/api/v1/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "username": "testuser",
    "email": "test@example.com",
    "password": "password123"
  }')

echo "Register response: $REGISTER_RESPONSE"

# Test 2: Login and get tokens
echo "Testing login..."
LOGIN_RESPONSE=$(curl -s -X POST http://localhost:8080/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "username": "testuser",
    "password": "password123"
  }')

echo "Login response: $LOGIN_RESPONSE"

# Extract access token (basic parsing)
ACCESS_TOKEN=$(echo $LOGIN_RESPONSE | grep -o '"access_token":"[^"]*' | cut -d'"' -f4)

if [ -z "$ACCESS_TOKEN" ]; then
  echo "❌ Failed to get access token"
  kill $SERVER_PID 2>/dev/null || true
  exit 1
fi

echo "Got access token: ${ACCESS_TOKEN:0:20}..."

# Test 3: Access protected endpoint with token
echo "Testing protected endpoint..."
PROFILE_RESPONSE=$(curl -s -X GET http://localhost:8080/api/v1/auth/profile \
  -H "Authorization: Bearer $ACCESS_TOKEN")

echo "Profile response: $PROFILE_RESPONSE"

# Test 4: Try to access without token (should fail)
echo "Testing unauthorized access..."
UNAUTH_RESPONSE=$(curl -s -w "\n%{http_code}" -X GET http://localhost:8080/api/v1/auth/profile)
UNAUTH_CODE=$(echo "$UNAUTH_RESPONSE" | tail -n1)

if [ "$UNAUTH_CODE" != "401" ]; then
  echo "❌ Expected 401 unauthorized, got $UNAUTH_CODE"
  kill $SERVER_PID 2>/dev/null || true
  exit 1
fi

echo "✅ Correctly rejected unauthorized request"

# Test 5: Create API key
echo "Testing API key creation..."
API_KEY_RESPONSE=$(curl -s -X POST http://localhost:8080/api/v1/auth/api-keys \
  -H "Authorization: Bearer $ACCESS_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "test-api-key",
    "permissions": {
      "permissions": [
        "ViewNeuron",
        "SendSignal"
      ]
    }
  }')

echo "API key response: $API_KEY_RESPONSE"

# Clean up
kill $SERVER_PID 2>/dev/null || true
rm -f test_auth.db

echo "✅ All authentication tests passed!"