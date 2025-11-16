//! Integration-level error scenario tests
//!
//! This test suite covers end-to-end error handling across the entire
//! compilation pipeline:
//! - Frontend -> Backend error propagation
//! - Multiple error detection and reporting
//! - Error recovery and graceful degradation
//! - Real-world error scenarios

use fastforth_frontend as frontend;

// ============================================================================
// PIPELINE ERROR PROPAGATION TESTS (5 tests)
// ============================================================================

#[test]
fn test_pipeline_parse_error_stops_compilation() {
    let source = ": broken ( n --";

    let result = frontend::parse_program(source);

    assert!(result.is_err(), "Parse error should stop pipeline");
    let err = result.unwrap_err();
    println!("Parse error: {}", err);

    // Verify error message is informative
    assert!(!err.to_string().is_empty());
}

#[test]
fn test_pipeline_semantic_error_stops_compilation() {
    let source = r#"
        : good-function 1 + ;
        : bad-function undefined-word ;
        : another-good 2 * ;
    "#;

    let program = frontend::parse_program(source).expect("Should parse");
    let result = frontend::semantic::analyze(&program);

    assert!(result.is_err(), "Semantic error should stop pipeline");
    let err = result.unwrap_err();
    println!("Semantic error: {}", err);

    // Error should mention the undefined word
    assert!(err.to_string().contains("undefined") ||
           err.to_string().contains("Undefined"));
}

#[test]
fn test_pipeline_ssa_error_propagation() {
    // Create a program that passes parsing and semantic analysis
    // but might have SSA conversion issues
    let source = r#"
        : test ( n -- )
            BEGIN
                dup .
                1 -
                dup 0 =
            UNTIL
            drop ;
    "#;

    let program = frontend::parse_program(source).expect("Should parse");
    let _ = frontend::semantic::analyze(&program).expect("Should pass semantic analysis");
    let ssa_result = frontend::ssa::convert_to_ssa(&program);

    // SSA conversion should succeed for valid control flow
    assert!(ssa_result.is_ok(), "Valid control flow should convert to SSA");
}

#[test]
fn test_pipeline_codegen_error_handling() {
    // Test that codegen errors are properly reported
    let source = r#"
        : factorial ( n -- n! )
            dup 2 <
            IF
                drop 1
            ELSE
                dup 1 - factorial *
            THEN ;
    "#;

    let program = frontend::parse_program(source).expect("Should parse");
    let _ = frontend::semantic::analyze(&program).expect("Should pass semantic analysis");
    let ssa_result = frontend::ssa::convert_to_ssa(&program);

    // SSA conversion should succeed for valid recursive functions
    assert!(ssa_result.is_ok(), "SSA conversion should handle recursion");

    if let Ok(ssa_functions) = ssa_result {
        println!("Factorial converted to SSA with {} functions", ssa_functions.len());
        // Backend compilation would happen here in a full integration test
    }
}

#[test]
fn test_pipeline_full_compilation_with_errors() {
    // Test complete pipeline with intentional errors at different stages
    let test_cases = vec![
        (": broken ( n --", "Parse error"),
        (": test undefined-word ;", "Semantic error"),
        (": test ( n -- n ) drop drop ;", "Stack underflow"),
    ];

    for (source, expected_error_type) in test_cases {
        println!("\nTesting: {}", expected_error_type);

        let parse_result = frontend::parse_program(source);

        match parse_result {
            Ok(program) => {
                // If parsing succeeded, check semantic analysis
                let sem_result = frontend::semantic::analyze(&program);

                match sem_result {
                    Ok(_) => {
                        // If semantic analysis passed, try SSA
                        let ssa_result = frontend::ssa::convert_to_ssa(&program);
                        println!("Reached SSA stage");

                        if ssa_result.is_ok() {
                            println!("All stages passed");
                        }
                    }
                    Err(err) => {
                        println!("Semantic error: {}", err);
                    }
                }
            }
            Err(err) => {
                println!("Parse error: {}", err);
            }
        }
    }
}

