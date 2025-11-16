# Code Coverage Documentation

## Overview

This document outlines the code coverage strategy, measurement tools, and improvement roadmap for the Fast-Forth JIT compiler project.

## Current Status

### Project Structure
- **Total Rust source files**: ~224 files
- **Test files**: ~35 files
- **Test file ratio**: ~15% (suggesting potential coverage gaps)

### Workspace Structure
```
fast-forth/
├── frontend/          # Forth parser and type inference
├── optimizer/         # IR optimization passes
├── backend/          # Cranelift code generation
├── benchmarks/       # Performance validation
├── tests/           # Integration and compliance tests
│   ├── compliance/  # ANS Forth compliance
│   ├── correctness/ # Differential testing
│   ├── performance/ # Performance benchmarks
│   ├── regression/  # Optimization regression tests
│   └── fuzz/        # Fuzz testing
└── src/             # Main crate with pattern system
```

## Coverage Measurement

### CI Configuration

The project has coverage measurement configured in `.github/workflows/test.yml`:

```yaml
coverage:
  name: Code Coverage
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v3
    - name: Install Rust
      uses: actions-rs/toolchain@v1
      with:
        profile: minimal
        toolchain: stable
        override: true
    - name: Install tarpaulin
      run: cargo install cargo-tarpaulin
    - name: Generate coverage
      run: cargo tarpaulin --verbose --all-features --workspace --timeout 120 --out Xml
    - name: Upload coverage to Codecov
      uses: codecov/codecov-action@v3
      with:
        files: ./cobertura.xml
        fail_ci_if_error: true
```

### Local Coverage Generation

#### Using cargo-tarpaulin (Linux/Ubuntu recommended)

```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Generate HTML and XML coverage reports
cargo tarpaulin --verbose --all-features --workspace --timeout 180 \
    --out Html --out Xml --output-dir coverage

# Open HTML report
open coverage/index.html  # macOS
xdg-open coverage/index.html  # Linux
```

#### Using cargo-llvm-cov (Cross-platform alternative)

```bash
# Install llvm-cov
cargo install cargo-llvm-cov

# Generate coverage
cargo llvm-cov --all-features --workspace --html

# Open report
open target/llvm-cov/html/index.html
```

#### Per-Module Coverage

```bash
# Coverage for specific crate
cargo tarpaulin --package fastforth-frontend --out Html

# Coverage for specific test suite
cargo tarpaulin --test integration_tests --out Html
```

### Coverage Quality Gates

#### Recommended Thresholds
- **Overall coverage**: ≥70% (baseline), target 85%
- **Critical modules**: ≥80%
  - Parser (`frontend/src/parser.rs`)
  - Codegen (`backend/src/codegen/`)
  - Type inference (`frontend/src/type_inference.rs`)
- **New code**: ≥75% (enforce via CI)

## Module-Level Coverage Analysis

### High Priority for Testing (Critical Paths)

#### 1. Frontend Module (`frontend/`)
**Purpose**: Forth parsing, lexing, and type inference

**Critical files** (should have ≥80% coverage):
- `src/parser.rs` - Main parser logic
- `src/lexer.rs` - Tokenization
- `src/type_inference.rs` - Stack effect inference
- `src/ssa.rs` - SSA IR generation

**Current test coverage**:
- Unit tests: `frontend/tests/integration_tests.rs`
- Benchmarks: `frontend/benches/parser_bench.rs`

**Coverage gaps** (likely):
- Error handling paths
- Edge cases in parser
- Type inference corner cases

#### 2. Backend Module (`backend/`)
**Purpose**: Cranelift code generation and JIT compilation

**Critical files** (should have ≥80% coverage):
- `src/codegen/mod.rs` - Main code generation
- `src/codegen/primitives.rs` - Primitive operations
- `src/codegen/control_flow.rs` - If/else/loop handling
- `src/codegen/stack_cache.rs` - Stack optimization
- `src/cranelift/mod.rs` - Cranelift integration
- `src/cranelift/translator.rs` - SSA to Cranelift IR

**Current test coverage**:
- Unit tests: `backend/tests/codegen_tests.rs`
- Stack cache tests: `backend/tests/stack_cache_tests.rs`
- Calling convention: `backend/tests/calling_convention_tests.rs`

**Coverage gaps** (likely):
- Error recovery in code generation
- Edge cases in control flow
- FFI integration paths

#### 3. Optimizer Module (`optimizer/`)
**Purpose**: IR optimization passes

