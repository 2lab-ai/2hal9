#!/bin/bash

# 2HAL9 Demo Recording Script

echo "🔴 2HAL9 Demo Recording Mode"
echo "============================"
echo
echo "This will record your demo session for later replay."
echo "Recordings will be saved to mvp/recordings/"
echo

# Build if needed
cargo build --release -p hal9_mvp

if [ $? -ne 0 ]; then
    echo "❌ Build failed!"
    exit 1
fi

echo "Starting recording mode..."
echo

# Run with record flag
cargo run --release -p hal9_mvp -- --record