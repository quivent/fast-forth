#!/bin/bash
# build.sh - Fast Forth CLI build script

set -e

echo "🚀 Fast Forth CLI Build Script"
echo "=============================="
echo ""

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo "❌ Error: Rust is not installed"
    echo "Please install Rust from: https://rustup.rs/"
    exit 1
fi

echo "✓ Rust version: $(rustc --version)"
echo ""

# Build type
BUILD_TYPE="${1:-debug}"

if [ "$BUILD_TYPE" = "release" ]; then
    echo "Building release version (optimized)..."
    cargo build --release
    BINARY_PATH="target/release/fastforth"
else
    echo "Building debug version..."
    cargo build
    BINARY_PATH="target/debug/fastforth"
fi

echo ""

# Check if build succeeded
if [ -f "$BINARY_PATH" ]; then
    echo "✓ Build successful!"
    echo ""
    echo "Binary location: $BINARY_PATH"
    echo "Size: $(du -h "$BINARY_PATH" | cut -f1)"
    echo ""
    echo "Quick test:"
    echo "  $BINARY_PATH --version"
    echo "  $BINARY_PATH --help"
    echo "  $BINARY_PATH repl"
    echo "  $BINARY_PATH run examples/hello.fth"
    echo ""

    # Run quick test
    echo "Running quick test..."
    if $BINARY_PATH --version; then
        echo "✓ Quick test passed!"
    else
        echo "⚠ Quick test failed"
    fi
else
    echo "❌ Build failed"
    exit 1
fi

echo ""
echo "Build complete! 🎉"
