# Platform-Specific Code Analysis

## Overview
This document catalogs all platform-specific code paths in the Fast Forth compiler and provides a testing strategy for each.

## 1. C Runtime Platform-Specific Code

### 1.1 Architecture-Specific Optimizations (`runtime/forth_runtime.h`)

**Location**: Lines 234-251

**Code Paths**:
```c
#ifdef __x86_64__
- fast_add() - x86-64 inline assembly optimized addition
- fast_mul() - x86-64 inline assembly optimized multiplication
#else
- Standard C fallback implementations
#endif
```

**Platforms Affected**:
- x86_64 (Linux, macOS, Windows)
- ARM64 (macOS M1/M2, Linux ARM)
- Other architectures (RISC-V, etc.)

**Testing Strategy**:
- Unit tests for both code paths
- Cross-compilation tests for ARM64
- Performance benchmarks comparing optimized vs fallback

### 1.2 POSIX Threading (`runtime/concurrency.c`, `runtime/concurrency.h`)

**Platform**: Unix-like systems (Linux, macOS, BSD)

**Dependencies**:
- pthread_create
- pthread_join
- pthread_mutex_*
- pthread_cond_*

**Code Paths**:
- forth_spawn() - OS thread creation
- forth_join() - Thread synchronization
- forth_channel_create/send/recv - Message passing

**Testing Strategy**:
- Unit tests for thread creation and joining
- Stress tests for concurrent operations
- Channel communication tests
- Windows compatibility via pthreads-win32 (future)

### 1.3 Build System Platform Differences (`build.rs`)

**Location**: Lines 23-30

**Code Paths**:
```rust
.flag_if_supported("-pthread")        // Unix only
.flag_if_supported("-O3")             // Optimization level varies
.flag_if_supported("-march=native")   // Architecture-specific
.flag_if_supported("-std=c11")        // C standard support
```

**Platform Variations**:
- Linux: All flags supported
- macOS: All flags supported (Clang)
- Windows MSVC: Different flag syntax (/std:c11, /O2, etc.)
- Windows MinGW: GCC-style flags supported

**Testing Strategy**:
- CI matrix with all three platforms
- Verify compilation succeeds on each
- Check that optimizations are actually applied

### 1.4 Standalone Binary Entry Point (`runtime/bootstrap.c`)

**Location**: Lines 307-340

**Code Paths**:
```c
#ifdef FORTH_STANDALONE
int main() { ... }
#endif
```

**Testing Strategy**:
- Build with and without FORTH_STANDALONE
- Test executable behavior when standalone

## 2. Rust Feature Flags

### 2.1 Backend Selection Features

**Features**:
1. `cranelift` - Cranelift JIT backend (fast compilation, 70-85% of C performance)
2. `llvm` - LLVM backend (slow compilation, 85-110% of C performance)

**Code Locations**:
- `/Users/joshkornreich/Documents/Projects/Ollama/llama/variants/fast-forth/src/backend.rs` (lines 36-40, 107-128)
- `/Users/joshkornreich/Documents/Projects/Ollama/llama/variants/fast-forth/backend/src/lib.rs` (lines 7-17)

**Testing Strategy**:
- Build and test with `--no-default-features --features cranelift`
- Build and test with `--no-default-features --features llvm`
- Build with neither feature (should fail gracefully)
- Test backend selection logic at each optimization level

### 2.2 Server Feature

**Feature**: `server` - HTTP server for pattern database

**Dependencies**: `tokio`, `axum`

**Code Locations**:
- `/Users/joshkornreich/Documents/Projects/Ollama/llama/variants/fast-forth/src/lib.rs` (lines 46-50)
- `/Users/joshkornreich/Documents/Projects/Ollama/llama/variants/fast-forth/src/bin/fastforth-server.rs` (entire file)

**Testing Strategy**:
- Build with `--features server`
- Build without `--features server`
- Test that fastforth-server binary requires server feature
- Integration tests for server endpoints

### 2.3 Inference Feature

**Feature**: `inference` - Type inference engine

**Code Locations**:
- `/Users/joshkornreich/Documents/Projects/Ollama/llama/variants/fast-forth/src/lib.rs` (lines 46-47)
- `/Users/joshkornreich/Documents/Projects/Ollama/llama/variants/fast-forth/src/main.rs` (lines 6, 107, 367)

**Testing Strategy**:
- Build with `--features inference`
- Build with `--no-default-features` (inference is in default features)
- Test inference-specific CLI commands

### 2.4 Verbose Feature

**Feature**: `verbose` - Detailed logging via tracing-subscriber

**Code Locations**:
- `/Users/joshkornreich/Documents/Projects/Ollama/llama/variants/fast-forth/src/main.rs` (line 244)

**Testing Strategy**:
- Test log output with `--features verbose`
- Verify minimal output without feature

## 3. Platform-Specific Test Matrix

### 3.1 Required CI Matrix

```yaml
matrix:
  os: [ubuntu-latest, macos-latest, windows-latest]
  arch: [x86_64, aarch64]
  features:
    - default
    - cranelift
    - llvm
    - server
    - inference
    - verbose
  exclude:
    - os: windows-latest
      arch: aarch64  # Windows ARM not commonly tested
```

### 3.2 Test Coverage Summary

