#!/bin/bash

echo "🚀 Testing 2HAL9 Claude API Integration"
echo "======================================="

# Check if API key is set
if [ -z "$ANTHROPIC_API_KEY" ]; then
    echo "⚠️  Warning: ANTHROPIC_API_KEY not set. API mode will fail and fallback to mock."
    echo "   To test real API, run: export ANTHROPIC_API_KEY='your-key-here'"
    echo ""
fi

# Start server in API mode with fallback
echo "Starting server with API configuration..."
cargo run --bin hal9-server -- examples/config-api-with-fallback.yaml &
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

# Test metrics to see initial state
echo -e "\n📊 Initial metrics..."
curl -s http://localhost:8080/api/v1/metrics | jq '.data | {tokens_total, memory_usage_mb}'

# Submit a test signal
echo -e "\n🚦 Submitting test signal..."
RESPONSE=$(curl -s -X POST http://localhost:8080/api/v1/signal \
  -H "Content-Type: application/json" \
  -d '{
    "content": "Create a Python function to calculate fibonacci numbers",
    "layer": "L4"
  }')

echo "$RESPONSE" | jq .

# Wait for processing
echo -e "\n⏳ Waiting for signal processing..."
sleep 5

# Check metrics after processing
echo -e "\n📊 Metrics after processing..."
METRICS=$(curl -s http://localhost:8080/api/v1/metrics)
echo "$METRICS" | jq '.data | {
  signals_processed,
  tokens_total,
  tokens_prompt,
  tokens_completion,
  processing_times: .processing_times | to_entries | map({key: .key, avg_ms: .value.avg_ms})
}'

# Check if we're in fallback mode
echo -e "\n🔍 Checking server logs for mode..."
echo "Recent server logs:"
echo "==================="
# The server logs will show if we're using API or fallback mode

# Test with intentionally bad API key to trigger fallback
if [ -n "$ANTHROPIC_API_KEY" ]; then
    echo -e "\n🧪 Testing fallback mechanism..."
    echo "Temporarily setting invalid API key to trigger fallback..."
    
    # Kill current server
    kill $SERVER_PID
    wait $SERVER_PID 2>/dev/null
    
    # Start with bad API key
    ANTHROPIC_API_KEY="invalid-key" cargo run --bin hal9-server -- examples/config-api-with-fallback.yaml &
    SERVER_PID=$!
    sleep 5
    
    # Send signal
    echo "Sending signal with invalid API key..."
    curl -s -X POST http://localhost:8080/api/v1/signal \
      -H "Content-Type: application/json" \
      -d '{
        "content": "Test fallback mechanism",
        "layer": "L4"
      }' | jq .
    
    sleep 3
    
    # Check if fallback was triggered
    echo -e "\n📊 Checking if fallback mode was activated..."
    curl -s http://localhost:8080/api/v1/metrics | jq '.data | {signals_processed, errors_by_type}'
fi

# Kill the server
echo -e "\n🛑 Stopping server..."
kill $SERVER_PID
wait $SERVER_PID 2>/dev/null

echo -e "\n✅ Test completed!"
echo ""
echo "💡 Tips:"
echo "   - Set ANTHROPIC_API_KEY to test real API integration"
echo "   - Check server logs for detailed API/fallback information"
echo "   - Monitor metrics endpoint for token usage and costs"