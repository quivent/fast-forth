//! Examples demonstrating SSA validation catching bugs

use fastforth_frontend::ssa::{
    SSAFunction, SSAInstruction, Register, BlockId, BinaryOperator,
};
use smallvec::smallvec;

#[test]
fn test_validation_catches_multiple_assignment() {
    // This test demonstrates validation catching a bug where
    // the same register is assigned twice (SSA violation)

    let mut func = SSAFunction::new("buggy_double_assign".to_string(), 1);

    func.blocks[0].instructions = vec![
        SSAInstruction::LoadInt { dest: Register(1), value: 10 },
        SSAInstruction::LoadInt { dest: Register(1), value: 20 }, // BUG: %1 assigned twice!
        SSAInstruction::BinaryOp {
            dest: Register(2),
            op: BinaryOperator::Add,
            left: Register(0),
            right: Register(1),
        },
        SSAInstruction::Return {
            values: smallvec![Register(2)],
        },
    ];

    // Validation should catch this
    let result = func.validate();
    assert!(result.is_err(), "Validation should catch multiple assignment");

    let err = result.unwrap_err();
    assert!(err.to_string().contains("assigned multiple times"));
}

#[test]
fn test_validation_catches_use_of_undefined_register() {
    // This test demonstrates validation catching a bug where
    // a register is used before being defined

    let mut func = SSAFunction::new("buggy_undefined_use".to_string(), 1);

    func.blocks[0].instructions = vec![
        SSAInstruction::LoadInt { dest: Register(1), value: 5 },
        SSAInstruction::BinaryOp {
            dest: Register(2),
            op: BinaryOperator::Mul,
            left: Register(1),
            right: Register(99), // BUG: %99 never defined!
        },
        SSAInstruction::Return {
            values: smallvec![Register(2)],
        },
    ];

    // Validation should catch this
    let result = func.validate();
    assert!(result.is_err(), "Validation should catch undefined register use");

    let err = result.unwrap_err();
    assert!(err.to_string().contains("never defined"));
}

#[test]
fn test_validation_accepts_correct_if_then_else() {
    // This test demonstrates a correctly formed IF-THEN-ELSE
    // with proper Phi nodes that passes validation

    let mut func = SSAFunction::new("correct_conditional".to_string(), 1);

    // Simulate: IF %0 > 10 THEN %0 * 2 ELSE %0 + 1 THEN
    // Entry block: bb0
    // Then block: bb1
    // Else block: bb2
    // Merge block: bb3

    // Add blocks
    func.blocks.push(fastforth_frontend::ssa::BasicBlock::new(BlockId(1))); // then
    func.blocks.push(fastforth_frontend::ssa::BasicBlock::new(BlockId(2))); // else
    func.blocks.push(fastforth_frontend::ssa::BasicBlock::new(BlockId(3))); // merge

    // Entry block: compare and branch
    func.blocks[0].instructions = vec![
        SSAInstruction::LoadInt { dest: Register(1), value: 10 },
        SSAInstruction::BinaryOp {
            dest: Register(2),
            op: BinaryOperator::Gt,
            left: Register(0),
            right: Register(1),
        },
        SSAInstruction::Branch {
            condition: Register(2),
            true_block: BlockId(1),
            false_block: BlockId(2),
        },
    ];

    // Then block: multiply by 2
    func.blocks[1].instructions = vec![
        SSAInstruction::LoadInt { dest: Register(3), value: 2 },
        SSAInstruction::BinaryOp {
            dest: Register(4),
            op: BinaryOperator::Mul,
            left: Register(0),
            right: Register(3),
        },
        SSAInstruction::Jump { target: BlockId(3) },
    ];

    // Else block: add 1
    func.blocks[2].instructions = vec![
        SSAInstruction::LoadInt { dest: Register(5), value: 1 },
        SSAInstruction::BinaryOp {
            dest: Register(6),
            op: BinaryOperator::Add,
            left: Register(0),
            right: Register(5),
        },
        SSAInstruction::Jump { target: BlockId(3) },
    ];

    // Merge block: Phi node to merge results
    func.blocks[3].instructions = vec![
        SSAInstruction::Phi {
            dest: Register(7),
            incoming: vec![
                (BlockId(1), Register(4)), // from then
                (BlockId(2), Register(6)), // from else
            ],
        },
        SSAInstruction::Return {
            values: smallvec![Register(7)],
        },
    ];

    // This should pass all validation checks
    let result = func.validate();
    assert!(result.is_ok(), "Well-formed SSA should pass validation: {:?}", result);
}

#[test]
fn test_validation_catches_malformed_phi() {
    // This test demonstrates validation catching a Phi node
    // that's not at the start of a block

    let mut func = SSAFunction::new("buggy_phi_placement".to_string(), 1);

    func.blocks.push(fastforth_frontend::ssa::BasicBlock::new(BlockId(1)));
    func.blocks.push(fastforth_frontend::ssa::BasicBlock::new(BlockId(2)));

    func.blocks[0].instructions = vec![
        SSAInstruction::Branch {
            condition: Register(0),
            true_block: BlockId(1),
            false_block: BlockId(2),
        },
    ];

    func.blocks[1].instructions = vec![
        SSAInstruction::Jump { target: BlockId(2) },
    ];

    // BUG: Phi node is not at the start of the block!
    func.blocks[2].instructions = vec![
        SSAInstruction::LoadInt { dest: Register(1), value: 42 }, // Other instruction first
        SSAInstruction::Phi {
            dest: Register(2),
            incoming: vec![
                (BlockId(0), Register(0)),
                (BlockId(1), Register(0)),
            ],
        }, // BUG: Phi not at start!
        SSAInstruction::Return {
            values: smallvec![Register(2)],
        },
    ];

    let result = func.validate();
    assert!(result.is_err(), "Validation should catch misplaced Phi node");

    let err = result.unwrap_err();
    assert!(err.to_string().contains("not at start"));
}

#[test]
fn test_validation_performance_on_large_function() {
    // This test creates a large function to ensure validation
    // performance is reasonable

    let mut func = SSAFunction::new("large_function".to_string(), 1);

    // Create a chain of 1000 additions
    let mut reg = 0;
    for i in 0..1000 {
        func.blocks[0].instructions.push(SSAInstruction::LoadInt {
            dest: Register(reg + 1),
            value: i,
        });
        func.blocks[0].instructions.push(SSAInstruction::BinaryOp {
            dest: Register(reg + 2),
            op: BinaryOperator::Add,
            left: Register(reg),
            right: Register(reg + 1),
        });
        reg += 2;
    }

    func.blocks[0].instructions.push(SSAInstruction::Return {
        values: smallvec![Register(reg)],
    });

    // Validation should complete in reasonable time
    let start = std::time::Instant::now();
    let result = func.validate();
    let duration = start.elapsed();

    assert!(result.is_ok(), "Large valid function should pass validation");
    assert!(duration.as_millis() < 100,
            "Validation should be fast, took {:?}", duration);
}
