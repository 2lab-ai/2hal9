#!/bin/bash
#
# run demo
# Auto-fixed by L1 migration script
#

set -euo pipefail

# Source common environment
source "$(dirname "$0")/../../common-env.sh"

# Original script content (modified for new paths)


# HAL9 Demo Script

echo "🧠 HAL9 - Hierarchical AI Neural Network Demo"
echo "=============================================="
echo ""

# Check if cargo is installed
if ! command -v cargo &> /dev/null; then
    log_error " Cargo is not installed. Please install Rust first."
    exit 1
fi

# Build the project
echo "📦 Building HAL9..."
cargo build --release

if [ $? -ne 0 ]; then
    echo "❌ Build failed!"
    exit 1
fi

echo "✅ Build successful!"
echo ""

# Run with 3-neuron configuration
echo "🚀 Starting HAL9 with 3-neuron configuration..."
echo ""
echo "Run the following command to start the server:"
echo ""
echo "  $HAL9_CLI_BIN start --config $HAL9_CONFIG_DIR/config-3neurons.yaml"
echo ""
echo "In another terminal, you can:"
echo ""
echo "  # Check status"
echo "  $HAL9_CLI_BIN status"
echo ""
echo "  # Send a signal"
echo "  $HAL9_CLI_BIN signal --to neuron-1 --content \"Create a web application\""
echo ""
echo "Press Ctrl+C to stop the server."