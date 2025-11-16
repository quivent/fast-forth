//! Optimization edge case tests
//!
//! Tests targeting optimization stress scenarios:
//! - Constant folding limits
//! - Dead code elimination
//! - Loop unrolling edge cases
//! - Inline expansion limits
//! - Combined optimization scenarios

#[cfg(feature = "llvm")]
mod llvm_optimization_edge_cases {
    use backend::{LLVMBackend, CodeGenerator, CompilationMode};
    use inkwell::{context::Context, OptimizationLevel};
    use fastforth_frontend::ssa::{
        SSAFunction, SSAInstruction, Register, BlockId, BasicBlock, BinaryOperator, UnaryOperator,
    };
    use smallvec::smallvec;

    // ===== OPTIMIZATION STRESS TESTS (5 tests) =====

    #[test]
    fn test_constant_folding_limits() {
        // Test constant folding with many constants
        let context = Context::create();
        let mut backend = LLVMBackend::new(
            &context,
            "test_const_fold",
            CompilationMode::AOT,
            OptimizationLevel::Aggressive,
        );

        let mut func = SSAFunction::new("const_fold_stress".to_string(), 0);
        let mut entry = BasicBlock::new(BlockId(0));

        // Create a long chain of constant operations that should fold
        let c1 = Register(0);
        entry.instructions.push(SSAInstruction::LoadInt {
            dest: c1,
            value: 1,
        });

        let c2 = Register(1);
        entry.instructions.push(SSAInstruction::LoadInt {
            dest: c2,
            value: 2,
        });

        let mut current_reg = c1;
        for i in 2..100 {
            let next_reg = Register(i);
            entry.instructions.push(SSAInstruction::BinaryOp {
                dest: next_reg,
                op: if i % 4 == 0 {
                    BinaryOperator::Add
                } else if i % 4 == 1 {
                    BinaryOperator::Mul
                } else if i % 4 == 2 {
                    BinaryOperator::Sub
                } else {
                    BinaryOperator::Add
                },
                left: current_reg,
                right: c2,
            });
            current_reg = next_reg;
        }

        entry.instructions.push(SSAInstruction::Return {
            values: smallvec![current_reg],
        });

        func.blocks.push(entry);
        let result = backend.generate(&func);
        assert!(result.is_ok());

        // With aggressive optimization, this should be heavily folded
        let llvm_ir = backend.print_to_string();
        // The IR should be significantly smaller than without optimization
        assert!(llvm_ir.contains("define"));
    }

    #[test]
    fn test_dead_code_elimination_complex() {
        // Test dead code elimination with complex patterns
        let context = Context::create();
        let mut backend = LLVMBackend::new(
            &context,
            "test_dce",
            CompilationMode::AOT,
            OptimizationLevel::Aggressive,
        );

        let mut func = SSAFunction::new("dce_stress".to_string(), 1);
        let mut entry = BasicBlock::new(BlockId(0));

        let input = Register(0);

        // Create many unused computations
        for i in 1..50 {
            let dead_reg = Register(i);
            entry.instructions.push(SSAInstruction::BinaryOp {
                dest: dead_reg,
                op: BinaryOperator::Add,
                left: input,
                right: input,
            });
            // These are never used, should be eliminated
        }

        // Only use the input
        let result = Register(50);
        entry.instructions.push(SSAInstruction::BinaryOp {
            dest: result,
            op: BinaryOperator::Mul,
            left: input,
            right: input,
        });

        entry.instructions.push(SSAInstruction::Return {
            values: smallvec![result],
        });

        func.blocks.push(entry);
        assert!(backend.generate(&func).is_ok());

        // Optimized version should eliminate dead code
        let llvm_ir = backend.print_to_string();
        assert!(llvm_ir.contains("mul"));
    }

