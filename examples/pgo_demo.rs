//! Profile-Guided Optimization Demo
//!
//! This example demonstrates the complete PGO workflow:
//! 1. Profile program execution to collect pattern frequencies
//! 2. Identify hot patterns (executed >10K times)
//! 3. Generate fused superinstructions for hot patterns
//! 4. Measure speedup from PGO optimizations
//! 5. Export/import profiling data
//!
//! Run with: cargo run --example pgo_demo

use fastforth_optimizer::{
    ForthIR, Instruction, Optimizer, OptimizationLevel, PGOOptimizer, WordDef,
};

fn main() {
    println!("═══════════════════════════════════════════════════════════");
    println!("  FastForth Profile-Guided Optimization (PGO) Demo");
    println!("═══════════════════════════════════════════════════════════");
    println!();

    // Example 1: Simple Pattern Detection
    println!("Example 1: Simple Pattern Detection");
    println!("────────────────────────────────────────────────────────────");
    demo_simple_pattern_detection();
    println!();

    // Example 2: Hot Loop Optimization
    println!("Example 2: Hot Loop Optimization");
    println!("────────────────────────────────────────────────────────────");
    demo_hot_loop_optimization();
    println!();

    // Example 3: Fibonacci with PGO
    println!("Example 3: Fibonacci with PGO");
    println!("────────────────────────────────────────────────────────────");
    demo_fibonacci_pgo();
    println!();

    // Example 4: Export/Import Profiling Data
    println!("Example 4: Export/Import Profiling Data");
    println!("────────────────────────────────────────────────────────────");
    demo_export_import();
    println!();

    // Example 5: Auto-Tuning Workflow
    println!("Example 5: Auto-Tuning Workflow");
    println!("────────────────────────────────────────────────────────────");
    demo_auto_tuning();
    println!();

    println!("═══════════════════════════════════════════════════════════");
    println!("  PGO Demo Complete!");
    println!("═══════════════════════════════════════════════════════════");
}

/// Demo 1: Simple pattern detection
fn demo_simple_pattern_detection() {
    // Create a simple program with repeating patterns
    let mut ir = ForthIR::new();
    ir.main = vec![
        Instruction::Literal(5),
        Instruction::Dup,
        Instruction::Add,  // Pattern: DUP ADD (2*)
        Instruction::Literal(3),
        Instruction::Dup,
        Instruction::Mul,  // Pattern: DUP MUL (square)
    ];

    let mut pgo = PGOOptimizer::new();
    pgo.enable_profiling();

    println!("Original code:");
    println!("  5 dup + 3 dup *");
    println!();

    // Profile the code (simulate 15,000 executions)
    println!("Profiling (15,000 iterations)...");
    for _ in 0..15_000 {
        pgo.profile_ir(&ir);
    }

    // Identify hot patterns
    let hot = pgo.identify_hot_patterns(10_000);
    println!("Hot patterns found: {}", hot.len());
    for (i, pattern) in hot.iter().enumerate().take(5) {
        println!(
            "  {}. {} (count: {}, speedup: {:.1}%)",
            i + 1,
            pattern.key,
            pattern.count,
            pattern.potential_speedup
        );
    }
    println!();

    // Generate fusions
    let fusions = pgo.generate_fusions(&hot);
    println!("Fusions generated: {}", fusions.len());
    for (i, (pattern, fused)) in fusions.iter().enumerate() {
        println!("  {}. {} → {:?}", i + 1, pattern, fused);
    }
}

/// Demo 2: Hot loop optimization
fn demo_hot_loop_optimization() {
    // Sieve-like loop with repeated patterns
    let mut ir = ForthIR::new();

    // : sieve-inner ( n -- n' ) dup + 1 + ;
    let sieve_inner = WordDef::new(
        "sieve-inner".to_string(),
        vec![
            Instruction::Dup,
            Instruction::Add,
            Instruction::Literal(1),
            Instruction::Add,
        ],
    );
    ir.add_word(sieve_inner);

    // Main: call sieve-inner repeatedly
    ir.main = vec![
        Instruction::Literal(2),
        Instruction::Call("sieve-inner".to_string()),
        Instruction::Call("sieve-inner".to_string()),
        Instruction::Call("sieve-inner".to_string()),
    ];

    let mut pgo = PGOOptimizer::new();
    pgo.enable_profiling();

    println!("Hot loop code:");
    println!("  : sieve-inner dup + 1 + ;");
    println!("  2 sieve-inner sieve-inner sieve-inner");
    println!();

    // Profile with many iterations
    println!("Profiling (50,000 iterations)...");
    for _ in 0..50_000 {
        pgo.profile_ir(&ir);
    }

    // Optimize with PGO
    match pgo.optimize(&ir, 10_000) {
        Ok((optimized_ir, stats)) => {
            println!("{}", stats);
            println!();

            // Show before/after instruction count
            let before_count = ir.instruction_count();
            let after_count = optimized_ir.instruction_count();
            let reduction =
                ((before_count - after_count) as f64 / before_count as f64) * 100.0;

            println!("Optimization results:");
            println!("  Before: {} instructions", before_count);
            println!("  After: {} instructions", after_count);
            println!("  Reduction: {:.1}%", reduction);
        }
        Err(e) => println!("Optimization failed: {}", e),
    }
}

