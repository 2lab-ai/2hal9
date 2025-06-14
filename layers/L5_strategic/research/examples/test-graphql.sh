#!/bin/bash
# Test script for HAL9 GraphQL API v2

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}Testing HAL9 GraphQL API v2...${NC}"

# Configuration
GRAPHQL_ENDPOINT="http://localhost:9000/graphql"
AUTH_TOKEN="${HAL9_AUTH_TOKEN:-test-token}"

# Helper function for GraphQL requests
graphql_request() {
    local query="$1"
    local variables="${2:-{}}"
    
    curl -s -X POST "$GRAPHQL_ENDPOINT" \
        -H "Content-Type: application/json" \
        -H "Authorization: Bearer $AUTH_TOKEN" \
        -d "{
            \"query\": \"$query\",
            \"variables\": $variables
        }" | jq .
}

# Test 1: Check GraphQL endpoint health
echo -e "\n${GREEN}Test 1: GraphQL Introspection${NC}"
graphql_request '
{
  __schema {
    queryType { name }
    mutationType { name }
    subscriptionType { name }
  }
}'

# Test 2: Get system metrics
echo -e "\n${GREEN}Test 2: System Metrics Query${NC}"
graphql_request '
query SystemStatus {
  systemMetrics {
    totalNeurons
    activeNeurons
    signalsProcessed
    averageResponseTimeMs
    memoryUsageMb
    cpuUsagePercent
  }
}'

# Test 3: List neurons
echo -e "\n${GREEN}Test 3: List Neurons with Pagination${NC}"
graphql_request '
query ListNeurons($pagination: PaginationInput) {
  neurons(pagination: $pagination) {
    edges {
      node {
        id
        name
        layer
        state
        metrics {
          processedCount
          successRate
        }
      }
    }
    pageInfo {
      hasNextPage
      totalCount
    }
  }
}' '{"pagination": {"limit": 5}}'

# Test 4: Send a signal
echo -e "\n${GREEN}Test 4: Send Signal Mutation${NC}"
graphql_request '
mutation SendTestSignal($input: SignalInput!) {
  sendSignal(input: $input) {
    id
    content
    layer
    status
    createdAt
  }
}' '{
  "input": {
    "content": "Test signal from GraphQL API",
    "layer": "L3",
    "priority": 5
  }
}'

# Test 5: Search memory
echo -e "\n${GREEN}Test 5: Memory Search Query${NC}"
graphql_request '
query SearchMemory($query: String!) {
  searchMemory(query: $query, limit: 3) {
    id
    key
    content
    embeddingSimilarity
  }
}' '{"query": "distributed systems"}'

# Test 6: Complex nested query
echo -e "\n${GREEN}Test 6: Complex Nested Query${NC}"
graphql_request '
query ComplexQuery {
  me {
    id
    email
    permissions
  }
  
  systemMetrics {
    totalNeurons
    signalsProcessed
  }
  
  neurons(layer: "L4", pagination: {limit: 2}) {
    edges {
      node {
        id
        name
        config
      }
    }
  }
}'

# Test 7: Test error handling
echo -e "\n${GREEN}Test 7: Error Handling (Invalid Query)${NC}"
graphql_request '
query InvalidQuery {
  nonExistentField
}'

# Test WebSocket subscription (requires wscat)
if command -v wscat &> /dev/null; then
    echo -e "\n${GREEN}Test 8: WebSocket Subscription Test${NC}"
    echo "Testing subscription connection..."
    
    # Create a test subscription
    cat > /tmp/subscription.json << EOF
{
  "id": "1",
  "type": "start",
  "payload": {
    "query": "subscription { signalUpdates { signalId status message timestamp } }"
  }
}
EOF
    
    echo "Subscription test requires manual verification."
    echo "Run: wscat -c ws://localhost:9000/graphql/ws -H \"Authorization: Bearer $AUTH_TOKEN\""
    echo "Then send: $(cat /tmp/subscription.json)"
else
    echo -e "\n${BLUE}Skipping WebSocket test (wscat not installed)${NC}"
fi

# Test GraphQL Playground
echo -e "\n${GREEN}Test 9: GraphQL Playground Availability${NC}"
PLAYGROUND_STATUS=$(curl -s -o /dev/null -w "%{http_code}" http://localhost:9000/graphql/playground)
if [ "$PLAYGROUND_STATUS" = "200" ]; then
    echo -e "${GREEN}✓ GraphQL Playground is available at http://localhost:9000/graphql/playground${NC}"
else
    echo -e "${RED}✗ GraphQL Playground returned status: $PLAYGROUND_STATUS${NC}"
fi

echo -e "\n${BLUE}GraphQL API tests completed!${NC}"
echo -e "${GREEN}For interactive testing, visit: http://localhost:9000/graphql/playground${NC}"