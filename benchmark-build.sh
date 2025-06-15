#!/bin/bash

echo "🚀 Build Performance Benchmark"
echo "=============================="
echo "System: $(uname -m) with $(sysctl -n hw.ncpu) cores"
echo "Memory: $(sysctl -n hw.memsize | awk '{print $1/1024/1024/1024 " GB"}')"
echo ""

# Test 1: Check performance
echo "1. Running cargo check..."
start_time=$(date +%s)
timeout 30 cargo check --all-targets -j 16 2>&1 | tail -5
end_time=$(date +%s)
check_time=$((end_time - start_time))
echo "   Check time: ${check_time}s"
echo ""

# Test 2: Single package build
echo "2. Building single package (genius_game_server)..."
start_time=$(date +%s)
timeout 60 cargo build -p genius_game_server -j 16 2>&1 | tail -5
end_time=$(date +%s)
single_time=$((end_time - start_time))
echo "   Single package time: ${single_time}s"
echo ""

# Test 3: Incremental build (no changes)
echo "3. Incremental build test..."
start_time=$(date +%s)
timeout 30 cargo build -j 16 2>&1 | tail -5
end_time=$(date +%s)
incremental_time=$((end_time - start_time))
echo "   Incremental time: ${incremental_time}s"
echo ""

echo "📊 Summary"
echo "=========="
echo "- Check: ${check_time}s"
echo "- Single package: ${single_time}s"
echo "- Incremental: ${incremental_time}s"
echo ""
echo "💡 Recommendations:"
echo "- Current build uses 707 crates, consider splitting workspace"
echo "- Install sccache: cargo install sccache"
echo "- Use 'cargo check' during development"
echo "- Build specific packages with -p flag"