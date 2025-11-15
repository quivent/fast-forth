# PGO Quick Start Guide

## Overview

Profile-Guided Superinstructions (PGO) for Fast Forth automatically detects and fuses common instruction patterns to achieve 20-50% speedup on hot loops.

## Basic Usage (30 seconds)

```rust
use fastforth_optimizer::{ForthIR, PGOOptimizer, PGOConfig};

// 1. Parse Forth code
let ir = ForthIR::parse("5 dup + 3 dup * 1 +")?;

// 2. Create optimizer and enable profiling
let mut pgo = PGOOptimizer::with_config(PGOConfig::aggressive());
pgo.enable_profiling();

// 3. Profile (simulate execution)
for _ in 0..10_000 {
    pgo.profile_ir(&ir);
}

// 4. Find hot patterns and fuse them
let hot = pgo.identify_hot_patterns_adaptive();
let fusions = pgo.generate_fusions(&hot);

// 5. Optimize IR
let (optimized, stats) = pgo.optimize(&ir, 5_000)?;

// 6. Check results
println!("{}", stats);  // See detailed statistics
if let Some(speedup) = pgo.measure_speedup() {
    println!("Speedup: {:.1}%", speedup);
}
```

## Three Configuration Presets

### Aggressive (Best for tight loops)
```rust
let mut pgo = PGOOptimizer::with_config(PGOConfig::aggressive());
// Hot threshold: 5,000 executions
// Max patterns: 150
// Min speedup: 3%
// Iterations: 5
// → Best for: 20-50% speedup target
```

### Balanced (Default, recommended)
```rust
let mut pgo = PGOOptimizer::new();  // Uses balanced config
// Hot threshold: 10,000 executions
// Max patterns: 100
// Min speedup: 5%
// Iterations: 3
// → Best for: General purpose (5-15% speedup)
```

### Conservative (Low risk)
```rust
let mut pgo = PGOOptimizer::with_config(PGOConfig::conservative());
// Hot threshold: 50,000 executions
// Max patterns: 50
// Min speedup: 10%
// Iterations: 1
// → Best for: Guaranteed positive speedup only
```

## Common Patterns (15+ supported)

| Pattern | Fusion | Speedup |
|---------|--------|---------|
| `5 dup +` | DupAdd | 83% |
| `7 dup *` | DupMul | 83% |
| `n 1 +` | IncOne | 83% |
| `n 1 -` | DecOne | 83% |
| `n 2 *` | MulTwo | 83% |
| `n 2 /` | DivTwo | 83% |
| `n 3 +` | LiteralAdd(3) | 80% |
| `n 4 *` | LiteralMul(4) | 80% |
| `a b over +` | OverAdd | 83% |
| `a b swap -` | SwapSub | 83% |

## Understanding Statistics Output

```
PGO Optimization Statistics (Iteration 1):
  Hot patterns found: 24          ← Number of patterns in top 1%
  Fusions generated: 18           ← Patterns successfully fused
  Fusions applied: 156            ← Total times fusions used
  Code reduction: 12.3%           ← Instructions eliminated
  Estimated speedup: 23.5%        ← Expected speedup
  Avg fusion ROI: 4.2             ← Cost/benefit ratio
  Pattern Database Statistics:
    Total patterns: 512           ← All patterns seen
    Hot patterns: 24              ← Top 1% (99th percentile)
    Total instructions: 45,000    ← Instructions profiled
    Coverage: 87.3%               ← % of execution in hot patterns
```

## Key Concepts

### 1. Adaptive Threshold (99th Percentile)
- Automatically finds hot patterns without manual tuning
- Works with any workload (tight loops or scattered code)
- Updates based on execution distribution

### 2. ROI (Return on Investment)
- Formula: `cycles_saved / pattern_length`
- Prioritizes patterns with high impact relative to size
- Prevents bloat while maximizing speedup

### 3. Pattern Ranking
1. **Cycle Savings**: How many cycles the fusion saves
2. **Execution Count**: How often the pattern executes
3. **Pattern Length**: Code size of the original pattern
4. **ROI Score**: Combination of above (primary ranking metric)

### 4. Greedy Matching
- Tries longest patterns first to avoid conflicts
- Prevents overlapping fusions
- Maximizes fusion opportunities

## Advanced Usage

### Custom Configuration

```rust
pub struct PGOConfig {
    pub hot_threshold: u64,           // Execution count threshold
    pub max_patterns: usize,          // Max patterns to track
    pub min_speedup_percent: f64,     // Min speedup to keep fusion
    pub adaptive_mode: bool,          // Auto-adjust threshold
    pub max_iterations: usize,        // Optimization passes
    pub profile_cycles: bool,         // Track cycles
}

// Create custom config
let mut config = PGOConfig::balanced();
config.hot_threshold = 7_500;
config.max_iterations = 4;

let mut pgo = PGOOptimizer::with_config(config);
```

### Measuring Actual Speedup

