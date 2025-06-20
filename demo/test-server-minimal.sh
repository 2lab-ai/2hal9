#!/bin/bash
# Test minimal HAL9 server functionality

set -e

echo "🔧 Testing HAL9 Server (Minimal Mode)..."
echo "======================================="
echo

# Setup environment
export DATABASE_URL="sqlite://./hal9.db"
export CLAUDE_MODE="mock"
export RUST_LOG="hal9=debug,tower_http=debug"

# Ensure database is set up
if [ ! -f "hal9.db" ]; then
    echo "📋 Setting up database..."
    ./layers/L3_operational/scripts/setup-database.sh
fi

echo
echo "🏗️ Building server (this may take a moment)..."
# Allow warnings but not errors
RUSTFLAGS="-A warnings" cargo build --bin hal9-server --release 2>&1 | grep -E "error\[E[0-9]+\]:" | head -20 || true

# Check if binary was created
if [ -f "target/release/hal9-server" ]; then
    echo "✅ Server binary built successfully!"
    echo
    echo "🚀 Starting HAL9 server..."
    
    # Start server in background
    target/release/hal9-server &
    SERVER_PID=$!
    
    # Give server time to start
    sleep 3
    
    echo
    echo "🔍 Testing server endpoints..."
    
    # Test health endpoint
    echo -n "  → Health check: "
    if curl -s http://localhost:3000/health | grep -q "healthy"; then
        echo "✅ PASSED"
    else
        echo "❌ FAILED"
    fi
    
    # Test metrics endpoint
    echo -n "  → Metrics endpoint: "
    if curl -s http://localhost:3000/metrics | grep -q "hal9"; then
        echo "✅ PASSED"
    else
        echo "❌ FAILED"
    fi
    
    # Clean up
    echo
    echo "🧹 Shutting down server..."
    kill $SERVER_PID 2>/dev/null || true
    
    echo
    echo "✨ Basic server test complete!"
else
    echo "❌ Failed to build server binary"
    echo
    echo "📊 Build errors:"
    RUSTFLAGS="-A warnings" cargo build --bin hal9-server 2>&1 | grep -E "error\[E[0-9]+\]:" | head -10
fi