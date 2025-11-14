# Fast Forth Testing Guide

## Overview

Fast Forth uses a comprehensive multi-layered testing approach to ensure correctness, performance, and robustness.

## Test Categories

### 1. Compliance Tests

**Purpose**: Verify ANS Forth standard compliance

**Location**: `tests/compliance/`

**Coverage**:
- Core word set (6.1): Stack manipulation, arithmetic, logic, control flow
- Extended word set (6.2): Double-cell, memory, strings, advanced control
- Edge cases: Stack underflow, division by zero, overflow
- Error conditions: Invalid input, undefined words

**Running**:
```bash
cargo test --test ans_forth_core
cargo test --test ans_forth_extended
```

**Writing New Tests**:
```rust
#[test]
fn test_new_word() {
    let mut engine = ForthEngine::new();
    engine.eval("test code").unwrap();
    assert_eq!(engine.stack(), &[expected]);
}
```

### 2. Performance Benchmarks

**Purpose**: Measure and compare execution speed

**Location**: `tests/performance/`, `benches/`

**Benchmarks**:
- Sieve of Eratosthenes (loops, arrays)
- Fibonacci (recursion vs iteration)
- Matrix multiplication (nested loops)
- Ackermann function (deep recursion)
- Factorial (simple recursion)
- Tower of Hanoi (recursive algorithm)

**Running**:
```bash
# Quick performance tests
cargo test --test performance

# Detailed benchmarks
cargo bench

# Specific benchmark
cargo bench -- sieve
```

**Benchmark Output**:
```
sieve_rust_10000        time:   [124.53 µs 125.21 µs 125.97 µs]
```

**Writing New Benchmarks**:
```rust
use criterion::{black_box, Criterion};

fn bench_my_algorithm(c: &mut Criterion) {
    c.bench_function("my_algorithm", |b| {
        b.iter(|| {
            my_algorithm(black_box(input))
        });
    });
}
```

### 3. Differential Testing

**Purpose**: Verify Fast Forth produces same results as GForth

**Location**: `tests/correctness/`

**Prerequisites**: GForth must be installed

**Running**:
```bash
# Check if GForth is available
gforth --version

# Run differential tests
cargo test --test differential_testing
```

**How It Works**:
1. Execute Forth code in both Fast Forth and GForth
2. Compare outputs
3. Report any discrepancies

**Adding New Tests**:
```rust
#[test]
fn test_my_code() {
    if !gforth_available() {
        return; // Skip if GForth not installed
    }
    differential_test("my forth code").unwrap();
}
```

### 4. Regression Tests

**Purpose**: Ensure optimizations don't break semantics

**Location**: `tests/regression/`

**What's Tested**:
- Constant folding: `2 3 +` → `5`
- Dead code elimination: `5 DROP 10` → `10`
- Loop unrolling: Unrolled loops = non-unrolled
- Tail call optimization: Recursive = iterative
- Inlining: Inlined = non-inlined
- Register allocation: Values preserved

**Running**:
```bash
cargo test --test optimization_tests
```

**Adding Regression Tests**:
```rust
#[test]
fn test_optimization_preserves_behavior() {
    let mut optimized = ForthEngine::new();
    let mut unoptimized = ForthEngine::new();

    optimized.eval("code").unwrap();
    unoptimized.eval("code").unwrap();

    assert_eq!(optimized.stack(), unoptimized.stack());
}
```

### 5. Fuzzing

**Purpose**: Find crashes, hangs, and edge cases

**Location**: `tests/fuzz/`

**Tools**: cargo-fuzz (libfuzzer)

**Setup**:
```bash
cargo install cargo-fuzz
cd tests/fuzz
```

**Running**:
```bash
# Run indefinitely
cargo fuzz run fuzz_parser

# Run for 5 minutes
cargo fuzz run fuzz_parser -- -max_total_time=300

# Run with specific seed
cargo fuzz run fuzz_parser corpus/seed_file
```

**Interpreting Results**:
- Crashes are saved to `artifacts/fuzz_parser/`
- Minimize crashes: `cargo fuzz cmin fuzz_parser`
- Reproduce: `cargo fuzz run fuzz_parser artifacts/crash-file`

**Writing Fuzz Targets**:
```rust
fuzz_target!(|data: &[u8]| {
    if let Ok(code) = std::str::from_utf8(data) {
        let mut engine = ForthEngine::new();
        let _ = engine.eval(code); // Don't panic!
    }
});
```

## CI/CD Integration

### GitHub Actions Workflows

#### 1. Test Workflow (`.github/workflows/test.yml`)
- Runs on: Every push and PR
- Platforms: Ubuntu, macOS
- Rust versions: stable, nightly
- Steps:
  1. Install Rust and GForth
  2. Run all tests
  3. Run clippy
  4. Check formatting

#### 2. Coverage Workflow
- Runs on: Every push
- Tool: cargo-tarpaulin
- Uploads to: Codecov
- Fails if: Coverage drops

