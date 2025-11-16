#!/bin/bash

# Test GForth output parsing
test_code="$1"

echo "Testing: $test_code"
echo "---"

printf "%s .s\nbye\n" "$test_code" | gforth 2>&1 | tee /tmp/gforth_output.txt

echo "---"
echo "Raw lines:"
cat /tmp/gforth_output.txt | nl

echo "---"
echo "Lines with '<':"
grep "<" /tmp/gforth_output.txt || echo "No lines with '<' found"
