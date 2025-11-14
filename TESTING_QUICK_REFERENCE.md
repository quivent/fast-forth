# Fast Forth Testing Quick Reference

## Test Execution

### Run All Tests
```bash
cargo test
```

### Run Specific Test Suite
```bash
cargo test --test integration_tests          # All integration tests
cargo test --lib                             # Library tests only
cargo test --test integration_tests compliance::  # Compliance tests
```

### Run Benchmarks
```bash
cargo bench                                  # All benchmarks
cargo bench -- sieve                         # Specific benchmark
cargo bench -- --save-baseline main          # Save baseline
cargo bench -- --baseline main               # Compare to baseline
```

### Run Fuzzer
```bash
cargo install cargo-fuzz                     # Install once
cd tests/fuzz
cargo fuzz run fuzz_parser                   # Run indefinitely
cargo fuzz run fuzz_parser -- -max_total_time=300  # 5 minutes
```

### Differential Testing (requires GForth)
```bash
# Install GForth
# Ubuntu: sudo apt-get install gforth
# macOS: brew install gforth

cargo test --test integration_tests correctness::
```

## Test Categories

| Category | Location | Tests | Purpose |
|----------|----------|-------|---------|
| **Compliance** | `tests/compliance/` | 40+ | ANS Forth standard |
| **Performance** | `tests/performance/` | 15+ | Benchmarking |
| **Correctness** | `tests/correctness/` | 10+ | Differential testing |
| **Regression** | `tests/regression/` | 10+ | Optimization validation |
| **Fuzzing** | `tests/fuzz/` | 1 | Crash detection |

## CI/CD Workflows

### Test Workflow
- **Trigger**: Every push/PR
- **Platforms**: Ubuntu, macOS
- **Checks**: Tests, clippy, formatting

### Fuzz Workflow
- **Schedule**: Daily 2 AM UTC
- **Duration**: 5 minutes
- **Artifacts**: Crashes saved

## File Locations

```
FastForth/
├── tests/
│   ├── compliance/          # ANS Forth compliance
│   ├── performance/         # Benchmark implementations
│   ├── correctness/         # Differential testing
│   ├── regression/          # Optimization tests
│   └── fuzz/               # Fuzzing targets
├── benches/                # Criterion benchmarks
├── docs/
│   ├── TESTING_GUIDE.md    # Full guide
│   └── BENCHMARK_RESULTS.md # Results template
└── .github/workflows/      # CI/CD
```

## Common Commands

```bash
# Build
cargo build --release

# Test with output
cargo test -- --nocapture

# Run specific test
cargo test test_arithmetic_addition

# Check formatting
cargo fmt -- --check

# Run clippy
cargo clippy -- -D warnings

# Generate coverage (requires cargo-tarpaulin)
cargo tarpaulin --out Html
```

## Quick Stats

- **Test Files**: 14 Rust files
- **Test Lines**: 1,151 lines
- **Compliance Tests**: 40+
- **Benchmark Algorithms**: 5
- **Documentation**: 25,000+ chars

## Performance Targets

| Metric | Target | Status |
|--------|--------|--------|
| Test Count | 1000+ | 40+ (4%) |
| Code Coverage | >90% | TBD |
| Build Time | <30s | TBD |
| Speedup vs GForth | 5-10x | TBD |

## Help & Documentation

- **Full Guide**: `docs/TESTING_GUIDE.md`
- **Completion Report**: `STREAM_7_TESTING_COMPLETION_REPORT.md`
- **README**: `README.md`

## Troubleshooting

### GForth Not Found
```bash
# Ubuntu/Debian
sudo apt-get install gforth

# macOS
brew install gforth
```

### Fuzzer Errors
```bash
cargo install cargo-fuzz
rustup install nightly
```

### Benchmark Variance
- Close other applications
- Use `--warm-up-time 3`
- Run multiple times: `--sample-size 100`
