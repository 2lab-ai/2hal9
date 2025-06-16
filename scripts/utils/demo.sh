#!/bin/bash

# Simple wrapper to run demos from project root

if [ "$1" == "quick" ]; then
    ./demo/quick-demo.sh
elif [ "$1" == "performance" ]; then
    ./demo/performance-benchmark.sh
elif [ "$1" == "verify" ]; then
    ./demo/verify-performance.sh
elif [ "$1" == "ai" ]; then
    ./demo/ai-neurons-demo.sh
else
    ./demo/run-all.sh
fi