/// Fuzzing target for optimization passes
///
/// Tests individual optimization passes for:
/// - Constant folding
/// - Dead code elimination
/// - Inlining
/// - Type specialization
/// - Stack caching

#![no_main]
use libfuzzer_sys::fuzz_target;
use fastforth_frontend::parse_program;
use fastforth_optimizer::{IRBuilder, optimize_ir, OptimizationLevel};

fuzz_target!(|data: &[u8]| {
    if let Ok(code) = std::str::from_utf8(data) {
        if let Ok(ast) = parse_program(code) {
            let mut builder = IRBuilder::new();
            if let Ok(ir) = builder.build_from_ast(&ast) {
                // Try different optimization levels
                for level in [OptimizationLevel::None, OptimizationLevel::Basic, OptimizationLevel::Aggressive] {
                    let ir_clone = ir.clone();
                    let _ = optimize_ir(ir_clone).with_level(level);
                }
            }
        }
    }
});
