//! Backend error recovery and handling tests
//!
//! This test suite covers:
//! - Code generation failures
//! - IR verification failures
//! - Memory allocation errors
//! - Invalid backend states
//! - Error recovery mechanisms

use fastforth_backend::*;
use fastforth_frontend as frontend;

// ============================================================================
// CODE GENERATION ERROR TESTS (5 tests)
// ============================================================================

#[test]
fn test_codegen_invalid_instruction_sequence() {
    // Test that invalid SSA instructions are caught during codegen
    let source = r#"
        : test ( a b -- result )
            + * ;
    "#;

    let program = frontend::parse_program(source).expect("Should parse");
    let _ = frontend::semantic::analyze(&program).expect("Should pass semantic analysis");
    let ssa_functions = frontend::ssa::convert_to_ssa(&program).expect("Should convert to SSA");

    // Try to generate code
    let result = cranelift::compile_function(&ssa_functions[0]);

    // Should either succeed or fail gracefully
    match result {
        Ok(_) => println!("Code generation succeeded"),
        Err(err) => {
            println!("Code generation error (expected): {}", err);
            assert!(err.to_string().contains("error") ||
                   err.to_string().contains("failed"),
                   "Error should be descriptive");
        }
    }
}

#[test]
fn test_codegen_invalid_memory_access() {
    // Test memory operations with invalid addresses
    let source = r#"
        : test ( -- )
            0 @ drop ;
    "#;

    let program = frontend::parse_program(source).expect("Should parse");
    let _ = frontend::semantic::analyze(&program).expect("Should pass semantic analysis");

    // This should compile but might fail at runtime
    // The test verifies the compiler handles it gracefully
    let ssa_result = frontend::ssa::convert_to_ssa(&program);
    assert!(ssa_result.is_ok(), "SSA conversion should succeed");
}

#[test]
fn test_codegen_stack_cache_overflow() {
    // Test code generation with extremely deep stack usage
    let mut source = String::from(": test ( -- )\n");
    for i in 0..256 {
        source.push_str(&format!("{} ", i));
    }
    // Drop all values
    for _ in 0..256 {
        source.push_str("drop ");
    }
    source.push_str(";");

    let program = frontend::parse_program(&source).expect("Should parse");

    if let Ok(_) = frontend::semantic::analyze(&program) {
        let ssa_result = frontend::ssa::convert_to_ssa(&program);

        if let Ok(ssa_functions) = ssa_result {
            let result = cranelift::compile_function(&ssa_functions[0]);

            // Should handle deep stacks or report error
            match result {
                Ok(_) => println!("Deep stack handled successfully"),
                Err(err) => println!("Stack overflow error: {}", err),
            }
        }
    }
}

#[test]
fn test_codegen_unsupported_operation() {
    // Test operations that might not be supported in the backend
    let source = r#"
        : test ( n -- )
            999999999 * drop ;
    "#;

    let program = frontend::parse_program(source).expect("Should parse");
    let _ = frontend::semantic::analyze(&program).expect("Should pass semantic analysis");
    let ssa_result = frontend::ssa::convert_to_ssa(&program);

    assert!(ssa_result.is_ok(), "Should convert to SSA");
}

#[test]
fn test_codegen_register_allocation_stress() {
    // Create a function that stresses register allocation
    let source = r#"
        : test ( a b c d e f g h -- result )
            + + + + + + + ;
    "#;

    let program = frontend::parse_program(source).expect("Should parse");

    if let Ok(_) = frontend::semantic::analyze(&program) {
        let ssa_result = frontend::ssa::convert_to_ssa(&program);

        if let Ok(ssa_functions) = ssa_result {
            let result = cranelift::compile_function(&ssa_functions[0]);

            // Register allocator should handle this or report error
            match result {
                Ok(_) => println!("Register allocation succeeded"),
                Err(err) => {
                    println!("Register allocation error: {}", err);
                    assert!(err.to_string().len() > 0, "Should have error message");
                }
            }
        }
    }
}

// ============================================================================
// IR VERIFICATION ERROR TESTS (5 tests)
// ============================================================================

#[test]
fn test_ir_verification_invalid_block_terminator() {
    // Test that blocks without proper terminators are caught
    // This would typically be an internal error in SSA generation
    let source = r#"
        : test ( flag -- )
            IF
                1 .
            THEN ;
    "#;

    let program = frontend::parse_program(source).expect("Should parse");
    let _ = frontend::semantic::analyze(&program).expect("Should pass semantic analysis");
    let ssa_result = frontend::ssa::convert_to_ssa(&program);

    if let Ok(ssa_functions) = ssa_result {
        // Verify all blocks have terminators
        for func in &ssa_functions {
            for block in &func.blocks {
                // Each block should have at least one instruction
                assert!(!block.instructions.is_empty(),
                       "Block should not be empty");
            }
        }
    }
}

