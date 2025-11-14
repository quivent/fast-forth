/// Regression tests for optimizations
///
/// Ensure that optimized code produces the same results as unoptimized code

use fast_forth::ForthEngine;

/// Test that constant folding optimization is correct
#[test]
fn test_constant_folding() {
    let mut optimized = ForthEngine::new();
    let mut unoptimized = ForthEngine::new();

    // In optimized version, "2 3 +" should be folded to "5"
    // In unoptimized version, it's executed at runtime

    optimized.eval("2 3 +").unwrap();
    unoptimized.eval("2 3 +").unwrap();

    assert_eq!(optimized.stack(), unoptimized.stack());
}

/// Test that dead code elimination doesn't break semantics
#[test]
fn test_dead_code_elimination() {
    // TODO: Test dead code elimination
    // Code like "5 DROP 10" should leave just 10 on stack
}

/// Test that loop unrolling preserves correctness
#[test]
fn test_loop_unrolling() {
    // TODO: Test loop unrolling
    // Unrolled loops should produce same results as non-unrolled
}

/// Test that tail call optimization preserves behavior
#[test]
fn test_tail_call_optimization() {
    // TODO: Test TCO
    // Recursive calls in tail position should be optimized
}

/// Test that inlining doesn't change semantics
#[test]
fn test_inlining() {
    // TODO: Test function inlining
    // Inlined functions should behave identically
}

/// Test that register allocation doesn't corrupt values
#[test]
fn test_register_allocation() {
    // TODO: Test register allocation
    // Values should be preserved correctly
}

/// Test that peephole optimizations are correct
#[test]
fn test_peephole_optimization() {
    // TODO: Test peephole optimization
    // Patterns like "DUP DROP" should be eliminated
}

/// Comprehensive regression test suite
#[test]
fn regression_suite() {
    let test_cases = vec![
        ("5 10 +", vec![15]),
        ("5 10 SWAP -", vec![5]),
        ("2 3 + 4 5 + *", vec![45]),
        ("10 DUP *", vec![100]),
    ];

    for (code, expected) in test_cases {
        let mut engine = ForthEngine::new();
        engine.eval(code).unwrap();
        assert_eq!(
            engine.stack(),
            &expected[..],
            "Failed for: {}",
            code
        );
    }
}
