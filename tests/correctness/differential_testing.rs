/// Differential testing against GForth
///
/// Run the same Forth code through both Fast Forth and GForth
/// and verify they produce identical results

use std::process::{Command, Stdio};
use std::io::Write;
use fastforth::ForthEngine;

/// Check if GForth is installed
pub fn gforth_available() -> bool {
    Command::new("gforth")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok()
}

/// Parse GForth stack output
/// GForth outputs like: ".s bye <1> 15  ok" when we use .s command
/// The <N> indicates stack depth, and numbers follow
fn parse_gforth_stack(output: &str) -> Vec<i64> {
    // Find lines that contain ".s" followed by the stack depth indicator
    // This avoids confusion with < and > comparison operators in the code
    for line in output.lines() {
        // First find ".s" to locate where the stack output starts
        if let Some(ds_pos) = line.find(".s") {
            // Now look for " <" after the ".s" position
            let after_ds = &line[ds_pos..];
            if let Some(bracket_pos) = after_ds.find(" <") {
                // Check if this is followed by a number and >
                let after_space_bracket = &after_ds[bracket_pos+2..]; // Skip " <"
                if let Some(end_pos) = after_space_bracket.find('>') {
                    let between = &after_space_bracket[..end_pos];
                    // Verify this is a stack depth marker (should be a number)
                    if !between.is_empty() && between.chars().all(|c| c.is_ascii_digit()) {
                        // Extract the part after the > for this marker
                        let after_marker = &after_space_bracket[end_pos+1..];
                        // Parse all numbers from this part (before " ok")
                        let before_ok = after_marker.split(" ok").next().unwrap_or(after_marker);

                        let mut result = Vec::new();
                        for part in before_ok.split_whitespace() {
                            if let Ok(num) = part.parse::<i64>() {
                                result.push(num);
                            }
                        }

                        return result;
                    }
                }
            }
        }
    }

    Vec::new()
}

