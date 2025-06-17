#!/bin/bash
# Start monitoring stack

set -e

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo "🔍 Starting HAL9 Monitoring Stack"
echo "================================"

# Start monitoring services
echo -e "${YELLOW}Starting monitoring services...${NC}"
docker-compose -f docker-compose.monitoring.yml up -d

# Wait for services to be ready
echo -e "${YELLOW}Waiting for services to start...${NC}"
sleep 10

# Check service status
echo -e "\n${YELLOW}Checking service status...${NC}"
docker-compose -f docker-compose.monitoring.yml ps

# Display access URLs
echo -e "\n${GREEN}Monitoring Stack Ready!${NC}"
echo "======================="
echo "📊 Grafana: http://localhost:3001"
echo "   Username: admin"
echo "   Password: hal9admin"
echo ""
echo "📈 Prometheus: http://localhost:9091"
echo "📦 cAdvisor: http://localhost:8081"
echo "🖥️  Node Exporter: http://localhost:9100/metrics"

echo -e "\n${YELLOW}Tips:${NC}"
echo "- Import dashboards from monitoring/grafana/dashboards/"
echo "- Configure alerts in Prometheus"
echo "- Check HAL9 metrics at http://localhost:8080/metrics"

# Open Grafana in browser (macOS)
if [[ "$OSTYPE" == "darwin"* ]]; then
    echo -e "\n${YELLOW}Opening Grafana in browser...${NC}"
    sleep 2
    open http://localhost:3001
fi