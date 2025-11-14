//! Code generation demonstration
//!
//! Shows how optimized IR is translated to efficient C code.

use fastforth_optimizer::*;
use fastforth_optimizer::codegen::{CCodegen, CodegenBackend};

fn main() {
    println!("FastForth Code Generation Demo\n");
    println!("===============================\n");

    demo_simple_codegen();
    demo_optimized_codegen();
    demo_word_codegen();
}

fn demo_simple_codegen() {
    println!("1. Simple Expression Code Generation");
    println!("=====================================\n");

    let code = "2 3 + 4 *";
    println!("Forth code: {}\n", code);

    let ir = ForthIR::parse(code).unwrap();
    let mut codegen = CCodegen::new();

    match codegen.generate(&ir) {
        Ok(c_code) => {
            println!("Generated C code:");
            println!("{}", c_code);
        }
        Err(e) => println!("Error: {}", e),
    }
}

fn demo_optimized_codegen() {
    println!("\n2. Optimized Code Generation");
    println!("=============================\n");

    let code = "5 dup + 1 + dup *";
    println!("Forth code: {}\n", code);

    // Generate without optimization
    println!("WITHOUT optimization:");
    let ir_unopt = ForthIR::parse(code).unwrap();
    let mut codegen = CCodegen::new();
    if let Ok(c_code) = codegen.generate(&ir_unopt) {
        println!("{}", c_code);
    }

    // Generate with optimization
    println!("\nWITH optimization:");
    let ir_opt = ForthIR::parse(code).unwrap();
    let optimizer = Optimizer::new(OptimizationLevel::Aggressive);
    let optimized = optimizer.optimize(ir_opt).unwrap();

    let mut codegen = CCodegen::new();
    if let Ok(c_code) = codegen.generate(&optimized) {
        println!("{}", c_code);
    }

    println!("\nNotice the optimized version uses:");
    println!("  - Superinstructions (TOS = TOS + TOS instead of dup then add)");
    println!("  - Constant folding where possible");
    println!("  - Reduced instruction count");
}

fn demo_word_codegen() {
    println!("\n3. Word Definition Code Generation");
    println!("===================================\n");

    let mut ir = ForthIR::new();

    // Define square word
    let square = WordDef::new(
        "square".to_string(),
        vec![Instruction::Dup, Instruction::Mul],
    );
    ir.add_word(square);

    // Define quad word
    let quad = WordDef::new(
        "quad".to_string(),
        vec![
            Instruction::Call("square".to_string()),
            Instruction::Call("square".to_string()),
        ],
    );
    ir.add_word(quad);

    ir.main = vec![
        Instruction::Literal(5),
        Instruction::Call("quad".to_string()),
    ];

    println!("Forth definitions:");
    println!("  : square dup * ;");
    println!("  : quad square square ;");
    println!("  5 quad\n");

    // Generate without inlining
    println!("WITHOUT inlining:");
    let mut codegen = CCodegen::new();
    if let Ok(c_code) = codegen.generate(&ir) {
        // Print just the function definitions
        for line in c_code.lines() {
            if line.starts_with("void") || line.contains("square") || line.contains("quad") {
                println!("{}", line);
            }
        }
    }

    // Generate with inlining
    println!("\nWITH inlining:");
    let optimizer = InlineOptimizer::new(OptimizationLevel::Aggressive);
    let optimized = optimizer.inline(&ir).unwrap();

    let mut codegen = CCodegen::new();
    if let Ok(c_code) = codegen.generate(&optimized) {
        for line in c_code.lines() {
            if line.starts_with("void forth_main") || line.contains("PUSH") || line.contains("TOS") {
                println!("{}", line);
            }
        }
    }

    println!("\nNotice how inlining eliminates function calls!");
}
