#!/bin/bash
# HAL9 Emergency Rollback Script

set -euo pipefail

# Configuration
NAMESPACE="${NAMESPACE:-hal9-prod}"
DEPLOYMENT_NAME="${DEPLOYMENT_NAME:-hal9}"

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${RED}🚨 HAL9 Emergency Rollback${NC}"
echo -e "${RED}==========================${NC}"

# 1. Check current status
echo -e "\n${YELLOW}📊 Current deployment status:${NC}"
kubectl rollout history deployment/${DEPLOYMENT_NAME} -n $NAMESPACE

# 2. Get user confirmation
echo -e "\n${RED}⚠️  WARNING: This will rollback to the previous version!${NC}"
read -p "Are you sure you want to continue? (yes/no): " confirm
if [ "$confirm" != "yes" ]; then
    echo -e "${YELLOW}Rollback cancelled.${NC}"
    exit 0
fi

# 3. Capture current state
echo -e "\n${YELLOW}📸 Capturing current state...${NC}"
CURRENT_VERSION=$(kubectl get deployment/${DEPLOYMENT_NAME} -n $NAMESPACE -o jsonpath='{.metadata.annotations.deployment-version}')
echo "Current version: $CURRENT_VERSION"

# Save current logs
kubectl logs deployment/${DEPLOYMENT_NAME} -n $NAMESPACE --tail=1000 > "rollback-logs-$(date +%Y%m%d-%H%M%S).txt"

# 4. Perform rollback
echo -e "\n${YELLOW}⏮️ Rolling back to previous version...${NC}"
kubectl rollout undo deployment/${DEPLOYMENT_NAME} -n $NAMESPACE

# 5. Monitor rollback progress
echo -e "\n${YELLOW}⏳ Monitoring rollback progress...${NC}"
kubectl rollout status deployment/${DEPLOYMENT_NAME} -n $NAMESPACE --timeout=300s

# 6. Verify rollback
echo -e "\n${YELLOW}🏥 Verifying rollback...${NC}"
sleep 10  # Give pods time to stabilize

# Check pod status
PODS=$(kubectl get pods -n $NAMESPACE -l app=hal9 -o jsonpath='{.items[*].metadata.name}')
ALL_HEALTHY=true
for pod in $PODS; do
    echo -n "Checking $pod... "
    if kubectl exec -n $NAMESPACE $pod -- curl -s http://localhost:3456/health &>/dev/null; then
        echo -e "${GREEN}✓${NC}"
    else
        echo -e "${RED}✗${NC}"
        ALL_HEALTHY=false
    fi
done

# 7. Rollback decision
if [ "$ALL_HEALTHY" = true ]; then
    echo -e "\n${GREEN}✅ Rollback completed successfully!${NC}"
    NEW_VERSION=$(kubectl get deployment/${DEPLOYMENT_NAME} -n $NAMESPACE -o jsonpath='{.metadata.annotations.deployment-version}')
    echo -e "${BLUE}Rolled back from: ${CURRENT_VERSION}${NC}"
    echo -e "${BLUE}Current version: ${NEW_VERSION}${NC}"
else
    echo -e "\n${RED}❌ Rollback verification failed!${NC}"
    echo -e "${YELLOW}Attempting to roll forward to original version...${NC}"
    kubectl rollout undo deployment/${DEPLOYMENT_NAME} -n $NAMESPACE
    kubectl rollout status deployment/${DEPLOYMENT_NAME} -n $NAMESPACE --timeout=300s
    exit 1
fi

# 8. Create incident report
echo -e "\n${YELLOW}📝 Creating incident report...${NC}"
cat > "rollback-report-$(date +%Y%m%d-%H%M%S).md" << EOF
# HAL9 Rollback Incident Report

**Date**: $(date)
**Operator**: ${USER}
**Environment**: ${NAMESPACE}

## Rollback Details
- **From Version**: ${CURRENT_VERSION}
- **To Version**: ${NEW_VERSION}
- **Reason**: Emergency rollback initiated

## Timeline
- Rollback initiated: $(date)
- Rollback completed: $(date)

## Impact
- Service downtime: ~2 minutes
- Affected services: HAL9 API

## Next Steps
1. Investigate root cause of failure
2. Fix issues in version ${CURRENT_VERSION}
3. Plan re-deployment with fixes

## Logs
See attached file: rollback-logs-$(date +%Y%m%d-%H%M%S).txt
EOF

echo -e "${BLUE}Incident report created: rollback-report-$(date +%Y%m%d-%H%M%S).md${NC}"

# 9. Send notification
if [ -n "${WEBHOOK_URL:-}" ]; then
    curl -X POST $WEBHOOK_URL \
        -H 'Content-Type: application/json' \
        -d "{
            \"text\": \"🚨 HAL9 Emergency Rollback: ${CURRENT_VERSION} → ${NEW_VERSION}\"
        }"
fi