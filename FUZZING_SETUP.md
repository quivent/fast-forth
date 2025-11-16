# 🔬 Fast Forth Long-Running Fuzzing Infrastructure

## Overview

Comprehensive multi-layered fuzzing infrastructure designed to run overnight/weekend to discover edge cases, crashes, and semantic bugs through:

- **Coverage-guided fuzzing** (LibFuzzer) - 5 targets
- **Property-based testing** (PropTest) - 100,000+ test cases
- **Differential fuzzing** (GForth oracle)
- **Stress testing** (extreme values, deep recursion)
- **Automated crash analysis** and minimization

## Quick Start

### 1. Verify Setup

```bash
./scripts/verify_fuzz_setup.sh
```

### 2. Quick Test (5 minutes)

```bash
./scripts/quick_fuzz.sh
```

### 3. Overnight Fuzzing (8 hours default)

```bash
# Start before bed
./scripts/fuzz_overnight.sh

# Or custom duration
FUZZ_DURATION_HOURS=24 ./scripts/fuzz_overnight.sh

# View report in the morning
open tests/fuzz/overnight_reports/fuzz_report_*.html
```

### 4. Analyze Results

```bash
# If crashes found
./scripts/analyze_crashes.sh tests/fuzz/overnight_reports/crashes/

# View detailed logs
ls tests/fuzz/overnight_reports/
```

## What Gets Tested

### LibFuzzer Targets (Coverage-Guided)

1. **fuzz_parser** - Parser robustness
   - Random byte sequences
   - Coverage-guided mutation
   - ~5M executions per 8 hours

2. **fuzz_compiler** - End-to-end compilation
   - Parse → AST → SSA → Optimize → Codegen
   - ~500k executions per 8 hours

3. **fuzz_ssa** - SSA construction
   - Phi node placement
   - Variable renaming
   - Control flow merges

4. **fuzz_optimizer** - Optimization passes
   - Constant folding
   - Dead code elimination
   - Inlining
   - Type specialization

5. **fuzz_codegen** - Code generation
   - Cranelift IR generation
   - Register allocation
   - Calling conventions

### Property-Based Tests

**Test configurations:**
- 10,000 cases (baseline)
- 50,000 cases (extended)
- 100,000 cases (deep exploration)

**Properties tested:**
- Arithmetic operations don't crash
- Stack operations maintain balance
- Control flow structures parse correctly
- Word definitions are valid
- Optimizations preserve semantics

### Differential Testing

**Against GForth oracle:**
- 50,000 random Forth programs
- Compares output for divergences
- Validates correctness

**Requires:** `brew install gforth` (macOS) or `apt install gforth` (Linux)

### Stress Tests

**Extreme values:**
- i64::MAX, i64::MIN
- Overflow/underflow scenarios
- Division by small values

**Pathological cases:**
- Deep recursion (1000+ levels)
- Large stacks (10,000+ items)
- Deeply nested control flow
- Memory-intensive operations

## Infrastructure Components

### Scripts

```
scripts/
├── fuzz_overnight.sh      # Main overnight fuzzing (8+ hours)
├── quick_fuzz.sh          # Quick 5-minute test
├── analyze_crashes.sh     # Crash analysis and minimization
└── verify_fuzz_setup.sh   # Setup verification
```

### Fuzz Targets

```
tests/fuzz/fuzz_targets/
├── fuzz_parser.rs         # Parser fuzzing
├── fuzz_compiler.rs       # End-to-end compilation
├── fuzz_ssa.rs            # SSA construction
├── fuzz_optimizer.rs      # Optimization passes
└── fuzz_codegen.rs        # Code generation
```

### Test Modules

```
tests/fuzz/src/
├── lib.rs                 # Library exports
├── property_tests.rs      # PropTest properties
└── stress_tests.rs        # Stress and edge cases
```

### Output Structure

```
tests/fuzz/
├── artifacts/             # LibFuzzer crashes
│   └── fuzz_parser/
│       └── crash-*
├── corpus/                # Interesting test cases
│   └── parser/
├── overnight_reports/     # HTML reports + logs
│   ├── fuzz_report_*.html
│   ├── crashes/
│   └── *.log
└── proptest-regressions/  # PropTest failures
```

## Expected Results (8-hour run)

### Coverage Statistics

| Component | Executions | Coverage | Crashes Expected |
|-----------|-----------|----------|------------------|
| Parser | ~5,000,000 | High | 0-2 |
| Compiler | ~500,000 | Medium-High | 0-3 |
| SSA | ~1,000,000 | Medium | 0-2 |
| Optimizer | ~300,000 | Medium | 0-1 |
| Property Tests | 160,000 | High | 0-5 |
| Differential | 50,000 | High | 0-10* |

