//! Comprehensive optimization demonstration
//!
//! Shows before/after examples for each optimization pass with metrics.

use fastforth_optimizer::*;

fn main() {
    println!("FastForth Optimizer - Demonstration\n");
    println!("====================================\n");

    demo_constant_folding();
    demo_superinstructions();
    demo_dead_code_elimination();
    demo_inlining();
    demo_stack_caching();
    demo_full_pipeline();
}

fn demo_constant_folding() {
    println!("\n1. CONSTANT FOLDING");
    println!("===================\n");

    let examples = vec![
        ("Simple arithmetic", "2 3 +"),
        ("Complex expression", "2 3 + 4 *"),
        ("Nested operations", "10 20 + 30 40 + * 2 /"),
        ("With dup", "5 dup +"),
    ];

    let folder = ConstantFolder::new();

    for (name, code) in examples {
        println!("Example: {}", name);
        let ir = ForthIR::parse(code).unwrap();
        let optimized = folder.fold(&ir).unwrap();

        println!("  Before: {} instructions", ir.instruction_count());
        println!("  After:  {} instructions", optimized.instruction_count());
        println!(
            "  Reduction: {}%\n",
            ((ir.instruction_count() - optimized.instruction_count()) as f64
                / ir.instruction_count() as f64
                * 100.0)
        );
    }
}

fn demo_superinstructions() {
    println!("\n2. SUPERINSTRUCTION RECOGNITION");
    println!("================================\n");

    let examples = vec![
        ("Double (dup +)", "5 dup +", "Fuses to: 2*"),
        ("Square (dup *)", "7 dup *", "Fuses to: square"),
        ("Increment (1 +)", "x 1 +", "Fuses to: x++"),
        ("Decrement (1 -)", "x 1 -", "Fuses to: x--"),
        ("Multiple patterns", "5 dup + 1 + dup *", "Multiple fusions"),
    ];

    let optimizer = SuperinstructionOptimizer::new();

    for (name, code, description) in examples {
        println!("Example: {}", name);
        let ir = ForthIR::parse(code).unwrap();
        let optimized = optimizer.recognize(&ir).unwrap();

        let stats = optimizer.get_stats(&ir, &optimized);
        println!("  {}", description);
        println!("  Before: {} instructions", stats.before_instructions);
        println!("  After:  {} instructions", stats.after_instructions);
        println!("  Reduction: {:.1}%\n", stats.reduction_percent);
    }
}

fn demo_dead_code_elimination() {
    println!("\n3. DEAD CODE ELIMINATION");
    println!("========================\n");

    let examples = vec![
        ("Identity (dup drop)", "5 dup drop"),
        ("Canceled swap", "1 2 swap swap +"),
        ("Unused computation", "1 2 + drop 5"),
        ("Complex dead code", "1 2 3 dup drop swap swap"),
    ];

    let eliminator = DeadCodeEliminator::new();

    for (name, code) in examples {
        println!("Example: {}", name);
        let ir = ForthIR::parse(code).unwrap();
        let optimized = eliminator.eliminate(&ir).unwrap();

        let stats = eliminator.get_stats(&ir, &optimized);
        println!("  Before: {} instructions", stats.before_instructions);
        println!("  After:  {} instructions", stats.after_instructions);
        println!("  Eliminated: {} instructions\n", stats.instructions_eliminated);
    }
}

fn demo_inlining() {
    println!("\n4. INLINING");
    println!("===========\n");

    let mut ir = ForthIR::new();

    // Define words
    let square = WordDef::new(
        "square".to_string(),
        vec![Instruction::Dup, Instruction::Mul],
    );
    ir.add_word(square);

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

    println!("Example: Nested inlining (5 quad)");
    println!("  Definition: quad = square square");
    println!("  Definition: square = dup *\n");

    for level in [
        OptimizationLevel::Basic,
        OptimizationLevel::Standard,
        OptimizationLevel::Aggressive,
    ]
    .iter()
    {
        let optimizer = InlineOptimizer::new(*level);
        let optimized = optimizer.inline(&ir).unwrap();
        let stats = optimizer.get_stats(&ir, &optimized);

        println!("  Level: {:?}", level);
        println!("    Calls before: {}", stats.calls_before);
        println!("    Calls after:  {}", stats.calls_after);
        println!("    Calls inlined: {}", stats.calls_inlined);
    }
    println!();
}