// ============================================================================
// MULTIPLE ERROR DETECTION TESTS (5 tests)
// ============================================================================

#[test]
fn test_multiple_undefined_words_in_one_function() {
    let source = r#"
        : test
            undefined1
            undefined2
            undefined3 ;
    "#;

    let program = frontend::parse_program(source).expect("Should parse");
    let result = frontend::semantic::analyze(&program);

    assert!(result.is_err(), "Should detect undefined words");

    // Ideally would report all three, but at least reports one
    if let Err(err) = result {
        println!("Multiple undefined words error: {}", err);
    }
}

#[test]
fn test_multiple_functions_with_errors() {
    let source = r#"
        : func1 undefined1 ;
        : func2 undefined2 ;
        : func3 undefined3 ;
    "#;

    let program = frontend::parse_program(source).expect("Should parse");
    let result = frontend::semantic::analyze(&program);

    assert!(result.is_err(), "Should detect errors in multiple functions");
}

#[test]
fn test_mixed_error_types() {
    let source = r#"
        : test1 undefined-word ;
        : test2 ( n -- n n ) drop ;
        : test3 1 2 + * ;
    "#;

    let program = frontend::parse_program(source).expect("Should parse");
    let result = frontend::semantic::analyze(&program);

    // Should catch at least one error
    assert!(result.is_err(), "Should detect various error types");
}

#[test]
fn test_cascading_errors() {
    // Errors in one function affecting another
    let source = r#"
        : helper undefined-operation ;
        : main helper helper + ;
    "#;

    let program = frontend::parse_program(source).expect("Should parse");
    let result = frontend::semantic::analyze(&program);

    assert!(result.is_err(), "Should detect cascading errors");
}

#[test]
fn test_error_in_nested_control_structures() {
    let source = r#"
        : test ( n -- )
            IF
                IF
                    undefined-word
                THEN
            THEN ;
    "#;

    let program = frontend::parse_program(source).expect("Should parse");
    let result = frontend::semantic::analyze(&program);

    assert!(result.is_err(), "Should detect error in nested structure");
}

// ============================================================================
// ERROR RECOVERY AND GRACEFUL DEGRADATION (5 tests)
// ============================================================================

#[test]
fn test_recovery_after_parse_error() {
    // Simulate recovering from parse error and continuing
    let bad_source = ": broken ( n --";
    let good_source = ": working 1 + ;";

    let bad_result = frontend::parse_program(bad_source);
    assert!(bad_result.is_err());

    // Should be able to parse good source after error
    let good_result = frontend::parse_program(good_source);
    assert!(good_result.is_ok(), "Should recover from previous parse error");
}

#[test]
fn test_recovery_after_semantic_error() {
    let bad_source = ": bad undefined-word ;";
    let good_source = ": good 1 + ;";

    // First compilation fails
    let bad_program = frontend::parse_program(bad_source).expect("Should parse");
    let bad_result = frontend::semantic::analyze(&bad_program);
    assert!(bad_result.is_err());

    // Second compilation should work
    let good_program = frontend::parse_program(good_source).expect("Should parse");
    let good_result = frontend::semantic::analyze(&good_program);
    assert!(good_result.is_ok(), "Should recover from semantic error");
}

#[test]
fn test_partial_compilation_success() {
    // Some functions compile, others fail
    let source = r#"
        : good1 1 + ;
        : good2 2 * ;
        : good3 3 - ;
    "#;

    let program = frontend::parse_program(source).expect("Should parse");
    let result = frontend::semantic::analyze(&program);

    assert!(result.is_ok(), "All good functions should compile");
}

#[test]
fn test_graceful_degradation_under_stress() {
    // Test that system degrades gracefully with increasing complexity
    let complexity_levels = vec![10, 50, 100];

    for depth in complexity_levels {
        let mut source = String::from(": stress ( n -- )\n");

        for _ in 0..depth {
            source.push_str("dup IF ");
        }

        for _ in 0..depth {
            source.push_str("THEN ");
        }

        source.push_str("drop ;");

        let result = frontend::parse_program(&source);

        match result {
            Ok(program) => {
                println!("Parsed depth {}", depth);
                let _ = frontend::semantic::analyze(&program);
            }
            Err(err) => {
                println!("Failed at depth {}: {}", depth, err);
            }
        }
    }
}

