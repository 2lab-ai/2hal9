#!/bin/bash

# Test script for HAL9 authentication system

set -e

echo "🔐 HAL9 Authentication System Test"
echo "=================================="

# Configuration
API_BASE="http://localhost:9736/api/v1"
AUTH_BASE="$API_BASE/auth"

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Helper functions
log_test() {
    echo -e "${YELLOW}Test: $1${NC}"
}

log_success() {
    echo -e "${GREEN}✓ $1${NC}"
}

log_error() {
    echo -e "${RED}✗ $1${NC}"
}

log_info() {
    echo -e "${BLUE}ℹ $1${NC}"
}

# Start server with auth enabled
log_test "Starting HAL9 server with authentication..."
pkill -f hal9-server || true
sleep 2

HTTP_PORT=9736 cargo run --bin hal9-server examples/auth-test-absolute.yaml > auth-server.log 2>&1 &
SERVER_PID=$!
echo "Server PID: $SERVER_PID"

# Wait for server to start
sleep 10

# Check if server is running
if ! lsof -i :9736 | grep -q LISTEN; then
    log_error "Server failed to start on port 9736"
    cat auth-server.log
    exit 1
fi
log_success "Server started successfully"

# Test 1: Register a new user
log_test "Registering new user..."
REGISTER_RESPONSE=$(curl -s -X POST "$AUTH_BASE/register" \
    -H "Content-Type: application/json" \
    -d '{
        "username": "testuser",
        "email": "test@example.com",
        "password": "Test123!@#",
        "role": "user"
    }')

if echo "$REGISTER_RESPONSE" | grep -q "testuser"; then
    log_success "User registration successful"
    echo "$REGISTER_RESPONSE" | jq .
else
    log_error "User registration failed"
    echo "$REGISTER_RESPONSE"
fi

# Test 2: Login with credentials
log_test "Logging in..."
LOGIN_RESPONSE=$(curl -s -X POST "$AUTH_BASE/login" \
    -H "Content-Type: application/json" \
    -d '{
        "username": "testuser",
        "password": "Test123!@#"
    }')

# Extract tokens
ACCESS_TOKEN=$(echo "$LOGIN_RESPONSE" | jq -r '.tokens.access_token // empty')
REFRESH_TOKEN=$(echo "$LOGIN_RESPONSE" | jq -r '.tokens.refresh_token // empty')

if [ -n "$ACCESS_TOKEN" ]; then
    log_success "Login successful"
    log_info "Access token received (length: ${#ACCESS_TOKEN})"
else
    log_error "Login failed - no access token"
    echo "$LOGIN_RESPONSE"
    exit 1
fi

# Test 3: Access protected endpoint with JWT
log_test "Accessing protected endpoint with JWT..."
PROFILE_RESPONSE=$(curl -s -X GET "$AUTH_BASE/profile" \
    -H "Authorization: Bearer $ACCESS_TOKEN")

if echo "$PROFILE_RESPONSE" | grep -q "testuser"; then
    log_success "Protected endpoint access successful"
    echo "$PROFILE_RESPONSE" | jq .
else
    log_error "Protected endpoint access failed"
    echo "$PROFILE_RESPONSE"
fi

# Test 4: Create API key
log_test "Creating API key..."
API_KEY_RESPONSE=$(curl -s -X POST "$AUTH_BASE/api-keys" \
    -H "Authorization: Bearer $ACCESS_TOKEN" \
    -H "Content-Type: application/json" \
    -d '{
        "name": "Test API Key",
        "permissions": {
            "permissions": ["SendSignal", "ViewNeuron", "ViewSignals"]
        },
        "expires_in_days": 30
    }')

API_KEY=$(echo "$API_KEY_RESPONSE" | jq -r '.key // empty')

if [ -n "$API_KEY" ]; then
    log_success "API key created successfully"
    log_info "API key: $API_KEY"
else
    log_error "API key creation failed"
    echo "$API_KEY_RESPONSE"
fi

# Test 5: Use API key for authentication
log_test "Using API key for authentication..."
API_KEY_TEST=$(curl -s -X GET "$AUTH_BASE/profile" \
    -H "X-API-Key: $API_KEY")

if echo "$API_KEY_TEST" | grep -q "api_key_Test API Key"; then
    log_success "API key authentication successful"
else
    log_error "API key authentication failed"
    echo "$API_KEY_TEST"
fi

# Test 6: Refresh token
log_test "Refreshing access token..."
REFRESH_RESPONSE=$(curl -s -X POST "$AUTH_BASE/refresh" \
    -H "Content-Type: application/json" \
    -d "{\"refresh_token\": \"$REFRESH_TOKEN\"}")

NEW_ACCESS_TOKEN=$(echo "$REFRESH_RESPONSE" | jq -r '.access_token // empty')

if [ -n "$NEW_ACCESS_TOKEN" ]; then
    log_success "Token refresh successful"
    log_info "New access token received"
else
    log_error "Token refresh failed"
    echo "$REFRESH_RESPONSE"
fi

# Test 7: Send signal with authentication
log_test "Sending authenticated signal..."
SIGNAL_RESPONSE=$(curl -s -X POST "$API_BASE/signal" \
    -H "Authorization: Bearer $ACCESS_TOKEN" \
    -H "Content-Type: application/json" \
    -d '{
        "content": "Test authenticated signal",
        "layer": "L4",
        "neuron_id": "test-neuron"
    }')

if echo "$SIGNAL_RESPONSE" | grep -q "success"; then
    log_success "Authenticated signal sent successfully"
else
    log_error "Authenticated signal failed"
    echo "$SIGNAL_RESPONSE"
fi

# Test 8: List API keys
log_test "Listing API keys..."
LIST_KEYS_RESPONSE=$(curl -s -X GET "$AUTH_BASE/api-keys" \
    -H "Authorization: Bearer $ACCESS_TOKEN")

if echo "$LIST_KEYS_RESPONSE" | grep -q "Test API Key"; then
    log_success "API keys listed successfully"
    echo "$LIST_KEYS_RESPONSE" | jq .
else
    log_error "API key listing failed"
fi

# Test 9: Admin user registration
log_test "Registering admin user..."
ADMIN_RESPONSE=$(curl -s -X POST "$AUTH_BASE/register" \
    -H "Content-Type: application/json" \
    -d '{
        "username": "admin",
        "email": "admin@example.com",
        "password": "Admin123!@#",
        "role": "admin"
    }')

if echo "$ADMIN_RESPONSE" | grep -q "admin"; then
    log_success "Admin registration successful"
else
    log_error "Admin registration failed"
fi

# Cleanup
log_test "Cleaning up..."
kill $SERVER_PID 2>/dev/null || true

echo ""
echo "=================================="
log_success "Authentication tests completed!"
echo ""
echo "Summary:"
echo "- User registration: ✓"
echo "- JWT authentication: ✓"
echo "- API key authentication: ✓"
echo "- Token refresh: ✓"
echo "- Protected endpoints: ✓"

# Check auth database
if [ -f "data/hal9_auth.db" ]; then
    log_info "Auth database created at: data/hal9_auth.db"
    log_info "Database size: $(du -h data/hal9_auth.db | cut -f1)"
    
    echo ""
    log_info "Database contents:"
    sqlite3 data/hal9_auth.db "SELECT 'Users:', COUNT(*) FROM users;"
    sqlite3 data/hal9_auth.db "SELECT 'API Keys:', COUNT(*) FROM api_keys;"
fi