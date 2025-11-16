//! Comprehensive tests for Extended ANS Forth words
//!
//! Tests for the 18 extended words implemented:
//! - Priority 1: Memory Operations (VARIABLE, CONSTANT, VALUE, !, @, +!)
//! - Priority 2: Advanced Stack (>R, R>, R@, 2>R, 2R>, 2R@)
//! - Priority 4: Base Conversion (DECIMAL, HEX, BINARY, OCTAL)

use fastforth::ForthEngine;

// ============================================================================
// PRIORITY 1: Memory Operations Tests
// ============================================================================

#[test]
fn test_store_and_fetch() {
    let mut engine = ForthEngine::new();

    // Test ! (store) and @ (fetch)
    // Push value and address, store, then fetch
    engine.eval("42 1000 !").unwrap();
    engine.eval("1000 @").unwrap();

    assert_eq!(engine.stack(), &[42], "Fetch should retrieve stored value");
}

#[test]
fn test_store_multiple_addresses() {
    let mut engine = ForthEngine::new();

    // Store different values at different addresses
    engine.eval("10 1000 !").unwrap();
    engine.eval("20 1008 !").unwrap();
    engine.eval("30 1010 !").unwrap();

    // Fetch in different order
    engine.eval("1010 @").unwrap();
    engine.eval("1000 @").unwrap();
    engine.eval("1008 @").unwrap();

    assert_eq!(engine.stack(), &[30, 10, 20], "Multiple addresses should work independently");
}

#[test]
fn test_plus_store() {
    let mut engine = ForthEngine::new();

    // Initialize memory location
    engine.eval("100 2000 !").unwrap();

    // Add 50 to it using +!
    engine.eval("50 2000 +!").unwrap();

    // Fetch result
    engine.eval("2000 @").unwrap();

    assert_eq!(engine.stack(), &[150], "+! should add to existing value");
}

#[test]
fn test_plus_store_negative() {
    let mut engine = ForthEngine::new();

    // Initialize memory location
    engine.eval("100 3000 !").unwrap();

    // Subtract using negative value with +!
    engine.eval("-30 3000 +!").unwrap();

    // Fetch result
    engine.eval("3000 @").unwrap();

    assert_eq!(engine.stack(), &[70], "+! should work with negative values");
}

#[test]
fn test_variable() {
    let mut engine = ForthEngine::new();

    // Define a variable
    let addr = engine.define_variable("COUNTER");

    // Variable name should push its address
    engine.eval("COUNTER").unwrap();

    assert_eq!(engine.stack(), &[addr], "VARIABLE should push its address");

    // Test storing and fetching from variable
    engine.clear_stack();
    engine.eval("42 COUNTER !").unwrap();
    engine.eval("COUNTER @").unwrap();

    assert_eq!(engine.stack(), &[42], "Should be able to store/fetch from variable");
}

#[test]
fn test_variable_increment() {
    let mut engine = ForthEngine::new();

    // Define variable and initialize
    engine.define_variable("X");
    engine.eval("10 X !").unwrap();

    // Increment it
    engine.eval("5 X +!").unwrap();

    // Check value
    engine.eval("X @").unwrap();

    assert_eq!(engine.stack(), &[15], "Should increment variable");
}

#[test]
fn test_constant() {
    let mut engine = ForthEngine::new();

    // Define a constant
    engine.define_constant("PI", 314);

    // Constant should push its value
    engine.eval("PI").unwrap();

    assert_eq!(engine.stack(), &[314], "CONSTANT should push its value");

    // Test using constant in expression
    engine.clear_stack();
    engine.eval("PI 2 *").unwrap();

    assert_eq!(engine.stack(), &[628], "Should use constant in expressions");
}

#[test]
fn test_value() {
    let mut engine = ForthEngine::new();

    // Define a value
    engine.define_value("LIMIT", 100);

    // Value should push its value
    engine.eval("LIMIT").unwrap();

    assert_eq!(engine.stack(), &[100], "VALUE should push its value");
}