\* Divergences from GForth (may indicate bugs OR spec differences)

### Performance Benchmarks

**Quick Fuzz (5 min/target):**
- Total time: ~25 minutes
- Parser executions: ~50,000
- Total test cases: ~60,000
- Good for: Pre-commit checks

**Overnight (8 hours):**
- Total time: 8 hours
- Total executions: ~7M
- Total test cases: ~210,000
- Good for: Nightly CI, pre-release

**Weekend (48 hours):**
- Total time: 48 hours
- Total executions: ~42M
- Total test cases: ~1.2M
- Good for: Major releases, deep testing

## Using the System

### Development Workflow

#### Before Committing
```bash
./scripts/quick_fuzz.sh
```

#### Nightly CI
```bash
FUZZ_DURATION_HOURS=1 ./scripts/fuzz_overnight.sh
```

#### Before Release
```bash
FUZZ_DURATION_HOURS=24 ./scripts/fuzz_overnight.sh
```

### When Bugs Are Found

1. **Check the report:**
   ```bash
   open tests/fuzz/overnight_reports/fuzz_report_*.html
   ```

2. **Analyze crashes:**
   ```bash
   ./scripts/analyze_crashes.sh tests/fuzz/artifacts/
   ```

3. **Reproduce:**
   ```bash
   cd tests/fuzz
   cargo +nightly fuzz run fuzz_parser artifacts/fuzz_parser/crash-abc123
   ```

4. **Debug with backtrace:**
   ```bash
   RUST_BACKTRACE=1 cargo +nightly fuzz run fuzz_parser artifacts/fuzz_parser/crash-abc123
   ```

5. **Minimize:**
   ```bash
   cargo +nightly fuzz cmin fuzz_parser artifacts/fuzz_parser/crash-abc123
   ```

6. **Add regression test:**
   - Copy minimized case to `tests/regression/`
   - Add test to prevent recurrence

### Manual Fuzzing

#### Run specific target
```bash
cd tests/fuzz
cargo +nightly fuzz run fuzz_parser -- -max_total_time=3600  # 1 hour
```

#### Run with custom corpus
```bash
cargo +nightly fuzz run fuzz_parser corpus/parser/
```

#### Generate coverage report
```bash
cargo +nightly fuzz coverage fuzz_parser
```

#### Run property tests
```bash
# Standard (1k cases)
cargo test --lib

# Extended (100k cases)
PROPTEST_CASES=100000 cargo test --lib

# Specific test
cargo test prop_arithmetic_no_crash
```

## Configuration

### Environment Variables

```bash
# Overnight fuzzing duration
export FUZZ_DURATION_HOURS=12

# PropTest configuration
export PROPTEST_CASES=100000
export PROPTEST_MAX_SHRINK_ITERS=10000
```

### LibFuzzer Options

Pass options after `--`:

```bash
cargo +nightly fuzz run fuzz_parser -- \
    -max_total_time=3600 \      # 1 hour
    -timeout=10 \               # 10 sec per test
    -rss_limit_mb=2048 \        # 2GB memory limit
    -print_final_stats=1 \      # Show stats
    -dict=dictionary.txt        # Use dictionary
```

## Installation

### Prerequisites

```bash
# Rust nightly (required)
rustup install nightly

# cargo-fuzz (will be auto-installed)
cargo +nightly install cargo-fuzz

# GForth (optional, for differential testing)
brew install gforth  # macOS
apt install gforth   # Linux
```

### Verify Installation

```bash
./scripts/verify_fuzz_setup.sh
```

## Troubleshooting

### "cargo-fuzz not found"

The overnight script auto-installs it, or run:
```bash
cargo +nightly install cargo-fuzz
```

### Build Errors

```bash
# Clean and rebuild
cd tests/fuzz
cargo clean
cargo build --bins
```

### Out of Memory

Reduce parallel targets or add memory limits:
```bash
# Edit fuzz_overnight.sh, add to libfuzzer commands:
-rss_limit_mb=2048
```

### Fuzzing Takes Forever

This is expected! Set a time limit:
```bash
FUZZ_DURATION_HOURS=1 ./scripts/fuzz_overnight.sh
```

### GForth Not Available

Differential testing will be skipped automatically. To install:
```bash
# macOS
brew install gforth

# Ubuntu/Debian
sudo apt install gforth

# Arch
sudo pacman -S gforth
```

