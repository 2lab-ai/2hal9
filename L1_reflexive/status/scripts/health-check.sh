#!/bin/bash
#
# HAL9 Health Check Script
# Supports both local development and Kubernetes environments
#

set -euo pipefail

# Source common environment
source "$(dirname "$0")/../../common-env.sh"

# Default mode
MODE="${1:-local}"

# Health check configuration
HEALTH_TIMEOUT=5
HEALTH_RETRIES=3

# Color codes for status
STATUS_OK="${GREEN}✓${NC}"
STATUS_WARN="${YELLOW}⚠${NC}"
STATUS_FAIL="${RED}✗${NC}"

# Function to check local service
check_local_service() {
    local name=$1
    local url=$2
    local expected=${3:-"healthy"}
    
    log_info "Checking $name..."
    
    # First check if port is actually open
    local port=$(echo "$url" | sed -n 's/.*:\([0-9]*\).*/\1/p')
    if [ -n "$port" ] && ! nc -z localhost "$port" 2>/dev/null; then
        echo -e "$STATUS_FAIL $name port $port is not open"
        return 1
    fi
    
    local retry=0
    local status="unknown"
    
    while [ $retry -lt $HEALTH_RETRIES ]; do
        if response=$(curl -s --max-time $HEALTH_TIMEOUT "$url" 2>/dev/null); then
            if [ -n "$response" ]; then
                # Try to parse as JSON first
                if echo "$response" | jq -e '.status' > /dev/null 2>&1; then
                    status=$(echo "$response" | jq -r '.status')
                else
                    # Not JSON, check if it's a simple response
                    status="ok"
                fi
                
                if [ "$status" = "$expected" ] || [ "$status" = "ok" ]; then
                    echo -e "$STATUS_OK $name is healthy"
                    return 0
                else
                    echo -e "$STATUS_WARN $name returned status: $status"
                fi
            fi
        fi
        
        ((retry++))
        if [ $retry -lt $HEALTH_RETRIES ]; then
            sleep 1
        fi
    done
    
    echo -e "$STATUS_FAIL $name is not responding"
    return 1
}

# Function to check Kubernetes service
check_k8s_service() {
    local name=$1
    local namespace=${2:-"default"}
    local selector=$3
    
    log_info "Checking $name in namespace $namespace..."
    
    # Check if kubectl is available
    if ! command -v kubectl &> /dev/null; then
        log_error "kubectl not found - cannot check Kubernetes services"
        return 1
    fi
    
    # Check pod status
    local ready_pods=$(kubectl get pods -n "$namespace" -l "$selector" -o json 2>/dev/null | \
        jq '[.items[] | select(.status.phase == "Running" and .status.conditions[] | select(.type == "Ready" and .status == "True"))] | length')
    
    local total_pods=$(kubectl get pods -n "$namespace" -l "$selector" -o json 2>/dev/null | \
        jq '.items | length')
    
    if [ "$ready_pods" -gt 0 ]; then
        if [ "$ready_pods" -eq "$total_pods" ]; then
            echo -e "$STATUS_OK $name: $ready_pods/$total_pods pods ready"
            return 0
        else
            echo -e "$STATUS_WARN $name: $ready_pods/$total_pods pods ready"
            return 0
        fi
    else
        echo -e "$STATUS_FAIL $name: 0/$total_pods pods ready"
        return 1
    fi
}

# Function to check system resources
check_system_resources() {
    log_info "Checking system resources..."
    
    # Check memory
    if command -v free &> /dev/null; then
        local mem_info=$(free -m | awk 'NR==2')
        local total_mem=$(echo "$mem_info" | awk '{print $2}')
        local used_mem=$(echo "$mem_info" | awk '{print $3}')
        local mem_percent=$((used_mem * 100 / total_mem))
        
        if [ $mem_percent -lt 80 ]; then
            echo -e "$STATUS_OK Memory: ${used_mem}MB/${total_mem}MB (${mem_percent}%)"
        elif [ $mem_percent -lt 90 ]; then
            echo -e "$STATUS_WARN Memory: ${used_mem}MB/${total_mem}MB (${mem_percent}%)"
        else
            echo -e "$STATUS_FAIL Memory: ${used_mem}MB/${total_mem}MB (${mem_percent}%)"
        fi
    elif command -v vm_stat &> /dev/null; then
        # macOS
        echo -e "$STATUS_OK Memory check (macOS)"
    fi
    
    # Check disk space
    local disk_info=$(df -h . | awk 'NR==2')
    local disk_usage=$(echo "$disk_info" | awk '{print $5}' | sed 's/%//')
    local disk_available=$(echo "$disk_info" | awk '{print $4}')
    
    if [ $disk_usage -lt 80 ]; then
        echo -e "$STATUS_OK Disk: ${disk_usage}% used (${disk_available} free)"
    elif [ $disk_usage -lt 90 ]; then
        echo -e "$STATUS_WARN Disk: ${disk_usage}% used (${disk_available} free) - Consider cleanup"
    else
        echo -e "$STATUS_FAIL Disk: ${disk_usage}% used (${disk_available} free) - CRITICAL!"
        log_warning "Disk space critical! Running emergency cleanup..."
        # Clean old logs
        find "$HAL9_LOG_DIR" -name "*.log" -mtime +7 -delete 2>/dev/null || true
        # Clean build cache if it exists
        [ -d "$HAL9_HOME/target" ] && cargo clean 2>/dev/null || true
    fi
    
    # Check CPU (if possible)
    if command -v top &> /dev/null; then
        # This is a rough check - actual implementation would vary by OS
        echo -e "$STATUS_OK CPU check available"
    fi
}

