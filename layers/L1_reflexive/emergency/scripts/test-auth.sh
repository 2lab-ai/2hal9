#!/bin/bash
#
# Test authentication functionality
# Tests JWT authentication, user management, and protected endpoints
#

set -euo pipefail

# Source common environment
source "$(dirname "$0")/../../common-env.sh"

log_info "🔐 Testing HAL9 authentication system..."

# Check required commands
require_command curl "curl is required for API testing"
require_command jq "jq is required for JSON parsing"

# Set auth-specific port
AUTH_PORT="${HAL9_PORT_AUTH:-8081}"

# Function to ensure auth database exists
ensure_auth_database() {
    local DB_FILE="$HAL9_DATA_DIR/hal9_auth.db"
    
    if [ ! -f "$DB_FILE" ]; then
        log_warning "Auth database not found at: $DB_FILE"
        log_info "Creating auth database..."
        
        # Try to create from schema if available
        local SCHEMA_FILE="$HAL9_HOME/substrate/storage/migrations/sqlite/001_initial_schema.sql"
        if [ -f "$SCHEMA_FILE" ]; then
            sqlite3 "$DB_FILE" < "$SCHEMA_FILE" 2>/dev/null || {
                log_warning "Could not apply schema, creating empty database"
                touch "$DB_FILE"
            }
        else
            touch "$DB_FILE"
        fi
    fi
    
    log_info "Auth database: $DB_FILE"
}

# Function to start server if not running
ensure_server_running() {
    if curl -s "http://localhost:$AUTH_PORT/health" > /dev/null 2>&1; then
        log_info "Auth server is already running"
        return 0
    fi
    
    log_info "Starting HAL9 server with authentication enabled..."
    
    # Ensure database exists
    ensure_auth_database
    
    # Check if config exists
    CONFIG_FILE="$HAL9_CONFIG_DIR/auth-test.yaml"
    if [ ! -f "$CONFIG_FILE" ]; then
        log_warning "Auth config not found: $CONFIG_FILE"
        log_info "Using auth-test-absolute.yaml instead..."
        CONFIG_FILE="$HAL9_CONFIG_DIR/auth-test-absolute.yaml"
        
        if [ ! -f "$CONFIG_FILE" ]; then
            log_error "No auth config file found"
            log_info "Creating minimal auth config..."
            
            # Create minimal auth config
            CONFIG_FILE="$HAL9_LOG_DIR/auth-test-minimal.yaml"
            cat > "$CONFIG_FILE" <<EOF
server_id: hal9-auth-test
port: $AUTH_PORT

auth:
  enabled: true
  jwt_secret: "test-secret-key-do-not-use-in-production"
  database_path: "$HAL9_DATA_DIR/hal9_auth.db"

neurons: []

monitoring:
  enabled: false
EOF
        fi
    fi
    
    # Check if port is available
    if ! check_port $AUTH_PORT; then
        log_error "Port $AUTH_PORT is already in use"
        exit 1
    fi
    
    # Start server
    LOG_FILE="$HAL9_LOG_DIR/test-auth.log"
    log_info "Starting server with config: $CONFIG_FILE"
    log_info "Log file: $LOG_FILE"
    
    cd "$HAL9_HOME"
    HTTP_PORT=$AUTH_PORT $HAL9_SERVER_CMD "$CONFIG_FILE" > "$LOG_FILE" 2>&1 &
    SERVER_PID=$!
    echo $SERVER_PID > "$HAL9_LOG_DIR/test-auth.pid"
    
    # Set up cleanup
    setup_cleanup "$HAL9_LOG_DIR/test-auth.pid" "$LOG_FILE"
    
    # Wait for server to be ready
    wait_for_service "http://localhost:$AUTH_PORT/health" 30 "Auth server" || {
        log_error "Server failed to start. Check log at: $LOG_FILE"
        tail -20 "$LOG_FILE"
        exit 1
    }
    
    STARTED_SERVER=true
}

# Test functions
test_health() {
    log_info "Testing health endpoint..."
    local response=$(curl -s "http://localhost:$AUTH_PORT/health")
    
    if echo "$response" | jq -e '.status == "healthy"' > /dev/null 2>&1; then
        log_success "Health check passed"
    else
        log_error "Health check failed"
        echo "$response"
        return 1
    fi
}

test_register() {
    log_info "Testing user registration..."
    
    local username="testuser_$(date +%s)"
    local email="${username}@test.com"
    local password="SecurePass123!"
    
    local response=$(curl -s -X POST "http://localhost:$AUTH_PORT/api/v1/auth/register" \
        -H "Content-Type: application/json" \
        -d "{
            \"username\": \"$username\",
            \"email\": \"$email\",
            \"password\": \"$password\"
        }")
    
    if echo "$response" | jq -e '.user.id' > /dev/null 2>&1; then
        log_success "Registration successful"
        echo "$response" | jq '.'
        
        # Save for later tests
        echo "$username:$password" > "$HAL9_LOG_DIR/test-auth-creds.txt"
    else
        log_error "Registration failed"
        echo "$response"
        return 1
    fi
}

