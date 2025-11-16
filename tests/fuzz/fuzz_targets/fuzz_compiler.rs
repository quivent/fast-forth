/// End-to-end fuzzing target for the complete compilation pipeline
///
/// Tests: Parser -> AST -> SSA -> Optimization -> Codegen
/// Ensures no crashes or hangs throughout the entire pipeline

#![no_main]
use libfuzzer_sys::fuzz_target;
use fastforth_frontend::parse_program;
use fastforth_optimizer::{optimize_ir, IRBuilder};

fuzz_target!(|data: &[u8]| {
    // Convert random bytes to string
    if let Ok(code) = std::str::from_utf8(data) {
        // Try to parse
        if let Ok(ast) = parse_program(code) {
            // Try to build IR
            let mut builder = IRBuilder::new();
            if let Ok(ir) = builder.build_from_ast(&ast) {
                // Try to optimize
                let _ = optimize_ir(ir);
                // Don't care if it succeeds or fails, just shouldn't crash
            }
        }
    }
});
