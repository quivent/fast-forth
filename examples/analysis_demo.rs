//! Data flow analysis demonstration
//!
//! Shows how analysis passes provide insights for optimization.

use fastforth_optimizer::*;
use fastforth_optimizer::analysis::*;

fn main() {
    println!("FastForth Analysis Demo\n");
    println!("=======================\n");

    demo_stack_depth_analysis();
    demo_reaching_definitions();
}

fn demo_stack_depth_analysis() {
    println!("1. Stack Depth Analysis");
    println!("=======================\n");

    let examples = vec![
        ("Simple", "1 2 +"),
        ("Deep stack", "1 2 3 4 5 + + + +"),
        ("With manipulation", "1 2 3 swap drop over"),
    ];

    for (name, code) in examples {
        println!("Example: {}", name);
        println!("  Code: {}\n", code);

        let ir = ForthIR::parse(code).unwrap();
        let analysis = StackDepthAnalysis::analyze(&ir.main);

        println!("  Stack depth progression:");
        for (i, inst) in ir.main.iter().enumerate() {
            let depth = analysis.get_depth(i).unwrap_or(0);
            println!("    {}: {:?} (depth: {})", i, inst, depth);
        }

        println!("  Maximum depth: {}\n", analysis.max_depth);
    }
}

fn demo_reaching_definitions() {
    println!("\n2. Reaching Definitions Analysis");
    println!("=================================\n");

    let code = "1 2 3 + *";
    println!("Example: {}\n", code);

    let ir = ForthIR::parse(code).unwrap();
    let analysis = ReachingDefinitions::analyze(&ir.main);

    println!("Reaching definitions at each instruction:");
    for (i, inst) in ir.main.iter().enumerate() {
        let reaching = analysis.get_reaching(i).unwrap();
        println!("  {}: {:?}", i, inst);
        println!("     Reaching: {:?}", reaching);
    }

    println!("\nThis analysis helps identify:");
    println!("  - Which values are used where");
    println!("  - Opportunities for dead code elimination");
    println!("  - Stack value lifetime");
}
