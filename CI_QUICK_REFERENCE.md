# CI Testing Quick Reference

## Quick Stats

- **Total CI Combinations**: 135+
- **Platform-Specific Tests**: 112 tests
- **Total Test Code**: 1,713 lines
- **Platforms**: 5 (Linux, macOS, Windows)
- **Architectures**: 2 (x86_64, ARM64)
- **Rust Versions**: 3 (stable, beta, nightly)
- **Features**: 9 combinations
- **Optimization Profiles**: 4

## Run Locally

### Platform Tests
```bash
# All platform tests
cargo test --test platform_tests

# Run all tests with all features
cargo test --all-features
```

### Feature Testing
```bash
# Cranelift backend
cargo test --no-default-features --features cranelift

# LLVM backend
cargo test --no-default-features --features llvm

# Interpreter mode
cargo test --no-default-features --features interpreter

# Server mode
cargo test --features server

# All features
cargo test --all-features
```

### Optimization Profiles
```bash
# Size-optimized build
RUSTFLAGS="-C opt-level=z -C lto=fat" cargo build --release

# Speed-optimized build
RUSTFLAGS="-C opt-level=3 -C target-cpu=native" cargo build --release

# Debug build
cargo build
```

### Coverage
```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Generate coverage report
cargo tarpaulin --all-features --out Html --output-dir coverage
```

### ARM64 Testing (Docker)
```bash
# Test on ARM64 via QEMU
docker run --rm --platform linux/arm64 \
  -v $(pwd):/workspace -w /workspace \
  rust:latest \
  cargo test --verbose
```

## CI Jobs

| Job | Purpose | Time | Required |
|-----|---------|------|----------|
| test-matrix | All platform/feature combos | 20-30m | Yes |
| test-arm64 | ARM64 architecture | 30-45m | Yes |
| test-optimizations | Optimization profiles | 60-96m | Yes |
| coverage | Code coverage | 5-10m | No |
| benchmark | Performance regression | 10-20m | No |
| security-audit | Dependency vulnerabilities | 2-3m | No |
| msrv | Minimum Rust version | 3-5m | No |
| feature-compatibility | Feature flag conflicts | 10-15m | No |

## Platform-Specific Test Files

```
tests/
├── platform_tests.rs          # Entry point (180 LOC)
└── platform/
    ├── mod.rs                 # Module declarations (15 LOC)
    ├── linux_tests.rs         # Linux-specific (104 LOC)
    ├── macos_tests.rs         # macOS-specific (176 LOC)
    ├── windows_tests.rs       # Windows-specific (207 LOC)
    ├── x86_64_tests.rs        # x86_64 architecture (168 LOC)
    └── aarch64_tests.rs       # ARM64 architecture (215 LOC)
```

## Coverage Quality Gates

- ❌ **Fail**: Coverage < 50%
- ⚠️ **Warn**: Coverage < 70%
- ✅ **Pass**: Coverage ≥ 70%

## Benchmark Alerts

- **Threshold**: 105% of baseline
- **Action**: Fail CI + PR comment
- **Platform**: Ubuntu only

## Matrix Exclusions

To reduce CI time, some combinations are excluded:
- Windows + LLVM features (compatibility issues)
- Older OS + beta/nightly Rust (redundant)
- Beta/nightly + non-critical features (optimization)

**Total Excluded**: ~27 combinations
**Net Tested**: ~108 combinations

## Platform Detection in Tests

```rust
#[cfg(target_os = "linux")]
// Linux-specific code

#[cfg(target_os = "macos")]
// macOS-specific code

#[cfg(target_os = "windows")]
// Windows-specific code

#[cfg(target_arch = "x86_64")]
// x86_64-specific code

#[cfg(target_arch = "aarch64")]
// ARM64-specific code

#[cfg(feature = "cranelift")]
// Cranelift feature code

#[cfg(feature = "llvm")]
// LLVM feature code
```

## Common Commands

```bash
# Check formatting
cargo fmt -- --check

# Run clippy
cargo clippy --all-features -- -D warnings

# Run all tests
cargo test --all-features

# Run benchmarks
cargo bench

# Security audit
cargo audit

# Check all feature combinations
cargo install cargo-hack
cargo hack check --feature-powerset --depth 2
```

## CI Workflow File

`.github/workflows/test.yml`

## Documentation

See `CI_CONFIGURATION_SUMMARY.md` for complete details.