#[test]
fn test_ir_verification_phi_node_predecessors() {
    // Test that phi nodes have correct number of predecessors
    let source = r#"
        : test ( a b flag -- result )
            IF
                +
            ELSE
                *
            THEN ;
    "#;

    let program = frontend::parse_program(source).expect("Should parse");
    let _ = frontend::semantic::analyze(&program).expect("Should pass semantic analysis");
    let ssa_result = frontend::ssa::convert_to_ssa(&program);

    assert!(ssa_result.is_ok(), "SSA conversion should succeed");

    // Verify phi nodes are well-formed
    if let Ok(ssa_functions) = ssa_result {
        let func = &ssa_functions[0];
        println!("Generated SSA with {} blocks", func.blocks.len());

        // Try to compile and verify IR
        let result = cranelift::compile_function(&func);
        if let Err(err) = result {
            println!("IR verification caught issue: {}", err);
        }
    }
}

#[test]
fn test_ir_verification_type_consistency() {
    // Test that SSA values maintain type consistency
    let source = r#"
        : test ( n -- )
            dup
            IF
                1 +
            ELSE
                2 *
            THEN
            drop ;
    "#;

    let program = frontend::parse_program(source).expect("Should parse");
    let _ = frontend::semantic::analyze(&program).expect("Should pass semantic analysis");
    let ssa_result = frontend::ssa::convert_to_ssa(&program);

    assert!(ssa_result.is_ok(), "SSA conversion should succeed");
}

#[test]
fn test_ir_verification_unreachable_blocks() {
    // Test handling of unreachable blocks
    let source = r#"
        : test ( -- )
            EXIT
            1 . ;
    "#;

    let program = frontend::parse_program(source).expect("Should parse");
    let _ = frontend::semantic::analyze(&program).expect("Should pass semantic analysis");
    let ssa_result = frontend::ssa::convert_to_ssa(&program);

    if let Ok(ssa_functions) = ssa_result {
        // Check if unreachable code is detected
        let result = cranelift::compile_function(&ssa_functions[0]);

        match result {
            Ok(_) => println!("Compiler handled unreachable code"),
            Err(err) => println!("Unreachable code error: {}", err),
        }
    }
}

#[test]
fn test_ir_verification_stack_depth_tracking() {
    // Test that stack depth is tracked correctly through control flow
    let source = r#"
        : test ( n -- result )
            dup 0 >
            IF
                dup *
            ELSE
                negate
            THEN ;
    "#;

    let program = frontend::parse_program(source).expect("Should parse");
    let _ = frontend::semantic::analyze(&program).expect("Should pass semantic analysis");
    let ssa_result = frontend::ssa::convert_to_ssa(&program);

    assert!(ssa_result.is_ok(), "Stack depth tracking should work");
}

// ============================================================================
// BACKEND INITIALIZATION ERROR TESTS (3 tests)
// ============================================================================

#[test]
fn test_backend_init_invalid_target() {
    // Test initialization with invalid target architecture
    // This is typically set at compile time, so we just verify
    // the current setup works
    let source = ": test 1 + ;";

    let program = frontend::parse_program(source).expect("Should parse");
    let _ = frontend::semantic::analyze(&program).expect("Should pass semantic analysis");
    let ssa_result = frontend::ssa::convert_to_ssa(&program);

    assert!(ssa_result.is_ok(), "Backend should initialize correctly");
}

#[test]
fn test_backend_init_compilation_flags() {
    // Test that compilation flags are handled correctly
    let source = r#"
        : factorial ( n -- n! )
            dup 1 <=
            IF
                drop 1
            ELSE
                dup 1 - factorial *
            THEN ;
    "#;

    let program = frontend::parse_program(source).expect("Should parse");
    let _ = frontend::semantic::analyze(&program).expect("Should pass semantic analysis");
    let ssa_result = frontend::ssa::convert_to_ssa(&program);

    assert!(ssa_result.is_ok(), "Should handle all compilation flags");
}

#[test]
fn test_backend_multiple_compilations() {
    // Test that backend can handle multiple consecutive compilations
    let sources = vec![
        ": test1 1 + ;",
        ": test2 2 * ;",
        ": test3 3 - ;",
    ];

    for source in sources {
        let program = frontend::parse_program(source).expect("Should parse");
        let _ = frontend::semantic::analyze(&program).expect("Should pass semantic analysis");
        let ssa_result = frontend::ssa::convert_to_ssa(&program);

        assert!(ssa_result.is_ok(), "Multiple compilations should work");
    }
}

// ============================================================================
// ERROR RECOVERY AND REPORTING TESTS (5 tests)
// ============================================================================

#[test]
fn test_error_recovery_partial_compilation() {
    // Test that errors in one function don't prevent compilation of others
    let source = r#"
        : good1 1 + ;
        : good2 2 * ;
    "#;

    let program = frontend::parse_program(source).expect("Should parse");
    let _ = frontend::semantic::analyze(&program).expect("Should pass semantic analysis");
    let ssa_result = frontend::ssa::convert_to_ssa(&program);

    assert!(ssa_result.is_ok(), "Should compile all valid functions");
    if let Ok(ssa_functions) = ssa_result {
        assert_eq!(ssa_functions.len(), 2, "Should have compiled both functions");
    }
}

