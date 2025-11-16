# Fast-Forth: Native Forth JIT Compiler

[![CI](https://github.com/YOUR_USERNAME/fast-forth/actions/workflows/test.yml/badge.svg)](https://github.com/YOUR_USERNAME/fast-forth/actions/workflows/test.yml)
[![codecov](https://codecov.io/gh/YOUR_USERNAME/fast-forth/branch/main/graph/badge.svg)](https://codecov.io/gh/YOUR_USERNAME/fast-forth)

A high-performance Forth compiler targeting native machine code via Cranelift JIT.

## 🎉 Current Status: Working JIT Compiler!

**Fast-forth successfully compiles Forth source code to native x86-64 and executes with correct results!**

```rust
// This works right now!
execute_program("42", true)           → Ok(42) ✅
execute_program("10 20 + 3 *", true)  → Ok(90) ✅
execute_program(": answer 42 ;", true) → Compiles ✅
```

## Quick Start

```bash
# Build the compiler
cd /tmp/fast-forth/cli
cargo build --release

# Run tests
cargo test

# Execute Forth code
./target/release/fastforth execute "10 20 + 3 *"
# Output: 90
```

## Documentation

- **[STATUS.md](STATUS.md)** - Current implementation status and test results
- **[ROADMAP.md](ROADMAP.md)** - Detailed next steps for full functionality
- **[COMPLETION_SUMMARY.md](COMPLETION_SUMMARY.md)** - Comprehensive implementation summary
- **[COVERAGE.md](COVERAGE.md)** - Code coverage documentation and measurement guide
- **[COVERAGE_GAP_ANALYSIS.md](COVERAGE_GAP_ANALYSIS.md)** - Detailed coverage gap analysis and improvement plan

## What's Working

✅ Complete compilation pipeline: Parser → AST → SSA → Cranelift → Native x86-64
✅ Top-level code execution
✅ 14 optimized builtin words
✅ Stack-based calling convention
✅ ~50ms compilation time
✅ Native execution speed

## Next Milestone: Recursion Support

See `ROADMAP.md` Phase 1 for detailed implementation plan (estimated 4-6 hours).

---

**Status**: Working JIT compiler (November 15, 2025)
**Next**: Recursion support (ROADMAP.md Phase 1)
**Goal**: Native Forth execution for llama CLI
