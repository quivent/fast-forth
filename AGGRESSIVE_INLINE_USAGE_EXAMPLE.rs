// Aggressive Inlining Engine Usage Examples
// Location: FastForth optimizer library
// This file demonstrates how to use the AggressiveInlineOptimizer

use fastforth_optimizer::{
    ForthIR, Instruction, WordDef,
    AggressiveInlineOptimizer, OptimizationLevel, CallGraph,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Example 1: Basic Inlining
    println!("Example 1: Basic Inlining");
    example_basic_inlining()?;

    // Example 2: Multi-Level Inlining
    println!("\nExample 2: Multi-Level Inlining");
    example_multi_level_inlining()?;

    // Example 3: Cycle Detection
    println!("\nExample 3: Cycle Detection");
    example_cycle_detection()?;

    // Example 4: Statistics
    println!("\nExample 4: Statistics Collection");
    example_statistics()?;

    // Example 5: Different Optimization Levels
    println!("\nExample 5: Optimization Levels");
    example_optimization_levels()?;

    Ok(())
}

/// Example 1: Basic inlining of a simple helper function
fn example_basic_inlining() -> Result<(), Box<dyn std::error::Error>> {
    let mut ir = ForthIR::new();

    // Define a small helper function: : double 2 * ;
    let double = WordDef::new(
        "double".to_string(),
        vec![
            Instruction::Literal(2),
            Instruction::Mul,
        ],
    );
    ir.add_word(double);

    // Main code: 5 double double 1 +
    ir.main = vec![
        Instruction::Literal(5),
        Instruction::Call("double".to_string()),
        Instruction::Call("double".to_string()),
        Instruction::Literal(1),
        Instruction::Add,
    ];

    println!("Original main: {:?}", ir.main);
    println!("Original call count: {}",
        ir.main.iter().filter(|i| matches!(i, Instruction::Call(_))).count());

    // Apply aggressive inlining at Standard level
    let optimizer = AggressiveInlineOptimizer::new(OptimizationLevel::Standard);
    let optimized = optimizer.inline(&ir)?;

    println!("Optimized main: {:?}", optimized.main);
    println!("Optimized call count: {}",
        optimized.main.iter().filter(|i| matches!(i, Instruction::Call(_))).count());

    let stats = optimizer.get_stats(&ir, &optimized);
    println!("Stats:\n{}", stats);

    Ok(())
}

/// Example 2: Multi-level inlining through call chains
fn example_multi_level_inlining() -> Result<(), Box<dyn std::error::Error>> {
    let mut ir = ForthIR::new();

    // Define chain: level3 → level2 → level1 → main

    // : level3 dup ;  (1 instruction)
    let level3 = WordDef::new(
        "level3".to_string(),
        vec![Instruction::Dup],
    );

    // : level2 level3 ;  (1 instruction, expands to 1)
    let level2 = WordDef::new(
        "level2".to_string(),
        vec![Instruction::Call("level3".to_string())],
    );

    // : level1 level2 ;  (1 instruction, expands to 1)
    let level1 = WordDef::new(
        "level1".to_string(),
        vec![Instruction::Call("level2".to_string())],
    );

    ir.add_word(level3);
    ir.add_word(level2);
    ir.add_word(level1);

    // Main: 5 level1
    ir.main = vec![
        Instruction::Literal(5),
        Instruction::Call("level1".to_string()),
    ];

    println!("Original IR: 1 main call, 3 word definitions");

    // Apply aggressive inlining
    let optimizer = AggressiveInlineOptimizer::new(OptimizationLevel::Aggressive);
    let optimized = optimizer.inline(&ir)?;

    // Check if all calls were inlined
    let remaining_calls = optimized.main.iter()
        .filter(|i| matches!(i, Instruction::Call(_)))
        .count();

    println!("After inlining: {} calls remain in main", remaining_calls);
    println!("Optimized main should contain just: [Literal(5), Dup]");

    Ok(())
}

