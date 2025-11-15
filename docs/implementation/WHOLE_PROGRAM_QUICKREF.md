# Whole-Program Optimization Quick Reference

## Quick Start

```rust
use fastforth_optimizer::{ForthIR, WholeProgramOptimizer, OptimizationLevel};

// Create optimizer
let optimizer = WholeProgramOptimizer::new(OptimizationLevel::Aggressive);

// Optimize IR
let optimized_ir = optimizer.optimize(&ir)?;

// Get statistics
let stats = optimizer.get_stats(&ir, &optimized_ir);
println!("{}", stats);
```

## Optimization Levels

| Level | Dead Code | Constants | Inlining | Specialization | Max Cost |
|-------|-----------|-----------|----------|----------------|----------|
| Basic | Yes | No | No | No | 10 |
| Standard | Yes | Yes | Yes | No | 20 |
| Aggressive | Yes | Yes | Yes | Yes | 50 |

## Call Graph Analysis

```rust
// Build call graph
let call_graph = CallGraph::build(&ir);

// Find unreachable words
let dead = call_graph.find_unreachable();

// Check for recursion
if call_graph.is_recursive("my_word") {
    println!("Recursive word detected");
}

// Find single-call words
let inlineable = call_graph.find_single_call_words();

// Get topological order
let topo_order = call_graph.topological_order();

// Call frequency
let count = call_graph.get_call_count("my_word");
```

## Side Effects Analysis

```rust
// Analyze side effects
let side_effects = call_graph.analyze_side_effects(&ir);

for (word, has_effects) in side_effects {
    if has_effects {
        println!("{} modifies state", word);
    }
}
```

## Inlineability Scoring

```rust
// Score word for inlining
let score = call_graph.inlineability_score(word, &call_graph);

if score > 70 {
    println!("Good candidate for inlining");
}
```

## Configuration

```rust
let mut optimizer = WholeProgramOptimizer::new(OptimizationLevel::Standard);

// Adjust inline threshold
optimizer.set_max_inline_cost(25);

// Enable/disable specialization
optimizer.set_aggressive_specialization(true);

// Run optimization
let result = optimizer.optimize(&ir)?;
```

## Statistics

```rust
let stats = optimizer.get_stats(&before_ir, &after_ir);

println!("Words before: {}", stats.words_before);
println!("Words after: {}", stats.words_after);
println!("Words eliminated: {}", stats.words_eliminated);
println!("Code reduction: {:.1}%", stats.code_size_reduction);
println!("Instructions: {} -> {}",
         stats.instructions_before,
         stats.instructions_after);
```

## Constant Propagation

The optimizer automatically propagates constants through:
- Arithmetic operations (Add, Sub, Mul, Div, Mod)
- Unary operations (Neg, Abs)
- Stack operations (Dup, Drop, Swap)

Example:
```forth
Before:  2 3 + 4 * .
After:   20 .
```

## Dead Code Elimination

Automatic removal of unreachable code:

```forth
Before:
: unused  ... ;  \ never called
: main  ... ;

After:
: main  ... ;     \ unused removed
```

## Single-Call Inlining

Words called exactly once are automatically inlined:

```forth
Before:
: helper  dup + ;
: main  5 helper . ;

After:
: main  5 dup + . ;  \ helper inlined and removed
```

## Word Specialization

Aggressive optimization creates specialized versions:

```forth
Before:
: addN  swap + ;
: main  10 5 addN drop  20 5 addN drop ;

After:
: addN__specialized_5  swap + ;
: main
  10 5 addN__specialized_5 drop
  20 5 addN__specialized_5 drop
;
```

## Common Patterns

### Measure Optimization Impact

```rust
let before_size = ir.instruction_count();
let optimized = optimizer.optimize(&ir)?;
let after_size = optimized.instruction_count();
let reduction = (before_size - after_size) as f64 / before_size as f64 * 100.0;
println!("Code reduction: {:.1}%", reduction);
```

### Debug Optimization

```rust
let call_graph = CallGraph::build(&ir);
println!("Call graph nodes: {}", call_graph.graph.node_count());

let unreachable = call_graph.find_unreachable();
println!("Dead code: {}", unreachable.join(", "));

let topo = call_graph.topological_order();
println!("Optimization order: {}", topo.join(" -> "));
```

### Profile Inlineability

```rust
for (name, word) in &ir.words {
    let score = call_graph.inlineability_score(word, &call_graph);
    let count = call_graph.get_call_count(name);
    println!("{}: cost={} calls={} score={}",
             name, word.cost, count, score);
}
```

## Performance Tips

1. **Use Aggressive mode** for maximum optimization
2. **Adjust max_inline_cost** based on target performance
3. **Run multiple passes** for iterative optimization:
   ```rust
   let mut current = ir;
   for _ in 0..5 {
       current = optimizer.optimize(current)?;
   }
   ```
4. **Combine with other passes** for best results
5. **Profile-guided** decisions in performance-critical code

## Error Handling

```rust
match optimizer.optimize(&ir) {
    Ok(optimized) => {
        // Successful optimization
        let stats = optimizer.get_stats(&ir, &optimized);
        println!("Optimization complete: {}", stats);
    }
    Err(e) => {
        // Handle error
        eprintln!("Optimization failed: {}", e);
    }
}
```

## Integration with Pipeline

The whole_program optimizer works as a standalone pass or can be integrated:

```rust
// Standalone
let wpo = WholeProgramOptimizer::new(OptimizationLevel::Aggressive);
let ir = wpo.optimize(&ir)?;

// In pipeline (future integration)
let optimizer = Optimizer::new(OptimizationLevel::Aggressive);
let ir = optimizer.optimize(&ir)?;  // Includes WPO
```

## Limitations

- Conservative constant propagation (clears on unknown calls)
- No cross-procedure value numbering
- No interprocedural register allocation
- Recursive functions are not specialized
- Large words (cost > 50) not inlined aggressively

## See Also

- Implementation: `optimizer/src/whole_program.rs`
- Full documentation: `WHOLE_PROGRAM_OPTIMIZATION_SUMMARY.md`
- IR module: `optimizer/src/ir.rs`
- Optimization levels: `OptimizationLevel` enum in `lib.rs`