```rust
use std::time::{Duration, Instant};

// Time baseline execution
let baseline_start = Instant::now();
execute_program(&ir);
let baseline = baseline_start.elapsed();

// Time optimized execution
let opt_start = Instant::now();
execute_program(&optimized_ir);
let optimized = opt_start.elapsed();

// Record times
pgo.set_baseline_time(baseline);
pgo.set_optimized_time(optimized);

// Get actual speedup
if let Some(speedup) = pgo.measure_speedup() {
    println!("Measured speedup: {:.1}%", speedup);
}
```

### Multi-Pass Optimization

```rust
let mut current_ir = ir.clone();

for iteration in 1..=5 {
    let mut pgo = PGOOptimizer::new();
    pgo.enable_profiling();

    // Profile current iteration
    for _ in 0..10_000 {
        pgo.profile_ir(&current_ir);
    }

    // Optimize
    let hot = pgo.identify_hot_patterns_adaptive();
    let (next_ir, stats) = pgo.optimize(&current_ir, 5_000)?;

    println!("Iteration {}: {} fusions applied", iteration, stats.fusions_applied);

    current_ir = next_ir;

    // Stop if diminishing returns
    if stats.fusions_applied < 5 {
        break;
    }
}
```

## Troubleshooting

### Problem: No patterns found
**Solution**: Increase profiling count or lower threshold
```rust
// Profile more times
for _ in 0..50_000 {
    pgo.profile_ir(&ir);
}

// Or use aggressive config
let pgo = PGOOptimizer::with_config(PGOConfig::aggressive());
```

### Problem: Low speedup detected
**Solution**: Check if code has hot loops
```rust
// Check coverage
let db_stats = pgo.database().stats();
println!("Coverage: {:.1}%", db_stats.coverage_percent);

// If <50%, code may not have hot patterns
// Consider different optimization (inlining, stack caching)
```

### Problem: Too many fusions (code bloat)
**Solution**: Use conservative config or higher threshold
```rust
let pgo = PGOOptimizer::with_config(PGOConfig::conservative());
// Higher thresholds reduce pattern count
```

## Performance Expectations

| Code Type | Speedup | Coverage |
|-----------|---------|----------|
| Tight loop (same pattern) | 25-50% | 95%+ |
| Mixed arithmetic | 15-25% | 80-90% |
| Scattered operations | 5-15% | 50-70% |
| No loops | <5% | <30% |

## Integration with Other Optimizations

PGO works well with:
- **Constant Folding**: Reduces patterns further
- **Dead Code Elimination**: Removes unused fusions
- **Inlining**: Exposes new patterns
- **Stack Caching**: Improves execution speed

**Recommended order**:
1. Constant folding
2. Inlining (exposes patterns)
3. **PGO (fuses patterns)**
4. Dead code elimination
5. Stack caching

## API Reference (Brief)

```rust
// Create optimizer
PGOOptimizer::new()                          // Default config
PGOOptimizer::with_config(config)           // Custom config

// Profiling
pgo.enable_profiling()
pgo.profile_ir(&ir)
pgo.disable_profiling()

// Analysis
pgo.identify_hot_patterns(count)            // Fixed threshold
pgo.identify_hot_patterns_adaptive()        // 99th percentile

// Fusion
pgo.generate_fusions(&patterns)
pgo.optimize(&ir, min_count) → (ir, stats)

// Measurement
pgo.set_baseline_time(duration)
pgo.set_optimized_time(duration)
pgo.measure_speedup() → Option<f64>

// Introspection
pgo.database() → &PatternDatabase
pgo.database_mut() → &mut PatternDatabase
```

## Example: Real Forth Program

```rust
let fib = ": fib ( n -- fib(n) )
             dup 1 <= if drop 1 else
               dup 1 - recurse >r
               dup 2 - recurse
               r> +
             then ;";

let ir = ForthIR::parse(fib)?;

let mut pgo = PGOOptimizer::with_config(PGOConfig::aggressive());
pgo.enable_profiling();

// Simulate calling fib(10) many times
for _ in 0..25_000 {
    pgo.profile_ir(&ir);
}

let hot = pgo.identify_hot_patterns_adaptive();
println!("Hot patterns in Fibonacci: {}", hot.len());

let (optimized, stats) = pgo.optimize(&ir, 5_000)?;
println!("Fusions applied: {}", stats.fusions_applied);
println!("Estimated speedup: {:.1}%", stats.estimated_speedup_percent);
```

## Tips & Tricks

1. **Profile for N×execution_count**: If expecting loops to run N times, profile at least N× to get accurate pattern distribution

2. **Use adaptive threshold**: `identify_hot_patterns_adaptive()` is recommended (99th percentile automatically adapts)

3. **Check coverage**: Low coverage (<50%) suggests no hot loops - try other optimizations

4. **Multiple iterations**: Fusions can expose new patterns, run multiple passes for best results

5. **Compare configurations**: Benchmark against different configs to find best for your code

## See Also

- Full documentation: `/Users/joshkornreich/Documents/Projects/FastForth/PGO_IMPLEMENTATION_SUMMARY.md`
- Implementation: `/Users/joshkornreich/Documents/Projects/FastForth/optimizer/src/pgo_superinstructions.rs`
- Tests: `/Users/joshkornreich/Documents/Projects/FastForth/optimizer/tests/pgo_integration_tests.rs`

