# Fast Forth Benchmarking - Quick Start Guide

**Framework**: Performance Validation System v1.0
**Last Updated**: 2025-11-14
**Status**: Production Ready

---

## Quick Start (5 minutes)

### 1. Run Complete Validation

```bash
cd /path/to/FastForth
cargo build --release -p performance_validation
./target/release/perf-validate
```

**Output**: Reports in `benchmarks/performance_validation/results/`

### 2. Run C Baselines Only

```bash
cd benchmarks/c_baseline
make run
```

**Output**: Console results + `benchmark_output.txt`

### 3. Run Individual Benchmarks

```bash
# Sieve
cd benchmarks/c_baseline
./sieve 8190 100

# Matrix
./matrix 100 10

# Fibonacci
./fibonacci 35 40
```

---

## Detailed Usage

### Performance Validation Framework

**Full Pipeline** (recommended):

```bash
# Build
cargo build --release -p performance_validation

# Run validation
./target/release/perf-validate

# Check results
ls -la benchmarks/performance_validation/results/
```

**Output Files**:
- `performance_report_*.md` - Human-readable analysis
- `performance_data_*.json` - Machine-readable results
- `history.json` - Historical baseline

**Interpretation**:
- Tables show time in milliseconds
- `vs Baseline` = speedup from optimizations
- `vs GCC` = ratio of Fast Forth to C performance
- `Status` column: ✓ (good), ⚠ (warning), ✗ (needs work)

---

### C Baseline Commands

```bash
cd benchmarks/c_baseline

# Build all
make all

# Run all benchmarks
make run

# Quick benchmark (smaller iterations)
make benchmark

# Compare optimization levels
make compare

# Test individual benchmarks
make test-sieve
make test-fibonacci
make test-matrix
make test-bubble
make test-string

# Clean
make clean
```

**Benchmark Parameters**:

| Command | Limit | Iterations | Purpose |
|---------|-------|-----------|---------|
| `make run` | Sieve: 8190 | 100 | Full suite |
| `make benchmark` | Sieve: 8190 | 10 | Quick test |
| `make test-sieve` | 8190 | 100 | Single bench |

**Example Output**:
```
Sieve(8190): Found 1027 primes
Average time: 0.004 ms
Validation: PASS (expected 1027 primes)
```

---

### Forth Benchmarks with GForth

**Prerequisites**: `gforth` installed

```bash
# Test Sieve
gforth benchmarks/forth/sieve.fth -e '8190 100 BENCHMARK-SIEVE bye'

# Test Fibonacci
gforth benchmarks/forth/fibonacci.fth -e '40 1000 BENCHMARK-FIBONACCI bye'

# Test Matrix
gforth benchmarks/forth/matrix.fth -e '100 10 BENCHMARK-MATRIX bye'

# Test CoreMark
gforth benchmarks/forth/coremark.fth -e '10000 BENCHMARK-COREMARK bye'
```

**Expected Format**:
```
Testing [Benchmark Name]...
[Benchmark](limit): Found/Result [count]
[duration] ms total, [average] ms average
```

---

## Performance Targets

### Goal: 1.0-1.2x C Speed

**What This Means**:

- `1.0x` = Match C performance exactly ✓ IDEAL
- `1.1x` = 10% slower than C ✓ EXCELLENT
- `1.2x` = 20% slower than C ✓ ACCEPTABLE
- `1.5x` = 50% slower than C ✗ NEEDS IMPROVEMENT
- `2.0x` = Twice as slow as C ✗ SIGNIFICANT WORK NEEDED

### Benchmark-Specific Targets

| Benchmark | Min | Max | Notes |
|-----------|-----|-----|-------|
| Sieve | 1.0x | 1.2x | Simple algorithm |
| Fibonacci | 1.0x | 1.2x | Match VFX (1.09x) |
| Matrix | 0.8x | 1.0x | Can beat C! |
| CoreMark | 0.9x | 1.1x | Computational kernel |

---

## Understanding Results

### Example Report Table

