//! Simple compilation example
//!
//! Demonstrates how to compile a simple Forth function to native code using the LLVM backend.

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
    println!("=== Fast Forth LLVM Backend Example ===\n");

    // Create LLVM context
    let context = Context::create();

    // Create backend
    let mut backend = LLVMBackend::new(
        &context,
        "example_module",
        CompilationMode::AOT,
        OptimizationLevel::Default,
    );

    // Create a simple function: double(x) = x * 2
    // In Forth: : DOUBLE 2 * ;
    let mut double_func = SSAFunction::new("double".to_string(), 1);

    // Entry block
    let entry = BlockId(0);
    let mut entry_block = BasicBlock::new(entry);

    // %0 = parameter (x)
    // %1 = constant 2
    // %2 = multiply %0, %1
    // return %2

    let param_reg = Register(0);
    let const_reg = Register(1);
    let result_reg = Register(2);

    entry_block.instructions.push(SSAInstruction::LoadInt {
        dest: const_reg,
        value: 2,
    });

    entry_block.instructions.push(SSAInstruction::BinaryOp {
        dest: result_reg,
        op: BinaryOperator::Mul,
        left: param_reg,
        right: const_reg,
    });

    entry_block.instructions.push(SSAInstruction::Return {
        values: smallvec![result_reg],
    });

    double_func.blocks.push(entry_block);

    println!("Compiling function: {}\n", double_func.name);
    println!("SSA IR:");
    println!("{}", double_func);

    // Generate LLVM IR
    backend.generate(&double_func)?;

    println!("\nGenerated LLVM IR:");
    println!("{}", backend.print_to_string());

    // Write object file
    let output_path = Path::new("double.o");
    backend.finalize(output_path)?;

    println!("\nObject file written to: {}", output_path.display());
    println!("\nTo create executable, run:");
    println!("  gcc double.o ../runtime/forth_runtime.c -o double");

    Ok(())
}

#[cfg(not(feature = "llvm"))]
fn main() {
    eprintln!("This example requires the 'llvm' feature to be enabled.");
    eprintln!("Run with: cargo run --example simple_compile --features llvm");
    std::process::exit(1);
}
