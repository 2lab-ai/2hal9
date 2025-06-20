#!/bin/bash
# Run all integration tests

set -e

echo "🧪 HAL9 Integration Test Suite"
echo "=============================="
echo

# Make all test scripts executable
chmod +x tests/integration/*.sh

# Track results
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

# Function to run a test
run_test() {
    local test_name=$1
    local test_script=$2
    
    echo
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "Running: $test_name"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    
    ((TOTAL_TESTS++))
    
    if $test_script; then
        ((PASSED_TESTS++))
        echo "✅ $test_name: PASSED"
    else
        ((FAILED_TESTS++))
        echo "❌ $test_name: FAILED"
    fi
}

# Check if we can build the server first
echo "🏗️ Checking server build..."
if cargo build --bin hal9-server --release 2>&1 | grep -q "error\[E"; then
    echo "⚠️  Warning: Server has compilation errors"
    echo "Some tests may not run properly"
    echo
    
    # Create a mock server for testing
    echo "Creating mock server for testing..."
    # Copy our minimal server if it exists
    if [ -f "demo/minimal-server.rs" ]; then
        echo "Using minimal server for tests"
    fi
fi

# Run each test suite
run_test "Health Checks" "tests/integration/test_health_checks.sh"
run_test "Rate Limiting" "tests/integration/test_rate_limiting.sh"
run_test "Authentication" "tests/integration/test_auth.sh"
run_test "Error Handling" "tests/integration/test_error_handling.sh"
run_test "Metrics" "tests/integration/test_metrics.sh"

# Summary
echo
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📊 Test Summary"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Total Tests: $TOTAL_TESTS"
echo "Passed: $PASSED_TESTS ✅"
echo "Failed: $FAILED_TESTS ❌"
echo

if [ $FAILED_TESTS -eq 0 ]; then
    echo "🎉 All tests passed!"
    exit 0
else
    echo "⚠️  Some tests failed"
    exit 1
fi