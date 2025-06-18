#!/bin/bash
# HAL9 Staging Deployment Script

set -euo pipefail

# Configuration
COMPOSE_FILE="docker-compose.staging.yml"
REGISTRY="ghcr.io/2lab-ai/2hal9"
IMAGE_TAG="${1:-latest}"

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${BLUE}🎭 HAL9 Staging Deployment${NC}"
echo -e "${BLUE}===========================${NC}"

# 1. Pull latest images
echo -e "\n${YELLOW}📥 Pulling images...${NC}"
docker pull "${REGISTRY}:${IMAGE_TAG}"
docker pull "${REGISTRY}:simple"

# 2. Create staging compose file
echo -e "\n${YELLOW}📝 Creating staging configuration...${NC}"
cat > $COMPOSE_FILE << EOF
version: '3.8'

services:
  hal9-server:
    image: ${REGISTRY}:${IMAGE_TAG}
    container_name: hal9-staging
    ports:
      - "3456:3456"
    environment:
      - ENVIRONMENT=staging
      - DATABASE_URL=\${DATABASE_URL}
      - REDIS_URL=redis://redis:6379
      - LOG_LEVEL=debug
    depends_on:
      - postgres
      - redis
    networks:
      - hal9-staging
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:3456/health"]
      interval: 30s
      timeout: 10s
      retries: 3

  postgres:
    image: postgres:15-alpine
    container_name: hal9-postgres-staging
    environment:
      - POSTGRES_DB=hal9_staging
      - POSTGRES_USER=hal9
      - POSTGRES_PASSWORD=\${DB_PASSWORD}
    volumes:
      - postgres-staging:/var/lib/postgresql/data
    networks:
      - hal9-staging
    restart: unless-stopped

  redis:
    image: redis:7-alpine
    container_name: hal9-redis-staging
    command: redis-server --appendonly yes
    volumes:
      - redis-staging:/data
    networks:
      - hal9-staging
    restart: unless-stopped

  consciousness-viz:
    image: ${REGISTRY}:simple
    container_name: hal9-consciousness-staging
    command: python /app/demo/consciousness-visualization/server.py
    ports:
      - "8765:8765"
    networks:
      - hal9-staging
    restart: unless-stopped

  self-org-dashboard:
    image: ${REGISTRY}:simple
    container_name: hal9-selforg-staging
    command: python /app/demo/self-organization-dashboard/server.py
    ports:
      - "8766:8766"
    networks:
      - hal9-staging
    restart: unless-stopped

volumes:
  postgres-staging:
  redis-staging:

networks:
  hal9-staging:
    driver: bridge
EOF

# 3. Stop existing staging
echo -e "\n${YELLOW}🛑 Stopping existing staging environment...${NC}"
docker compose -f $COMPOSE_FILE down || true

# 4. Start staging environment
echo -e "\n${YELLOW}🚀 Starting staging environment...${NC}"
docker compose -f $COMPOSE_FILE up -d

# 5. Wait for services to be ready
echo -e "\n${YELLOW}⏳ Waiting for services to start...${NC}"
sleep 10

# 6. Run database migrations
echo -e "\n${YELLOW}🗄️ Running database migrations...${NC}"
docker compose -f $COMPOSE_FILE exec -T hal9-server \
    /app/scripts/migrate.sh || echo "No migration script found"

# 7. Health checks
echo -e "\n${YELLOW}🏥 Running health checks...${NC}"
services=("hal9-server:3456" "consciousness-viz:8765" "self-org-dashboard:8766")
for service in "${services[@]}"; do
    name="${service%%:*}"
    port="${service##*:}"
    echo -n "Checking $name... "
    if curl -s -f "http://localhost:$port" &>/dev/null; then
        echo -e "${GREEN}✓${NC}"
    else
        echo -e "${RED}✗${NC}"
        echo -e "${RED}Service $name is not responding${NC}"
        docker compose -f $COMPOSE_FILE logs $name
        exit 1
    fi
done

# 8. Run integration tests
echo -e "\n${YELLOW}🧪 Running integration tests...${NC}"
docker run --rm \
    --network hal9-staging \
    -e API_URL=http://hal9-server:3456 \
    ${REGISTRY}:${IMAGE_TAG} \
    npm test || echo "No tests found"

# 9. Performance test
echo -e "\n${YELLOW}⚡ Running performance test...${NC}"
docker run --rm \
    --network hal9-staging \
    -v $(pwd)/reports:/reports \
    loadimpact/k6 run -e API_URL=http://hal9-server:3456 - <<'EOF'
import http from 'k6/http';
import { check } from 'k6';

export let options = {
  stages: [
    { duration: '30s', target: 10 },
    { duration: '1m', target: 50 },
    { duration: '30s', target: 0 },
  ],
};

export default function() {
  let res = http.get(`\${__ENV.API_URL}/api/games`);
  check(res, {
    'status is 200': (r) => r.status === 200,
    'response time < 500ms': (r) => r.timings.duration < 500,
  });
}
EOF

# 10. Generate staging report
echo -e "\n${YELLOW}📋 Generating staging report...${NC}"
cat > staging-report.md << EOF
# HAL9 Staging Deployment Report

**Date**: $(date)
**Version**: ${IMAGE_TAG}

## Services Status
$(docker compose -f $COMPOSE_FILE ps)

## Resource Usage
$(docker stats --no-stream --format "table {{.Container}}\t{{.CPUPerc}}\t{{.MemUsage}}")

## Recent Logs
### HAL9 Server
\`\`\`
$(docker compose -f $COMPOSE_FILE logs --tail=20 hal9-server)
\`\`\`

## Access URLs
- API: http://localhost:3456
- Consciousness Viz: http://localhost:8765
- Self-Org Dashboard: http://localhost:8766
EOF

# 11. Success
echo -e "\n${GREEN}✅ Staging deployment completed successfully!${NC}"
echo -e "${BLUE}Version: ${IMAGE_TAG}${NC}"
echo -e "${BLUE}API URL: http://localhost:3456${NC}"
echo -e "${BLUE}Report: ./staging-report.md${NC}"