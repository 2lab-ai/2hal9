#!/bin/bash
# Database Integration Test Script

set -e

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo "🗄️ Testing Database Integration"
echo "==============================="

# PostgreSQL Tests
echo -e "\n${YELLOW}PostgreSQL Tests${NC}"
echo "----------------"

# Test connection
echo -n "Testing PostgreSQL connection... "
if docker-compose exec -T postgres psql -U hal9 -d hal9db -c "SELECT 1;" > /dev/null 2>&1; then
    echo -e "${GREEN}✅ Connected${NC}"
else
    echo -e "${RED}❌ Failed${NC}"
    exit 1
fi

# Create test table
echo -n "Creating test table... "
docker-compose exec -T postgres psql -U hal9 -d hal9db << EOF > /dev/null 2>&1
CREATE TABLE IF NOT EXISTS test_integration (
    id SERIAL PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
EOF
echo -e "${GREEN}✅ Created${NC}"

# Insert test data
echo -n "Inserting test data... "
docker-compose exec -T postgres psql -U hal9 -d hal9db << EOF > /dev/null 2>&1
INSERT INTO test_integration (name) VALUES
    ('Test Entry 1'),
    ('Test Entry 2'),
    ('Test Entry 3');
EOF
echo -e "${GREEN}✅ Inserted${NC}"

# Query test data
echo -n "Querying test data... "
result=$(docker-compose exec -T postgres psql -U hal9 -d hal9db -t -c "SELECT COUNT(*) FROM test_integration;" | tr -d ' ')
if [ "$result" = "3" ]; then
    echo -e "${GREEN}✅ Found 3 records${NC}"
else
    echo -e "${RED}❌ Expected 3 records, found $result${NC}"
fi

# Check existing tables
echo -e "\n${YELLOW}Existing Tables:${NC}"
docker-compose exec -T postgres psql -U hal9 -d hal9db -c "\dt" | grep -E "public\.|Name" || echo "No tables found"

# Clean up test table
echo -n "Cleaning up test table... "
docker-compose exec -T postgres psql -U hal9 -d hal9db -c "DROP TABLE IF EXISTS test_integration;" > /dev/null 2>&1
echo -e "${GREEN}✅ Cleaned${NC}"

# Redis Tests
echo -e "\n${YELLOW}Redis Tests${NC}"
echo "-----------"

# Test connection
echo -n "Testing Redis connection... "
if docker-compose exec -T redis redis-cli ping | grep -q "PONG"; then
    echo -e "${GREEN}✅ Connected${NC}"
else
    echo -e "${RED}❌ Failed${NC}"
    exit 1
fi

# Set test value
echo -n "Setting test value... "
docker-compose exec -T redis redis-cli SET test:key "Hello HAL9" > /dev/null 2>&1
echo -e "${GREEN}✅ Set${NC}"

# Get test value
echo -n "Getting test value... "
value=$(docker-compose exec -T redis redis-cli GET test:key | tr -d '\r')
if [ "$value" = "Hello HAL9" ]; then
    echo -e "${GREEN}✅ Retrieved correctly${NC}"
else
    echo -e "${RED}❌ Expected 'Hello HAL9', got '$value'${NC}"
fi

# Test expiration
echo -n "Testing key expiration... "
docker-compose exec -T redis redis-cli SETEX test:expire 2 "temporary" > /dev/null 2>&1
sleep 3
expired=$(docker-compose exec -T redis redis-cli GET test:expire | tr -d '\r')
if [ -z "$expired" ] || [ "$expired" = "(nil)" ]; then
    echo -e "${GREEN}✅ Key expired correctly${NC}"
else
    echo -e "${RED}❌ Key should have expired${NC}"
fi

# Test pub/sub
echo -n "Testing pub/sub... "
docker-compose exec -T redis redis-cli PUBLISH test:channel "test message" > /dev/null 2>&1
echo -e "${GREEN}✅ Published${NC}"

# Clean up
echo -n "Cleaning up Redis test data... "
docker-compose exec -T redis redis-cli DEL test:key > /dev/null 2>&1
echo -e "${GREEN}✅ Cleaned${NC}"

# Check Redis info
echo -e "\n${YELLOW}Redis Info:${NC}"
docker-compose exec -T redis redis-cli INFO server | grep -E "redis_version|uptime_in_seconds" | head -2

# Test from HAL9 server
echo -e "\n${YELLOW}Testing Database Access from HAL9 Server${NC}"
echo "---------------------------------------"

# Check if server can resolve database hosts
echo -n "Checking PostgreSQL hostname resolution... "
if docker-compose exec -T hal9-server ping -c 1 postgres > /dev/null 2>&1; then
    echo -e "${GREEN}✅ Can reach postgres host${NC}"
else
    echo -e "${RED}❌ Cannot reach postgres host${NC}"
fi

echo -n "Checking Redis hostname resolution... "
if docker-compose exec -T hal9-server ping -c 1 redis > /dev/null 2>&1; then
    echo -e "${GREEN}✅ Can reach redis host${NC}"
else
    echo -e "${RED}❌ Cannot reach redis host${NC}"
fi

echo -e "\n${GREEN}Database Integration Tests Complete!${NC}"
echo "Both PostgreSQL and Redis are properly configured and accessible."