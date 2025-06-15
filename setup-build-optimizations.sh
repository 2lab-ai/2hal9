#!/bin/bash

echo "🚀 Setting up Rust build optimizations..."

# Check if sccache is installed
if ! command -v sccache &> /dev/null; then
    echo "📦 Installing sccache for compilation caching..."
    cargo install sccache
fi

# Check if lld is installed
if ! command -v lld &> /dev/null && ! command -v ld.lld &> /dev/null; then
    echo "📦 Installing LLVM for lld linker..."
    if command -v brew &> /dev/null; then
        brew install llvm
        echo "export PATH=\"/usr/local/opt/llvm/bin:\$PATH\"" >> ~/.zshrc
    else
        echo "⚠️  Please install LLVM manually for lld linker support"
    fi
fi

# Set environment variables
echo "🔧 Setting environment variables..."
export RUSTC_WRAPPER=sccache
export CARGO_BUILD_JOBS=16
export CARGO_INCREMENTAL=1
export RUST_BACKTRACE=1

# Create or update .env file
cat > .env << EOF
# Rust build optimizations
export RUSTC_WRAPPER=sccache
export CARGO_BUILD_JOBS=16
export CARGO_INCREMENTAL=1
export RUST_BACKTRACE=1

# Optional: Use mold linker (even faster than lld)
# export RUSTFLAGS="-C link-arg=-fuse-ld=mold"
EOF

echo "✅ Build optimizations configured!"
echo ""
echo "📌 To use these optimizations:"
echo "   source .env"
echo ""
echo "📊 Current settings:"
echo "   CPU cores: $(sysctl -n hw.ncpu)"
echo "   Parallel jobs: 16"
echo "   Compiler cache: sccache"
echo "   Linker: lld"
echo ""
echo "🏃 Run 'make build-all' to test the optimized build"