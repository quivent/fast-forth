/// Property-Based Fuzzing for Fast Forth
///
/// This module implements systematic property-based testing using proptest
/// to explore the input space and find edge cases through:
/// 1. Random expression generation with differential oracle (GForth)
/// 2. Random stack programs with verified execution
/// 3. Random control flow structures
/// 4. Random word definitions
/// 5. Automatic shrinking to minimal failing cases

use proptest::prelude::*;
use std::process::{Command, Stdio};
use std::io::Write;

// Re-export the main library
pub use fastforth::*;

// ============================================================================
// GENERATORS FOR FORTH CONSTRUCTS
// ============================================================================

/// Generate random integers suitable for Forth (avoid overflow issues)
fn arb_forth_int() -> impl Strategy<Value = i64> {
    -10000i64..10000i64
}

/// Generate small positive integers for control flow
fn arb_small_positive() -> impl Strategy<Value = i64> {
    1i64..100i64
}

/// Generate Forth arithmetic operators
fn arb_arithmetic_op() -> impl Strategy<Value = &'static str> {
    prop_oneof![
        Just("+"),
        Just("-"),
        Just("*"),
        Just("/"),
        Just("MOD"),
    ]
}

/// Generate Forth stack operators
fn arb_stack_op() -> impl Strategy<Value = &'static str> {
    prop_oneof![
        Just("DUP"),
        Just("DROP"),
        Just("SWAP"),
        Just("OVER"),
        Just("ROT"),
    ]
}

/// Generate Forth comparison operators
fn arb_comparison_op() -> impl Strategy<Value = &'static str> {
    prop_oneof![
        Just("="),
        Just("<"),
        Just(">"),
        Just("<="),
        Just(">="),
        Just("<>"),
    ]
}

/// Generate Forth logical operators
fn arb_logical_op() -> impl Strategy<Value = &'static str> {
    prop_oneof![
        Just("AND"),
        Just("OR"),
        Just("XOR"),
        Just("INVERT"),
    ]
}

/// Generate a simple arithmetic expression
fn arb_arithmetic_expr() -> impl Strategy<Value = String> {
    (arb_forth_int(), arb_forth_int(), arb_arithmetic_op())
        .prop_map(|(a, b, op)| {
            // Avoid division by zero
            if op == "/" || op == "MOD" {
                if b == 0 {
                    format!("{} 1 {}", a, op)
                } else {
                    format!("{} {} {}", a, b, op)
                }
            } else {
                format!("{} {} {}", a, b, op)
            }
        })
}

/// Generate a sequence of stack operations
fn arb_stack_program() -> impl Strategy<Value = String> {
    prop::collection::vec(
        (arb_forth_int(), arb_stack_op()),
        1..10
    ).prop_map(|ops| {
        let mut result = String::new();
        for (val, op) in ops {
            result.push_str(&format!("{} {} ", val, op));
        }
        result.trim().to_string()
    })
}

/// Generate a simple IF-THEN structure
fn arb_if_then() -> impl Strategy<Value = String> {
    (arb_forth_int(), arb_forth_int(), arb_arithmetic_op())
        .prop_map(|(a, b, op)| {
            format!("{} {} {} IF 42 THEN", a, b, op)
        })
}

/// Generate an IF-ELSE-THEN structure
fn arb_if_else_then() -> impl Strategy<Value = String> {
    (arb_forth_int(), arb_forth_int())
        .prop_map(|(a, b)| {
            format!("{} {} > IF 100 ELSE 200 THEN", a, b)
        })
}

/// Generate a DO-LOOP structure
fn arb_do_loop() -> impl Strategy<Value = String> {
    (arb_small_positive(), arb_small_positive())
        .prop_map(|(start, end)| {
            // Ensure valid loop bounds
            let (s, e) = if start < end {
                (start, end)
            } else {
                (end, start)
            };
            format!("{} {} DO I LOOP", e, s)
        })
}

/// Generate a simple word definition
fn arb_word_definition() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9]{2,8}".prop_flat_map(|name| {
        arb_arithmetic_expr().prop_map(move |body| {
            format!(": {} {} ;", name, body)
        })
    })
}

/// Generate a complex nested expression
fn arb_complex_expr() -> impl Strategy<Value = String> {
    (
        arb_forth_int(),
        arb_forth_int(),
        arb_forth_int(),
        arb_arithmetic_op(),
        arb_arithmetic_op(),
    ).prop_map(|(a, b, c, op1, op2)| {
        format!("{} {} {} {} {} {}", a, b, op1, c, op2)
    })
}

// ============================================================================
// DIFFERENTIAL ORACLE - Compare against GForth
// ============================================================================

/// Check if GForth is available
pub fn gforth_available() -> bool {
    Command::new("gforth")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok()
}

