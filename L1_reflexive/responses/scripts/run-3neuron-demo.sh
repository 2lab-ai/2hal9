#!/bin/bash

# Check if a specific config is requested
CONFIG=${1:-"examples/config-demo-scenarios.yaml"}

echo "🚀 Starting HAL9 3-Neuron Demo Server..."
echo "=================================="
echo ""
echo "Configuration: $CONFIG"
echo "HTTP API: http://localhost:8080"
echo ""
echo "Available configurations:"
echo "  - examples/config-3neurons.yaml (basic)"
echo "  - examples/config-3neurons-enhanced.yaml (enhanced)"
echo "  - examples/config-demo-scenarios.yaml (full demo)"
echo ""
echo "Press Ctrl+C to stop the server"
echo ""

# Run the server with specified configuration
cargo run --bin hal9-server "$CONFIG"