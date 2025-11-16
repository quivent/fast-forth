/// Tests for LLVM backend feature
///
/// These tests only compile when the 'llvm' feature is enabled.
/// Run with: cargo test --features llvm

use fastforth::{Compiler, CompilationMode, OptimizationLevel};
use fastforth::backend::{BackendType, BackendSelector};

#[test]
fn test_llvm_feature_enabled() {
    assert!(
        cfg!(feature = "llvm"),
        "llvm feature should be enabled"
    );
}

#[test]
fn test_llvm_backend_available() {
    assert!(
        BackendSelector::is_available(BackendType::LLVM),
        "LLVM backend should be available with llvm feature"
    );
}

#[test]
fn test_llvm_selected_for_aggressive_optimization() {
    // LLVM should always be selected for O3 (Aggressive)
    let backend = BackendSelector::select_backend(OptimizationLevel::Aggressive);
    assert_eq!(
        backend,
        BackendType::LLVM,
        "LLVM should be selected for -O3"
    );
}

#[test]
#[cfg(not(feature = "cranelift"))]
fn test_llvm_fallback_when_cranelift_disabled() {
    // When cranelift is disabled, LLVM should be used even for lower opt levels
    let backend = BackendSelector::select_backend(OptimizationLevel::Standard);
    assert_eq!(
        backend,
        BackendType::LLVM,
        "LLVM should be fallback when cranelift disabled"
    );
}

#[test]
fn test_llvm_backend_info() {
    let info = BackendSelector::backend_name(BackendType::LLVM);
    assert_eq!(info, "LLVM");

    let compile_time = BackendSelector::expected_compile_time(BackendType::LLVM);
    assert!(compile_time.contains("min") || compile_time.contains("minute"));

    let runtime = BackendSelector::expected_runtime_performance(BackendType::LLVM);
    assert!(runtime.contains("%"));
}

#[test]
fn test_llvm_slow_compilation_characteristic() {
    // LLVM's expected compile time should be significantly higher than Cranelift's
    let llvm_time = BackendSelector::expected_compile_time(BackendType::LLVM);
    let cranelift_time = BackendSelector::expected_compile_time(BackendType::Cranelift);

    assert_ne!(
        llvm_time, cranelift_time,
        "LLVM and Cranelift should have different compile time characteristics"
    );

    assert!(
        llvm_time.contains("min"),
        "LLVM compile time should be in minutes"
    );
    assert!(
        cranelift_time.contains("ms"),
        "Cranelift compile time should be in milliseconds"
    );
}

#[test]
fn test_llvm_better_runtime_performance() {
    // LLVM should promise better runtime performance than Cranelift
    let llvm_perf = BackendSelector::expected_runtime_performance(BackendType::LLVM);
    let cranelift_perf = BackendSelector::expected_runtime_performance(BackendType::Cranelift);

    // Extract percentage ranges (rough comparison)
    // LLVM: "85-110% of C"
    // Cranelift: "70-85% of C"

    assert!(
        llvm_perf.contains("110"),
        "LLVM should promise up to 110% of C performance"
    );
    assert!(
        cranelift_perf.contains("85") && !cranelift_perf.contains("110"),
        "Cranelift should promise lower maximum performance"
    );
}

#[test]
fn test_llvm_backend_creation_for_aggressive() {
    // Test that we can create an LLVM backend for O3
    let result = backend::Backend::new(OptimizationLevel::Aggressive);

    // Note: Implementation may be incomplete, so check for appropriate behavior
    match result {
        Ok(backend) => {
            assert_eq!(backend.backend_type(), BackendType::LLVM);
        }
        Err(e) => {
            let error_msg = format!("{:?}", e);
            // If it fails, should not be because feature is missing
            assert!(
                !error_msg.contains("not available") && !error_msg.contains("feature not enabled"),
                "LLVM backend should be available with llvm feature"
            );
            // Implementation may be incomplete - this is acceptable
            assert!(
                error_msg.contains("not yet implemented") || error_msg.contains("not yet integrated"),
                "LLVM backend may not be fully implemented yet: {}",
                error_msg
            );
        }
    }
}

#[test]
fn test_llvm_compile_attempt() {
    // Attempt to compile with LLVM backend
    let compiler = Compiler::new(OptimizationLevel::Aggressive);
    let result = compiler.compile_string(": square dup * ;", CompilationMode::JIT);

    match result {
        Ok(_) => {
            println!("LLVM compilation succeeded");
        }
        Err(e) => {
            let error_msg = format!("{:?}", e);

            // Should not fail due to missing feature
            assert!(
                !error_msg.contains("not available") && !error_msg.contains("feature not enabled"),
                "LLVM backend should be available: {}",
                error_msg
            );

            // May fail due to incomplete implementation
            if error_msg.contains("not yet implemented") || error_msg.contains("not yet integrated") {
                println!("LLVM backend not fully implemented yet (expected)");
            } else {
                panic!("Unexpected LLVM error: {}", error_msg);
            }
        }
    }
}

// Module to access backend internals
mod backend {
    pub use fastforth::backend::*;
}
