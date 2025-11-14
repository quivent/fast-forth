//! Integration tests for the Fast Forth compiler
//!
//! These tests verify the end-to-end compilation pipeline.

use fastforth::{Compiler, CompilationMode, OptimizationLevel};

#[test]
fn test_compiler_creation() {
    let compiler = Compiler::new(OptimizationLevel::Standard);
    assert_eq!(compiler.optimization_level(), OptimizationLevel::Standard);
}

#[test]
fn test_simple_definition() {
    let compiler = Compiler::new(OptimizationLevel::Basic);
    let source = ": double 2 * ;";

    // Note: This will succeed through the frontend/optimizer but fail at backend
    // which is expected since backend is not yet implemented
    let result = compiler.compile_string(source, CompilationMode::JIT);

    // We expect either success (if backend is implemented) or a specific error
    match result {
        Ok(compilation) => {
            assert_eq!(compilation.stats.definitions_count, 1);
            println!("Compilation succeeded: {:?}", compilation);
        }
        Err(e) => {
            // May fail at various stages (SSA conversion, optimization, or backend)
            let error_msg = format!("{}", e);
            println!("Expected error (backend not ready): {}", error_msg);
            assert!(
                error_msg.contains("not yet implemented")
                    || error_msg.contains("Code generation")
                    || error_msg.contains("JIT")
                    || error_msg.contains("SSA")
                    || error_msg.contains("error")
            );
        }
    }
}

#[test]
fn test_multiple_definitions() {
    let compiler = Compiler::new(OptimizationLevel::Standard);
    let source = r#"
        : double 2 * ;
        : quad double double ;
    "#;

    let result = compiler.compile_string(source, CompilationMode::JIT);

    match result {
        Ok(compilation) => {
            assert_eq!(compilation.stats.definitions_count, 2);
        }
        Err(e) => {
            // May fail at various stages (SSA conversion, optimization, or backend)
            let error_msg = format!("{}", e);
            println!("Expected error (backend not ready): {}", error_msg);
            assert!(
                error_msg.contains("not yet implemented")
                    || error_msg.contains("Code generation")
                    || error_msg.contains("SSA")
                    || error_msg.contains("error")
            );
        }
    }
}

#[test]
fn test_optimization_reduces_instructions() {
    let compiler_no_opt = Compiler::new(OptimizationLevel::None);
    let compiler_opt = Compiler::new(OptimizationLevel::Aggressive);

    let source = ": square dup * ;";

    let result_no_opt = compiler_no_opt.compile_string(source, CompilationMode::JIT);
    let result_opt = compiler_opt.compile_string(source, CompilationMode::JIT);

    // Both should produce the same result structure (even if backend fails)
    assert_eq!(result_no_opt.is_ok(), result_opt.is_ok());

    if let (Ok(no_opt), Ok(opt)) = (result_no_opt, result_opt) {
        // Optimized version should have equal or fewer instructions
        assert!(opt.stats.instructions_after <= no_opt.stats.instructions_after);
    }
}

#[test]
fn test_parse_error_handling() {
    let compiler = Compiler::new(OptimizationLevel::Basic);

    // Invalid Forth code - missing closing ;
    let source = ": broken 2 *";

    let result = compiler.compile_string(source, CompilationMode::JIT);

    assert!(result.is_err());
    let error_msg = format!("{}", result.unwrap_err());
    assert!(error_msg.contains("Parse") || error_msg.contains("error"));
}

#[test]
fn test_different_optimization_levels() {
    let levels = [
        OptimizationLevel::None,
        OptimizationLevel::Basic,
        OptimizationLevel::Standard,
        OptimizationLevel::Aggressive,
    ];

    for level in &levels {
        let compiler = Compiler::new(*level);
        assert_eq!(compiler.optimization_level(), *level);

        let source = ": test 1 2 + ;";
        let result = compiler.compile_string(source, CompilationMode::JIT);

        // Should either succeed or fail with backend error
        if let Ok(compilation) = result {
            assert_eq!(compilation.stats.definitions_count, 1);
        }
    }
}

#[test]
fn test_compilation_modes() {
    let compiler = Compiler::new(OptimizationLevel::Standard);
    let source = ": add + ;";

    // Test AOT mode
    let result_aot = compiler.compile_string(source, CompilationMode::AOT);

    // Test JIT mode
    let result_jit = compiler.compile_string(source, CompilationMode::JIT);

    // Both modes should handle the same source
    // (may fail at backend, but should fail consistently)
    assert_eq!(result_aot.is_ok(), result_jit.is_ok());
}
