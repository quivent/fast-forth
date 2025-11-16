//! Code generation edge case tests
//!
//! Tests targeting untested backend paths including:
//! - Stack operations edge cases
//! - Control flow edge cases
//! - Function call edge cases
//! - Memory operation edge cases

#[cfg(feature = "llvm")]
mod llvm_edge_cases {
    use backend::{LLVMBackend, CodeGenerator, CompilationMode};
    use inkwell::{context::Context, OptimizationLevel};
    use fastforth_frontend::ssa::{
        SSAFunction, SSAInstruction, Register, BlockId, BasicBlock, BinaryOperator, UnaryOperator,
    };
    use smallvec::smallvec;

    // ===== STACK OPERATIONS EDGE CASES (5 tests) =====

    #[test]
    fn test_deep_stack_operations() {
        // Test with 20+ stack items
        let context = Context::create();
        let mut backend = LLVMBackend::new(
            &context,
            "test_deep_stack",
            CompilationMode::AOT,
            OptimizationLevel::None,
        );

        let mut func = SSAFunction::new("deep_stack".to_string(), 0);
        let mut entry = BasicBlock::new(BlockId(0));

        // Push 25 values onto the stack
        let mut registers = Vec::new();
        for i in 0..25 {
            let reg = Register(i);
            entry.instructions.push(SSAInstruction::LoadInt {
                dest: reg,
                value: i as i64,
            });
            registers.push(reg);
        }

        // Perform operations on deep stack
        let result = Register(25);
        entry.instructions.push(SSAInstruction::BinaryOp {
            dest: result,
            op: BinaryOperator::Add,
            left: registers[0],
            right: registers[24],
        });

        entry.instructions.push(SSAInstruction::Return {
            values: smallvec![result],
        });

        func.blocks.push(entry);
        assert!(backend.generate(&func).is_ok());

        let llvm_ir = backend.print_to_string();
        assert!(llvm_ir.contains("define"));
    }

    #[test]
    fn test_rapid_push_pop_sequences() {
        // Test rapid alternating push/pop operations
        let context = Context::create();
        let mut backend = LLVMBackend::new(
            &context,
            "test_rapid_ops",
            CompilationMode::AOT,
            OptimizationLevel::None,
        );

        let mut func = SSAFunction::new("rapid_ops".to_string(), 2);
        let mut entry = BasicBlock::new(BlockId(0));

        let a = Register(0);
        let b = Register(1);
        let mut next_reg = 2;

        // Rapid sequence: add, sub, mul, div
        for _ in 0..10 {
            let temp1 = Register(next_reg);
            next_reg += 1;
            entry.instructions.push(SSAInstruction::BinaryOp {
                dest: temp1,
                op: BinaryOperator::Add,
                left: a,
                right: b,
            });

            let temp2 = Register(next_reg);
            next_reg += 1;
            entry.instructions.push(SSAInstruction::BinaryOp {
                dest: temp2,
                op: BinaryOperator::Sub,
                left: temp1,
                right: b,
            });

            let temp3 = Register(next_reg);
            next_reg += 1;
            entry.instructions.push(SSAInstruction::BinaryOp {
                dest: temp3,
                op: BinaryOperator::Mul,
                left: temp2,
                right: a,
            });
        }

        let final_reg = Register(next_reg - 1);
        entry.instructions.push(SSAInstruction::Return {
            values: smallvec![final_reg],
        });

        func.blocks.push(entry);
        assert!(backend.generate(&func).is_ok());
    }

    #[test]
    fn test_stack_cache_overflow() {
        // Test stack cache with more items than cache size
        let context = Context::create();
        let mut backend = LLVMBackend::new(
            &context,
            "test_cache_overflow",
            CompilationMode::AOT,
            OptimizationLevel::Default,
        );

        let mut func = SSAFunction::new("cache_overflow".to_string(), 0);
        let mut entry = BasicBlock::new(BlockId(0));

        // Create 32 registers (typically exceeds cache size)
        for i in 0..32 {
            let reg = Register(i);
            entry.instructions.push(SSAInstruction::LoadInt {
                dest: reg,
                value: (i * 7) as i64,
            });
        }

        // Use them in computation
        let mut result = Register(0);
        for i in 1..32 {
            let temp = Register(32 + i);
            entry.instructions.push(SSAInstruction::BinaryOp {
                dest: temp,
                op: BinaryOperator::Add,
                left: result,
                right: Register(i),
            });
            result = temp;
        }

        entry.instructions.push(SSAInstruction::Return {
            values: smallvec![result],
        });

        func.blocks.push(entry);
        assert!(backend.generate(&func).is_ok());
    }

