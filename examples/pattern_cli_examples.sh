#!/bin/bash
# Pattern CLI Usage Examples

echo "=== Fast Forth Pattern CLI Examples ==="
echo ""

# Example 1: Initialize database with seed data
echo "Example 1: Initialize pattern database"
echo "$ fastforth patterns init --db=patterns.db --seed"
echo ""

# Example 2: List all patterns
echo "Example 2: List all patterns"
echo "$ fastforth patterns list"
echo ""

# Example 3: List patterns by category
echo "Example 3: List recursive patterns"
echo "$ fastforth patterns list --category=recursive"
echo ""

# Example 4: Query patterns with filters
echo "Example 4: Query O(1) patterns in JSON format"
echo "$ fastforth patterns query --perf='O(1)' --format=json"
echo ""

# Example 5: Show specific pattern
echo "Example 5: Show pattern details"
echo "$ fastforth patterns show DUP_TRANSFORM_001"
echo ""

# Example 6: Search patterns
echo "Example 6: Search for factorial patterns"
echo "$ fastforth patterns search factorial"
echo ""

# Example 7: Query by tags
echo "Example 7: Query patterns by tags"
echo "$ fastforth patterns query --tags='recursion,optimized'"
echo ""

# Example 8: Export patterns
echo "Example 8: Export patterns to JSON"
echo "$ fastforth patterns export --output=patterns.json"
echo ""

# Example 9: Show statistics
echo "Example 9: Show pattern library statistics"
echo "$ fastforth patterns stats"
echo ""

# Example 10: Query with limit
echo "Example 10: List first 5 patterns"
echo "$ fastforth patterns list --limit=5"
echo ""

# Example 11: Advanced query
echo "Example 11: Advanced query - recursive patterns with O(n) complexity"
echo "$ fastforth patterns query --category=recursive --perf='O(n)' --format=json"
echo ""

# Example 12: Import patterns
echo "Example 12: Import patterns from JSON"
echo "$ fastforth patterns import --input=custom_patterns.json"
echo ""
