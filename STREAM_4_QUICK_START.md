# Stream 4: Pattern System - Quick Start Guide

## Installation

### 1. Build with Pattern Support
```bash
cd /Users/joshkornreich/Documents/Projects/FastForth
cargo build --release
```

### 2. Initialize Pattern Database
```bash
# Create and seed pattern database
./target/release/fastforth patterns init --db=patterns.db --seed

# Verify installation
./target/release/fastforth patterns list --limit=5
```

---

## CLI Usage

### List Patterns
```bash
# List all patterns (table format)
fastforth patterns list

# List with JSON output
fastforth patterns list --format=json

# Filter by category
fastforth patterns list --category=recursive

# Filter by performance
fastforth patterns list --perf="O(1)"

# Limit results
fastforth patterns list --limit=10
```

### Show Pattern Details
```bash
# Show specific pattern
fastforth patterns show DUP_TRANSFORM_001

# Show with JSON output
fastforth patterns show DUP_TRANSFORM_001 --format=json
```

### Query Patterns
```bash
# Query by category
fastforth patterns query --category=recursive --format=json

# Query by performance class
fastforth patterns query --perf="O(1)" --limit=5

# Query by tags
fastforth patterns query --tags="factorial,optimized"

# Complex query
fastforth patterns query \
  --category=recursive \
  --perf="O(n)" \
  --tags="tail-call" \
  --format=json
```

### Search Patterns
```bash
# Search in description, tags, and category
fastforth patterns search factorial

# Search with table output
fastforth patterns search "loop" --format=table
```

### Statistics
```bash
# Show pattern library statistics
fastforth patterns stats

# Statistics in JSON
fastforth patterns stats --format=json
```

### Import/Export
```bash
# Export patterns to JSON
fastforth patterns export --output=my_patterns.json

# Import patterns from JSON
fastforth patterns import --input=custom_patterns.json
```

---

## HTTP API Usage

### Start Pattern Server
```bash
# Start HTTP API server
fastforth server --patterns --port=8080
```

### API Endpoints

**Health Check**
```bash
curl http://localhost:8080/health
```

**List All Patterns**
```bash
curl http://localhost:8080/patterns
```

**Get Pattern by ID**
```bash
curl http://localhost:8080/patterns/DUP_TRANSFORM_001
```

**Query Patterns**
```bash
curl -X POST http://localhost:8080/patterns/query \
  -H "Content-Type: application/json" \
  -d '{
    "category": "recursive",
    "limit": 10
  }'
```

---

## Rust API Usage

### Basic Pattern Retrieval
```rust
use fastforth::patterns::{PatternDatabase, PatternId};

// Open database
let db = PatternDatabase::open("patterns.db")?;

// Get pattern by ID
let pattern_id = PatternId("DUP_TRANSFORM_001".to_string());
if let Some(pattern) = db.get(&pattern_id)? {
    println!("Pattern: {}", pattern.metadata.description);
    println!("Template:\n{}", pattern.metadata.code_template);
}
```

### Query Patterns
```rust
use fastforth::patterns::{PatternDatabase, PatternQuery};

let db = PatternDatabase::open("patterns.db")?;

// Query recursive patterns
let query = PatternQuery {
    category: Some("recursive".to_string()),
    ..Default::default()
};

let patterns = db.query(&query)?;
for pattern in patterns {
    println!("{}: {}", pattern.metadata.id, pattern.metadata.description);
}
```

### Pattern Template Instantiation
```rust
use fastforth::patterns::instantiate_pattern;
use std::collections::HashMap;

let template = ": NAME ( n -- n² )\n  dup * ;";
let mut substitutions = HashMap::new();
substitutions.insert("NAME".to_string(), "square".to_string());

let code = instantiate_pattern(template, &substitutions)?;
println!("{}", code);
// Output: : square ( n -- n² )
//           dup * ;
```

### Pattern Validation
```rust
use fastforth::patterns::{PatternValidator, PatternId};

let validator = PatternValidator::new(false);

let code = r#"
\ PATTERN: DUP_TRANSFORM_001
: square ( n -- n² )
  dup * ;
"#;

if let Some(pattern_id) = validator.validate_code(code)? {
    println!("Found pattern: {}", pattern_id);
}
```

---

## Pattern Categories

