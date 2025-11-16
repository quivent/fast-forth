# 🔬 Fast Forth Fuzzing Infrastructure

Comprehensive long-running fuzzing infrastructure for finding edge cases and bugs.

## Quick Start

```bash
# Quick 5-minute fuzz (development)
./scripts/quick_fuzz.sh

# Overnight fuzzing (8 hours by default)
./scripts/fuzz_overnight.sh

# Custom duration (12 hours)
FUZZ_DURATION_HOURS=12 ./scripts/fuzz_overnight.sh

# Analyze any crashes found
./scripts/analyze_crashes.sh tests/fuzz/overnight_reports/crashes/
```

## Fuzzing Strategies

### 1. Coverage-Guided Fuzzing (LibFuzzer)

**What it does:** Mutates random inputs, tracks code coverage, prioritizes inputs that reach new code paths.

**Targets:**
- `fuzz_parser` - Parser robustness
- `fuzz_compiler` - End-to-end compilation
- `fuzz_ssa` - SSA construction
- `fuzz_optimizer` - Optimization passes
- `fuzz_codegen` - Code generation

**Usage:**
```bash
cd tests/fuzz

# Run single target for 1 hour
cargo +nightly fuzz run fuzz_parser -- -max_total_time=3600

# Run with custom corpus
cargo +nightly fuzz run fuzz_parser corpus/parser/

# Generate coverage report
cargo +nightly fuzz coverage fuzz_parser
```

**Benefits:**
- ✅ Discovers crashes automatically
- ✅ Builds corpus of interesting test cases
- ✅ Coverage-guided (explores new code paths)
- ✅ Minimizes failing cases
- ❌ Generates mostly invalid inputs
- ❌ May not find semantic bugs

### 2. Property-Based Testing (PropTest)

**What it does:** Generates structured, valid Forth programs and tests invariants.

**Properties tested:**
- Parser never crashes on valid syntax
- Compilation is deterministic
- Optimizations preserve semantics
- Stack operations maintain balance

**Usage:**
```bash
cd tests/fuzz

# Standard run (1,000 cases per property)
cargo test --lib

# Extended run (100,000 cases)
PROPTEST_CASES=100000 cargo test --lib

# Specific property
cargo test prop_arithmetic_no_crash

# With detailed output
cargo test --lib -- --nocapture
```

**Benefits:**
- ✅ Generates valid Forth programs
- ✅ Tests semantic properties
- ✅ Automatic shrinking to minimal failing case
- ✅ Deterministic (same seed = same tests)
- ❌ Slower than raw fuzzing
- ❌ Limited by quality of generators

### 3. Differential Fuzzing

**What it does:** Compares Fast Forth output against GForth (oracle).

**Usage:**
```bash
# Install GForth first
# macOS: brew install gforth
# Linux: apt-get install gforth

cd tests/fuzz

# Run differential tests
cargo test differential_tests

# Extended differential testing
PROPTEST_CASES=50000 cargo test differential_tests
```

**Benefits:**
- ✅ Finds semantic divergences
- ✅ Validates correctness against known-good implementation
- ✅ Catches optimization bugs
- ❌ Requires GForth installed
- ❌ Slower (spawns processes)

### 4. Stress Testing

**What it does:** Tests extreme values and pathological cases.

**Test cases:**
- Max/min integer values
- Deep recursion (1000+ levels)
- Large stacks (10,000+ items)
- Deeply nested control flow
- Memory-intensive operations

**Usage:**
```bash
cd tests/fuzz
cargo test stress_tests
```

### 5. Overnight Fuzzing

**What it does:** Runs all strategies in parallel for hours/days.

**Features:**
- Runs LibFuzzer on all 5 targets simultaneously
- Runs PropTest with 10k, 50k, 100k cases
- Runs differential testing
- Runs stress tests
- Generates HTML report
- Saves all crashes and interesting corpus

**Usage:**
```bash
# 8-hour overnight run (default)
./scripts/fuzz_overnight.sh

# 24-hour weekend run
FUZZ_DURATION_HOURS=24 ./scripts/fuzz_overnight.sh

# View report when done
open tests/fuzz/overnight_reports/fuzz_report_*.html
```

**Expected Results:**
- **Coverage:** 50,000+ parser executions, 10,000+ compiler executions
- **Properties:** 160,000+ property test cases
- **Corpus:** 100+ interesting test cases
- **Runtime:** Configurable (default 8 hours)

## Analyzing Results

### Crashes Found

```bash
# List all crashes
find tests/fuzz/artifacts -name "crash-*"
find tests/fuzz/overnight_reports/crashes -type f

# Analyze crashes (auto-minimizes)
./scripts/analyze_crashes.sh tests/fuzz/artifacts/

# Reproduce a crash
cd tests/fuzz
cargo +nightly fuzz run fuzz_parser artifacts/fuzz_parser/crash-abc123

# Debug with backtrace
RUST_BACKTRACE=1 cargo +nightly fuzz run fuzz_parser artifacts/fuzz_parser/crash-abc123

# Minimize crash case
cargo +nightly fuzz cmin fuzz_parser artifacts/fuzz_parser/crash-abc123
```

### No Crashes Found

```bash
# Review interesting corpus cases
ls tests/fuzz/corpus/

# Add corpus cases to regression tests
cp tests/fuzz/corpus/*.fth tests/regression/

# Review coverage
cargo +nightly fuzz coverage fuzz_parser
```

## Adding New Fuzz Targets

1. **Create fuzz target:**

```rust
// tests/fuzz/fuzz_targets/fuzz_myfeature.rs
#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Your fuzzing logic
});
```