/// Demo 3: Fibonacci with PGO
fn demo_fibonacci_pgo() {
    // Create Fibonacci implementation
    let mut ir = ForthIR::new();

    // : fib-step ( a b -- b a+b )
    //   over over + swap
    // This becomes: OVER OVER ADD SWAP
    let fib_step = WordDef::new(
        "fib-step".to_string(),
        vec![
            Instruction::Over,
            Instruction::Over,
            Instruction::Add,
            Instruction::Swap,
        ],
    );
    ir.add_word(fib_step);

    // Main: 1 1 fib-step fib-step fib-step ...
    ir.main = vec![
        Instruction::Literal(1),
        Instruction::Literal(1),
        Instruction::Call("fib-step".to_string()),
        Instruction::Call("fib-step".to_string()),
        Instruction::Call("fib-step".to_string()),
        Instruction::Call("fib-step".to_string()),
        Instruction::Call("fib-step".to_string()),
    ];

    let mut pgo = PGOOptimizer::new();
    pgo.enable_profiling();

    println!("Fibonacci code:");
    println!("  : fib-step over over + swap ;");
    println!("  1 1 fib-step fib-step fib-step ...");
    println!();

    // Profile
    println!("Profiling (20,000 iterations)...");
    for _ in 0..20_000 {
        pgo.profile_ir(&ir);
    }

    // Show hot patterns
    let hot = pgo.identify_hot_patterns(5_000);
    println!("Hot patterns in Fibonacci:");
    for (i, pattern) in hot.iter().enumerate().take(3) {
        println!(
            "  {}. {} (count: {}, potential speedup: {:.1}%)",
            i + 1,
            pattern.key,
            pattern.count,
            pattern.potential_speedup
        );
    }
}

/// Demo 4: Export/Import profiling data
fn demo_export_import() {
    // Create and profile some code
    let mut ir = ForthIR::new();
    ir.main = vec![
        Instruction::Literal(10),
        Instruction::Dup,
        Instruction::Mul,
    ];

    let mut pgo = PGOOptimizer::new();
    pgo.enable_profiling();

    for _ in 0..12_000 {
        pgo.profile_ir(&ir);
    }

    // Export profiling data
    let json = pgo.export_data();
    println!("Exported profiling data:");
    println!("  Size: {} bytes", json.len());
    println!();

    // Create new optimizer and import data
    let mut pgo2 = PGOOptimizer::new();
    match pgo2.import_data(&json) {
        Ok(_) => {
            println!("Successfully imported profiling data");
            let stats = pgo2.database().stats();
            println!("  {}", stats);
        }
        Err(e) => println!("Import failed: {}", e),
    }
}

/// Demo 5: Auto-tuning workflow
fn demo_auto_tuning() {
    println!("Auto-tuning workflow:");
    println!();

    // Create program
    let mut ir = ForthIR::new();
    ir.main = vec![
        Instruction::Literal(5),
        Instruction::Dup,
        Instruction::Add,
        Instruction::Literal(1),
        Instruction::Add,
        Instruction::Dup,
        Instruction::Mul,
    ];

    // Iteration 1: Initial profile
    println!("Iteration 1: Initial Profile");
    let mut pgo = PGOOptimizer::new();
    pgo.enable_profiling();

    for _ in 0..15_000 {
        pgo.profile_ir(&ir);
    }

    match pgo.optimize(&ir, 10_000) {
        Ok((ir1, stats1)) => {
            println!("  {}", stats1);
            println!();

            // Iteration 2: Re-profile optimized code
            println!("Iteration 2: Re-profile Optimized Code");
            pgo.enable_profiling();

            for _ in 0..15_000 {
                pgo.profile_ir(&ir1);
            }

            match pgo.optimize(&ir1, 10_000) {
                Ok((ir2, stats2)) => {
                    println!("  {}", stats2);
                    println!();

                    // Compare
                    let original = ir.instruction_count();
                    let after1 = ir1.instruction_count();
                    let after2 = ir2.instruction_count();

                    println!("Convergence:");
                    println!("  Original: {} instructions", original);
                    println!("  After iteration 1: {} instructions", after1);
                    println!("  After iteration 2: {} instructions", after2);

                    if after2 == after1 {
                        println!("  ✓ Converged!");
                    } else {
                        println!("  → Further optimization possible");
                    }
                }
                Err(e) => println!("Iteration 2 failed: {}", e),
            }
        }
        Err(e) => println!("Iteration 1 failed: {}", e),
    }
}
