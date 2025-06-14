#!/bin/bash

# HAL9 Deployment Verification Script
# Validates that a deployment is healthy and ready for traffic

set -e

# Configuration
DEPLOYMENT_ENV=${1:-"blue"}
NAMESPACE="hal9"
TIMEOUT=300
REQUIRED_LAYERS=("substrate" "protocol" "l1-reflexive" "l2-implementation" 
                 "l3-operational" "l4-tactical" "l5-strategic" "orchestration" "intelligence")

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo "=== HAL9 Deployment Verification ==="
echo "Environment: $DEPLOYMENT_ENV"
echo "Namespace: $NAMESPACE"
echo

# Function to check if a deployment is ready
check_deployment() {
    local deployment=$1
    local ready=$(kubectl get deployment $deployment -n $NAMESPACE -o jsonpath='{.status.readyReplicas}' 2>/dev/null || echo "0")
    local desired=$(kubectl get deployment $deployment -n $NAMESPACE -o jsonpath='{.spec.replicas}' 2>/dev/null || echo "0")
    
    if [ "$ready" == "$desired" ] && [ "$ready" -gt 0 ]; then
        return 0
    else
        return 1
    fi
}

# Function to check layer health
check_layer_health() {
    local layer=$1
    local pod=$(kubectl get pod -n $NAMESPACE -l layer=$layer,environment=$DEPLOYMENT_ENV -o jsonpath='{.items[0].metadata.name}' 2>/dev/null)
    
    if [ -z "$pod" ]; then
        return 1
    fi
    
    # Check health endpoint
    kubectl exec -n $NAMESPACE $pod -- curl -s -f http://localhost:8080/health > /dev/null 2>&1
    return $?
}

# Function to verify inter-layer communication
check_layer_communication() {
    local from_layer=$1
    local to_layer=$2
    local pod=$(kubectl get pod -n $NAMESPACE -l layer=$from_layer,environment=$DEPLOYMENT_ENV -o jsonpath='{.items[0].metadata.name}' 2>/dev/null)
    
    if [ -z "$pod" ]; then
        return 1
    fi
    
    # Test connection to target layer
    kubectl exec -n $NAMESPACE $pod -- nc -zv hal9-$to_layer-$DEPLOYMENT_ENV 8080 > /dev/null 2>&1
    return $?
}

# 1. Check all deployments are ready
echo "Checking deployments..."
all_ready=true
for layer in "${REQUIRED_LAYERS[@]}"; do
    deployment="hal9-$layer-$DEPLOYMENT_ENV"
    if check_deployment $deployment; then
        echo -e "${GREEN}✓${NC} $deployment is ready"
    else
        echo -e "${RED}✗${NC} $deployment is not ready"
        all_ready=false
    fi
done

if [ "$all_ready" != "true" ]; then
    echo -e "${RED}Not all deployments are ready. Exiting.${NC}"
    exit 1
fi

# 2. Check layer health
echo
echo "Checking layer health..."
all_healthy=true
for layer in "${REQUIRED_LAYERS[@]}"; do
    if check_layer_health $layer; then
        echo -e "${GREEN}✓${NC} $layer is healthy"
    else
        echo -e "${RED}✗${NC} $layer health check failed"
        all_healthy=false
    fi
done

if [ "$all_healthy" != "true" ]; then
    echo -e "${RED}Not all layers are healthy. Exiting.${NC}"
    exit 1
fi

# 3. Check inter-layer communication
echo
echo "Checking inter-layer communication..."
communication_ok=true

# Define layer communication paths
declare -A layer_connections=(
    ["substrate"]="protocol"
    ["protocol"]="l1-reflexive"
    ["l1-reflexive"]="l2-implementation"
    ["l2-implementation"]="l3-operational"
    ["l3-operational"]="l4-tactical"
    ["l4-tactical"]="l5-strategic"
)

for from_layer in "${!layer_connections[@]}"; do
    to_layer=${layer_connections[$from_layer]}
    if check_layer_communication $from_layer $to_layer; then
        echo -e "${GREEN}✓${NC} $from_layer → $to_layer communication OK"
    else
        echo -e "${RED}✗${NC} $from_layer → $to_layer communication FAILED"
        communication_ok=false
    fi
done

if [ "$communication_ok" != "true" ]; then
    echo -e "${RED}Layer communication issues detected. Exiting.${NC}"
    exit 1
fi

# 4. Check database connectivity
echo
echo "Checking database connectivity..."
db_pod=$(kubectl get pod -n $NAMESPACE -l layer=substrate,environment=$DEPLOYMENT_ENV -o jsonpath='{.items[0].metadata.name}' 2>/dev/null)
if kubectl exec -n $NAMESPACE $db_pod -- pg_isready -h $DB_HOST -p 5432 > /dev/null 2>&1; then
    echo -e "${GREEN}✓${NC} Database connection OK"
else
    echo -e "${RED}✗${NC} Database connection FAILED"
    exit 1
fi

# 5. Check Redis connectivity
echo
echo "Checking Redis connectivity..."
if kubectl exec -n $NAMESPACE $db_pod -- redis-cli -h $REDIS_HOST ping > /dev/null 2>&1; then
    echo -e "${GREEN}✓${NC} Redis connection OK"
else
    echo -e "${RED}✗${NC} Redis connection FAILED"
    exit 1
fi

# 6. Run smoke tests
echo
echo "Running smoke tests..."
smoke_test_pod="hal9-smoke-test-$RANDOM"
kubectl run $smoke_test_pod \
    --image=hal9/smoke-tests:latest \
    --restart=Never \
    --rm \
    -i \
    --namespace=$NAMESPACE \
    --env="TARGET_ENV=$DEPLOYMENT_ENV" \
    -- /smoke-tests.sh

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✓${NC} Smoke tests passed"
else
    echo -e "${RED}✗${NC} Smoke tests failed"
    exit 1
fi

# 7. Check metrics are being collected
echo
echo "Checking metrics collection..."
metrics_query="sum(up{namespace=\"$NAMESPACE\",environment=\"$DEPLOYMENT_ENV\"})"
metric_count=$(curl -s "http://prometheus:9090/api/v1/query?query=$metrics_query" | jq -r '.data.result[0].value[1]' 2>/dev/null || echo "0")

if [ "$metric_count" -gt "0" ]; then
    echo -e "${GREEN}✓${NC} Metrics are being collected ($metric_count targets up)"
else
    echo -e "${RED}✗${NC} No metrics being collected"
    exit 1
fi

# 8. Final validation
echo
echo "=== Deployment Verification Complete ==="
echo -e "${GREEN}All checks passed!${NC}"
echo
echo "Summary:"
echo "- All deployments ready: ✓"
echo "- All layers healthy: ✓"
echo "- Inter-layer communication: ✓"
echo "- Database connectivity: ✓"
echo "- Redis connectivity: ✓"
echo "- Smoke tests: ✓"
echo "- Metrics collection: ✓"
echo
echo "Deployment $DEPLOYMENT_ENV is ready for traffic!"

exit 0