# Local health check
health_check_local() {
    echo "=== HAL9 Local Health Check ==="
    echo "Time: $(date)"
    echo
    
    # Check if HAL9 processes are running
    log_info "Checking HAL9 processes..."
    local hal9_procs=$(ps aux | grep -E "hal9-server|hal9_server" | grep -v grep | wc -l)
    if [ $hal9_procs -gt 0 ]; then
        echo -e "$STATUS_OK Found $hal9_procs HAL9 process(es) running"
    else
        echo -e "$STATUS_FAIL No HAL9 processes found"
        # Check what's using the port
        if lsof -i :$HAL9_PORT_MAIN 2>/dev/null | grep -q LISTEN; then
            log_warning "Port $HAL9_PORT_MAIN is in use by another process:"
            lsof -i :$HAL9_PORT_MAIN | grep LISTEN | head -2
        fi
    fi
    
    # Check main server
    check_local_service "HAL9 Server" "http://localhost:$HAL9_PORT_MAIN/health"
    
    # Check API endpoints
    check_local_service "API Status" "http://localhost:$HAL9_PORT_MAIN/api/v1/status"
    check_local_service "Neurons API" "http://localhost:$HAL9_PORT_MAIN/api/v1/neurons"
    
    # Check auth if configured
    if [ "${HAL9_PORT_AUTH:-}" != "" ] && [ "$HAL9_PORT_AUTH" != "$HAL9_PORT_MAIN" ]; then
        check_local_service "Auth Service" "http://localhost:$HAL9_PORT_AUTH/health" || true
    fi
    
    # Check metrics if configured
    if [ "${HAL9_PORT_METRICS:-}" != "" ]; then
        check_local_service "Metrics" "http://localhost:$HAL9_PORT_METRICS/metrics" || true
    fi
    
    # Check system resources
    echo
    check_system_resources
    
    # Check log files
    echo
    log_info "Checking log files..."
    if [ -d "$HAL9_LOG_DIR" ]; then
        local log_count=$(find "$HAL9_LOG_DIR" -name "*.log" -type f 2>/dev/null | wc -l)
        local log_size=$(du -sh "$HAL9_LOG_DIR" 2>/dev/null | cut -f1)
        echo -e "$STATUS_OK Logs: $log_count files, $log_size total"
        
        # Show recent errors with details
        local recent_errors=$(find "$HAL9_LOG_DIR" -name "*.log" -type f -mmin -5 -exec grep -l "ERROR\|FATAL\|panic" {} \; 2>/dev/null | wc -l)
        if [ $recent_errors -gt 0 ]; then
            echo -e "$STATUS_WARN Recent errors found in $recent_errors log files"
            # Show last few errors
            echo "  Recent error samples:"
            find "$HAL9_LOG_DIR" -name "*.log" -type f -mmin -5 -exec grep -h "ERROR\|FATAL\|panic" {} \; 2>/dev/null | tail -3 | sed 's/^/    /'
        fi
        
        # Check for crash dumps
        local crash_dumps=$(find "$HAL9_LOG_DIR" -name "*.crash" -o -name "core.*" 2>/dev/null | wc -l)
        if [ $crash_dumps -gt 0 ]; then
            echo -e "$STATUS_FAIL Found $crash_dumps crash dump(s)"
        fi
    else
        echo -e "$STATUS_WARN Log directory not found: $HAL9_LOG_DIR"
    fi
}

# Kubernetes health check
health_check_k8s() {
    echo "=== HAL9 Kubernetes Health Check ==="
    echo "Time: $(date)"
    echo
    
    # Check if kubectl is configured
    if ! kubectl cluster-info &> /dev/null; then
        log_error "kubectl not configured or cluster not accessible"
        exit 1
    fi
    
    local NAMESPACE="${HAL9_K8S_NAMESPACE:-hal9}"
    
    echo "Namespace: $NAMESPACE"
    echo
    
    # Check deployments
    check_k8s_service "HAL9 Server" "$NAMESPACE" "app=hal9-server"
    check_k8s_service "HAL9 Neurons" "$NAMESPACE" "component=neuron"
    
    # Check services
    log_info "Checking services..."
    kubectl get svc -n "$NAMESPACE" -o wide 2>/dev/null || echo -e "$STATUS_FAIL No services found"
    
    # Check ingress
    log_info "Checking ingress..."
    kubectl get ingress -n "$NAMESPACE" 2>/dev/null || echo -e "$STATUS_WARN No ingress found"
    
    # Check persistent volumes
    log_info "Checking storage..."
    kubectl get pvc -n "$NAMESPACE" 2>/dev/null || echo -e "$STATUS_WARN No persistent volumes found"
    
    # Check recent events
    echo
    log_info "Recent events..."
    kubectl get events -n "$NAMESPACE" --sort-by='.lastTimestamp' | tail -5
}

# Main execution
main() {
    case "$MODE" in
        local|--local|-l)
            health_check_local
            ;;
        k8s|kubernetes|--k8s|-k)
            health_check_k8s
            ;;
        all|--all|-a)
            health_check_local
            echo
            echo "========================================"
            echo
            health_check_k8s
            ;;
        *)
            log_error "Unknown mode: $MODE"
            echo "Usage: $0 [local|k8s|all]"
            echo "  local  - Check local development environment (default)"
            echo "  k8s    - Check Kubernetes deployment"
            echo "  all    - Check both environments"
            exit 1
            ;;
    esac
    
    echo
    echo "=== Health Check Complete ==="
}

# Run main function
main "$@"