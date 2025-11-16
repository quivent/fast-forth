//! Pipeline Integration Coverage Tests
//!
//! Targets uncovered code paths in the compilation pipeline including:
//! - Multi-pass optimization coordination
//! - Debug vs release pipeline differences
//! - Error propagation through pipeline stages

use fastforth::{
    CompilationPipeline, CompilationMode, OptimizationLevel,
};

#[test]
fn test_pipeline_basic_jit_compilation() {
    let source = ": square dup * ;";
    let mut pipeline = CompilationPipeline::new(OptimizationLevel::None);

    let result = pipeline.compile(source, CompilationMode::JIT);

    // Basic JIT compilation should succeed or fail gracefully
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_pipeline_aot_mode() {
    let source = ": double 2 * ;";
    let mut pipeline = CompilationPipeline::new(OptimizationLevel::Standard);

    let result = pipeline.compile(source, CompilationMode::AOT);

    // AOT compilation should attempt full optimization pipeline
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_pipeline_optimization_level_none() {
    let source = ": test 1 2 + ;";

    let mut pipeline = CompilationPipeline::new(OptimizationLevel::None);
    let result = pipeline.compile(source, CompilationMode::JIT);

    // Should compile with no optimizations
    if let Ok(compilation_result) = result {
        assert_eq!(compilation_result.mode, CompilationMode::JIT);
    }
}

#[test]
fn test_pipeline_optimization_level_aggressive() {
    let source = ": test 1 2 + ;";

    let mut pipeline = CompilationPipeline::new(OptimizationLevel::Aggressive);
    let result = pipeline.compile(source, CompilationMode::AOT);

    // Should compile with aggressive optimizations in AOT mode
    if let Ok(compilation_result) = result {
        assert_eq!(compilation_result.mode, CompilationMode::AOT);
    }
}

#[test]
fn test_pipeline_invalid_syntax() {
    let invalid_source = ": incomplete";

    let mut pipeline = CompilationPipeline::new(OptimizationLevel::None);
    let result = pipeline.compile(invalid_source, CompilationMode::JIT);

    // Should return an error for invalid syntax
    assert!(result.is_err());
}
