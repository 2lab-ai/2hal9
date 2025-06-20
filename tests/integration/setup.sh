#!/bin/bash
# Integration test setup script

set -e

echo "🧪 Setting up HAL9 Integration Test Environment"
echo "=============================================="
echo

# Create test directory structure
mkdir -p tests/integration/{health,auth,rate_limit,neurons}

# Environment setup
export TEST_DATABASE_URL="sqlite://./test_hal9.db"
export CLAUDE_MODE="mock"
export JWT_SECRET="test-secret-key-for-integration-tests"
export RUST_LOG="hal9=debug"

echo "📋 Creating test database..."
rm -f test_hal9.db
DATABASE_URL=$TEST_DATABASE_URL ./layers/L3_operational/scripts/setup-database.sh

echo
echo "✅ Test environment ready!"
echo
echo "Test Configuration:"
echo "  Database: $TEST_DATABASE_URL"
echo "  Claude Mode: $CLAUDE_MODE"
echo "  JWT Secret: [CONFIGURED]"
echo "  Log Level: $RUST_LOG"