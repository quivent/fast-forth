# Property-Based Fuzzing Quick Start

## TL;DR

```bash
cd tests/fuzz

# Quick validation (< 1 second)
./run_property_tests.sh quick

# Full test suite (~5 minutes)
./run_property_tests.sh standard

# Deep exploration (~15 minutes)
./run_property_tests.sh deep

# Test GForth oracle
./run_property_tests.sh oracle

# Show statistics
./run_property_tests.sh stats
```

## What Is This?

Property-based fuzzing uses **proptest** to:
1. **Generate** thousands of random valid Forth programs
2. **Test** that they don't crash the compiler
3. **Shrink** failing cases to minimal examples
4. **Compare** against GForth (differential oracle)

## Example Generated Tests

The fuzzer automatically generates programs like:

```forth
# Random arithmetic
9847 3621 +
-542 891 *
10000 -9999 /

# Random stack operations
42 DUP DUP DROP SWAP
-17 99 OVER ROT

# Random control flow
1547 8932 > IF 42 ELSE 24 THEN
50 10 DO I 5 > IF 100 THEN LOOP

# Random definitions
: abc 17 42 + ;
: xyz DUP DUP * SWAP ;
```

## Test Coverage

| Test Suite | Cases | Runtime |
|------------|-------|---------|
| Arithmetic operations | 1000 | ~1 min |
| Stack operations | 1000 | ~1 min |
| Control flow (IF/THEN) | 500 | ~30 sec |
| Control flow (IF/ELSE) | 500 | ~30 sec |
| Loops (DO/LOOP) | 500 | ~30 sec |
| Word definitions | 1000 | ~1 min |
| Complex expressions | 1000 | ~1 min |
| Commutativity tests | 1000 | ~30 sec |
| **Total** | **~6000** | **~5 min** |

## Running Tests

### Quick Validation
```bash
./run_property_tests.sh quick
```
Runs only corpus tests (40+ known edge cases). Use this for fast feedback.

### Standard Run (Default)
```bash
./run_property_tests.sh standard
```
Runs all property tests with 1000 cases each (~6000 total cases).

### Deep Exploration
```bash
./run_property_tests.sh deep
```
Runs with 10000 cases per property (~60000 total cases). Takes 10-15 minutes.

### Differential Testing
```bash
./run_property_tests.sh differential
```
Compares Fast Forth against GForth. Requires GForth to be installed:
```bash
brew install gforth  # macOS
apt-get install gforth  # Linux
```

### Custom Case Count
```bash
PROPTEST_CASES=5000 ./run_property_tests.sh standard
```

## What Gets Tested?

### 1. No Crash Property
Every generated program should either:
- Parse successfully, OR
- Return a proper error (not crash/panic)

```rust
proptest! {
    #[test]
    fn prop_arithmetic_no_crash(a, b, op) {
        let code = format!("{} {} {}", a, b, op);
        // Should not panic
        let _ = parse_program(&code);
    }
}
```

### 2. Differential Oracle
When GForth is available, results are compared:

```rust
proptest! {
    #[test]
    fn diff_arithmetic_against_gforth(a, b, op) {
        let gforth_result = run_gforth(&code);
        let fastforth_result = run_fast_forth(&code);
        assert_eq!(gforth_result, fastforth_result);
    }
}
```

### 3. Algebraic Properties
Test mathematical properties:

```rust
// Addition is commutative
assert!(parse("a b +") == parse("b a +"))

// Multiplication is commutative
assert!(parse("a b *") == parse("b a *"))
```

## Shrinking Example

When a test fails, proptest finds the **minimal** failing case:

```
Initial failure: (a=9847, b=3621, op="*")
After shrinking: (a=1, b=0, op="*")
```

This makes debugging much easier!

## Corpus of Edge Cases

The `CORPUS` constant contains known interesting cases:

```forth
# Division by zero
1 0 /

# Integer overflow
2147483647 1 +

# Stack underflow
DROP
SWAP

# Empty loops
0 0 DO I LOOP

# Negative conditionals
-1 IF 42 THEN
```

Run corpus tests:
```bash
cargo test corpus_tests
```

## CI Integration

Property tests run automatically on:
- Every PR
- Every push to main
- Nightly (extended runs)

See `.github/workflows/fuzz.yml`

## Interpreting Results

### Success
```
test prop_arithmetic_no_crash ... ok
test prop_stack_ops_no_crash ... ok
```
All generated cases passed!

### Failure with Shrinking
```
Test failed for inputs: a=9847, b=3621, op="*"
Shrinking...
Minimal failing case: a=1, b=0, op="*"
```
The bug is in how `0 *` is handled.

### Regression Saved
```
Saving failing case to proptest-regressions/
```
This case will be re-run on future tests to prevent regressions.

## Adding New Properties

1. Create a generator:
```rust
fn arb_my_construct() -> impl Strategy<Value = String> {
    // Generate random instances of your construct
}
```

2. Add a property test:
```rust
proptest! {
    #[test]
    fn prop_my_construct_no_crash(input in arb_my_construct()) {
        let _ = parse_program(&input);
    }
}
```

3. Run it:
```bash
cargo test prop_my_construct_no_crash
```

## Troubleshooting

### Tests are slow
```bash
# Reduce case count
PROPTEST_CASES=100 ./run_property_tests.sh standard
```

### GForth not found
```bash
# Install GForth
brew install gforth  # macOS
apt-get install gforth  # Linux

# Or skip differential tests
./run_property_tests.sh standard
```

### Want verbose output
```bash
PROPTEST_VERBOSE=1 cargo test --lib
```

## Resources

- Full documentation: `README.md`
- Implementation details: `../../docs/PROPERTY_BASED_FUZZING.md`
- Proptest book: https://proptest-rs.github.io/proptest/
- GForth docs: https://www.complang.tuwien.ac.at/forth/gforth/Docs-html/

## Summary

Property-based fuzzing provides:
- ✅ Systematic input space exploration
- ✅ Automatic test case generation
- ✅ Shrinking to minimal failures
- ✅ Differential oracle (GForth)
- ✅ ~6000 test cases per run
- ✅ CI integration

Start with `./run_property_tests.sh quick` and go from there!
