#!/bin/bash
# HAL9 Production Deployment Script

set -euo pipefail

# Configuration
NAMESPACE="hal9-prod"
DEPLOYMENT_NAME="hal9"
IMAGE_TAG="${1:-latest}"
REGISTRY="ghcr.io/2lab-ai/2hal9"

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${BLUE}🚀 HAL9 Production Deployment${NC}"
echo -e "${BLUE}================================${NC}"

# 1. Pre-deployment checks
echo -e "\n${YELLOW}📋 Pre-deployment checks...${NC}"

# Check kubectl connectivity
if ! kubectl cluster-info &>/dev/null; then
    echo -e "${RED}❌ Cannot connect to Kubernetes cluster${NC}"
    exit 1
fi

# Check namespace exists
if ! kubectl get namespace $NAMESPACE &>/dev/null; then
    echo -e "${YELLOW}Creating namespace $NAMESPACE...${NC}"
    kubectl create namespace $NAMESPACE
fi

# 2. Database migration
echo -e "\n${YELLOW}🗄️ Running database migrations...${NC}"
kubectl run migration-job \
    --image="${REGISTRY}:${IMAGE_TAG}" \
    --namespace=$NAMESPACE \
    --rm -i --restart=Never -- \
    /app/scripts/migrate.sh

# 3. Deploy application
echo -e "\n${YELLOW}📦 Deploying HAL9 v${IMAGE_TAG}...${NC}"

# Update deployment
kubectl set image deployment/${DEPLOYMENT_NAME} \
    hal9="${REGISTRY}:${IMAGE_TAG}" \
    --namespace=$NAMESPACE \
    --record

# Wait for rollout
echo -e "${YELLOW}⏳ Waiting for rollout to complete...${NC}"
kubectl rollout status deployment/${DEPLOYMENT_NAME} \
    --namespace=$NAMESPACE \
    --timeout=300s

# 4. Health check
echo -e "\n${YELLOW}🏥 Running health checks...${NC}"
PODS=$(kubectl get pods -n $NAMESPACE -l app=hal9 -o jsonpath='{.items[*].metadata.name}')
for pod in $PODS; do
    echo -n "Checking $pod... "
    if kubectl exec -n $NAMESPACE $pod -- curl -s http://localhost:3456/health &>/dev/null; then
        echo -e "${GREEN}✓${NC}"
    else
        echo -e "${RED}✗${NC}"
        echo -e "${RED}Health check failed for $pod${NC}"
        exit 1
    fi
done

# 5. Smoke tests
echo -e "\n${YELLOW}🧪 Running smoke tests...${NC}"
SERVICE_URL=$(kubectl get service hal9-service -n $NAMESPACE -o jsonpath='{.status.loadBalancer.ingress[0].hostname}')
if [ -z "$SERVICE_URL" ]; then
    SERVICE_URL=$(kubectl get service hal9-service -n $NAMESPACE -o jsonpath='{.status.loadBalancer.ingress[0].ip}')
fi

# Test endpoints
endpoints=("/api/games" "/api/metrics" "/health")
for endpoint in "${endpoints[@]}"; do
    echo -n "Testing $endpoint... "
    if curl -s -f "http://${SERVICE_URL}${endpoint}" &>/dev/null; then
        echo -e "${GREEN}✓${NC}"
    else
        echo -e "${RED}✗${NC}"
        echo -e "${RED}Endpoint $endpoint is not responding${NC}"
        exit 1
    fi
done

# 6. Performance check
echo -e "\n${YELLOW}⚡ Checking performance metrics...${NC}"
RESPONSE_TIME=$(curl -o /dev/null -s -w '%{time_total}' "http://${SERVICE_URL}/api/games")
if (( $(echo "$RESPONSE_TIME < 0.5" | bc -l) )); then
    echo -e "${GREEN}✓ Response time: ${RESPONSE_TIME}s${NC}"
else
    echo -e "${YELLOW}⚠️  Response time: ${RESPONSE_TIME}s (slow)${NC}"
fi

# 7. Update monitoring
echo -e "\n${YELLOW}📊 Updating monitoring dashboards...${NC}"
kubectl annotate deployment/${DEPLOYMENT_NAME} \
    --namespace=$NAMESPACE \
    --overwrite \
    deployment-version="${IMAGE_TAG}" \
    deployment-time="$(date -u +%Y-%m-%dT%H:%M:%SZ)"

# 8. Notification
echo -e "\n${GREEN}✅ Deployment completed successfully!${NC}"
echo -e "${BLUE}Version: ${IMAGE_TAG}${NC}"
echo -e "${BLUE}URL: http://${SERVICE_URL}${NC}"
echo -e "${BLUE}Namespace: ${NAMESPACE}${NC}"

# Send notification (Slack/Discord webhook)
if [ -n "${WEBHOOK_URL:-}" ]; then
    curl -X POST $WEBHOOK_URL \
        -H 'Content-Type: application/json' \
        -d "{
            \"text\": \"HAL9 v${IMAGE_TAG} deployed to production successfully!\"
        }"
fi