```
| Benchmark | Optimization | Time (ms) | vs Baseline | vs GCC | Status |
|-----------|--------------|-----------|-------------|--------|--------|
| sieve     | None         | 0.050     | 1.00x       | 12.5x  | ✗      |
| sieve     | Inlining     | 0.042     | 1.19x       | 10.5x  | ⚠      |
| sieve     | PGO          | 0.035     | 1.43x       | 8.75x  | ⚠      |
| sieve     | Aggressive   | 0.005     | 10.0x       | 1.25x  | ✓      |
| sieve     | gcc -O2      | 0.004     | -           | 1.00x  | base   |
```

**Interpretation**:
- Baseline (None): Performance without optimizations
- Inlining: ~20% faster than baseline
- PGO: ~40% faster than baseline
- Aggressive: 10x faster than baseline, 1.25x vs C
- Final row: C reference point

---

## Customizing Benchmarks

### C Benchmark Parameters

**File**: `benchmarks/c_baseline/Makefile`

```makefile
# Modify these lines:
./sieve 8190 100        # Change 8190 (limit) or 100 (iterations)
./fibonacci 35 40       # Change 35 (n) or 40 (iterations)
./matrix 100 10         # Change 100 (size) or 10 (iterations)
```

### Forth Benchmark Parameters

**File**: `benchmarks/forth/sieve.fth` (last lines)

```forth
\ Uncomment desired test:
8190 100 BENCHMARK-SIEVE    \ limit iterations
\ 1000 100 BENCHMARK-SIEVE  \ smaller test
```

### Validation Configuration

**File**: `benchmarks/performance_validation/src/main.rs`

```rust
impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            iterations: 100,           // More = more stable
            warmup: true,              // Warmup runs
            warmup_iterations: 10,     // Warmup count
            target_gcc_ratio: 1.0,     // Goal: match C
            regression_threshold: 0.05, // 5% alert threshold
        }
    }
}
```

---

## Troubleshooting

### "Forth benchmark not found"

**Problem**: GForth can't locate .fth files

**Solution**:
```bash
# Use absolute paths
gforth /absolute/path/to/benchmarks/forth/sieve.fth -e 'bye'

# Or run from correct directory
cd /path/to/FastForth
gforth benchmarks/forth/sieve.fth -e 'bye'
```

### "gcc: not found"

**Problem**: C compiler not available

**Solution**:
```bash
# macOS
brew install gcc

# Linux (Ubuntu/Debian)
sudo apt-get install build-essential

# Linux (RedHat/CentOS)
sudo yum install gcc
```

### C benchmark compilation fails

**Problem**: Missing math library

**Solution**: Already handled in Makefile with `-lm` for matrix

**Manual compilation**:
```bash
gcc -Wall -Wextra -O2 matrix.c -o matrix -lm
```

### Performance_validation won't compile

**Problem**: Workspace configuration issues

**Solution**:
```bash
# Ensure performance_validation is in Cargo.toml
cat /path/to/FastForth/Cargo.toml | grep performance_validation

# Should show:
# "benchmarks/performance_validation",

# Try clean build
cargo clean
cargo build --release -p performance_validation
```

---

## Interpreting Output

### Benchmark Run Output

```
Fast Forth Performance Validation
===================================

Step 1: Running C baseline benchmarks...
  Running sieve (C)... 0.004 ms
  Running fibonacci (C)... 1.968 ms
  Running matrix (C)... 0.465 ms
✓ C baselines complete

Step 2: Running Fast Forth benchmarks...
  Running sieve (Forth)... ✓
  Running fibonacci (Forth)... ✓
  Running matrix (Forth)... ✓
✓ Fast Forth benchmarks complete

Step 3: Analyzing optimization impact...
✓ Optimization analysis complete

Step 4: Checking for regressions...
✓ No performance regressions detected

Step 5: Generating performance report...
✓ Report generated

Performance Summary
==================
[Table with results]
```

### Generated Report Sections

1. **Executive Summary**
   - Average optimization speedup
   - Average performance vs GCC
   - Best performing benchmark

2. **Detailed Benchmark Results**
   - Complete timing table
   - Each optimization level
   - C baseline for reference

3. **Optimization Impact Analysis**
   - Per-benchmark analysis
   - Speedup from each optimization
   - Improvement percentages

4. **Performance vs Target Goals**
   - Status for each benchmark
   - Target achievement
   - Performance classification

5. **Regression Analysis**
   - Historical comparison
   - Degradation warnings
   - Trend analysis

---

## Key Metrics Explained

