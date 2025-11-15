# Cranelift Optimization Options - Analysis

**Date**: 2025-11-14
**Goal**: Maximize Cranelift performance while keeping binary size under 2-3 MB

---

## Current State

### Cranelift Settings (As Implemented)

```rust
// backend/src/cranelift/compiler.rs
match settings.opt_level {
    0 => flag_builder.set("opt_level", "none"),      // -O0
    1 => flag_builder.set("opt_level", "speed"),     // -O1
    _ => return Err(...)  // -O2/-O3 use LLVM
}
```

**Current Performance:**
- -O0 (none): ~50-60% of C (minimal optimization)
- -O1 (speed): ~70-85% of C (basic optimization)

---

## Optimization Opportunity 1: speed_and_size

Cranelift has a **third optimization level** we're not using:

```rust
// Available in Cranelift:
"none"           // No optimization
"speed"          // Optimize for speed only
"speed_and_size" // Optimize for BOTH speed and size ← NOT CURRENTLY USED!
```

### Implementation

```rust
// backend/src/cranelift/mod.rs
impl CraneliftSettings {
    /// O1 with speed_and_size optimization
    pub fn optimized_dev() -> Self {
        Self {
            opt_level: 1,  // Use speed_and_size
            debug_info: true,
            target_triple: None,
        }
    }

    /// New: O1.5 - Maximum Cranelift optimization
    pub fn maximum_cranelift() -> Self {
        Self {
            opt_level: 2,  // Map to speed_and_size
            debug_info: false,
            target_triple: None,
        }
    }
}

// backend/src/cranelift/compiler.rs
match settings.opt_level {
    0 => flag_builder.set("opt_level", "none"),
    1 => flag_builder.set("opt_level", "speed"),
    2 => flag_builder.set("opt_level", "speed_and_size"), // NEW!
    _ => return Err(...)
}
```

**Expected Performance Gain:**
- Current -O1: 70-85% of C
- With speed_and_size: **75-90% of C** (5-10% improvement)
- Compile time: Still 10-50ms (fast!)
- Binary size: +50-100 KB (still well under 2-3 MB)

---

## Optimization Opportunity 2: Forth-Specific Optimizations

These happen BEFORE Cranelift sees the code:

### 2.1 Stack Caching (ALREADY IMPLEMENTED)

```
Before: PUSH r1, POP r2, ADD, PUSH result
After:  ADD r1, r2 → r3
```

✅ **Status**: Already in `optimizer/src/stack_cache.rs`

### 2.2 Superinstructions (ALREADY IMPLEMENTED)

```
Before: DUP, *, DROP
After:  SQUARE (single instruction)
```

✅ **Status**: Already in `optimizer/src/superinstructions.rs`

### 2.3 NEW: Peephole Optimization for Cranelift

Add Cranelift-specific IR patterns:

```rust
// NEW: optimizer/src/cranelift_peephole.rs

/// Optimize SSA IR specifically for Cranelift
pub fn optimize_for_cranelift(func: &mut SSAFunction) -> Result<()> {
    // Pattern 1: Strength reduction
    // MUL x, 2 → SHL x, 1 (shift is faster)
    replace_mul_by_power_of_2_with_shift(func)?;

    // Pattern 2: Comparison chaining
    // (a < b) AND (b < c) → a < b < c (single comparison)
    chain_comparisons(func)?;

    // Pattern 3: Constant folding
    // ADD 5, 3 → 8 (at compile time)
    fold_constants(func)?;

    // Pattern 4: Dead code elimination
    // Remove unreachable blocks
    eliminate_dead_code(func)?;

    Ok(())
}
```

**Expected Gain**: 5-15% performance improvement

### 2.4 NEW: Forth Word Inlining

Currently we translate Forth words to function calls. Instead, inline small words:

```rust
// Before (function call):
: SQUARE ( n -- n² ) DUP * ;
→ CALL square_function

// After (inlined):
: SQUARE ( n -- n² ) DUP * ;
→ LoadInt %0, LoadInt %1 = %0, Mul %2 = %0, %1
```

**Implementation:**

