//! Code generation integration tests

#[cfg(feature = "llvm")]
mod llvm_tests {
    use backend::{LLVMBackend, CodeGenerator, CompilationMode};
    use inkwell::{context::Context, OptimizationLevel};
    use fastforth_frontend::ssa::{
        SSAFunction, SSAInstruction, Register, BlockId, BasicBlock, BinaryOperator, UnaryOperator,
    };
    use smallvec::smallvec;
    use std::path::Path;

    #[test]
    fn test_simple_addition() {
        let context = Context::create();
        let mut backend = LLVMBackend::new(
            &context,
            "test_add",
            CompilationMode::AOT,
            OptimizationLevel::None,
        );

        // Create function: add(a, b) = a + b
        let mut func = SSAFunction::new("add".to_string(), 2);

        let mut entry = BasicBlock::new(BlockId(0));
        let a_reg = Register(0);
        let b_reg = Register(1);
        let result_reg = Register(2);

        entry.instructions.push(SSAInstruction::BinaryOp {
            dest: result_reg,
            op: BinaryOperator::Add,
            left: a_reg,
            right: b_reg,
        });

        entry.instructions.push(SSAInstruction::Return {
            values: smallvec![result_reg],
        });

        func.blocks.push(entry);

        // Generate code
        assert!(backend.generate(&func).is_ok());

        // Verify module
        let llvm_ir = backend.print_to_string();
        assert!(llvm_ir.contains("define"));
        assert!(llvm_ir.contains("add"));
    }

    #[test]
    fn test_multiplication() {
        let context = Context::create();
        let mut backend = LLVMBackend::new(
            &context,
            "test_mul",
            CompilationMode::AOT,
            OptimizationLevel::None,
        );

        // Create function: mul(a, b) = a * b
        let mut func = SSAFunction::new("mul".to_string(), 2);

        let mut entry = BasicBlock::new(BlockId(0));
        let a_reg = Register(0);
        let b_reg = Register(1);
        let result_reg = Register(2);

        entry.instructions.push(SSAInstruction::BinaryOp {
            dest: result_reg,
            op: BinaryOperator::Mul,
            left: a_reg,
            right: b_reg,
        });

        entry.instructions.push(SSAInstruction::Return {
            values: smallvec![result_reg],
        });

        func.blocks.push(entry);

        assert!(backend.generate(&func).is_ok());
    }

    #[test]
    fn test_comparison() {
        let context = Context::create();
        let mut backend = LLVMBackend::new(
            &context,
            "test_cmp",
            CompilationMode::AOT,
            OptimizationLevel::None,
        );

        // Create function: lt(a, b) = a < b
        let mut func = SSAFunction::new("lt".to_string(), 2);

        let mut entry = BasicBlock::new(BlockId(0));
        let a_reg = Register(0);
        let b_reg = Register(1);
        let result_reg = Register(2);

        entry.instructions.push(SSAInstruction::BinaryOp {
            dest: result_reg,
            op: BinaryOperator::Lt,
            left: a_reg,
            right: b_reg,
        });

        entry.instructions.push(SSAInstruction::Return {
            values: smallvec![result_reg],
        });

        func.blocks.push(entry);

        assert!(backend.generate(&func).is_ok());

        let llvm_ir = backend.print_to_string();
        assert!(llvm_ir.contains("icmp"));
    }

    #[test]
    fn test_unary_negate() {
        let context = Context::create();
        let mut backend = LLVMBackend::new(
            &context,
            "test_neg",
            CompilationMode::AOT,
            OptimizationLevel::None,
        );

        // Create function: neg(x) = -x
        let mut func = SSAFunction::new("neg".to_string(), 1);

        let mut entry = BasicBlock::new(BlockId(0));
        let x_reg = Register(0);
        let result_reg = Register(1);

        entry.instructions.push(SSAInstruction::UnaryOp {
            dest: result_reg,
            op: UnaryOperator::Negate,
            operand: x_reg,
        });

        entry.instructions.push(SSAInstruction::Return {
            values: smallvec![result_reg],
        });

        func.blocks.push(entry);

        assert!(backend.generate(&func).is_ok());

        let llvm_ir = backend.print_to_string();
        assert!(llvm_ir.contains("sub"));
    }

    #[test]
    fn test_constant_loading() {
        let context = Context::create();
        let mut backend = LLVMBackend::new(
            &context,
            "test_const",
            CompilationMode::AOT,
            OptimizationLevel::None,
        );

        // Create function: constant() = 42
        let mut func = SSAFunction::new("constant".to_string(), 0);

        let mut entry = BasicBlock::new(BlockId(0));
        let const_reg = Register(0);

        entry.instructions.push(SSAInstruction::LoadInt {
            dest: const_reg,
            value: 42,
        });

        entry.instructions.push(SSAInstruction::Return {
            values: smallvec![const_reg],
        });

        func.blocks.push(entry);

        assert!(backend.generate(&func).is_ok());

        let llvm_ir = backend.print_to_string();
        assert!(llvm_ir.contains("42"));
    }

