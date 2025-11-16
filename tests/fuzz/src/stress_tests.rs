/// Stress tests with extreme values and edge cases
///
/// These tests push the compiler to its limits with:
/// - Maximum/minimum integer values
/// - Deep recursion
/// - Large stack depths
/// - Complex nested control flow
/// - Memory-intensive operations

use proptest::prelude::*;

/// Test with maximum integer values
#[test]
fn test_max_int_operations() {
    let test_cases = vec![
        "9223372036854775807 .",              // i64::MAX
        "-9223372036854775808 .",             // i64::MIN
        "9223372036854775807 1 + .",          // Overflow
        "-9223372036854775808 1 - .",         // Underflow
        "9223372036854775807 dup * .",        // Multiplication overflow
        "9223372036854775807 -1 * .",         // Sign change
    ];

    for code in test_cases {
        use fastforth_frontend::parse_program;
        // Should not crash, may fail gracefully
        let _ = parse_program(code);
    }
}

/// Test deep recursion
#[test]
fn test_deep_recursion() {
    let depths = vec![10, 100, 1000];

    for depth in depths {
        let code = format!(": recurse {} 0 do i 1 + recurse loop ; recurse", depth);

        use fastforth_frontend::parse_program;
        let _ = parse_program(&code);
    }
}

/// Test large stack depth
#[test]
fn test_large_stack() {
    let sizes = vec![100, 1000, 10000];

    for size in sizes {
        let mut code = String::new();
        for i in 0..size {
            code.push_str(&format!("{} ", i));
        }
        code.push_str(". ");

        use fastforth_frontend::parse_program;
        let _ = parse_program(&code);
    }
}

/// Test deeply nested control structures
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop_deep_nesting(depth in 1..20usize) {
        let mut code = String::new();

        // Create deeply nested IF-THEN
        for _ in 0..depth {
            code.push_str("1 IF ");
        }
        code.push_str("42 ");
        for _ in 0..depth {
            code.push_str("THEN ");
        }

        use fastforth_frontend::parse_program;
        let _ = parse_program(&code);
    }

    #[test]
    fn prop_deep_loops(depth in 1..10usize) {
        let mut code = String::new();

        // Create deeply nested DO-LOOP
        for i in 0..depth {
            code.push_str(&format!("10 0 DO "));
        }
        code.push_str("i ");
        for _ in 0..depth {
            code.push_str("LOOP ");
        }

        use fastforth_frontend::parse_program;
        let _ = parse_program(&code);
    }
}

/// Test with random combinations of extreme values
proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    #[test]
    fn prop_extreme_arithmetic(
        op in prop_oneof![Just("+"), Just("-"), Just("*")],
        use_max in prop::bool::ANY,
        use_min in prop::bool::ANY,
    ) {
        let a = if use_max { i64::MAX } else { i64::MIN };
        let b = if use_min { i64::MIN } else { i64::MAX };

        let code = format!("{} {} {}", a, b, op);

        use fastforth_frontend::parse_program;
        let _ = parse_program(&code);
    }

    #[test]
    fn prop_division_by_small_values(
        dividend in -1000..1000i64,
        divisor in -10..10i64,
    ) {
        // Avoid division by zero
        if divisor == 0 {
            return Ok(());
        }

        let code = format!("{} {} /", dividend, divisor);

        use fastforth_frontend::parse_program;
        let _ = parse_program(&code);
    }
}

/// Test memory-intensive operations
#[test]
fn test_memory_intensive() {
    let test_cases = vec![
        // Large string literals
        format!(": bigstr S\" {} \" ; bigstr", "x".repeat(10000)),

        // Many word definitions
        {
            let mut code = String::new();
            for i in 0..1000 {
                code.push_str(&format!(": word{} {} ; ", i, i));
            }
            code
        },

        // Long variable chains
        {
            let mut code = String::new();
            for i in 0..100 {
                code.push_str(&format!("VARIABLE var{} ", i));
            }
            code
        },
    ];

    for code in test_cases {
        use fastforth_frontend::parse_program;
        let _ = parse_program(&code);
    }
}

/// Test pathological cases that have found bugs in other compilers
#[test]
fn test_pathological_cases() {
    let corpus = vec![
        // Empty structures
        "IF THEN",
        "BEGIN UNTIL",
        "DO LOOP",

        // Unbalanced stack in branches
        "1 IF 2 3 ELSE 4 THEN",

        // Multiple exits
        "BEGIN DUP WHILE DUP REPEAT",

        // Comments in weird places
        ": test ( comment ) 42 ( another ) ; test",

        // Zero iterations
        "0 0 DO i LOOP",
        "1 0 DO i LOOP",

        // Negative loop bounds
        "-10 -20 DO i LOOP",

        // Stack manipulation edge cases
        "1 2 3 ROT ROT ROT",  // Identity?
        "1 DUP DROP DUP DROP",  // Redundant ops

        // Definition inside definition (should fail)
        ": outer : inner 42 ; ; outer",

        // Very long word name
        &format!(": {} 42 ; {}", "a".repeat(1000), "a".repeat(1000)),

        // Unicode in comments (if supported)
        "( 你好世界 ) 42",

        // Control characters
        ":\t\ntest\t\n42\t\n;\t\ntest",

        // Many spaces
        "1      2      +",
        "   :   test   42   ;   test   ",
    ];

    for code in corpus {
        use fastforth_frontend::parse_program;
        let _ = parse_program(code);
    }
}

/// Test random fuzzing-discovered patterns
/// (Add interesting cases found by fuzzing here)
#[test]
fn test_fuzz_corpus() {
    let corpus = vec![
        // Add patterns discovered by overnight fuzzing
        // Example: "CASE 0 OF ENDOF ENDCASE"
    ];

    for code in corpus {
        use fastforth_frontend::parse_program;
        let _ = parse_program(code);
    }
}
