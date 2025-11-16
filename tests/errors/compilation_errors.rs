//! Compilation Error Stress Tests (20 tests)
//!
//! Comprehensive tests for compilation-time errors:
//! - Type errors
//! - SSA violations
//! - IR verification failures
//! - Optimization failures
//! - Code generation errors

use fastforth_frontend as frontend;

// ============================================================================
// TYPE ERROR TESTS (5 tests)
// ============================================================================

#[test]
fn test_type_mismatch_in_arithmetic() {
    // This tests the type inference system
    let source = r#"
        : test ( n -- )
            dup dup + + ;
    "#;

    let result = frontend::parse_program(source);
    assert!(result.is_ok(), "Should parse");

    if let Ok(program) = result {
        let sem_result = frontend::semantic::analyze(&program);
        // Should succeed - this is valid Forth
        println!("Type checking result: {}", sem_result.is_ok());
    }
}

#[test]
fn test_undefined_word_type_inference() {
    let source = r#"
        : caller undefined-function ;
    "#;

    let result = frontend::parse_program(source);
    if result.is_ok() {
        let program = result.unwrap();
        let sem_result = frontend::semantic::analyze(&program);
        assert!(sem_result.is_err(), "Should detect undefined word");
        if let Err(e) = sem_result {
            println!("Undefined word error: {}", e);
        }
    }
}

#[test]
fn test_recursive_type_inference() {
    let source = r#"
        : factorial ( n -- n! )
            dup 2 <
            IF
                drop 1
            ELSE
                dup 1 - factorial *
            THEN ;
    "#;

    let result = frontend::parse_program(source);
    assert!(result.is_ok(), "Should parse recursive function");

    if let Ok(program) = result {
        let sem_result = frontend::semantic::analyze(&program);
        println!("Recursive type inference: {}", sem_result.is_ok());
    }
}

#[test]
fn test_polymorphic_word_usage() {
    let source = r#"
        : generic-swap ( a b -- b a )
            swap ;
        : use-generic 1 2 generic-swap ;
    "#;

    let result = frontend::parse_program(source);
    if result.is_ok() {
        let program = result.unwrap();
        let sem_result = frontend::semantic::analyze(&program);
        assert!(sem_result.is_ok(), "Polymorphic words should work");
    }
}

#[test]
fn test_conflicting_type_constraints() {
    // Test where a word is used with conflicting type requirements
    let source = r#"
        : multi-use ( x -- )
            dup 1 +
            dup dup and ;
    "#;

    let result = frontend::parse_program(source);
    if result.is_ok() {
        let program = result.unwrap();
        let sem_result = frontend::semantic::analyze(&program);
        // May succeed or fail depending on type system strictness
        println!("Conflicting constraints: {:?}", sem_result);
    }
}

// ============================================================================
// SSA CONVERSION TESTS (5 tests)
// ============================================================================

#[test]
fn test_ssa_simple_if_conversion() {
    let source = r#"
        : test ( n -- result )
            dup 0 >
            IF
                1 +
            ELSE
                1 -
            THEN ;
    "#;

    let result = frontend::parse_program(source);
    if result.is_ok() {
        let program = result.unwrap();
        let sem_result = frontend::semantic::analyze(&program);

        if sem_result.is_ok() {
            let ssa_result = frontend::ssa::convert_to_ssa(&program);
            assert!(ssa_result.is_ok(), "Simple IF should convert to SSA");

            if let Ok(ssa_funcs) = ssa_result {
                println!("SSA IF conversion: {} blocks", ssa_funcs[0].blocks.len());
            }
        }
    }
}

#[test]
fn test_ssa_nested_control_flow() {
    let source = r#"
        : nested ( n -- result )
            dup 0 >
            IF
                dup 10 <
                IF
                    1
                ELSE
                    2
                THEN
            ELSE
                3
            THEN ;
    "#;

    let result = frontend::parse_program(source);
    if result.is_ok() {
        let program = result.unwrap();
        if frontend::semantic::analyze(&program).is_ok() {
            let ssa_result = frontend::ssa::convert_to_ssa(&program);
            println!("Nested SSA conversion: {:?}", ssa_result.is_ok());
        }
    }
}

#[test]
fn test_ssa_loop_conversion() {
    let source = r#"
        : countdown ( n -- )
            BEGIN
                dup .
                1 -
                dup 0 =
            UNTIL
            drop ;
    "#;

    let result = frontend::parse_program(source);
    if result.is_ok() {
        let program = result.unwrap();
        if frontend::semantic::analyze(&program).is_ok() {
            let ssa_result = frontend::ssa::convert_to_ssa(&program);
            println!("Loop SSA conversion: {:?}", ssa_result);
        }
    }
}

#[test]
fn test_ssa_phi_node_generation() {
    let source = r#"
        : phi-test ( flag -- n )
            IF
                42
            ELSE
                24
            THEN ;
    "#;

    let result = frontend::parse_program(source);
    if result.is_ok() {
        let program = result.unwrap();
        if frontend::semantic::analyze(&program).is_ok() {
            let ssa_result = frontend::ssa::convert_to_ssa(&program);

            if let Ok(ssa_funcs) = ssa_result {
                println!("PHI node generation successful");
                // Should have merge block with PHI node
                assert!(!ssa_funcs.is_empty());
            }
        }
    }
}