### Available Categories
- **dup_transform** - Patterns using dup for transformation
- **conditional** - If/then conditional patterns
- **accumulator_loop** - Loop-based accumulation
- **recursive** - Standard recursive patterns
- **tail_recursive** - Tail-call optimized recursion
- **binary_op** - Binary operation patterns
- **unary_op** - Unary operation patterns
- **stack_manipulation** - Stack manipulation patterns
- **optimization** - Optimization patterns (bit shifts, etc.)

### Performance Classes
- **O(1)** - Constant time
- **O(log n)** - Logarithmic time
- **O(n)** - Linear time
- **O(n log n)** - Linearithmic time
- **O(n²)** - Quadratic time
- **O(2^n)** - Exponential time

---

## Common Patterns Reference

### DUP_TRANSFORM_001 - Square
```forth
\ PATTERN: DUP_TRANSFORM_001
: square ( n -- n² )
  dup * ;
```

### CONDITIONAL_001 - Absolute Value
```forth
\ PATTERN: CONDITIONAL_001
: abs ( n -- |n| )
  dup 0 < if negate then ;
```

### RECURSIVE_001 - Factorial
```forth
\ PATTERN: RECURSIVE_001
: factorial ( n -- n! )
  dup 2 < if drop 1 else dup 1- recurse * then ;
```

### TAIL_RECURSIVE_001 - Tail Factorial
```forth
\ PATTERN: TAIL_RECURSIVE_001
: factorial-tail ( n acc -- n! )
  over 1 <= if nip else over * swap 1- swap recurse then ;
```

### ACCUMULATOR_LOOP_001 - Sum
```forth
\ PATTERN: ACCUMULATOR_LOOP_001
: sum-to-n ( n -- sum )
  0 swap 1+ 1 do i + loop ;
```

---

## Agent Usage Examples

### Example 1: Find Pattern for Factorial
```python
import requests

# Query for factorial patterns
response = requests.post('http://localhost:8080/patterns/query', json={
    'tags': ['factorial']
})

patterns = response.json()['data']
for pattern in patterns:
    print(f"{pattern['metadata']['id']}: {pattern['metadata']['description']}")
    print(f"Performance: {pattern['metadata']['performance_class']}")
```

### Example 2: Generate Code from Pattern
```python
# Get recursive factorial pattern
response = requests.get('http://localhost:8080/patterns/RECURSIVE_001')
pattern = response.json()['data']

# Use template to generate code
template = pattern['metadata']['code_template']
print(f"Generated code:\n{template}")
```

### Example 3: Validate Code Contains Pattern
```rust
let validator = PatternValidator::new(true); // strict mode

let code = generate_factorial_code()?;
let pattern_id = validator.validate_code(&code)?;

match pattern_id {
    Some(id) => println!("Code uses pattern: {}", id),
    None => eprintln!("Warning: No pattern ID found (strict mode)"),
}
```

---

## Troubleshooting

### Database Not Found
```bash
# Ensure database is initialized
fastforth patterns init --db=patterns.db --seed
```

### No Patterns Returned
```bash
# Check pattern count
fastforth patterns stats

# List all patterns to verify seeding
fastforth patterns list
```

### HTTP Server Not Starting
```bash
# Build with server feature
cargo build --release --features server

# Verify port is available
lsof -i :8080
```

---

## Performance Tips

1. **Use Specific Queries**: Filter by category and performance class to reduce result set
2. **Cache Pattern Database**: Reuse database connection for multiple queries
3. **Query by ID**: Direct ID lookup is fastest (< 1ms)
4. **Limit Results**: Use `--limit` for large result sets
5. **Use HTTP API**: For remote access and concurrent queries

---

## Next Steps

1. Review available patterns: `fastforth patterns list`
2. Explore pattern details: `fastforth patterns show <ID>`
3. Query by category: `fastforth patterns query --category=<CATEGORY>`
4. Integrate into your workflow
5. Consider contributing new patterns

---

## Resources

- **Full Documentation**: `/Users/joshkornreich/Documents/Projects/FastForth/STREAM_4_PATTERN_SYSTEM_REPORT.md`
- **Database Schema**: `/Users/joshkornreich/Documents/Projects/FastForth/patterns/schema.sql`
- **Seed Data**: `/Users/joshkornreich/Documents/Projects/FastForth/patterns/seed.sql`
- **Examples**: `/Users/joshkornreich/Documents/Projects/FastForth/examples/pattern_usage.rs`

---

## Support

For issues or questions:
1. Check pattern statistics: `fastforth patterns stats`
2. Verify database integrity: Re-initialize with `--seed`
3. Review example scripts in `examples/`
