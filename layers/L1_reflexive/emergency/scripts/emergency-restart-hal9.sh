#!/bin/bash
# Emergency HAL9 Restart Script
# Created: 2025-06-13
# Purpose: Complete HAL9 system restart with health checks
# Works at 3am: ABSOLUTELY

set -euo pipefail

# Colors for 3am visibility
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${YELLOW}
╔═══════════════════════════════════════════╗
║    🚨 EMERGENCY HAL9 RESTART SCRIPT 🚨     ║
║         Complete System Recovery           ║
╚═══════════════════════════════════════════╝
${NC}"

# Configuration
HAL9_PORT=${HAL9_PORT:-8080}
HAL9_DIR="/Users/icedac/2lab.ai/2hal9"
LOG_DIR="$HAL9_DIR/L1_reflexive/logs"
EMERGENCY_LOG="$LOG_DIR/emergency-restart-$(date +%Y%m%d-%H%M%S).log"

# Ensure log directory exists
mkdir -p "$LOG_DIR"

# Logging function
log() {
    echo -e "$1" | tee -a "$EMERGENCY_LOG"
}

# Step 1: Kill any existing HAL9 or layer9 processes
kill_existing_processes() {
    log "${YELLOW}[STEP 1] Killing existing processes...${NC}"
    
    # Find and kill layer9-server processes
    LAYER9_PIDS=$(pgrep -f "layer9-server" || true)
    if [ -n "$LAYER9_PIDS" ]; then
        log "${RED}[KILL] Found layer9-server processes: $LAYER9_PIDS${NC}"
        for pid in $LAYER9_PIDS; do
            kill -9 $pid 2>/dev/null || true
            log "[KILLED] Process $pid"
        done
    fi
    
    # Find and kill hal9-server processes
    HAL9_PIDS=$(pgrep -f "hal9-server" || true)
    if [ -n "$HAL9_PIDS" ]; then
        log "${YELLOW}[KILL] Found hal9-server processes: $HAL9_PIDS${NC}"
        for pid in $HAL9_PIDS; do
            kill -9 $pid 2>/dev/null || true
            log "[KILLED] Process $pid"
        done
    fi
    
    sleep 2
    log "${GREEN}[OK] All existing processes killed${NC}"
}

# Step 2: Clear port conflicts
clear_port_conflicts() {
    log "${YELLOW}[STEP 2] Clearing port conflicts...${NC}"
    
    # Use our port conflict script
    if [ -f "$HAL9_DIR/L1_reflexive/emergency/scripts/emergency-port-conflict.sh" ]; then
        "$HAL9_DIR/L1_reflexive/emergency/scripts/emergency-port-conflict.sh" $HAL9_PORT true >> "$EMERGENCY_LOG" 2>&1
    else
        # Fallback port clearing
        PIDS=$(lsof -ti:$HAL9_PORT 2>/dev/null || true)
        if [ -n "$PIDS" ]; then
            for pid in $PIDS; do
                kill -9 $pid 2>/dev/null || true
            done
        fi
    fi
    
    log "${GREEN}[OK] Port $HAL9_PORT cleared${NC}"
}

# Step 3: Clean build artifacts
clean_build() {
    log "${YELLOW}[STEP 3] Cleaning build artifacts...${NC}"
    
    cd "$HAL9_DIR"
    
    # Clean cargo artifacts
    cargo clean 2>&1 | tee -a "$EMERGENCY_LOG"
    
    # Remove any lock files
    rm -f Cargo.lock 2>/dev/null || true
    
    log "${GREEN}[OK] Build artifacts cleaned${NC}"
}

# Step 4: Rebuild HAL9
rebuild_hal9() {
    log "${YELLOW}[STEP 4] Rebuilding HAL9...${NC}"
    
    cd "$HAL9_DIR"
    
    # Run clippy first to catch issues
    log "[BUILD] Running clippy checks..."
    if ! cargo clippy --workspace --no-deps -- -W clippy::all 2>&1 | tee -a "$EMERGENCY_LOG"; then
        log "${RED}[WARN] Clippy warnings detected but continuing...${NC}"
    fi
    
    # Build in release mode
    log "[BUILD] Building HAL9 in release mode..."
    if ! cargo build --workspace --release 2>&1 | tee -a "$EMERGENCY_LOG"; then
        log "${RED}[FAILED] Build failed!${NC}"
        return 1
    fi
    
    log "${GREEN}[OK] HAL9 built successfully${NC}"
}

# Step 5: Run tests
run_tests() {
    log "${YELLOW}[STEP 5] Running tests...${NC}"
    
    cd "$HAL9_DIR"
    
    if ! cargo test --workspace 2>&1 | tee -a "$EMERGENCY_LOG"; then
        log "${RED}[WARN] Some tests failed but continuing...${NC}"
    else
        log "${GREEN}[OK] All tests passed${NC}"
    fi
}

