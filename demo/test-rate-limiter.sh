#!/bin/bash
# Test rate limiting functionality

echo "🚦 Testing HAL9 Rate Limiter"
echo "========================="
echo

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check if server is running
echo "Checking if HAL9 server is running..."
if ! curl -s http://localhost:8080/health > /dev/null 2>&1; then
    echo -e "${RED}❌ HAL9 server is not running!${NC}"
    echo "Please start the server first with: cargo run --release --bin hal9-server"
    exit 1
fi

echo -e "${GREEN}✓ Server is running${NC}"
echo

# Test endpoint
ENDPOINT="http://localhost:8080/api/v1/status"

# Default rate limit is 60 requests per minute
echo "Testing rate limiting (default: 60 requests/minute)..."
echo "Making 65 requests rapidly..."
echo

# Make requests and count successes/failures
SUCCESS=0
RATE_LIMITED=0

for i in {1..65}; do
    RESPONSE=$(curl -s -w "\n%{http_code}" $ENDPOINT 2>/dev/null)
    HTTP_CODE=$(echo "$RESPONSE" | tail -n1)
    
    if [ "$HTTP_CODE" = "200" ]; then
        ((SUCCESS++))
        echo -ne "\r${GREEN}✓${NC} Success: $SUCCESS | ${RED}⚡${NC} Rate Limited: $RATE_LIMITED"
    elif [ "$HTTP_CODE" = "429" ]; then
        ((RATE_LIMITED++))
        echo -ne "\r${GREEN}✓${NC} Success: $SUCCESS | ${RED}⚡${NC} Rate Limited: $RATE_LIMITED"
        
        # Check for Retry-After header
        if [ $RATE_LIMITED -eq 1 ]; then
            echo
            RETRY_AFTER=$(curl -s -I $ENDPOINT | grep -i "Retry-After" | cut -d' ' -f2 | tr -d '\r')
            if [ ! -z "$RETRY_AFTER" ]; then
                echo -e "\n${YELLOW}Rate limit hit! Retry-After: ${RETRY_AFTER} seconds${NC}"
            fi
        fi
    else
        echo -e "\n${RED}Unexpected response code: $HTTP_CODE${NC}"
    fi
done

echo
echo
echo "Summary:"
echo "--------"
echo -e "${GREEN}Successful requests: $SUCCESS${NC}"
echo -e "${RED}Rate limited requests: $RATE_LIMITED${NC}"
echo

# Test with different IPs (using X-Forwarded-For header)
echo "Testing with different client IPs..."
echo

IP1_SUCCESS=0
IP2_SUCCESS=0

# Make 5 requests from each "IP"
for i in {1..5}; do
    # IP 1
    RESPONSE=$(curl -s -w "\n%{http_code}" -H "X-Forwarded-For: 192.168.1.100" $ENDPOINT 2>/dev/null)
    HTTP_CODE=$(echo "$RESPONSE" | tail -n1)
    [ "$HTTP_CODE" = "200" ] && ((IP1_SUCCESS++))
    
    # IP 2
    RESPONSE=$(curl -s -w "\n%{http_code}" -H "X-Forwarded-For: 192.168.1.200" $ENDPOINT 2>/dev/null)
    HTTP_CODE=$(echo "$RESPONSE" | tail -n1)
    [ "$HTTP_CODE" = "200" ] && ((IP2_SUCCESS++))
done

echo -e "IP 192.168.1.100: ${GREEN}$IP1_SUCCESS successful requests${NC}"
echo -e "IP 192.168.1.200: ${GREEN}$IP2_SUCCESS successful requests${NC}"
echo

# Test token refill
echo "Testing token refill..."
echo "Waiting for rate limit window to reset (60 seconds)..."

# Show countdown
for i in {60..1}; do
    echo -ne "\r${YELLOW}Time remaining: $i seconds${NC}  "
    sleep 1
done
echo

# Try again after window reset
echo "Making new request after window reset..."
RESPONSE=$(curl -s -w "\n%{http_code}" $ENDPOINT 2>/dev/null)
HTTP_CODE=$(echo "$RESPONSE" | tail -n1)

if [ "$HTTP_CODE" = "200" ]; then
    echo -e "${GREEN}✓ Request successful - tokens refilled!${NC}"
else
    echo -e "${RED}✗ Request failed with code: $HTTP_CODE${NC}"
fi

echo
echo "Rate limiter test complete!"