#!/bin/bash
#
# L1 Reflexive Layer Dependency Checker
# Ensures all required tools and dependencies are available
#

set -euo pipefail

echo "=== HAL9 L1 Reflexive Layer Dependency Check ==="
echo

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Track missing dependencies
MISSING_DEPS=0
WARNINGS=0

# Function to check if a command exists
check_command() {
    local cmd=$1
    local required=${2:-true}
    local description=${3:-""}
    
    if command -v "$cmd" &> /dev/null; then
        echo -e "${GREEN}✓${NC} $cmd found"
        if [ -n "$description" ]; then
            echo "  └─ $description"
        fi
    else
        if [ "$required" = true ]; then
            echo -e "${RED}✗${NC} $cmd NOT FOUND (required)"
            if [ -n "$description" ]; then
                echo "  └─ $description"
            fi
            ((MISSING_DEPS++))
        else
            echo -e "${YELLOW}⚠${NC} $cmd not found (optional)"
            if [ -n "$description" ]; then
                echo "  └─ $description"
            fi
            ((WARNINGS++))
        fi
    fi
}

# Function to check environment variable
check_env() {
    local var=$1
    local required=${2:-false}
    local description=${3:-""}
    
    if [ -n "${!var:-}" ]; then
        echo -e "${GREEN}✓${NC} $var is set"
        if [ -n "$description" ]; then
            echo "  └─ $description"
        fi
    else
        if [ "$required" = true ]; then
            echo -e "${RED}✗${NC} $var NOT SET (required)"
            if [ -n "$description" ]; then
                echo "  └─ $description"
            fi
            ((MISSING_DEPS++))
        else
            echo -e "${YELLOW}⚠${NC} $var not set (optional)"
            if [ -n "$description" ]; then
                echo "  └─ $description"
            fi
            ((WARNINGS++))
        fi
    fi
}

# Function to check file/directory exists
check_path() {
    local path=$1
    local type=${2:-"file"} # file or directory
    local required=${3:-true}
    local description=${4:-""}
    
    if [ "$type" = "directory" ]; then
        if [ -d "$path" ]; then
            echo -e "${GREEN}✓${NC} Directory exists: $path"
        else
            if [ "$required" = true ]; then
                echo -e "${RED}✗${NC} Directory NOT FOUND: $path"
                ((MISSING_DEPS++))
            else
                echo -e "${YELLOW}⚠${NC} Directory not found: $path"
                ((WARNINGS++))
            fi
        fi
    else
        if [ -f "$path" ]; then
            echo -e "${GREEN}✓${NC} File exists: $path"
        else
            if [ "$required" = true ]; then
                echo -e "${RED}✗${NC} File NOT FOUND: $path"
                ((MISSING_DEPS++))
            else
                echo -e "${YELLOW}⚠${NC} File not found: $path"
                ((WARNINGS++))
            fi
        fi
    fi
    
    if [ -n "$description" ]; then
        echo "  └─ $description"
    fi
}

echo "Checking required commands..."
echo "=============================="
check_command "cargo" true "Rust build tool"
check_command "curl" true "HTTP client for API testing"
check_command "jq" true "JSON processor for parsing responses"
check_command "git" true "Version control"

echo
echo "Checking optional commands..."
echo "=============================="
check_command "docker" false "Container runtime"
check_command "kubectl" false "Kubernetes CLI (for production)"
check_command "npm" false "Node package manager (for browser tests)"
check_command "bc" false "Calculator (for performance metrics)"
check_command "prometheus" false "Metrics collection"
check_command "grafana-cli" false "Grafana dashboard management"

echo
echo "Checking environment variables..."
echo "================================="
check_env "HAL9_HOME" false "HAL9 base directory"
check_env "ANTHROPIC_API_KEY" false "Claude API key (uses mock if not set)"
check_env "DATABASE_URL" false "Database connection string"
check_env "REDIS_URL" false "Redis connection string"
check_env "LOG_LEVEL" false "Logging verbosity"

