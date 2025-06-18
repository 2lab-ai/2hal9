#!/bin/bash
# Deploy monitoring stack for HAL9

set -e

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo "📊 Deploying HAL9 Monitoring Stack"
echo "================================="
echo

# Check dependencies
echo "Checking dependencies..."
for cmd in docker docker-compose; do
    if ! command -v $cmd &> /dev/null; then
        echo -e "${RED}Error: $cmd is not installed${NC}"
        exit 1
    fi
done
echo -e "${GREEN}✓ Dependencies satisfied${NC}"
echo

# Create monitoring network
echo "Creating monitoring network..."
docker network create monitoring 2>/dev/null || echo "Network already exists"

# Create data directories
echo "Creating data directories..."
mkdir -p data/{prometheus,grafana,alertmanager}
chmod 777 data/grafana  # Grafana needs write permissions

# Deploy Prometheus
echo -e "${BLUE}Deploying Prometheus...${NC}"
docker run -d \
    --name prometheus \
    --network monitoring \
    -p 9091:9090 \
    -v $(pwd)/layers/L3_operational/configuration/prometheus/prometheus.yml:/etc/prometheus/prometheus.yml:ro \
    -v $(pwd)/layers/L3_operational/configuration/prometheus/alerts:/etc/prometheus/alerts:ro \
    -v $(pwd)/data/prometheus:/prometheus \
    --restart unless-stopped \
    prom/prometheus:latest \
    --config.file=/etc/prometheus/prometheus.yml \
    --storage.tsdb.path=/prometheus \
    --web.console.libraries=/usr/share/prometheus/console_libraries \
    --web.console.templates=/usr/share/prometheus/consoles \
    --web.enable-lifecycle

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✓ Prometheus deployed${NC}"
else
    echo -e "${RED}✗ Prometheus deployment failed${NC}"
fi

# Deploy Grafana
echo -e "${BLUE}Deploying Grafana...${NC}"
docker run -d \
    --name grafana \
    --network monitoring \
    -p 3000:3000 \
    -v $(pwd)/layers/L3_operational/configuration/grafana/provisioning:/etc/grafana/provisioning:ro \
    -v $(pwd)/data/grafana:/var/lib/grafana \
    -e GF_SECURITY_ADMIN_PASSWORD=admin \
    -e GF_USERS_ALLOW_SIGN_UP=false \
    --restart unless-stopped \
    grafana/grafana:latest

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✓ Grafana deployed${NC}"
else
    echo -e "${RED}✗ Grafana deployment failed${NC}"
fi

# Deploy Alertmanager (optional)
echo -e "${BLUE}Deploying Alertmanager...${NC}"
cat > /tmp/alertmanager.yml << EOF
global:
  resolve_timeout: 5m

route:
  group_by: ['alertname', 'cluster', 'service']
  group_wait: 10s
  group_interval: 10s
  repeat_interval: 1h
  receiver: 'default'

receivers:
  - name: 'default'
    webhook_configs:
      - url: 'http://hal9-server:8080/api/v1/alerts'
        send_resolved: true
EOF

docker run -d \
    --name alertmanager \
    --network monitoring \
    -p 9093:9093 \
    -v /tmp/alertmanager.yml:/etc/alertmanager/config.yml \
    -v $(pwd)/data/alertmanager:/alertmanager \
    --restart unless-stopped \
    prom/alertmanager:latest \
    --config.file=/etc/alertmanager/config.yml \
    --storage.path=/alertmanager

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✓ Alertmanager deployed${NC}"
else
    echo -e "${YELLOW}⚠ Alertmanager deployment failed (optional)${NC}"
fi

# Deploy Node Exporter for system metrics
echo -e "${BLUE}Deploying Node Exporter...${NC}"
docker run -d \
    --name node-exporter \
    --network monitoring \
    -p 9100:9100 \
    --pid="host" \
    --restart unless-stopped \
    prom/node-exporter:latest

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✓ Node Exporter deployed${NC}"
else
    echo -e "${YELLOW}⚠ Node Exporter deployment failed (optional)${NC}"
fi

# Wait for services to start
echo
echo "Waiting for services to start..."
sleep 10

# Check service health
echo
echo "Checking service health..."
echo

# Prometheus
if curl -s http://localhost:9091/-/healthy > /dev/null; then
    echo -e "${GREEN}✓ Prometheus is healthy${NC}"
else
    echo -e "${RED}✗ Prometheus health check failed${NC}"
fi

# Grafana
if curl -s http://localhost:3000/api/health > /dev/null; then
    echo -e "${GREEN}✓ Grafana is healthy${NC}"
else
    echo -e "${RED}✗ Grafana health check failed${NC}"
fi

# Summary
echo
echo -e "${YELLOW}Monitoring Stack Deployed!${NC}"
echo "========================="
echo
echo "Access URLs:"
echo "- Grafana: http://localhost:3000 (admin/admin)"
echo "- Prometheus: http://localhost:9091"
echo "- Alertmanager: http://localhost:9093"
echo
echo "Dashboards available in Grafana:"
echo "- HAL9 Overview"
echo "- HAL9 Consciousness Metrics"
echo
echo "To stop monitoring stack:"
echo "docker stop prometheus grafana alertmanager node-exporter"
echo
echo "To remove monitoring stack:"
echo "docker rm prometheus grafana alertmanager node-exporter"