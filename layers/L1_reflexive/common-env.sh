#!/bin/bash
#
# Common environment setup for L1 Reflexive scripts
# Source this file in all L1 scripts: source "$(dirname "$0")/../common-env.sh"
#

# Get the absolute path to the HAL9 project root
export HAL9_HOME="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

# Set up common paths
export HAL9_CONFIG_DIR="$HAL9_HOME/L5_strategic/research/examples"
export HAL9_DATA_DIR="$HAL9_HOME/substrate/storage/databases"
export HAL9_LOG_DIR="${HAL9_LOG_DIR:-$HAL9_HOME/logs}"
export HAL9_CACHE_DIR="${HAL9_CACHE_DIR:-$HAL9_HOME/L1_reflexive/cache}"

# Binary paths (prefer built binaries, fallback to cargo run)
if [ -f "$HAL9_HOME/target/debug/hal9-server" ]; then
    export HAL9_SERVER_BIN="$HAL9_HOME/target/debug/hal9-server"
    export HAL9_SERVER_CMD="$HAL9_SERVER_BIN"
else
    export HAL9_SERVER_BIN="hal9-server"
    export HAL9_SERVER_CMD="cargo run --bin hal9-server --"
fi

if [ -f "$HAL9_HOME/target/debug/hal9-cli" ]; then
    export HAL9_CLI_BIN="$HAL9_HOME/target/debug/hal9-cli"
    export HAL9_CLI_CMD="$HAL9_CLI_BIN"
else
    export HAL9_CLI_BIN="hal9-cli"
    export HAL9_CLI_CMD="cargo run --bin hal9-cli --"
fi

if [ -f "$HAL9_HOME/target/debug/hal9-codegen" ]; then
    export HAL9_CODEGEN_BIN="$HAL9_HOME/target/debug/hal9-codegen"
    export HAL9_CODEGEN_CMD="$HAL9_CODEGEN_BIN"
else
    export HAL9_CODEGEN_BIN="hal9-codegen"
    export HAL9_CODEGEN_CMD="cargo run --bin hal9-codegen --"
fi

# Port allocations
export HAL9_PORT_MAIN="${HAL9_PORT_MAIN:-8080}"
export HAL9_PORT_AUTH="${HAL9_PORT_AUTH:-8081}"
export HAL9_PORT_METRICS="${HAL9_PORT_METRICS:-9090}"
export HAL9_PORT_GRAPHQL="${HAL9_PORT_GRAPHQL:-8082}"
export HAL9_PORT_ADMIN="${HAL9_PORT_ADMIN:-8083}"

# Create necessary directories
mkdir -p "$HAL9_LOG_DIR" "$HAL9_CACHE_DIR"

# Common functions

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if a command exists
require_command() {
    local cmd=$1
    local msg=${2:-"$cmd is required but not found"}
    
    if ! command -v "$cmd" &> /dev/null; then
        log_error "$msg"
        exit 1
    fi
}

# Check if a port is available
check_port() {
    local port=$1
    if lsof -Pi :$port -sTCP:LISTEN -t >/dev/null 2>&1; then
        return 1
    else
        return 0
    fi
}

# Wait for a service to be ready
wait_for_service() {
    local url=$1
    local timeout=${2:-30}
    local service_name=${3:-"service"}
    
    log_info "Waiting for $service_name to be ready at $url..."
    
    local count=0
    while [ $count -lt $timeout ]; do
        if curl -s "$url" > /dev/null 2>&1; then
            log_success "$service_name is ready!"
            return 0
        fi
        sleep 1
        ((count++))
        echo -n "."
    done
    echo
    
    log_error "$service_name failed to start within $timeout seconds"
    return 1
}

# Clean up function for scripts
cleanup() {
    local pid_file=$1
    local log_file=${2:-""}
    
    if [ -f "$pid_file" ]; then
        local pid=$(cat "$pid_file")
        if kill -0 "$pid" 2>/dev/null; then
            log_info "Stopping process $pid..."
            kill "$pid"
            sleep 2
            if kill -0 "$pid" 2>/dev/null; then
                log_warning "Process didn't stop gracefully, forcing..."
                kill -9 "$pid"
            fi
        fi
        rm -f "$pid_file"
    fi
    
    if [ -n "$log_file" ] && [ -f "$log_file" ]; then
        log_info "Log file saved at: $log_file"
    fi
}

# Set up trap for cleanup on exit
setup_cleanup() {
    local pid_file=$1
    local log_file=${2:-""}
    
    trap "cleanup '$pid_file' '$log_file'" EXIT INT TERM
}

# Export common functions
export -f log_info log_success log_warning log_error
export -f require_command check_port wait_for_service
export -f cleanup setup_cleanup