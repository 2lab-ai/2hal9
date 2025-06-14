#!/bin/bash

# AI Neurons Demo - Watch AI components self-organize!

echo "
🤖 HAL9 AI Neurons Self-Organization Demo
==========================================

Watch as AI components (Visual, Audio, Logic, etc.) 
discover each other and form functional layers!
"

cd "$(dirname "$0")/../layers/L2_implementation/neurons/examples"

echo "Compiling AI neurons demo..."
rustc --edition 2021 working_ai_demo.rs -o /tmp/hal9_ai_demo 2>/dev/null

if [ $? -eq 0 ]; then
    /tmp/hal9_ai_demo
else
    # Try pre-compiled version
    if [ -f "working_ai_demo" ]; then
        ./working_ai_demo
    else
        echo "Compilation failed. Please check Rust installation."
    fi
fi

echo "
🎯 What happened:
- Visual, Audio, Logic neurons started separate
- They discovered each other through capabilities
- Formed layers based on function (not pre-design!)
- Created a working AI architecture spontaneously
"