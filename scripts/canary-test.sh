#!/bin/bash
# Canary deployment test script

set -e

URL=${1:-http://localhost:8080}
DURATION=${2:-300}  # 5 minutes default
THRESHOLD=${3:-99}  # 99% success rate threshold

echo "Running canary tests against: $URL"
echo "Duration: ${DURATION}s"
echo "Success threshold: ${THRESHOLD}%"

START_TIME=$(date +%s)
END_TIME=$((START_TIME + DURATION))

# Counters
TOTAL=0
SUCCESS=0
FAILED=0

# Response time tracking
declare -a RESPONSE_TIMES

# Test function
run_test() {
    local start=$(date +%s%N)
    
    # Run a typical API call
    if curl -s -f -o /dev/null \
        -H "X-API-Key: ${API_KEY:-test-key}" \
        -w "%{http_code}" \
        "$URL/api/v1/neurons" > /dev/null 2>&1; then
        local end=$(date +%s%N)
        local duration=$((($end - $start) / 1000000))  # Convert to ms
        RESPONSE_TIMES+=($duration)
        return 0
    else
        return 1
    fi
}

# Main test loop
echo "Starting canary tests..."
while [ $(date +%s) -lt $END_TIME ]; do
    if run_test; then
        ((SUCCESS++))
    else
        ((FAILED++))
    fi
    ((TOTAL++))
    
    # Show progress every 10 requests
    if [ $((TOTAL % 10)) -eq 0 ]; then
        SUCCESS_RATE=$(awk "BEGIN {printf \"%.2f\", $SUCCESS * 100.0 / $TOTAL}")
        echo -ne "\rRequests: $TOTAL | Success: $SUCCESS | Failed: $FAILED | Success Rate: ${SUCCESS_RATE}%"
    fi
    
    # Small delay between requests
    sleep 0.1
done

echo ""  # New line after progress

# Calculate statistics
SUCCESS_RATE=$(awk "BEGIN {printf \"%.2f\", $SUCCESS * 100.0 / $TOTAL}")

# Calculate response time percentiles
if [ ${#RESPONSE_TIMES[@]} -gt 0 ]; then
    # Sort response times
    IFS=$'\n' SORTED=($(sort -n <<<"${RESPONSE_TIMES[*]}"))
    unset IFS
    
    # Calculate percentiles
    P50_INDEX=$((${#SORTED[@]} * 50 / 100))
    P95_INDEX=$((${#SORTED[@]} * 95 / 100))
    P99_INDEX=$((${#SORTED[@]} * 99 / 100))
    
    P50=${SORTED[$P50_INDEX]}
    P95=${SORTED[$P95_INDEX]}
    P99=${SORTED[$P99_INDEX]}
    
    # Calculate average
    SUM=0
    for t in "${RESPONSE_TIMES[@]}"; do
        SUM=$((SUM + t))
    done
    AVG=$((SUM / ${#RESPONSE_TIMES[@]}))
fi

# Display results
echo ""
echo "===== CANARY TEST RESULTS ====="
echo "Total Requests: $TOTAL"
echo "Successful: $SUCCESS"
echo "Failed: $FAILED"
echo "Success Rate: ${SUCCESS_RATE}%"
echo ""
echo "Response Times (ms):"
echo "  Average: ${AVG:-N/A}"
echo "  P50: ${P50:-N/A}"
echo "  P95: ${P95:-N/A}"
echo "  P99: ${P99:-N/A}"
echo ""

# Check if canary passed
if (( $(echo "$SUCCESS_RATE >= $THRESHOLD" | bc -l) )); then
    echo "✓ CANARY PASSED (${SUCCESS_RATE}% >= ${THRESHOLD}%)"
    
    # Additional checks
    if [ ! -z "$P95" ] && [ $P95 -gt 1000 ]; then
        echo "⚠ WARNING: P95 response time is high (${P95}ms > 1000ms)"
    fi
    
    exit 0
else
    echo "✗ CANARY FAILED (${SUCCESS_RATE}% < ${THRESHOLD}%)"
    exit 1
fi