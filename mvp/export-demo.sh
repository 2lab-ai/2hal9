#!/bin/bash
# HAL9 Export Demo Script

set -e

echo "🎬 HAL9 Export Demo"
echo "==================="
echo

# Check if a recording exists
RECORDING_DIR="mvp/recordings"
if [ ! -d "$RECORDING_DIR" ] || [ -z "$(ls -A $RECORDING_DIR 2>/dev/null)" ]; then
    echo "❌ No recordings found in $RECORDING_DIR"
    echo "📹 First create a recording with: ./mvp/record-demo.sh"
    exit 1
fi

# List available recordings
echo "📼 Available recordings:"
echo
select RECORDING in "$RECORDING_DIR"/*.json; do
    if [ -n "$RECORDING" ]; then
        break
    fi
done

echo
echo "Selected: $(basename $RECORDING)"
echo

# Show export options
echo "🎨 Export options:"
echo "  1) Animated SVG (view in browser)"
echo "  2) Frame sequence (for GIF conversion)"
echo "  3) GIF conversion script"
echo "  4) All of the above"
echo

read -p "Select option (1-4): " OPTION

case $OPTION in
    1)
        echo "🎨 Exporting to SVG..."
        cargo run -p hal9_mvp -- --export-svg="$RECORDING"
        ;;
    2)
        echo "🖼️ Exporting frames..."
        cargo run -p hal9_mvp -- --export-frames="$RECORDING"
        ;;
    3)
        echo "📝 Creating GIF script..."
        cargo run -p hal9_mvp -- --export-gif-script="$RECORDING"
        ;;
    4)
        echo "🎯 Exporting all formats..."
        echo
        echo "1️⃣ SVG..."
        cargo run -p hal9_mvp -- --export-svg="$RECORDING"
        echo
        echo "2️⃣ Frames..."
        cargo run -p hal9_mvp -- --export-frames="$RECORDING"
        echo
        echo "3️⃣ GIF script..."
        cargo run -p hal9_mvp -- --export-gif-script="$RECORDING"
        ;;
    *)
        echo "❌ Invalid option"
        exit 1
        ;;
esac

echo
echo "✅ Export complete!"
echo

# Show next steps
if [ -f "${RECORDING%.json}.svg" ]; then
    echo "💡 View SVG animation:"
    echo "   open ${RECORDING%.json}.svg"
fi

if [ -d "${RECORDING%.json}_frames" ]; then
    echo "💡 Convert frames to GIF:"
    echo "   convert -delay 3 -loop 0 ${RECORDING%.json}_frames/*.svg demo.gif"
fi

if [ -f "${RECORDING%.json}.gif.sh" ]; then
    echo "💡 Run GIF conversion script:"
    echo "   bash ${RECORDING%.json}.gif.sh"
fi