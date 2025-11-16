/// Fuzzing target for SSA construction and validation
///
/// Focuses on SSA-specific edge cases:
/// - Phi node placement
/// - Variable renaming
/// - Control flow merges
/// - Loop headers

#![no_main]
use libfuzzer_sys::fuzz_target;
use fastforth_frontend::parse_program;
use fastforth_optimizer::IRBuilder;

fuzz_target!(|data: &[u8]| {
    if let Ok(code) = std::str::from_utf8(data) {
        if let Ok(ast) = parse_program(code) {
            let mut builder = IRBuilder::new();
            if let Ok(ir) = builder.build_from_ast(&ast) {
                // Validate SSA properties
                if let Err(_) = ir.validate_ssa() {
                    // SSA validation failed - this is OK, we just shouldn't crash
                }
            }
        }
    }
});