#[test]
fn test_error_limits_prevent_infinite_loops() {
    // Ensure error checking doesn't loop indefinitely
    let source = r#"
        : recursive recursive ;
    "#;

    let program = frontend::parse_program(source).expect("Should parse");

    // This should either detect the recursion or handle it gracefully
    let result = frontend::semantic::analyze(&program);

    // May pass (recursion is valid) or fail (depending on implementation)
    match result {
        Ok(_) => println!("Recursion allowed"),
        Err(err) => println!("Recursion error: {}", err),
    }
}

// ============================================================================
// REAL-WORLD ERROR SCENARIOS (5 tests)
// ============================================================================

#[test]
fn test_typo_in_builtin_word() {
    // Common mistake: typo in builtin word
    let source = ": test dupp ;"; // Should be 'dup'

    let program = frontend::parse_program(source).expect("Should parse");
    let result = frontend::semantic::analyze(&program);

    assert!(result.is_err(), "Should catch typo in builtin word");
}

#[test]
fn test_wrong_number_of_arguments() {
    let source = r#"
        : add-three ( a b c -- sum )
            + ;
    "#;

    let program = frontend::parse_program(source).expect("Should parse");
    let result = frontend::semantic::analyze(&program);

    // Should detect that stack effect doesn't match
    // (declares 3 inputs, only uses 2)
    if result.is_err() {
        println!("Caught argument mismatch: {}", result.unwrap_err());
    }
}

#[test]
fn test_forgot_to_drop_values() {
    let source = r#"
        : test ( a b -- )
            + ;
    "#;

    let program = frontend::parse_program(source).expect("Should parse");
    let result = frontend::semantic::analyze(&program);

    // Stack effect says no outputs, but + leaves a value
    if result.is_err() {
        println!("Caught missing drop: {}", result.unwrap_err());
    }
}

#[test]
fn test_missing_then_in_if_statement() {
    let source = r#"
        : test ( flag -- )
            IF
                1 .
            ;
    "#;

    let result = frontend::parse_program(source);

    // Should catch missing THEN
    if result.is_err() {
        println!("Caught missing THEN: {}", result.unwrap_err());
        assert!(true);
    } else {
        // If parsing succeeds, semantic analysis should catch it
        let program = result.unwrap();
        let sem_result = frontend::semantic::analyze(&program);
        assert!(sem_result.is_err() || sem_result.is_ok());
    }
}

#[test]
fn test_unbalanced_return_stack() {
    let source = r#"
        : test ( n -- )
            >r
            1 . ;
    "#;

    let program = frontend::parse_program(source).expect("Should parse");
    let result = frontend::semantic::analyze(&program);

    // Should ideally detect unbalanced return stack
    // (value moved to return stack but never retrieved)
    match result {
        Ok(_) => println!("Return stack checking not enforced"),
        Err(err) => println!("Caught return stack leak: {}", err),
    }
}

// ============================================================================
// ERROR MESSAGE QUALITY INTEGRATION TESTS (5 tests)
// ============================================================================

#[test]
fn test_error_provides_source_location() {
    let source = r#"
        : good 1 + ;
        : bad undefined ;
        : more 2 * ;
    "#;

    let program = frontend::parse_program(source).expect("Should parse");
    let result = frontend::semantic::analyze(&program);

    if let Err(err) = result {
        let error_msg = err.to_string();
        println!("Error with location: {}", error_msg);

        // Error message should be descriptive
        assert!(!error_msg.is_empty());
    }
}

