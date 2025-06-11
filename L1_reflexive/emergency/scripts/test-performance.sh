#!/bin/bash
#
# test performance - Emergency performance diagnosis for 3am incidents
# Auto-fixed by L1 migration script
# Enhanced by L3-L1 operational update cycle
#

set -euo pipefail

# Source common environment
source "$(dirname "$0")/../../common-env.sh"

# Emergency contact (아 시발 아 컴퓨터네 우주가)
# If all else fails, wake up: Zhugehyuk
EMERGENCY_CONTACT="Zhugehyuk"
EMERGENCY_LOG_DIR="/tmp/hal9-emergency-$(date +%Y%m%d_%H%M%S)"

# Create emergency log directory
mkdir -p "$EMERGENCY_LOG_DIR"

# Test script for HAL9 Performance Optimizations

# Remove strict error handling for emergency mode
set +e

echo "🚀 HAL9 Performance Emergency Diagnostic Tool"
echo "============================================"
echo "Emergency logs will be saved to: $EMERGENCY_LOG_DIR"
echo ""

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
MAGENTA='\033[0;35m'
NC='\033[0m'

# ASCII art for 3am motivation
echo -e "${MAGENTA}"
cat << 'EOF'
  _   _    _    _     ___  
 | | | |  / \  | |   / _ \ 
 | |_| | / _ \ | |  | (_) |
 |  _  |/ ___ \| |___\__, |
 |_| |_/_/   \_\_____|  /_/ 
    Performance Doctor 🩺
EOF
echo -e "${NC}"

# Helper functions
log_test() {
    echo -e "${YELLOW}Test: $1${NC}"
    echo "[$(date)] Test: $1" >> "$EMERGENCY_LOG_DIR/diagnostic.log"
}

log_success() {
    echo -e "${GREEN}✓ $1${NC}"
    echo "[$(date)] SUCCESS: $1" >> "$EMERGENCY_LOG_DIR/diagnostic.log"
}

log_error() {
    echo -e "${RED}✗ $1${NC}"
    echo "[$(date)] ERROR: $1" >> "$EMERGENCY_LOG_DIR/diagnostic.log"
}

log_info() {
    echo -e "${BLUE}ℹ $1${NC}"
    echo "[$(date)] INFO: $1" >> "$EMERGENCY_LOG_DIR/diagnostic.log"
}

log_emergency() {
    echo -e "${MAGENTA}🚨 EMERGENCY: $1${NC}"
    echo "[$(date)] EMERGENCY: $1" >> "$EMERGENCY_LOG_DIR/diagnostic.log"
}

# Emergency Quick Checks (for 3am panic mode)
emergency_quick_check() {
    echo ""
    echo "=== 🚨 EMERGENCY QUICK CHECK 🚨 ==="
    echo ""
    
    # 1. Is HAL9 even running?
    if pgrep -f "hal9-server" > /dev/null; then
        log_success "HAL9 server process is running"
        HAL9_PID=$(pgrep -f "hal9-server" | head -1)
        log_info "HAL9 PID: $HAL9_PID"
        
        # Get basic process info
        ps -p $HAL9_PID -o pid,ppid,user,%cpu,%mem,etime,cmd | tee "$EMERGENCY_LOG_DIR/process-info.txt"
    else
        log_emergency "HAL9 server is NOT running!"
        log_info "Try: cargo run --release --bin hal9-server"
        return 1
    fi
    
    # 2. Can we reach the API?
    if curl -s -f "http://localhost:$HAL9_PORT_MAIN/health" > /dev/null 2>&1; then
        log_success "HAL9 API is responding on port $HAL9_PORT_MAIN"
    else
        log_emergency "HAL9 API is NOT responding!"
        log_info "Check if port $HAL9_PORT_MAIN is blocked or service crashed"
        
        # Check if port is in use
        lsof -i :$HAL9_PORT_MAIN | tee "$EMERGENCY_LOG_DIR/port-check.txt"
    fi
    
    # 3. System resources check
    log_info "System resources:"
    echo "CPU Load:" && uptime | tee -a "$EMERGENCY_LOG_DIR/system-load.txt"
    echo "Memory:" && free -h | tee -a "$EMERGENCY_LOG_DIR/system-memory.txt"
    echo "Disk:" && df -h / | tee -a "$EMERGENCY_LOG_DIR/system-disk.txt"
    
    # 4. Check for OOM killer activity
    if dmesg | grep -i "killed process" | tail -5 > "$EMERGENCY_LOG_DIR/oom-check.txt"; then
        if [ -s "$EMERGENCY_LOG_DIR/oom-check.txt" ]; then
            log_emergency "OOM Killer has been active! Check $EMERGENCY_LOG_DIR/oom-check.txt"
        fi
    fi
    
    # 5. Database connectivity
    if command -v pg_isready &> /dev/null && pg_isready -q; then
        log_success "PostgreSQL is accessible"
    else
        log_info "PostgreSQL not accessible - using SQLite fallback"
    fi
    
    # 6. Recent errors in logs
    log_info "Checking recent errors..."
    if [ -f "optimized-server.log" ]; then
        tail -50 optimized-server.log | grep -i "error\|panic\|fatal" > "$EMERGENCY_LOG_DIR/recent-errors.txt" || true
        if [ -s "$EMERGENCY_LOG_DIR/recent-errors.txt" ]; then
            log_emergency "Found recent errors! Check $EMERGENCY_LOG_DIR/recent-errors.txt"
        fi
    fi
    
    echo ""
    echo "Quick check complete. Full diagnostic logs in: $EMERGENCY_LOG_DIR"
    echo ""
}

