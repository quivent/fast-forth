#!/bin/bash
# Install Rust for Fast Forth development
# This enables full optimizations (85-110% of C performance)

set -e

echo "════════════════════════════════════════════════════════════"
echo "  Fast Forth - Rust Installation"
echo "════════════════════════════════════════════════════════════"
echo ""
echo "This will install the Rust toolchain to enable:"
echo "  ✓ 85-110% of C performance (vs 30-50% minimal compiler)"
echo "  ✓ Hindley-Milner type inference"
echo "  ✓ Advanced LLVM optimizations"
echo "  ✓ Ability to modify Fast Forth compiler"
echo ""
echo "Download size: ~1.5 GB"
echo "Install time: 5-25 minutes"
echo ""

# Check if Rust is already installed
if command -v cargo &> /dev/null; then
    RUST_VERSION=$(rustc --version)
    echo "Rust is already installed: $RUST_VERSION"
    echo ""
    read -p "Reinstall anyway? (y/n): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo "Skipping Rust installation."
        exit 0
    fi
fi

# Confirm installation
read -p "Install Rust now? (y/n): " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "Installation cancelled."
    echo ""
    echo "You can still use the minimal Fast Forth compiler:"
    echo "  make -C minimal_forth"
    echo "  ./minimal_forth/forth"
    echo ""
    exit 0
fi

echo ""
echo "Installing Rust..."
echo "════════════════════════════════════════════════════════════"

# Install Rust using rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

# Source cargo environment
source "$HOME/.cargo/env"

echo ""
echo "════════════════════════════════════════════════════════════"
echo "  Rust installation complete!"
echo "════════════════════════════════════════════════════════════"
echo ""
rustc --version
cargo --version
echo ""

# Check if we're in the Fast Forth directory
if [ -f "Cargo.toml" ]; then
    echo "Building Fast Forth with full optimizations..."
    echo "This will take 2-5 minutes..."
    echo ""

    cargo build --release

    echo ""
    echo "════════════════════════════════════════════════════════════"
    echo "  Build complete!"
    echo "════════════════════════════════════════════════════════════"
    echo ""
    echo "Binary location: target/release/fastforth"
    echo "Binary size: $(du -h target/release/fastforth | cut -f1)"
    echo ""
    echo "Install globally:"
    echo "  cargo install --path ."
    echo ""
    echo "Test it:"
    echo "  ./target/release/fastforth --version"
    echo "  ./target/release/fastforth repl"
    echo ""
else
    echo "Not in Fast Forth directory. To build Fast Forth:"
    echo "  cd /path/to/fast-forth"
    echo "  cargo build --release"
    echo ""
fi

echo "════════════════════════════════════════════════════════════"
echo "  Performance Comparison"
echo "════════════════════════════════════════════════════════════"
echo ""
echo "  Minimal compiler:  30-50% of C  (no dependencies)"
echo "  Rust+LLVM:         85-110% of C (just installed!)"
echo ""
echo "  Speedup: 2-3x faster code execution!"
echo ""
