#!/bin/bash

# Performance benchmark - See HAL9's lightning speed!

echo "
⚡ HAL9 Performance Benchmark
==============================

Let's see how fast consciousness can emerge...
"

cd "$(dirname "$0")/../layers/L2_implementation/neurons/examples"

# Try quick benchmark first
if [ -f "quick_benchmark.rs" ]; then
    echo "Running quick performance test..."
    rustc --edition 2021 -O quick_benchmark.rs -o /tmp/hal9_quick_bench 2>/dev/null
    if [ $? -eq 0 ]; then
        /tmp/hal9_quick_bench
    fi
fi

echo "
Want detailed analysis? Run:
./demo/verify-performance.sh
"