2. **Add to Cargo.toml:**

```toml
[[bin]]
name = "fuzz_myfeature"
path = "fuzz_targets/fuzz_myfeature.rs"
test = false
doc = false
```

3. **Update overnight script:**

Add to `scripts/fuzz_overnight.sh`:
```bash
run_libfuzzer_myfeature() {
    cargo +nightly fuzz run fuzz_myfeature -- \
        -max_total_time="${DURATION_SECONDS}"
}
```

## Adding New Properties

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

## Directory Structure

```
tests/fuzz/
├── Cargo.toml                 # Fuzz package config
├── README.md                  # This file
├── fuzz_targets/              # LibFuzzer targets
│   ├── fuzz_parser.rs
│   ├── fuzz_compiler.rs
│   ├── fuzz_ssa.rs
│   ├── fuzz_optimizer.rs
│   └── fuzz_codegen.rs
├── src/
│   ├── lib.rs
│   ├── property_tests.rs      # PropTest properties
│   └── stress_tests.rs        # Stress tests
├── artifacts/                 # Crashes from libfuzzer
├── corpus/                    # Interesting test cases
├── overnight_reports/         # HTML reports
└── proptest-regressions/      # PropTest failures

scripts/
├── fuzz_overnight.sh          # Main overnight fuzzing
├── quick_fuzz.sh              # Quick 5-minute fuzz
└── analyze_crashes.sh         # Crash analysis tool
```

## Configuration

### Environment Variables

```bash
# PropTest
export PROPTEST_CASES=100000           # Test cases per property
export PROPTEST_MAX_SHRINK_ITERS=10000 # Shrinking iterations

# Overnight fuzzing
export FUZZ_DURATION_HOURS=12          # Fuzzing duration

# LibFuzzer (pass via --)
# See: https://llvm.org/docs/LibFuzzer.html#options
```

### Common LibFuzzer Options

```bash
# Time limits
-max_total_time=3600          # Fuzz for 1 hour
-timeout=10                   # Per-test timeout

# Corpus management
-merge=1                      # Merge corpus
-minimize_crash=1             # Minimize crashes

# Output
-print_final_stats=1          # Print statistics
-verbosity=1                  # Logging level

# Coverage
-print_coverage=1             # Show coverage info
-print_corpus_stats=1         # Corpus statistics
```

## Best Practices

### Development Workflow

1. **Before committing:**
   ```bash
   ./scripts/quick_fuzz.sh  # 5 minutes
   ```

2. **Before merging PR:**
   ```bash
   cd tests/fuzz
   PROPTEST_CASES=10000 cargo test --lib
   ```

3. **Nightly CI:**
   ```bash
   FUZZ_DURATION_HOURS=1 ./scripts/fuzz_overnight.sh
   ```

4. **Before release:**
   ```bash
   FUZZ_DURATION_HOURS=24 ./scripts/fuzz_overnight.sh
   ```

### When Bugs Are Found

1. **Reproduce:** Verify the crash is reproducible
2. **Minimize:** Use `cargo fuzz cmin` to find minimal case
3. **Debug:** Add backtrace and debug symbols
4. **Fix:** Implement the fix
5. **Regress:** Add to regression suite
6. **Re-fuzz:** Run overnight fuzzing to find similar bugs

### Corpus Management

- **Keep interesting cases:** Save corpus files that increase coverage
- **Merge corpuses:** `cargo fuzz cmin` merges and minimizes
- **Share corpuses:** Commit interesting cases to `tests/corpus/`
- **Seed from real code:** Add real Forth programs to corpus

## Performance Expectations

### Quick Fuzz (5 min per target)
- Parser: ~50,000 execs
- Compiler: ~5,000 execs
- Runtime: 25 minutes total

### Overnight (8 hours)
- Parser: ~5M execs
- Compiler: ~500k execs
- Properties: 160k cases
- Runtime: 8 hours

### Weekend (48 hours)
- Parser: ~60M execs
- Compiler: ~6M execs
- Properties: 1M+ cases
- Runtime: 48 hours

## Continuous Integration

See `.github/workflows/fuzz.yml` for CI configuration.

**PR checks:**
- Quick fuzz (5 min)
- Property tests (1k cases)
- ~10 minutes

**Nightly:**
- Extended fuzzing (1 hour)
- Property tests (10k cases)
- Differential testing
- ~2 hours

**Release:**
- Overnight fuzzing (8 hours)
- All strategies
- Generate report

## Resources

- [LibFuzzer Documentation](https://llvm.org/docs/LibFuzzer.html)
- [Rust Fuzz Book](https://rust-fuzz.github.io/book/)
- [PropTest Book](https://proptest-rs.github.io/proptest/)
- [Fuzzing Best Practices](https://google.github.io/fuzzing/overview/)

## Troubleshooting

### "cargo-fuzz not found"

```bash
cargo +nightly install cargo-fuzz
```

### "nightly toolchain not installed"

```bash
rustup install nightly
```

### Fuzzing runs forever

This is expected! Let it run. Set duration:
```bash
cargo fuzz run target -- -max_total_time=300  # 5 minutes
```

### No crashes but want more coverage

- Increase duration
- Add dictionary (`-dict=dictionary.txt`)
- Seed with real programs
- Add custom mutators

### Out of memory

- Reduce parallel targets
- Add `-rss_limit_mb=2048` to LibFuzzer
- Use smaller corpus

## Support

- **Issues:** Found a bug? Open an issue with minimized test case
- **Corpus:** Found interesting case? Submit PR with corpus file
- **New targets:** Want to fuzz something new? Add a fuzz target!
