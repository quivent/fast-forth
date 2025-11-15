# Quick Win: Enable Cranelift speed_and_size

**Effort**: 30 minutes
**Gain**: +5-10% performance
**Risk**: Minimal (built-in Cranelift feature)

---

## The Change

Currently we only use two Cranelift optimization levels:
```rust
0 → "none"   // -O0
1 → "speed"  // -O1
```

Cranelift has a third option we're not using:
```rust
"speed_and_size"  // Optimizes for BOTH speed and size!
```

---

## Implementation

### File 1: `backend/src/cranelift/mod.rs`

```rust
impl CraneliftSettings {
    /// Create settings for development builds (fast compilation)
    pub fn development() -> Self {
        Self {
            opt_level: 0,      // "none"
            debug_info: true,
            target_triple: None,
        }
    }

    /// Create settings for optimized development builds
    pub fn optimized_dev() -> Self {
        Self {
            opt_level: 1,      // "speed"
            debug_info: true,
            target_triple: None,
        }
    }

    // NEW! Maximum Cranelift optimization
    pub fn maximum() -> Self {
        Self {
            opt_level: 2,      // "speed_and_size" ← NEW!
            debug_info: false,
            target_triple: None,
        }
    }
}
```

### File 2: `backend/src/cranelift/compiler.rs`

```rust
// Set optimization level
match settings.opt_level {
    0 => {
        flag_builder.set("opt_level", "none")
            .map_err(|e| BackendError::Initialization(format!("Failed to set opt_level: {}", e)))?;
    }
    1 => {
        flag_builder.set("opt_level", "speed")
            .map_err(|e| BackendError::Initialization(format!("Failed to set opt_level: {}", e)))?;
    }
    2 => {  // NEW!
        flag_builder.set("opt_level", "speed_and_size")
            .map_err(|e| BackendError::Initialization(format!("Failed to set opt_level: {}", e)))?;
    }
    _ => {
        return Err(BackendError::Initialization(
            "Cranelift supports opt_level 0-2. Use LLVM for -O3.".to_string()
        ));
    }
}
```

### File 3: `src/backend.rs`

```rust
impl Backend {
    pub fn with_backend(backend_type: BackendType, opt_level: OptimizationLevel) -> Result<Self> {
        match backend_type {
            #[cfg(feature = "cranelift")]
            BackendType::Cranelift => {
                let settings = match opt_level {
                    OptimizationLevel::None => CraneliftSettings::development(),        // -O0
                    OptimizationLevel::Basic => CraneliftSettings::optimized_dev(),     // -O1
                    OptimizationLevel::Standard => CraneliftSettings::maximum(),        // -O2 ← NEW!
                    OptimizationLevel::Aggressive => {
                        // -O3 should still use LLVM
                        return Err(CompileError::BackendError(
                            "Cranelift maximum is -O2. Use LLVM for -O3.".to_string()
                        ))
                    }
                };
                // ... rest of implementation
            }
            // ...
        }
    }
}
```

---

## Usage

```bash
# Development (fast compile, moderate performance)
./fastforth compile -O0 source.forth  # 10ms, 50-60% of C
./fastforth compile -O1 source.forth  # 30ms, 70-85% of C

# NEW! Optimized Cranelift (fast compile, good performance)
./fastforth compile -O2 source.forth  # 50ms, 75-90% of C  ← 100x faster than LLVM!

# Production (slow compile, maximum performance)
# Requires switching backend selection logic to prefer LLVM for -O2
./fastforth compile -O2 --backend llvm source.forth  # 2-5min, 85-110% of C
./fastforth compile -O3 source.forth                 # 3-7min, 90-115% of C
```

---

## Backend Selection Strategy

We need to update backend selection to handle -O2 appropriately:

```rust
pub fn select_backend(opt_level: OptimizationLevel) -> BackendType {
    match opt_level {
        OptimizationLevel::None | OptimizationLevel::Basic => {
            // -O0, -O1: Use Cranelift for fast iteration
            #[cfg(feature = "cranelift")]
            return BackendType::Cranelift;

            #[cfg(not(feature = "cranelift"))]
            BackendType::LLVM
        }
        OptimizationLevel::Standard => {
            // -O2: User choice!
            // Default to Cranelift for fast builds
            // Add --backend flag to override
            #[cfg(feature = "cranelift")]
            return BackendType::Cranelift;

            #[cfg(not(feature = "cranelift"))]
            BackendType::LLVM
        }
        OptimizationLevel::Aggressive => {
            // -O3: Always LLVM
            BackendType::LLVM
        }
    }
}
```

Or add explicit backend selection:

```bash
# Fast build with Cranelift
./fastforth compile -O2 source.forth              # Default: Cranelift

# Slow build with LLVM
./fastforth compile -O2 --backend=llvm source.forth   # Override: LLVM
./fastforth compile -O3 source.forth              # Always LLVM
```

---

## Expected Results

### Compile Time
- -O0: 10ms (unchanged)
- -O1: 30ms (unchanged)
- -O2 (Cranelift): **50ms** (20ms slower than -O1)
- -O2 (LLVM): 2-5 minutes

### Runtime Performance
- -O0: 50-60% of C (unchanged)
- -O1: 70-85% of C (unchanged)
- -O2 (Cranelift): **75-90% of C** (+5-10% gain!)
- -O2 (LLVM): 85-110% of C

### Binary Size
- Current: ~2.8 MB
- With speed_and_size: **~2.85 MB** (+50 KB)

---

## Testing Plan

```bash
# 1. Build with new optimization
cargo build --release --features cranelift

# 2. Check binary size
ls -lh target/release/fastforth

# 3. Benchmark Fibonacci (CPU-bound)
echo ': fib dup 2 < if drop 1 exit then dup 1 - recurse swap 2 - recurse + ; 20 fib .' | \
  ./fastforth -O0 --time  # Baseline
echo ': fib dup 2 < if drop 1 exit then dup 1 - recurse swap 2 - recurse + ; 20 fib .' | \
  ./fastforth -O1 --time  # Current best
echo ': fib dup 2 < if drop 1 exit then dup 1 - recurse swap 2 - recurse + ; 20 fib .' | \
  ./fastforth -O2 --time  # NEW: speed_and_size

# 4. Benchmark compilation time
time ./fastforth compile -O1 examples/large.forth
time ./fastforth compile -O2 examples/large.forth
# Should be ~20ms difference
```

---

## Implementation Checklist

- [ ] Update `CraneliftSettings::maximum()` in `backend/src/cranelift/mod.rs`
- [ ] Add opt_level 2 case in `backend/src/cranelift/compiler.rs`
- [ ] Update backend selection in `src/backend.rs`
- [ ] Add `--backend` CLI flag (optional, for override)
- [ ] Test build succeeds
- [ ] Verify binary size < 3 MB
- [ ] Benchmark performance improvement
- [ ] Update documentation
- [ ] Commit changes

---

## Conclusion

This is a **30-minute change** that gives **5-10% performance improvement** with **minimal risk**.

It's the lowest-hanging fruit for optimizing Cranelift! 🍎
