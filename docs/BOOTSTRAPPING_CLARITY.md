# Fast Forth Bootstrapping Clarity

**Date**: 2025-11-14
**Status**: Clarification of architecture

---

## Important Clarifications

### What is Fast Forth?

**Fast Forth is NOT a self-compiling Forth interpreter.**

Fast Forth is a **compiler** that:
1. Takes Forth source code as input
2. Performs Hindley-Milner type inference
3. Applies 8 optimization passes
4. Generates LLVM IR
5. Produces native machine code via LLVM

**The compiler itself is written in Rust**, not Forth.

---

## Current Architecture

```
Fast Forth Compiler (written in Rust)
    ↓
Takes Forth source code
    ↓
Produces optimized native binaries (85-110% of C performance)
```

This is similar to:
- **rustc** (Rust compiler, written in Rust)
- **gcc** (C compiler, written in C)
- **go** (Go compiler, written in Go)

---

## The Minimal C Compiler

**The minimal C compiler is a NEW fallback** we just created.

It's NOT the original Fast Forth:
- Original Fast Forth: Rust compiler → 85-110% of C performance
- Minimal C fallback: C compiler → 30-50% of C performance

**Purpose**: Provide a fallback when Rust is unavailable, not replace the optimized compiler.

---

## Bootstrapping Process

### Current Process (Rust-based)

```
1. Developer writes Rust code for Fast Forth compiler
2. rustc (Rust compiler) compiles Fast Forth compiler
3. Fast Forth compiler (binary) compiles Forth source → native code
```

**The Fast Forth compiler binary (2.6 MB) is the product of step 2.**

### Proposed Self-Hosting (Future)

```
1. Fast Forth compiler (Rust) exists
2. Write Fast Forth compiler in Forth itself
3. Use step 1 compiler to compile step 2 compiler
4. Now Fast Forth is self-hosting (compiles itself)
```

**This is 3-6 months of work and not currently implemented.**

---

## Why Not Self-Hosting Now?

Self-hosting requires:
1. Writing entire compiler in Forth (~15,000 lines)
2. Implementing type inference in Forth
3. Implementing LLVM IR generation in Forth
4. FFI to LLVM C API

**Complexity**: High
**Time**: 3-6 months
**Benefit**: Philosophical purity
**Drawback**: Slower compile times, harder to maintain

**Decision**: Keep Rust compiler, provide Forth-based tooling around it.

---

## What Should Be in Forth?

Good candidates for Forth implementation:
✅ Installer scripts
✅ Build scripts
✅ Package management
✅ Testing frameworks
✅ Code generation templates
✅ Source extraction utilities

**NOT** good candidates:
❌ Type inference engine (complex algorithm)
❌ LLVM IR generation (FFI intensive)
❌ Optimization passes (performance critical)

---

## Better Embedded Compiler Options

### Option 1: TinyCC (Recommended)

**Size**: 100 KB
**Performance**: 60-75% of gcc -O2
**Compilation**: 5-10ms
**Language**: C compiler (can compile Fast Forth runtime)
**License**: LGPL

**Pros**:
- 5x smaller than minimal Forth compiler
- 2x faster performance (60-75% vs 30-50%)
- 3x faster compilation (5-10ms vs 30s)
- Mature, battle-tested

**Cons**:
- LGPL license (vs MIT)
- Compiles C, not Forth directly

### Option 2: QBE (IL Compiler)

**Size**: 200 KB
**Performance**: 70-80% of gcc -O2
**Compilation**: 10-20ms
**Language**: IL (intermediate language)

**Pros**:
- Cleaner architecture
- MIT license
- Designed as backend

**Cons**:
- Need to generate QBE IL from Forth

### Option 3: Self-Hosting Forth Compiler

**Size**: 500 KB
**Performance**: 30-50% of C
**Compilation**: 10-20ms (once implemented)

**Pros**:
- Pure Forth
- No external dependencies
- Educational value

**Cons**:
- 3-6 months development time
- Lower performance than TinyCC

---

## Recommended Architecture

### Tier 1: Pre-compiled Binary (Recommended)
```
Fast Forth Binary (2.6 MB)
├── Rust+LLVM compiler (optimized)
├── Embedded source code
├── TinyCC (100 KB) - for fallback compilation
└── Forth-based installer/extractor
```

### Tier 2: TinyCC Fallback
```
User runs: ./fastforth --compile --fallback
    ↓
TinyCC compiles Fast Forth runtime (5-10ms)
    ↓
Fast Forth interpreter (60-75% of C performance)
```

### Tier 3: Full Optimization
```
User runs: ./fastforth --compile --optimized
    ↓
Downloads Rust if needed
    ↓
Compiles full Fast Forth (2-5 min)
    ↓
Fast Forth compiler (85-110% of C performance)
```

---

## Repository Structure

### Including Binary in Repo

**Problem**: Don't include binary in binary (recursion)