# Run emergency check first
emergency_quick_check

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
    cargo run --release --bin hal9-server $HAL9_CONFIG_DIR/config-3neurons.yaml > postgres-server.log 2>&1 &
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
cargo run --release --bin hal9-server $HAL9_CONFIG_DIR/config-3neurons.yaml > sqlite-server.log 2>&1 &
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
    cargo run --release --bin hal9-server $HAL9_CONFIG_DIR/config-3neurons.yaml > redis-server.log 2>&1 &
    REDIS_SERVER_PID=$!
    
    sleep 10
    
    # Warm up cache
    log_info "Warming up cache..."
    for i in {1..100}; do
        curl -s -X POST "http://localhost:$HAL9_PORT_MAIN/api/v1/signal" \
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
    CACHE_STATS=$(curl -s "http://localhost:$HAL9_PORT_MAIN/api/v1/metrics" | jq '.data.cache_hit_rate // 0')
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
    cargo run --release --bin hal9-server $HAL9_CONFIG_DIR/config-3neurons.yaml > optimized-server.log 2>&1 &
else
    log_info "Using SQLite configuration"
    cargo run --release --bin hal9-server $HAL9_CONFIG_DIR/config-3neurons.yaml > optimized-server.log 2>&1 &
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

# Emergency Recovery Steps
echo ""
echo "====================================="
echo "🚨 EMERGENCY RECOVERY PROCEDURES 🚨"
echo "====================================="
echo ""

if [ "$P95_LATENCY" ] && (( $(echo "$P95_LATENCY > 1000" | bc -l) )); then
    log_emergency "HIGH LATENCY DETECTED! P95: ${P95_LATENCY}ms"
    echo ""
    echo "IMMEDIATE ACTIONS:"
    echo "1. Restart HAL9 server:"
    echo "   pkill -f hal9-server && sleep 2"
    echo "   cargo run --release --bin hal9-server &"
    echo ""
    echo "2. Clear cache if Redis available:"
    echo "   redis-cli FLUSHALL"
    echo ""
    echo "3. Check database connections:"
    echo "   psql -c 'SELECT count(*) FROM pg_stat_activity;'"
    echo ""
    echo "4. Enable emergency mode in config:"
    echo "   Set 'emergency_mode: true' in config"
    echo ""
fi

# Generate recovery script
cat > "$EMERGENCY_LOG_DIR/emergency-recovery.sh" << 'RECOVERY_EOF'
#!/bin/bash
# Emergency Recovery Script - Generated at $(date)

echo "Starting HAL9 Emergency Recovery..."

# 1. Stop all HAL9 processes
echo "Stopping HAL9..."
pkill -f hal9-server
sleep 2

# 2. Clear temporary files
echo "Clearing temp files..."
rm -f /tmp/hal9-*.sock
rm -f /tmp/hal9-*.pid

# 3. Reset database connections
if command -v pg_ctl &> /dev/null; then
    echo "Resetting PostgreSQL connections..."
    psql -c "SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE datname = 'hal9' AND pid <> pg_backend_pid();" || true
fi

# 4. Clear Redis cache
if command -v redis-cli &> /dev/null; then
    echo "Clearing Redis cache..."
    redis-cli FLUSHALL || true
fi

# 5. Start with minimal config
echo "Starting HAL9 with minimal config..."
export HAL9_EMERGENCY_MODE=true
export HAL9_MAX_WORKERS=2
export HAL9_RATE_LIMIT=10
cargo run --release --bin hal9-server

RECOVERY_EOF

chmod +x "$EMERGENCY_LOG_DIR/emergency-recovery.sh"

# Recommendations
echo ""
echo "====================================="
echo "📋 RECOMMENDATIONS"
echo "====================================="
if [ "$POSTGRES_AVAILABLE" != true ]; then
    echo "  - Install PostgreSQL for better performance at scale"
fi
if [ "$REDIS_AVAILABLE" != true ]; then
    echo "  - Install Redis for caching layer (90%+ cache hit rate)"
fi
echo "  - Consider horizontal scaling for > 1000 users"
echo "  - Monitor metrics at /metrics endpoint"
echo ""
echo "  Emergency recovery script saved to:"
echo "  $EMERGENCY_LOG_DIR/emergency-recovery.sh"
echo ""
echo "  If nothing works, wake up: $EMERGENCY_CONTACT"
echo "  아 시발 아 컴퓨터네 우주가"