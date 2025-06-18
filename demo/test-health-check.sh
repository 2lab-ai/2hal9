#!/bin/bash
# Test enhanced health check functionality

echo "🏥 Testing HAL9 Enhanced Health Check"
echo "===================================="
echo

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Base URL
BASE_URL=${1:-http://localhost:8080}

# Check if server is running
echo "Checking if HAL9 server is running..."
if ! curl -s $BASE_URL/health > /dev/null 2>&1; then
    echo -e "${RED}❌ HAL9 server is not running!${NC}"
    echo "Please start the server first with: cargo run --release --bin hal9-server"
    exit 1
fi

echo -e "${GREEN}✓ Server is running${NC}"
echo

# Test simple health check
echo -e "${BLUE}1. Simple Health Check${NC}"
echo "   Endpoint: /health"
echo "   Response:"
curl -s $BASE_URL/health | jq '.' || curl -s $BASE_URL/health
echo

# Test liveness probe
echo -e "${BLUE}2. Kubernetes Liveness Probe${NC}"
echo "   Endpoint: /livez"
RESPONSE=$(curl -s -w "\n%{http_code}" $BASE_URL/livez)
HTTP_CODE=$(echo "$RESPONSE" | tail -n1)
BODY=$(echo "$RESPONSE" | head -n-1)
echo "   Status: $HTTP_CODE"
echo "   Response: $BODY"
echo

# Test readiness probe
echo -e "${BLUE}3. Kubernetes Readiness Probe${NC}"
echo "   Endpoint: /readyz"
RESPONSE=$(curl -s -w "\n%{http_code}" $BASE_URL/readyz)
HTTP_CODE=$(echo "$RESPONSE" | tail -n1)
BODY=$(echo "$RESPONSE" | head -n-1)
echo "   Status: $HTTP_CODE"
echo "   Response: $BODY"
echo

# Test detailed health check
echo -e "${BLUE}4. Detailed Health Check${NC}"
echo "   Endpoint: /health/detailed?detailed=true"
echo "   Response:"
curl -s "$BASE_URL/health/detailed?detailed=true" | jq '.' || curl -s "$BASE_URL/health/detailed?detailed=true"
echo

# Test health check with timeout
echo -e "${BLUE}5. Health Check with Custom Timeout${NC}"
echo "   Endpoint: /health/detailed?detailed=true&timeout_ms=1000"
echo "   Response:"
curl -s "$BASE_URL/health/detailed?detailed=true&timeout_ms=1000" | jq '.status, .checks_passed, .checks_total' 2>/dev/null || echo "Not available"
echo

# Performance test
echo -e "${BLUE}6. Health Check Performance Test${NC}"
echo "   Running 100 health checks..."

TOTAL_TIME=0
SUCCESS_COUNT=0
FAILED_COUNT=0

for i in {1..100}; do
    START_TIME=$(date +%s.%N)
    HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" $BASE_URL/health)
    END_TIME=$(date +%s.%N)
    
    ELAPSED=$(echo "$END_TIME - $START_TIME" | bc)
    TOTAL_TIME=$(echo "$TOTAL_TIME + $ELAPSED" | bc)
    
    if [ "$HTTP_CODE" = "200" ]; then
        ((SUCCESS_COUNT++))
    else
        ((FAILED_COUNT++))
    fi
    
    # Show progress
    echo -ne "\r   Progress: $i/100"
done

AVG_TIME=$(echo "scale=3; $TOTAL_TIME / 100" | bc)
echo -e "\r   ${GREEN}✓ Complete${NC}                    "
echo "   Average response time: ${AVG_TIME}s"
echo "   Success rate: $SUCCESS_COUNT/100"
if [ $FAILED_COUNT -gt 0 ]; then
    echo -e "   ${RED}Failed requests: $FAILED_COUNT${NC}"
fi
echo

# Summary
echo -e "${YELLOW}Summary:${NC}"
echo "- Simple health check: Fast endpoint for basic monitoring"
echo "- Liveness probe: Kubernetes will restart pod if this fails"
echo "- Readiness probe: Kubernetes won't route traffic if this fails"
echo "- Detailed health: Complete system health with all components"
echo
echo "Use these endpoints in your monitoring tools:"
echo "- Prometheus: $BASE_URL/health/detailed?detailed=true"
echo "- Kubernetes: livez and readyz endpoints"
echo "- Load balancer: $BASE_URL/health"