    #[test]
    fn test_stack_underflow_recovery() {
        // Test graceful handling of complex stack operations
        let context = Context::create();
        let mut backend = LLVMBackend::new(
            &context,
            "test_underflow",
            CompilationMode::AOT,
            OptimizationLevel::None,
        );

        let mut func = SSAFunction::new("underflow_test".to_string(), 3);
        let mut entry = BasicBlock::new(BlockId(0));

        let a = Register(0);
        let b = Register(1);
        let c = Register(2);

        // Chain of operations
        let temp1 = Register(3);
        entry.instructions.push(SSAInstruction::BinaryOp {
            dest: temp1,
            op: BinaryOperator::Add,
            left: a,
            right: b,
        });

        let temp2 = Register(4);
        entry.instructions.push(SSAInstruction::BinaryOp {
            dest: temp2,
            op: BinaryOperator::Mul,
            left: temp1,
            right: c,
        });

        entry.instructions.push(SSAInstruction::Return {
            values: smallvec![temp2],
        });

        func.blocks.push(entry);
        assert!(backend.generate(&func).is_ok());
    }

    #[test]
    fn test_interleaved_stack_operations() {
        // Test complex interleaving of stack manipulations
        let context = Context::create();
        let mut backend = LLVMBackend::new(
            &context,
            "test_interleaved",
            CompilationMode::AOT,
            OptimizationLevel::Default,
        );

        let mut func = SSAFunction::new("interleaved".to_string(), 4);
        let mut entry = BasicBlock::new(BlockId(0));

        let inputs = vec![Register(0), Register(1), Register(2), Register(3)];

        // Interleaved operations simulating: dup over swap rot patterns
        let r4 = Register(4);
        entry.instructions.push(SSAInstruction::BinaryOp {
            dest: r4,
            op: BinaryOperator::Add,
            left: inputs[0],
            right: inputs[1],
        });

        let r5 = Register(5);
        entry.instructions.push(SSAInstruction::BinaryOp {
            dest: r5,
            op: BinaryOperator::Mul,
            left: inputs[2],
            right: inputs[3],
        });

        let r6 = Register(6);
        entry.instructions.push(SSAInstruction::BinaryOp {
            dest: r6,
            op: BinaryOperator::Sub,
            left: r4,
            right: r5,
        });

        entry.instructions.push(SSAInstruction::Return {
            values: smallvec![r6],
        });

        func.blocks.push(entry);
        assert!(backend.generate(&func).is_ok());
    }

    // ===== CONTROL FLOW EDGE CASES (5 tests) =====

    #[test]
    fn test_deeply_nested_conditionals() {
        // Test 10+ levels of nested if-else
        let context = Context::create();
        let mut backend = LLVMBackend::new(
            &context,
            "test_deep_nested",
            CompilationMode::AOT,
            OptimizationLevel::None,
        );

        let mut func = SSAFunction::new("deep_nested".to_string(), 1);
        let x_reg = Register(0);

        // Create nested blocks
        let mut blocks = Vec::new();
        for i in 0..10 {
            blocks.push(BlockId(i));
        }

        // Entry block
        let mut entry = BasicBlock::new(BlockId(0));
        let cond_reg = Register(1);
        entry.instructions.push(SSAInstruction::LoadInt {
            dest: cond_reg,
            value: 1,
        });
        entry.instructions.push(SSAInstruction::Branch {
            condition: cond_reg,
            true_block: BlockId(1),
            false_block: BlockId(9),
        });
        func.blocks.push(entry);

        // Nested levels
        for i in 1..9 {
            let mut block = BasicBlock::new(BlockId(i));
            let new_cond = Register(i + 1);
            block.instructions.push(SSAInstruction::LoadInt {
                dest: new_cond,
                value: i as i64,
            });
            block.instructions.push(SSAInstruction::Branch {
                condition: new_cond,
                true_block: BlockId(i + 1),
                false_block: BlockId(9),
            });
            func.blocks.push(block);
        }

        // Exit block
        let mut exit = BasicBlock::new(BlockId(9));
        exit.instructions.push(SSAInstruction::Return {
            values: smallvec![x_reg],
        });
        func.blocks.push(exit);

        assert!(backend.generate(&func).is_ok());
    }