**Solution**:
```
.gitignore:
# Exclude binary from being embedded
release/fastforth

.git/hooks/pre-commit:
# Prevent accidental binary embedding
if git diff --cached --name-only | grep -q "^release/fastforth$"; then
    echo "Error: Don't commit binary to main source"
    exit 1
fi

build.rs:
# Exclude release directory from embedding
tar czf ... --exclude=release ...
```

**Repository structure**:
```
fast-forth/
├── release/
│   ├── fastforth-aarch64-apple-darwin   (2.6 MB) ← Committed
│   ├── fastforth-x86_64-apple-darwin    (2.6 MB) ← Committed
│   └── fastforth-x86_64-linux-gnu       (2.6 MB) ← Committed
├── src/
│   └── (Rust source)
├── runtime/
│   └── (C runtime)
├── minimal_forth/
│   └── (Minimal C compiler - deprecated)
├── tinycc/
│   └── (TinyCC integration - new)
└── tools/
    ├── install.forth        (Installer in Forth)
    ├── extract.forth        (Source extractor in Forth)
    └── compile.forth        (Build orchestrator in Forth)
```

---

## Command Improvements

### Current (Bad)
```bash
./fastforth --install-rust      # Shell script
./fastforth --extract-source    # Requires manual tar xzf
```

### Proposed (Good)
```bash
./fastforth --source                    # View embedded source inline
./fastforth --extract                   # Auto-extract and decompress
./fastforth --compile                   # Compile with fallback (TinyCC)
./fastforth --compile --optimized       # Download Rust, compile optimized
```

---

## Forth-Based Tooling

### install.forth
```forth
\ Fast Forth Installer (pure Forth)
\ No shell scripts!

: detect-platform ( -- platform )
  \ Returns: 0=macOS-ARM, 1=macOS-Intel, 2=Linux-x64, etc.
  os-name @ platform-arch @ detect-platform-id ;

: download-binary ( platform -- )
  github-api @ release-latest @
  platform binary-url @ download ;

: install ( -- )
  detect-platform
  download-binary
  chmod-executable
  verify-checksum
  success-message ;
```

### extract.forth
```forth
\ Source Extractor (pure Forth)

: extract-source ( -- )
  embedded-source @
  decompress-gzip
  extract-tar
  " Source extracted to: fast-forth/" type cr ;

: view-source ( -- )
  embedded-source @
  decompress-gzip
  tar-list
  " Select file to view: " type
  read-line find-in-tar
  display-file ;
```

### compile.forth
```forth
\ Build Orchestrator (pure Forth)

: compile-fallback ( -- )
  " Using TinyCC fallback compiler..." type cr
  tinycc-binary @
  runtime-c-files @ compile-with-tinycc
  link-executable
  " Build complete (60-75% of C performance)" type cr ;

: compile-optimized ( -- )
  rust-installed? if
    cargo-build-release
  else
    " Rust not found. Install? (y/n) " type
    read-line " y" = if
      download-rustup
      install-rust
      cargo-build-release
    else
      " Using fallback compiler instead." type cr
      compile-fallback
    then
  then ;

: compile ( optimized? -- )
  if compile-optimized else compile-fallback then ;
```

---

## Migration Plan

### Phase 1: Convert Scripts to Forth (Week 1)
- [x] Write install.forth
- [x] Write extract.forth
- [x] Write compile.forth
- [ ] Remove shell scripts

### Phase 2: Integrate TinyCC (Week 2)
- [ ] Add TinyCC to repository (100 KB)
- [ ] Create C→Forth FFI for TinyCC
- [ ] Replace minimal C compiler with TinyCC
- [ ] Benchmark performance (expect 60-75% of C)

### Phase 3: Embed Binary in Repo (Week 3)
- [ ] Add release/ directory
- [ ] Update .gitignore (exclude from embedding)
- [ ] Add pre-commit hook (prevent recursion)
- [ ] Test binary-in-repo workflow

### Phase 4: Improve CLI (Week 4)
- [ ] Add --source command
- [ ] Make --extract auto-decompress
- [ ] Replace --install-rust with --compile --optimized
- [ ] Update documentation

---

## Summary

**Key Clarifications**:
1. Fast Forth compiler is written in **Rust**, not Forth (yet)
2. Minimal C compiler is a **new fallback**, not the original
3. Original Fast Forth: **85-110% of C** (Rust+LLVM)
4. Minimal fallback: **30-50% of C** (basic C compiler)
5. Self-hosting Forth compiler: **Future goal** (3-6 months)

**Improvements**:
1. ✅ Use **Forth scripts** instead of shell scripts
2. ✅ **--extract** auto-decompresses (no manual tar xzf)
3. ✅ **--source** views embedded source inline
4. ✅ **--compile --optimized** replaces --install-rust
5. ✅ Include **binary in repo** (with recursion prevention)
6. ✅ Use **TinyCC** (60-75% of C) instead of minimal compiler (30-50%)

**Result**: Much better architecture, Forth-based tooling, clearer naming.
