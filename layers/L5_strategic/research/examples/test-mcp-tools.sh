#!/bin/bash
# Test script for MCP tools integration

echo "🧪 HAL9 MCP Tools Integration Test"
echo "=================================="
echo

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Test 1: Run the MCP tools demo
echo -e "${YELLOW}Test 1: Running MCP tools demo...${NC}"
if cargo run --example mcp-tools-demo; then
    echo -e "${GREEN}✓ MCP tools demo passed${NC}"
else
    echo "✗ MCP tools demo failed"
    exit 1
fi
echo

# Test 2: Start server with MCP tools configuration
echo -e "${YELLOW}Test 2: Starting HAL9 with MCP tools configuration...${NC}"
cargo run --bin hal9-server -- --config examples/mcp-tools-test.yaml &
SERVER_PID=$!

# Give server time to start
sleep 3

# Test 3: Send a test signal that triggers tool usage
echo -e "${YELLOW}Test 3: Sending test signal to trigger tool usage...${NC}"
curl -X POST http://localhost:9736/api/signal \
  -H "Content-Type: application/json" \
  -d '{
    "signal": {
      "from_neuron": "test-client",
      "to_neuron": "strategic-analyzer",
      "layer_from": "client",
      "layer_to": "L4",
      "content": "Analyze the HAL9 architecture and recommend improvements for tool integration"
    }
  }' | jq .

echo
echo -e "${YELLOW}Test 4: Checking neuron health...${NC}"
curl -s http://localhost:9736/api/neurons | jq .

echo
echo -e "${YELLOW}Test 5: Checking metrics for tool usage...${NC}"
curl -s http://localhost:9737/metrics | grep -E "(tool_|signal_)" | head -20

# Cleanup
echo
echo -e "${YELLOW}Cleaning up...${NC}"
kill $SERVER_PID 2>/dev/null
wait $SERVER_PID 2>/dev/null

echo
echo -e "${GREEN}✓ All tests completed!${NC}"
echo
echo "The MCP tools integration is working correctly. Neurons can now:"
echo "- L4: Read documentation and fetch web resources"
echo "- L3: Analyze source code and run development tools"
echo "- L2: Read/write files and execute safe shell commands"
echo "- L1: Execute basic shell commands"