fn demo_stack_caching() {
    println!("\n5. STACK CACHING");
    println!("================\n");

    let code = "1 2 3 4 5 + + + +";
    println!("Example: Deep stack operations");
    println!("  Code: {}\n", code);

    let ir = ForthIR::parse(code).unwrap();

    for cache_size in [1, 2, 3, 4].iter() {
        let optimizer = StackCacheOptimizer::new(*cache_size);
        let optimized = optimizer.optimize(&ir).unwrap();

        println!("  Cache size: {} registers", cache_size);
        println!("    Instructions: {}", optimized.instruction_count());

        // Count cached operations
        let cached_count = optimized
            .main
            .iter()
            .filter(|i| {
                matches!(
                    i,
                    Instruction::CachedDup { .. }
                        | Instruction::CachedSwap { .. }
                        | Instruction::CachedOver { .. }
                )
            })
            .count();
        println!("    Cached operations: {}", cached_count);
    }
    println!();
}

fn demo_full_pipeline() {
    println!("\n6. FULL OPTIMIZATION PIPELINE");
    println!("==============================\n");

    let code = "5 dup + 1 + dup * 2 / 10 swap -";
    println!("Example: Complex expression");
    println!("  Code: {}\n", code);

    let ir = ForthIR::parse(code).unwrap();

    println!("  Original: {} instructions\n", ir.instruction_count());

    for level in [
        OptimizationLevel::None,
        OptimizationLevel::Basic,
        OptimizationLevel::Standard,
        OptimizationLevel::Aggressive,
    ]
    .iter()
    {
        let optimizer = Optimizer::new(*level);
        let optimized = optimizer.optimize(ir.clone()).unwrap();

        let reduction = ((ir.instruction_count() - optimized.instruction_count()) as f64
            / ir.instruction_count() as f64
            * 100.0);

        println!("  {:?}:", level);
        println!("    Instructions: {}", optimized.instruction_count());
        println!("    Reduction: {:.1}%", reduction);
    }

    println!("\n\n7. BEFORE/AFTER COMPARISON");
    println!("===========================\n");

    // Show a detailed before/after
    let detailed_example = "5 dup + 1 + 2 * dup drop";
    println!("Code: {}\n", detailed_example);

    let before = ForthIR::parse(detailed_example).unwrap();
    let optimizer = Optimizer::new(OptimizationLevel::Aggressive);
    let after = optimizer.optimize(before.clone()).unwrap();

    println!("BEFORE optimization:");
    println!("  Instructions: {}", before.instruction_count());
    for (i, inst) in before.main.iter().enumerate() {
        println!("    {}: {:?}", i, inst);
    }

    println!("\nAFTER optimization:");
    println!("  Instructions: {}", after.instruction_count());
    for (i, inst) in after.main.iter().enumerate() {
        println!("    {}: {:?}", i, inst);
    }

    let reduction = ((before.instruction_count() - after.instruction_count()) as f64
        / before.instruction_count() as f64
        * 100.0);
    println!("\n  Total reduction: {:.1}%", reduction);

    println!("\n\n8. PERFORMANCE TARGETS");
    println!("=======================\n");
    println!("  ✓ Stack caching: 2-3x speedup on stack-heavy code");
    println!("  ✓ Superinstructions: 20-30% code size reduction");
    println!("  ✓ Constant folding: Eliminates runtime overhead for literals");
    println!("  ✓ Combined optimizations: 80-100% of hand-written C");
    println!("\nRun benchmarks with: cargo bench");
}
