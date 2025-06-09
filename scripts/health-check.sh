#!/bin/bash

# HAL9 Comprehensive Health Check Script
# Performs deep health validation of the hierarchical architecture

set -e

# Configuration
NAMESPACE="hal9"
COMPREHENSIVE=${1:-"--quick"}
OUTPUT_FORMAT=${2:-"text"} # text or json

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Health check results
declare -A health_results
overall_health="healthy"

# Function to record health result
record_result() {
    local component=$1
    local status=$2
    local message=$3
    
    health_results["$component"]="$status:$message"
    
    if [ "$status" != "healthy" ]; then
        overall_health="unhealthy"
    fi
}

# Function to check layer health
check_layer_health() {
    local layer=$1
    local layer_health="healthy"
    local message=""
    
    # Get all pods for this layer
    local pods=$(kubectl get pods -n $NAMESPACE -l layer=$layer -o jsonpath='{.items[*].metadata.name}')
    
    if [ -z "$pods" ]; then
        record_result "layer_$layer" "critical" "No pods found"
        return
    fi
    
    # Check each pod
    local total_pods=0
    local healthy_pods=0
    
    for pod in $pods; do
        total_pods=$((total_pods + 1))
        
        # Check pod status
        local status=$(kubectl get pod $pod -n $NAMESPACE -o jsonpath='{.status.phase}')
        if [ "$status" == "Running" ]; then
            # Check health endpoint
            if kubectl exec -n $NAMESPACE $pod -- curl -s -f http://localhost:8080/health > /dev/null 2>&1; then
                healthy_pods=$((healthy_pods + 1))
            fi
        fi
    done
    
    if [ $healthy_pods -eq $total_pods ]; then
        message="All $total_pods pods healthy"
    elif [ $healthy_pods -gt 0 ]; then
        layer_health="degraded"
        message="$healthy_pods/$total_pods pods healthy"
    else
        layer_health="critical"
        message="No healthy pods"
    fi
    
    record_result "layer_$layer" "$layer_health" "$message"
}

# Function to check signal propagation
check_signal_propagation() {
    local orchestrator_pod=$(kubectl get pod -n $NAMESPACE -l component=orchestrator -o jsonpath='{.items[0].metadata.name}' 2>/dev/null)
    
    if [ -z "$orchestrator_pod" ]; then
        record_result "signal_propagation" "critical" "Orchestrator not found"
        return
    fi
    
    # Send test signal and measure propagation time
    local result=$(kubectl exec -n $NAMESPACE $orchestrator_pod -- hal9-cli test signal --measure-latency 2>/dev/null || echo "error")
    
    if [[ "$result" == "error" ]]; then
        record_result "signal_propagation" "critical" "Test failed"
    else
        local latency=$(echo "$result" | grep -oP 'latency: \K\d+')
        if [ "$latency" -lt 10 ]; then
            record_result "signal_propagation" "healthy" "Latency: ${latency}ms"
        elif [ "$latency" -lt 50 ]; then
            record_result "signal_propagation" "warning" "Latency: ${latency}ms (elevated)"
        else
            record_result "signal_propagation" "critical" "Latency: ${latency}ms (high)"
        fi
    fi
}

# Function to check resource usage
check_resource_usage() {
    local high_cpu_pods=0
    local high_memory_pods=0
    
    # Get resource usage for all pods
    local pods=$(kubectl get pods -n $NAMESPACE -l app=hal9 -o jsonpath='{.items[*].metadata.name}')
    
    for pod in $pods; do
        # Get CPU and memory usage
        local usage=$(kubectl top pod $pod -n $NAMESPACE --no-headers 2>/dev/null || echo "0m 0Mi")
        local cpu=$(echo $usage | awk '{print $2}' | sed 's/m//')
        local memory=$(echo $usage | awk '{print $3}' | sed 's/Mi//')
        
        # Check thresholds (CPU > 80%, Memory > 80% of limits)
        if [ "$cpu" -gt 800 ]; then
            high_cpu_pods=$((high_cpu_pods + 1))
        fi
        
        if [ "$memory" -gt 3276 ]; then  # Assuming 4Gi limit
            high_memory_pods=$((high_memory_pods + 1))
        fi
    done
    
    if [ $high_cpu_pods -eq 0 ] && [ $high_memory_pods -eq 0 ]; then
        record_result "resource_usage" "healthy" "All pods within limits"
    elif [ $high_cpu_pods -lt 5 ] && [ $high_memory_pods -lt 5 ]; then
        record_result "resource_usage" "warning" "CPU: $high_cpu_pods pods high, Memory: $high_memory_pods pods high"
    else
        record_result "resource_usage" "critical" "CPU: $high_cpu_pods pods high, Memory: $high_memory_pods pods high"
    fi
}

# Function to check database health
check_database_health() {
    local db_pod=$(kubectl get pod -n $NAMESPACE -l component=database -o jsonpath='{.items[0].metadata.name}' 2>/dev/null)
    
    if [ -z "$db_pod" ]; then
        # Try using application pod to check database
        db_pod=$(kubectl get pod -n $NAMESPACE -l layer=substrate -o jsonpath='{.items[0].metadata.name}' 2>/dev/null)
    fi
    
    if [ -z "$db_pod" ]; then
        record_result "database" "critical" "Cannot access database"
        return
    fi
    
    # Check PostgreSQL
    if kubectl exec -n $NAMESPACE $db_pod -- pg_isready -h postgres -p 5432 > /dev/null 2>&1; then
        # Check replication lag
        local lag=$(kubectl exec -n $NAMESPACE $db_pod -- psql -h postgres -U hal9 -d hal9 -t -c "SELECT EXTRACT(EPOCH FROM (now() - pg_last_xact_replay_timestamp()))::int" 2>/dev/null || echo "999")
        
        if [ "$lag" -lt 5 ]; then
            record_result "database" "healthy" "PostgreSQL OK, replication lag: ${lag}s"
        else
            record_result "database" "warning" "PostgreSQL OK, replication lag: ${lag}s (high)"
        fi
    else
        record_result "database" "critical" "PostgreSQL connection failed"
    fi
    
    # Check Redis
    if kubectl exec -n $NAMESPACE $db_pod -- redis-cli -h redis ping > /dev/null 2>&1; then
        record_result "redis" "healthy" "Redis OK"
    else
        record_result "redis" "critical" "Redis connection failed"
    fi
}

