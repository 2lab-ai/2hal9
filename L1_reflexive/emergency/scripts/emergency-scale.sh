#!/bin/bash
#
# Emergency scaling script for HAL9 production incidents
# For when autoscaling isn't fast enough (시발!)
#

set -euo pipefail

# Source common environment
source "$(dirname "$0")/../../common-env.sh"

# Emergency settings
NAMESPACE="${HAL9_K8S_NAMESPACE:-hal9}"
EMERGENCY_REPLICAS="${1:-20}"  # Default to 20 replicas
DEPLOYMENT="hal9-server"

# ASCII art for 3am motivation
echo -e "${MAGENTA}"
cat << 'EOF'
  ____  ____    _    _     _____ _ 
 / ___| / ___|  / \  | |   | ____| |
 \___ \| |     / _ \ | |   |  _| | |
  ___) | |___ / ___ \| |___| |___|_|
 |____/ \____/_/   \_\_____|_____(_)
         Emergency Scaling 🚀
EOF
echo -e "${NC}"

# Check if kubectl is available
if ! command -v kubectl &> /dev/null; then
    log_error "kubectl not found! This script requires kubectl"
    log_info "For local emergency, try:"
    log_info "  pkill -f hal9-server"
    log_info "  HAL9_MAX_WORKERS=20 cargo run --release --bin hal9-server"
    exit 1
fi

# Check cluster access
if ! kubectl cluster-info &> /dev/null; then
    log_error "Cannot access Kubernetes cluster!"
    log_info "Check your kubeconfig or VPN connection"
    exit 1
fi

# Current status
log_info "Checking current status..."
CURRENT_REPLICAS=$(kubectl get deployment $DEPLOYMENT -n $NAMESPACE -o jsonpath='{.spec.replicas}' 2>/dev/null || echo "0")
READY_REPLICAS=$(kubectl get deployment $DEPLOYMENT -n $NAMESPACE -o jsonpath='{.status.readyReplicas}' 2>/dev/null || echo "0")

echo ""
echo "Current state:"
echo "  Deployment: $DEPLOYMENT"
echo "  Namespace: $NAMESPACE"
echo "  Current replicas: $CURRENT_REPLICAS"
echo "  Ready replicas: $READY_REPLICAS"
echo ""

# Emergency scale
log_warning "EMERGENCY SCALING TO $EMERGENCY_REPLICAS REPLICAS!"

# Scale immediately
kubectl scale deployment $DEPLOYMENT -n $NAMESPACE --replicas=$EMERGENCY_REPLICAS

# Also patch HPA to allow more replicas temporarily
log_info "Updating HPA limits..."
kubectl patch hpa hal9-hpa -n $NAMESPACE --type='json' \
  -p='[{"op": "replace", "path": "/spec/minReplicas", "value":'$EMERGENCY_REPLICAS'}]' || true

# Force rollout if pods are stuck
log_info "Forcing rollout restart..."
kubectl rollout restart deployment/$DEPLOYMENT -n $NAMESPACE

# Monitor the scaling
log_info "Monitoring scale-up progress..."
echo ""

for i in {1..30}; do
    READY=$(kubectl get deployment $DEPLOYMENT -n $NAMESPACE -o jsonpath='{.status.readyReplicas}' 2>/dev/null || echo "0")
    DESIRED=$(kubectl get deployment $DEPLOYMENT -n $NAMESPACE -o jsonpath='{.spec.replicas}' 2>/dev/null || echo "0")
    
    echo -ne "\rProgress: $READY/$DESIRED pods ready... "
    
    if [ "$READY" -ge "$EMERGENCY_REPLICAS" ]; then
        echo ""
        log_success "Emergency scaling complete! $READY pods ready"
        break
    fi
    
    sleep 2
done

echo ""

# Check pod events for issues
log_info "Checking for pod issues..."
kubectl get events -n $NAMESPACE --field-selector involvedObject.kind=Pod \
  --sort-by='.lastTimestamp' | tail -10

# Generate recovery commands
cat > /tmp/hal9-emergency-recovery.sh << EOF
#!/bin/bash
# HAL9 Emergency Recovery Commands
# Generated at $(date)

# 1. Check current status
kubectl get pods -n $NAMESPACE -l app=hal9

# 2. Check resource usage
kubectl top pods -n $NAMESPACE -l app=hal9

# 3. Check recent errors
kubectl logs -n $NAMESPACE -l app=hal9 --tail=50 | grep -i error

# 4. Scale back to normal after incident
kubectl scale deployment $DEPLOYMENT -n $NAMESPACE --replicas=5
kubectl patch hpa hal9-hpa -n $NAMESPACE --type='json' \
  -p='[{"op": "replace", "path": "/spec/minReplicas", "value":5}]'

# 5. If nothing works
echo "Wake up: Zhugehyuk"
echo "아 시발 아 컴퓨터네 우주가"
EOF

chmod +x /tmp/hal9-emergency-recovery.sh

echo ""
echo "=================================="
echo "🚨 EMERGENCY SCALING ACTIVE 🚨"
echo "=================================="
echo ""
echo "Next steps:"
echo "1. Monitor metrics: kubectl top pods -n $NAMESPACE"
echo "2. Check logs: kubectl logs -n $NAMESPACE -l app=hal9 --tail=50"
echo "3. Recovery script: /tmp/hal9-emergency-recovery.sh"
echo ""
echo "Remember to scale back down after the incident!"
echo ""

# Performance tips
echo "Quick fixes while pods scale up:"
echo "- Enable caching if not already: redis-cli SET hal9:cache:enabled true"
echo "- Increase rate limits: kubectl set env deployment/$DEPLOYMENT RATE_LIMIT=1000 -n $NAMESPACE"
echo "- Enable circuit breakers: kubectl set env deployment/$DEPLOYMENT CIRCUIT_BREAKER=true -n $NAMESPACE"
echo ""

log_success "Emergency scaling initiated. Good luck! 🍀"