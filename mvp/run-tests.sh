#!/bin/bash

# 2HAL9 MVP Test Runner

echo "🧪 2HAL9 MVP Test Suite"
echo "======================"
echo

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[0;33m'
NC='\033[0m' # No Color

# Run tests with various options
echo "Running all tests..."
echo

# Basic test run
echo -e "${YELLOW}[1/4] Running unit tests...${NC}"
cargo test --package hal9_mvp -- --nocapture 2>&1 | grep -E "(test result:|passed|failed|FAILED)"

echo
echo -e "${YELLOW}[2/4] Running tests with release optimizations...${NC}"
cargo test --package hal9_mvp --release -- --nocapture 2>&1 | grep -E "(test result:|passed|failed)"

echo
echo -e "${YELLOW}[3/4] Running specific test modules...${NC}"
echo "  - Signal tests..."
cargo test --package hal9_mvp signal_structure_tests -- --nocapture 2>&1 | grep -E "(passed|failed)"

echo "  - Neuron processing tests..."
cargo test --package hal9_mvp neuron_processing_tests -- --nocapture 2>&1 | grep -E "(passed|failed)"

echo "  - Recording tests..."
cargo test --package hal9_mvp recording_system_tests -- --nocapture 2>&1 | grep -E "(passed|failed)"

echo "  - Integration tests..."
cargo test --package hal9_mvp integration_flow_tests -- --nocapture 2>&1 | grep -E "(passed|failed)"

echo "  - Performance tests..."
cargo test --package hal9_mvp performance_tests -- --nocapture 2>&1 | grep -E "(passed|failed)"

echo "  - Error handling tests..."
cargo test --package hal9_mvp error_handling_tests -- --nocapture 2>&1 | grep -E "(passed|failed)"

echo
echo -e "${YELLOW}[4/4] Running coverage summary...${NC}"
cargo test --package hal9_mvp coverage_summary -- --nocapture

echo
echo -e "${GREEN}✓ Test suite complete!${NC}"
echo

# Optional: Run with detailed output
if [ "$1" == "--verbose" ]; then
    echo "Running with verbose output..."
    cargo test --package hal9_mvp -- --nocapture --test-threads=1
fi

# Optional: Run benchmarks
if [ "$1" == "--bench" ]; then
    echo "Running performance benchmarks..."
    cargo test --package hal9_mvp performance_tests --release -- --nocapture --test-threads=1
fi