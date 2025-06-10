#!/bin/bash

# Test script for HAL9 Performance Optimizations

set -e

echo "🚀 HAL9 Performance Optimization Test"
echo "===================================="

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Helper functions
log_test() {
    echo -e "${YELLOW}Test: $1${NC}"
}

log_success() {
    echo -e "${GREEN}✓ $1${NC}"
}

log_error() {
    echo -e "${RED}✗ $1${NC}"
}

log_info() {
    echo -e "${BLUE}ℹ $1${NC}"
}

# Check dependencies
log_test "Checking dependencies..."

# Check if k6 is installed
if ! command -v k6 &> /dev/null; then
    log_info "k6 not found. Installing..."
    if [[ "$OSTYPE" == "darwin"* ]]; then
        brew install k6
    else
        sudo apt-key adv --keyserver hkp://keyserver.ubuntu.com:80 --recv-keys C5AD17C747E3415A3642D57D77C6C491D6AC1D69
        echo "deb https://dl.k6.io/deb stable main" | sudo tee /etc/apt/sources.list.d/k6.list
        sudo apt-get update
        sudo apt-get install k6
    fi
fi
log_success "k6 is available"

# Check if PostgreSQL is running
if command -v pg_isready &> /dev/null && pg_isready -q; then
    log_success "PostgreSQL is running"
    POSTGRES_AVAILABLE=true
else
    log_info "PostgreSQL not running. Tests will use SQLite"
    POSTGRES_AVAILABLE=false
fi

# Check if Redis is running
if command -v redis-cli &> /dev/null && redis-cli ping > /dev/null 2>&1; then
    log_success "Redis is running"
    REDIS_AVAILABLE=true
else
    log_info "Redis not running. Tests will run without caching"
    REDIS_AVAILABLE=false
fi

# Test 1: Build with performance features
log_test "Building HAL9 with performance optimizations..."
cargo build --release --features "performance"
log_success "Build completed"

# Test 2: Database performance comparison
if [ "$POSTGRES_AVAILABLE" = true ]; then
    log_test "Testing PostgreSQL performance..."
    
    # Start server with PostgreSQL
    DATABASE_URL="postgresql://postgres:postgres@localhost/hal9_test" \
    DATABASE_TYPE="postgres" \
    cargo run --release --bin hal9-server examples/config-3neurons.yaml > postgres-server.log 2>&1 &
    PG_SERVER_PID=$!
    
    sleep 10
    
    # Run quick performance test
    k6 run --quiet --duration 30s --vus 10 tests/load/k6-test.js > postgres-perf.txt
    
    kill $PG_SERVER_PID 2>/dev/null || true
    
    # Extract metrics
    PG_P95=$(grep "http_req_duration" postgres-perf.txt | grep "p(95)" | awk '{print $NF}')
    log_success "PostgreSQL p95 latency: ${PG_P95}ms"
fi

# Test 3: SQLite baseline
log_test "Testing SQLite performance (baseline)..."
DATABASE_TYPE="sqlite" \
cargo run --release --bin hal9-server examples/config-3neurons.yaml > sqlite-server.log 2>&1 &
SQLITE_SERVER_PID=$!

sleep 10

# Run quick performance test
k6 run --quiet --duration 30s --vus 10 tests/load/k6-test.js > sqlite-perf.txt

kill $SQLITE_SERVER_PID 2>/dev/null || true

# Extract metrics
SQLITE_P95=$(grep "http_req_duration" sqlite-perf.txt | grep "p(95)" | awk '{print $NF}')
log_success "SQLite p95 latency: ${SQLITE_P95}ms"

# Test 4: Redis caching impact
if [ "$REDIS_AVAILABLE" = true ]; then
    log_test "Testing with Redis caching..."
    
    REDIS_URL="redis://localhost:6379" \
    CACHE_ENABLED="true" \
    cargo run --release --bin hal9-server examples/config-3neurons.yaml > redis-server.log 2>&1 &
    REDIS_SERVER_PID=$!
    
    sleep 10
    
    # Warm up cache
    log_info "Warming up cache..."
    for i in {1..100}; do
        curl -s -X POST "http://localhost:8080/api/v1/signal" \
            -H "Content-Type: application/json" \
            -d '{"content": "Cache warmup signal", "layer": "L4", "neuron_id": "test"}' > /dev/null
    done
    
    # Run performance test
    k6 run --quiet --duration 30s --vus 10 tests/load/k6-test.js > redis-perf.txt
    
    kill $REDIS_SERVER_PID 2>/dev/null || true
    
    # Extract metrics
    REDIS_P95=$(grep "http_req_duration" redis-perf.txt | grep "p(95)" | awk '{print $NF}')
    log_success "Redis-cached p95 latency: ${REDIS_P95}ms"
    
    # Check cache hit rate
    CACHE_STATS=$(curl -s "http://localhost:8080/api/v1/metrics" | jq '.data.cache_hit_rate // 0')
    log_info "Cache hit rate: ${CACHE_STATS}%"
