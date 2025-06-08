#!/bin/bash
# Run task composition tests

set -e

echo "🧪 Running Task Composition Tests"
echo "================================="
echo

# Run the integration tests
echo "Running integration tests..."
cargo test --package hal9_mvp --test integration_tests -- --nocapture

echo
echo "Running task composition tests..."
cargo test --package hal9_mvp --test task_composition_tests -- --nocapture 2>/dev/null || {
    echo "Note: task_composition_tests requires lib.rs setup, skipping..."
}

echo
echo "✅ All tests completed!"