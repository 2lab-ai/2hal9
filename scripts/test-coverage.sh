#!/bin/bash
# Test coverage script for HAL9

set -e

echo "🧪 HAL9 Test Coverage Report"
echo "==========================="

# Install tarpaulin if not already installed
if ! command -v cargo-tarpaulin &> /dev/null; then
    echo "📦 Installing cargo-tarpaulin..."
    cargo install cargo-tarpaulin
fi

# Clean previous coverage
rm -rf target/coverage
mkdir -p target/coverage

echo ""
echo "🔍 Running tests with coverage..."

# Run tests with coverage
# Exclude test files and examples from coverage
cargo tarpaulin \
    --out Html \
    --out Lcov \
    --output-dir target/coverage \
    --workspace \
    --exclude-files "**/tests/*" \
    --exclude-files "**/examples/*" \
    --exclude-files "**/benches/*" \
    --exclude-files "**/build.rs" \
    --ignore-panics \
    --timeout 300 \
    --target-dir target/tarpaulin \
    || true

echo ""
echo "📊 Coverage Summary:"
echo "==================="

# Extract coverage percentage from lcov
if [ -f "target/coverage/lcov.info" ]; then
    # Calculate coverage
    TOTAL_LINES=$(grep -E "^DA:" target/coverage/lcov.info | wc -l)
    COVERED_LINES=$(grep -E "^DA:[0-9]+,[1-9]" target/coverage/lcov.info | wc -l)
    
    if [ $TOTAL_LINES -gt 0 ]; then
        COVERAGE=$((COVERED_LINES * 100 / TOTAL_LINES))
        echo "Overall Coverage: ${COVERAGE}%"
        echo "Covered Lines: ${COVERED_LINES}"
        echo "Total Lines: ${TOTAL_LINES}"
        
        # Check if we meet the 80% target
        if [ $COVERAGE -ge 80 ]; then
            echo ""
            echo "✅ Coverage target (80%) achieved!"
        else
            echo ""
            echo "⚠️  Coverage is below 80% target"
            echo "   Need to cover $((TOTAL_LINES * 80 / 100 - COVERED_LINES)) more lines"
        fi
    fi
fi

echo ""
echo "📄 Detailed reports available at:"
echo "   - HTML: target/coverage/tarpaulin-report.html"
echo "   - LCOV: target/coverage/lcov.info"

# Generate coverage badge
if [ -f "target/coverage/lcov.info" ] && [ $COVERAGE -gt 0 ]; then
    COLOR="red"
    if [ $COVERAGE -ge 80 ]; then
        COLOR="brightgreen"
    elif [ $COVERAGE -ge 60 ]; then
        COLOR="yellow"
    elif [ $COVERAGE -ge 40 ]; then
        COLOR="orange"
    fi
    
    echo ""
    echo "🏷️  Coverage Badge:"
    echo "   https://img.shields.io/badge/coverage-${COVERAGE}%25-${COLOR}"
fi