echo
echo "Checking production environment variables..."
echo "============================================"
check_env "SLACK_WEBHOOK_URL" false "Slack notifications"
check_env "PAGERDUTY_TOKEN" false "PagerDuty alerts"
check_env "SENTRY_DSN" false "Error tracking"

echo
echo "Checking project structure..."
echo "============================="
# Get the project root (parent of L1_reflexive)
PROJECT_ROOT="$(cd "$(dirname "$0")/.." && pwd)"

check_path "$PROJECT_ROOT/Cargo.toml" "file" true "Root Cargo workspace"
check_path "$PROJECT_ROOT/L5_strategic/research/examples" "directory" true "Configuration examples"
check_path "$PROJECT_ROOT/substrate/storage/databases" "directory" true "Database directory"
check_path "$PROJECT_ROOT/substrate/tooling/rust/legacy-crates" "directory" true "Source code"

echo
echo "Checking build targets..."
echo "========================"
check_path "$PROJECT_ROOT/target/debug/hal9-server" "file" false "Debug server binary"
check_path "$PROJECT_ROOT/target/debug/hal9-cli" "file" false "Debug CLI binary"
check_path "$PROJECT_ROOT/target/release/hal9-server" "file" false "Release server binary"
check_path "$PROJECT_ROOT/target/release/hal9-cli" "file" false "Release CLI binary"

echo
echo "Checking system resources..."
echo "==========================="
# Check available memory
if command -v free &> /dev/null; then
    MEM_AVAILABLE=$(free -m | awk 'NR==2{print $7}')
    if [ "$MEM_AVAILABLE" -lt 1024 ]; then
        echo -e "${YELLOW}⚠${NC} Low memory: ${MEM_AVAILABLE}MB available (recommend 1GB+)"
        ((WARNINGS++))
    else
        echo -e "${GREEN}✓${NC} Memory: ${MEM_AVAILABLE}MB available"
    fi
elif command -v vm_stat &> /dev/null; then
    # macOS
    echo -e "${GREEN}✓${NC} Memory check (macOS)"
else
    echo -e "${YELLOW}⚠${NC} Cannot check memory availability"
    ((WARNINGS++))
fi

# Check disk space
DISK_AVAILABLE=$(df -BG . | awk 'NR==2{print $4}' | sed 's/G//')
if [ "$DISK_AVAILABLE" -lt 5 ]; then
    echo -e "${YELLOW}⚠${NC} Low disk space: ${DISK_AVAILABLE}GB available (recommend 5GB+)"
    ((WARNINGS++))
else
    echo -e "${GREEN}✓${NC} Disk space: ${DISK_AVAILABLE}GB available"
fi

echo
echo "=============================="
echo "Summary:"
echo "=============================="

if [ $MISSING_DEPS -eq 0 ]; then
    echo -e "${GREEN}✓ All required dependencies are satisfied${NC}"
else
    echo -e "${RED}✗ Missing $MISSING_DEPS required dependencies${NC}"
fi

if [ $WARNINGS -gt 0 ]; then
    echo -e "${YELLOW}⚠ $WARNINGS optional dependencies missing${NC}"
fi

echo
echo "Recommendations:"
echo "==============="

if [ $MISSING_DEPS -gt 0 ]; then
    echo "Install missing required dependencies:"
    echo
    if ! command -v cargo &> /dev/null; then
        echo "  # Install Rust:"
        echo "  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    fi
    if ! command -v jq &> /dev/null; then
        echo "  # Install jq:"
        echo "  # macOS: brew install jq"
        echo "  # Ubuntu: sudo apt-get install jq"
    fi
fi

if [ ! -d "$PROJECT_ROOT/target/debug" ]; then
    echo
    echo "Build the project:"
    echo "  cd $PROJECT_ROOT"
    echo "  cargo build"
fi

if [ -z "${HAL9_HOME:-}" ]; then
    echo
    echo "Set up environment:"
    echo "  export HAL9_HOME='$PROJECT_ROOT'"
    echo "  export HAL9_CONFIG_DIR='\$HAL9_HOME/L5_strategic/research/examples'"
    echo "  export HAL9_DATA_DIR='\$HAL9_HOME/substrate/storage/databases'"
fi

echo
exit $MISSING_DEPS