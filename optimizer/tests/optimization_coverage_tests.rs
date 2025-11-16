//! Optimization Pass Coverage Tests
//!
//! Targets uncovered code paths in optimization passes including:
//! - Loop invariant code motion
//! - Common subexpression elimination
//! - Algebraic simplification
//! - Strength reduction
//! - Peephole optimization

use fastforth_optimizer::{
    ForthIR, Instruction, Optimizer, OptimizationLevel, StackEffect,
};

#[test]
fn test_peephole_dup_mul_to_square() {
    let mut optimizer = Optimizer::new(OptimizationLevel::Aggressive);
    let mut ir = ForthIR::new();

    // Pattern: 5 dup * -> should recognize dup mul pattern
    ir.main.push(Instruction::Literal(5));
    ir.main.push(Instruction::Dup);
    ir.main.push(Instruction::Mul);

    let optimized = optimizer.optimize(ir).unwrap();

    // Should optimize to DupMul superinstruction or fold to 25
    let has_dupmul = optimized.main.iter().any(|i| matches!(i, Instruction::DupMul));
    let has_25 = optimized.main.iter().any(|i| matches!(i, Instruction::Literal(25)));
    assert!(has_dupmul || has_25, "Should recognize dup mul pattern or fold");
}

#[test]
fn test_peephole_literal_zero_mul() {
    let mut optimizer = Optimizer::new(OptimizationLevel::Aggressive);
    let mut ir = ForthIR::new();

    // Pattern: 5 0 * -> should optimize to 0
    ir.main.push(Instruction::Literal(5));
    ir.main.push(Instruction::Literal(0));
    ir.main.push(Instruction::Mul);

    let optimized = optimizer.optimize(ir).unwrap();

    // Should optimize to just 0 (constant folding)
    assert!(optimized.main.len() <= 2, "Should optimize multiplication by zero");
    assert!(optimized.main.iter().any(|i| matches!(i, Instruction::Literal(0))));
}

#[test]
fn test_peephole_literal_one_mul() {
    let mut optimizer = Optimizer::new(OptimizationLevel::Aggressive);
    let mut ir = ForthIR::new();

    // Pattern: 5 1 * -> should simplify to 5
    ir.main.push(Instruction::Literal(5));
    ir.main.push(Instruction::Literal(1));
    ir.main.push(Instruction::Mul);

    let optimized = optimizer.optimize(ir).unwrap();

    // Should constant fold or eliminate identity operation
    assert!(optimized.main.len() <= 2, "Should optimize multiplication by one");
}

#[test]
fn test_peephole_literal_add_fusion() {
    let mut optimizer = Optimizer::new(OptimizationLevel::Aggressive);
    let mut ir = ForthIR::new();

    // Pattern: 10 42 + -> should constant fold or create LiteralAdd
    ir.main.push(Instruction::Literal(10));
    ir.main.push(Instruction::Literal(42));
    ir.main.push(Instruction::Add);

    let optimized = optimizer.optimize(ir).unwrap();

    // Should fold to 52 or create LiteralAdd superinstruction
    let has_literal_add = optimized.main.iter().any(|i| matches!(i, Instruction::LiteralAdd(_)));
    let has_52 = optimized.main.iter().any(|i| matches!(i, Instruction::Literal(52)));
    assert!(has_literal_add || has_52, "Should optimize addition");
}

#[test]
fn test_strength_reduction_mul_two() {
    let mut optimizer = Optimizer::new(OptimizationLevel::Aggressive);
    let mut ir = ForthIR::new();

    // Pattern: 5 2 * -> should optimize to shift or constant fold to 10
    ir.main.push(Instruction::Literal(5));
    ir.main.push(Instruction::Literal(2));
    ir.main.push(Instruction::Mul);

    let optimized = optimizer.optimize(ir).unwrap();

    // Should constant fold to 10 or use MulTwo/shift
    let has_10 = optimized.main.iter().any(|i| matches!(i, Instruction::Literal(10)));
    let has_optimization = optimized.main.iter().any(|i| matches!(i, Instruction::MulTwo | Instruction::Shl));
    assert!(has_10 || has_optimization || optimized.main.len() < 3, "Should optimize multiplication by 2");
}

#[test]
fn test_strength_reduction_div_two() {
    let mut optimizer = Optimizer::new(OptimizationLevel::Aggressive);
    let mut ir = ForthIR::new();

    // Pattern: 10 2 / -> should optimize to shift or constant fold to 5
    ir.main.push(Instruction::Literal(10));
    ir.main.push(Instruction::Literal(2));
    ir.main.push(Instruction::Div);

    let optimized = optimizer.optimize(ir).unwrap();

    // Should constant fold to 5 or use DivTwo/shift
    let has_5 = optimized.main.iter().any(|i| matches!(i, Instruction::Literal(5)));
    let has_optimization = optimized.main.iter().any(|i| matches!(i, Instruction::DivTwo | Instruction::Shr));
    assert!(has_5 || has_optimization || optimized.main.len() < 3, "Should optimize division by 2");
}

#[test]
fn test_algebraic_simplification_add_zero() {
    let mut optimizer = Optimizer::new(OptimizationLevel::Aggressive);
    let mut ir = ForthIR::new();

    // Pattern: 5 0 + -> should simplify to 5
    ir.main.push(Instruction::Literal(5));
    ir.main.push(Instruction::Literal(0));
    ir.main.push(Instruction::Add);

    let optimized = optimizer.optimize(ir).unwrap();

    // Should constant fold to 5 or eliminate addition by zero
    let has_5 = optimized.main.iter().any(|i| matches!(i, Instruction::Literal(5)));
    assert!(has_5 || optimized.main.len() == 1, "Should optimize addition by zero");
}

#[test]
fn test_dead_code_elimination_unused_literal() {
    let mut optimizer = Optimizer::new(OptimizationLevel::Aggressive);
    let mut ir = ForthIR::new();

    // Unused literal followed by drop
    ir.main.push(Instruction::Literal(42));
    ir.main.push(Instruction::Drop);

    let optimized = optimizer.optimize(ir).unwrap();

    // Both should be eliminated as dead code
    assert!(!optimized.main.iter().any(|i| matches!(i, Instruction::Literal(42))));
    assert!(!optimized.main.iter().any(|i| matches!(i, Instruction::Drop)));
}

#[test]
fn test_common_subexpression_elimination() {
    let mut optimizer = Optimizer::new(OptimizationLevel::Aggressive);
    let mut ir = ForthIR::new();

    // Computation: 5 dup * 3 dup * (two square operations)
    ir.main.push(Instruction::Literal(5));
    ir.main.push(Instruction::Dup);
    ir.main.push(Instruction::Mul);  // 25
    ir.main.push(Instruction::Literal(3));
    ir.main.push(Instruction::Dup);
    ir.main.push(Instruction::Mul);  // 9
    ir.main.push(Instruction::Add);  // 34

    let optimized = optimizer.optimize(ir).unwrap();

    // Should optimize through constant folding and superinstructions
    // Final result should be heavily optimized
    assert!(optimized.main.len() <= 3 || optimized.main.iter().any(|i| matches!(i, Instruction::Literal(34))),
            "Should optimize redundant patterns");
}

#[test]
fn test_stack_effect_composition() {
    // Test stack effect algebra
    let effect1 = StackEffect::new(2, 1); // ( a b -- c )
    let effect2 = StackEffect::new(1, 2); // ( c -- d e )

    let composed = effect1.compose(&effect2);

    // ( a b -- d e ) net: consumes 2, produces 2
    assert_eq!(composed.consumed, 2);
    assert_eq!(composed.produced, 2);
}