fi

# Test 5: Load test at scale
log_test "Running 1000-user load test..."

# Start optimized server
if [ "$POSTGRES_AVAILABLE" = true ] && [ "$REDIS_AVAILABLE" = true ]; then
    log_info "Using PostgreSQL + Redis configuration"
    DATABASE_URL="postgresql://postgres:postgres@localhost/hal9_test" \
    DATABASE_TYPE="postgres" \
    REDIS_URL="redis://localhost:6379" \
    CACHE_ENABLED="true" \
    cargo run --release --bin hal9-server examples/config-3neurons.yaml > optimized-server.log 2>&1 &
else
    log_info "Using SQLite configuration"
    cargo run --release --bin hal9-server examples/config-3neurons.yaml > optimized-server.log 2>&1 &
fi

SERVER_PID=$!
sleep 15

# Run full load test
log_info "Starting k6 load test (this will take ~10 minutes)..."
k6 run tests/load/k6-test.js | tee load-test-results.txt

# Kill server
kill $SERVER_PID 2>/dev/null || true

# Parse results
log_test "Analyzing results..."

# Extract key metrics
TOTAL_REQUESTS=$(grep "http_reqs" load-test-results.txt | awk '{print $2}')
AVG_RATE=$(grep "http_reqs" load-test-results.txt | grep "/s" | awk '{print $3}')
P95_LATENCY=$(grep "http_req_duration" load-test-results.txt | grep "p(95)" | head -1 | awk '{print $3}')
P99_LATENCY=$(grep "http_req_duration" load-test-results.txt | grep "p(99)" | head -1 | awk '{print $3}')
ERROR_RATE=$(grep "http_req_failed" load-test-results.txt | awk '{print $3}')

echo ""
echo "====================================="
echo "Performance Test Results Summary"
echo "====================================="
echo ""
echo "Configuration:"
if [ "$POSTGRES_AVAILABLE" = true ]; then
    echo "  Database: PostgreSQL ✓"
else
    echo "  Database: SQLite"
fi
if [ "$REDIS_AVAILABLE" = true ]; then
    echo "  Cache: Redis ✓"
else
    echo "  Cache: None"
fi
echo ""
echo "Load Test Results (1000 concurrent users):"
echo "  Total Requests: $TOTAL_REQUESTS"
echo "  Average Rate: $AVG_RATE req/s"
echo "  P95 Latency: $P95_LATENCY ms"
echo "  P99 Latency: $P99_LATENCY ms"
echo "  Error Rate: $ERROR_RATE"
echo ""

# Performance comparison
if [ "$POSTGRES_AVAILABLE" = true ]; then
    echo "Database Performance:"
    echo "  SQLite p95: ${SQLITE_P95}ms"
    echo "  PostgreSQL p95: ${PG_P95}ms"
    
    # Calculate improvement
    if command -v bc &> /dev/null; then
        IMPROVEMENT=$(echo "scale=2; (${SQLITE_P95} - ${PG_P95}) / ${SQLITE_P95} * 100" | bc)
        echo "  Improvement: ${IMPROVEMENT}%"
    fi
fi

# Check if we met targets
echo ""
echo "Target Metrics:"
if (( $(echo "$P95_LATENCY < 500" | bc -l) )); then
    log_success "P95 < 500ms target: ACHIEVED ($P95_LATENCY ms)"
else
    log_error "P95 < 500ms target: MISSED ($P95_LATENCY ms)"
fi

if (( $(echo "$P99_LATENCY < 1000" | bc -l) )); then
    log_success "P99 < 1000ms target: ACHIEVED ($P99_LATENCY ms)"
else
    log_error "P99 < 1000ms target: MISSED ($P99_LATENCY ms)"
fi

# Cleanup
rm -f postgres-perf.txt sqlite-perf.txt redis-perf.txt
rm -f postgres-server.log sqlite-server.log redis-server.log optimized-server.log

echo ""
log_success "Performance optimization tests completed!"

# Recommendations
echo ""
echo "Recommendations:"
if [ "$POSTGRES_AVAILABLE" != true ]; then
    echo "  - Install PostgreSQL for better performance at scale"
fi
if [ "$REDIS_AVAILABLE" != true ]; then
    echo "  - Install Redis for caching layer (90%+ cache hit rate)"
fi
echo "  - Consider horizontal scaling for > 1000 users"
echo "  - Monitor metrics at /metrics endpoint"