### Time (ms)
Milliseconds to complete benchmark. Lower is better.
- 0.004 ms: Excellent (sub-microsecond)
- 1.0 ms: Good
- 10.0 ms: Acceptable
- 100+ ms: Needs optimization

### vs Baseline (Speedup)
How much faster than baseline (no optimizations):
- 1.0x: Same as baseline
- 1.5x: 50% faster than baseline
- 2.0x: Twice as fast as baseline

### vs GCC (Ratio)
How Fast Forth compares to C:
- 1.0x: Same speed as C
- 1.2x: 20% slower (acceptable)
- 0.8x: 20% faster (beating C!)

### Status Indicator
- ✓ (green): Meets target (≤1.2x vs C)
- ⚠ (yellow): Approaching limit (≤1.5x)
- ✗ (red): Exceeds target (>1.5x)

---

## Best Practices

### Accurate Benchmarking

1. **Close other applications**
   - Reduces system noise
   - More consistent results

2. **Run multiple times**
   - Default 100 iterations
   - Captures performance variation
   - Shows if results are stable

3. **Compare same conditions**
   - Same CPU scaling
   - Same background load
   - Same compilation flags

4. **Track results over time**
   - Save reports with dates
   - Monitor for regressions
   - Identify optimization impact

### Optimization Strategy

1. **Baseline First**
   - Run "None" optimization level
   - Establish starting point
   - Measure effect of each optimization

2. **One At A Time**
   - Inlining → measure
   - PGO → measure
   - All → measure
   - Identify which helps most

3. **Focus on Bottlenecks**
   - Profile hot code paths
   - Optimize frequently-called functions
   - 80/20 rule: optimize 20% to get 80% speedup

4. **Validate Improvements**
   - Always verify correctness
   - Check for regressions
   - Measure impact

---

## File Locations

### Source Benchmarks
- Forth: `benchmarks/forth/{sieve,fibonacci,matrix,coremark}.fth`
- C: `benchmarks/c_baseline/{sieve,fibonacci,matrix,bubble_sort,string_ops}.c`
- Rust: `benchmarks/performance_validation/src/*.rs`

### Build Artifacts
- C executables: `benchmarks/c_baseline/{sieve,fibonacci,matrix,bubble_sort,string_ops}`
- Rust binary: `target/release/perf-validate`

### Reports
- Output directory: `benchmarks/performance_validation/results/`
- History: `benchmarks/performance_validation/results/history.json`

### Configuration
- Rust config: `benchmarks/performance_validation/src/main.rs` (ValidationConfig)
- C config: `benchmarks/c_baseline/Makefile`
- Workspace: `FastForth/Cargo.toml`

---

## Next Steps

### After Running Benchmarks

1. **Analyze Results**
   - Read generated reports
   - Identify performance gaps
   - Compare to targets

2. **Plan Optimizations**
   - List top bottlenecks
   - Prioritize optimizations
   - Estimate impact

3. **Implement & Test**
   - Make optimization changes
   - Re-run benchmarks
   - Measure improvement

4. **Track Progress**
   - Save results with dates
   - Monitor trends
   - Celebrate improvements!

---

## Getting Help

### Common Questions

**Q: Why is my result slower than expected?**
A: Check for system noise. Close other applications and run again.

**Q: How can I compare Fast Forth vs GForth?**
A: Run same .fth file with both interpreters and compare times.

**Q: Should all benchmarks hit 1.0x target?**
A: No! Matrix can be 0.8x (faster than C). Range is 0.8-1.2x.

**Q: How do I know if my optimization worked?**
A: Compare "Before" and "After" reports. Look for speedup in "vs Baseline" column.

---

## Summary

```
┌─────────────────────────────────────────────────┐
│     Fast Forth Performance Benchmarking         │
├─────────────────────────────────────────────────┤
│ Quick test:     make run                        │
│ Full validation: perf-validate                  │
│ Individual:     ./benchmark_name args           │
│ With GForth:    gforth file.fth -e 'word bye'  │
│                                                  │
│ Target:  1.0-1.2x C speed                       │
│ Status:  Production ready                       │
│ Output:  results/performance_report_*.md        │
└─────────────────────────────────────────────────┘
```

---

**Framework Version**: 1.0.0
**Last Updated**: 2025-11-14
**Status**: PRODUCTION READY ✅