**Critical files** (should have ≥75% coverage):
- `src/constant_fold.rs` - Constant folding
- `src/dead_code.rs` - Dead code elimination
- `src/inline.rs` - Function inlining
- `src/stack_cache.rs` - Stack caching optimization
- `src/type_specialization.rs` - Type-based specialization

**Current test coverage**:
- Type specialization: `optimizer/tests/type_specialization_tests.rs`
- PGO integration: `optimizer/tests/pgo_integration_tests.rs`

**Coverage gaps** (likely):
- Optimization edge cases
- Interaction between passes
- Performance-critical paths

#### 4. Main Crate (`src/`)
**Purpose**: Pattern system, semantic diff, diagnostics

**Critical files** (should have ≥70% coverage):
- `src/compiler.rs` - Main compilation pipeline
- `src/pipeline.rs` - Compilation phases
- `src/engine.rs` - Execution engine
- `src/patterns/` - Pattern matching system
- `src/errors/` - Error handling

**Coverage gaps** (likely):
- Pattern system complex scenarios
- Error formatting
- Diagnostic generation

### Lower Priority (Supporting Infrastructure)

- `src/server/` - HTTP server (optional feature)
- `src/provenance/` - Metadata tracking
- `src/diagnostics/` - Advanced diagnostics
- `benchmarks/` - Performance validation (not critical for coverage)

## Identified Coverage Gaps

### 1. Error Handling Paths
**Issue**: Error paths are often under-tested
**Impact**: High - affects reliability
**Recommendation**: Add negative test cases for:
- Invalid Forth syntax
- Type inference failures
- Code generation errors
- Runtime errors

### 2. Edge Cases in Control Flow
**Issue**: Complex control flow (nested if/else, loops) may lack coverage
**Impact**: High - affects correctness
**Recommendation**: Add tests for:
- Deeply nested conditionals
- Loop edge cases (empty, single iteration)
- Early returns and breaks

### 3. Integration Between Modules
**Issue**: Unit tests exist but integration coverage unclear
**Impact**: Medium - affects system reliability
**Recommendation**: Expand `tests/integration_tests.rs` with:
- End-to-end compilation scenarios
- Cross-module data flow
- Optimization pipeline combinations

### 4. Platform-Specific Code
**Issue**: macOS vs Linux differences not fully covered
**Impact**: Medium - affects portability
**Recommendation**: Add CI matrix testing for:
- Different OS platforms
- Different architectures (x86-64, ARM64)

### 5. FFI and External Integration
**Issue**: C runtime integration may lack coverage
**Impact**: High - affects FFI correctness
**Recommendation**: Add tests for:
- FFI boundary conditions
- C runtime calls
- Memory management across FFI

## Coverage Improvement Roadmap

### Phase 1: Baseline Measurement (Week 1)
- [x] Fix compilation issues for coverage tools
- [ ] Run initial coverage analysis with tarpaulin
- [ ] Generate baseline coverage report
- [ ] Identify modules below 50% coverage
- **Goal**: Establish baseline metrics

### Phase 2: Critical Path Coverage (Weeks 2-3)
- [ ] Achieve ≥80% coverage for parser
- [ ] Achieve ≥80% coverage for code generator
- [ ] Achieve ≥75% coverage for type inference
- [ ] Add negative test cases for error paths
- **Goal**: Cover critical compilation pipeline

### Phase 3: Integration Coverage (Week 4)
- [ ] Expand integration test suite
- [ ] Add cross-module interaction tests
- [ ] Test optimization pipeline combinations
- [ ] Validate FFI integration
- **Goal**: ≥70% overall coverage

### Phase 4: Quality Gates (Week 5)
- [ ] Add coverage badge to README
- [ ] Set up coverage quality gates in CI
- [ ] Fail CI if coverage drops below threshold
- [ ] Add per-PR coverage reports
- **Goal**: Prevent coverage regression

### Phase 5: Continuous Improvement (Ongoing)
- [ ] Track coverage trends over time
- [ ] Identify and test uncovered branches
- [ ] Add property-based tests for core algorithms
- [ ] Maintain ≥85% coverage target
- **Goal**: Maintain high coverage standards

## Adding Coverage Badge

### Codecov Badge
Once Codecov integration is working, add to README.md:

```markdown
[![codecov](https://codecov.io/gh/YOUR_USERNAME/fast-forth/branch/main/graph/badge.svg)](https://codecov.io/gh/YOUR_USERNAME/fast-forth)
```

### Coverage Visualization
Add coverage sunburst or grid:

