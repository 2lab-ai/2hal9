#!/bin/bash
#
# monitor
# Auto-fixed by L1 migration script
#

set -euo pipefail

# Source common environment
source "$(dirname "$0")/../../common-env.sh"

# Original script content (modified for new paths)


# HAL9 Deployment Monitoring Script
# Real-time monitoring during deployment operations

set -e

# Configuration
DURATION=${1:-"30m"}
NAMESPACE="hal9"
REFRESH_INTERVAL=5
ALERT_THRESHOLD_ERROR_RATE=0.001  # 0.1%
ALERT_THRESHOLD_LATENCY_P99=50    # 50ms
ALERT_THRESHOLD_CPU=80            # 80%
ALERT_THRESHOLD_MEMORY=85         # 85%

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Calculate end time
if [[ "$DURATION" =~ ^([0-9]+)([mh])$ ]]; then
    VALUE="${BASH_REMATCH[1]}"
    UNIT="${BASH_REMATCH[2]}"
    
    case $UNIT in
        m) END_TIME=$(($(date +%s) + VALUE * 60)) ;;
        h) END_TIME=$(($(date +%s) + VALUE * 3600)) ;;
    esac
else
    echo "Invalid duration format. Use format like '30m' or '2h'"
    exit 1
fi

# Function to get metric value from Prometheus
get_metric() {
    local query=$1
    local value=$(curl -s "http://prometheus:9090/api/v1/query?query=$query" | \
        jq -r '.data.result[0].value[1] // "0"' 2>/dev/null || echo "0")
    echo $value
}

# Function to format number with color
format_value() {
    local value=$1
    local threshold=$2
    local format=$3
    
    if (( $(echo "$value > $threshold" | bc -l) )); then
        echo -e "${RED}${value}${format}${NC}"
    elif (( $(echo "$value > $threshold * 0.8" | bc -l) )); then
        echo -e "${YELLOW}${value}${format}${NC}"
    else
        echo -e "${GREEN}${value}${format}${NC}"
    fi
}

# Function to check for alerts
check_alerts() {
    local alerts=$(curl -s http://prometheus:9090/api/v1/alerts | \
        jq -r '.data.alerts[] | select(.state=="firing") | .labels.alertname' 2>/dev/null)
    
    if [ -n "$alerts" ]; then
        echo -e "${RED}⚠️  Active Alerts:${NC}"
        echo "$alerts" | while read alert; do
            echo "   - $alert"
        done
        echo
    fi
}

# Function to display monitoring dashboard
display_dashboard() {
    clear
    echo -e "${BLUE}HAL9 Deployment Monitor${NC}"
    echo -e "${BLUE}======================${NC}"
    echo "Time: $(date '+%Y-%m-%d %H:%M:%S')"
    echo "Remaining: $((($END_TIME - $(date +%s)) / 60)) minutes"
    echo
    
    # Error Rate
    local error_rate=$(get_metric 'rate(hal9_errors_total[5m])')
    error_rate=$(printf "%.4f" $error_rate)
    echo -e "Error Rate: $(format_value $error_rate $ALERT_THRESHOLD_ERROR_RATE '%')"
    
    # Latency
    local latency_p50=$(get_metric 'histogram_quantile(0.5, rate(hal9_request_duration_seconds_bucket[5m]))')
    local latency_p99=$(get_metric 'histogram_quantile(0.99, rate(hal9_request_duration_seconds_bucket[5m]))')
    latency_p50=$(printf "%.1f" $(echo "$latency_p50 * 1000" | bc))
    latency_p99=$(printf "%.1f" $(echo "$latency_p99 * 1000" | bc))
    echo -e "Latency P50: ${GREEN}${latency_p50}ms${NC} | P99: $(format_value $latency_p99 $ALERT_THRESHOLD_LATENCY_P99 'ms')"
    
    # Throughput
    local throughput=$(get_metric 'rate(hal9_requests_total[5m])')
    throughput=$(printf "%.0f" $throughput)
    echo -e "Throughput: ${BLUE}${throughput} req/s${NC}"
    
    echo
    echo -e "${BLUE}Resource Usage${NC}"
    echo "--------------"
    
    # CPU Usage
    local cpu_usage=$(kubectl top nodes | grep -v NAME | awk '{sum+=$3} END {print sum/NR}' | sed 's/%//')
    echo -e "CPU Usage: $(format_value $cpu_usage $ALERT_THRESHOLD_CPU '%')"
    
    # Memory Usage
    local memory_usage=$(kubectl top nodes | grep -v NAME | awk '{sum+=$5} END {print sum/NR}' | sed 's/%//')
    echo -e "Memory Usage: $(format_value $memory_usage $ALERT_THRESHOLD_MEMORY '%')"
    
    # Pod Status
    local total_pods=$(kubectl get pods -n $NAMESPACE -l app=hal9 --no-headers | wc -l)
    local ready_pods=$(kubectl get pods -n $NAMESPACE -l app=hal9 --field-selector=status.phase=Running --no-headers | wc -l)
    echo -e "Pods: ${ready_pods}/${total_pods} Running"
    
    echo
    echo -e "${BLUE}Layer Status${NC}"
    echo "------------"
    
    # Layer health summary
    LAYERS=("l1-reflexive" "l2-implementation" "l3-operational" "l4-tactical" "l5-strategic")
    for layer in "${LAYERS[@]}"; do
        local layer_pods=$(kubectl get pods -n $NAMESPACE -l layer=$layer --no-headers | wc -l)
        local layer_ready=$(kubectl get pods -n $NAMESPACE -l layer=$layer --field-selector=status.phase=Running --no-headers | wc -l)
        
        if [ $layer_ready -eq $layer_pods ] && [ $layer_pods -gt 0 ]; then
            echo -e "${GREEN}✓${NC} $layer: $layer_ready/$layer_pods"
        else
            echo -e "${RED}✗${NC} $layer: $layer_ready/$layer_pods"
        fi
    done
    
    echo
    echo -e "${BLUE}Recent Events${NC}"
    echo "-------------"
    kubectl get events -n $NAMESPACE --sort-by='.lastTimestamp' | tail -5 | grep -v "LAST SEEN" || echo "No recent events"
    
    # Check for active alerts
    echo
    check_alerts
}

