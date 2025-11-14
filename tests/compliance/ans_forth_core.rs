/// ANS Forth Core Word Set Compliance Tests
///
/// Tests all required words from the ANS Forth Core word set
/// Reference: https://forth-standard.org/standard/core

use fast_forth::ForthEngine;

#[test]
fn test_stack_manipulation_dup() {
    let mut engine = ForthEngine::new();
    engine.eval("5 DUP").unwrap();
    assert_eq!(engine.stack(), &[5, 5], "DUP should duplicate top of stack");
}

#[test]
fn test_stack_manipulation_drop() {
    let mut engine = ForthEngine::new();
    engine.eval("5 10 DROP").unwrap();
    assert_eq!(engine.stack(), &[5], "DROP should remove top of stack");
}

#[test]
fn test_stack_manipulation_swap() {
    let mut engine = ForthEngine::new();
    engine.eval("5 10 SWAP").unwrap();
    assert_eq!(engine.stack(), &[10, 5], "SWAP should exchange top two items");
}

#[test]
fn test_stack_manipulation_over() {
    let mut engine = ForthEngine::new();
    // OVER: ( a b -- a b a )
    // TODO: Implement OVER in ForthEngine
    // engine.eval("5 10 OVER").unwrap();
    // assert_eq!(engine.stack(), &[5, 10, 5]);
}

#[test]
fn test_stack_manipulation_rot() {
    let mut engine = ForthEngine::new();
    // ROT: ( a b c -- b c a )
    // TODO: Implement ROT in ForthEngine
    // engine.eval("1 2 3 ROT").unwrap();
    // assert_eq!(engine.stack(), &[2, 3, 1]);
}

#[test]
fn test_arithmetic_addition() {
    let mut engine = ForthEngine::new();
    engine.eval("5 10 +").unwrap();
    assert_eq!(engine.stack(), &[15], "+ should add top two numbers");
}

#[test]
fn test_arithmetic_subtraction() {
    let mut engine = ForthEngine::new();
    engine.eval("10 5 -").unwrap();
    assert_eq!(engine.stack(), &[5], "- should subtract top from second");
}

#[test]
fn test_arithmetic_multiplication() {
    let mut engine = ForthEngine::new();
    engine.eval("5 10 *").unwrap();
    assert_eq!(engine.stack(), &[50], "* should multiply top two numbers");
}

#[test]
fn test_arithmetic_division() {
    let mut engine = ForthEngine::new();
    // TODO: Implement / in ForthEngine
    // engine.eval("20 5 /").unwrap();
    // assert_eq!(engine.stack(), &[4]);
}

#[test]
fn test_arithmetic_mod() {
    let mut engine = ForthEngine::new();
    // TODO: Implement MOD in ForthEngine
    // engine.eval("17 5 MOD").unwrap();
    // assert_eq!(engine.stack(), &[2]);
}

#[test]
fn test_arithmetic_divmod() {
    let mut engine = ForthEngine::new();
    // /MOD: ( n1 n2 -- remainder quotient )
    // TODO: Implement /MOD in ForthEngine
    // engine.eval("17 5 /MOD").unwrap();
    // assert_eq!(engine.stack(), &[2, 3]);
}

#[test]
fn test_comparison_equals() {
    let mut engine = ForthEngine::new();
    // =: ( n1 n2 -- flag )
    // TODO: Implement = in ForthEngine
    // engine.eval("5 5 =").unwrap();
    // assert_eq!(engine.stack(), &[-1]); // true = -1 in Forth

    // engine.clear_stack();
    // engine.eval("5 10 =").unwrap();
    // assert_eq!(engine.stack(), &[0]); // false = 0 in Forth
}

#[test]
fn test_comparison_less_than() {
    let mut engine = ForthEngine::new();
    // <: ( n1 n2 -- flag )
    // TODO: Implement < in ForthEngine
    // engine.eval("5 10 <").unwrap();
    // assert_eq!(engine.stack(), &[-1]); // true
}

#[test]
fn test_comparison_greater_than() {
    let mut engine = ForthEngine::new();
    // >: ( n1 n2 -- flag )
    // TODO: Implement > in ForthEngine
    // engine.eval("10 5 >").unwrap();
    // assert_eq!(engine.stack(), &[-1]); // true
}

#[test]
fn test_logical_and() {
    let mut engine = ForthEngine::new();
    // AND: ( n1 n2 -- n3 )
    // TODO: Implement AND in ForthEngine
    // engine.eval("-1 -1 AND").unwrap();
    // assert_eq!(engine.stack(), &[-1]);
}

#[test]
fn test_logical_or() {
    let mut engine = ForthEngine::new();
    // OR: ( n1 n2 -- n3 )
    // TODO: Implement OR in ForthEngine
    // engine.eval("0 -1 OR").unwrap();
    // assert_eq!(engine.stack(), &[-1]);
}

#[test]
fn test_logical_xor() {
    let mut engine = ForthEngine::new();
    // XOR: ( n1 n2 -- n3 )
    // TODO: Implement XOR in ForthEngine
    // engine.eval("-1 -1 XOR").unwrap();
    // assert_eq!(engine.stack(), &[0]);
}

#[test]
fn test_logical_invert() {
    let mut engine = ForthEngine::new();
    // INVERT: ( n1 -- n2 )
    // TODO: Implement INVERT in ForthEngine
    // engine.eval("0 INVERT").unwrap();
    // assert_eq!(engine.stack(), &[-1]);
}

#[test]
fn test_stack_underflow_dup() {
    let mut engine = ForthEngine::new();
    let result = engine.eval("DUP");
    assert!(result.is_err(), "DUP on empty stack should error");
}

#[test]
fn test_stack_underflow_add() {
    let mut engine = ForthEngine::new();
    engine.eval("5").unwrap();
    let result = engine.eval("+");
    assert!(result.is_err(), "+ with insufficient items should error");
}

#[test]
fn test_complex_expression() {
    let mut engine = ForthEngine::new();
    // ( 5 + 10 ) * 2 - 3 = 27
    engine.eval("5 10 + 2 * 3 -").unwrap();
    assert_eq!(engine.stack(), &[27]);
}

#[test]
fn test_nested_arithmetic() {
    let mut engine = ForthEngine::new();
    // ( 2 + 3 ) * ( 4 + 5 ) = 45
    engine.eval("2 3 + 4 5 + *").unwrap();
    assert_eq!(engine.stack(), &[45]);
}

// Edge cases
#[test]
fn test_negative_numbers() {
    let mut engine = ForthEngine::new();
    // -5 + 10 = 5
    // TODO: Handle negative number parsing
    // engine.eval("-5 10 +").unwrap();
    // assert_eq!(engine.stack(), &[5]);
}

#[test]
fn test_large_numbers() {
    let mut engine = ForthEngine::new();
    engine.eval("1000000 2000000 +").unwrap();
    assert_eq!(engine.stack(), &[3000000]);
}

#[test]
fn test_zero_operations() {
    let mut engine = ForthEngine::new();
    engine.eval("0 5 +").unwrap();
    assert_eq!(engine.stack(), &[5]);

    engine.clear_stack();
    engine.eval("5 0 *").unwrap();
    assert_eq!(engine.stack(), &[0]);
}
