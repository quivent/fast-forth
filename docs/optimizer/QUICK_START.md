# FastForth Optimizer - Quick Start Guide

## Installation

```bash
cd ~/Documents/Projects/FastForth
cargo build --release
```

## Running Examples

### 1. Optimization Demonstration
```bash
cargo run --example optimization_demo
```

This shows before/after comparisons for each optimization pass.

### 2. Code Generation Demo
```bash
cargo run --example codegen_demo
```

See how optimized IR is translated to C code.

### 3. Analysis Demo
```bash
cargo run --example analysis_demo
```

Demonstrates stack depth and data flow analysis.

## Running Benchmarks

```bash
# All benchmarks
cargo bench

# Specific optimizations
cargo bench constant_folding
cargo bench superinstructions
cargo bench stack_caching
cargo bench full_pipeline
```

## Running Tests

```bash
# All tests
cargo test

# Specific modules
cargo test stack_cache
cargo test superinstructions
cargo test constant_fold

# With output
cargo test -- --nocapture
```

## Basic Usage

```rust
use fastforth_optimizer::{ForthIR, Optimizer, OptimizationLevel};

// Parse Forth code
let ir = ForthIR::parse("5 dup + 1 + dup *")?;

// Create optimizer
let optimizer = Optimizer::new(OptimizationLevel::Aggressive);

// Optimize
let optimized = optimizer.optimize(ir)?;
```

## Optimization Levels

| Level | Optimizations | Use Case |
|-------|---------------|----------|
| `None` | None | Debugging only |
| `Basic` | Constant folding, simple DCE | Fast compile time |
| `Standard` | + inlining, stack caching | Balanced |
| `Aggressive` | Maximum optimization | Maximum performance |

## File Structure

```
FastForth/optimizer/
├── src/
│   ├── lib.rs              # Main API
│   ├── ir.rs               # IR definition
│   ├── stack_cache.rs      # Stack caching (2-3x speedup)
│   ├── superinstructions.rs # Pattern fusion (20-30% reduction)
│   ├── constant_fold.rs    # Compile-time evaluation
│   ├── dead_code.rs        # Dead code elimination
│   ├── inline.rs           # Function inlining
│   ├── analysis.rs         # Data flow analysis
│   └── codegen.rs          # C code generation
├── benches/
│   └── optimizer_bench.rs  # Performance benchmarks
├── examples/
│   ├── optimization_demo.rs # Before/after examples
│   ├── codegen_demo.rs     # Code generation
│   └── analysis_demo.rs    # Analysis tools
└── tests/                   # Integration tests
```

## Common Patterns

### Apply individual optimization passes

```rust
// Constant folding only
let folder = ConstantFolder::new();
let folded = folder.fold(&ir)?;

// Superinstructions only
let superopt = SuperinstructionOptimizer::new();
let fused = superopt.recognize(&ir)?;
```

### Generate C code

```rust
use fastforth_optimizer::codegen::{CCodegen, CodegenBackend};

let mut codegen = CCodegen::new();
let c_code = codegen.generate(&optimized)?;
std::fs::write("output.c", c_code)?;
```

### Get optimization statistics

```rust
let optimizer = SuperinstructionOptimizer::new();
let optimized = optimizer.recognize(&ir)?;
let stats = optimizer.get_stats(&ir, &optimized);

println!("Reduction: {:.1}%", stats.reduction_percent);
```

## Performance Targets

✓ **Stack caching**: 2-3x speedup on stack-heavy code
✓ **Superinstructions**: 20-30% code size reduction
✓ **Constant folding**: Zero runtime overhead for constants
✓ **Combined**: 80-100% of hand-written C performance

## Next Steps

1. Read the [full documentation](OPTIMIZER_README.md)
2. Run the examples to see optimizations in action
3. Run benchmarks to verify performance
4. Integrate into your Forth compiler

## Support

- Documentation: `docs/optimizer/OPTIMIZER_README.md`
- Examples: `examples/`
- Tests: Run `cargo test`
- Benchmarks: Run `cargo bench`