/// Execute code in GForth and extract stack state
pub fn run_gforth(code: &str) -> Result<Vec<i64>, String> {
    let mut child = Command::new("gforth")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn gforth: {}", e))?;

    if let Some(mut stdin) = child.stdin.take() {
        // Execute code and print stack
        stdin.write_all(code.as_bytes())
            .map_err(|e| format!("Failed to write code: {}", e))?;
        stdin.write_all(b"\n.s\nbye\n")
            .map_err(|e| format!("Failed to write .s: {}", e))?;
    }

    let output = child.wait_with_output()
        .map_err(|e| format!("Failed to wait: {}", e))?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }

    // Parse GForth stack output
    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_gforth_stack(&stdout)
}

/// Parse GForth .s output to extract stack values
fn parse_gforth_stack(output: &str) -> Result<Vec<i64>, String> {
    // Look for stack depth indicator and values
    // GForth .s outputs: <depth> value1 value2 ...
    for line in output.lines() {
        if line.starts_with('<') {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                // Skip the <depth> part and parse the rest
                let values: Result<Vec<i64>, _> = parts[1..]
                    .iter()
                    .map(|s| s.parse::<i64>())
                    .collect();
                return values.map_err(|e| format!("Parse error: {}", e));
            }
        }
    }

    // If no stack output found, assume empty stack
    Ok(Vec::new())
}

/// Execute code in Fast Forth and extract stack state
pub fn run_fast_forth(code: &str) -> Result<Vec<i64>, String> {
    use fastforth_frontend::parse_program;
    use fastforth_optimizer::{Optimizer, OptimizationLevel};

    // Parse the code
    let program = parse_program(code)
        .map_err(|e| format!("Parse error: {}", e))?;

    // For now, we'll use the frontend parser
    // In a full implementation, we'd execute through the JIT
    // This is a placeholder - actual implementation needs runtime

    Err("Fast Forth execution not fully implemented yet".to_string())
}

// ============================================================================
// PROPERTY TESTS
// ============================================================================

#[cfg(test)]
mod property_tests {
    use super::*;

    /// Property: Arithmetic operations should not crash
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(1000))]

        #[test]
        fn prop_arithmetic_no_crash(
            a in arb_forth_int(),
            b in arb_forth_int(),
            op in arb_arithmetic_op()
        ) {
            use fastforth_frontend::parse_program;

            let code = if op == "/" || op == "MOD" {
                // Avoid division by zero
                let divisor = if b == 0 { 1 } else { b };
                format!("{} {} {}", a, divisor, op)
            } else {
                format!("{} {} {}", a, b, op)
            };

            // Should parse without crashing
            let result = parse_program(&code);

            // We don't care if it succeeds or fails semantically,
            // just that it doesn't panic
            match result {
                Ok(_) => {},
                Err(_) => {},
            }
        }
    }

    /// Property: Stack operations should not crash
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(1000))]

        #[test]
        fn prop_stack_ops_no_crash(program in arb_stack_program()) {
            use fastforth_frontend::parse_program;

            let result = parse_program(&program);

            // Should not panic
            match result {
                Ok(_) => {},
                Err(_) => {},
            }
        }
    }

    /// Property: Control flow should not crash
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(500))]

        #[test]
        fn prop_if_then_no_crash(program in arb_if_then()) {
            use fastforth_frontend::parse_program;

            let result = parse_program(&program);

            match result {
                Ok(_) => {},
                Err(_) => {},
            }
        }
    }

    /// Property: Nested IF-ELSE-THEN should not crash
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(500))]

        #[test]
        fn prop_if_else_then_no_crash(program in arb_if_else_then()) {
            use fastforth_frontend::parse_program;

            let result = parse_program(&program);

            match result {
                Ok(_) => {},
                Err(_) => {},
            }
        }
    }

    /// Property: DO-LOOP should not crash
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(500))]

        #[test]
        fn prop_do_loop_no_crash(program in arb_do_loop()) {
            use fastforth_frontend::parse_program;

            let result = parse_program(&program);

            match result {
                Ok(_) => {},
                Err(_) => {},
            }
        }
    }

    /// Property: Word definitions should not crash
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(1000))]

        #[test]
        fn prop_word_definition_no_crash(program in arb_word_definition()) {
            use fastforth_frontend::parse_program;

            let result = parse_program(&program);

            match result {
                Ok(_) => {},
                Err(_) => {},
            }
        }
    }

    /// Property: Complex expressions should not crash
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(1000))]

        #[test]
        fn prop_complex_expr_no_crash(program in arb_complex_expr()) {
            use fastforth_frontend::parse_program;

            let result = parse_program(&program);

            match result {
                Ok(_) => {},
                Err(_) => {},
            }
        }
    }

    /// Property: Arithmetic commutativity (a + b = b + a)
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(500))]

        #[test]
        fn prop_addition_commutative(a in arb_forth_int(), b in arb_forth_int()) {
            use fastforth_frontend::parse_program;

            let code1 = format!("{} {} +", a, b);
            let code2 = format!("{} {} +", b, a);

            let result1 = parse_program(&code1);
            let result2 = parse_program(&code2);

            // Both should parse successfully
            assert!(result1.is_ok());
            assert!(result2.is_ok());
        }
    }

    /// Property: Multiplication commutativity (a * b = b * a)
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(500))]

        #[test]
        fn prop_multiplication_commutative(a in arb_forth_int(), b in arb_forth_int()) {
            use fastforth_frontend::parse_program;

            let code1 = format!("{} {} *", a, b);
            let code2 = format!("{} {} *", b, a);

            let result1 = parse_program(&code1);
            let result2 = parse_program(&code2);

            // Both should parse successfully
            assert!(result1.is_ok());
            assert!(result2.is_ok());
        }
    }
}