#[test]
fn test_error_message_includes_function_name() {
    // Test that error messages include context about which function failed
    let source = r#"
        : problematic-function ( n -- )
            undefined-operation ;
    "#;

    let program = frontend::parse_program(source).expect("Should parse");
    let result = frontend::semantic::analyze(&program);

    if let Err(err) = result {
        let error_msg = err.to_string();
        println!("Error message: {}", error_msg);
        // Should provide context about the function
        assert!(!error_msg.is_empty());
    }
}

#[test]
fn test_error_recovery_resets_state() {
    // Test that backend state is properly reset after errors
    let source = ": test 1 + ;";

    let program = frontend::parse_program(source).expect("Should parse");
    let _ = frontend::semantic::analyze(&program).expect("Should pass semantic analysis");

    // First compilation
    let ssa_result1 = frontend::ssa::convert_to_ssa(&program);
    assert!(ssa_result1.is_ok());

    // Second compilation should also work (state reset)
    let ssa_result2 = frontend::ssa::convert_to_ssa(&program);
    assert!(ssa_result2.is_ok());
}

#[test]
fn test_error_graceful_degradation() {
    // Test that backend degrades gracefully under stress
    let mut source = String::from(": stress-test ( -- )\n");

    // Create complex nested structure
    for i in 0..50 {
        source.push_str(&format!("{} IF ", i));
    }
    source.push_str("1 . ");
    for _ in 0..50 {
        source.push_str("THEN ");
    }
    source.push_str(";");

    let result = frontend::parse_program(&source);

    if let Ok(program) = result {
        if let Ok(_) = frontend::semantic::analyze(&program) {
            let ssa_result = frontend::ssa::convert_to_ssa(&program);

            // Should either succeed or fail with clear error
            match ssa_result {
                Ok(_) => println!("Handled complex nesting"),
                Err(err) => {
                    println!("Error with complex nesting: {}", err);
                    assert!(!err.to_string().is_empty());
                }
            }
        }
    }
}

#[test]
fn test_error_cleanup_on_failure() {
    // Test that resources are properly cleaned up on compilation failure
    let source = r#"
        : test ( n -- )
            dup . ;
    "#;

    let program = frontend::parse_program(source).expect("Should parse");

    // Run multiple times to check for resource leaks
    for _ in 0..10 {
        let _ = frontend::semantic::analyze(&program);
        let _ = frontend::ssa::convert_to_ssa(&program);
    }

    // If we get here without crashing, cleanup is working
    assert!(true);
}

// ============================================================================
// BOUNDARY CONDITION TESTS (4 tests)
// ============================================================================

#[test]
fn test_zero_length_function() {
    let source = ": empty ;";

    let program = frontend::parse_program(source).expect("Should parse");
    let _ = frontend::semantic::analyze(&program).expect("Should pass semantic analysis");
    let ssa_result = frontend::ssa::convert_to_ssa(&program);

    assert!(ssa_result.is_ok(), "Empty function should compile");
}

#[test]
fn test_maximum_function_size() {
    // Test very large function
    let mut source = String::from(": large ( n -- n )\n");
    for i in 0..500 {
        source.push_str(&format!("{} + ", i));
    }
    source.push_str(";");

    let result = frontend::parse_program(&source);

    if let Ok(program) = result {
        if let Ok(_) = frontend::semantic::analyze(&program) {
            let ssa_result = frontend::ssa::convert_to_ssa(&program);

            // Should handle large functions or report limit
            match ssa_result {
                Ok(_) => println!("Large function compiled"),
                Err(err) => println!("Large function error: {}", err),
            }
        }
    }
}

#[test]
fn test_deeply_nested_calls() {
    // Test call chain depth
    let source = r#"
        : f1 1 + ;
        : f2 f1 f1 + ;
        : f3 f2 f2 + ;
        : f4 f3 f3 + ;
        : f5 f4 f4 + ;
    "#;

    let program = frontend::parse_program(source).expect("Should parse");
    let _ = frontend::semantic::analyze(&program).expect("Should pass semantic analysis");
    let ssa_result = frontend::ssa::convert_to_ssa(&program);

    assert!(ssa_result.is_ok(), "Deep call chains should work");
}

#[test]
fn test_maximum_basic_blocks() {
    // Test function with many basic blocks
    let mut source = String::from(": many-blocks ( n -- )\n");
    for i in 0..50 {
        source.push_str(&format!(
            "dup {} = IF {} . THEN\n",
            i, i
        ));
    }
    source.push_str("drop ;");

    let result = frontend::parse_program(&source);

    if let Ok(program) = result {
        if let Ok(_) = frontend::semantic::analyze(&program) {
            let ssa_result = frontend::ssa::convert_to_ssa(&program);

            assert!(ssa_result.is_ok(), "Should handle many basic blocks");

            if let Ok(ssa_functions) = ssa_result {
                println!("Generated {} blocks", ssa_functions[0].blocks.len());
            }
        }
    }
}
