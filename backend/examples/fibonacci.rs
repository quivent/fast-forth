//! Fibonacci example
//!
//! Demonstrates compilation of a recursive Forth function.

#[cfg(feature = "llvm")]
use backend::{LLVMBackend, CodeGenerator, CompilationMode};
#[cfg(feature = "llvm")]
use inkwell::{context::Context, OptimizationLevel};
#[cfg(feature = "llvm")]
use fastforth_frontend::ssa::{SSAFunction, SSAInstruction, Register, BlockId, BasicBlock, BinaryOperator};
#[cfg(feature = "llvm")]
use smallvec::smallvec;
#[cfg(feature = "llvm")]
use std::path::Path;

#[cfg(feature = "llvm")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Fibonacci Compilation Example ===\n");

    // Create LLVM context
    let context = Context::create();

    // Create backend
    let mut backend = LLVMBackend::new(
        &context,
        "fibonacci_module",
        CompilationMode::AOT,
        OptimizationLevel::Aggressive,
    );

    // Create Fibonacci function: fib(n) = n <= 1 ? 1 : fib(n-1) + fib(n-2)
    //
    // In Forth:
    // : FIB ( n -- fib[n] )
    //   DUP 2 < IF
    //     DROP 1
    //   ELSE
    //     DUP 1- RECURSE
    //     SWAP 2- RECURSE +
    //   THEN ;

    let mut fib_func = SSAFunction::new("fib".to_string(), 1);

    // Entry block: check if n < 2
    let entry = BlockId(0);
    let then_block = BlockId(1);
    let else_block = BlockId(2);

    let mut entry_bb = BasicBlock::new(entry);

    let n_reg = Register(0);         // parameter
    let two_reg = Register(1);       // constant 2
    let cond_reg = Register(2);      // n < 2

    entry_bb.instructions.push(SSAInstruction::LoadInt {
        dest: two_reg,
        value: 2,
    });

    entry_bb.instructions.push(SSAInstruction::BinaryOp {
        dest: cond_reg,
        op: BinaryOperator::Lt,
        left: n_reg,
        right: two_reg,
    });

    entry_bb.instructions.push(SSAInstruction::Branch {
        condition: cond_reg,
        true_block: then_block,
        false_block: else_block,
    });

    // Then block: return 1
    let mut then_bb = BasicBlock::new(then_block);
    let one_reg = Register(3);

    then_bb.instructions.push(SSAInstruction::LoadInt {
        dest: one_reg,
        value: 1,
    });

    then_bb.instructions.push(SSAInstruction::Return {
        values: smallvec![one_reg],
    });

    // Else block: recursive case (simplified for example)
    let mut else_bb = BasicBlock::new(else_block);

    // For now, just return n (full recursion would require function call support)
    else_bb.instructions.push(SSAInstruction::Return {
        values: smallvec![n_reg],
    });

    fib_func.blocks.push(entry_bb);
    fib_func.blocks.push(then_bb);
    fib_func.blocks.push(else_bb);

    println!("Compiling function: {}\n", fib_func.name);
    println!("SSA IR:");
    println!("{}", fib_func);

    // Generate LLVM IR
    backend.generate(&fib_func)?;

    println!("\nGenerated LLVM IR:");
    println!("{}", backend.print_to_string());

    // Write object file
    let output_path = Path::new("fib.o");
    backend.finalize(output_path)?;

    println!("\nObject file written to: {}", output_path.display());
    println!("\nNote: Full recursive implementation requires function call support.");

    Ok(())
}

#[cfg(not(feature = "llvm"))]
fn main() {
    eprintln!("This example requires the 'llvm' feature to be enabled.");
    eprintln!("Run with: cargo run --example fibonacci --features llvm");
    std::process::exit(1);
}