#### 3. Benchmark Workflow
- Runs on: Every PR
- Compares: PR vs main branch
- Fails if: >5% performance regression
- Posts: Results as PR comment

#### 4. Fuzz Workflow (`.github/workflows/fuzz.yml`)
- Runs: Daily at 2 AM UTC
- Duration: 5 minutes per target
- Uploads: Crash artifacts

## Performance Regression Detection

### How It Works
1. Baseline benchmarks stored for main branch
2. PR benchmarks compared against baseline
3. Alert triggered if any benchmark >5% slower
4. Results posted as PR comment

### Example Output
```
Performance Regression Detected!

| Benchmark | Main | PR | Change |
|-----------|------|----|---------
| sieve_1000 | 125µs | 145µs | +16% ⚠️ |
```

### Addressing Regressions
1. Investigate what changed
2. Profile the code
3. Optimize hot paths
4. Verify optimization doesn't break correctness
5. Re-run benchmarks

## Code Coverage

### Running Locally
```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Generate coverage
cargo tarpaulin --out Html

# View report
open tarpaulin-report.html
```

### Coverage Targets
- Overall: >90%
- Core functionality: >95%
- Error handling: >80%

### Improving Coverage
1. Identify uncovered lines: `cargo tarpaulin --out Lcov`
2. Write tests for uncovered code
3. Consider if code is testable
4. Mark untestable code with `#[cfg(not(tarpaulin_include))]`

## Test Organization

```
tests/
├── compliance/
│   ├── ans_forth_core.rs        # Core words (6.1)
│   └── ans_forth_extended.rs    # Extended words (6.2)
├── performance/
│   ├── mod.rs                   # Shared utilities
│   ├── sieve.rs                 # Sieve benchmark
│   ├── fibonacci.rs             # Fibonacci benchmark
│   ├── matrix.rs                # Matrix multiplication
│   └── recursion.rs             # Recursive algorithms
├── correctness/
│   └── differential_testing.rs  # GForth comparison
├── regression/
│   └── optimization_tests.rs    # Optimization correctness
└── fuzz/
    └── fuzz_targets/
        └── fuzz_parser.rs       # Parser fuzzing
```

## Best Practices

### Writing Tests

1. **One concept per test**: Each test should verify one specific behavior
2. **Clear names**: `test_stack_underflow_dup` not `test1`
3. **Arrange-Act-Assert**: Setup, execute, verify
4. **Test edge cases**: Empty input, zero, negative, large values
5. **Test error conditions**: Invalid input, stack underflow, etc.

### Writing Benchmarks

1. **Use black_box**: Prevent compiler from optimizing away code
2. **Realistic inputs**: Use representative data
3. **Consistent environment**: Disable CPU frequency scaling
4. **Multiple iterations**: Let Criterion determine sample size
5. **Document baselines**: Comment expected performance

### Fuzzing Strategy

1. **Start with corpus**: Include known-good and edge case inputs
2. **Run continuously**: Set up daily fuzzing in CI
3. **Minimize crashes**: Use `cargo fuzz cmin` to simplify
4. **Add to regression**: Turn crashes into regression tests
5. **Monitor coverage**: Use coverage-guided fuzzing

## Common Issues

### GForth Not Found
```bash
# Ubuntu/Debian
sudo apt-get install gforth

# macOS
brew install gforth

# Arch Linux
sudo pacman -S gforth
```

### Benchmark Variance
- Close other applications
- Use `cargo bench -- --warm-up-time 3`
- Run multiple times: `cargo bench -- --sample-size 100`

### Fuzzer Slow
- Use multiple cores: `cargo fuzz run fuzz_parser -- -jobs=4`
- Focus on coverage: `cargo fuzz run fuzz_parser -- -use_value_profile=1`

### Coverage Low
- Check for dead code
- Add integration tests
- Test error paths
- Use proptest for property-based testing

## Advanced Topics

### Property-Based Testing

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_arithmetic_properties(a in 0i64..1000, b in 0i64..1000) {
        let mut engine = ForthEngine::new();
        engine.eval(&format!("{} {} +", a, b)).unwrap();
        assert_eq!(engine.stack(), &[a + b]);
    }
}
```

### Snapshot Testing

```rust
use insta::assert_snapshot;

#[test]
fn test_output_format() {
    let output = generate_output();
    assert_snapshot!(output);
}
```

### Mutation Testing

```bash
cargo install cargo-mutants
cargo mutants
```

## Resources

- [Rust Testing Book](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [Criterion User Guide](https://bheisler.github.io/criterion.rs/book/)
- [cargo-fuzz Book](https://rust-fuzz.github.io/book/cargo-fuzz.html)
- [ANS Forth Test Suite](https://forth-standard.org/standard/testsuite)

## Contributing

When adding new features:
1. Write tests first (TDD)
2. Ensure all tests pass
3. Add benchmarks for performance-critical code
4. Update this guide if adding new test categories
5. Maintain >90% coverage

## Questions?

Open an issue on GitHub or consult the main README.md