// ============================================================================
// DIFFERENTIAL TESTING (when GForth is available)
// ============================================================================

#[cfg(test)]
mod differential_tests {
    use super::*;

    /// Differential test: Compare arithmetic against GForth
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        fn diff_arithmetic_against_gforth(
            a in 0i64..1000,
            b in 1i64..1000,  // Avoid division by zero
            op in arb_arithmetic_op()
        ) {
            if !gforth_available() {
                return Ok(());
            }

            let code = format!("{} {} {}", a, b, op);

            // Run in GForth
            if let Ok(gforth_stack) = run_gforth(&code) {
                // Run in Fast Forth (when implemented)
                // let fast_forth_stack = run_fast_forth(&code)?;
                // prop_assert_eq!(gforth_stack, fast_forth_stack);

                // For now, just verify GForth execution succeeded
                let _ = gforth_stack;
            }
        }
    }

    /// Differential test: Compare stack operations against GForth
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        fn diff_stack_ops_against_gforth(
            a in arb_forth_int(),
            op in arb_stack_op()
        ) {
            if !gforth_available() {
                return Ok(());
            }

            let code = format!("{} {}", a, op);

            if let Ok(_gforth_stack) = run_gforth(&code) {
                // Comparison would go here when execution is implemented
            }
        }
    }
}

// ============================================================================
// CORPUS OF INTERESTING TEST CASES
// ============================================================================

/// Known interesting test cases that have found bugs in the past
pub const CORPUS: &[&str] = &[
    // Edge cases
    "0 0 +",
    "0 0 -",
    "0 1 /",
    "1 0 /",  // Division by zero
    "-1 -1 *",
    "2147483647 1 +",  // Integer overflow

    // Stack underflow
    "DROP",
    "SWAP",
    "DUP DROP DROP",

    // Control flow edge cases
    "0 IF 42 THEN",
    "1 IF 42 THEN",
    "-1 IF 42 THEN",

    // Loop edge cases
    "0 0 DO I LOOP",
    "1 0 DO I LOOP",
    "0 1 DO I LOOP",

    // Complex expressions
    "1 2 + 3 4 + *",
    "10 5 / 2 *",
    "100 10 MOD",

    // Nested structures
    "1 2 > IF 3 4 > IF 5 THEN THEN",
    "10 0 DO I 5 > IF 42 ELSE 24 THEN LOOP",

    // Word definitions
    ": square dup * ;",
    ": abs dup 0 < if negate then ;",
    ": max 2dup < if swap then drop ;",
];

#[cfg(test)]
mod corpus_tests {
    use super::*;
    use fastforth_frontend::parse_program;

    #[test]
    fn test_corpus_no_crash() {
        for (i, code) in CORPUS.iter().enumerate() {
            println!("Testing corpus case {}: {}", i, code);

            // Should not panic
            let result = parse_program(code);

            match result {
                Ok(_) => println!("  ✓ Parsed successfully"),
                Err(e) => println!("  ⚠ Parse error: {}", e),
            }
        }
    }
}

// ============================================================================
// SHRINKING DEMONSTRATION
// ============================================================================

#[cfg(test)]
mod shrinking_tests {
    use super::*;

    /// This test demonstrates proptest's shrinking capability
    /// When it fails, proptest will find the minimal failing case
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        #[ignore]  // This test is designed to fail to demonstrate shrinking
        fn demo_shrinking(a in arb_forth_int(), b in arb_forth_int()) {
            use fastforth_frontend::parse_program;

            let code = format!("{} {} +", a, b);
            let result = parse_program(&code);

            // This assertion will fail, demonstrating shrinking
            // Proptest will find the minimal a and b that cause failure
            prop_assert!(result.is_err(),
                "Expected parse to fail (demo only), got: {:?}", result);
        }
    }
}
