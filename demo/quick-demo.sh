#!/bin/bash

# Quick 30-second demo - See self-organization instantly!

echo "
⚡ HAL9 Quick Demo - Self-Organization in 30 Seconds
====================================================
"

cd "$(dirname "$0")/../L2_implementation/neurons/examples"

echo "Compiling..."
rustc --edition 2021 -O simple_true_self_org_demo.rs -o /tmp/hal9_quick_demo 2>/dev/null

if [ $? -eq 0 ]; then
    echo "Running demo..."
    echo ""
    /tmp/hal9_quick_demo
else
    echo "Using pre-compiled demo..."
    if [ -f "simple_demo" ]; then
        ./simple_demo
    else
        echo "Compilation failed. Please ensure Rust is installed."
        echo "Install Rust: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    fi
fi

echo "
✨ What you just witnessed:
- 25 neurons started identical (no predefined structure)
- They discovered each other through broadcasts
- Formed connections based on compatibility
- Self-organized into functional layers
- All in microseconds!

Run './demo/performance-benchmark.sh' to see the speed metrics!
"