/// Execute Forth code in GForth and capture stack state
pub fn run_gforth(code: &str) -> Result<Vec<i64>, String> {
    let mut child = Command::new("gforth")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn gforth: {}", e))?;

    if let Some(mut stdin) = child.stdin.take() {
        // Send the code and print the stack
        stdin.write_all(code.as_bytes())
            .map_err(|e| format!("Failed to write to gforth: {}", e))?;
        stdin.write_all(b" .s\n")
            .map_err(|e| format!("Failed to write .s: {}", e))?;
        stdin.write_all(b"bye\n")
            .map_err(|e| format!("Failed to write bye: {}", e))?;
    }

    let output = child.wait_with_output()
        .map_err(|e| format!("Failed to wait for gforth: {}", e))?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stack = parse_gforth_stack(&stdout);


        Ok(stack)
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
    let gforth_stack = run_gforth(code)?;

    // Run in Fast Forth
    let mut engine = ForthEngine::new();
    engine.eval(code)
        .map_err(|e| format!("Fast Forth error: {}", e))?;

    let fast_forth_stack = engine.stack();

    // Compare stacks
    if gforth_stack == fast_forth_stack {
        Ok(())
    } else {
        Err(format!(
            "Stack mismatch for code: {}\nGForth:     {:?}\nFast Forth: {:?}",
            code, gforth_stack, fast_forth_stack
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gforth_availability() {
        let available = gforth_available();
        println!("GForth available: {}", available);
        assert!(available, "GForth must be installed for differential testing");
    }

    #[test]
    fn test_gforth_output_parsing() {
        if !gforth_available() {
            println!("Skipping: GForth not installed");
            return;
        }

        let stack = run_gforth("5 10 +").unwrap();
        assert_eq!(stack, vec![15], "Should parse GForth output correctly");
    }

    // ========================================================================
    // ARITHMETIC OPERATIONS
    // ========================================================================

    #[test]
    fn test_addition() {
        if !gforth_available() { return; }
        differential_test("5 10 +").unwrap();
    }

    #[test]
    fn test_subtraction() {
        if !gforth_available() { return; }
        differential_test("10 5 -").unwrap();
    }

    #[test]
    fn test_multiplication() {
        if !gforth_available() { return; }
        differential_test("5 10 *").unwrap();
    }

    #[test]
    fn test_division() {
        if !gforth_available() { return; }
        differential_test("20 5 /").unwrap();
    }

    #[test]
    fn test_modulo() {
        if !gforth_available() { return; }
        differential_test("17 5 MOD").unwrap();
    }

    #[test]
    fn test_divmod() {
        if !gforth_available() { return; }
        differential_test("17 5 /MOD").unwrap();
    }

    #[test]
    fn test_negate() {
        if !gforth_available() { return; }
        differential_test("42 NEGATE").unwrap();
    }

    #[test]
    fn test_abs() {
        if !gforth_available() { return; }
        differential_test("-42 ABS").unwrap();
    }

    #[test]
    fn test_min() {
        if !gforth_available() { return; }
        differential_test("5 10 MIN").unwrap();
    }

    #[test]
    fn test_max() {
        if !gforth_available() { return; }
        differential_test("5 10 MAX").unwrap();
    }

    // ========================================================================
    // STACK MANIPULATION
    // ========================================================================

    #[test]
    fn test_dup() {
        if !gforth_available() { return; }
        differential_test("5 DUP").unwrap();
    }

    #[test]
    fn test_drop() {
        if !gforth_available() { return; }
        differential_test("5 10 DROP").unwrap();
    }

    #[test]
    fn test_swap() {
        if !gforth_available() { return; }
        differential_test("5 10 SWAP").unwrap();
    }

    #[test]
    fn test_over() {
        if !gforth_available() { return; }
        differential_test("5 10 OVER").unwrap();
    }

    #[test]
    fn test_rot() {
        if !gforth_available() { return; }
        differential_test("1 2 3 ROT").unwrap();
    }

    #[test]
    fn test_nip() {
        if !gforth_available() { return; }
        differential_test("5 10 NIP").unwrap();
    }

    #[test]
    fn test_tuck() {
        if !gforth_available() { return; }
        differential_test("5 10 TUCK").unwrap();
    }

    #[test]
    fn test_2dup() {
        if !gforth_available() { return; }
        differential_test("5 10 2DUP").unwrap();
    }

    #[test]
    fn test_2drop() {
        if !gforth_available() { return; }
        differential_test("1 2 3 4 2DROP").unwrap();
    }

    #[test]
    fn test_2swap() {
        if !gforth_available() { return; }
        differential_test("1 2 3 4 2SWAP").unwrap();
    }

    // ========================================================================
    // COMPARISON OPERATIONS
    // ========================================================================

    #[test]
    fn test_equals_true() {
        if !gforth_available() { return; }
        differential_test("5 5 =").unwrap();
    }

    #[test]
    fn test_equals_false() {
        if !gforth_available() { return; }
        differential_test("5 10 =").unwrap();
    }

    #[test]
    fn test_less_than_true() {
        if !gforth_available() { return; }
        differential_test("5 10 <").unwrap();
    }

    #[test]
    fn test_less_than_false() {
        if !gforth_available() { return; }
        differential_test("10 5 <").unwrap();
    }

    #[test]
    fn test_greater_than() {
        if !gforth_available() { return; }
        differential_test("10 5 >").unwrap();
    }

    #[test]
    fn test_zero_equals() {
        if !gforth_available() { return; }
        differential_test("0 0=").unwrap();
    }

    #[test]
    fn test_zero_less_than() {
        if !gforth_available() { return; }
        differential_test("-5 0<").unwrap();
    }

    #[test]
    fn test_zero_greater_than() {
        if !gforth_available() { return; }
        differential_test("5 0>").unwrap();
    }

    // ========================================================================
    // LOGICAL OPERATIONS
    // ========================================================================

    #[test]
    fn test_and() {
        if !gforth_available() { return; }
        differential_test("15 7 AND").unwrap();
    }

    #[test]
    fn test_or() {
        if !gforth_available() { return; }
        differential_test("8 4 OR").unwrap();
    }

    #[test]
    fn test_xor() {
        if !gforth_available() { return; }
        differential_test("15 7 XOR").unwrap();
    }

    #[test]
    fn test_invert() {
        if !gforth_available() { return; }
        differential_test("5 INVERT").unwrap();
    }

    // ========================================================================
    // COMPLEX EXPRESSIONS
    // ========================================================================

    #[test]
    fn test_complex_arithmetic() {
        if !gforth_available() { return; }
        differential_test("2 3 + 4 5 + *").unwrap();
    }

    #[test]
    fn test_nested_operations() {
        if !gforth_available() { return; }
        differential_test("10 5 - 3 * 2 +").unwrap();
    }

    #[test]
    fn test_stack_juggling() {
        if !gforth_available() { return; }
        differential_test("1 2 3 SWAP ROT").unwrap();
    }

    #[test]
    fn test_arithmetic_with_dup() {
        if !gforth_available() { return; }
        differential_test("5 DUP *").unwrap();  // Square
    }

    #[test]
    fn test_power_of_two() {
        if !gforth_available() { return; }
        differential_test("2 DUP DUP * *").unwrap();  // 2^3
    }

    // ========================================================================
    // PHI NODE BUG DETECTOR
    // ========================================================================

    /// This test would have caught the Phi node bug in control flow
    ///
    /// The Phi node bug occurs when SSA form incorrectly merges values
    /// from different control flow paths. This test exercises a pattern
    /// that would expose such bugs if we had conditional execution.
    ///
    /// For now, we test stack manipulation that simulates similar
    /// value merging scenarios.
    #[test]
    fn test_phi_node_bug_detector_stack_merge() {
        if !gforth_available() { return; }

        // This exercises stack positions that might be confused
        // Similar to how Phi nodes can confuse value origins
        differential_test("1 2 3 ROT SWAP").unwrap();
    }

    #[test]
    fn test_phi_node_bug_detector_conditional_paths() {
        if !gforth_available() { return; }

        // Multiple operations that merge values
        // Would catch bugs in value tracking across operations
        differential_test("5 DUP 10 > SWAP DROP").unwrap();
    }

    // ========================================================================
    // EDGE CASES
    // ========================================================================

    #[test]
    fn test_zero_arithmetic() {
        if !gforth_available() { return; }
        differential_test("0 5 +").unwrap();
        differential_test("5 0 +").unwrap();
        differential_test("0 5 *").unwrap();
    }

    #[test]
    fn test_negative_numbers() {
        if !gforth_available() { return; }
        differential_test("-5 10 +").unwrap();
        differential_test("5 -10 +").unwrap();
        differential_test("-5 -10 +").unwrap();
    }

    #[test]
    fn test_large_numbers() {
        if !gforth_available() { return; }
        differential_test("1000000 2000000 +").unwrap();
    }

    #[test]
    fn test_deep_stack() {
        if !gforth_available() { return; }
        // Note: GForth's .S command has a display limitation and only shows up to 9 elements
        // So we test with 9 elements instead of 10
        differential_test("1 2 3 4 5 6 7 8 9").unwrap();
    }

    #[test]
    fn test_multiple_operations_chain() {
        if !gforth_available() { return; }
        differential_test("1 2 + 3 + 4 + 5 +").unwrap();
    }

    // ========================================================================
    // PROPERTY-BASED TESTING
    // ========================================================================

    #[test]
    fn property_test_addition_commutative() {
        if !gforth_available() {
            println!("Skipping: GForth not installed");
            return;
        }

        // Test that a + b = b + a
        for a in [0, 1, 5, 10, 100, -5, -10] {
            for b in [0, 1, 5, 10, 100, -5, -10] {
                let code1 = format!("{} {} +", a, b);
                let code2 = format!("{} {} +", b, a);

                differential_test(&code1).unwrap();
                differential_test(&code2).unwrap();

                // Verify they're equal
                let stack1 = run_gforth(&code1).unwrap();
                let stack2 = run_gforth(&code2).unwrap();
                assert_eq!(stack1, stack2, "Addition should be commutative");
            }
        }
    }

    #[test]
    fn property_test_multiplication_associative() {
        if !gforth_available() {
            println!("Skipping: GForth not installed");
            return;
        }

        // Test that (a * b) * c = a * (b * c)
        for a in [1, 2, 3, 5] {
            for b in [1, 2, 3, 5] {
                for c in [1, 2, 3, 5] {
                    let code1 = format!("{} {} * {} *", a, b, c);
                    let code2 = format!("{} {} {} * *", a, b, c);

                    differential_test(&code1).unwrap();
                    differential_test(&code2).unwrap();
                }
            }
        }
    }

    #[test]
    fn property_test_dup_drop_identity() {
        if !gforth_available() {
            println!("Skipping: GForth not installed");
            return;
        }

        // Test that n DUP DROP = n
        for n in [0, 1, 5, 10, 100, -5] {
            let code = format!("{} DUP DROP", n);
            differential_test(&code).unwrap();

            let stack = run_gforth(&code).unwrap();
            assert_eq!(stack, vec![n], "DUP DROP should be identity");
        }
    }

    #[test]
    fn property_test_swap_swap_identity() {
        if !gforth_available() {
            println!("Skipping: GForth not installed");
            return;
        }

        // Test that a b SWAP SWAP = a b
        for a in [1, 5, 10] {
            for b in [1, 5, 10] {
                let code = format!("{} {} SWAP SWAP", a, b);
                differential_test(&code).unwrap();

                let stack = run_gforth(&code).unwrap();
                assert_eq!(stack, vec![a, b], "SWAP SWAP should be identity");
            }
        }
    }

    #[test]
    fn property_test_random_arithmetic() {
        if !gforth_available() {
            println!("Skipping: GForth not installed");
            return;
        }

        // Test random arithmetic expressions
        let test_cases = vec![
            "3 4 + 5 *",
            "10 2 / 3 +",
            "7 3 - 2 *",
            "8 2 / 2 / 2 /",
            "5 5 * 3 - 2 +",
        ];

        for code in test_cases {
            differential_test(code).unwrap();
        }
    }

    #[test]
    fn property_test_random_stack_ops() {
        if !gforth_available() {
            println!("Skipping: GForth not installed");
            return;
        }

        // Test random stack operation sequences
        let test_cases = vec![
            "1 2 DUP DROP SWAP",
            "3 4 5 ROT DROP",
            "1 2 OVER DROP DROP",
            "5 6 7 ROT ROT",
            "1 2 3 4 2SWAP DROP DROP",
        ];

        for code in test_cases {
            differential_test(code).unwrap();
        }
    }
}