#[test]
fn test_value_update() {
    let mut engine = ForthEngine::new();

    // Define and use value
    engine.define_value("SIZE", 50);
    engine.eval("SIZE").unwrap();
    assert_eq!(engine.stack(), &[50]);

    // Update value
    engine.clear_stack();
    engine.update_value("SIZE", 75).unwrap();
    engine.eval("SIZE").unwrap();

    assert_eq!(engine.stack(), &[75], "Should update value with TO");
}

// ============================================================================
// PRIORITY 2: Advanced Stack Operations (Return Stack)
// ============================================================================

#[test]
fn test_to_r_and_r_from() {
    let mut engine = ForthEngine::new();

    // Move value to return stack
    engine.eval("42 >R").unwrap();

    assert_eq!(engine.stack(), &[] as &[i64], "Data stack should be empty after >R");
    assert_eq!(engine.return_stack(), &[42], "Return stack should have value");

    // Move back to data stack
    engine.eval("R>").unwrap();

    assert_eq!(engine.stack(), &[42], "Value should return to data stack");
    assert_eq!(engine.return_stack(), &[] as &[i64], "Return stack should be empty");
}

#[test]
fn test_r_fetch() {
    let mut engine = ForthEngine::new();

    // Move value to return stack
    engine.eval("99 >R").unwrap();

    // Copy from return stack without removing
    engine.eval("R@").unwrap();

    assert_eq!(engine.stack(), &[99], "R@ should copy to data stack");
    assert_eq!(engine.return_stack(), &[99], "R@ should not remove from return stack");

    // Copy again
    engine.eval("R@").unwrap();

    assert_eq!(engine.stack(), &[99, 99], "R@ can be called multiple times");
}

#[test]
fn test_return_stack_multiple_values() {
    let mut engine = ForthEngine::new();

    // Move multiple values
    engine.eval("1 >R 2 >R 3 >R").unwrap();

    assert_eq!(engine.return_stack(), &[1, 2, 3], "Return stack preserves order");

    // Retrieve in reverse order (LIFO)
    engine.eval("R> R> R>").unwrap();

    assert_eq!(engine.stack(), &[3, 2, 1], "Return stack is LIFO");
}

#[test]
fn test_2to_r_and_2r_from() {
    let mut engine = ForthEngine::new();

    // Move two values to return stack
    engine.eval("10 20 2>R").unwrap();

    assert_eq!(engine.stack(), &[] as &[i64], "Data stack should be empty");
    assert_eq!(engine.return_stack(), &[10, 20], "Return stack should have both values");

    // Move both back
    engine.eval("2R>").unwrap();

    assert_eq!(engine.stack(), &[10, 20], "Both values should return in order");
    assert_eq!(engine.return_stack(), &[] as &[i64], "Return stack should be empty");
}

#[test]
fn test_2r_fetch() {
    let mut engine = ForthEngine::new();

    // Move two values to return stack
    engine.eval("5 15 2>R").unwrap();

    // Copy both without removing
    engine.eval("2R@").unwrap();

    assert_eq!(engine.stack(), &[5, 15], "2R@ should copy both values");
    assert_eq!(engine.return_stack(), &[5, 15], "2R@ should not remove values");
}

#[test]
fn test_return_stack_complex_operations() {
    let mut engine = ForthEngine::new();

    // Complex sequence: swap values using return stack
    engine.eval("1 2 3").unwrap();
    engine.eval(">R >R").unwrap();  // Save 3 and 2
    engine.eval("R> R>").unwrap();  // Retrieve in reverse order

    assert_eq!(engine.stack(), &[1, 2, 3], "Complex return stack operations");
}

#[test]
fn test_return_stack_underflow() {
    let mut engine = ForthEngine::new();

    // Try to pop from empty return stack
    let result = engine.eval("R>");

    assert!(result.is_err(), "R> on empty stack should error");
    assert!(result.unwrap_err().to_string().contains("underflow"));
}

