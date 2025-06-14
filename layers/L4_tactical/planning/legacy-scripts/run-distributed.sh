#!/bin/bash
# Script to run HAL9 in distributed mode with two servers

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if servers are already running
check_existing_servers() {
    if lsof -i:9001 >/dev/null 2>&1; then
        log_error "Port 9001 is already in use. Please stop existing server."
        exit 1
    fi
    
    if lsof -i:9002 >/dev/null 2>&1; then
        log_error "Port 9002 is already in use. Please stop existing server."
        exit 1
    fi
}

# Build the project
build_project() {
    log_info "Building HAL9..."
    cd "$PROJECT_ROOT"
    cargo build --release
    log_info "Build complete"
}

# Start server 1
start_server1() {
    log_info "Starting Server 1 (Strategic Layer)..."
    cd "$PROJECT_ROOT"
    
    ./target/release/hal9-server examples/distributed-2servers.yaml > server1.log 2>&1 &
    SERVER1_PID=$!
    echo $SERVER1_PID > server1.pid
    
    sleep 2
    
    if kill -0 $SERVER1_PID 2>/dev/null; then
        log_info "Server 1 started (PID: $SERVER1_PID)"
        log_info "  - TCP: localhost:9001"
        log_info "  - HTTP: localhost:8080"
    else
        log_error "Server 1 failed to start"
        cat server1.log
        exit 1
    fi
}

# Start server 2
start_server2() {
    log_info "Starting Server 2 (Architecture & Implementation Layers)..."
    cd "$PROJECT_ROOT"
    
    # Start on different HTTP port
    HTTP_PORT=8081 ./target/release/hal9-server examples/distributed-server2.yaml > server2.log 2>&1 &
    SERVER2_PID=$!
    echo $SERVER2_PID > server2.pid
    
    sleep 3
    
    if kill -0 $SERVER2_PID 2>/dev/null; then
        log_info "Server 2 started (PID: $SERVER2_PID)"
        log_info "  - TCP: localhost:9002"
        log_info "  - HTTP: localhost:8081"
    else
        log_error "Server 2 failed to start"
        cat server2.log
        exit 1
    fi
}

# Show status
show_status() {
    echo
    log_info "Distributed HAL9 is running!"
    echo
    echo -e "${BLUE}Server Status:${NC}"
    echo "  Server 1 (Strategic):        http://localhost:8080/api/v1/status"
    echo "  Server 2 (Arch & Impl):      http://localhost:8081/api/v1/status"
    echo
    echo -e "${BLUE}Network Status:${NC}"
    echo "  Server 1 Network:            http://localhost:8080/api/v1/network/status"
    echo "  Server 2 Network:            http://localhost:8081/api/v1/network/status"
    echo
    echo -e "${BLUE}Test Commands:${NC}"
    echo "  # Send a test signal:"
    echo "  ./target/release/hal9 signal forward --from client --to strategic-main --content \"Create a web application\""
    echo
    echo "  # Check server 1 status:"
    echo "  curl http://localhost:8080/api/v1/status | jq"
    echo
    echo "  # Check server 2 status:"
    echo "  curl http://localhost:8081/api/v1/status | jq"
    echo
    echo -e "${BLUE}Logs:${NC}"
    echo "  tail -f server1.log    # Server 1 logs"
    echo "  tail -f server2.log    # Server 2 logs"
    echo
    echo -e "${BLUE}Stop Servers:${NC}"
    echo "  ./scripts/stop-distributed.sh"
    echo
}

# Monitor logs
monitor_logs() {
    log_info "Monitoring server logs (Ctrl+C to stop)..."
    echo
    
    # Use tail with labeled output
    tail -f server1.log server2.log | while read line; do
        if [[ $line == "==> server1.log <==" ]]; then
            echo -e "\n${GREEN}[SERVER 1]${NC}"
        elif [[ $line == "==> server2.log <==" ]]; then
            echo -e "\n${BLUE}[SERVER 2]${NC}"
        else
            echo "$line"
        fi
    done
}

# Main execution
main() {
    log_info "Starting HAL9 Distributed Mode"
    
    check_existing_servers
    build_project
    start_server1
    start_server2
    show_status
    
    # Ask if user wants to monitor logs
    read -p "Monitor server logs? (y/N) " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        monitor_logs
    fi
}

# Handle cleanup on exit
cleanup() {
    log_warn "Shutting down servers..."
    
    if [ -f server1.pid ]; then
        kill $(cat server1.pid) 2>/dev/null || true
        rm server1.pid
    fi
    
    if [ -f server2.pid ]; then
        kill $(cat server2.pid) 2>/dev/null || true
        rm server2.pid
    fi
    
    log_info "Servers stopped"
}

trap cleanup EXIT

# Run main
main