## Advanced Usage

### Adding New Fuzz Targets

1. Create target file:
   ```rust
   // tests/fuzz/fuzz_targets/fuzz_myfeature.rs
   #![no_main]
   use libfuzzer_sys::fuzz_target;

   fuzz_target!(|data: &[u8]| {
       // Your fuzzing logic
   });
   ```

2. Register in Cargo.toml:
   ```toml
   [[bin]]
   name = "fuzz_myfeature"
   path = "fuzz_targets/fuzz_myfeature.rs"
   test = false
   doc = false
   ```

3. Add to overnight script (optional):
   ```bash
   # In scripts/fuzz_overnight.sh
   run_libfuzzer_myfeature() { ... }
   ```

### Adding New Properties

```rust
// tests/fuzz/src/property_tests.rs

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    #[test]
    fn prop_my_invariant(input in arb_my_generator()) {
        // Test your invariant
        assert!(my_property_holds(&input));
    }
}
```

### Custom Fuzzing Dictionary

Create `tests/fuzz/dictionary.txt`:
```
# Forth keywords
"DUP"
"DROP"
"SWAP"
"IF"
"THEN"
"LOOP"
```

Use it:
```bash
cargo +nightly fuzz run fuzz_parser -- -dict=dictionary.txt
```

### Corpus Management

```bash
# Merge corpuses
cargo +nightly fuzz cmin fuzz_parser

# Minimize corpus
cargo +nightly fuzz cmin -s1 fuzz_parser

# Add seed files
cp my_test.fth tests/fuzz/corpus/parser/
```

## CI Integration

### GitHub Actions Example

```yaml
# .github/workflows/fuzz.yml
name: Fuzzing

on:
  pull_request:
    paths: ['**.rs', 'Cargo.*']
  schedule:
    - cron: '0 2 * * *'  # 2 AM daily

jobs:
  quick-fuzz:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: dtolnay/rust-toolchain@nightly
      - run: cargo +nightly install cargo-fuzz
      - run: ./scripts/quick_fuzz.sh

  overnight-fuzz:
    runs-on: ubuntu-latest
    if: github.event_name == 'schedule'
    steps:
      - uses: actions/checkout@v3
      - uses: dtolnay/rust-toolchain@nightly
      - run: cargo +nightly install cargo-fuzz
      - run: FUZZ_DURATION_HOURS=4 ./scripts/fuzz_overnight.sh
      - uses: actions/upload-artifact@v3
        with:
          name: fuzz-report
          path: tests/fuzz/overnight_reports/
```

## Best Practices

1. ✅ **Run quick fuzz before commits** - Catches obvious issues
2. ✅ **Run overnight before releases** - Deep testing
3. ✅ **Keep corpus in git** - Regression prevention
4. ✅ **Minimize crashes immediately** - Easier debugging
5. ✅ **Add regressions to test suite** - Prevent reoccurrence
6. ✅ **Review interesting corpus cases** - May reveal design issues
7. ✅ **Update stress tests** - Add fuzzer-discovered patterns
8. ✅ **Monitor coverage trends** - Ensure increasing coverage

## Documentation

- **Main guide:** `/Users/joshkornreich/Documents/Projects/Ollama/llama/variants/fast-forth/fuzz/README.md`
- **Quick reference:** `tests/fuzz/QUICK_REFERENCE.txt`
- **LibFuzzer docs:** https://llvm.org/docs/LibFuzzer.html
- **PropTest docs:** https://proptest-rs.github.io/proptest/
- **Rust Fuzz Book:** https://rust-fuzz.github.io/book/

## Support

**Found a bug?**
- Open issue with minimized test case
- Include fuzzer output and backtrace

**Want to contribute?**
- Add new fuzz targets
- Improve property generators
- Add corpus test cases
- Enhance crash analysis

## Summary

This fuzzing infrastructure provides:

- ✅ **5 coverage-guided fuzz targets** (LibFuzzer)
- ✅ **100,000+ property-based tests** (PropTest)
- ✅ **Differential testing** against GForth oracle
- ✅ **Stress testing** with extreme values
- ✅ **Automated crash analysis** and minimization
- ✅ **HTML reporting** with statistics
- ✅ **Overnight capability** (8+ hours)
- ✅ **CI/CD integration** ready

**Expected runtime:** 8 hours (configurable)
**Expected coverage:** ~7M executions, 210k test cases
**Expected findings:** 0-20 crashes/divergences
**Next steps:** Run `./scripts/fuzz_overnight.sh` before bed!
