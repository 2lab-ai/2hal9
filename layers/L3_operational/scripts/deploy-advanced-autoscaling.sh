#!/bin/bash
#
# Deploy advanced autoscaling configuration
# Run this to enable better scaling behavior
#

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

NAMESPACE="${HAL9_K8S_NAMESPACE:-hal9}"
AUTOSCALING_CONFIG="/Users/icedac/2lab.ai/2hal9/L3_operational/architecture/kubernetes/autoscaling-optimized.yaml"

echo "🚀 Deploying Advanced Autoscaling Configuration"
echo "Namespace: $NAMESPACE"
echo ""

# Check kubectl access
if ! kubectl cluster-info &> /dev/null; then
    echo -e "${RED}✗ Cannot access Kubernetes cluster${NC}"
    echo "Please check your kubeconfig or VPN connection"
    exit 1
fi

# Check if namespace exists
if ! kubectl get namespace $NAMESPACE &> /dev/null; then
    echo -e "${YELLOW}⚠ Namespace $NAMESPACE doesn't exist, creating...${NC}"
    kubectl create namespace $NAMESPACE
fi

# Check current HPA status
echo "📊 Current autoscaling status:"
kubectl get hpa -n $NAMESPACE 2>/dev/null || echo "No HPA found"
echo ""

# Deploy advanced autoscaling
echo "🔧 Applying advanced autoscaling configuration..."
if kubectl apply -f "$AUTOSCALING_CONFIG"; then
    echo -e "${GREEN}✓ Advanced autoscaling deployed successfully${NC}"
else
    echo -e "${RED}✗ Failed to deploy autoscaling${NC}"
    exit 1
fi

# Verify deployment
echo ""
echo "✅ Verifying deployment..."
sleep 2

# Check HPA
if kubectl get hpa hal9-hpa-advanced -n $NAMESPACE &> /dev/null; then
    echo -e "${GREEN}✓ HPA deployed${NC}"
    kubectl get hpa hal9-hpa-advanced -n $NAMESPACE
else
    echo -e "${RED}✗ HPA not found${NC}"
fi

# Check VPA
if kubectl get vpa hal9-vpa -n $NAMESPACE &> /dev/null; then
    echo -e "${GREEN}✓ VPA deployed${NC}"
else
    echo -e "${YELLOW}⚠ VPA not found (may need VPA addon installed)${NC}"
fi

# Check PDB
if kubectl get pdb hal9-pdb -n $NAMESPACE &> /dev/null; then
    echo -e "${GREEN}✓ PodDisruptionBudget deployed${NC}"
else
    echo -e "${RED}✗ PDB not found${NC}"
fi

# Show recommendations
echo ""
echo "📋 Next steps:"
echo "1. Monitor autoscaling behavior: watch kubectl get hpa -n $NAMESPACE"
echo "2. Check metrics server: kubectl top nodes && kubectl top pods -n $NAMESPACE"
echo "3. Test scaling: hey -z 30s -c 100 http://your-hal9-endpoint/api/v1/status"
echo ""
echo -e "${GREEN}✅ Advanced autoscaling configuration deployed!${NC}"
echo ""
echo "⚡ Performance tips:"
echo "- The new config scales more aggressively (100% increase allowed)"
echo "- Scale-down is conservative (10 min stabilization)"
echo "- Circuit breaker metrics will trigger scaling"
echo "- VPA will right-size pods over time"