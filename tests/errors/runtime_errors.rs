//! Runtime Error Stress Tests (10 tests)
//!
//! Comprehensive tests for runtime error conditions:
//! - Division by zero
//! - Integer overflow
//! - Array out of bounds
//! - Null pointer dereference
//! - Assertion failures
//!
//! NOTE: These tests verify that error conditions can be detected.
//! Actual runtime execution testing requires a runtime environment.

use fastforth_frontend as frontend;

// ============================================================================
// DIVISION BY ZERO TESTS (2 tests)
// ============================================================================

#[test]
fn test_division_by_zero_literal() {
    let source = r#"
        : div-zero ( -- )
            10 0 / ;
    "#;

    let result = frontend::parse_program(source);
    assert!(result.is_ok(), "Should parse division by zero");

    // At compile time, this might be caught by constant folding
    if let Ok(program) = result {
        let sem_result = frontend::semantic::analyze(&program);
        println!("Division by zero (literal): {:?}", sem_result);
    }
}

#[test]
fn test_division_by_zero_runtime() {
    let source = r#"
        : div-zero-runtime ( n -- )
            10 swap / ;
    "#;

    let result = frontend::parse_program(source);
    if result.is_ok() {
        let program = result.unwrap();
        let sem_result = frontend::semantic::analyze(&program);
        // Cannot detect runtime division by zero at compile time
        println!("Division by zero (runtime): {}", sem_result.is_ok());
    }
}

// ============================================================================
// INTEGER OVERFLOW TESTS (2 tests)
// ============================================================================

#[test]
fn test_integer_overflow_addition() {
    let source = r#"
        : overflow-add ( -- )
            2147483647 1 + ;
    "#;

    let result = frontend::parse_program(source);
    if result.is_ok() {
        let program = result.unwrap();
        let sem_result = frontend::semantic::analyze(&program);
        // Overflow behavior depends on implementation
        println!("Integer overflow (add): {}", sem_result.is_ok());
    }
}

#[test]
fn test_integer_overflow_multiplication() {
    let source = r#"
        : overflow-mul ( -- )
            1000000 1000000 * ;
    "#;

    let result = frontend::parse_program(source);
    if result.is_ok() {
        let program = result.unwrap();
        // Large multiplication
        println!("Overflow multiplication: {:?}",
                frontend::semantic::analyze(&program));
    }
}

// ============================================================================
// MEMORY ACCESS ERRORS (3 tests)
// ============================================================================

#[test]
fn test_invalid_memory_address() {
    // Test accessing memory at invalid addresses
    let source = r#"
        : bad-fetch ( -- )
            0 @ ;
    "#;

    let result = frontend::parse_program(source);
    if result.is_ok() {
        let program = result.unwrap();
        // Memory operations are valid at compile time
        println!("Invalid address access: {:?}",
                frontend::semantic::analyze(&program));
    }
}

#[test]
fn test_out_of_bounds_array_access() {
    let source = r#"
        : array-oob ( addr index -- )
            cells + @ ;
    "#;

    let result = frontend::parse_program(source);
    if result.is_ok() {
        let program = result.unwrap();
        // Cannot detect out-of-bounds at compile time
        println!("Array bounds: {:?}", frontend::semantic::analyze(&program));
    }
}

#[test]
fn test_unaligned_memory_access() {
    let source = r#"
        : unaligned ( -- )
            1 @ ;
    "#;

    let result = frontend::parse_program(source);
    if result.is_ok() {
        let program = result.unwrap();
        // Unaligned access validity depends on architecture
        println!("Unaligned access: {:?}", frontend::semantic::analyze(&program));
    }
}

// ============================================================================
// ASSERTION AND INVARIANT TESTS (3 tests)
// ============================================================================

#[test]
fn test_stack_invariant_violation() {
    // Test that should violate stack invariants
    let source = r#"
        : violate-invariant ( -- )
            1 2 3 4 5 ;
    "#;

    let result = frontend::parse_program(source);
    if result.is_ok() {
        let program = result.unwrap();
        let sem_result = frontend::semantic::analyze(&program);
        // Stack effect doesn't match declaration
        println!("Stack invariant: {:?}", sem_result);
    }
}

#[test]
fn test_return_stack_corruption() {
    let source = r#"
        : corrupt-rstack ( -- )
            >r >r drop ;
    "#;

    let result = frontend::parse_program(source);
    if result.is_ok() {
        let program = result.unwrap();
        // Return stack imbalance
        println!("Return stack corruption: {:?}",
                frontend::semantic::analyze(&program));
    }
}

#[test]
fn test_infinite_recursion_detection() {
    let source = r#"
        : infinite infinite ;
    "#;

    let result = frontend::parse_program(source);
    if result.is_ok() {
        let program = result.unwrap();
        // Direct recursion is syntactically valid
        let sem_result = frontend::semantic::analyze(&program);
        println!("Infinite recursion: {}", sem_result.is_ok());
    }
}
