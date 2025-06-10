#!/bin/bash

# Build script for ha-prompter
# Creates release binaries for multiple platforms

set -e

PROJECT_NAME="ha-prompter"
VERSION=$(grep '^version' Cargo.toml | sed 's/.*"\(.*\)"/\1/')

echo "Building $PROJECT_NAME v$VERSION..."

# Build for current platform
echo "Building for native platform..."
cargo build --release

# Function to build for a specific target
build_target() {
    local target=$1
    echo "Building for $target..."
    
    if cargo build --release --target "$target" 2>/dev/null; then
        echo "✓ Built for $target"
        
        # Create tarball
        local binary_name="$PROJECT_NAME"
        if [[ "$target" == *"windows"* ]]; then
            binary_name="$PROJECT_NAME.exe"
        fi
        
        cd "target/$target/release"
        tar -czf "../../../$PROJECT_NAME-$VERSION-$target.tar.gz" "$binary_name"
        cd ../../../
        echo "✓ Created $PROJECT_NAME-$VERSION-$target.tar.gz"
    else
        echo "✗ Failed to build for $target (target might not be installed)"
    fi
}

# Build for multiple targets if cross-compilation is set up
if command -v cross &> /dev/null; then
    echo "Cross detected, building for multiple platforms..."
    
    # macOS
    build_target "x86_64-apple-darwin"
    build_target "aarch64-apple-darwin"
    
    # Linux  
    build_target "x86_64-unknown-linux-gnu"
    build_target "aarch64-unknown-linux-gnu"
    
    # Windows
    build_target "x86_64-pc-windows-msvc"
else
    echo "Note: Install 'cross' for cross-platform builds"
    echo "cargo install cross"
fi

echo ""
echo "Build complete!"
echo "Native binary: target/release/$PROJECT_NAME"

# Create npm package
if [ -f "package.json" ]; then
    echo ""
    echo "Creating npm package..."
    npm pack
    echo "✓ Created npm package"
fi

echo ""
echo "To publish:"
echo "  - Rust: cargo publish"
echo "  - NPM: npm publish"
echo "  - GitHub: Upload .tar.gz files to release"