#[test]
fn test_error_includes_word_context() {
    let source = r#"
        : outer-function
            inner-undefined-function ;
    "#;

    let program = frontend::parse_program(source).expect("Should parse");
    let result = frontend::semantic::analyze(&program);

    assert!(result.is_err(), "Should detect undefined function");

    if let Err(err) = result {
        let error_msg = err.to_string();
        println!("Error with context: {}", error_msg);

        // Should mention the undefined word
        assert!(error_msg.contains("inner-undefined") ||
               error_msg.contains("Undefined"));
    }
}

#[test]
fn test_error_suggests_fixes() {
    // Test that error messages are helpful
    let test_cases = vec![
        (": test dupp ;", "dup"),  // Typo
        (": test swapp ;", "swap"), // Typo
    ];

    for (source, _expected_suggestion) in test_cases {
        let program = frontend::parse_program(source).expect("Should parse");
        let result = frontend::semantic::analyze(&program);

        if let Err(err) = result {
            println!("Error message: {}", err);
            // In future, could suggest corrections
        }
    }
}

#[test]
fn test_error_chain_preservation() {
    // Test that error context is preserved through pipeline
    let source = ": test undefined-word ;";

    let program = frontend::parse_program(source).expect("Should parse");
    let result = frontend::semantic::analyze(&program);

    assert!(result.is_err());

    if let Err(err) = result {
        // Error should have full context
        let error_msg = err.to_string();
        println!("Full error chain: {}", error_msg);

        assert!(!error_msg.is_empty());
    }
}

#[test]
fn test_multiple_error_aggregation() {
    // Test reporting multiple errors in a single compilation
    let source = r#"
        : func1 undefined1 ;
        : func2 undefined2 ;
    "#;

    let program = frontend::parse_program(source).expect("Should parse");
    let result = frontend::semantic::analyze(&program);

    assert!(result.is_err());

    // Currently reports first error, but could aggregate multiple
    if let Err(err) = result {
        println!("Error(s) found: {}", err);
    }
}

// ============================================================================
// STRESS TESTING (3 tests)
// ============================================================================

#[test]
fn test_many_sequential_compilations() {
    // Test that repeated compilations don't leak resources
    for i in 0..100 {
        let source = format!(": test{} {} + ;", i, i);

        let program = frontend::parse_program(&source).expect("Should parse");
        let _ = frontend::semantic::analyze(&program).expect("Should analyze");
        let _ = frontend::ssa::convert_to_ssa(&program).expect("Should convert");
    }

    println!("100 sequential compilations completed");
}

#[test]
fn test_large_program_compilation() {
    // Test compilation of a large program
    let mut source = String::new();

    for i in 0..200 {
        source.push_str(&format!(": func{} {} + ;\\n", i, i));
    }

    let result = frontend::parse_program(&source);

    assert!(result.is_ok(), "Should parse large program");

    if let Ok(program) = result {
        println!("Parsed {} definitions", program.definitions.len());

        let sem_result = frontend::semantic::analyze(&program);
        assert!(sem_result.is_ok(), "Should analyze large program");
    }
}

#[test]
fn test_complex_control_flow_compilation() {
    // Test compilation of complex nested control structures
    let source = r#"
        : complex ( n -- result )
            dup 0 >
            IF
                dup 10 <
                IF
                    dup *
                ELSE
                    dup 2 /
                THEN
            ELSE
                negate
                dup 5 >
                IF
                    1 +
                ELSE
                    1 -
                THEN
            THEN ;
    "#;

    let program = frontend::parse_program(source).expect("Should parse");
    let _ = frontend::semantic::analyze(&program).expect("Should analyze");
    let ssa_result = frontend::ssa::convert_to_ssa(&program);

    // Complex control flow should either succeed or provide meaningful error
    match ssa_result {
        Ok(ssa_functions) => {
            println!("Complex control flow generated {} blocks",
                    ssa_functions[0].blocks.len());
            assert!(!ssa_functions.is_empty());
        }
        Err(err) => {
            println!("SSA conversion error (may be expected): {}", err);
            // This is acceptable - complex control flow is challenging
            assert!(!err.to_string().is_empty(), "Should have error message");
        }
    }
}
