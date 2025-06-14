#!/bin/bash
# Emergency Port Conflict Resolution Script
# Created: 2025-06-13
# Purpose: Handle port conflicts preventing HAL9 from starting
# Works at 3am: YES

set -euo pipefail

# ASCII art for 3am visibility
echo "
╔═══════════════════════════════════════════╗
║    🚨 EMERGENCY PORT CONFLICT HANDLER 🚨   ║
║         Port 8080 Conflict Resolution      ║
╚═══════════════════════════════════════════╝
"

PORT=${1:-8080}
FORCE=${2:-false}

# Function to check what's using the port
check_port() {
    echo "[CHECK] Looking for processes on port $PORT..."
    
    # macOS compatible lsof command
    if command -v lsof &> /dev/null; then
        PROCESS_INFO=$(lsof -ti:$PORT 2>/dev/null || echo "")
        if [ -n "$PROCESS_INFO" ]; then
            echo "[FOUND] Process using port $PORT:"
            lsof -i:$PORT
            return 0
        fi
    fi
    
    echo "[CLEAR] Port $PORT is available"
    return 1
}

# Function to kill process on port
kill_port_process() {
    local pid=$1
    local process_name=$(ps -p $pid -o comm= 2>/dev/null || echo "unknown")
    
    echo "[KILL] Attempting to kill process $pid ($process_name)..."
    
    # Try graceful shutdown first
    if [ "$FORCE" != "true" ]; then
        echo "[SOFT] Sending SIGTERM to $pid..."
        kill -TERM $pid 2>/dev/null || true
        sleep 2
        
        # Check if still running
        if kill -0 $pid 2>/dev/null; then
            echo "[WARN] Process $pid still running after SIGTERM"
        else
            echo "[OK] Process $pid terminated gracefully"
            return 0
        fi
    fi
    
    # Force kill if needed
    echo "[HARD] Sending SIGKILL to $pid..."
    kill -9 $pid 2>/dev/null || true
    sleep 1
    
    if kill -0 $pid 2>/dev/null; then
        echo "[ERROR] Failed to kill process $pid"
        return 1
    else
        echo "[OK] Process $pid force killed"
        return 0
    fi
}

# Main execution
main() {
    echo "[START] Emergency port conflict resolution for port $PORT"
    echo "[TIME] $(date '+%Y-%m-%d %H:%M:%S')"
    
    # Check current port status
    if check_port; then
        # Port is in use
        PIDS=$(lsof -ti:$PORT 2>/dev/null || echo "")
        
        if [ -z "$PIDS" ]; then
            echo "[ERROR] Port appears in use but no PID found"
            exit 1
        fi
        
        # Check if it's layer9-server specifically
        for pid in $PIDS; do
            PROCESS_CMD=$(ps -p $pid -o args= 2>/dev/null || echo "")
            if [[ "$PROCESS_CMD" == *"layer9-server"* ]]; then
                echo "[ALERT] Found layer9-server on port $PORT (PID: $pid)"
                echo "[INFO] Command: $PROCESS_CMD"
                
                # This is the known conflict from postmortem
                echo "[ACTION] Killing layer9-server to free port for HAL9..."
                if kill_port_process $pid; then
                    echo "[SUCCESS] layer9-server killed, port $PORT is now free"
                else
                    echo "[FAILED] Could not kill layer9-server"
                    exit 1
                fi
            else
                echo "[WARN] Non-layer9 process on port $PORT (PID: $pid)"
                if [ "$FORCE" == "true" ]; then
                    kill_port_process $pid
                else
                    echo "[SKIP] Use 'force' parameter to kill non-layer9 processes"
                fi
            fi
        done
        
        # Verify port is now free
        sleep 1
        if check_port; then
            echo "[FAILED] Port $PORT still in use after cleanup"
            exit 1
        else
            echo "[SUCCESS] Port $PORT is now available for HAL9"
        fi
    else
        echo "[OK] Port $PORT is already free"
    fi
    
    # Log the resolution
    LOG_FILE="/tmp/hal9-port-conflicts.log"
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] Resolved port $PORT conflict" >> $LOG_FILE
    
    echo "
╔═══════════════════════════════════════════╗
║          ✅ PORT CONFLICT RESOLVED         ║
║         You can now start HAL9 server      ║
╚═══════════════════════════════════════════╝
"
    
    # Suggest next action
    echo "[NEXT] Run: cargo run --bin hal9-server -- --port $PORT"
}

# Handle errors
trap 'echo "[ERROR] Script failed at line $LINENO"' ERR

# Show usage
if [ "$1" == "--help" ] || [ "$1" == "-h" ]; then
    echo "Usage: $0 [port] [force]"
    echo "  port:  Port number to check (default: 8080)"
    echo "  force: Kill any process on port, not just layer9 (default: false)"
    echo ""
    echo "Example:"
    echo "  $0          # Check and clear port 8080"
    echo "  $0 3000     # Check and clear port 3000"
    echo "  $0 8080 true # Force kill anything on port 8080"
    exit 0
fi

# Execute
main

# 아 시발 포트 충돌 때문에 새벽에 깨는거 진짜 싫다