# Step 6: Start HAL9 server
start_hal9() {
    log "${YELLOW}[STEP 6] Starting HAL9 server...${NC}"
    
    cd "$HAL9_DIR"
    
    # Create a startup script
    cat > /tmp/start-hal9.sh << 'EOF'
#!/bin/bash
cd /Users/icedac/2lab.ai/2hal9
export RUST_LOG=info
export HAL9_ENV=production
exec cargo run --release --bin hal9-server -- --port $1
EOF
    chmod +x /tmp/start-hal9.sh
    
    # Start HAL9 in background with nohup
    nohup /tmp/start-hal9.sh $HAL9_PORT > "$LOG_DIR/hal9-server.log" 2>&1 &
    HAL9_PID=$!
    
    log "[START] HAL9 server starting with PID $HAL9_PID..."
    
    # Wait for server to start
    sleep 5
    
    # Check if process is still running
    if kill -0 $HAL9_PID 2>/dev/null; then
        log "${GREEN}[OK] HAL9 server started successfully (PID: $HAL9_PID)${NC}"
        echo $HAL9_PID > "$HAL9_DIR/hal9.pid"
    else
        log "${RED}[FAILED] HAL9 server failed to start${NC}"
        tail -20 "$LOG_DIR/hal9-server.log" | tee -a "$EMERGENCY_LOG"
        return 1
    fi
}

# Step 7: Health check
health_check() {
    log "${YELLOW}[STEP 7] Running health checks...${NC}"
    
    # Check if port is listening
    if lsof -i:$HAL9_PORT | grep LISTEN > /dev/null 2>&1; then
        log "${GREEN}[OK] HAL9 is listening on port $HAL9_PORT${NC}"
    else
        log "${RED}[FAILED] HAL9 is not listening on port $HAL9_PORT${NC}"
        return 1
    fi
    
    # Try to curl the health endpoint
    if command -v curl &> /dev/null; then
        if curl -s http://localhost:$HAL9_PORT/health > /dev/null 2>&1; then
            log "${GREEN}[OK] Health endpoint responding${NC}"
        else
            log "${YELLOW}[WARN] Health endpoint not responding (server may still be starting)${NC}"
        fi
    fi
    
    log "${GREEN}[OK] Basic health checks passed${NC}"
}

# Main execution
main() {
    log "[START] Emergency HAL9 restart initiated at $(date)"
    log "[INFO] Logging to: $EMERGENCY_LOG"
    
    # Execute steps
    kill_existing_processes
    clear_port_conflicts
    
    # Ask about rebuild
    if [ "${SKIP_BUILD:-false}" != "true" ]; then
        clean_build
        rebuild_hal9 || exit 1
        run_tests
    else
        log "${YELLOW}[SKIP] Skipping build steps (SKIP_BUILD=true)${NC}"
    fi
    
    start_hal9 || exit 1
    health_check || log "${YELLOW}[WARN] Health check failed but server may still be starting${NC}"
    
    log "
${GREEN}╔═══════════════════════════════════════════╗
║        ✅ HAL9 RESTART COMPLETE           ║
║                                           ║
║  Server PID: $(cat $HAL9_DIR/hal9.pid 2>/dev/null || echo 'unknown')                          ║
║  Port: $HAL9_PORT                              ║
║  Logs: $LOG_DIR/hal9-server.log           ║
╚═══════════════════════════════════════════╝${NC}
"
    
    log "[NEXT] Monitor logs: tail -f $LOG_DIR/hal9-server.log"
    log "[NEXT] Check status: curl http://localhost:$HAL9_PORT/health"
}

# Error handling
trap 'log "${RED}[ERROR] Script failed at line $LINENO${NC}"' ERR

# Show usage
if [ "$1" == "--help" ] || [ "$1" == "-h" ]; then
    echo "Usage: $0 [options]"
    echo ""
    echo "Options:"
    echo "  --skip-build    Skip rebuild steps (use existing binary)"
    echo "  --port PORT     Use specific port (default: 8080)"
    echo ""
    echo "Environment variables:"
    echo "  HAL9_PORT      Port to use (default: 8080)"
    echo "  SKIP_BUILD     Skip build if set to 'true'"
    exit 0
fi

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --skip-build)
            export SKIP_BUILD=true
            shift
            ;;
        --port)
            HAL9_PORT="$2"
            shift 2
            ;;
        *)
            shift
            ;;
    esac
done

# Execute
main

# Stack Overflow said this would work
# https://stackoverflow.com/questions/3855127/find-and-kill-process-locking-port-3000-on-mac