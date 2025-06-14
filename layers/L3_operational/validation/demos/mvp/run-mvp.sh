#!/bin/bash

# HAL9 MVP Demo Runner

echo "🧠 HAL9 MVP - Hierarchical AI Orchestration Demo"
echo "================================================"
echo ""
echo "This demo shows how a user request flows through"
echo "3 layers of AI neurons (L4→L3→L2), each handling"
echo "a different level of abstraction."
echo ""
echo "Building MVP..."
cargo build --release -p hal9_mvp

if [ $? -ne 0 ]; then
    echo "❌ Build failed!"
    exit 1
fi

echo ""
echo "Starting demo..."
echo ""

cargo run --release -p hal9_mvp