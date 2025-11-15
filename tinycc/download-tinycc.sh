#!/bin/bash
# Download TinyCC for all platforms
# TinyCC: https://bellard.org/tcc/

set -e

VERSION="0.9.27"
BASE_URL="https://download.savannah.gnu.org/releases/tinycc"

echo "Downloading TinyCC v${VERSION} for all platforms..."
echo ""

# Create platform directories
mkdir -p linux-x86_64 linux-i686 linux-arm64
mkdir -p windows-x86_64 windows-i686
mkdir -p macos-x86_64

# Download TinyCC source
if [ ! -f "tcc-${VERSION}.tar.bz2" ]; then
    echo "Downloading TinyCC source..."
    curl -L "${BASE_URL}/tcc-${VERSION}.tar.bz2" -o "tcc-${VERSION}.tar.bz2"
fi

# Extract
if [ ! -d "tcc-${VERSION}" ]; then
    echo "Extracting..."
    tar xjf "tcc-${VERSION}.tar.bz2"
fi

cd "tcc-${VERSION}"

# ============================================================================
# Linux x86-64
# ============================================================================
echo ""
echo "Building for Linux x86-64..."
./configure --prefix="$PWD/../linux-x86_64" --cpu=x86_64
make clean
make
make install
echo "✓ Linux x86-64 complete"

# ============================================================================
# Linux ARM64
# ============================================================================
echo ""
echo "Building for Linux ARM64..."
make clean
./configure --prefix="$PWD/../linux-arm64" --cpu=aarch64 --cross-prefix=aarch64-linux-gnu-
make
make install || echo "Note: ARM64 cross-compile may require aarch64-linux-gnu-gcc"
echo "✓ Linux ARM64 complete (if cross-compiler available)"

# ============================================================================
# Windows x86-64 (cross-compile from Linux/macOS)
# ============================================================================
if command -v x86_64-w64-mingw32-gcc &> /dev/null; then
    echo ""
    echo "Building for Windows x86-64..."
    make clean
    ./configure --prefix="$PWD/../windows-x86_64" --cpu=x86_64 --cross-prefix=x86_64-w64-mingw32-
    make
    make install
    echo "✓ Windows x86-64 complete"
else
    echo "⚠ Skipping Windows build (mingw-w64 not installed)"
    echo "   Install with: brew install mingw-w64 (macOS) or apt-get install mingw-w64 (Linux)"
fi

# ============================================================================
# Summary
# ============================================================================
cd ..
echo ""
echo "═══════════════════════════════════════════════════════════"
echo "  TinyCC Build Summary"
echo "═══════════════════════════════════════════════════════════"
echo ""

for dir in linux-x86_64 linux-arm64 windows-x86_64; do
    if [ -d "$dir/bin" ]; then
        echo "$dir:"
        ls -lh "$dir/bin/tcc" 2>/dev/null || ls -lh "$dir/bin/tcc.exe" 2>/dev/null || echo "  (not built)"
    fi
done

echo ""
echo "Binary sizes:"
find . -name "tcc" -o -name "tcc.exe" | while read f; do
    size=$(du -h "$f" | cut -f1)
    echo "  $f: $size"
done

echo ""
echo "To use:"
echo "  Linux x86-64:   ./linux-x86_64/bin/tcc"
echo "  Linux ARM64:    ./linux-arm64/bin/tcc"
echo "  Windows x86-64: ./windows-x86_64/bin/tcc.exe"
echo ""
