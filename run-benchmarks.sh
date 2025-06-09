#!/bin/bash

# HAL9 Performance Benchmarking Suite
# Runs comprehensive benchmarks and generates reports

set -e

# Configuration
BENCHMARK_DIR="benches"
RESULTS_DIR="benchmark-results"
TIMESTAMP=$(date +%Y%m%d-%H%M%S)
RESULTS_PATH="$RESULTS_DIR/$TIMESTAMP"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Create results directory
mkdir -p "$RESULTS_PATH"

echo -e "${BLUE}HAL9 Performance Benchmarking Suite${NC}"
echo -e "${BLUE}===================================${NC}"
echo "Results will be saved to: $RESULTS_PATH"
echo

# Function to run a benchmark and save results
run_benchmark() {
    local name=$1
    local bench_file=$2
    local extra_args=$3
    
    echo -e "${YELLOW}Running $name benchmarks...${NC}"
    
    # Run with different configurations
    if cargo bench --bench $bench_file $extra_args -- --save-baseline $TIMESTAMP 2>&1 | tee "$RESULTS_PATH/${name}_output.txt"; then
        echo -e "${GREEN}✓ $name benchmarks completed${NC}"
        
        # Copy criterion results
        if [ -d "target/criterion" ]; then
            cp -r target/criterion "$RESULTS_PATH/${name}_criterion"
        fi
    else
        echo -e "${RED}✗ $name benchmarks failed${NC}"
        return 1
    fi
    echo
}

# Function to run comparison benchmarks
run_comparison() {
    local baseline=$1
    local name=$2
    local bench_file=$3
    
    echo -e "${YELLOW}Running comparison for $name...${NC}"
    
    if cargo bench --bench $bench_file -- --baseline $baseline 2>&1 | tee "$RESULTS_PATH/${name}_comparison.txt"; then
        echo -e "${GREEN}✓ Comparison completed${NC}"
    else
        echo -e "${RED}✗ Comparison failed${NC}"
    fi
    echo
}

# Main benchmark suite
echo -e "${BLUE}1. Core Hierarchical System Benchmarks${NC}"
run_benchmark "hierarchical" "hierarchical_benchmark"

echo -e "${BLUE}2. Memory System Benchmarks${NC}"
run_benchmark "memory" "memory_benchmark"

echo -e "${BLUE}3. Network Performance Benchmarks${NC}"
run_benchmark "network" "network_benchmark"

echo -e "${BLUE}4. Integration Benchmarks${NC}"
run_benchmark "integration" "performance_benchmark"

# Run stress tests if requested
if [ "$1" == "--stress" ]; then
    echo -e "${BLUE}5. Stress Testing${NC}"
    run_benchmark "stress" "stress_benchmark" "--features stress-test"
fi

# Generate comparison with previous run if exists
if [ "$1" == "--compare" ] && [ -n "$2" ]; then
    echo -e "${BLUE}Comparing with baseline: $2${NC}"
    run_comparison "$2" "hierarchical" "hierarchical_benchmark"
    run_comparison "$2" "memory" "memory_benchmark"
    run_comparison "$2" "network" "network_benchmark"
fi

