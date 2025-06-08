#!/bin/bash
# Script to stop distributed HAL9 servers

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
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

# Stop servers
stop_servers() {
    cd "$PROJECT_ROOT"
    
    # Stop server 1
    if [ -f server1.pid ]; then
        PID=$(cat server1.pid)
        if kill -0 $PID 2>/dev/null; then
            log_info "Stopping Server 1 (PID: $PID)..."
            kill $PID
            sleep 1
            if kill -0 $PID 2>/dev/null; then
                log_warn "Server 1 didn't stop gracefully, forcing..."
                kill -9 $PID
            fi
        else
            log_warn "Server 1 process not found"
        fi
        rm server1.pid
    else
        log_warn "No PID file for Server 1"
    fi
    
    # Stop server 2
    if [ -f server2.pid ]; then
        PID=$(cat server2.pid)
        if kill -0 $PID 2>/dev/null; then
            log_info "Stopping Server 2 (PID: $PID)..."
            kill $PID
            sleep 1
            if kill -0 $PID 2>/dev/null; then
                log_warn "Server 2 didn't stop gracefully, forcing..."
                kill -9 $PID
            fi
        else
            log_warn "Server 2 process not found"
        fi
        rm server2.pid
    else
        log_warn "No PID file for Server 2"
    fi
    
    # Clean up any stray processes
    pkill -f "hal9-server examples/distributed" 2>/dev/null || true
    
    log_info "All servers stopped"
}

# Check if servers are running
check_servers() {
    RUNNING=0
    
    if lsof -i:9001 >/dev/null 2>&1; then
        log_info "Server 1 is running on port 9001"
        RUNNING=1
    fi
    
    if lsof -i:9002 >/dev/null 2>&1; then
        log_info "Server 2 is running on port 9002"
        RUNNING=1
    fi
    
    if [ $RUNNING -eq 0 ]; then
        log_info "No distributed servers are running"
    fi
}

# Main
main() {
    log_info "Stopping HAL9 Distributed Servers"
    
    check_servers
    stop_servers
    
    # Optional: Clean up log files
    read -p "Remove log files? (y/N) " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        rm -f "$PROJECT_ROOT"/server*.log
        log_info "Log files removed"
    fi
}

# Run main
main