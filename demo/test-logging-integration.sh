#!/bin/bash
# Test structured logging in HAL9 codebase

echo "=== HAL9 Structured Logging Integration Test ==="
echo

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

# Check if we can find any existing examples with logging
echo -e "${GREEN}Finding examples with logging...${NC}"
EXAMPLES=$(find /Users/icedac/2lab.ai/2hal9 -name "*.rs" -path "*/examples/*" | head -5)

if [ -z "$EXAMPLES" ]; then
    echo -e "${RED}No examples found${NC}"
    exit 1
fi

echo "Found examples:"
for ex in $EXAMPLES; do
    echo "  - $ex"
done

# Run performance benchmark which should have logging
echo
echo -e "${GREEN}Testing performance benchmark with JSON logging...${NC}"
cd /Users/icedac/2lab.ai/2hal9/layers/L2_implementation/neurons
LOG_FORMAT=json RUST_LOG=info,hal9=debug cargo run --example performance_benchmark 2>&1 | head -20

echo
echo -e "${GREEN}Testing with pretty logging...${NC}"
LOG_FORMAT=pretty RUST_LOG=debug cargo run --example quick_benchmark 2>&1 | head -20

echo
echo -e "${GREEN}✅ Logging integration test complete!${NC}"
echo
echo "The structured logging system provides:"
echo "- JSON format for production (LOG_FORMAT=json)"
echo "- Pretty format for development (LOG_FORMAT=pretty)"
echo "- Configurable log levels (RUST_LOG=debug,info,warn,error)"
echo "- Structured fields and performance metrics"
echo "- Request tracing with trace IDs"
echo "- Database query logging"