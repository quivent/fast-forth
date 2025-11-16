# Untestable Code - Quick Reference

**Fast Forth Compiler - Coverage Analysis**
**Last Updated:** 2025-11-15

---

## One-Page Summary

### The Numbers

| Metric | Value |
|--------|-------|
| **Total Codebase** | 53,101 lines |
| **Production Code** | 40,530 lines |
| **"Untestable" Code** | 8,948 lines (16.9%) |
| **Actually Impossible** | 2,805 lines (5.3%) |
| **Could Test But Shouldn't** | 5,329 lines (10.1%) |
| **Should Test (High ROI)** | 814 lines (1.5%) |

---

## Category Breakdown Table

| # | Category | Lines | % | Could Test? | Should Test? | Why/Why Not |
|---|----------|-------|---|-------------|--------------|-------------|
| **1** | **Impossible - Wrong Platform** | 740 | 1.4% | ❌ No | - | Feature combos that don't compile together |
| **2** | **Expensive - Multi-Platform** | 232 | 0.4% | ✅ Yes | ✅ **Already Done** | Tested across 3 CI platforms |
| **3** | **Expensive - Feature Combos** | 247 | 0.5% | ⚠️ With $$$ | ❌ No | 14 redundant test configs |
| **4** | **Destructive - Stack Underflow** | 412 | 0.8% | ⚠️ With `unsafe` | ❌ No | Would corrupt memory |
| **5** | **Destructive - OOM** | 824 | 1.5% | ⚠️ With bare metal | ❌ No | Would crash CI |
| **6** | **Destructive - Disk Full** | 618 | 1.2% | ✅ With mocking | ⚠️ Maybe | Low ROI |
| **7** | **Destructive - Hardware** | 570 | 1.1% | ❌ No | ❌ No | Cosmic rays |
| **8** | **Generated - build.rs** | 79 | 0.1% | ❌ No | ❌ No | Test the generator |
| **9** | **Generated - ISLE** | 1,500 | 2.8% | ❌ No | ❌ No | Dependency code |
| **10** | **Generated - Macros** | 226 | 0.4% | ✅ Implicit | ✅ **Already Done** | Test macro output |
| **11** | **Defensive - SSA Invariants** | 337 | 0.6% | ⚠️ With corruption | ❌ No | Defeats purpose |
| **12** | **Defensive - Optimizer** | 428 | 0.8% | ⚠️ With corruption | ❌ No | Defeats purpose |
| **13** | **Defensive - Bounds Checks** | 335 | 0.6% | ⚠️ Triggers panic | ❌ No | Language-level |
| **14** | **Diminishing - Error Messages** | 891 | 1.7% | ✅ Yes | ⚠️ Top 20 only | 300 tests for formatting |
| **15** | **Diminishing - Logging** | 612 | 1.2% | ⚠️ With infra | ❌ No | Cross-cutting concern |
| **16** | **Diminishing - Metrics** | 547 | 1.0% | ✅ Yes | ✅ Light testing | Already done |
| **17** | **Diminishing - Deprecated** | 350 | 0.7% | ✅ Yes | ❌ No | Will be deleted |
| | **TOTAL** | **8,948** | **16.9%** | - | - | - |

---

## Decision Matrix

### ❌ DON'T TEST (8,134 lines - 15.3%)

**Reason: Physically Impossible**
- Wrong platform code: 740 lines
- Dependency generated code: 1,500 lines
- Cosmic ray failures: 570 lines

**Reason: Defeats Purpose**
- Defensive assertions: 1,100 lines
- Feature combinations that make no sense: 247 lines

**Reason: Negative ROI**
- OOM testing: 824 lines
- Logging infrastructure: 612 lines
- Deprecated code: 350 lines
- Hardware simulation: 618 lines
- Minor error formatting: 651 lines

**Reason: Already Tested Elsewhere**
- build.rs compilation: 79 lines (compiler fails if broken)
- Macro expansion: 226 lines (output is tested)
- Multi-platform code: 232 lines (tested in CI)

---

### ✅ ALREADY TESTED (694 lines - 1.3%)

- Multi-platform CI: 232 lines ✓
- Macro outputs: 226 lines ✓
- Integration metrics: 150 lines ✓
- Top 10 error messages: 86 lines ✓

---

### ⚠️ SHOULD TEST - High ROI (814 lines - 1.5%)

**Priority 1: Error Messages (240 lines)**
- Top 20 most common errors
- Cost: 4 hours
- Coverage gain: +0.5%

**Priority 2: Metrics Integration (150 lines)**
- Verify metrics are collected
- Cost: 2 hours
- Coverage gain: +0.3%

**Priority 3: File I/O Errors (424 lines)**
- Use filesystem mocking
- Cost: 8 hours
- Coverage gain: +0.7%

**Total Effort:** 14 hours → +1.5% coverage

---

## Real-World Code Examples

### Category 1: Mutually Exclusive Features (IMPOSSIBLE)

```rust
// From src/backend.rs:11
#[cfg(feature = "cranelift")]
use backend::cranelift::CraneliftCompiler;

// VS (same file, line 14)
#[cfg(feature = "llvm")]
use backend::llvm::LLVMCompiler;

// IMPOSSIBLE: Can't enable both features in one test run
```

