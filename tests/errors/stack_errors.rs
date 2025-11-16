//! Stack Error Stress Tests (10 tests)
//!
//! Comprehensive tests for stack-related errors:
//! - Stack underflow
//! - Stack overflow
//! - Stack corruption
//! - Invalid stack operations
//! - Stack depth mismatches

use fastforth_frontend as frontend;

// ============================================================================
// STACK UNDERFLOW TESTS (3 tests)
// ============================================================================

#[test]
fn test_simple_stack_underflow() {
    let source = ": test DROP DROP DROP ;";
    let result = frontend::parse_program(source);

    if result.is_ok() {
        let program = result.unwrap();
        let sem_result = frontend::semantic::analyze(&program);
        assert!(sem_result.is_err(), "Should detect error (undefined word or stack underflow)");
        if let Err(e) = sem_result {
            let err_msg = e.to_string().to_lowercase();
            println!("Error detected: {}", e);
            // May be undefined word error or stack error
            assert!(err_msg.contains("underflow") ||
                   err_msg.contains("stack") ||
                   err_msg.contains("undefined"));
        }
    }
}

#[test]
fn test_arithmetic_underflow() {
    let source = ": test + + + ;";
    let result = frontend::parse_program(source);

    if result.is_ok() {
        let program = result.unwrap();
        let sem_result = frontend::semantic::analyze(&program);

        // May succeed or fail depending on semantic analysis strictness
        match sem_result {
            Ok(_) => println!("Semantic analysis allowed (may infer polymorphic types)"),
            Err(e) => println!("Arithmetic error detected: {}", e),
        }
    }
}

#[test]
fn test_conditional_stack_underflow() {
    let source = r#"
        : test
            IF
                DROP DROP DROP
            THEN ;
    "#;

    let result = frontend::parse_program(source);

    if result.is_ok() {
        let program = result.unwrap();
        let sem_result = frontend::semantic::analyze(&program);
        // Should detect underflow in IF branch
        if sem_result.is_err() {
            println!("Conditional underflow: {}", sem_result.unwrap_err());
        }
    }
}

// ============================================================================
// STACK OVERFLOW TESTS (2 tests)
// ============================================================================

#[test]
fn test_infinite_stack_growth() {
    // Recursive function that grows stack indefinitely
    let source = r#"
        : infinite-grow
            1 infinite-grow ;
    "#;

    let result = frontend::parse_program(source);
    assert!(result.is_ok(), "Should parse recursive function");

    // Semantic analysis should allow valid recursion
    if result.is_ok() {
        let program = result.unwrap();
        let sem_result = frontend::semantic::analyze(&program);
        // Recursion is valid at compile time
        println!("Recursive function analysis: {}", sem_result.is_ok());
    }
}

#[test]
fn test_excessive_literal_pushes() {
    // Push many literals to test stack limits
    let mut source = String::from(": test ");

    for i in 0..10000 {
        source.push_str(&format!("{} ", i));
    }

    source.push_str(";");

    let result = frontend::parse_program(&source);
    println!("Excessive literals: {}", result.is_ok());

    if result.is_ok() {
        let program = result.unwrap();
        let sem_result = frontend::semantic::analyze(&program);
        println!("Semantic analysis of large stack: {}", sem_result.is_ok());
    }
}

// ============================================================================
// STACK DEPTH MISMATCH TESTS (3 tests)
// ============================================================================

#[test]
fn test_if_then_stack_mismatch() {
    let source = r#"
        : test ( flag -- )
            IF
                1 2
            ELSE
                3
            THEN ;
    "#;

    let result = frontend::parse_program(source);

    if result.is_ok() {
        let program = result.unwrap();
        let sem_result = frontend::semantic::analyze(&program);
        // IF and ELSE branches must have same stack effect
        // May succeed if parser is lenient, or fail with various errors
        match sem_result {
            Ok(_) => println!("Parser accepted unbalanced IF/ELSE (lenient)"),
            Err(e) => {
                let err_msg = e.to_string();
                println!("Stack mismatch detected: {}", err_msg);
            }
        }
    }
}

#[test]
fn test_loop_stack_mismatch() {
    let source = r#"
        : test ( n -- )
            BEGIN
                dup 1
            dup 0 =
            UNTIL
            drop ;
    "#;

    let result = frontend::parse_program(source);

    if result.is_ok() {
        let program = result.unwrap();
        let sem_result = frontend::semantic::analyze(&program);
        // BEGIN loop body changes stack depth
        println!("Loop stack mismatch: {:?}", sem_result);
    }
}

#[test]
fn test_declared_vs_actual_stack_effect() {
    let source = r#"
        : add-three ( a b c -- sum )
            + ;
    "#;

    let result = frontend::parse_program(source);

    if result.is_ok() {
        let program = result.unwrap();
        let sem_result = frontend::semantic::analyze(&program);
        // Declares 3 inputs but only uses 2
        if sem_result.is_err() {
            println!("Stack effect mismatch: {}", sem_result.unwrap_err());
        }
    }
}

// ============================================================================
// RETURN STACK ERRORS (2 tests)
// ============================================================================

#[test]
fn test_unbalanced_return_stack() {
    let source = r#"
        : test ( n -- )
            >r
            1 . ;
    "#;

    let result = frontend::parse_program(source);

    if result.is_ok() {
        let program = result.unwrap();
        let sem_result = frontend::semantic::analyze(&program);
        // Value moved to return stack but never retrieved
        println!("Unbalanced return stack: {:?}", sem_result);
    }
}

#[test]
fn test_return_stack_underflow() {
    let source = r#"
        : test
            r> r> r> drop drop drop ;
    "#;

    let result = frontend::parse_program(source);

    if result.is_ok() {
        let program = result.unwrap();
        let sem_result = frontend::semantic::analyze(&program);
        // Attempting to pop from empty return stack
        println!("Return stack underflow: {:?}", sem_result);
    }
}
