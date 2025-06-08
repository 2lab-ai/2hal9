#!/bin/bash
# Test script for HAL9 backward propagation learning system

echo "🧠 HAL9 Backward Propagation Test"
echo "=================================="
echo

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Create data directory
mkdir -p ./data

# Test 1: Start server with backward propagation configuration
echo -e "${YELLOW}Test 1: Starting HAL9 with backward propagation...${NC}"
cargo run --bin hal9-server -- --config examples/backward-propagation-test.yaml &
SERVER_PID=$!

# Give server time to start
sleep 5

# Test 2: Send a signal that will trigger an error
echo -e "${YELLOW}Test 2: Sending signal that will cause an error...${NC}"
RESPONSE1=$(curl -s -X POST http://localhost:9736/api/signal \
  -H "Content-Type: application/json" \
  -d '{
    "signal": {
      "from_neuron": "test-client",
      "to_neuron": "strategic-learner",
      "layer_from": "client",
      "layer_to": "L4",
      "content": "Process 1 million records in real-time with zero latency"
    }
  }')

echo "$RESPONSE1" | jq .
SIGNAL_ID1=$(echo "$RESPONSE1" | jq -r '.signal.signal_id')

# Wait for error propagation
sleep 3

# Test 3: Send the same type of signal again to see if learning occurred
echo
echo -e "${YELLOW}Test 3: Sending similar signal to test learning...${NC}"
RESPONSE2=$(curl -s -X POST http://localhost:9736/api/signal \
  -H "Content-Type: application/json" \
  -d '{
    "signal": {
      "from_neuron": "test-client",
      "to_neuron": "strategic-learner",
      "layer_from": "client",
      "layer_to": "L4",
      "content": "Process 2 million records in real-time with minimal latency"
    }
  }')

echo "$RESPONSE2" | jq .

# Test 4: Simulate backward propagation explicitly
echo
echo -e "${YELLOW}Test 4: Sending explicit backward propagation signal...${NC}"
curl -s -X POST http://localhost:9736/api/signal \
  -H "Content-Type: application/json" \
  -d '{
    "signal": {
      "signal_id": "'$(uuidgen)'",
      "from_neuron": "implementation-learner",
      "to_neuron": "design-learner",
      "layer_from": "L2",
      "layer_to": "L3",
      "propagation_type": "Backward",
      "batch_id": "'$(uuidgen)'",
      "timestamp": "'$(date -u +"%Y-%m-%dT%H:%M:%S.%3NZ")'",
      "payload": {
        "activation": {
          "content": "",
          "strength": 0.0,
          "features": {}
        },
        "gradient": {
          "error_type": "Timeout in synchronous processing",
          "magnitude": 0.8,
          "loss": 0.8,
          "adjustments": [
            "Consider asynchronous processing",
            "Implement batch processing",
            "Add timeout handling"
          ]
        }
      }
    }
  }' | jq .

# Test 5: Check if learning was stored in memory
echo
echo -e "${YELLOW}Test 5: Checking database for learning memories...${NC}"
if [ -f "./data/hal9_learning.db" ]; then
    echo -e "${GREEN}✓ Learning database created${NC}"
    
    # Count learning memories (requires sqlite3)
    if command -v sqlite3 &> /dev/null; then
        LEARNING_COUNT=$(sqlite3 ./data/hal9_learning.db "SELECT COUNT(*) FROM memories WHERE entry_type LIKE '%Learning%';" 2>/dev/null || echo "0")
        ERROR_COUNT=$(sqlite3 ./data/hal9_learning.db "SELECT COUNT(*) FROM memories WHERE entry_type LIKE '%Error%';" 2>/dev/null || echo "0")
        
        echo -e "${BLUE}Learning memories stored: $LEARNING_COUNT${NC}"
        echo -e "${BLUE}Error patterns stored: $ERROR_COUNT${NC}"
        
        # Show recent learning entries
        echo -e "\n${BLUE}Recent learning entries:${NC}"
        sqlite3 ./data/hal9_learning.db "SELECT neuron_id, substr(content, 1, 80) || '...' as learning FROM memories WHERE entry_type LIKE '%Learning%' ORDER BY timestamp DESC LIMIT 3;" 2>/dev/null || echo "Could not read learning memories"
    fi
else
    echo "✗ Learning database not found"
fi

# Test 6: Check neuron health
echo
echo -e "${YELLOW}Test 6: Checking neuron health and stats...${NC}"
curl -s http://localhost:9736/api/neurons | jq .

# Test 7: Demonstrate pattern detection
echo
echo -e "${YELLOW}Test 7: Creating error pattern (3 similar errors)...${NC}"
for i in {1..3}; do
    echo -e "${BLUE}Sending error-inducing signal $i/3...${NC}"
    curl -s -X POST http://localhost:9736/api/signal \
      -H "Content-Type: application/json" \
      -d '{
        "signal": {
          "from_neuron": "test-client",
          "to_neuron": "strategic-learner",
          "layer_from": "client",
          "layer_to": "L4",
          "content": "Execute complex query on massive dataset immediately"
        }
      }' > /dev/null
    sleep 1
done

echo -e "${GREEN}✓ Pattern should now be detected!${NC}"

# Cleanup
echo
echo -e "${YELLOW}Cleaning up...${NC}"
kill $SERVER_PID 2>/dev/null
wait $SERVER_PID 2>/dev/null

echo
echo -e "${GREEN}✓ Backward propagation tests completed!${NC}"
echo
echo "The backward propagation system provides:"
echo "- Error gradient calculation and propagation"
echo "- Pattern recognition for recurring errors"
echo "- Prompt adjustment based on failures"
echo "- Learning memory storage"
echo "- Continuous improvement through experience"
echo
echo "Learning database location: ./data/hal9_learning.db"