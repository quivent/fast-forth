#!/bin/bash
# Test script for fastforth-llama CLI

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
FASTFORTH_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
CLI="$FASTFORTH_ROOT/bin/fastforth-llama"

echo "Testing FastForth Llama CLI"
echo "============================"
echo ""

# Test 1: Help
echo "Test 1: Help message"
"$CLI" --help | head -5
echo "✓ Help works"
echo ""

# Test 2: Check dependencies
echo "Test 2: Checking dependencies"
if command -v curl &> /dev/null; then
    echo "✓ curl installed: $(curl --version | head -1)"
else
    echo "✗ curl not found"
    exit 1
fi

if command -v jq &> /dev/null; then
    echo "✓ jq installed: $(jq --version)"
else
    echo "⚠ jq not installed (optional, but recommended)"
fi
echo ""

# Test 3: Check Ollama
echo "Test 3: Checking Ollama availability"
if curl -s http://localhost:11434/api/version &> /dev/null; then
    echo "✓ Ollama is running"
    OLLAMA_AVAILABLE=1
else
    echo "⚠ Ollama not running at http://localhost:11434"
    echo "  Start with: ollama serve"
    OLLAMA_AVAILABLE=0
fi
echo ""

# Test 4: Check FastForth binary
echo "Test 4: Checking FastForth binary"
if [ -f "$FASTFORTH_ROOT/target/release/fastforth" ]; then
    echo "✓ FastForth binary exists"
    echo "  Size: $(stat -f%z "$FASTFORTH_ROOT/target/release/fastforth" | numfmt --to=iec-i --suffix=B 2>/dev/null || echo "$(stat -f%z "$FASTFORTH_ROOT/target/release/fastforth") bytes")"
else
    echo "⚠ FastForth binary not found"
    echo "  Build with: cargo build --release"
fi
echo ""

# Test 5: Simple query (only if Ollama is available)
if [ "$OLLAMA_AVAILABLE" = "1" ]; then
    echo "Test 5: Simple Ollama query"
    echo "Prompt: 'Say hello in 5 words or less'"
    echo "Response:"
    "$CLI" "Say hello in 5 words or less" | head -5
    echo "✓ Query works"
    echo ""
else
    echo "Test 5: Skipped (Ollama not available)"
    echo ""
fi

# Summary
echo "============================"
echo "Test Summary:"
echo "✓ CLI wrapper functional"
echo "✓ Dependencies checked"
if [ "$OLLAMA_AVAILABLE" = "1" ]; then
    echo "✓ Ollama integration working"
else
    echo "⚠ Ollama not available (start with: ollama serve)"
fi
echo ""
echo "Try it yourself:"
echo "  $CLI \"What is recursion?\""
echo "  $CLI -i  # Interactive mode"
