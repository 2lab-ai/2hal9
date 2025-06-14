#!/bin/bash

# HAL9 MVP Demo Script

clear

echo "🚀 Starting HAL9 MVP Demo..."
echo
sleep 1

# Run the MVP with a pre-selected scenario
(echo "1"; sleep 8; echo "q") | ./target/release/hal9_mvp

echo
echo "✨ Demo complete! The hierarchical processing flow is shown above."
echo
echo "Key takeaways:"
echo "  1. User request flows through 3 layers (L4→L3→L2)"
echo "  2. Each layer handles different abstraction levels"
echo "  3. Visual hierarchy shows parent-child signal relationships"
echo "  4. Timestamps track processing flow"
echo