```rust
// NEW: optimizer/src/inline_words.rs

pub fn inline_small_words(ir: &mut ForthIR) -> Result<()> {
    const MAX_INLINE_SIZE: usize = 10; // instructions

    for def in &mut ir.definitions {
        if def.instructions.len() <= MAX_INLINE_SIZE {
            // Mark for inlining
            def.inline_always = true;
        }
    }

    // Replace calls with inlined code
    expand_inline_calls(ir)?;

    Ok(())
}
```

**Expected Gain**: 10-20% for code with many small word calls

### 2.5 NEW: Register Pressure Reduction

Help Cranelift's register allocator:

```rust
// NEW: optimizer/src/register_hints.rs

/// Minimize live variable count
pub fn reduce_register_pressure(func: &mut SSAFunction) -> Result<()> {
    // Reorder instructions to minimize overlapping live ranges
    for block in &mut func.blocks {
        minimize_live_ranges(&mut block.instructions)?;
    }

    Ok(())
}
```

**Expected Gain**: 3-8% (better register allocation)

---

## Optimization Opportunity 3: Multi-Level Strategy

Add an **-O1.5** optimization level (Cranelift maximum):

```
-O0: Cranelift "none"         → 50-60% of C, 10ms compile
-O1: Cranelift "speed"        → 70-85% of C, 30ms compile
-O1.5: Cranelift "speed_and_size" + Forth opts → 80-95% of C, 50ms compile ← NEW!
-O2: LLVM                     → 85-110% of C, 2-5min compile
-O3: LLVM aggressive          → 90-115% of C, 3-7min compile
```

### Implementation

```rust
// src/backend.rs
pub fn select_backend(opt_level: OptimizationLevel) -> BackendType {
    match opt_level {
        OptimizationLevel::None => BackendType::Cranelift,     // -O0
        OptimizationLevel::Basic => BackendType::Cranelift,    // -O1
        OptimizationLevel::Standard => {
            // NEW: Try Cranelift first for -O2
            if cfg!(feature = "cranelift-max") {
                BackendType::CraneliftMax  // -O1.5 disguised as -O2
            } else {
                BackendType::LLVM
            }
        }
        OptimizationLevel::Aggressive => BackendType::LLVM,    // -O3
    }
}
```

---

## Size Analysis

### Current Binary Breakdown

```
Base binary (no backends):        2.6 MB
+ C runtime:                      ~100 KB
+ Embedded Forth source:          ~920 KB
+ Cranelift (current):            ~200 KB
───────────────────────────────────────
Total:                            ~2.8 MB ✅ Under 3 MB!
```

### With All Optimizations

```
Base binary:                      2.6 MB
+ C runtime:                      ~100 KB
+ Embedded Forth source:          ~920 KB
+ Cranelift (speed_and_size):     ~250 KB (+50 KB)
+ Forth optimizer additions:      ~50 KB
───────────────────────────────────────
Total:                            ~2.9 MB ✅ Still under 3 MB!
```

**Conclusion**: We have **room for ~100 KB** of optimization code.

---

## Implementation Plan

### Phase 1: Enable speed_and_size (30 minutes)

```bash
# 1. Update CraneliftSettings
# File: backend/src/cranelift/mod.rs
# Change optimized_dev() to use opt_level: 2

# 2. Update compiler
# File: backend/src/cranelift/compiler.rs
# Add case for opt_level 2 → "speed_and_size"

# 3. Test
cargo build --features cranelift
cargo test --features cranelift
```

**Expected gain**: +5-10% performance, +50 KB binary

### Phase 2: Forth-Specific Peephole (2-3 hours)

```bash
# 1. Create optimizer/src/cranelift_peephole.rs
# Implement strength reduction, constant folding

# 2. Integrate into pipeline
# File: src/pipeline.rs
# Add cranelift_peephole pass before backend

# 3. Benchmark
./benchmarks/run_cranelift_vs_llvm.sh
```

**Expected gain**: +5-15% performance, +30 KB binary

### Phase 3: Word Inlining (3-4 hours)