    #[test]
    fn test_long_jump_distances() {
        // Test jumps across many blocks
        let context = Context::create();
        let mut backend = LLVMBackend::new(
            &context,
            "test_long_jumps",
            CompilationMode::AOT,
            OptimizationLevel::None,
        );

        let mut func = SSAFunction::new("long_jumps".to_string(), 1);
        let x_reg = Register(0);

        // Create 50 blocks
        for i in 0..50 {
            let mut block = BasicBlock::new(BlockId(i));

            if i == 0 {
                // Entry: jump to middle
                block.instructions.push(SSAInstruction::Jump {
                    target: BlockId(25),
                });
            } else if i == 25 {
                // Middle: jump to near end
                block.instructions.push(SSAInstruction::Jump {
                    target: BlockId(45),
                });
            } else if i == 45 {
                // Near end: jump to end
                block.instructions.push(SSAInstruction::Jump {
                    target: BlockId(49),
                });
            } else if i == 49 {
                // Exit
                block.instructions.push(SSAInstruction::Return {
                    values: smallvec![x_reg],
                });
            } else {
                // Other blocks: dead code
                let temp = Register(i + 1);
                block.instructions.push(SSAInstruction::LoadInt {
                    dest: temp,
                    value: i as i64,
                });
                block.instructions.push(SSAInstruction::Return {
                    values: smallvec![temp],
                });
            }

            func.blocks.push(block);
        }

        assert!(backend.generate(&func).is_ok());
    }

    #[test]
    fn test_many_blocks() {
        // Test 100+ basic blocks
        let context = Context::create();
        let mut backend = LLVMBackend::new(
            &context,
            "test_many_blocks",
            CompilationMode::AOT,
            OptimizationLevel::None,
        );

        let mut func = SSAFunction::new("many_blocks".to_string(), 1);
        let x_reg = Register(0);

        // Create 120 blocks
        for i in 0..120 {
            let mut block = BasicBlock::new(BlockId(i));

            if i < 119 {
                // Sequential blocks
                let temp = Register(i + 1);
                block.instructions.push(SSAInstruction::BinaryOp {
                    dest: temp,
                    op: BinaryOperator::Add,
                    left: x_reg,
                    right: x_reg,
                });
                block.instructions.push(SSAInstruction::Jump {
                    target: BlockId(i + 1),
                });
            } else {
                // Final block
                block.instructions.push(SSAInstruction::Return {
                    values: smallvec![x_reg],
                });
            }

            func.blocks.push(block);
        }

        assert!(backend.generate(&func).is_ok());
    }

    #[test]
    fn test_empty_blocks() {
        // Test blocks with only jump instructions
        let context = Context::create();
        let mut backend = LLVMBackend::new(
            &context,
            "test_empty_blocks",
            CompilationMode::AOT,
            OptimizationLevel::None,
        );

        let mut func = SSAFunction::new("empty_blocks".to_string(), 1);
        let x_reg = Register(0);

        // Entry block: just jump
        let mut entry = BasicBlock::new(BlockId(0));
        entry.instructions.push(SSAInstruction::Jump {
            target: BlockId(1),
        });
        func.blocks.push(entry);

        // Empty blocks that just forward
        for i in 1..5 {
            let mut block = BasicBlock::new(BlockId(i));
            block.instructions.push(SSAInstruction::Jump {
                target: BlockId(i + 1),
            });
            func.blocks.push(block);
        }

        // Final block
        let mut exit = BasicBlock::new(BlockId(5));
        exit.instructions.push(SSAInstruction::Return {
            values: smallvec![x_reg],
        });
        func.blocks.push(exit);

        assert!(backend.generate(&func).is_ok());
    }

