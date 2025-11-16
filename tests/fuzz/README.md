# Property-Based Fuzzing for Fast Forth

This directory contains comprehensive property-based testing infrastructure using proptest to systematically explore the input space and find edge cases.

## Overview

The fuzzing infrastructure consists of two complementary approaches:

1. **Property-Based Testing (proptest)** - Systematic generation and testing of valid Forth programs
2. **Coverage-Guided Fuzzing (libfuzzer)** - Mutation-based fuzzing to find crashes

## Property-Based Testing

### Running Tests

```bash
# Run all property tests with default settings (1000 cases per property)
cd tests/fuzz
cargo test --lib

# Run with more test cases for deeper exploration
PROPTEST_CASES=10000 cargo test --lib

# Run specific test suites
cargo test prop_arithmetic_no_crash
cargo test prop_stack_ops_no_crash
cargo test differential_tests

# Run corpus tests
cargo test corpus_tests
```

### Test Strategies

#### 1. Random Expression Generation
Generates random arithmetic expressions and tests them:
- **Test**: `prop_arithmetic_no_crash`
- **Cases**: 1000 by default
- **Example**: `proptest!(|(a, b, c in 0..100)| { test("a b + c *") })`

#### 2. Random Stack Programs
Generates sequences of stack operations:
- **Test**: `prop_stack_ops_no_crash`
- **Cases**: 1000 by default
- **Operations**: DUP, DROP, SWAP, OVER, ROT

#### 3. Random Control Flow
Generates nested control structures:
- **Tests**: `prop_if_then_no_crash`, `prop_if_else_then_no_crash`, `prop_do_loop_no_crash`
- **Cases**: 500 per test
- **Structures**: IF-THEN, IF-ELSE-THEN, DO-LOOP

#### 4. Random Definitions
Generates word definitions:
- **Test**: `prop_word_definition_no_crash`
- **Cases**: 1000 by default
- **Example**: `: square dup * ;`

#### 5. Differential Testing
Compares Fast Forth output against GForth (when available):
- **Tests**: `diff_arithmetic_against_gforth`, `diff_stack_ops_against_gforth`
- **Cases**: 100 per test (slower due to process spawning)
- **Requires**: GForth installed (`apt-get install gforth`)

### Shrinking

When a test fails, proptest automatically **shrinks** the failing case to the minimal example that reproduces the bug. This makes debugging much easier.

Example:
```
If test fails with inputs (a=9847, b=3621, op="*")
Proptest will shrink to minimal case like (a=1, b=0, op="*")
```

### Corpus of Interesting Cases

The `CORPUS` constant contains known edge cases that have found bugs in other Forth implementations:
- Division by zero
- Integer overflow
- Stack underflow
- Empty loops
- Nested control flow

Run corpus tests:
```bash
cargo test test_corpus_no_crash
```

## Coverage-Guided Fuzzing

### Running LibFuzzer

```bash
# Install cargo-fuzz (requires nightly Rust)
cargo +nightly install cargo-fuzz

# Run fuzzer for 5 minutes
cd tests/fuzz
cargo +nightly fuzz run fuzz_parser -- -max_total_time=300

# Run with specific timeout
cargo +nightly fuzz run fuzz_parser -- -max_total_time=3600  # 1 hour

# View crash artifacts
ls artifacts/fuzz_parser/
```

## CI Integration

Both fuzzing approaches run in CI:

### Property Tests (runs on every PR)
- Runs 1000 cases per property
- Includes differential testing against GForth
- Fast feedback (< 5 minutes)

### LibFuzzer (runs nightly)
- Runs for 5 minutes per night
- Longer runs on releases
- Finds deeper bugs through mutation

See `.github/workflows/fuzz.yml` for configuration.

## Test Statistics

Current test coverage:
- **9 property-based test suites**
- **~6000 test cases per run** (default settings)
- **40+ corpus test cases**
- **2 differential test suites** (when GForth available)

Expected runtime:
- Property tests: 2-5 minutes (1000 cases)
- Corpus tests: < 1 second
- Differential tests: 1-2 minutes (100 cases)

## Findings and Regressions

When proptest finds a bug, it saves the failing case to `proptest-regressions/`:
```
tests/fuzz/proptest-regressions/property_tests/prop_arithmetic_no_crash.txt
```

These regression tests are automatically re-run on future test runs to prevent regressions.

## Configuration

Configure proptest behavior via environment variables:
```bash
# Number of test cases per property
export PROPTEST_CASES=10000

# Maximum shrinking iterations
export PROPTEST_MAX_SHRINK_ITERS=10000

# Disable shrinking (faster but less useful failures)
export PROPTEST_MAX_SHRINK_ITERS=0
```

Or in code:
```rust
proptest! {
    #![proptest_config(ProptestConfig {
        cases: 10000,
        max_shrink_iters: 10000,
        ..ProptestConfig::default()
    })]

    #[test]
    fn my_test(...) { ... }
}
```

## Adding New Properties

To add a new property test:

1. Create a generator in `src/property_tests.rs`:
```rust
fn arb_my_construct() -> impl Strategy<Value = String> {
    // ...
}
```

2. Add a property test:
```rust
proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    #[test]
    fn prop_my_construct_no_crash(input in arb_my_construct()) {
        use fastforth_frontend::parse_program;
        let result = parse_program(&input);
        // Assert properties...
    }
}
```

3. Run the test:
```bash
cargo test prop_my_construct_no_crash
```

## Best Practices

1. **Start with simple properties** - "Does not crash" is a great starting point
2. **Use differential oracles** - Compare against GForth when possible
3. **Build up complexity** - Start with simple expressions, then add nesting
4. **Let proptest shrink** - Don't disable shrinking, it finds minimal bugs
5. **Save interesting cases to CORPUS** - Build up a regression suite
6. **Run locally before CI** - Catch issues early

## Resources

- [Proptest Book](https://proptest-rs.github.io/proptest/)
- [Rust Fuzz Book](https://rust-fuzz.github.io/book/)
- [ANS Forth Standard](https://forth-standard.org/)
