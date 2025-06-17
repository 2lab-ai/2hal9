#!/bin/bash

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🔐 HAL9 Authentication System Test${NC}"
echo "======================================"
echo ""

# Base URL
BASE_URL="http://localhost:8080"

# Test data
TEST_USER="testuser_$(date +%s)"
TEST_EMAIL="${TEST_USER}@example.com"
TEST_PASSWORD="SecurePass123!"

# Function to pretty print JSON
pretty_json() {
    echo "$1" | jq . 2>/dev/null || echo "$1"
}

# Function to extract value from JSON
extract_json() {
    echo "$1" | jq -r "$2" 2>/dev/null || echo "ERROR"
}

# Test 1: Register new user
echo -e "${YELLOW}👤 Test 1: Register new user${NC}"
REGISTER_RESPONSE=$(curl -s -X POST ${BASE_URL}/api/v1/auth/register \
  -H "Content-Type: application/json" \
  -d "{
    \"username\": \"${TEST_USER}\",
    \"email\": \"${TEST_EMAIL}\",
    \"password\": \"${TEST_PASSWORD}\",
    \"role\": \"user\"
  }")

if [[ $REGISTER_RESPONSE == *"username"* ]]; then
    echo -e "${GREEN}✅ Registration successful${NC}"
    pretty_json "$REGISTER_RESPONSE"
else
    echo -e "${RED}❌ Registration failed${NC}"
    echo "Response: $REGISTER_RESPONSE"
    echo -e "\n${YELLOW}⚠️  Authentication may not be enabled. Follow AUTHENTICATION_SETUP.md to enable it.${NC}"
    exit 1
fi

echo ""

# Test 2: Login
echo -e "${YELLOW}🔓 Test 2: Login${NC}"
LOGIN_RESPONSE=$(curl -s -X POST ${BASE_URL}/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d "{
    \"username\": \"${TEST_USER}\",
    \"password\": \"${TEST_PASSWORD}\"
  }")

ACCESS_TOKEN=$(extract_json "$LOGIN_RESPONSE" ".tokens.access_token")
REFRESH_TOKEN=$(extract_json "$LOGIN_RESPONSE" ".tokens.refresh_token")

if [[ $ACCESS_TOKEN != "ERROR" ]] && [[ $ACCESS_TOKEN != "null" ]]; then
    echo -e "${GREEN}✅ Login successful${NC}"
    echo "Access Token: ${ACCESS_TOKEN:0:20}..."
    echo "Refresh Token: ${REFRESH_TOKEN:0:20}..."
else
    echo -e "${RED}❌ Login failed${NC}"
    pretty_json "$LOGIN_RESPONSE"
    exit 1
fi

echo ""

# Test 3: Get profile (protected endpoint)
echo -e "${YELLOW}👤 Test 3: Get user profile${NC}"
PROFILE_RESPONSE=$(curl -s -X GET ${BASE_URL}/api/v1/auth/profile \
  -H "Authorization: Bearer ${ACCESS_TOKEN}")

if [[ $PROFILE_RESPONSE == *"username"* ]]; then
    echo -e "${GREEN}✅ Profile retrieved successfully${NC}"
    pretty_json "$PROFILE_RESPONSE"
else
    echo -e "${RED}❌ Failed to get profile${NC}"
    echo "Response: $PROFILE_RESPONSE"
fi

echo ""

# Test 4: Create API key
echo -e "${YELLOW}🔑 Test 4: Create API key${NC}"
API_KEY_RESPONSE=$(curl -s -X POST ${BASE_URL}/api/v1/auth/api-keys \
  -H "Authorization: Bearer ${ACCESS_TOKEN}" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Test API Key",
    "expires_in_days": 30
  }')

API_KEY=$(extract_json "$API_KEY_RESPONSE" ".key")

if [[ $API_KEY != "ERROR" ]] && [[ $API_KEY != "null" ]]; then
    echo -e "${GREEN}✅ API key created successfully${NC}"
    echo "API Key: ${API_KEY}"
else
    echo -e "${RED}❌ Failed to create API key${NC}"
    pretty_json "$API_KEY_RESPONSE"
fi

echo ""

# Test 5: Use API key
if [[ $API_KEY != "ERROR" ]] && [[ $API_KEY != "null" ]]; then
    echo -e "${YELLOW}🔑 Test 5: Use API key${NC}"
    API_KEY_TEST=$(curl -s -X GET ${BASE_URL}/api/v1/status \
      -H "X-API-Key: ${API_KEY}")
    
    if [[ $API_KEY_TEST == *"running"* ]]; then
        echo -e "${GREEN}✅ API key authentication successful${NC}"
    else
        echo -e "${RED}❌ API key authentication failed${NC}"
        echo "Response: $API_KEY_TEST"
    fi
fi

echo ""

# Test 6: Refresh token
echo -e "${YELLOW}🔄 Test 6: Refresh token${NC}"
REFRESH_RESPONSE=$(curl -s -X POST ${BASE_URL}/api/v1/auth/refresh \
  -H "Content-Type: application/json" \
  -d "{
    \"refresh_token\": \"${REFRESH_TOKEN}\"
  }")

NEW_ACCESS_TOKEN=$(extract_json "$REFRESH_RESPONSE" ".access_token")

if [[ $NEW_ACCESS_TOKEN != "ERROR" ]] && [[ $NEW_ACCESS_TOKEN != "null" ]]; then
    echo -e "${GREEN}✅ Token refresh successful${NC}"
    echo "New Access Token: ${NEW_ACCESS_TOKEN:0:20}..."
else
    echo -e "${RED}❌ Token refresh failed${NC}"
    pretty_json "$REFRESH_RESPONSE"
fi

echo ""
echo -e "${BLUE}Summary:${NC}"
echo "========"
if [[ $ACCESS_TOKEN != "ERROR" ]]; then
    echo -e "${GREEN}✅ Authentication system is working properly${NC}"
    echo ""
    echo "Next steps:"
    echo "1. Enable authentication in production with strong JWT_SECRET"
    echo "2. Configure SSL/TLS for secure communication"
    echo "3. Set up proper user management"
else
    echo -e "${YELLOW}⚠️  Authentication is not enabled${NC}"
    echo ""
    echo "To enable authentication:"
    echo "1. Follow the guide in docs/AUTHENTICATION_SETUP.md"
    echo "2. Add JWT_SECRET to your environment"
    echo "3. Restart the services"
fi