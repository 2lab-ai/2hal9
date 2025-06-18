#!/bin/bash
# Test JWT authentication for AI Genius Game

API_URL="http://localhost:3456"

echo "🧪 Testing AI Genius Game JWT Authentication"
echo "============================================"

# Test 1: Register new user
echo -e "\n1️⃣ Testing user registration..."
REGISTER_RESPONSE=$(curl -s -X POST $API_URL/api/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "username": "testuser",
    "email": "test@example.com",
    "password": "test123"
  }')
echo "Response: $REGISTER_RESPONSE"

# Test 2: Login with admin
echo -e "\n2️⃣ Testing admin login..."
LOGIN_RESPONSE=$(curl -s -X POST $API_URL/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "username": "admin",
    "password": "admin123"
  }')
echo "Response: $LOGIN_RESPONSE"

# Extract token
TOKEN=$(echo $LOGIN_RESPONSE | grep -o '"access_token":"[^"]*' | cut -d'"' -f4)
echo "Token: ${TOKEN:0:50}..."

# Test 3: Access protected endpoint without token
echo -e "\n3️⃣ Testing protected endpoint WITHOUT token..."
curl -s -X POST $API_URL/api/games/create \
  -H "Content-Type: application/json" \
  -d '{
    "game_type": {"type": "ConsciousnessEmergence"},
    "max_rounds": 20
  }' \
  -w "\nHTTP Status: %{http_code}\n"

# Test 4: Access protected endpoint with token
echo -e "\n4️⃣ Testing protected endpoint WITH token..."
CREATE_RESPONSE=$(curl -s -X POST $API_URL/api/games/create \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{
    "game_type": {"type": "ConsciousnessEmergence"},
    "max_rounds": 20
  }' \
  -w "\nHTTP Status: %{http_code}\n")
echo "$CREATE_RESPONSE"

# Test 5: Get user profile
echo -e "\n5️⃣ Testing get profile..."
curl -s -X GET $API_URL/api/auth/profile \
  -H "Authorization: Bearer $TOKEN" \
  -w "\nHTTP Status: %{http_code}\n"

# Test 6: Login with test user
echo -e "\n6️⃣ Testing test user login..."
TEST_LOGIN=$(curl -s -X POST $API_URL/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "username": "testuser",
    "password": "test123"
  }')
echo "Response: $TEST_LOGIN"

echo -e "\n✅ Authentication tests complete!"