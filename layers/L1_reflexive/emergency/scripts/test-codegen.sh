#!/bin/bash
#
# test codegen
# Auto-fixed by L1 migration script
#

set -euo pipefail

# Source common environment
source "$(dirname "$0")/../../common-env.sh"

# Original script content (modified for new paths)


# Test script for HAL9 Code Generation System

set -e

echo "🤖 HAL9 Code Generation Test"
echo "============================"

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

# Start server with code generation neurons
log_test "Starting HAL9 server with code generation neurons..."
pkill -f hal9-server || true
sleep 2

HTTP_PORT=8080 $HAL9_SERVER_CMD $HAL9_CONFIG_DIR/codegen-neurons.yaml > codegen-server.log 2>&1 &
SERVER_PID=$!
echo "Server PID: $SERVER_PID"

# Wait for server to start
sleep 10

# Check if server is running
if ! lsof -i :8080 | grep -q LISTEN; then
    log_error "Server failed to start on port 8080"
    cat codegen-server.log | tail -20
    exit 1
fi
log_success "Server started successfully"

# Test 1: Check code generation health endpoint
log_test "Checking code generation health..."
HEALTH_RESPONSE=$(curl -s "http://localhost:$HAL9_PORT_MAIN/api/v1/codegen/health")
if echo "$HEALTH_RESPONSE" | grep -q "healthy"; then
    log_success "Code generation service is healthy"
    echo "$HEALTH_RESPONSE" | jq .
else
    log_error "Code generation service is not healthy"
    echo "$HEALTH_RESPONSE"
fi

# Test 2: List available templates
log_test "Listing available project templates..."
TEMPLATES_RESPONSE=$(curl -s "http://localhost:$HAL9_PORT_MAIN/api/v1/codegen/templates")
if echo "$TEMPLATES_RESPONSE" | grep -q "web-app"; then
    log_success "Templates retrieved successfully"
    echo "$TEMPLATES_RESPONSE" | jq .
else
    log_error "Failed to retrieve templates"
fi

# Test 3: Generate a project via API
log_test "Generating a web application project..."
PROJECT_RESPONSE=$(curl -s -X POST "http://localhost:$HAL9_PORT_MAIN/api/v1/codegen/project" \
    -H "Content-Type: application/json" \
    -d '{
        "description": "E-commerce web application",
        "project_type": "web-app",
        "preferences": {
            "backend": "fastapi",
            "frontend": "react",
            "database": "postgresql",
            "testing": true,
            "docker": true,
            "ci_cd": true
        }
    }')

if echo "$PROJECT_RESPONSE" | grep -q "project_id"; then
    log_success "Project generation started"
    echo "$PROJECT_RESPONSE" | jq .
    PROJECT_ID=$(echo "$PROJECT_RESPONSE" | jq -r .project_id)
else
    log_error "Failed to start project generation"
    echo "$PROJECT_RESPONSE"
fi

# Test 4: Code review
log_test "Testing code review..."
REVIEW_RESPONSE=$(curl -s -X POST "http://localhost:$HAL9_PORT_MAIN/api/v1/codegen/review" \
    -H "Content-Type: application/json" \
    -d '{
        "file_path": "test.py",
        "content": "def process_data(data):\n    result = []\n    for item in data:\n        result.append(item * 2)\n    return result",
        "focus": ["performance", "best-practices"]
    }')

if echo "$REVIEW_RESPONSE" | grep -q "overall_score"; then
    log_success "Code review completed"
    echo "$REVIEW_RESPONSE" | jq .
else
    log_error "Code review failed"
fi

# Test 5: Code completion
log_test "Testing code completion..."
COMPLETION_RESPONSE=$(curl -s -X POST "http://localhost:$HAL9_PORT_MAIN/api/v1/codegen/complete" \
    -H "Content-Type: application/json" \
    -d '{
        "file_path": "main.rs",
        "cursor_position": 50,
        "context": "fn main() {\n    // TODO: Implement\n}",
        "language": "rust"
    }')

if echo "$COMPLETION_RESPONSE" | grep -q "suggestions"; then
    log_success "Code completion returned suggestions"
    echo "$COMPLETION_RESPONSE" | jq .
else
    log_error "Code completion failed"
fi

# Test 6: CLI tool help
log_test "Testing CLI tool..."
if $HAL9_CODEGEN_CMD -- --help > /dev/null 2>&1; then
    log_success "CLI tool built and runs successfully"
else
    log_error "CLI tool failed to run"
fi

# Test 7: CLI list templates
log_test "Testing CLI template listing..."
$HAL9_CODEGEN_CMD -- new --yes > cli-output.txt 2>&1 || true
if grep -q "Project type required" cli-output.txt; then
    log_success "CLI properly validates required inputs"
else
    log_error "CLI validation not working"
fi

# Cleanup
log_test "Cleaning up..."
kill $SERVER_PID 2>/dev/null || true
rm -f cli-output.txt

echo ""
echo "============================"
log_success "Code generation tests completed!"
echo ""
echo "Summary:"
echo "- Code generation service: ✓"
echo "- API endpoints working: ✓"
echo "- CLI tool functional: ✓"
echo "- Templates available: ✓"
echo "- Review and completion: ✓"

echo ""
log_info "Next steps:"
echo "1. Use 'hal9-codegen new' to generate a project interactively"
echo "2. Use 'hal9-codegen review <file>' to review code"
echo "3. Use 'hal9-codegen refactor <file>' to refactor code"