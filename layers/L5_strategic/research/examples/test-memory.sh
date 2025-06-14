#!/bin/bash
# Test script for HAL9 persistent memory system

echo "🧠 HAL9 Memory System Test"
echo "========================="
echo

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Create data directory
mkdir -p ./data

# Test 1: Start server with memory configuration
echo -e "${YELLOW}Test 1: Starting HAL9 with memory system...${NC}"
cargo run --bin hal9-server -- --config examples/memory-test.yaml &
SERVER_PID=$!

# Give server time to start and initialize database
sleep 5

# Test 2: Send first signal to create memories
echo -e "${YELLOW}Test 2: Sending first signal to create memories...${NC}"
RESPONSE1=$(curl -s -X POST http://localhost:9736/api/signal \
  -H "Content-Type: application/json" \
  -d '{
    "signal": {
      "from_neuron": "test-client",
      "to_neuron": "strategic-memory",
      "layer_from": "client",
      "layer_to": "L4",
      "content": "Design a user authentication system"
    }
  }')

echo "$RESPONSE1" | jq .
SIGNAL_ID1=$(echo "$RESPONSE1" | jq -r '.signal.signal_id')

# Wait for processing
sleep 2

# Test 3: Send similar signal to test memory recall
echo
echo -e "${YELLOW}Test 3: Sending similar signal to test memory recall...${NC}"
RESPONSE2=$(curl -s -X POST http://localhost:9736/api/signal \
  -H "Content-Type: application/json" \
  -d '{
    "signal": {
      "from_neuron": "test-client",
      "to_neuron": "strategic-memory",
      "layer_from": "client",
      "layer_to": "L4",
      "content": "Design a secure login system"
    }
  }')

echo "$RESPONSE2" | jq .

# Test 4: Check if memories are being stored
echo
echo -e "${YELLOW}Test 4: Checking database for stored memories...${NC}"
if [ -f "./data/hal9_memory.db" ]; then
    echo -e "${GREEN}✓ Memory database created${NC}"
    
    # Count memories (requires sqlite3)
    if command -v sqlite3 &> /dev/null; then
        MEMORY_COUNT=$(sqlite3 ./data/hal9_memory.db "SELECT COUNT(*) FROM memories;" 2>/dev/null || echo "0")
        echo -e "${BLUE}Total memories stored: $MEMORY_COUNT${NC}"
        
        # Show recent memories
        echo -e "\n${BLUE}Recent memory entries:${NC}"
        sqlite3 ./data/hal9_memory.db "SELECT neuron_id, entry_type, substr(content, 1, 50) || '...' as content_preview FROM memories ORDER BY timestamp DESC LIMIT 5;" 2>/dev/null || echo "Could not read memories"
    fi
else
    echo "✗ Memory database not found"
fi

# Test 5: Send error signal to test error memory
echo
echo -e "${YELLOW}Test 5: Sending error-inducing signal...${NC}"
curl -s -X POST http://localhost:9736/api/signal \
  -H "Content-Type: application/json" \
  -d '{
    "signal": {
      "from_neuron": "test-client",
      "to_neuron": "strategic-memory",
      "layer_from": "client", 
      "layer_to": "L4",
      "content": "INVALID_TASK: @#$%^&*()"
    }
  }' | jq .

# Test 6: Check neuron health with memory
echo
echo -e "${YELLOW}Test 6: Checking neuron health...${NC}"
curl -s http://localhost:9736/api/neurons | jq .

# Test 7: Memory search test (future feature)
echo
echo -e "${YELLOW}Test 7: Testing memory search capabilities...${NC}"
echo -e "${BLUE}Note: Semantic search will be available in future updates${NC}"

# Cleanup
echo
echo -e "${YELLOW}Cleaning up...${NC}"
kill $SERVER_PID 2>/dev/null
wait $SERVER_PID 2>/dev/null

echo
echo -e "${GREEN}✓ Memory system tests completed!${NC}"
echo
echo "The persistent memory system provides:"
echo "- Task and result storage for each neuron"
echo "- Memory context in prompts for better decision making"
echo "- Learning from past experiences and errors"
echo "- Foundation for future semantic search capabilities"
echo
echo "Memory database location: ./data/hal9_memory.db"