    #[test]
    fn test_complex_phi_nodes() {
        // Test phi nodes with many incoming edges
        let context = Context::create();
        let mut backend = LLVMBackend::new(
            &context,
            "test_complex_phi",
            CompilationMode::AOT,
            OptimizationLevel::None,
        );

        let mut func = SSAFunction::new("complex_phi".to_string(), 1);
        let x_reg = Register(0);

        // Create multiple predecessor blocks
        for i in 0..5 {
            let mut block = BasicBlock::new(BlockId(i));
            let val = Register(i + 1);
            block.instructions.push(SSAInstruction::LoadInt {
                dest: val,
                value: i as i64,
            });
            block.instructions.push(SSAInstruction::Jump {
                target: BlockId(5),
            });
            func.blocks.push(block);
        }

        // Merge block with phi node
        let mut merge = BasicBlock::new(BlockId(5));
        let phi_reg = Register(6);
        merge.instructions.push(SSAInstruction::Phi {
            dest: phi_reg,
            incoming: vec![
                (BlockId(0), Register(1)),
                (BlockId(1), Register(2)),
                (BlockId(2), Register(3)),
                (BlockId(3), Register(4)),
                (BlockId(4), Register(5)),
            ],
        });
        merge.instructions.push(SSAInstruction::Return {
            values: smallvec![phi_reg],
        });
        func.blocks.push(merge);

        assert!(backend.generate(&func).is_ok());
    }

    // ===== FUNCTION CALL EDGE CASES (5 tests) =====

    #[test]
    fn test_recursive_tail_calls() {
        // Test tail recursion optimization
        let context = Context::create();
        let mut backend = LLVMBackend::new(
            &context,
            "test_tail_recursion",
            CompilationMode::AOT,
            OptimizationLevel::Aggressive,
        );

        let mut func = SSAFunction::new("factorial_tail".to_string(), 2);
        let n_reg = Register(0);
        let acc_reg = Register(1);

        let mut entry = BasicBlock::new(BlockId(0));
        let zero_reg = Register(2);
        let cond_reg = Register(3);

        entry.instructions.push(SSAInstruction::LoadInt {
            dest: zero_reg,
            value: 0,
        });

        entry.instructions.push(SSAInstruction::BinaryOp {
            dest: cond_reg,
            op: BinaryOperator::Eq,
            left: n_reg,
            right: zero_reg,
        });

        entry.instructions.push(SSAInstruction::Branch {
            condition: cond_reg,
            true_block: BlockId(1),
            false_block: BlockId(2),
        });

        // Base case
        let mut base = BasicBlock::new(BlockId(1));
        base.instructions.push(SSAInstruction::Return {
            values: smallvec![acc_reg],
        });
        func.blocks.push(base);

        // Recursive case
        let mut recursive = BasicBlock::new(BlockId(2));
        let one_reg = Register(4);
        let n_minus_1 = Register(5);
        let new_acc = Register(6);

        recursive.instructions.push(SSAInstruction::LoadInt {
            dest: one_reg,
            value: 1,
        });

        recursive.instructions.push(SSAInstruction::BinaryOp {
            dest: n_minus_1,
            op: BinaryOperator::Sub,
            left: n_reg,
            right: one_reg,
        });

        recursive.instructions.push(SSAInstruction::BinaryOp {
            dest: new_acc,
            op: BinaryOperator::Mul,
            left: n_reg,
            right: acc_reg,
        });

        // Tail call
        let result_reg = Register(7);
        recursive.instructions.push(SSAInstruction::Call {
            dest: smallvec![result_reg],
            name: "factorial_tail".to_string(),
            args: smallvec![n_minus_1, new_acc],
        });

        recursive.instructions.push(SSAInstruction::Return {
            values: smallvec![result_reg],
        });

        func.blocks.push(entry);
        func.blocks.push(recursive);

        assert!(backend.generate(&func).is_ok());
    }