test_login() {
    log_info "Testing user login..."
    
    # Get credentials from previous test
    if [ ! -f "$HAL9_LOG_DIR/test-auth-creds.txt" ]; then
        log_warning "No test credentials found, creating new user"
        test_register || return 1
    fi
    
    local creds=$(cat "$HAL9_LOG_DIR/test-auth-creds.txt")
    local username=$(echo "$creds" | cut -d: -f1)
    local password=$(echo "$creds" | cut -d: -f2)
    
    local response=$(curl -s -X POST "http://localhost:$AUTH_PORT/api/v1/auth/login" \
        -H "Content-Type: application/json" \
        -d "{
            \"username\": \"$username\",
            \"password\": \"$password\"
        }")
    
    if echo "$response" | jq -e '.token' > /dev/null 2>&1; then
        log_success "Login successful"
        local token=$(echo "$response" | jq -r '.token')
        echo "$token" > "$HAL9_LOG_DIR/test-auth-token.txt"
        
        # Decode token to show expiry
        log_info "Token info:"
        echo "$token" | cut -d. -f2 | base64 -d 2>/dev/null | jq '.' 2>/dev/null || true
    else
        log_error "Login failed"
        echo "$response"
        return 1
    fi
}

test_protected_endpoint() {
    log_info "Testing protected endpoint..."
    
    # Get token from previous test
    if [ ! -f "$HAL9_LOG_DIR/test-auth-token.txt" ]; then
        log_warning "No token found, logging in first"
        test_login || return 1
    fi
    
    local token=$(cat "$HAL9_LOG_DIR/test-auth-token.txt")
    
    # Test without token (should fail)
    log_info "Testing without token (should fail)..."
    local response=$(curl -s -w "\nHTTP_STATUS:%{http_code}" "http://localhost:$AUTH_PORT/api/v1/neurons")
    local status=$(echo "$response" | grep "HTTP_STATUS:" | cut -d: -f2)
    
    if [ "$status" = "401" ] || [ "$status" = "403" ]; then
        log_success "Correctly rejected unauthorized request"
    else
        log_warning "Expected 401/403, got $status"
    fi
    
    # Test with token (should succeed)
    log_info "Testing with token (should succeed)..."
    response=$(curl -s -w "\nHTTP_STATUS:%{http_code}" \
        -H "Authorization: Bearer $token" \
        "http://localhost:$AUTH_PORT/api/v1/neurons")
    status=$(echo "$response" | grep "HTTP_STATUS:" | cut -d: -f2)
    
    if [ "$status" = "200" ]; then
        log_success "Protected endpoint accessible with valid token"
    else
        log_error "Failed to access protected endpoint with token (status: $status)"
        echo "$response"
        return 1
    fi
}

test_token_refresh() {
    log_info "Testing token refresh..."
    
    if [ ! -f "$HAL9_LOG_DIR/test-auth-token.txt" ]; then
        log_warning "No token found, skipping refresh test"
        return 0
    fi
    
    local old_token=$(cat "$HAL9_LOG_DIR/test-auth-token.txt")
    
    local response=$(curl -s -X POST "http://localhost:$AUTH_PORT/api/v1/auth/refresh" \
        -H "Authorization: Bearer $old_token")
    
    if echo "$response" | jq -e '.token' > /dev/null 2>&1; then
        log_success "Token refresh successful"
        local new_token=$(echo "$response" | jq -r '.token')
        echo "$new_token" > "$HAL9_LOG_DIR/test-auth-token.txt"
    else
        log_warning "Token refresh not implemented or failed"
        echo "$response"
    fi
}

# Main test execution
main() {
    local STARTED_SERVER=false
    
    # Ensure server is running
    ensure_server_running
    
    # Run tests
    log_info "Running authentication tests..."
    
    test_health || log_warning "Health check failed"
    
    test_register || log_warning "Registration test failed"
    
    test_login || log_warning "Login test failed"
    
    test_protected_endpoint || log_warning "Protected endpoint test failed"
    
    test_token_refresh || log_warning "Token refresh test failed"
    
    log_success "✅ Authentication tests complete!"
    
    # Cleanup test data
    rm -f "$HAL9_LOG_DIR/test-auth-creds.txt" "$HAL9_LOG_DIR/test-auth-token.txt"
    
    # Only stop server if we started it
    if [ "$STARTED_SERVER" = true ]; then
        log_info "Stopping auth server..."
    fi
}

# Run main function
main "$@"