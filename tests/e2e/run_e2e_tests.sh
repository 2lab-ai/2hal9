#!/bin/bash
# E2E Test Runner for HAL9 Server
# This script manages the server lifecycle and runs all E2E tests

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
LOG_DIR="$PROJECT_ROOT/artifacts/e2e-logs"
SERVER_PID=""
SERVER_PORT="${HAL9_PORT:-3000}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${BLUE}[E2E]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[✓]${NC} $1"
}

print_error() {
    echo -e "${RED}[✗]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[!]${NC} $1"
}

# Cleanup function
cleanup() {
    print_status "Cleaning up..."
    
    if [ ! -z "$SERVER_PID" ]; then
        print_status "Stopping server (PID: $SERVER_PID)"
        kill $SERVER_PID 2>/dev/null || true
        wait $SERVER_PID 2>/dev/null || true
    fi
    
    # Kill any orphaned server processes
    pkill -f "hal9-server" 2>/dev/null || true
}

# Set trap to cleanup on exit
trap cleanup EXIT

# Check dependencies
check_dependencies() {
    print_status "Checking dependencies..."
    
    if ! command -v cargo &> /dev/null; then
        print_error "cargo not found. Please install Rust."
        exit 1
    fi
    
    if ! command -v jq &> /dev/null; then
        print_warning "jq not found. Some tests may fail."
    fi
    
    print_success "Dependencies OK"
}

# Build the project
build_project() {
    print_status "Building project..."
    
    cd "$PROJECT_ROOT"
    
    if cargo build --release --bin hal9-server; then
        print_success "Build successful"
    else
        print_error "Build failed"
        exit 1
    fi
}

# Start the server
start_server() {
    print_status "Starting HAL9 server..."
    
    mkdir -p "$LOG_DIR"
    local log_file="$LOG_DIR/server-$(date +%Y%m%d-%H%M%S).log"
    
    # Set environment variables
    export RUST_LOG=info
    export HAL9_PORT=$SERVER_PORT
    export DATABASE_URL="${DATABASE_URL:-postgresql://postgres:postgres@localhost/hal9_test}"
    export CLAUDE_MODE="${CLAUDE_MODE:-mock}"
    
    # Start server in background
    cd "$PROJECT_ROOT"
    cargo run --release --bin hal9-server > "$log_file" 2>&1 &
    SERVER_PID=$!
    
    print_status "Server started with PID $SERVER_PID"
    print_status "Server logs: $log_file"
    
    # Wait for server to be ready
    local max_wait=30
    local waited=0
    
    while [ $waited -lt $max_wait ]; do
        if curl -s "http://localhost:$SERVER_PORT/health" > /dev/null 2>&1; then
            print_success "Server is ready!"
            return 0
        fi
        
        # Check if server process is still running
        if ! ps -p $SERVER_PID > /dev/null 2>&1; then
            print_error "Server process died unexpectedly"
            echo "Last 20 lines of server log:"
            tail -20 "$log_file"
            exit 1
        fi
        
        sleep 1
        waited=$((waited + 1))
        echo -n "."
    done
    
    print_error "Server failed to start within $max_wait seconds"
    echo "Last 20 lines of server log:"
    tail -20 "$log_file"
    exit 1
}

# Run E2E tests
run_e2e_tests() {
    print_status "Running E2E tests..."
    
    cd "$PROJECT_ROOT"
    
    # Set test environment
    export HAL9_TEST_URL="http://localhost:$SERVER_PORT"
    export RUST_TEST_THREADS=1  # Run tests sequentially for consistency
    
    local test_output="$LOG_DIR/test-results-$(date +%Y%m%d-%H%M%S).log"
    
    # Run specific E2E test modules
    if cargo test --test e2e -- --test-threads=1 --nocapture | tee "$test_output"; then
        print_success "All E2E tests passed!"
        
        # Print summary
        echo
        print_status "Test Summary:"
        grep -E "(test .* ... ok|test result:|passed)" "$test_output" | tail -20
    else
        print_error "E2E tests failed"
        
        # Print failures
        echo
        print_status "Failed tests:"
        grep -E "(FAILED|error:|panicked at)" "$test_output" | head -20
        
        exit 1
    fi
}

# Run shell-based integration tests
run_shell_tests() {
    print_status "Running shell-based integration tests..."
    
    local shell_test_dir="$PROJECT_ROOT/tests/integration"
    
    if [ -d "$shell_test_dir" ]; then
        for test_script in "$shell_test_dir"/*.sh; do
            if [ -f "$test_script" ] && [ -x "$test_script" ]; then
                local test_name=$(basename "$test_script")
                print_status "Running $test_name"
                
                if HAL9_URL="http://localhost:$SERVER_PORT" "$test_script"; then
                    print_success "$test_name passed"
                else
                    print_error "$test_name failed"
                fi
            fi
        done
    fi
}

# Generate test report
generate_report() {
    print_status "Generating test report..."
    
    local report_file="$PROJECT_ROOT/reports/E2E_TEST_REPORT_$(date +%Y-%m-%d).md"
    
    cat > "$report_file" << EOF
# E2E Test Report - $(date +%Y-%m-%d)

## Summary

- **Date**: $(date)
- **Server Port**: $SERVER_PORT
- **Test Mode**: ${CLAUDE_MODE:-mock}
- **Total Duration**: ${TOTAL_DURATION}s

## Test Results

### Rust E2E Tests
\`\`\`
$(grep -E "test result:|running \d+ test" "$LOG_DIR/test-results-"*.log | tail -5)
\`\`\`

### Performance Metrics
\`\`\`
$(grep -E "concurrent_creation|signal_propagation|consciousness_metrics|Self-organization completed" "$LOG_DIR/test-results-"*.log | tail -10)
\`\`\`

## Server Health During Tests

- CPU Usage: $(ps aux | grep hal9-server | grep -v grep | awk '{print $3}')%
- Memory Usage: $(ps aux | grep hal9-server | grep -v grep | awk '{print $4}')%

## Recommendations

1. Monitor performance metrics for regression
2. Add more WebSocket test scenarios
3. Implement load testing with higher concurrency
4. Add chaos testing for resilience

EOF

    print_success "Report generated: $report_file"
}

# Main execution
main() {
    local start_time=$(date +%s)
    
    echo "================================================"
    echo "       HAL9 E2E Test Suite"
    echo "================================================"
    echo
    
    # Parse arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            --port)
                SERVER_PORT="$2"
                shift 2
                ;;
            --skip-build)
                SKIP_BUILD=true
                shift
                ;;
            --auth)
                export AUTH_ENABLED=true
                shift
                ;;
            *)
                print_error "Unknown option: $1"
                echo "Usage: $0 [--port PORT] [--skip-build] [--auth]"
                exit 1
                ;;
        esac
    done
    
    check_dependencies
    
    if [ "$SKIP_BUILD" != "true" ]; then
        build_project
    fi
    
    start_server
    run_e2e_tests
    run_shell_tests
    
    local end_time=$(date +%s)
    TOTAL_DURATION=$((end_time - start_time))
    
    generate_report
    
    echo
    echo "================================================"
    print_success "E2E test suite completed in ${TOTAL_DURATION}s"
    echo "================================================"
}

# Run main function
main "$@"