# Function to check consensus mechanism
check_consensus() {
    local protocol_pod=$(kubectl get pod -n $NAMESPACE -l layer=protocol -o jsonpath='{.items[0].metadata.name}' 2>/dev/null)
    
    if [ -z "$protocol_pod" ]; then
        record_result "consensus" "critical" "Protocol layer not found"
        return
    fi
    
    # Check consensus status
    local consensus_status=$(kubectl exec -n $NAMESPACE $protocol_pod -- hal9-cli consensus status 2>/dev/null || echo "error")
    
    if [[ "$consensus_status" == *"active"* ]]; then
        record_result "consensus" "healthy" "Consensus mechanism active"
    elif [[ "$consensus_status" == *"degraded"* ]]; then
        record_result "consensus" "warning" "Consensus degraded"
    else
        record_result "consensus" "critical" "Consensus inactive"
    fi
}

# Function to check learning system
check_learning_system() {
    local cognitive_pod=$(kubectl get pod -n $NAMESPACE -l layer=cognitive -o jsonpath='{.items[0].metadata.name}' 2>/dev/null)
    
    if [ -z "$cognitive_pod" ]; then
        record_result "learning" "critical" "Cognitive layer not found"
        return
    fi
    
    # Check learning metrics
    local learning_rate=$(kubectl exec -n $NAMESPACE $cognitive_pod -- hal9-cli learning metrics --format json | jq -r '.learning_rate' 2>/dev/null || echo "0")
    
    if (( $(echo "$learning_rate > 0" | bc -l) )); then
        record_result "learning" "healthy" "Learning rate: $learning_rate"
    else
        record_result "learning" "warning" "Learning inactive"
    fi
}

# Main health check flow
echo -e "${BLUE}HAL9 Health Check${NC}"
echo -e "${BLUE}=================${NC}"
echo "Mode: $COMPREHENSIVE"
echo

# Quick health checks (always run)
echo "Running basic health checks..."

# Check all layers
LAYERS=("substrate" "protocol" "l1-reflexive" "l2-implementation" 
        "l3-operational" "l4-tactical" "l5-strategic" "orchestration" "intelligence")

for layer in "${LAYERS[@]}"; do
    check_layer_health $layer
done

# Check core functionality
check_signal_propagation
check_resource_usage
check_database_health

# Comprehensive checks
if [ "$COMPREHENSIVE" == "--comprehensive" ]; then
    echo
    echo "Running comprehensive health checks..."
    
    check_consensus
    check_learning_system
    
    # Additional comprehensive checks
    # Check metrics collection
    local metrics_up=$(curl -s http://prometheus:9090/api/v1/query?query=up | jq -r '.data.result | length' 2>/dev/null || echo "0")
    if [ "$metrics_up" -gt 0 ]; then
        record_result "metrics" "healthy" "$metrics_up targets up"
    else
        record_result "metrics" "critical" "No metrics targets"
    fi
    
    # Check log aggregation
    local logs_count=$(curl -s http://elasticsearch:9200/_cat/count/hal9-* | awk '{print $3}' 2>/dev/null || echo "0")
    if [ "$logs_count" -gt 0 ]; then
        record_result "logging" "healthy" "$logs_count log entries"
    else
        record_result "logging" "warning" "No recent logs"
    fi
fi

# Output results
echo
echo -e "${BLUE}Health Check Results${NC}"
echo -e "${BLUE}===================${NC}"

if [ "$OUTPUT_FORMAT" == "json" ]; then
    # JSON output
    echo "{"
    echo "  \"timestamp\": \"$(date -u +%Y-%m-%dT%H:%M:%SZ)\","
    echo "  \"overall_health\": \"$overall_health\","
    echo "  \"components\": {"
    
    first=true
    for component in "${!health_results[@]}"; do
        IFS=':' read -r status message <<< "${health_results[$component]}"
        if [ "$first" != true ]; then echo ","; fi
        echo -n "    \"$component\": {\"status\": \"$status\", \"message\": \"$message\"}"
        first=false
    done
    
    echo
    echo "  }"
    echo "}"
else
    # Text output
    for component in "${!health_results[@]}"; do
        IFS=':' read -r status message <<< "${health_results[$component]}"
        
        case $status in
            "healthy")
                echo -e "${GREEN}✓${NC} $component: $message"
                ;;
            "warning")
                echo -e "${YELLOW}⚠${NC} $component: $message"
                ;;
            "critical")
                echo -e "${RED}✗${NC} $component: $message"
                ;;
        esac
    done
    
    echo
    echo -e "Overall Health: $([ "$overall_health" == "healthy" ] && echo -e "${GREEN}HEALTHY${NC}" || echo -e "${RED}UNHEALTHY${NC}")"
fi

# Exit with appropriate code
[ "$overall_health" == "healthy" ] && exit 0 || exit 1