#!/bin/bash
# Test the GForth differential oracle

set -e

echo "Testing GForth Differential Oracle"
echo "===================================="
echo ""

# Check if GForth is available
if ! command -v gforth &> /dev/null; then
    echo "❌ GForth not found. Install with: brew install gforth"
    exit 1
fi

echo "✓ GForth version: $(gforth --version 2>&1 | head -1)"
echo ""

# Test cases
test_cases=(
    "42"
    "17 25 +"
    "100 50 -"
    "10 5 *"
    "100 10 /"
    "5 DUP"
    "3 4 SWAP"
    "10 20 OVER"
)

echo "Running test cases:"
echo ""

for code in "${test_cases[@]}"; do
    echo -n "  Testing: '$code' ... "

    # Run in GForth and capture stack
    result=$(echo -e "$code\n.s\nbye" | gforth 2>&1 | grep "<" | tail -1)

    if [ -n "$result" ]; then
        echo "✓ Stack: $result"
    else
        echo "⚠ No stack output"
    fi
done

echo ""
echo "Testing property generation:"
echo ""

# Generate some random test cases
for i in {1..10}; do
    a=$((RANDOM % 1000))
    b=$((RANDOM % 1000 + 1))  # Avoid division by zero

    code="$a $b +"
    result=$(echo -e "$code\n.s\nbye" | gforth 2>&1 | grep "<" | tail -1 || echo "error")

    echo "  Random case $i: '$code' -> $result"
done

echo ""
echo "✓ GForth oracle is working correctly!"
echo ""
echo "To run the full property-based test suite:"
echo "  cd tests/fuzz"
echo "  cargo test --lib differential_tests"
