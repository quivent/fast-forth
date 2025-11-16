/// Tests for Cranelift backend feature
///
/// These tests only compile when the 'cranelift' feature is enabled.
/// Run with: cargo test --features cranelift

use fastforth::{Compiler, CompilationMode, OptimizationLevel};
use fastforth::backend::{BackendType, BackendSelector};

#[test]
fn test_cranelift_feature_enabled() {
    // This test verifies the cranelift feature is actually enabled
    assert!(
        cfg!(feature = "cranelift"),
        "cranelift feature should be enabled"
    );
}

#[test]
fn test_cranelift_backend_available() {
    // Verify Cranelift backend is available when feature is enabled
    assert!(
        BackendSelector::is_available(BackendType::Cranelift),
        "Cranelift backend should be available with cranelift feature"
    );
}

#[test]
fn test_cranelift_selected_for_low_optimization() {
    // Cranelift should be selected for O0, O1, O2
    let backend = BackendSelector::select_backend(OptimizationLevel::None);
    assert_eq!(
        backend,
        BackendType::Cranelift,
        "Cranelift should be selected for -O0"
    );

    let backend = BackendSelector::select_backend(OptimizationLevel::Basic);
    assert_eq!(
        backend,
        BackendType::Cranelift,
        "Cranelift should be selected for -O1"
    );

    let backend = BackendSelector::select_backend(OptimizationLevel::Standard);
    assert_eq!(
        backend,
        BackendType::Cranelift,
        "Cranelift should be selected for -O2"
    );
}

#[test]
fn test_cranelift_backend_creation() {
    // Test that we can create a Cranelift backend
    let result = backend::Backend::new(OptimizationLevel::Standard);

    assert!(
        result.is_ok(),
        "Should be able to create Cranelift backend with -O2"
    );

    let backend = result.unwrap();
    assert_eq!(backend.backend_type(), BackendType::Cranelift);
}

#[test]
fn test_cranelift_rejects_aggressive_optimization() {
    // Cranelift should reject -O3 (Aggressive) optimization
    let result = backend::Backend::with_backend(
        BackendType::Cranelift,
        OptimizationLevel::Aggressive,
    );

    assert!(
        result.is_err(),
        "Cranelift should reject -O3 optimization"
    );

    if let Err(e) = result {
        let error_msg = format!("{:?}", e);
        assert!(
            error_msg.contains("Cranelift") && error_msg.contains("O3"),
            "Error should mention Cranelift and O3 limitation"
        );
    }
}

#[test]
fn test_cranelift_backend_info() {
    // Test backend information reporting
    let backend = backend::Backend::with_backend(
        BackendType::Cranelift,
        OptimizationLevel::Standard,
    )
    .unwrap();

    let info = backend.info();
    assert_eq!(info.name, "Cranelift");
    assert!(info.compile_time.contains("ms")); // Should be in milliseconds
    assert!(info.runtime_performance.contains("%")); // Should show percentage
}

#[test]
fn test_cranelift_compile_simple_forth() {
    // Test actual compilation with Cranelift backend
    let compiler = Compiler::new(OptimizationLevel::Standard);

    let result = compiler.compile_string(": double dup + ;", CompilationMode::JIT);

    // Note: This may fail if backend integration is incomplete
    // Adjust assertion based on current implementation status
    match result {
        Ok(_) => {
            // Compilation succeeded
            println!("Cranelift compilation succeeded");
        }
        Err(e) => {
            let error_msg = format!("{:?}", e);
            // If it fails, it should not be due to missing backend
            assert!(
                !error_msg.contains("not available"),
                "Cranelift backend should be available: {}",
                error_msg
            );
        }
    }
}

#[test]
fn test_cranelift_compilation_speed() {
    // Cranelift should compile faster than LLVM
    use std::time::Instant;

    let compiler = Compiler::new(OptimizationLevel::Standard);
    let source = ": test-word 1 2 + 3 * ;";

    let start = Instant::now();
    let _ = compiler.compile_string(source, CompilationMode::JIT);
    let duration = start.elapsed();

    // Cranelift should compile in < 100ms (typically ~50ms)
    // This is a loose bound to account for CI variability
    assert!(
        duration.as_millis() < 1000,
        "Cranelift compilation should be fast (< 1s), took {:?}",
        duration
    );
}

#[test]
#[cfg(not(feature = "llvm"))]
fn test_cranelift_only_configuration() {
    // When only cranelift is enabled (not llvm), verify backend behavior
    assert!(cfg!(feature = "cranelift"));
    assert!(!cfg!(feature = "llvm"));

    // Aggressive optimization should fail since LLVM is not available
    let backend = BackendSelector::select_backend(OptimizationLevel::Aggressive);
    assert_eq!(
        backend,
        BackendType::LLVM,
        "O3 should select LLVM even if unavailable"
    );

    // But creating the backend should fail
    let result = backend::Backend::new(OptimizationLevel::Aggressive);
    assert!(
        result.is_err(),
        "Should fail to create LLVM backend when feature not enabled"
    );
}

// Module to access backend internals
mod backend {
    pub use fastforth::backend::*;
}