    #[test]
    fn test_control_flow_branch() {
        let context = Context::create();
        let mut backend = LLVMBackend::new(
            &context,
            "test_branch",
            CompilationMode::AOT,
            OptimizationLevel::None,
        );

        // Create function: abs(x) = x < 0 ? -x : x
        let mut func = SSAFunction::new("abs".to_string(), 1);

        let entry_id = BlockId(0);
        let then_id = BlockId(1);
        let else_id = BlockId(2);

        // Entry: check if x < 0
        let mut entry = BasicBlock::new(entry_id);
        let x_reg = Register(0);
        let zero_reg = Register(1);
        let cond_reg = Register(2);

        entry.instructions.push(SSAInstruction::LoadInt {
            dest: zero_reg,
            value: 0,
        });

        entry.instructions.push(SSAInstruction::BinaryOp {
            dest: cond_reg,
            op: BinaryOperator::Lt,
            left: x_reg,
            right: zero_reg,
        });

        entry.instructions.push(SSAInstruction::Branch {
            condition: cond_reg,
            true_block: then_id,
            false_block: else_id,
        });

        // Then: return -x
        let mut then_block = BasicBlock::new(then_id);
        let neg_reg = Register(3);

        then_block.instructions.push(SSAInstruction::UnaryOp {
            dest: neg_reg,
            op: UnaryOperator::Negate,
            operand: x_reg,
        });

        then_block.instructions.push(SSAInstruction::Return {
            values: smallvec![neg_reg],
        });

        // Else: return x
        let mut else_block = BasicBlock::new(else_id);

        else_block.instructions.push(SSAInstruction::Return {
            values: smallvec![x_reg],
        });

        func.blocks.push(entry);
        func.blocks.push(then_block);
        func.blocks.push(else_block);

        assert!(backend.generate(&func).is_ok());

        let llvm_ir = backend.print_to_string();
        assert!(llvm_ir.contains("br i1"));
    }

    #[test]
    fn test_optimization_levels() {
        for opt_level in &[
            OptimizationLevel::None,
            OptimizationLevel::Less,
            OptimizationLevel::Default,
            OptimizationLevel::Aggressive,
        ] {
            let context = Context::create();
            let mut backend = LLVMBackend::new(
                &context,
                "test_opt",
                CompilationMode::AOT,
                *opt_level,
            );

            let mut func = SSAFunction::new("test".to_string(), 1);

            let mut entry = BasicBlock::new(BlockId(0));
            let x_reg = Register(0);

            entry.instructions.push(SSAInstruction::Return {
                values: smallvec![x_reg],
            });

            func.blocks.push(entry);

            assert!(backend.generate(&func).is_ok());
        }
    }

    #[test]
    fn test_multiple_operations() {
        let context = Context::create();
        let mut backend = LLVMBackend::new(
            &context,
            "test_multi",
            CompilationMode::AOT,
            OptimizationLevel::Default,
        );

        // Create function: f(a, b) = (a + b) * (a - b)
        let mut func = SSAFunction::new("multi_op".to_string(), 2);

        let mut entry = BasicBlock::new(BlockId(0));
        let a_reg = Register(0);
        let b_reg = Register(1);
        let sum_reg = Register(2);
        let diff_reg = Register(3);
        let result_reg = Register(4);

        entry.instructions.push(SSAInstruction::BinaryOp {
            dest: sum_reg,
            op: BinaryOperator::Add,
            left: a_reg,
            right: b_reg,
        });

        entry.instructions.push(SSAInstruction::BinaryOp {
            dest: diff_reg,
            op: BinaryOperator::Sub,
            left: a_reg,
            right: b_reg,
        });

        entry.instructions.push(SSAInstruction::BinaryOp {
            dest: result_reg,
            op: BinaryOperator::Mul,
            left: sum_reg,
            right: diff_reg,
        });

        entry.instructions.push(SSAInstruction::Return {
            values: smallvec![result_reg],
        });

        func.blocks.push(entry);

        assert!(backend.generate(&func).is_ok());
    }
}

#[cfg(not(feature = "llvm"))]
mod no_llvm_tests {
    #[test]
    fn test_llvm_feature_disabled() {
        // This test passes when LLVM feature is disabled
        assert!(true);
    }
}
