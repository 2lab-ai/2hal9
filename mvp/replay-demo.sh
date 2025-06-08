#!/bin/bash

# HAL9 Demo Replay Script

echo "🎬 HAL9 Demo Replay Mode"
echo "========================="
echo

# Check if recordings exist
if [ ! -d "mvp/recordings" ] || [ -z "$(ls -A mvp/recordings 2>/dev/null)" ]; then
    echo "❌ No recordings found in mvp/recordings/"
    echo
    echo "To create a recording, run: ./mvp/record-demo.sh"
    exit 1
fi

echo "Available recordings:"
echo
i=1
recordings=()
for file in mvp/recordings/*.json; do
    if [ -f "$file" ]; then
        echo "  [$i] $(basename "$file")"
        recordings+=("$file")
        ((i++))
    fi
done

echo
echo "Select a recording to replay (or 'q' to quit):"
read -p "> " selection

if [ "$selection" = "q" ]; then
    echo "Exiting..."
    exit 0
fi

# Validate selection
if ! [[ "$selection" =~ ^[0-9]+$ ]] || [ "$selection" -lt 1 ] || [ "$selection" -gt "${#recordings[@]}" ]; then
    echo "❌ Invalid selection"
    exit 1
fi

# Get selected file
selected_file="${recordings[$((selection-1))]}"

echo
echo "Playing: $(basename "$selected_file")"
echo

# Run replay
cargo run --release -p hal9_mvp -- --replay="$selected_file"