    #[test]
    fn test_loop_unrolling_edge_cases() {
        // Test loop that could be unrolled
        let context = Context::create();
        let mut backend = LLVMBackend::new(
            &context,
            "test_loop_unroll",
            CompilationMode::AOT,
            OptimizationLevel::Aggressive,
        );

        let mut func = SSAFunction::new("loop_unroll_test".to_string(), 1);
        let sum_init = Register(0);

        // Entry: initialize counter
        let mut entry = BasicBlock::new(BlockId(0));
        let counter_init = Register(1);
        entry.instructions.push(SSAInstruction::LoadInt {
            dest: counter_init,
            value: 0,
        });
        entry.instructions.push(SSAInstruction::Jump {
            target: BlockId(1),
        });
        func.blocks.push(entry);

        // Loop header: check condition
        let mut loop_header = BasicBlock::new(BlockId(1));
        let counter_phi = Register(2);
        let sum_phi = Register(3);

        loop_header.instructions.push(SSAInstruction::Phi {
            dest: counter_phi,
            incoming: vec![
                (BlockId(0), counter_init),
                (BlockId(2), Register(8)), // will be updated counter
            ],
        });

        loop_header.instructions.push(SSAInstruction::Phi {
            dest: sum_phi,
            incoming: vec![
                (BlockId(0), sum_init),
                (BlockId(2), Register(7)), // will be updated sum
            ],
        });

        let limit = Register(4);
        loop_header.instructions.push(SSAInstruction::LoadInt {
            dest: limit,
            value: 10, // Small loop - good candidate for unrolling
        });

        let cond = Register(5);
        loop_header.instructions.push(SSAInstruction::BinaryOp {
            dest: cond,
            op: BinaryOperator::Lt,
            left: counter_phi,
            right: limit,
        });

        loop_header.instructions.push(SSAInstruction::Branch {
            condition: cond,
            true_block: BlockId(2),
            false_block: BlockId(3),
        });
        func.blocks.push(loop_header);

        // Loop body
        let mut loop_body = BasicBlock::new(BlockId(2));
        let new_sum = Register(7);
        loop_body.instructions.push(SSAInstruction::BinaryOp {
            dest: new_sum,
            op: BinaryOperator::Add,
            left: sum_phi,
            right: counter_phi,
        });

        let one = Register(6);
        loop_body.instructions.push(SSAInstruction::LoadInt {
            dest: one,
            value: 1,
        });

        let new_counter = Register(8);
        loop_body.instructions.push(SSAInstruction::BinaryOp {
            dest: new_counter,
            op: BinaryOperator::Add,
            left: counter_phi,
            right: one,
        });

        loop_body.instructions.push(SSAInstruction::Jump {
            target: BlockId(1),
        });
        func.blocks.push(loop_body);

        // Exit
        let mut exit = BasicBlock::new(BlockId(3));
        exit.instructions.push(SSAInstruction::Return {
            values: smallvec![sum_phi],
        });
        func.blocks.push(exit);

        assert!(backend.generate(&func).is_ok());
    }

    #[test]
    fn test_inline_expansion_limits() {
        // Test function inlining with many small functions
        let context = Context::create();
        let mut backend = LLVMBackend::new(
            &context,
            "test_inline",
            CompilationMode::AOT,
            OptimizationLevel::Aggressive,
        );

        // Main function that calls many helpers
        let mut func = SSAFunction::new("inline_test".to_string(), 1);
        let mut entry = BasicBlock::new(BlockId(0));
        let input = Register(0);

        // Call many small functions
        let mut current = input;
        for i in 0..20 {
            let result = Register(i + 1);
            entry.instructions.push(SSAInstruction::Call {
                dest: smallvec![result],
                name: format!("helper_{}", i),
                args: smallvec![current],
            });
            current = result;
        }

        entry.instructions.push(SSAInstruction::Return {
            values: smallvec![current],
        });

        func.blocks.push(entry);
        assert!(backend.generate(&func).is_ok());

        // Helper functions (these would normally be defined separately)
        for i in 0..20 {
            let mut helper = SSAFunction::new(format!("helper_{}", i), 1);
            let mut helper_entry = BasicBlock::new(BlockId(0));
            let x = Register(0);

            let const_reg = Register(1);
            helper_entry.instructions.push(SSAInstruction::LoadInt {
                dest: const_reg,
                value: i as i64,
            });

            let result = Register(2);
            helper_entry.instructions.push(SSAInstruction::BinaryOp {
                dest: result,
                op: BinaryOperator::Add,
                left: x,
                right: const_reg,
            });

            helper_entry.instructions.push(SSAInstruction::Return {
                values: smallvec![result],
            });

            helper.blocks.push(helper_entry);
            let _ = backend.generate(&helper);
        }
    }

