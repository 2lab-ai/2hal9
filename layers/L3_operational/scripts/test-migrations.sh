#!/bin/bash
# Test database migrations

set -euo pipefail

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${GREEN}🧪 Testing HAL9 Database Migrations${NC}"
echo "======================================"
echo ""

# Test SQLite migrations
echo -e "${GREEN}Testing SQLite migrations...${NC}"
TEST_DB="/tmp/hal9_test_$(date +%s).db"
./run-migrations.sh --type sqlite --url "sqlite:$TEST_DB"

echo ""
echo "Verifying SQLite tables..."
sqlite3 "$TEST_DB" ".tables" | tr ' ' '\n' | sort

echo ""
echo "Checking game tables..."
sqlite3 "$TEST_DB" "SELECT name FROM sqlite_master WHERE type='table' AND name LIKE 'game%' ORDER BY name;"

# Clean up
rm -f "$TEST_DB"

echo ""
echo -e "${GREEN}✅ SQLite migrations passed!${NC}"

# Test PostgreSQL if available
if command -v psql &> /dev/null; then
    echo ""
    echo -e "${GREEN}Testing PostgreSQL migrations...${NC}"
    
    # Check if PostgreSQL is running
    if pg_isready -q 2>/dev/null; then
        TEST_DB="hal9_test_$(date +%s)"
        createdb "$TEST_DB" 2>/dev/null || true
        
        ./run-migrations.sh --type postgres --url "postgresql://localhost/$TEST_DB"
        
        echo ""
        echo "Verifying PostgreSQL tables..."
        psql -d "$TEST_DB" -c "\dt" | grep -E "(games|game_players|achievements|leaderboard)"
        
        # Clean up
        dropdb "$TEST_DB" 2>/dev/null || true
        
        echo ""
        echo -e "${GREEN}✅ PostgreSQL migrations passed!${NC}"
    else
        echo -e "${YELLOW}⚠️  PostgreSQL is not running, skipping tests${NC}"
    fi
else
    echo ""
    echo -e "${YELLOW}⚠️  PostgreSQL not installed, skipping tests${NC}"
fi

echo ""
echo -e "${GREEN}🎉 All migration tests completed!${NC}"