//! Simple example of Fast Forth frontend usage

use fastforth_frontend::*;

fn main() -> Result<()> {
    println!("Fast Forth Frontend Example\n");

    // Example 1: Simple arithmetic
    println!("Example 1: Simple arithmetic");
    let source1 = ": double 2 * ;";
    let program1 = parse_program(source1)?;
    println!("Parsed: {:?}", program1.definitions[0].name);
    semantic::analyze(&program1)?;
    println!("Semantic analysis: OK\n");

    // Example 2: Stack manipulation
    println!("Example 2: Stack manipulation");
    let source2 = ": swap-add swap + ;";
    let program2 = parse_program(source2)?;
    println!("Parsed: {:?}", program2.definitions[0].name);
    semantic::analyze(&program2)?;
    println!("Semantic analysis: OK\n");

    // Example 3: Multiple definitions
    println!("Example 3: Multiple definitions");
    let source3 = r#"
        : double 2 * ;
        : triple 3 * ;
        : six-times double triple ;
    "#;
    let program3 = parse_program(source3)?;
    println!("Parsed {} definitions", program3.definitions.len());
    semantic::analyze(&program3)?;
    println!("Semantic analysis: OK\n");

    // Example 4: Stack effect inference
    println!("Example 4: Stack effect inference");
    let program4 = parse_program(": add-one 1 + ;")?;
    let mut stack_inference = stack_effects::StackEffectInference::new();
    let effects = stack_inference.analyze_program(&program4)?;
    for (name, effect) in &effects {
        println!("  {} : {}", name, effect);
    }
    println!();

    // Example 5: Control structures
    println!("Example 5: Control structures");
    let source5 = ": abs dup 0 < IF negate THEN ;";
    let program5 = parse_program(source5)?;
    println!("Parsed: {:?}", program5.definitions[0].name);
    println!("Body has {} words", program5.definitions[0].body.len());

    // Example 6: Detect undefined word
    println!("\nExample 6: Error detection - undefined word");
    let source6 = ": test undefined-word ;";
    let program6 = parse_program(source6)?;
    match semantic::analyze(&program6) {
        Ok(_) => println!("  Unexpected success"),
        Err(e) => println!("  Correctly detected error: {}", e),
    }

    println!("\nAll examples completed successfully!");
    Ok(())
}
