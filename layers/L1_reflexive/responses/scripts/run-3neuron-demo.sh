#!/bin/bash
#
# run 3neuron demo
# Auto-fixed by L1 migration script
#

set -euo pipefail

# Source common environment
source "$(dirname "$0")/../../common-env.sh"

# Original script content (modified for new paths)


# Check if a specific config is requested
CONFIG=${1:-"$HAL9_CONFIG_DIR/config-demo-scenarios.yaml"}

echo "🚀 Starting HAL9 3-Neuron Demo Server..."
echo "=================================="
echo ""
echo "Configuration: $CONFIG"
echo "HTTP API: http://localhost:$HAL9_PORT_MAIN"
echo ""
echo "Available configurations:"
echo "  - $HAL9_CONFIG_DIR/config-3neurons.yaml (basic)"
echo "  - $HAL9_CONFIG_DIR/config-3neurons-enhanced.yaml (enhanced)"
echo "  - $HAL9_CONFIG_DIR/config-demo-scenarios.yaml (full demo)"
echo ""
echo "Press Ctrl+C to stop the server"
echo ""

# Run the server with specified configuration
$HAL9_SERVER_CMD "$CONFIG"