```markdown
[![codecov](https://codecov.io/gh/YOUR_USERNAME/fast-forth/branch/main/graphs/sunburst.svg)](https://codecov.io/gh/YOUR_USERNAME/fast-forth)
```

## CI Integration Enhancements

### Enhanced Coverage Job

```yaml
coverage:
  name: Code Coverage
  runs-on: ubuntu-latest

  steps:
    - uses: actions/checkout@v3

    - name: Install Rust
      uses: actions-rs/toolchain@v1
      with:
        profile: minimal
        toolchain: stable
        override: true

    - name: Install tarpaulin
      run: cargo install cargo-tarpaulin

    - name: Generate coverage
      run: |
        cargo tarpaulin --verbose --all-features --workspace \
          --timeout 180 --out Html --out Xml --output-dir coverage

    - name: Upload coverage to Codecov
      uses: codecov/codecov-action@v3
      with:
        files: ./coverage/cobertura.xml
        fail_ci_if_error: true
        flags: unittests
        name: codecov-umbrella

    - name: Archive coverage report
      uses: actions/upload-artifact@v3
      with:
        name: coverage-report
        path: coverage/

    - name: Coverage quality gate
      run: |
        COVERAGE=$(grep -Po 'line-rate="\K[0-9.]+' coverage/cobertura.xml | head -1)
        COVERAGE_PCT=$(echo "$COVERAGE * 100" | bc)
        echo "Coverage: $COVERAGE_PCT%"
        if (( $(echo "$COVERAGE < 0.70" | bc -l) )); then
          echo "Coverage below 70% threshold!"
          exit 1
        fi
```

### Per-PR Coverage Comments

Add to `.github/workflows/test.yml`:

```yaml
- name: Comment PR with coverage
  uses: py-cov-action/python-coverage-comment-action@v3
  with:
    GITHUB_TOKEN: ${{ github.token }}
```

## Best Practices

### Writing Testable Code
1. **Small, focused functions** - Easier to test and achieve full coverage
2. **Separate pure logic from I/O** - Makes unit testing simpler
3. **Use dependency injection** - Enable mocking and isolated testing
4. **Explicit error types** - Test error paths systematically

### Test Organization
1. **Unit tests** - In same file or module as implementation
2. **Integration tests** - In `tests/` directory
3. **Benchmarks** - In `benches/` directory (separate from coverage)
4. **Examples** - In `examples/` (can contribute to coverage)

### Coverage Metrics to Track
- **Line coverage** - Percentage of lines executed
- **Branch coverage** - Percentage of conditional branches taken
- **Function coverage** - Percentage of functions called
- **Region coverage** - Percentage of code regions executed

## Troubleshooting

### Tarpaulin Compilation Errors

**Issue**: LLVM not found or compilation fails
**Solution**: Use feature flags to exclude problematic dependencies
```bash
cargo tarpaulin --workspace --no-default-features --features cranelift,inference
```

**Issue**: macOS compatibility problems
**Solution**: Use `cargo-llvm-cov` instead
```bash
cargo install cargo-llvm-cov
cargo llvm-cov --all-features --workspace --html
```

### CI Coverage Failing

**Issue**: Codecov upload fails
**Solution**: Check token configuration and file path
```yaml
- name: Upload coverage to Codecov
  uses: codecov/codecov-action@v3
  with:
    files: ./cobertura.xml
    token: ${{ secrets.CODECOV_TOKEN }}
    fail_ci_if_error: false  # Don't fail on upload errors initially
```

## Resources

- [cargo-tarpaulin documentation](https://github.com/xd009642/tarpaulin)
- [cargo-llvm-cov documentation](https://github.com/taiki-e/cargo-llvm-cov)
- [Codecov documentation](https://docs.codecov.io/)
- [Rust testing best practices](https://doc.rust-lang.org/book/ch11-00-testing.html)

## Summary

### Current State
- Coverage measurement configured in CI with tarpaulin
- Test suite includes unit, integration, compliance, and fuzz tests
- Approximately 15% of files are test files (224 source files, 35 test files)

### Immediate Actions Required
1. Fix compilation issues blocking coverage measurement
2. Generate baseline coverage report
3. Identify modules below 70% coverage
4. Add coverage badge to README
5. Set up quality gates in CI

### Long-term Goals
- Achieve ≥85% overall code coverage
- Maintain ≥80% coverage for critical modules
- Prevent coverage regression via CI checks
- Track coverage trends over time
- Integrate coverage into development workflow

---

**Last Updated**: 2025-11-15
**Coverage Target**: 70% baseline, 85% target
**CI Status**: Configured, needs testing
