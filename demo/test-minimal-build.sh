#!/bin/bash
# Test minimal build of HAL9 server

echo "🔧 Testing minimal HAL9 server build..."
echo

# First, let's check what features are causing issues
echo "📋 Checking feature flags..."
cargo tree -p hal9-server --features 2>&1 | head -20

echo
echo "🏗️ Building with minimal features..."

# Try building with just the core features
export SQLX_OFFLINE=true
export DATABASE_URL="sqlite://./hal9.db"

# Build with warnings allowed
RUSTFLAGS="-A warnings" cargo build --bin hal9-server 2>&1 | grep -E "^error\[E" | head -10

echo
echo "📊 Error summary:"
cargo build --bin hal9-server 2>&1 | grep "^error" | cut -d: -f1 | sort | uniq -c | sort -nr