// ============================================================================
// PRIORITY 4: Base Conversion Tests
// ============================================================================

#[test]
fn test_decimal_base() {
    let mut engine = ForthEngine::new();

    engine.eval("DECIMAL").unwrap();

    // Base should be set to 10 (we can't directly test this without number parsing,
    // but we can verify the command doesn't error)
    assert_eq!(engine.stack(), &[] as &[i64], "DECIMAL should not affect stack");
}

#[test]
fn test_hex_base() {
    let mut engine = ForthEngine::new();

    engine.eval("HEX").unwrap();

    assert_eq!(engine.stack(), &[] as &[i64], "HEX should not affect stack");
}

#[test]
fn test_binary_base() {
    let mut engine = ForthEngine::new();

    engine.eval("BINARY").unwrap();

    assert_eq!(engine.stack(), &[] as &[i64], "BINARY should not affect stack");
}

#[test]
fn test_octal_base() {
    let mut engine = ForthEngine::new();

    engine.eval("OCTAL").unwrap();

    assert_eq!(engine.stack(), &[] as &[i64], "OCTAL should not affect stack");
}

#[test]
fn test_base_switching() {
    let mut engine = ForthEngine::new();

    // Switch between different bases
    engine.eval("DECIMAL").unwrap();
    engine.eval("HEX").unwrap();
    engine.eval("BINARY").unwrap();
    engine.eval("DECIMAL").unwrap();

    assert_eq!(engine.stack(), &[] as &[i64], "Base switching should work smoothly");
}

// ============================================================================
// Integration Tests: Combining Extended Words
// ============================================================================

#[test]
fn test_variable_with_return_stack() {
    let mut engine = ForthEngine::new();

    // Define variable
    engine.define_variable("TEMP");

    // Use return stack to preserve value during calculation
    engine.eval("10 >R 5 R> +").unwrap();
    engine.eval("TEMP !").unwrap();

    engine.eval("TEMP @").unwrap();

    assert_eq!(engine.stack(), &[15], "Variable with return stack");
}

#[test]
fn test_multiple_variables() {
    let mut engine = ForthEngine::new();

    // Define multiple variables
    engine.define_variable("A");
    engine.define_variable("B");
    engine.define_variable("C");

    // Store values
    engine.eval("10 A ! 20 B ! 30 C !").unwrap();

    // Perform calculation
    engine.eval("A @ B @ + C @ +").unwrap();

    assert_eq!(engine.stack(), &[60], "Multiple variables calculation");
}

#[test]
fn test_constant_and_value_mix() {
    let mut engine = ForthEngine::new();

    // Use LIMIT instead of MAX to avoid collision with builtin MAX word
    engine.define_constant("LIMIT", 100);
    engine.define_value("CURRENT", 50);

    // Calculate remaining
    engine.eval("LIMIT CURRENT -").unwrap();

    assert_eq!(engine.stack(), &[50], "Constant and value together");
}

#[test]
fn test_memory_operations_sequence() {
    let mut engine = ForthEngine::new();

    // Complex memory operation sequence
    engine.eval("5 4000 !").unwrap();      // Store 5
    engine.eval("10 4000 +!").unwrap();    // Add 10 (now 15)
    engine.eval("4000 @ DUP *").unwrap();  // Fetch and square

    assert_eq!(engine.stack(), &[225], "Memory operation sequence: 15 * 15");
}

#[test]
fn test_return_stack_temporary_storage() {
    let mut engine = ForthEngine::new();

    // Use return stack for temporary storage during calculation
    engine.eval("3 4 >R DUP * R> DUP * +").unwrap();

    // (3*3) + (4*4) = 9 + 16 = 25
    assert_eq!(engine.stack(), &[25], "Return stack temporary storage");
}

