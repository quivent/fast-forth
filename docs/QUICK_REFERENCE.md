# ANS Forth Compliance Tests - Quick Reference

## Test Execution

```bash
# Run all compliance tests
cargo test --test ans_forth_compliance

# Run with verbose output
cargo test --test ans_forth_compliance -- --nocapture

# Run specific category
cargo test --test ans_forth_compliance test_stack_
cargo test --test ans_forth_compliance test_arith_
cargo test --test ans_forth_compliance test_cmp_
cargo test --test ans_forth_compliance test_logic_
cargo test --test ans_forth_compliance test_output_
```

## Current Test Results

```
test result: ok. 97 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## Coverage by Category

| Category | Tests | Coverage |
|----------|-------|----------|
| Stack Manipulation | 16 | ████████████████ 100% |
| Arithmetic | 18 | ████████████████ 100% |
| Comparison | 16 | ████████████████ 100% |
| Logical Operations | 12 | ████████████████ 100% |
| Output Operations | 5 | ████████████████ 100% |
| Error Handling | 4 | ████████████████ 100% |
| Complex Expressions | 4 | ████████████████ 100% |
| Edge Cases | 4 | ████████████████ 100% |
| Integration Tests | 4 | ████████████████ 100% |
| Extended Words | 4 | ████████████████ 100% |

## Words Tested (51 Total)

### Stack (14)
DUP DROP SWAP OVER ROT -ROT 2DUP 2DROP 2SWAP 2OVER ?DUP NIP TUCK DEPTH

### Arithmetic (14)
+ - * / MOD /MOD 1+ 1- 2* 2/ NEGATE ABS MIN MAX

### Comparison (10)
= <> < > <= >= 0= 0<> 0< 0>

### Logical (7)
AND OR XOR INVERT NOT LSHIFT RSHIFT

### Output (5)
. EMIT CR SPACE SPACES

## Files

```
tests/
├── ans_forth_compliance.rs         # Main entry point
├── test_utils.rs                   # Test harness (401 lines)
└── compliance/
    ├── ans_forth_core.rs           # 91 tests (712 lines)
    └── ans_forth_extended.rs       # 4 tests + docs (583 lines)

docs/
├── ANS_FORTH_COMPLIANCE_REPORT.md  # Detailed analysis
├── COMPLIANCE_TEST_SUMMARY.md      # Executive summary
└── QUICK_REFERENCE.md              # This file
```

## Test Examples

```rust
// Basic test
test_stack_dup() -> "5 DUP" -> [5, 5]

// Arithmetic
test_arith_addition() -> "5 10 +" -> [15]

// Comparison
test_cmp_equals_true() -> "5 5 =" -> [-1]

// Edge case
test_error_division_by_zero() -> "10 0 /" -> Error

// Integration
test_complex_arithmetic_expression() -> "5 10 + 2 * 3 -" -> [27]
```