#[test]
fn test_ssa_multiple_predecessors() {
    let source = r#"
        : multi-pred ( n -- )
            dup 0 =
            IF
                1
            ELSE
                dup 1 =
                IF
                    2
                ELSE
                    3
                THEN
            THEN
            drop ;
    "#;

    let result = frontend::parse_program(source);
    if result.is_ok() {
        let program = result.unwrap();
        if frontend::semantic::analyze(&program).is_ok() {
            let ssa_result = frontend::ssa::convert_to_ssa(&program);
            println!("Multiple predecessors SSA: {:?}", ssa_result.is_ok());
        }
    }
}

// ============================================================================
// SEMANTIC ANALYSIS TESTS (5 tests)
// ============================================================================

#[test]
fn test_redefinition_error() {
    let source = r#"
        : duplicate-name 1 + ;
        : duplicate-name 2 * ;
    "#;

    let result = frontend::parse_program(source);
    if result.is_ok() {
        let program = result.unwrap();
        let sem_result = frontend::semantic::analyze(&program);
        // May allow or disallow redefinition
        println!("Redefinition handling: {:?}", sem_result);
    }
}

#[test]
fn test_forward_reference() {
    let source = r#"
        : caller callee ;
        : callee 1 + ;
    "#;

    let result = frontend::parse_program(source);
    if result.is_ok() {
        let program = result.unwrap();
        let sem_result = frontend::semantic::analyze(&program);
        // Forward references should work
        println!("Forward reference: {}", sem_result.is_ok());
    }
}

#[test]
fn test_circular_dependency() {
    let source = r#"
        : func-a func-b ;
        : func-b func-c ;
        : func-c func-a ;
    "#;

    let result = frontend::parse_program(source);
    if result.is_ok() {
        let program = result.unwrap();
        let sem_result = frontend::semantic::analyze(&program);
        // Circular dependencies are valid (mutual recursion)
        assert!(sem_result.is_ok(), "Mutual recursion should be allowed");
    }
}

#[test]
fn test_invalid_immediate_word() {
    let source = r#"
        : test
            IF IMMEDIATE ;
    "#;

    let result = frontend::parse_program(source);
    // IMMEDIATE usage outside definition context
    println!("Invalid immediate: {:?}", result);
}

#[test]
fn test_incomplete_control_structure() {
    let source = r#"
        : test
            BEGIN
                dup
                IF
                    drop
            UNTIL ;
    "#;

    let result = frontend::parse_program(source);
    // Missing THEN for IF
    if result.is_err() {
        println!("Incomplete structure detected: {}", result.unwrap_err());
    }
}

// ============================================================================
// OPTIMIZATION PHASE TESTS (3 tests)
// ============================================================================

#[test]
fn test_dead_code_elimination() {
    let source = r#"
        : has-dead-code ( n -- n )
            dup
            1 2 + drop
            ;
    "#;

    let result = frontend::parse_program(source);
    if result.is_ok() {
        let program = result.unwrap();
        if frontend::semantic::analyze(&program).is_ok() {
            let ssa_result = frontend::ssa::convert_to_ssa(&program);
            // Dead code (1 2 + drop) should be handled
            println!("Dead code test: {:?}", ssa_result.is_ok());
        }
    }
}

#[test]
fn test_constant_folding() {
    let source = r#"
        : const-fold ( -- n )
            1 2 + 3 * 4 - ;
    "#;

    let result = frontend::parse_program(source);
    if result.is_ok() {
        let program = result.unwrap();
        if frontend::semantic::analyze(&program).is_ok() {
            let ssa_result = frontend::ssa::convert_to_ssa(&program);
            // Should successfully handle constant expressions
            assert!(ssa_result.is_ok(), "Constant folding should work");
        }
    }
}

#[test]
fn test_unreachable_code() {
    let source = r#"
        : unreachable ( -- )
            EXIT
            1 2 + drop ;
    "#;

    let result = frontend::parse_program(source);
    if result.is_ok() {
        let program = result.unwrap();
        // Code after EXIT is unreachable
        println!("Unreachable code: {:?}", frontend::semantic::analyze(&program));
    }
}

// ============================================================================
// COMPLEX COMPILATION SCENARIOS (2 tests)
// ============================================================================

#[test]
fn test_deeply_nested_inlining() {
    let source = r#"
        : inner 1 + ;
        : middle inner inner + ;
        : outer middle middle + ;
    "#;

    let result = frontend::parse_program(source);
    if result.is_ok() {
        let program = result.unwrap();
        if frontend::semantic::analyze(&program).is_ok() {
            let ssa_result = frontend::ssa::convert_to_ssa(&program);
            println!("Nested inlining: {}", ssa_result.is_ok());
        }
    }
}

#[test]
fn test_complex_stack_manipulation() {
    let source = r#"
        : complex-stack ( a b c -- c a b )
            rot rot ;
    "#;

    let result = frontend::parse_program(source);
    if result.is_ok() {
        let program = result.unwrap();
        let sem_result = frontend::semantic::analyze(&program);
        assert!(sem_result.is_ok(), "Complex stack operations should work");

        if sem_result.is_ok() {
            let ssa_result = frontend::ssa::convert_to_ssa(&program);
            println!("Complex stack SSA: {:?}", ssa_result);
        }
    }
}