/// Example 3: Cycle detection prevents infinite inlining
fn example_cycle_detection() -> Result<(), Box<dyn std::error::Error>> {
    let mut ir = ForthIR::new();

    // Define recursive function: : factorial dup 1 > if dup 1 - factorial * then ;
    // For simplicity, we'll use a direct recursive call
    let factorial = WordDef::new(
        "factorial".to_string(),
        vec![
            Instruction::Dup,
            Instruction::Literal(1),
            Instruction::Gt,
            Instruction::Call("factorial".to_string()), // Recursive call
        ],
    );

    ir.add_word(factorial);
    ir.main = vec![
        Instruction::Literal(5),
        Instruction::Call("factorial".to_string()),
    ];

    // Build call graph to detect cycles
    let call_graph = CallGraph::build(&ir);
    let cycles = call_graph.find_cycles();

    println!("Detected {} cycle(s)", cycles.len());
    if !cycles.is_empty() {
        println!("Cycle contains: {:?}", cycles[0]);
    }

    // Attempt inlining (should preserve recursive call)
    let optimizer = AggressiveInlineOptimizer::new(OptimizationLevel::Aggressive);
    let optimized = optimizer.inline(&ir)?;

    let has_call = optimized.main.iter().any(|i| matches!(i, Instruction::Call(_)));
    println!("Recursive call preserved: {}", has_call);

    Ok(())
}

/// Example 4: Collecting and analyzing statistics
fn example_statistics() -> Result<(), Box<dyn std::error::Error>> {
    let mut ir = ForthIR::new();

    // Create a realistic program with multiple helpers
    let add1 = WordDef::new("add1".to_string(), vec![Instruction::Literal(1), Instruction::Add]);
    let mul2 = WordDef::new("mul2".to_string(), vec![Instruction::Literal(2), Instruction::Mul]);
    let quad = WordDef::new(
        "quad".to_string(),
        vec![
            Instruction::Call("mul2".to_string()),
            Instruction::Call("mul2".to_string()),
        ],
    );

    ir.add_word(add1);
    ir.add_word(mul2);
    ir.add_word(quad);

    ir.main = vec![
        Instruction::Literal(5),
        Instruction::Call("quad".to_string()),
        Instruction::Call("add1".to_string()),
        Instruction::Call("add1".to_string()),
    ];

    let original_size = ir.instruction_count();
    println!("Original program size: {} instructions", original_size);

    // Optimize at different levels
    for level in &[OptimizationLevel::Basic, OptimizationLevel::Standard, OptimizationLevel::Aggressive] {
        let optimizer = AggressiveInlineOptimizer::new(*level);
        let optimized = optimizer.inline(&ir)?;
        let stats = optimizer.get_stats(&ir, &optimized);

        println!("\nOptimization Level: {:?}", level);
        println!("{}", stats);
    }

    Ok(())
}

/// Example 5: Impact of different optimization levels
fn example_optimization_levels() -> Result<(), Box<dyn std::error::Error>> {
    let mut ir = ForthIR::new();

    // Create a program with varying function sizes
    let tiny = WordDef::new("tiny".to_string(), vec![Instruction::Dup]);
    let small = WordDef::new(
        "small".to_string(),
        vec![Instruction::Dup, Instruction::Add],
    );
    let medium = WordDef::new(
        "medium".to_string(),
        vec![
            Instruction::Dup, Instruction::Add, Instruction::Mul, Instruction::Sub,
        ],
    );

    ir.add_word(tiny);
    ir.add_word(small);
    ir.add_word(medium);

    // Call all three
    ir.main = vec![
        Instruction::Literal(5),
        Instruction::Call("tiny".to_string()),
        Instruction::Call("small".to_string()),
        Instruction::Call("medium".to_string()),
    ];

    println!("Original main calls: 3");
    println!("\nLet's see what gets inlined at each level:");

    let levels = vec![
        ("None", OptimizationLevel::None),
        ("Basic", OptimizationLevel::Basic),
        ("Standard", OptimizationLevel::Standard),
        ("Aggressive", OptimizationLevel::Aggressive),
    ];

    for (name, level) in levels {
        let optimizer = AggressiveInlineOptimizer::new(level);
        let optimized = optimizer.inline(&ir)?;

        let remaining_calls = optimized.main.iter()
            .filter(|i| matches!(i, Instruction::Call(_)))
            .count();

        let inlined = 3 - remaining_calls;

        println!("\n{:12} → Inlined: {:?}, Remaining: {}", name,
            if inlined > 0 { format!("{} calls", inlined) } else { "none".to_string() },
            remaining_calls);
    }

    Ok(())
}

