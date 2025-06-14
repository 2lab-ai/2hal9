#!/bin/bash

# HAL9 MVP Web Interface Runner

echo "🌐 Starting HAL9 Web Interface..."
echo

# Build if needed
cargo build --release -p hal9_mvp

if [ $? -ne 0 ]; then
    echo "❌ Build failed!"
    exit 1
fi

echo
echo "🚀 Launching web server..."
echo

# Run with web flag
cargo run --release -p hal9_mvp -- --web