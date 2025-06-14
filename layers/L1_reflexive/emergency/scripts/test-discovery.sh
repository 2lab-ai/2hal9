#!/bin/bash
#
# test discovery
# Auto-fixed by L1 migration script
#

set -euo pipefail

# Source common environment
source "$(dirname "$0")/../../common-env.sh"

# Original script content (modified for new paths)


# Kill any existing servers
pkill -f "hal9-server" || true
sleep 1

# Create log directory
mkdir -p logs

# Start server 1 with debug logging
echo "Starting server 1..."
RUST_LOG=debug,hyper=info,tower=info HTTP_PORT=8081 $HAL9_SERVER_BIN \
    ./$HAL9_CONFIG_DIR/config-server1.yaml \
    > logs/server1-debug.log 2>&1 &

SERVER1_PID=$!
echo "Server 1 started with PID $SERVER1_PID"

sleep 2

# Start server 2 with debug logging
echo "Starting server 2..."
RUST_LOG=debug,hyper=info,tower=info HTTP_PORT=8082 $HAL9_SERVER_BIN \
    ./$HAL9_CONFIG_DIR/config-server2.yaml \
    > logs/server2-debug.log 2>&1 &

SERVER2_PID=$!
echo "Server 2 started with PID $SERVER2_PID"

echo ""
echo "Servers started. Waiting 15 seconds for discovery..."
echo "Logs are in logs/server1-debug.log and logs/server2-debug.log"
echo ""

sleep 15

echo "Checking discovery status..."
echo ""
echo "=== Server 1 Status ==="
curl -s http://localhost:8081/api/v1/status | jq '.data.network_status' || echo "Failed to get status"
echo ""
echo "=== Server 2 Status ==="
curl -s http://localhost:8082/api/v1/status | jq '.data.network_status' || echo "Failed to get status"

echo ""
echo "Checking logs for discovery messages..."
echo ""
echo "=== Server 1 Discovery Logs ==="
grep -i "discovery\|broadcast" logs/server1-debug.log | tail -20
echo ""
echo "=== Server 2 Discovery Logs ==="
grep -i "discovery\|broadcast" logs/server2-debug.log | tail -20

echo ""
echo "To stop servers: kill $SERVER1_PID $SERVER2_PID"
echo "Or run: pkill -f 'hal9-server'"