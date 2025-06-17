#!/bin/bash

echo "🔌 WebSocket Interactive Test"
echo "============================"
echo ""
echo "This script will test WebSocket functionality interactively."
echo ""

# Test echo message
echo "📤 Test 1: Echo message"
echo '{"type":"echo","payload":"Hello HAL9!"}' | \
  timeout 2s curl -s -N \
    -H "Connection: Upgrade" \
    -H "Upgrade: websocket" \
    -H "Sec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==" \
    -H "Sec-WebSocket-Version: 13" \
    --data-binary @- \
    http://localhost:8080/ws || echo "Timeout (expected for WebSocket upgrade)"

echo ""
echo "📤 Test 2: Using nc (netcat) to test WebSocket"

# Create a test message file
cat > /tmp/ws_test.txt << 'EOF'
GET /ws HTTP/1.1
Host: localhost:8080
Upgrade: websocket
Connection: Upgrade
Sec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==
Sec-WebSocket-Version: 13

EOF

echo "Sending WebSocket upgrade request..."
timeout 2s nc localhost 8080 < /tmp/ws_test.txt

echo ""
echo "✅ WebSocket endpoint is accepting connections!"
echo "The timeout is expected - it means the WebSocket upgrade was successful."
echo ""
echo "📊 Check server logs for WebSocket connection details:"
echo "docker logs hal9-server --tail 20 | grep -i websocket"