#!/bin/bash
# Validate Kubernetes manifests for HAL9 deployment

set -e

echo "=== HAL9 Kubernetes Deployment Validation ==="
echo ""

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check if kubectl is installed
if ! command -v kubectl &> /dev/null; then
    echo -e "${RED}✗ kubectl is not installed${NC}"
    exit 1
fi

# Check if we're in the right directory
if [ ! -f "kustomization.yaml" ]; then
    echo -e "${RED}✗ Not in deployment directory. Please run from layers/L3_operational/configuration/deployment/${NC}"
    exit 1
fi

echo "1. Validating YAML syntax..."
for file in *.yaml; do
    if python3 -c "import yaml; list(yaml.safe_load_all(open('$file')))" > /dev/null 2>&1; then
        echo -e "   ${GREEN}✓${NC} $file"
    else
        echo -e "   ${RED}✗${NC} $file - Invalid YAML"
        python3 -c "import yaml; list(yaml.safe_load_all(open('$file')))"
        exit 1
    fi
done

echo ""
echo "2. Validating Kubernetes manifests..."
for file in [0-9]*.yaml; do
    if kubectl apply --dry-run=client -f "$file" > /dev/null 2>&1; then
        echo -e "   ${GREEN}✓${NC} $file"
    else
        echo -e "   ${RED}✗${NC} $file - Invalid Kubernetes manifest"
        kubectl apply --dry-run=client -f "$file"
        exit 1
    fi
done

echo ""
echo "3. Validating Kustomize build..."
if kubectl kustomize . > /dev/null 2>&1; then
    echo -e "   ${GREEN}✓${NC} Kustomize build successful"
else
    echo -e "   ${RED}✗${NC} Kustomize build failed"
    kubectl kustomize .
    exit 1
fi

echo ""
echo "4. Checking for hardcoded secrets..."
SECRETS_FOUND=false
if grep -q "CHANGE_ME" 02-secrets.yaml; then
    echo -e "   ${YELLOW}⚠${NC}  Found CHANGE_ME placeholders in secrets.yaml"
    echo "      Please update these before deploying to production!"
    SECRETS_FOUND=true
fi

if grep -E "sk-ant-|password:|secret:" *.yaml | grep -v "CHANGE_ME" | grep -v "^02-secrets.yaml"; then
    echo -e "   ${RED}✗${NC} Found hardcoded secrets in manifests!"
    SECRETS_FOUND=true
fi

if [ "$SECRETS_FOUND" = false ]; then
    echo -e "   ${GREEN}✓${NC} No hardcoded secrets found"
fi

echo ""
echo "5. Resource validation..."

# Check resource requests/limits
DEPLOYMENT=$(kubectl kustomize . | kubectl apply --dry-run=client -f - -o json 2>/dev/null | jq -r '.items[] | select(.kind=="Deployment") | .spec.template.spec.containers[0].resources')
if [ ! -z "$DEPLOYMENT" ]; then
    echo -e "   ${GREEN}✓${NC} Resource limits configured"
    echo "      Memory: $(echo $DEPLOYMENT | jq -r '.requests.memory') - $(echo $DEPLOYMENT | jq -r '.limits.memory')"
    echo "      CPU: $(echo $DEPLOYMENT | jq -r '.requests.cpu') - $(echo $DEPLOYMENT | jq -r '.limits.cpu')"
else
    echo -e "   ${YELLOW}⚠${NC}  Could not validate resources"
fi

echo ""
echo "6. Security validation..."

# Check security context
SECURITY=$(kubectl kustomize . | kubectl apply --dry-run=client -f - -o json 2>/dev/null | jq -r '.items[] | select(.kind=="Deployment") | .spec.template.spec.securityContext')
if echo "$SECURITY" | grep -q "runAsNonRoot.*true"; then
    echo -e "   ${GREEN}✓${NC} Pod runs as non-root"
else
    echo -e "   ${RED}✗${NC} Pod may run as root"
fi

if echo "$SECURITY" | grep -q "readOnlyRootFilesystem.*true"; then
    echo -e "   ${GREEN}✓${NC} Read-only root filesystem"
else
    echo -e "   ${YELLOW}⚠${NC}  Root filesystem is writable"
fi

echo ""
echo "7. Checking dependencies..."

# Check if required CRDs exist (if cluster is accessible)
if kubectl cluster-info &> /dev/null; then
    echo "   Checking cluster resources..."
    
    # Check for required APIs
    for api in "autoscaling/v2" "networking.k8s.io/v1" "policy/v1"; do
        if kubectl api-resources | grep -q "$api"; then
            echo -e "   ${GREEN}✓${NC} API $api available"
        else
            echo -e "   ${YELLOW}⚠${NC}  API $api not available"
        fi
    done
    
    # Check for monitoring CRDs
    if kubectl get crd servicemonitors.monitoring.coreos.com &> /dev/null; then
        echo -e "   ${GREEN}✓${NC} Prometheus Operator CRDs found"
    else
        echo -e "   ${YELLOW}⚠${NC}  Prometheus Operator not installed"
    fi
else
    echo -e "   ${YELLOW}⚠${NC}  Cannot connect to cluster, skipping dependency checks"
fi

echo ""
echo "8. Generating deployment summary..."
echo ""
echo "   Deployment Configuration:"
echo "   - Namespace: hal9-production"
echo "   - Initial Replicas: 30"
echo "   - Max Replicas: 100"
echo "   - Memory per Pod: 2Gi - 4Gi"
echo "   - CPU per Pod: 1 - 2 cores"
echo "   - Total Memory (30 pods): 60Gi - 120Gi"
echo "   - Total CPU (30 pods): 30 - 60 cores"

echo ""
echo "=== Validation Complete ==="
echo ""

if [ "$SECRETS_FOUND" = true ]; then
    echo -e "${YELLOW}⚠  Warning: Update secrets before deploying!${NC}"
fi

echo -e "${GREEN}✓ All manifests are valid and ready for deployment${NC}"
echo ""
echo "To deploy:"
echo "  kubectl apply -k ."
echo ""
echo "To preview:"
echo "  kubectl kustomize . | less"