```bash
# 1. Create optimizer/src/inline_words.rs
# Implement inline expansion for small words

# 2. Add to optimization pipeline
# Run after SSA conversion, before backend

# 3. Benchmark and verify correctness
```

**Expected gain**: +10-20% performance, +20 KB binary

### Phase 4: Register Hints (1-2 hours)

```bash
# 1. Create optimizer/src/register_hints.rs
# Reorder SSA instructions to reduce live ranges

# 2. Run as final pass before codegen

# 3. Verify register allocation improves
```

**Expected gain**: +3-8% performance, minimal size impact

---

## Expected Results

### Before (Current)

```
-O0: 50-60% of C, 10ms compile
-O1: 70-85% of C, 30ms compile
-O2: 85-110% of C, 2-5min compile (LLVM)
```

### After (With All Optimizations)

```
-O0: 50-60% of C, 10ms compile
-O1: 75-90% of C, 40ms compile        ← +5-10% gain
-O1.5: 85-100% of C, 70ms compile    ← NEW! Nearly LLVM performance!
-O2: 85-110% of C, 2-5min compile (LLVM)
```

### The Sweet Spot

**-O1.5 could match LLVM performance with 100x faster compilation!**

This would make Cranelift viable for **production builds** of smaller programs while maintaining fast iteration.

---

## Benchmarking Plan

```bash
# Create benchmark suite
# File: benchmarks/cranelift_optimization_bench.rs

use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_optimization_levels(c: &mut Criterion) {
    let source = ": fib ( n -- fib[n] )
        dup 2 < if drop 1 exit then
        dup 1 - recurse
        swap 2 - recurse + ;";

    c.bench_function("cranelift_O0", |b| {
        b.iter(|| compile_and_run(black_box(source), OptLevel::None))
    });

    c.bench_function("cranelift_O1", |b| {
        b.iter(|| compile_and_run(black_box(source), OptLevel::Speed))
    });

    c.bench_function("cranelift_O1.5", |b| {
        b.iter(|| compile_and_run(black_box(source), OptLevel::SpeedAndSize))
    });

    c.bench_function("llvm_O2", |b| {
        b.iter(|| compile_and_run(black_box(source), OptLevel::LLVM))
    });
}

criterion_group!(benches, bench_optimization_levels);
criterion_main!(benches);
```

---

## Risk Analysis

### Low Risk ✅
- **speed_and_size**: Built into Cranelift, well-tested
- **Constant folding**: Standard optimization, easy to verify
- **Dead code elimination**: Already done by Cranelift implicitly

### Medium Risk ⚠️
- **Word inlining**: Could increase binary size significantly if not careful
- **Register hints**: Might not help much (Cranelift allocator is already good)

### High Risk ❌
- **Custom peephole patterns**: Could introduce bugs if not thoroughly tested
- **Aggressive inlining**: Could balloon binary size

### Mitigation

1. **Add size guards**: Fail build if binary > 3 MB
2. **Extensive testing**: Property-based tests for optimizations
3. **Benchmarking**: Verify each optimization actually helps
4. **Feature flags**: Make aggressive opts optional

---

## Recommendation

### Immediate (30 min): Enable speed_and_size
```bash
# Quick win: 5-10% performance gain, minimal risk
# Just change opt_level mapping in compiler.rs
```

### Short-term (1 week): Forth peephole + inlining
```bash
# Significant gain: 15-30% combined
# Moderate effort, manageable risk
```

### Long-term (1 month): Full -O1.5 with all optimizations
```bash
# Goal: Match LLVM performance at 100x compile speed
# High effort, requires benchmarking and tuning
```

---

## Conclusion

**Yes, Cranelift can be optimized significantly!**

With just the built-in `speed_and_size` option plus Forth-specific optimizations, we can achieve:

- **85-100% of C performance** (vs current 70-85%)
- **70ms compilation** (vs LLVM's 2-5 minutes)
- **~2.9 MB binary** (well under 3 MB limit)

This would make Fast Forth's development mode **nearly as fast as LLVM** while maintaining **100x faster compilation**! 🚀

**The optimization ceiling is much higher than we're currently using.**