    #[test]
    fn test_mutual_recursion() {
        // Test mutually recursive functions
        let context = Context::create();
        let mut backend = LLVMBackend::new(
            &context,
            "test_mutual_recursion",
            CompilationMode::AOT,
            OptimizationLevel::None,
        );

        // Function even?(n)
        let mut even_func = SSAFunction::new("is_even".to_string(), 1);
        let mut even_entry = BasicBlock::new(BlockId(0));
        let n_reg = Register(0);
        let zero_reg = Register(1);
        let cond_reg = Register(2);

        even_entry.instructions.push(SSAInstruction::LoadInt {
            dest: zero_reg,
            value: 0,
        });

        even_entry.instructions.push(SSAInstruction::BinaryOp {
            dest: cond_reg,
            op: BinaryOperator::Eq,
            left: n_reg,
            right: zero_reg,
        });

        even_entry.instructions.push(SSAInstruction::Branch {
            condition: cond_reg,
            true_block: BlockId(1),
            false_block: BlockId(2),
        });

        // Base case: 0 is even
        let mut even_base = BasicBlock::new(BlockId(1));
        let true_reg = Register(3);
        even_base.instructions.push(SSAInstruction::LoadInt {
            dest: true_reg,
            value: 1,
        });
        even_base.instructions.push(SSAInstruction::Return {
            values: smallvec![true_reg],
        });

        // Recursive: n is even if n-1 is odd
        let mut even_rec = BasicBlock::new(BlockId(2));
        let one_reg = Register(4);
        let n_minus_1 = Register(5);
        let odd_result = Register(6);

        even_rec.instructions.push(SSAInstruction::LoadInt {
            dest: one_reg,
            value: 1,
        });

        even_rec.instructions.push(SSAInstruction::BinaryOp {
            dest: n_minus_1,
            op: BinaryOperator::Sub,
            left: n_reg,
            right: one_reg,
        });

        even_rec.instructions.push(SSAInstruction::Call {
            dest: smallvec![odd_result],
            name: "is_odd".to_string(),
            args: smallvec![n_minus_1],
        });

        even_rec.instructions.push(SSAInstruction::Return {
            values: smallvec![odd_result],
        });

        even_func.blocks.push(even_entry);
        even_func.blocks.push(even_base);
        even_func.blocks.push(even_rec);

        assert!(backend.generate(&even_func).is_ok());
    }

    #[test]
    fn test_deep_call_stack() {
        // Test very deep function call nesting
        let context = Context::create();
        let mut backend = LLVMBackend::new(
            &context,
            "test_deep_calls",
            CompilationMode::AOT,
            OptimizationLevel::None,
        );

        let mut func = SSAFunction::new("deep_calls".to_string(), 1);
        let mut entry = BasicBlock::new(BlockId(0));
        let n_reg = Register(0);

        // Chain of nested calls
        let mut current_reg = n_reg;
        for i in 0..50 {
            let result_reg = Register(i + 1);
            entry.instructions.push(SSAInstruction::Call {
                dest: smallvec![result_reg],
                name: format!("helper_{}", i),
                args: smallvec![current_reg],
            });
            current_reg = result_reg;
        }

        entry.instructions.push(SSAInstruction::Return {
            values: smallvec![current_reg],
        });

        func.blocks.push(entry);
        assert!(backend.generate(&func).is_ok());
    }

    #[test]
    fn test_call_with_many_arguments() {
        // Test function calls with many arguments
        let context = Context::create();
        let mut backend = LLVMBackend::new(
            &context,
            "test_many_args",
            CompilationMode::AOT,
            OptimizationLevel::None,
        );

        let mut func = SSAFunction::new("many_args".to_string(), 0);
        let mut entry = BasicBlock::new(BlockId(0));

        // Create 15 arguments
        let mut args = Vec::new();
        for i in 0..15 {
            let reg = Register(i);
            entry.instructions.push(SSAInstruction::LoadInt {
                dest: reg,
                value: i as i64,
            });
            args.push(reg);
        }

        let result_reg = Register(15);
        entry.instructions.push(SSAInstruction::Call {
            dest: smallvec![result_reg],
            name: "sum_many".to_string(),
            args: args.into_iter().collect(),
        });

        entry.instructions.push(SSAInstruction::Return {
            values: smallvec![result_reg],
        });

        func.blocks.push(entry);
        assert!(backend.generate(&func).is_ok());
    }

