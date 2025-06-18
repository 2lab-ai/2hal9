#!/bin/bash
# Test HAL9 monitoring functionality

echo "📊 Testing HAL9 Monitoring System"
echo "================================"
echo

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Base URLs
HAL9_URL=${1:-http://localhost:8080}
PROMETHEUS_URL=${2:-http://localhost:9091}
GRAFANA_URL=${3:-http://localhost:3000}

# Check if services are running
echo "Checking services..."
echo

# HAL9 Server
if curl -s $HAL9_URL/health > /dev/null 2>&1; then
    echo -e "${GREEN}✓ HAL9 server is running${NC}"
else
    echo -e "${RED}✗ HAL9 server is not running${NC}"
    echo "Please start the server first"
    exit 1
fi

# Prometheus
if curl -s $PROMETHEUS_URL/-/healthy > /dev/null 2>&1; then
    echo -e "${GREEN}✓ Prometheus is running${NC}"
else
    echo -e "${YELLOW}⚠ Prometheus is not running${NC}"
    echo "Run: ./layers/L3_operational/scripts/deploy-monitoring.sh"
fi

# Grafana
if curl -s $GRAFANA_URL/api/health > /dev/null 2>&1; then
    echo -e "${GREEN}✓ Grafana is running${NC}"
else
    echo -e "${YELLOW}⚠ Grafana is not running${NC}"
fi
echo

# Test 1: Check Prometheus metrics endpoint
echo -e "${BLUE}1. Testing Prometheus Metrics Endpoint${NC}"
echo "   URL: $HAL9_URL/metrics"
METRICS=$(curl -s $HAL9_URL/metrics | head -20)
if [ ! -z "$METRICS" ]; then
    echo -e "${GREEN}✓ Metrics endpoint is working${NC}"
    echo "   Sample metrics:"
    echo "$METRICS" | grep "^hal9_" | head -5 | sed 's/^/   /'
else
    echo -e "${RED}✗ Metrics endpoint not responding${NC}"
fi
echo

# Test 2: Generate some load for metrics
echo -e "${BLUE}2. Generating Load for Metrics${NC}"
echo "   Making 50 API calls..."
for i in {1..50}; do
    curl -s $HAL9_URL/api/v1/status > /dev/null 2>&1 &
    if [ $((i % 10)) -eq 0 ]; then
        echo -ne "\r   Progress: $i/50"
    fi
done
wait
echo -e "\r   ${GREEN}✓ Load generation complete${NC}"
echo

# Test 3: Query Prometheus
if curl -s $PROMETHEUS_URL/-/healthy > /dev/null 2>&1; then
    echo -e "${BLUE}3. Querying Prometheus${NC}"
    
    # Query signal rate
    QUERY="rate(hal9_signals_processed_total[1m])"
    RESULT=$(curl -s "$PROMETHEUS_URL/api/v1/query?query=$QUERY" | jq -r '.data.result[0].value[1]' 2>/dev/null)
    if [ ! -z "$RESULT" ] && [ "$RESULT" != "null" ]; then
        echo -e "   Signal processing rate: ${GREEN}$RESULT/sec${NC}"
    else
        echo "   Signal processing rate: No data yet"
    fi
    
    # Query uptime
    QUERY="hal9_server_uptime_seconds"
    RESULT=$(curl -s "$PROMETHEUS_URL/api/v1/query?query=$QUERY" | jq -r '.data.result[0].value[1]' 2>/dev/null)
    if [ ! -z "$RESULT" ] && [ "$RESULT" != "null" ]; then
        UPTIME=$(echo "$RESULT / 60" | bc 2>/dev/null || echo "0")
        echo -e "   Server uptime: ${GREEN}${UPTIME} minutes${NC}"
    fi
    
    # Query active neurons
    QUERY="hal9_neurons_active"
    RESULT=$(curl -s "$PROMETHEUS_URL/api/v1/query?query=$QUERY" | jq -r '.data.result[0].value[1]' 2>/dev/null)
    if [ ! -z "$RESULT" ] && [ "$RESULT" != "null" ]; then
        echo -e "   Active neurons: ${GREEN}$RESULT${NC}"
    fi
    echo
fi

# Test 4: Check Grafana dashboards
if curl -s $GRAFANA_URL/api/health > /dev/null 2>&1; then
    echo -e "${BLUE}4. Checking Grafana Dashboards${NC}"
    
    # List dashboards
    DASHBOARDS=$(curl -s -u admin:admin $GRAFANA_URL/api/search?type=dash-db | jq -r '.[].title' 2>/dev/null)
    if [ ! -z "$DASHBOARDS" ]; then
        echo "   Available dashboards:"
        echo "$DASHBOARDS" | sed 's/^/   - /'
    else
        echo "   No dashboards found (may need to wait for provisioning)"
    fi
    echo
fi

# Test 5: Verify specific metrics
echo -e "${BLUE}5. Verifying Specific Metrics${NC}"
METRICS_TO_CHECK=(
    "hal9_server_uptime_seconds"
    "hal9_signals_sent_total"
    "hal9_signals_processed_total"
    "hal9_neurons_active"
    "hal9_claude_tokens_used_total"
)

FOUND=0
TOTAL=${#METRICS_TO_CHECK[@]}

for metric in "${METRICS_TO_CHECK[@]}"; do
    if curl -s $HAL9_URL/metrics | grep -q "^$metric"; then
        ((FOUND++))
        echo -e "   ${GREEN}✓${NC} $metric"
    else
        echo -e "   ${RED}✗${NC} $metric"
    fi
done

echo "   Found $FOUND/$TOTAL metrics"
echo

# Summary
echo -e "${YELLOW}Monitoring Test Summary${NC}"
echo "======================"
echo
if [ $FOUND -eq $TOTAL ]; then
    echo -e "${GREEN}✓ All metrics are being exported${NC}"
else
    echo -e "${YELLOW}⚠ Some metrics are missing${NC}"
fi

if curl -s $PROMETHEUS_URL/-/healthy > /dev/null 2>&1; then
    echo -e "${GREEN}✓ Prometheus is collecting metrics${NC}"
else
    echo -e "${RED}✗ Prometheus is not running${NC}"
fi

if curl -s $GRAFANA_URL/api/health > /dev/null 2>&1; then
    echo -e "${GREEN}✓ Grafana dashboards are available${NC}"
    echo
    echo "Access Grafana at: $GRAFANA_URL"
    echo "Username: admin"
    echo "Password: admin"
else
    echo -e "${RED}✗ Grafana is not running${NC}"
fi
echo
echo "Monitoring test complete!"