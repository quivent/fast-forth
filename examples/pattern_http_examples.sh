#!/bin/bash
# Pattern HTTP API Usage Examples

echo "=== Fast Forth Pattern HTTP API Examples ==="
echo ""

# Example 1: Health check
echo "Example 1: Health check"
echo "$ curl http://localhost:8080/health"
echo ""

# Example 2: List all patterns
echo "Example 2: List all patterns"
echo "$ curl http://localhost:8080/patterns"
echo ""

# Example 3: Get pattern by ID
echo "Example 3: Get specific pattern"
echo "$ curl http://localhost:8080/patterns/DUP_TRANSFORM_001"
echo ""

# Example 4: Query patterns (POST)
echo "Example 4: Query recursive patterns"
cat << 'EOF'
$ curl -X POST http://localhost:8080/patterns/query \
  -H "Content-Type: application/json" \
  -d '{
    "category": "recursive",
    "limit": 10
  }'
EOF
echo ""
echo ""

# Example 5: Query by performance class
echo "Example 5: Query O(1) patterns"
cat << 'EOF'
$ curl -X POST http://localhost:8080/patterns/query \
  -H "Content-Type: application/json" \
  -d '{
    "performance_class": "O(1)",
    "limit": 5
  }'
EOF
echo ""
echo ""

# Example 6: Query by tags
echo "Example 6: Query patterns with 'factorial' tag"
cat << 'EOF'
$ curl -X POST http://localhost:8080/patterns/query \
  -H "Content-Type: application/json" \
  -d '{
    "tags": ["factorial"]
  }'
EOF
echo ""
echo ""

# Example 7: Complex query
echo "Example 7: Complex query with multiple filters"
cat << 'EOF'
$ curl -X POST http://localhost:8080/patterns/query \
  -H "Content-Type: application/json" \
  -d '{
    "category": "recursive",
    "performance_class": "O(n)",
    "tags": ["optimized"],
    "limit": 5,
    "offset": 0
  }'
EOF
echo ""
echo ""

# Example 8: Get categories
echo "Example 8: List all pattern categories"
echo "$ curl http://localhost:8080/patterns/categories"
echo ""

# Example 9: Query by stack effect
echo "Example 9: Find patterns with specific stack effect"
cat << 'EOF'
$ curl -X POST http://localhost:8080/patterns/query \
  -H "Content-Type: application/json" \
  -d '{
    "stack_effect": "( n -- n! )"
  }'
EOF
echo ""
echo ""

# Example 10: Pagination
echo "Example 10: Paginated query"
cat << 'EOF'
$ curl -X POST http://localhost:8080/patterns/query \
  -H "Content-Type: application/json" \
  -d '{
    "limit": 10,
    "offset": 20
  }'
EOF
echo ""
