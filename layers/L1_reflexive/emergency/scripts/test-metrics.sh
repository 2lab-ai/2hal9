#!/bin/bash
#
# test metrics
# Auto-fixed by L1 migration script
#

set -euo pipefail

# Source common environment
source "$(dirname "$0")/../../common-env.sh"

# Original script content (modified for new paths)


echo "🚀 Testing HAL9 Enhanced Metrics System"
echo "======================================="

# Start server in background
echo "Starting server..."
$HAL9_SERVER_CMD -- $HAL9_CONFIG_DIR/config-3neurons-enhanced.yaml &
SERVER_PID=$!

# Wait for server to start
echo "Waiting for server to start..."
sleep 5

# Check if server is running
if ! ps -p $SERVER_PID > /dev/null; then
    echo "❌ Server failed to start"
    exit 1
fi

echo "✅ Server started (PID: $SERVER_PID)"

# Test health endpoint
echo -e "\n📍 Testing health endpoint..."
curl -s http://localhost:$HAL9_PORT_MAIN/health | jq .

# Test metrics endpoint
echo -e "\n📊 Testing metrics endpoint..."
curl -s http://localhost:$HAL9_PORT_MAIN/api/v1/metrics | jq .

# Submit a test signal
echo -e "\n🚦 Submitting test signal..."
curl -s -X POST http://localhost:$HAL9_PORT_MAIN/api/v1/signal \
  -H "Content-Type: application/json" \
  -d '{
    "content": "create web app with authentication",
    "layer": "L4"
  }' | jq .

# Wait for signal processing
sleep 3

# Check metrics again
echo -e "\n📊 Checking metrics after signal processing..."
curl -s http://localhost:$HAL9_PORT_MAIN/api/v1/metrics | jq .

# Export metrics in JSON format
echo -e "\n📥 Exporting metrics..."
curl -s "http://localhost:$HAL9_PORT_MAIN/api/v1/metrics/export?format=json" | jq .

# Check server status
echo -e "\n🔍 Checking server status..."
curl -s http://localhost:$HAL9_PORT_MAIN/api/v1/status | jq .

# Kill the server
echo -e "\n🛑 Stopping server..."
kill $SERVER_PID
wait $SERVER_PID 2>/dev/null

echo -e "\n✅ Test completed successfully!"