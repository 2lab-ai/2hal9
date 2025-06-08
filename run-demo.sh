#!/bin/bash

# 2HAL9 Demo Script

echo "🧠 2HAL9 - Hierarchical AI Neural Network Demo"
echo "=============================================="
echo ""

# Check if cargo is installed
if ! command -v cargo &> /dev/null; then
    echo "Error: Cargo is not installed. Please install Rust first."
    exit 1
fi

# Build the project
echo "📦 Building 2HAL9..."
cargo build --release

if [ $? -ne 0 ]; then
    echo "❌ Build failed!"
    exit 1
fi

echo "✅ Build successful!"
echo ""

# Run with 3-neuron configuration
echo "🚀 Starting 2HAL9 with 3-neuron configuration..."
echo ""
echo "Run the following command to start the server:"
echo ""
echo "  ./target/release/2hal9 start --config examples/config-3neurons.yaml"
echo ""
echo "In another terminal, you can:"
echo ""
echo "  # Check status"
echo "  ./target/release/2hal9 status"
echo ""
echo "  # Send a signal"
echo "  ./target/release/2hal9 signal --to neuron-1 --content \"Create a web application\""
echo ""
echo "Press Ctrl+C to stop the server."