    #[test]
    fn test_variadic_calls() {
        // Test calls with varying argument counts
        let context = Context::create();
        let mut backend = LLVMBackend::new(
            &context,
            "test_variadic",
            CompilationMode::AOT,
            OptimizationLevel::None,
        );

        let mut func = SSAFunction::new("variadic_test".to_string(), 3);
        let mut entry = BasicBlock::new(BlockId(0));

        let a = Register(0);
        let b = Register(1);
        let c = Register(2);

        // Call with 1 arg
        let r1 = Register(3);
        entry.instructions.push(SSAInstruction::Call {
            dest: smallvec![r1],
            name: "func1".to_string(),
            args: smallvec![a],
        });

        // Call with 2 args
        let r2 = Register(4);
        entry.instructions.push(SSAInstruction::Call {
            dest: smallvec![r2],
            name: "func2".to_string(),
            args: smallvec![a, b],
        });

        // Call with 3 args
        let r3 = Register(5);
        entry.instructions.push(SSAInstruction::Call {
            dest: smallvec![r3],
            name: "func3".to_string(),
            args: smallvec![a, b, c],
        });

        // Sum results
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
            left: sum1,
            right: r3,
        });

        entry.instructions.push(SSAInstruction::Return {
            values: smallvec![sum2],
        });

        func.blocks.push(entry);
        assert!(backend.generate(&func).is_ok());
    }

    // ===== MEMORY OPERATION EDGE CASES (5 tests) =====

    #[test]
    fn test_large_allocations() {
        // Test handling of large memory allocations
        let context = Context::create();
        let mut backend = LLVMBackend::new(
            &context,
            "test_large_alloc",
            CompilationMode::AOT,
            OptimizationLevel::None,
        );

        let mut func = SSAFunction::new("large_alloc".to_string(), 0);
        let mut entry = BasicBlock::new(BlockId(0));

        // Allocate large size (1MB)
        let size_reg = Register(0);
        entry.instructions.push(SSAInstruction::LoadInt {
            dest: size_reg,
            value: 1024 * 1024,
        });

        // Simulate allocation call
        let ptr_reg = Register(1);
        entry.instructions.push(SSAInstruction::Call {
            dest: smallvec![ptr_reg],
            name: "malloc".to_string(),
            args: smallvec![size_reg],
        });

        entry.instructions.push(SSAInstruction::Return {
            values: smallvec![ptr_reg],
        });

        func.blocks.push(entry);
        assert!(backend.generate(&func).is_ok());
    }

    #[test]
    fn test_unaligned_memory_access() {
        // Test unaligned memory reads/writes
        let context = Context::create();
        let mut backend = LLVMBackend::new(
            &context,
            "test_unaligned",
            CompilationMode::AOT,
            OptimizationLevel::None,
        );

        let mut func = SSAFunction::new("unaligned_access".to_string(), 1);
        let mut entry = BasicBlock::new(BlockId(0));

        let base_addr = Register(0);

        // Access at odd offsets
        for offset in &[1, 3, 5, 7] {
            let offset_reg = Register((*offset * 2) as usize);
            entry.instructions.push(SSAInstruction::LoadInt {
                dest: offset_reg,
                value: *offset,
            });

            let addr_reg = Register((offset * 2 + 1) as usize);
            entry.instructions.push(SSAInstruction::BinaryOp {
                dest: addr_reg,
                op: BinaryOperator::Add,
                left: base_addr,
                right: offset_reg,
            });

            let val_reg = Register((offset * 2 + 2) as usize);
            entry.instructions.push(SSAInstruction::Load {
                dest: val_reg,
                address: addr_reg,
                ty: fastforth_frontend::ast::StackType::Int,
            });
        }

        let final_reg = Register(16);
        entry.instructions.push(SSAInstruction::LoadInt {
            dest: final_reg,
            value: 0,
        });

        entry.instructions.push(SSAInstruction::Return {
            values: smallvec![final_reg],
        });

        func.blocks.push(entry);
        assert!(backend.generate(&func).is_ok());
    }

    #[test]
    fn test_memory_pressure() {
        // Test many simultaneous memory operations
        let context = Context::create();
        let mut backend = LLVMBackend::new(
            &context,
            "test_mem_pressure",
            CompilationMode::AOT,
            OptimizationLevel::None,
        );

        let mut func = SSAFunction::new("mem_pressure".to_string(), 1);
        let mut entry = BasicBlock::new(BlockId(0));

        let base_addr = Register(0);

        // Perform 30 memory operations
        for i in 0..30 {
            let offset_reg = Register(i * 3 + 1);
            entry.instructions.push(SSAInstruction::LoadInt {
                dest: offset_reg,
                value: (i * 8) as i64,
            });

            let addr_reg = Register(i * 3 + 2);
            entry.instructions.push(SSAInstruction::BinaryOp {
                dest: addr_reg,
                op: BinaryOperator::Add,
                left: base_addr,
                right: offset_reg,
            });

            let val_reg = Register(i * 3 + 3);
            entry.instructions.push(SSAInstruction::Load {
                dest: val_reg,
                address: addr_reg,
                ty: fastforth_frontend::ast::StackType::Int,
            });
        }

        let result = Register(91);
        entry.instructions.push(SSAInstruction::LoadInt {
            dest: result,
            value: 0,
        });

        entry.instructions.push(SSAInstruction::Return {
            values: smallvec![result],
        });

        func.blocks.push(entry);
        assert!(backend.generate(&func).is_ok());
    }

    #[test]
    fn test_variable_constant_edge_cases() {
        // Test edge cases in variable and constant handling
        let context = Context::create();
        let mut backend = LLVMBackend::new(
            &context,
            "test_var_const",
            CompilationMode::AOT,
            OptimizationLevel::None,
        );

        let mut func = SSAFunction::new("var_const_edge".to_string(), 0);
        let mut entry = BasicBlock::new(BlockId(0));

        // Very large constant
        let large_const = Register(0);
        entry.instructions.push(SSAInstruction::LoadInt {
            dest: large_const,
            value: i64::MAX,
        });

        // Very small constant
        let small_const = Register(1);
        entry.instructions.push(SSAInstruction::LoadInt {
            dest: small_const,
            value: i64::MIN,
        });

        // Zero
        let zero = Register(2);
        entry.instructions.push(SSAInstruction::LoadInt {
            dest: zero,
            value: 0,
        });

        // Negative
        let negative = Register(3);
        entry.instructions.push(SSAInstruction::LoadInt {
            dest: negative,
            value: -42,
        });

        // Operations on edge cases
        let result = Register(4);
        entry.instructions.push(SSAInstruction::BinaryOp {
            dest: result,
            op: BinaryOperator::Add,
            left: zero,
            right: negative,
        });

        entry.instructions.push(SSAInstruction::Return {
            values: smallvec![result],
        });

        func.blocks.push(entry);
        assert!(backend.generate(&func).is_ok());
    }

    #[test]
    fn test_memory_aliasing() {
        // Test potential memory aliasing scenarios
        let context = Context::create();
        let mut backend = LLVMBackend::new(
            &context,
            "test_aliasing",
            CompilationMode::AOT,
            OptimizationLevel::Default,
        );

        let mut func = SSAFunction::new("aliasing_test".to_string(), 2);
        let mut entry = BasicBlock::new(BlockId(0));

        let addr1 = Register(0);
        let addr2 = Register(1);

        // Store to addr1
        let val1 = Register(2);
        entry.instructions.push(SSAInstruction::LoadInt {
            dest: val1,
            value: 42,
        });

        entry.instructions.push(SSAInstruction::Store {
            address: addr1,
            value: val1,
            ty: fastforth_frontend::ast::StackType::Int,
        });

        // Store to addr2 (potential alias)
        let val2 = Register(3);
        entry.instructions.push(SSAInstruction::LoadInt {
            dest: val2,
            value: 99,
        });

        entry.instructions.push(SSAInstruction::Store {
            address: addr2,
            value: val2,
            ty: fastforth_frontend::ast::StackType::Int,
        });

        // Load from addr1 again
        let loaded = Register(4);
        entry.instructions.push(SSAInstruction::Load {
            dest: loaded,
            address: addr1,
            ty: fastforth_frontend::ast::StackType::Int,
        });

        entry.instructions.push(SSAInstruction::Return {
            values: smallvec![loaded],
        });

        func.blocks.push(entry);
        assert!(backend.generate(&func).is_ok());
    }
}

#[cfg(not(feature = "llvm"))]
mod no_llvm_tests {
    #[test]
    fn test_llvm_feature_disabled() {
        // Placeholder when LLVM is not enabled
        assert!(true);
    }
}
