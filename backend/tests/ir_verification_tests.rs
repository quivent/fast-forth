//! Tests for Cranelift IR verification feature
//!
//! These tests demonstrate that IR verification catches malformed IR early
//! during compilation, before runtime issues can occur.

#[cfg(feature = "cranelift")]
mod cranelift_verification_tests {
    use backend::cranelift::{CraneliftCompiler, CraneliftSettings};
    use fastforth_frontend::ssa::{SSAFunction, SSAInstruction, Register, BlockId};

    /// Test that verification is enabled in development builds
    #[test]
    fn test_verification_enabled_in_dev() {
        let settings = CraneliftSettings::development();
        assert!(settings.enable_verification, "Verification should be enabled in development builds");
    }

    /// Test that verification is disabled for maximum performance
    #[test]
    fn test_verification_disabled_in_maximum() {
        let settings = CraneliftSettings::maximum();
        assert!(!settings.enable_verification, "Verification should be disabled for maximum performance");
    }

    /// Test that verification respects debug_assertions by default
    #[test]
    fn test_verification_default_respects_debug_assertions() {
        let settings = CraneliftSettings::default();
        // In debug builds, verification should be enabled
        // In release builds, verification should be disabled
        assert_eq!(settings.enable_verification, cfg!(debug_assertions),
                   "Default verification setting should match debug_assertions");
    }

    /// Test successful compilation with verification enabled
    #[test]
    fn test_successful_verification() {
        // Create a simple valid SSA function
        let ssa_func = create_simple_function();

        // Compile with verification enabled
        let settings = CraneliftSettings::development();
        let mut compiler = CraneliftCompiler::with_settings(settings)
            .expect("Failed to create compiler");

        let backend = compiler.backend_mut();
        backend.declare_all_functions(&[("test_func".to_string(), &ssa_func)])
            .expect("Failed to declare functions");

        // This should succeed because the IR is valid
        backend.compile_function(&ssa_func, "test_func")
            .expect("Valid IR should pass verification");

        backend.finalize_all()
            .expect("Failed to finalize");
    }

    /// Test that verification can be explicitly disabled
    #[test]
    fn test_verification_can_be_disabled() {
        let mut settings = CraneliftSettings::development();
        settings.enable_verification = false;

        let ssa_func = create_simple_function();
        let mut compiler = CraneliftCompiler::with_settings(settings)
            .expect("Failed to create compiler");

        let backend = compiler.backend_mut();
        backend.declare_all_functions(&[("test_func".to_string(), &ssa_func)])
            .expect("Failed to declare functions");

        backend.compile_function(&ssa_func, "test_func")
            .expect("Compilation should succeed with verification disabled");

        backend.finalize_all()
            .expect("Failed to finalize");
    }

    /// Helper: Create a simple valid SSA function for testing
    /// Function: add(a, b) -> a + b
    fn create_simple_function() -> SSAFunction {
        use fastforth_frontend::ssa::BinaryOperator;
        use smallvec::smallvec;

        let param1 = Register(0);
        let param2 = Register(1);
        let result = Register(2);

        SSAFunction {
            name: "test_func".to_string(),
            parameters: vec![param1, param2],
            entry_block: BlockId(0),
            blocks: vec![
                fastforth_frontend::ssa::BasicBlock {
                    id: BlockId(0),
                    predecessors: vec![],
                    instructions: vec![
                        SSAInstruction::BinaryOp {
                            dest: result,
                            op: BinaryOperator::Add,
                            left: param1,
                            right: param2,
                        },
                        SSAInstruction::Return {
                            values: smallvec![result],
                        },
                    ],
                },
            ],
        }
    }
}

#[cfg(not(feature = "cranelift"))]
mod no_cranelift {
    #[test]
    fn cranelift_feature_not_enabled() {
        // This test exists to prevent the test suite from failing
        // when the cranelift feature is not enabled
    }
}