// ============================================================================
// Advanced Usage Patterns
// ============================================================================

/// Pattern 1: Using INLINE directive for forced inlining
#[allow(dead_code)]
fn pattern_forced_inline() {
    // In your Forth code:
    // : expensive-computation INLINE ... lots of code ... ;
    //
    // The optimizer will inline this function regardless of size,
    // overriding normal cost thresholds.

    // In Rust, this is represented as:
    let mut expensive = WordDef::new(
        "expensive-computation".to_string(),
        vec![
            Instruction::Dup, Instruction::Dup, Instruction::Mul,
            Instruction::Dup, Instruction::Mul, Instruction::Dup,
            Instruction::Mul, Instruction::Add, Instruction::Add,
        ],
    );
    expensive.is_inline = true; // Set INLINE flag

    // The optimizer will now inline this large function
}

/// Pattern 2: Analyzing call graph structure
#[allow(dead_code)]
fn pattern_analyze_call_graph() -> Result<(), Box<dyn std::error::Error>> {
    let mut ir = ForthIR::new();

    // Build your IR...
    let a = WordDef::new("a".to_string(), vec![Instruction::Dup]);
    let b = WordDef::new("b".to_string(), vec![Instruction::Call("a".to_string())]);
    ir.add_word(a);
    ir.add_word(b);

    let call_graph = CallGraph::build(&ir);

    // Analyze structure
    let callees_of_b = call_graph.get_callees("b");
    let call_count = call_graph.get_call_count("b", "a");
    let cycles = call_graph.find_cycles();

    println!("b calls: {:?}", callees_of_b);
    println!("b → a call count: {}", call_count);
    println!("Cycles detected: {}", cycles.len());

    Ok(())
}

/// Pattern 3: Iterative optimization with statistics
#[allow(dead_code)]
fn pattern_iterative_optimization() -> Result<(), Box<dyn std::error::Error>> {
    let mut ir = ForthIR::new();

    // Build IR...

    // Apply optimization and track improvements
    let optimizer = AggressiveInlineOptimizer::new(OptimizationLevel::Aggressive);
    let optimized = optimizer.inline(&ir)?;
    let stats = optimizer.get_stats(&ir, &optimized);

    // Make decisions based on statistics
    if stats.code_bloat_factor > 2.5 {
        println!("Warning: Code bloat factor is {}", stats.code_bloat_factor);
    }

    if stats.calls_inlined > stats.calls_before / 2 {
        println!("Good: Inlined more than 50% of calls");
    }

    Ok(())
}

// ============================================================================
// Expected Output
// ============================================================================

/*
Example 1: Basic Inlining
Original main: [Literal(5), Call("double"), Call("double"), Literal(1), Add]
Original call count: 2
Optimized main: [Literal(5), Literal(2), Mul, Literal(2), Mul, Literal(1), Add]
Optimized call count: 0
Stats:
Aggressive Inline Statistics:
├─ Calls before:        2
├─ Calls after:         0
├─ Calls inlined:       2 (100.0%)
├─ Instructions before: 5
├─ Instructions after:  7
├─ Code bloat:          1.40x
├─ Cycles detected:     0
├─ Cycles remaining:    0
├─ Words before:        1
└─ Words after:         1

Example 2: Multi-Level Inlining
Original IR: 1 main call, 3 word definitions
After inlining: 0 calls remain in main
Optimized main should contain just: [Literal(5), Dup]

Example 3: Cycle Detection
Detected 1 cycle(s)
Cycle contains: {"factorial"}
Recursive call preserved: true

Example 4: Statistics Collection
Original program size: 11 instructions

Optimization Level: Basic
... (statistics)

Optimization Level: Standard
... (statistics)

Optimization Level: Aggressive
... (statistics)

Example 5: Optimization Levels
Original main calls: 3

Let's see what gets inlined at each level:

None         → Inlined: none, Remaining: 3
Basic        → Inlined: "2 calls", Remaining: 1
Standard     → Inlined: "3 calls", Remaining: 0
Aggressive   → Inlined: "3 calls", Remaining: 0
*/