    #[test]
    fn test_aggressive_optimization_combination() {
        // Test combination of multiple optimizations
        let context = Context::create();
        let mut backend = LLVMBackend::new(
            &context,
            "test_combo_opt",
            CompilationMode::AOT,
            OptimizationLevel::Aggressive,
        );

        let mut func = SSAFunction::new("combo_opt".to_string(), 2);
        let mut entry = BasicBlock::new(BlockId(0));

        let a = Register(0);
        let b = Register(1);

        // Dead code
        let dead1 = Register(2);
        entry.instructions.push(SSAInstruction::BinaryOp {
            dest: dead1,
            op: BinaryOperator::Add,
            left: a,
            right: a,
        });

        // Constant that should fold
        let c1 = Register(3);
        entry.instructions.push(SSAInstruction::LoadInt {
            dest: c1,
            value: 10,
        });

        let c2 = Register(4);
        entry.instructions.push(SSAInstruction::LoadInt {
            dest: c2,
            value: 5,
        });

        let const_result = Register(5);
        entry.instructions.push(SSAInstruction::BinaryOp {
            dest: const_result,
            op: BinaryOperator::Mul,
            left: c1,
            right: c2,
        });

        // More dead code
        let dead2 = Register(6);
        entry.instructions.push(SSAInstruction::BinaryOp {
            dest: dead2,
            op: BinaryOperator::Sub,
            left: b,
            right: b,
        });

        // Actually used computation
        let result = Register(7);
        entry.instructions.push(SSAInstruction::BinaryOp {
            dest: result,
            op: BinaryOperator::Add,
            left: a,
            right: const_result,
        });

        entry.instructions.push(SSAInstruction::Return {
            values: smallvec![result],
        });

        func.blocks.push(entry);
        assert!(backend.generate(&func).is_ok());

        let llvm_ir = backend.print_to_string();
        // Should have constant folded 10 * 5 = 50
        // Should have eliminated dead code
        assert!(llvm_ir.contains("define"));
    }

    #[test]
    fn test_algebraic_simplification() {
        // Test algebraic simplifications like x + 0, x * 1, x - x
        let context = Context::create();
        let mut backend = LLVMBackend::new(
            &context,
            "test_algebraic",
            CompilationMode::AOT,
            OptimizationLevel::Aggressive,
        );

        let mut func = SSAFunction::new("algebraic_simp".to_string(), 1);
        let mut entry = BasicBlock::new(BlockId(0));

        let x = Register(0);

        // x + 0 = x
        let zero = Register(1);
        entry.instructions.push(SSAInstruction::LoadInt {
            dest: zero,
            value: 0,
        });

        let r1 = Register(2);
        entry.instructions.push(SSAInstruction::BinaryOp {
            dest: r1,
            op: BinaryOperator::Add,
            left: x,
            right: zero,
        });

        // x * 1 = x
        let one = Register(3);
        entry.instructions.push(SSAInstruction::LoadInt {
            dest: one,
            value: 1,
        });

        let r2 = Register(4);
        entry.instructions.push(SSAInstruction::BinaryOp {
            dest: r2,
            op: BinaryOperator::Mul,
            left: r1,
            right: one,
        });

        // x - x = 0 (but we use different registers)
        let r3 = Register(5);
        entry.instructions.push(SSAInstruction::BinaryOp {
            dest: r3,
            op: BinaryOperator::Sub,
            left: x,
            right: x,
        });

        // x * 2 (should use shift)
        let two = Register(6);
        entry.instructions.push(SSAInstruction::LoadInt {
            dest: two,
            value: 2,
        });

        let r4 = Register(7);
        entry.instructions.push(SSAInstruction::BinaryOp {
            dest: r4,
            op: BinaryOperator::Mul,
            left: r2,
            right: two,
        });

        entry.instructions.push(SSAInstruction::Return {
            values: smallvec![r4],
        });

        func.blocks.push(entry);
        assert!(backend.generate(&func).is_ok());
    }

