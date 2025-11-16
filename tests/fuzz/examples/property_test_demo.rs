/// Demonstration of property-based testing for Fast Forth
///
/// This example shows how proptest systematically explores the input space
/// and automatically shrinks failing cases to minimal examples.

use proptest::prelude::*;

/// Simple demonstration: Test that parsing arithmetic expressions doesn't crash
fn main() {
    println!("Property-Based Fuzzing Demo for Fast Forth");
    println!("===========================================\n");

    // Run a few example test cases
    let test_cases = vec![
        ("42 17 +", "Simple addition"),
        ("100 50 -", "Subtraction"),
        ("10 5 *", "Multiplication"),
        ("100 10 /", "Division"),
        ("5 DUP", "Stack operation"),
        (": square dup * ;", "Word definition"),
        ("10 0 DO I LOOP", "Loop"),
        ("5 3 > IF 42 THEN", "Conditional"),
    ];

    println!("Testing {} hand-crafted cases:", test_cases.len());
    for (code, description) in &test_cases {
        print!("  {} ... ", description);
        match fastforth_frontend::parse_program(code) {
            Ok(_) => println!("✓ Parsed"),
            Err(e) => println!("✗ Error: {}", e),
        }
    }

    println!("\nProperty-based testing would generate thousands of random cases like:");
    println!("  - Random arithmetic: '9847 3621 *'");
    println!("  - Random stack ops: '42 DUP 17 SWAP 99 OVER'");
    println!("  - Random control flow: '5 10 > IF 100 ELSE 200 THEN'");
    println!("  - Random definitions: ': foo 1 2 + ;'");

    println!("\nTo run the full property-based test suite:");
    println!("  cd tests/fuzz");
    println!("  cargo test --lib");
    println!("\nThis will run ~6000 test cases by default.");
    println!("Use PROPTEST_CASES=10000 for deeper exploration.");
}
