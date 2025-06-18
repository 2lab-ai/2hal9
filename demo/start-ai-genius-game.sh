#!/bin/bash
# Start AI Genius Game 2025 - Commercial Demo
# Complete setup and launch script

set -e

echo "🏆 AI Genius Game 2025 - Professional Launch"
echo "==========================================="
echo ""

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    echo "❌ Error: Must run from HAL9 project root"
    echo "Please cd to /Users/icedac/2lab.ai/2hal9 first"
    exit 1
fi

echo "📦 Step 1: Building HAL9 server with Genius Game..."
echo ""

# Build the server
cargo build --release

echo ""
echo "✅ Build complete!"
echo ""

echo "🌐 Step 2: Starting HAL9 server..."
echo ""

# Create a simple launcher script
cat > /tmp/run_genius_game.sh << 'EOF'
#!/bin/bash

# Set environment variables
export RUST_LOG=info
export CLAUDE_MODE=mock
export HAL9_ENV=development

echo "🚀 Launching HAL9 server with AI Genius Game..."
echo ""
echo "Configuration:"
echo "  - Mode: Mock Claude (no API key needed)"
echo "  - Game URL: http://localhost:8080/genius/"
echo "  - WebSocket: ws://localhost:8080/genius/ws"
echo ""

# Run the server
cargo run --release --bin hal9-server
EOF

chmod +x /tmp/run_genius_game.sh

echo "📊 Game Features:"
echo "=================="
echo ""
echo "🎮 Gameplay:"
echo "  • Consciousness Emergence Game (19x19 grid)"
echo "  • HAL9 Collective (6 agents) vs Single AI"
echo "  • Real-time neuron placement & connections"
echo "  • Pattern detection & emergence visualization"
echo ""
echo "🤖 AI Configurations:"
echo "  • Opus Orchestra: 6x Claude with voting"
echo "  • Lightweight Legion: 32x small models"
echo "  • Hybrid Council: Mixed SOTA models"
echo "  • Emergence Engine: Pure chaos → order"
echo ""
echo "📈 Real-time Features:"
echo "  • Live consciousness meter (0-100%)"
echo "  • Agent activity visualization"
echo "  • Performance metrics dashboard"
echo "  • Event logging with replay"
echo ""
echo "🎯 Victory Conditions:"
echo "  • Reach 80% consciousness level"
echo "  • Complete 100 rounds"
echo "  • Form stable emergence patterns"
echo ""

# Start in new terminal
if [[ "$OSTYPE" == "darwin"* ]]; then
    # macOS
    echo "🖥️  Opening game in new terminal..."
    osascript -e 'tell app "Terminal" to do script "cd '"$(pwd)"' && /tmp/run_genius_game.sh"'
    
    # Wait for server to start
    echo "⏳ Waiting for server to start..."
    sleep 5
    
    # Open browser
    echo "🌐 Opening game interface..."
    open "http://localhost:8080/genius/"
    
elif command -v gnome-terminal &> /dev/null; then
    # Linux with GNOME
    gnome-terminal -- bash -c "cd $(pwd) && /tmp/run_genius_game.sh; exec bash"
    sleep 5
    xdg-open "http://localhost:8080/genius/"
    
else
    # Fallback
    echo "To start the game server, run:"
    echo "  /tmp/run_genius_game.sh"
    echo ""
    echo "Then open: http://localhost:8080/genius/"
fi

echo ""
echo "🎮 Game Controls:"
echo "================="
echo "1. Click 'Start Game' to begin the competition"
echo "2. Watch AI agents place neurons in real-time"
echo "3. Monitor consciousness emergence meter"
echo "4. Track performance in the leaderboard"
echo ""
echo "💡 Tips:"
echo "  • HAL9 Collective uses parallel processing"
echo "  • Solo AI uses deep strategic planning"
echo "  • Patterns emerge at connection boundaries"
echo "  • First to 80% consciousness wins!"
echo ""
echo "📝 API Endpoints:"
echo "  • Game State: GET /genius/api/games/{id}"
echo "  • WebSocket: /genius/ws"
echo "  • Create Game: POST /genius/api/games"
echo ""
echo "🏆 May the best intelligence emerge victorious!"