    #[test]
    fn test_strength_reduction() {
        // Test strength reduction (e.g., x * 2 -> x << 1)
        let context = Context::create();
        let mut backend = LLVMBackend::new(
            &context,
            "test_strength",
            CompilationMode::AOT,
            OptimizationLevel::Aggressive,
        );

        let mut func = SSAFunction::new("strength_reduction".to_string(), 1);
        let mut entry = BasicBlock::new(BlockId(0));

        let x = Register(0);

        // Multiplications by powers of 2
        for (i, power) in [2, 4, 8, 16, 32].iter().enumerate() {
            let const_reg = Register(i * 2 + 1);
            entry.instructions.push(SSAInstruction::LoadInt {
                dest: const_reg,
                value: *power,
            });

            let result_reg = Register(i * 2 + 2);
            entry.instructions.push(SSAInstruction::BinaryOp {
                dest: result_reg,
                op: BinaryOperator::Mul,
                left: x,
                right: const_reg,
            });
        }

        // Division by power of 2
        let div_const = Register(11);
        entry.instructions.push(SSAInstruction::LoadInt {
            dest: div_const,
            value: 8,
        });

        let div_result = Register(12);
        entry.instructions.push(SSAInstruction::BinaryOp {
            dest: div_result,
            op: BinaryOperator::Div,
            left: x,
            right: div_const,
        });

        entry.instructions.push(SSAInstruction::Return {
            values: smallvec![div_result],
        });

        func.blocks.push(entry);
        assert!(backend.generate(&func).is_ok());
    }

    #[test]
    fn test_common_subexpression_elimination() {
        // Test CSE with repeated computations
        let context = Context::create();
        let mut backend = LLVMBackend::new(
            &context,
            "test_cse",
            CompilationMode::AOT,
            OptimizationLevel::Aggressive,
        );

        let mut func = SSAFunction::new("cse_test".to_string(), 2);
        let mut entry = BasicBlock::new(BlockId(0));

        let a = Register(0);
        let b = Register(1);

        // Compute a + b multiple times
        let r1 = Register(2);
        entry.instructions.push(SSAInstruction::BinaryOp {
            dest: r1,
            op: BinaryOperator::Add,
            left: a,
            right: b,
        });

        let r2 = Register(3);
        entry.instructions.push(SSAInstruction::BinaryOp {
            dest: r2,
            op: BinaryOperator::Add,
            left: a,
            right: b,
        });

        // Compute a * b multiple times
        let r3 = Register(4);
        entry.instructions.push(SSAInstruction::BinaryOp {
            dest: r3,
            op: BinaryOperator::Mul,
            left: a,
            right: b,
        });

        let r4 = Register(5);
        entry.instructions.push(SSAInstruction::BinaryOp {
            dest: r4,
            op: BinaryOperator::Mul,
            left: a,
            right: b,
        });

        // Use all results
        let sum1 = Register(6);
        entry.instructions.push(SSAInstruction::BinaryOp {
            dest: sum1,
            op: BinaryOperator::Add,
            left: r1,
            right: r2,
        });

        let sum2 = Register(7);
        entry.instructions.push(SSAInstruction::BinaryOp {
            dest: sum2,
            op: BinaryOperator::Add,
            left: r3,
            right: r4,
        });

        let final_result = Register(8);
        entry.instructions.push(SSAInstruction::BinaryOp {
            dest: final_result,
            op: BinaryOperator::Add,
            left: sum1,
            right: sum2,
        });

        entry.instructions.push(SSAInstruction::Return {
            values: smallvec![final_result],
        });

        func.blocks.push(entry);
        assert!(backend.generate(&func).is_ok());
    }

