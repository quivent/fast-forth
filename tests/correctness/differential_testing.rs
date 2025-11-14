/// Differential testing against GForth
///
/// Run the same Forth code through both Fast Forth and GForth
/// and verify they produce identical results

use std::process::{Command, Stdio};
use std::io::Write;
use fast_forth::ForthEngine;

/// Check if GForth is installed
pub fn gforth_available() -> bool {
    Command::new("gforth")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok()
}

/// Execute Forth code in GForth and capture output
pub fn run_gforth(code: &str) -> Result<String, String> {
    let mut child = Command::new("gforth")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn gforth: {}", e))?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(code.as_bytes())
            .map_err(|e| format!("Failed to write to gforth: {}", e))?;
        stdin.write_all(b"\nbye\n")
            .map_err(|e| format!("Failed to write bye: {}", e))?;
    }

    let output = child.wait_with_output()
        .map_err(|e| format!("Failed to wait for gforth: {}", e))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

/// Compare Fast Forth output to GForth output
pub fn differential_test(code: &str) -> Result<(), String> {
    if !gforth_available() {
        return Err("GForth not installed - skipping differential test".to_string());
    }

    // Run in GForth
    let gforth_output = run_gforth(code)?;

    // Run in Fast Forth
    let mut engine = ForthEngine::new();
    engine.eval(code)
        .map_err(|e| format!("Fast Forth error: {}", e))?;

    let fast_forth_stack = format!("{:?}", engine.stack());

    // Compare (this is simplified - real implementation would parse GForth output)
    if gforth_output.contains(&fast_forth_stack) {
        Ok(())
    } else {
        Err(format!(
            "Output mismatch:\nGForth: {}\nFast Forth: {}",
            gforth_output, fast_forth_stack
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gforth_availability() {
        // Just check if gforth is available
        let available = gforth_available();
        println!("GForth available: {}", available);
    }

    #[test]
    fn test_simple_arithmetic() {
        if !gforth_available() {
            println!("Skipping: GForth not installed");
            return;
        }

        // TODO: Implement proper differential testing
        // differential_test("5 10 +").unwrap();
    }

    #[test]
    fn test_stack_operations() {
        if !gforth_available() {
            println!("Skipping: GForth not installed");
            return;
        }

        // TODO: Test stack operations
        // differential_test("5 10 SWAP").unwrap();
    }

    #[test]
    fn test_complex_expression() {
        if !gforth_available() {
            println!("Skipping: GForth not installed");
            return;
        }

        // TODO: Test complex expressions
        // differential_test("2 3 + 4 5 + *").unwrap();
    }

    // Property-based testing: generate random Forth programs
    // and verify Fast Forth matches GForth
    #[test]
    fn property_test_arithmetic() {
        use proptest::prelude::*;

        if !gforth_available() {
            println!("Skipping: GForth not installed");
            return;
        }

        // TODO: Implement property-based testing
        // proptest!(|(a in 0i64..1000, b in 0i64..1000)| {
        //     let code = format!("{} {} +", a, b);
        //     differential_test(&code).unwrap();
        // });
    }
}
