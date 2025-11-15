#!/bin/bash
# Fast Forth Universal Installer
# Detects platform and provides multiple installation options

set -e

VERSION="0.1.0"
REPO="quivent/fast-forth"

echo "════════════════════════════════════════════════════════════"
echo "  Fast Forth Installer v${VERSION}"
echo "════════════════════════════════════════════════════════════"
echo ""

# Detect platform
OS=$(uname -s)
ARCH=$(uname -m)

case "$OS" in
    Darwin)
        case "$ARCH" in
            arm64|aarch64)
                PLATFORM="aarch64-apple-darwin"
                ;;
            x86_64)
                PLATFORM="x86_64-apple-darwin"
                ;;
            *)
                echo "Unsupported macOS architecture: $ARCH"
                exit 1
                ;;
        esac
        ;;
    Linux)
        case "$ARCH" in
            x86_64)
                PLATFORM="x86_64-unknown-linux-gnu"
                ;;
            aarch64|arm64)
                PLATFORM="aarch64-unknown-linux-gnu"
                ;;
            *)
                echo "Unsupported Linux architecture: $ARCH"
                exit 1
                ;;
        esac
        ;;
    MINGW*|MSYS*|CYGWIN*)
        PLATFORM="x86_64-pc-windows-msvc"
        BINARY="fastforth.exe"
        ;;
    *)
        echo "Unsupported operating system: $OS"
        exit 1
        ;;
esac

echo "Detected platform: $PLATFORM"
echo ""

# Installation options
echo "Choose installation method:"
echo ""
echo "  1. Pre-compiled binary (Recommended)"
echo "     Size: 2.6 MB | Time: 10s | Performance: 85-110% of C"
echo "     Dependencies: ZERO"
echo ""
echo "  2. Minimal C compiler (Fallback)"
echo "     Size: 50 KB source | Time: 30s | Performance: 30-50% of C"
echo "     Dependencies: gcc/clang"
echo ""
echo "  3. Build from source (Full Rust)"
echo "     Size: 1.5 GB download | Time: 5-25 min | Performance: 85-110% of C"
echo "     Dependencies: Rust toolchain"
echo ""

read -p "Enter choice [1-3]: " choice

case $choice in
    1)
        echo ""
        echo "Downloading pre-compiled binary..."
        echo "════════════════════════════════════════════════════════════"

        BINARY_URL="https://github.com/${REPO}/releases/latest/download/fastforth-${PLATFORM}"

        if command -v curl &> /dev/null; then
            curl -L "$BINARY_URL" -o fastforth
        elif command -v wget &> /dev/null; then
            wget "$BINARY_URL" -O fastforth
        else
            echo "Error: curl or wget required for download"
            exit 1
        fi

        chmod +x fastforth

        echo ""
        echo "✓ Installation complete!"
        echo ""
        echo "Binary: ./fastforth"
        echo "Size: $(du -h fastforth | cut -f1)"
        echo ""
        echo "Test it:"
        echo "  ./fastforth --version"
        echo "  ./fastforth repl"
        echo ""
        echo "Extract embedded source:"
        echo "  ./fastforth --extract-source"
        echo ""
        ;;

    2)
        echo ""
        echo "Downloading minimal compiler source..."
        echo "════════════════════════════════════════════════════════════"

        if [ -d "fast-forth" ]; then
            echo "Error: fast-forth directory already exists"
            exit 1
        fi

        git clone https://github.com/${REPO}.git fast-forth
        cd fast-forth

        echo ""
        echo "Building minimal compiler..."
        make -C minimal_forth

        echo ""
        echo "✓ Build complete!"
        echo ""
        echo "Binary: ./forth"
        echo "Size: $(du -h forth | cut -f1)"
        echo "Performance: 30-50% of C"
        echo ""
        echo "Test it:"
        echo "  ./forth"
        echo ""
        echo "For full optimizations (85-110% of C):"
        echo "  ./scripts/install-rust.sh"
        echo "  cargo build --release"
        echo ""
        ;;

    3)
        echo ""
        echo "Installing Rust and building from source..."
        echo "════════════════════════════════════════════════════════════"

        # Check if Rust is installed
        if ! command -v cargo &> /dev/null; then
            echo "Installing Rust... (this will take 5-25 minutes)"
            curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
            source "$HOME/.cargo/env"
        else
            echo "Rust already installed: $(rustc --version)"
        fi

        if [ -d "fast-forth" ]; then
            echo "Error: fast-forth directory already exists"
            exit 1
        fi

        git clone https://github.com/${REPO}.git fast-forth
        cd fast-forth

        echo ""
        echo "Building with full optimizations... (2-5 minutes)"
        cargo build --release

        echo ""
        echo "✓ Build complete!"
        echo ""
        echo "Binary: target/release/fastforth"
        echo "Size: $(du -h target/release/fastforth | cut -f1)"
        echo "Performance: 85-110% of C"
        echo ""
        echo "Install globally:"
        echo "  cargo install --path ."
        echo ""
        echo "Test it:"
        echo "  ./target/release/fastforth --version"
        echo "  ./target/release/fastforth repl"
        echo ""
        ;;

    *)
        echo "Invalid choice"
        exit 1
        ;;
esac

echo "════════════════════════════════════════════════════════════"
echo "  Installation Complete!"
echo "════════════════════════════════════════════════════════════"
echo ""
echo "Documentation: https://github.com/${REPO}"
echo "Report issues: https://github.com/${REPO}/issues"
echo ""