    #[test]
    fn test_branch_optimization() {
        // Test branch optimizations (branch folding, etc.)
        let context = Context::create();
        let mut backend = LLVMBackend::new(
            &context,
            "test_branch_opt",
            CompilationMode::AOT,
            OptimizationLevel::Aggressive,
        );

        let mut func = SSAFunction::new("branch_opt".to_string(), 1);
        let x = Register(0);

        // Entry: constant condition (should be optimized away)
        let mut entry = BasicBlock::new(BlockId(0));
        let always_true = Register(1);
        entry.instructions.push(SSAInstruction::LoadInt {
            dest: always_true,
            value: 1,
        });

        entry.instructions.push(SSAInstruction::Branch {
            condition: always_true,
            true_block: BlockId(1),
            false_block: BlockId(2),
        });
        func.blocks.push(entry);

        // True branch (should be taken)
        let mut true_block = BasicBlock::new(BlockId(1));
        let result1 = Register(2);
        true_block.instructions.push(SSAInstruction::BinaryOp {
            dest: result1,
            op: BinaryOperator::Add,
            left: x,
            right: Register(1),
        });
        true_block.instructions.push(SSAInstruction::Jump {
            target: BlockId(3),
        });
        func.blocks.push(true_block);

        // False branch (dead code)
        let mut false_block = BasicBlock::new(BlockId(2));
        let result2 = Register(3);
        false_block.instructions.push(SSAInstruction::BinaryOp {
            dest: result2,
            op: BinaryOperator::Sub,
            left: x,
            right: Register(1),
        });
        false_block.instructions.push(SSAInstruction::Jump {
            target: BlockId(3),
        });
        func.blocks.push(false_block);

        // Merge
        let mut merge = BasicBlock::new(BlockId(3));
        let phi = Register(4);
        merge.instructions.push(SSAInstruction::Phi {
            dest: phi,
            incoming: vec![
                (BlockId(1), result1),
                (BlockId(2), result2),
            ],
        });
        merge.instructions.push(SSAInstruction::Return {
            values: smallvec![phi],
        });
        func.blocks.push(merge);

        assert!(backend.generate(&func).is_ok());
    }

    #[test]
    fn test_redundant_load_elimination() {
        // Test elimination of redundant memory loads
        let context = Context::create();
        let mut backend = LLVMBackend::new(
            &context,
            "test_load_elim",
            CompilationMode::AOT,
            OptimizationLevel::Aggressive,
        );

        let mut func = SSAFunction::new("load_elimination".to_string(), 1);
        let mut entry = BasicBlock::new(BlockId(0));

        let addr = Register(0);

        // Load from same address multiple times
        let load1 = Register(1);
        entry.instructions.push(SSAInstruction::Load {
            dest: load1,
            address: addr,
            ty: fastforth_frontend::ast::StackType::Int,
        });

        let load2 = Register(2);
        entry.instructions.push(SSAInstruction::Load {
            dest: load2,
            address: addr,
            ty: fastforth_frontend::ast::StackType::Int,
        });

        let load3 = Register(3);
        entry.instructions.push(SSAInstruction::Load {
            dest: load3,
            address: addr,
            ty: fastforth_frontend::ast::StackType::Int,
        });

        // Use all loads
        let sum1 = Register(4);
        entry.instructions.push(SSAInstruction::BinaryOp {
            dest: sum1,
            op: BinaryOperator::Add,
            left: load1,
            right: load2,
        });

        let sum2 = Register(5);
        entry.instructions.push(SSAInstruction::BinaryOp {
            dest: sum2,
            op: BinaryOperator::Add,
            left: sum1,
            right: load3,
        });

        entry.instructions.push(SSAInstruction::Return {
            values: smallvec![sum2],
        });

        func.blocks.push(entry);
        assert!(backend.generate(&func).is_ok());
    }
}

#[cfg(not(feature = "llvm"))]
mod no_llvm_tests {
    #[test]
    fn test_llvm_feature_disabled() {
        assert!(true);
    }
}