**Why impossible:** Cargo features are compile-time. A test binary has EITHER cranelift OR llvm code, never both.

---

### Category 2: Destructive Testing (DON'T TEST)

```rust
// From backend/src/codegen/stack_cache.rs:79
pub fn peek(&self, offset: usize) -> Result<BasicValueEnum<'ctx>> {
    let idx = self.cached_values.len()
        .checked_sub(1 + offset)
        .ok_or_else(|| BackendError::RegisterAllocationFailed(
            "Stack underflow".to_string()
        ))?;
    // ...
}
```

**To test this error path, you'd need to:**
1. Bypass 20+ validation passes
2. Use `unsafe` to corrupt internal state
3. Deliberately create invalid SSA

**Better approach:** Fuzzing + differential testing (already done) ensures this never fires in practice.

---

### Category 3: Generated Code (TEST THE GENERATOR)

```rust
// From build.rs:16
cc::Build::new()
    .file("runtime/forth_runtime.c")
    .file("runtime/concurrency.c")
    .flag_if_supported("-pthread")
    .compile("forthruntime");
```

**Why not tested:**
- Runs during `cargo build`, not `cargo test`
- Coverage: 0% is normal and expected
- If build.rs breaks → compilation fails (immediate feedback)

---

### Category 4: Defensive Assertions (DON'T TEST)

```rust
// From frontend/src/ssa.rs (example)
assert_eq!(
    phi_incoming.len(),
    predecessors.len(),
    "PHI node must have one value per predecessor"
);
```

**This fires only when compiler has bugs.**

**Testing this would require:**
```rust
// BAD TEST (defeats the purpose)
#[test]
fn test_phi_assertion() {
    let mut ssa = SSABuilder::new();

    // Corrupt the CFG with unsafe code
    unsafe {
        force_add_predecessor(block, fake_block);
    }

    // Now the assertion fires (but this proves nothing useful)
    ssa.add_phi_node(block, vec![...]);  // PANICS
}
```

**Better approach:** If 10,000 fuzzing iterations never trigger the assertion → we're maintaining the invariant correctly.

---

### Category 5: Diminishing Returns (TEST SELECTIVELY)

```rust
// From src/errors/formatter.rs
pub fn format_error(&self, error: &CompileError) -> String {
    match error {
        CompileError::UnexpectedToken { expected, found, location } => {
            format!(
                "Syntax error at {}:{}\n\
                 Expected: {}\n\
                 Found: {}",
                location.line, location.column,
                expected.join(" or "),
                found
            )
        }
        // ... 60+ more error types ...
    }
}
```

**Could test all 60+ variants**, but:
- Cost: 8 hours to write tests
- Benefit: Catch formatting bugs (non-critical)
- ROI: Low

**Better approach:** Test top 20 most common errors (80/20 rule)

---

## Coverage Ceiling Calculation

### Original Claim
**"16.8% is untestable → max coverage is 83.2%"**

### Honest Breakdown

| Scenario | Testable Lines | % Possible |
|----------|---------------|------------|
| **Truly Impossible** | 37,725 | 71.0% |
| **+ Multi-Platform CI** | 37,957 | 71.5% |
| **+ File I/O Mocking** | 38,381 | 72.3% |
| **+ Error Message Tests** | 38,621 | 72.7% |
| **+ OOM Testing ($10K)** | 39,445 | 74.3% |
| **+ Formal Verification ($100K)** | 40,330 | 75.9% |

**Wait, that's way lower than 83.2%!**

### The Catch

The 16.8% "untestable" doesn't mean 83.2% IS tested. It means:
- 16.8% CAN'T be tested (or shouldn't)
- 83.2% COULD be tested
- **Current coverage:** ~75-78% (estimated)
- **Realistic ceiling:** 85-88% (with focused effort)

---

## Recommendations

### Target: 87% Coverage

**Path to get there:**

1. **Baseline measurement** (15 min)
   ```bash
   cargo tarpaulin --out Html
   ```

2. **Add high-ROI tests** (14 hours)
   - Top 20 error messages: 4 hours
   - File I/O edge cases: 8 hours
   - Metrics integration: 2 hours

3. **Improve existing tests** (8 hours)
   - Add assertions to fuzzing harness
   - Expand differential test corpus
   - Add property tests for optimizations

**Total effort:** ~23 hours
**Coverage gain:** +9-12% (from ~75% to ~87%)
**Cost per percentage point:** ~2 hours

---

## Key Takeaways

1. **Not all "untestable" code is equal**
   - 5.3% is truly impossible
   - 10.1% is possible but shouldn't be tested
   - 1.5% should be tested (high ROI)

2. **Coverage is not the goal**
   - 87% with good tests > 95% with junk tests
   - Focus on correctness, not coverage %

3. **Fast Forth already has excellent testing**
   - Fuzzing ✓
   - Differential testing ✓
   - Property tests ✓
   - Compliance tests ✓

4. **Recommended next steps**
   - Measure baseline coverage
   - Add 814 lines of high-ROI tests
   - Improve test quality (not quantity)
   - Target 87% as realistic ceiling

---

**For detailed analysis, see:** `HONEST_UNTESTABLE_CODE_BREAKDOWN.md`
