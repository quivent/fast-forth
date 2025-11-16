/// Fuzzing target for code generation
///
/// Tests the backend code generation:
/// - Cranelift IR generation
/// - Register allocation
/// - Stack frame setup
/// - Function calling conventions

#![no_main]
use libfuzzer_sys::fuzz_target;
use fastforth_frontend::parse_program;
use fastforth_optimizer::{IRBuilder, optimize_ir};

#[cfg(feature = "cranelift")]
use backend::CraneliftCodegen;

fuzz_target!(|data: &[u8]| {
    if let Ok(code) = std::str::from_utf8(data) {
        if let Ok(ast) = parse_program(code) {
            let mut builder = IRBuilder::new();
            if let Ok(ir) = builder.build_from_ast(&ast) {
                if let Ok(optimized) = optimize_ir(ir) {
                    #[cfg(feature = "cranelift")]
                    {
                        let mut codegen = CraneliftCodegen::new();
                        // Try to generate code - may fail, but shouldn't crash
                        let _ = codegen.compile(&optimized);
                    }
                }
            }
        }
    }
});
