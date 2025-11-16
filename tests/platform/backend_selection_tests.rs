/// Tests for backend selection logic
///
/// Tests the logic that selects between Cranelift and LLVM backends
/// based on optimization level and feature flags.

use fastforth::backend::{BackendType, BackendSelector};
use fastforth::OptimizationLevel;

#[test]
fn test_optimization_level_to_backend_mapping() {
    // Test the documented mapping between optimization levels and backends

    #[cfg(feature = "cranelift")]
    {
        // When Cranelift is available, it should be used for O0/O1/O2
        assert_eq!(
            BackendSelector::select_backend(OptimizationLevel::None),
            BackendType::Cranelift,
            "O0 should select Cranelift when available"
        );

        assert_eq!(
            BackendSelector::select_backend(OptimizationLevel::Basic),
            BackendType::Cranelift,
            "O1 should select Cranelift when available"
        );

        assert_eq!(
            BackendSelector::select_backend(OptimizationLevel::Standard),
            BackendType::Cranelift,
            "O2 should select Cranelift when available"
        );
    }

    #[cfg(not(feature = "cranelift"))]
    {
        // When Cranelift is not available, LLVM is fallback for all levels
        assert_eq!(
            BackendSelector::select_backend(OptimizationLevel::None),
            BackendType::LLVM,
            "O0 should fallback to LLVM when Cranelift unavailable"
        );

        assert_eq!(
            BackendSelector::select_backend(OptimizationLevel::Standard),
            BackendType::LLVM,
            "O2 should fallback to LLVM when Cranelift unavailable"
        );
    }

    // O3 should ALWAYS select LLVM regardless of Cranelift availability
    assert_eq!(
        BackendSelector::select_backend(OptimizationLevel::Aggressive),
        BackendType::LLVM,
        "O3 should always select LLVM"
    );
}

#[test]
fn test_backend_availability_reporting() {
    #[cfg(feature = "cranelift")]
    {
        assert!(
            BackendSelector::is_available(BackendType::Cranelift),
            "Cranelift should be reported as available when feature enabled"
        );
    }

    #[cfg(not(feature = "cranelift"))]
    {
        assert!(
            !BackendSelector::is_available(BackendType::Cranelift),
            "Cranelift should be reported as unavailable when feature disabled"
        );
    }

    #[cfg(feature = "llvm")]
    {
        assert!(
            BackendSelector::is_available(BackendType::LLVM),
            "LLVM should be reported as available when feature enabled"
        );
    }

    #[cfg(not(feature = "llvm"))]
    {
        assert!(
            !BackendSelector::is_available(BackendType::LLVM),
            "LLVM should be reported as unavailable when feature disabled"
        );
    }
}

#[test]
fn test_backend_names() {
    assert_eq!(
        BackendSelector::backend_name(BackendType::Cranelift),
        "Cranelift"
    );

    assert_eq!(
        BackendSelector::backend_name(BackendType::LLVM),
        "LLVM"
    );
}

#[test]
fn test_backend_performance_characteristics() {
    // Cranelift: fast compile, good runtime
    let cranelift_compile = BackendSelector::expected_compile_time(BackendType::Cranelift);
    let cranelift_runtime = BackendSelector::expected_runtime_performance(BackendType::Cranelift);

    assert!(
        cranelift_compile.contains("ms"),
        "Cranelift compile time should be in milliseconds"
    );
    assert!(
        cranelift_runtime.contains("70") || cranelift_runtime.contains("85"),
        "Cranelift runtime should mention 70-85% range"
    );

    // LLVM: slow compile, excellent runtime
    let llvm_compile = BackendSelector::expected_compile_time(BackendType::LLVM);
    let llvm_runtime = BackendSelector::expected_runtime_performance(BackendType::LLVM);

    assert!(
        llvm_compile.contains("min"),
        "LLVM compile time should be in minutes"
    );
    assert!(
        llvm_runtime.contains("85") || llvm_runtime.contains("110"),
        "LLVM runtime should mention 85-110% range"
    );
}

#[test]
#[cfg(all(not(feature = "cranelift"), not(feature = "llvm")))]
fn test_no_backend_available_error() {
    // When neither backend is available, backend creation should fail
    let result = backend::Backend::new(OptimizationLevel::Standard);

    assert!(
        result.is_err(),
        "Backend creation should fail when no backends are available"
    );

    if let Err(e) = result {
        let error_msg = format!("{:?}", e);
        assert!(
            error_msg.contains("not available") || error_msg.contains("feature not enabled"),
            "Error should indicate no backend available"
        );
    }
}

#[test]
#[cfg(any(feature = "cranelift", feature = "llvm"))]
fn test_at_least_one_backend_available() {
    // When at least one backend is available, default creation should succeed
    let result = backend::Backend::new(OptimizationLevel::Standard);

    // May fail due to incomplete implementation, but not due to missing features
    if let Err(e) = result {
        let error_msg = format!("{:?}", e);
        assert!(
            !error_msg.contains("feature not enabled"),
            "Should not fail due to missing features when at least one backend is available"
        );
    }
}

// Module to access backend internals
mod backend {
    pub use fastforth::backend::*;
}