#[test]
fn test_nested_return_stack() {
    let mut engine = ForthEngine::new();

    // Nested return stack usage
    engine.eval("1 2 3 4").unwrap();
    engine.eval("2>R 2>R").unwrap();        // Save all four
    engine.eval("2R> 2R>").unwrap();        // Retrieve all four in original order

    assert_eq!(engine.stack(), &[1, 2, 3, 4], "Nested 2>R operations should restore original order");
}

// ============================================================================
// Edge Cases and Error Handling
// ============================================================================

#[test]
fn test_fetch_uninitialized_memory() {
    let mut engine = ForthEngine::new();

    // Fetch from uninitialized address should return 0
    engine.eval("9999 @").unwrap();

    assert_eq!(engine.stack(), &[0], "Uninitialized memory should be 0");
}

#[test]
fn test_plus_store_uninitialized() {
    let mut engine = ForthEngine::new();

    // +! on uninitialized memory (treated as 0)
    engine.eval("42 10000 +!").unwrap();
    engine.eval("10000 @").unwrap();

    assert_eq!(engine.stack(), &[42], "+! on uninitialized memory");
}

#[test]
fn test_stack_underflow_store() {
    let mut engine = ForthEngine::new();

    // Try to store with insufficient stack
    let result = engine.eval("5 !");

    assert!(result.is_err(), "! with insufficient stack should error");
}

#[test]
fn test_stack_underflow_fetch() {
    let mut engine = ForthEngine::new();

    // Try to fetch with empty stack
    let result = engine.eval("@");

    assert!(result.is_err(), "@ with empty stack should error");
}

#[test]
fn test_2r_fetch_underflow() {
    let mut engine = ForthEngine::new();

    // Try 2R@ with only one value on return stack
    engine.eval("1 >R").unwrap();
    let result = engine.eval("2R@");

    assert!(result.is_err(), "2R@ with insufficient return stack should error");
}

// ============================================================================
// Output/Print Tests
// ============================================================================

#[test]
fn test_dot_output() {
    let mut engine = ForthEngine::new();

    engine.eval("42 .").unwrap();

    assert!(engine.output().contains("42"), "Dot should output value");
    assert_eq!(engine.stack(), &[] as &[i64], "Dot should consume value");
}

#[test]
fn test_multiple_dot_output() {
    let mut engine = ForthEngine::new();

    engine.eval("1 . 2 . 3 .").unwrap();

    let output = engine.output();
    assert!(output.contains("1"), "Output should contain 1");
    assert!(output.contains("2"), "Output should contain 2");
    assert!(output.contains("3"), "Output should contain 3");
}

// ============================================================================
// Performance and Stress Tests
// ============================================================================

#[test]
fn test_many_variables() {
    let mut engine = ForthEngine::new();

    // Define many variables
    for i in 0..100 {
        engine.define_variable(&format!("VAR{}", i));
    }

    // Access them
    engine.eval("VAR0").unwrap();
    engine.eval("VAR50").unwrap();
    engine.eval("VAR99").unwrap();

    // Should have 3 addresses on stack
    assert_eq!(engine.stack().len(), 3, "Should handle many variables");
}

#[test]
fn test_deep_return_stack() {
    let mut engine = ForthEngine::new();

    // Push many values to return stack
    for i in 1..=20 {
        engine.eval(&format!("{} >R", i)).unwrap();
    }

    assert_eq!(engine.return_stack().len(), 20, "Return stack depth");

    // Pop them all
    for _ in 0..20 {
        engine.eval("R>").unwrap();
    }

    assert_eq!(engine.return_stack().len(), 0, "Return stack cleared");
}

#[test]
fn test_memory_intensive() {
    let mut engine = ForthEngine::new();

    // Store values at many addresses
    for i in 0..100 {
        let addr = 5000 + (i * 8);
        engine.eval(&format!("{} {} !", i, addr)).unwrap();
    }

    // Verify a few
    engine.eval("5000 @").unwrap();  // Should be 0
    engine.eval("5008 @").unwrap();  // Should be 1

    // Just verify we can store and retrieve
    assert_eq!(engine.stack(), &[0, 1], "Memory stress test");
}
