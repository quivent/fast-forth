/// Fuzzing target for Forth parser
///
/// Generate random Forth code and ensure:
/// - No crashes
/// - No infinite loops
/// - Proper error handling

#![no_main]
use libfuzzer_sys::fuzz_target;
use fastforth_frontend::parse_program;

fuzz_target!(|data: &[u8]| {
    // Convert random bytes to string
    if let Ok(code) = std::str::from_utf8(data) {
        // Try to parse the code
        // We don't care if it succeeds or fails,
        // just that it doesn't crash or hang
        let _ = parse_program(code);
    }
});
