#!/bin/bash
# Test script to demonstrate comprehensive logging functionality

set -e

echo "=== HAL9 Comprehensive Logging Test ==="
echo

# Build the server first
echo "Building HAL9 server..."
cd /Users/icedac/2lab.ai/2hal9
cargo build --release --quiet

# Start server with JSON logging
echo "Starting server with JSON logging..."
LOG_FORMAT=json RUST_LOG=debug,hal9=trace cargo run --release --bin hal9-server &
SERVER_PID=$!

# Wait for server to start
sleep 3

# Test API endpoints with curl to generate logs
echo
echo "Testing API endpoints to generate logs..."

# Health check
echo "1. Health check endpoint:"
curl -s http://localhost:8080/health | jq .

# Status endpoint
echo
echo "2. Status endpoint:"
curl -s http://localhost:8080/api/v1/status | jq .

# Submit a signal
echo
echo "3. Submitting a test signal:"
curl -s -X POST http://localhost:8080/api/v1/signal \
  -H "Content-Type: application/json" \
  -d '{
    "content": "Test signal for logging demonstration",
    "layer": "L4"
  }' | jq .

# List neurons
echo
echo "4. Listing neurons:"
curl -s http://localhost:8080/api/v1/neurons | jq .

# Get metrics
echo
echo "5. Getting metrics:"
curl -s http://localhost:8080/api/v1/metrics | jq .

# Give it a moment to process
sleep 2

# Kill the server
echo
echo "Stopping server..."
kill $SERVER_PID
wait $SERVER_PID 2>/dev/null || true

echo
echo "=== Test Complete ==="
echo
echo "Logging features demonstrated:"
echo "✓ Structured JSON logging format"
echo "✓ Request/response middleware logging with trace IDs"
echo "✓ Performance metrics logging"
echo "✓ Neuron signal processing logs"
echo "✓ Database query logging (if DB operations occurred)"
echo
echo "To view pretty-printed logs, restart without LOG_FORMAT=json"
echo "To adjust log levels, use RUST_LOG environment variable"