| Platform/Feature | Unit Tests | Integration Tests | Bench Tests |
|------------------|------------|-------------------|-------------|
| x86_64 optimizations | ✓ Required | ✓ Required | ✓ Required |
| ARM64 fallback | ✓ Required | Recommended | Optional |
| pthread concurrency | ✓ Required | ✓ Required | Optional |
| Cranelift backend | ✓ Required | ✓ Required | ✓ Required |
| LLVM backend | ✓ Required | ✓ Required | ✓ Required |
| Server feature | ✓ Required | ✓ Required | N/A |
| Inference feature | ✓ Required | ✓ Required | ✓ Required |
| Verbose logging | ✓ Required | N/A | N/A |
| Standalone binary | ✓ Required | Recommended | N/A |

## 4. Untestable Code Paths

### 4.1 True Cross-Platform Limitations

**Cannot test on single machine**:
1. x86_64 inline assembly when running on ARM64 (requires emulation/cross-compile)
2. pthread implementation when running on Windows (requires WSL or pthreads-win32)
3. Windows-specific code when running on Linux/macOS

**Mitigation Strategy**:
- Use GitHub Actions CI with matrix builds
- Require all PRs to pass on all platforms
- Use cross-compilation for ARM64 tests
- Use Docker containers for consistent environments

### 4.2 Feature Combination Explosion

**Total Feature Combinations**: 2^6 = 64 combinations

**Practical Testing Strategy**:
1. Test each feature individually (6 tests)
2. Test default feature set (1 test)
3. Test common production configurations:
   - `cranelift,inference` (fast development)
   - `llvm,inference` (production)
   - `server,inference,cranelift` (pattern server)
4. Total: ~12 feature combination tests

## 5. Test Implementation Plan

### Phase 1: C Runtime Tests
- [ ] Create `runtime/tests/test_platform_optimizations.c`
- [ ] Create `runtime/tests/test_concurrency_posix.c`
- [ ] Add build system test for compiler flags

### Phase 2: Rust Feature Tests
- [ ] Create `tests/platform/feature_cranelift.rs`
- [ ] Create `tests/platform/feature_llvm.rs`
- [ ] Create `tests/platform/feature_server.rs`
- [ ] Create `tests/platform/feature_inference.rs`

### Phase 3: Integration Tests
- [ ] Create `tests/integration/backend_selection.rs`
- [ ] Create `tests/integration/feature_interactions.rs`

### Phase 4: CI Configuration
- [ ] Add `.github/workflows/platform-matrix.yml`
- [ ] Configure cross-compilation for ARM64
- [ ] Add platform-specific test reporting

## 6. Code Coverage Analysis

### Current Estimated Coverage:
- **C Runtime**: ~60% (missing concurrency edge cases, platform-specific paths)
- **Rust Backend Selection**: ~75% (missing feature combination tests)
- **Feature Flags**: ~40% (mostly tested implicitly)

### Target Coverage (Post-Implementation):
- **C Runtime**: 90%+ (untestable: cross-platform inline asm)
- **Rust Backend Selection**: 95%+
- **Feature Flags**: 90%+

## 7. Testing Commands

### Manual Testing:

```bash
# Test default features
cargo test

# Test Cranelift backend only
cargo test --no-default-features --features cranelift,inference

# Test LLVM backend only
cargo test --no-default-features --features llvm,inference

# Test server feature
cargo test --features server

# Test all features
cargo test --all-features

# Test C runtime
cd runtime/tests && make test

# Cross-compile for ARM64 (macOS)
cargo build --target aarch64-apple-darwin

# Cross-compile for ARM64 (Linux)
cargo build --target aarch64-unknown-linux-gnu
```

### CI Testing Matrix:

```bash
# GitHub Actions will test:
for os in ubuntu macos windows; do
  for features in default cranelift llvm server; do
    cargo test --features $features
  done
done
```

## 8. Recommendations

1. **Immediate Actions**:
   - Add CI matrix for platform/feature combinations
   - Create unit tests for each feature flag path
   - Test C runtime concurrency primitives

2. **Medium-term**:
   - Add cross-compilation support for ARM64
   - Create integration tests for feature interactions
   - Add performance regression tests for platform-specific optimizations

3. **Long-term**:
   - Consider Windows pthread compatibility layer
   - Add ARM64-specific optimizations similar to x86_64
   - Create platform-specific benchmark suite

## 9. Risk Assessment

**High Risk** (untested, critical functionality):
- Backend selection logic (wrong backend = poor performance or compilation failure)
- Concurrency primitives (race conditions, deadlocks)
- x86_64 inline assembly (crashes on wrong usage)

**Medium Risk** (untested, non-critical):
- Feature flag combinations
- Verbose logging paths
- Standalone binary mode

**Low Risk** (well-tested elsewhere):
- Standard C fallback paths
- Basic Cranelift integration (tested by upstream)

## 10. Test Count Summary

**Total Platform-Specific Code Paths Found**: 15
- C runtime architecture-specific: 2 paths (x86_64, fallback)
- C runtime threading: 1 path (pthread)
- Build system flags: 4 variations (per platform)
- Rust feature flags: 6 features
- Feature combinations: ~12 critical combinations

**Tests Written** (after implementation): ~45
- Unit tests: ~25
- Integration tests: ~15
- Build/compilation tests: ~5

**Coverage Improvement Estimate**: +35% absolute coverage
- Current: ~55% average
- Target: ~90% average (excluding cross-platform untestable code)