# Function to log metrics to file
log_metrics() {
    local log_file="/tmp/hal9-deployment-monitor-$(date +%Y%m%d-%H%M%S).log"
    local timestamp=$(date -u +%Y-%m-%dT%H:%M:%SZ)
    
    local error_rate=$(get_metric 'rate(hal9_errors_total[5m])')
    local latency_p99=$(get_metric 'histogram_quantile(0.99, rate(hal9_request_duration_seconds_bucket[5m]))')
    local throughput=$(get_metric 'rate(hal9_requests_total[5m])')
    
    echo "$timestamp,$error_rate,$latency_p99,$throughput" >> $log_file
}

# Function to send alerts
send_alert() {
    local alert_type=$1
    local message=$2
    
    # Log alert
    echo "[$(date)] ALERT: $alert_type - $message" >> /tmp/hal9-deployment-alerts.log
    
    # Send to Slack (if configured)
    if [ -n "$SLACK_WEBHOOK_URL" ]; then
        curl -X POST $SLACK_WEBHOOK_URL \
            -H 'Content-Type: application/json' \
            -d "{\"text\":\"🚨 HAL9 Deployment Alert: $alert_type\\n$message\"}" \
            2>/dev/null
    fi
    
    # Page on-call (if configured)
    if [ -n "$PAGERDUTY_TOKEN" ]; then
        curl -X POST https://api.pagerduty.com/incidents \
            -H "Authorization: Token token=$PAGERDUTY_TOKEN" \
            -H "Content-Type: application/json" \
            -d "{
                \"incident\": {
                    \"type\": \"incident\",
                    \"title\": \"HAL9 Deployment Alert: $alert_type\",
                    \"service\": {
                        \"id\": \"$PAGERDUTY_SERVICE_ID\",
                        \"type\": \"service_reference\"
                    },
                    \"body\": {
                        \"type\": \"incident_body\",
                        \"details\": \"$message\"
                    }
                }
            }" 2>/dev/null
    fi
}

# Main monitoring loop
echo "Starting deployment monitoring for $DURATION..."
echo "Press Ctrl+C to stop"
echo

# Trap to handle graceful exit
trap 'echo -e "\n${YELLOW}Monitoring stopped by user${NC}"; exit 0' INT

while [ $(date +%s) -lt $END_TIME ]; do
    # Display dashboard
    display_dashboard
    
    # Log metrics
    log_metrics
    
    # Check thresholds and send alerts if needed
    error_rate=$(get_metric 'rate(hal9_errors_total[5m])')
    if (( $(echo "$error_rate > $ALERT_THRESHOLD_ERROR_RATE" | bc -l) )); then
        send_alert "High Error Rate" "Error rate is $error_rate (threshold: $ALERT_THRESHOLD_ERROR_RATE)"
    fi
    
    latency_p99=$(get_metric 'histogram_quantile(0.99, rate(hal9_request_duration_seconds_bucket[5m]))')
    latency_p99_ms=$(echo "$latency_p99 * 1000" | bc)
    if (( $(echo "$latency_p99_ms > $ALERT_THRESHOLD_LATENCY_P99" | bc -l) )); then
        send_alert "High Latency" "P99 latency is ${latency_p99_ms}ms (threshold: ${ALERT_THRESHOLD_LATENCY_P99}ms)"
    fi
    
    # Sleep before next refresh
    sleep $REFRESH_INTERVAL
done

echo
echo -e "${GREEN}Monitoring completed successfully${NC}"
echo "Metrics logged to: /tmp/hal9-deployment-monitor-*.log"
echo "Alerts logged to: /tmp/hal9-deployment-alerts.log"