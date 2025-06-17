#!/bin/bash
# Simple WebSocket test using curl

echo "🧪 Testing WebSocket Endpoints"
echo "============================="

# Test if WebSocket endpoint exists
echo -e "\n🔌 Testing WebSocket upgrade..."
response=$(curl -s -o /dev/null -w "%{http_code}" \
    -H "Connection: Upgrade" \
    -H "Upgrade: websocket" \
    -H "Sec-WebSocket-Version: 13" \
    -H "Sec-WebSocket-Key: x3JJHMbDL1EzLkh9GBhXDw==" \
    http://localhost:8080/ws)

if [ "$response" = "101" ]; then
    echo "✅ WebSocket endpoint found (expects 101 Switching Protocols)"
elif [ "$response" = "426" ]; then
    echo "✅ WebSocket endpoint exists (426 Upgrade Required)"
elif [ "$response" = "404" ]; then
    echo "❌ WebSocket endpoint not found (404)"
else
    echo "⚠️  Unexpected response: HTTP $response"
fi

# Check if server has WebSocket routes
echo -e "\n📡 Checking for WebSocket routes in server logs..."
docker-compose logs hal9-server 2>&1 | grep -i websocket | tail -5 || echo "No WebSocket mentions in logs"

echo -e "\n💡 WebSocket Implementation Status:"
echo "Based on the response, WebSocket support needs to be implemented in the server."
echo "This would typically involve:"
echo "1. Adding a WebSocket handler at /ws endpoint"
echo "2. Implementing message routing for signals"
echo "3. Managing WebSocket connections and subscriptions"