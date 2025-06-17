#!/bin/bash
# HAL9 API Test Script

set -e

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Base URL
BASE_URL="http://localhost:8080"

echo "🧪 Testing HAL9 API Endpoints"
echo "============================"

# Function to test endpoint
test_endpoint() {
    local method=$1
    local endpoint=$2
    local description=$3
    local data=$4
    
    echo -e "\n${YELLOW}Testing: $description${NC}"
    echo "Method: $method"
    echo "Endpoint: $endpoint"
    
    if [ -z "$data" ]; then
        response=$(curl -s -w "\n%{http_code}" -X $method "$BASE_URL$endpoint" -H "Content-Type: application/json")
    else
        response=$(curl -s -w "\n%{http_code}" -X $method "$BASE_URL$endpoint" -H "Content-Type: application/json" -d "$data")
    fi
    
    # Extract status code and body
    http_code=$(echo "$response" | tail -n1)
    body=$(echo "$response" | sed '$d')
    
    if [ "$http_code" -ge 200 ] && [ "$http_code" -lt 300 ]; then
        echo -e "${GREEN}✅ Success (HTTP $http_code)${NC}"
        echo "Response: $body" | jq . 2>/dev/null || echo "Response: $body"
    else
        echo -e "${RED}❌ Failed (HTTP $http_code)${NC}"
        echo "Response: $body" | jq . 2>/dev/null || echo "Response: $body"
    fi
}

# Test health endpoint
test_endpoint "GET" "/health" "Health Check"

# Test API v1 endpoints
test_endpoint "GET" "/api/v1/health" "API Health Check"

# Test server status
test_endpoint "GET" "/api/v1/status" "Server Status"

# Test neurons list
test_endpoint "GET" "/api/v1/neurons" "List Neurons"

# Test layers
test_endpoint "GET" "/api/v1/layers" "List Layers"

# Test metrics
test_endpoint "GET" "/api/v1/metrics" "Get Metrics"

# Test signal submission (will likely require auth)
echo -e "\n${YELLOW}Testing Signal Submission (might require auth)${NC}"
signal_data='{
    "content": "Test signal from API test",
    "layer": "L2",
    "neuron_id": "test-neuron"
}'
test_endpoint "POST" "/api/v1/signals" "Submit Signal" "$signal_data"

# Test auth endpoints (if enabled)
echo -e "\n${YELLOW}Testing Auth Endpoints${NC}"

# Test registration
register_data='{
    "username": "testuser",
    "email": "test@example.com",
    "password": "testpassword123"
}'
test_endpoint "POST" "/api/v1/auth/register" "User Registration" "$register_data"

# Test login
login_data='{
    "username": "testuser",
    "password": "testpassword123"
}'
test_endpoint "POST" "/api/v1/auth/login" "User Login" "$login_data"

# Test codegen endpoints
echo -e "\n${YELLOW}Testing Codegen Endpoints${NC}"
test_endpoint "GET" "/api/v1/codegen/health" "Codegen Health Check"
test_endpoint "GET" "/api/v1/codegen/templates" "List Templates"

echo -e "\n${GREEN}API Testing Complete!${NC}"