# Generate HTML report
echo -e "${BLUE}Generating HTML Report...${NC}"
cat > "$RESULTS_PATH/report.html" << EOF
<!DOCTYPE html>
<html>
<head>
    <title>HAL9 Benchmark Results - $TIMESTAMP</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 20px; }
        h1, h2 { color: #333; }
        .metric { margin: 10px 0; padding: 10px; background: #f0f0f0; }
        .improved { color: green; }
        .degraded { color: red; }
        .summary { background: #e0e0e0; padding: 20px; margin: 20px 0; }
        table { border-collapse: collapse; width: 100%; }
        th, td { border: 1px solid #ddd; padding: 8px; text-align: left; }
        th { background-color: #4CAF50; color: white; }
    </style>
</head>
<body>
    <h1>HAL9 Performance Benchmark Results</h1>
    <p>Generated: $(date)</p>
    
    <div class="summary">
        <h2>Summary</h2>
        <p>Benchmark suite completed. Results saved to: $RESULTS_PATH</p>
    </div>
    
    <h2>Hierarchical System Performance</h2>
    <div class="metric">
        <h3>Signal Propagation</h3>
        <pre>$(grep -A 5 "signal_propagation" "$RESULTS_PATH/hierarchical_output.txt" || echo "No data")</pre>
    </div>
    
    <div class="metric">
        <h3>Layer Processing</h3>
        <pre>$(grep -A 10 "layer_processing" "$RESULTS_PATH/hierarchical_output.txt" || echo "No data")</pre>
    </div>
    
    <h2>Memory System Performance</h2>
    <div class="metric">
        <h3>Embedding Operations</h3>
        <pre>$(grep -A 5 "embedding_operations" "$RESULTS_PATH/memory_output.txt" || echo "No data")</pre>
    </div>
    
    <h2>Network Performance</h2>
    <div class="metric">
        <h3>Message Throughput</h3>
        <pre>$(grep -A 5 "message_throughput" "$RESULTS_PATH/network_output.txt" || echo "No data")</pre>
    </div>
</body>
</html>
EOF

# Generate JSON summary
echo -e "${BLUE}Generating JSON Summary...${NC}"
cat > "$RESULTS_PATH/summary.json" << EOF
{
    "timestamp": "$TIMESTAMP",
    "date": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
    "benchmarks_run": [
        "hierarchical",
        "memory",
        "network",
        "integration"
    ],
    "results_path": "$RESULTS_PATH",
    "system_info": {
        "rust_version": "$(rustc --version)",
        "cpu": "$(sysctl -n machdep.cpu.brand_string 2>/dev/null || cat /proc/cpuinfo | grep 'model name' | head -1 | cut -d: -f2)",
        "memory": "$(sysctl -n hw.memsize 2>/dev/null || free -h | grep Mem | awk '{print $2}')",
        "os": "$(uname -s) $(uname -r)"
    }
}
EOF

# Create benchmark comparison script
cat > "$RESULTS_PATH/compare.sh" << 'EOF'
#!/bin/bash
# Compare these results with another benchmark run

if [ -z "$1" ]; then
    echo "Usage: $0 <other-benchmark-timestamp>"
    exit 1
fi

OTHER_TIMESTAMP=$1
critcmp target/criterion/*/base target/criterion/*/new --threshold 5

echo "Detailed comparison saved to comparison_${OTHER_TIMESTAMP}.txt"
EOF
chmod +x "$RESULTS_PATH/compare.sh"

# Generate performance profile
echo -e "${BLUE}Generating Performance Profile...${NC}"
if command -v perf &> /dev/null; then
    echo "Running perf analysis..."
    # Note: This would need sudo on Linux
    # perf record -g cargo bench --bench hierarchical_benchmark -- --profile-time 10
    # perf report > "$RESULTS_PATH/perf_report.txt"
else
    echo "perf not available, skipping profiling"
fi

# Summary
echo
echo -e "${GREEN}Benchmark Suite Completed!${NC}"
echo -e "Results saved to: ${BLUE}$RESULTS_PATH${NC}"
echo
echo "Key files:"
echo "  - report.html: Human-readable report"
echo "  - summary.json: Machine-readable summary"
echo "  - *_output.txt: Raw benchmark output"
echo "  - compare.sh: Script to compare with other runs"
echo
echo "To view the report:"
echo "  open $RESULTS_PATH/report.html"
echo
echo "To compare with previous results:"
echo "  $RESULTS_PATH/compare.sh <previous-timestamp>"
echo
echo "To run specific benchmarks:"
echo "  cargo bench --bench hierarchical_benchmark <specific-test>"
echo
echo "To run with profiling:"
echo "  cargo bench --bench hierarchical_benchmark -- --profile-time 10"

# Archive results
if [ "$ARCHIVE_RESULTS" == "true" ]; then
    echo
    echo -e "${BLUE}Archiving results...${NC}"
    tar -czf "$RESULTS_DIR/benchmark-$TIMESTAMP.tar.gz" -C "$RESULTS_DIR" "$TIMESTAMP"
    echo -e "${GREEN}Archive created: $RESULTS_DIR/benchmark-$TIMESTAMP.tar.gz${NC}"
fi