//! Control Flow Code Generation
//!
//! Generate native code for Forth control structures (IF/THEN/ELSE, DO/LOOP, BEGIN/UNTIL, etc.)

use crate::error::{BackendError, Result};
use fastforth_frontend::ssa::{Register, BlockId};
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::values::BasicValueEnum;
use inkwell::basic_block::BasicBlock;
use std::collections::HashMap;

/// Control flow code generator
pub struct ControlFlowCodegen<'ctx> {
    context: &'ctx Context,
}

impl<'ctx> ControlFlowCodegen<'ctx> {
    pub fn new(context: &'ctx Context) -> Self {
        Self { context }
    }

    /// Generate conditional branch (IF/ELSE/THEN)
    pub fn generate_branch(
        &self,
        builder: &Builder<'ctx>,
        values: &mut HashMap<Register, BasicValueEnum<'ctx>>,
        blocks: &HashMap<BlockId, BasicBlock<'ctx>>,
        condition: Register,
        true_block: BlockId,
        false_block: BlockId,
    ) -> Result<()> {
        // Get condition value
        let cond_val = values
            .get(&condition)
            .ok_or_else(|| BackendError::InvalidIR(format!("Undefined condition register: {}", condition)))?;

        // Convert to i1 if necessary
        let cond_bool = if cond_val.is_int_value() {
            let int_val = cond_val.into_int_value();
            if int_val.get_type().get_bit_width() == 1 {
                int_val
            } else {
                // Compare with zero (Forth convention: 0 is false, non-zero is true)
                let zero = int_val.get_type().const_zero();
                builder.build_int_compare(
                    inkwell::IntPredicate::NE,
                    int_val,
                    zero,
                    "cond"
                ).map_err(|e| BackendError::CodeGenError(e.to_string()))?
            }
        } else {
            return Err(BackendError::CodeGenError("Condition must be integer value".to_string()));
        };

        // Get basic blocks
        let then_bb = blocks
            .get(&true_block)
            .ok_or_else(|| BackendError::InvalidIR(format!("Undefined then block: {}", true_block.0)))?;

        let else_bb = blocks
            .get(&false_block)
            .ok_or_else(|| BackendError::InvalidIR(format!("Undefined else block: {}", false_block.0)))?;

        // Build conditional branch
        builder.build_conditional_branch(cond_bool, *then_bb, *else_bb)
            .map_err(|e| BackendError::CodeGenError(e.to_string()))?;

        Ok(())
    }

    /// Generate unconditional jump
    pub fn generate_jump(
        &self,
        builder: &Builder<'ctx>,
        blocks: &HashMap<BlockId, BasicBlock<'ctx>>,
        target: BlockId,
    ) -> Result<()> {
        let target_bb = blocks
            .get(&target)
            .ok_or_else(|| BackendError::InvalidIR(format!("Undefined target block: {}", target.0)))?;

        builder.build_unconditional_branch(*target_bb)
            .map_err(|e| BackendError::CodeGenError(e.to_string()))?;

        Ok(())
    }

    /// Generate loop structure (DO/LOOP)
    ///
    /// Forth DO/LOOP structure:
    /// ```forth
    /// limit start DO
    ///   ... body ...
    /// LOOP
    /// ```
    ///
    /// LLVM IR equivalent:
    /// ```llvm
    /// entry:
    ///   %limit = ...
    ///   %start = ...
    ///   br label %loop_header
    ///
    /// loop_header:
    ///   %i = phi [%start, %entry], [%i_next, %loop_body]
    ///   %cond = icmp slt %i, %limit
    ///   br i1 %cond, label %loop_body, label %loop_exit
    ///
    /// loop_body:
    ///   ... body ...
    ///   %i_next = add %i, 1
    ///   br label %loop_header
    ///
    /// loop_exit:
    ///   ...
    /// ```
    pub fn generate_do_loop(
        &self,
        builder: &Builder<'ctx>,
        values: &mut HashMap<Register, BasicValueEnum<'ctx>>,
        blocks: &HashMap<BlockId, BasicBlock<'ctx>>,
        start: Register,
        limit: Register,
        body_block: BlockId,
        exit_block: BlockId,
        loop_counter: Register,
    ) -> Result<()> {
        let start_val = values
            .get(&start)
            .ok_or_else(|| BackendError::InvalidIR(format!("Undefined start register: {}", start)))?
            .into_int_value();

        let limit_val = values
            .get(&limit)
            .ok_or_else(|| BackendError::InvalidIR(format!("Undefined limit register: {}", limit)))?
            .into_int_value();

        let body_bb = blocks
            .get(&body_block)
            .ok_or_else(|| BackendError::InvalidIR(format!("Undefined body block: {}", body_block.0)))?;

        let exit_bb = blocks
            .get(&exit_block)
            .ok_or_else(|| BackendError::InvalidIR(format!("Undefined exit block: {}", exit_block.0)))?;

        // Create loop header block
        let current_fn = builder.get_insert_block().unwrap().get_parent().unwrap();
        let header_bb = self.context.append_basic_block(current_fn, "loop_header");

        // Jump to header from current block
        builder.build_unconditional_branch(header_bb)
            .map_err(|e| BackendError::CodeGenError(e.to_string()))?;

        // Build loop header
        builder.position_at_end(header_bb);
        let i_phi = builder.build_phi(self.context.i64_type(), "i")
            .map_err(|e| BackendError::CodeGenError(e.to_string()))?;
        i_phi.add_incoming(&[(&start_val, builder.get_insert_block().unwrap())]);

        // Store loop counter for body
        values.insert(loop_counter, i_phi.as_basic_value());

        // Compare i < limit
        let cond = builder.build_int_compare(
            inkwell::IntPredicate::SLT,
            i_phi.as_basic_value().into_int_value(),
            limit_val,
            "loop_cond"
        ).map_err(|e| BackendError::CodeGenError(e.to_string()))?;

        builder.build_conditional_branch(cond, *body_bb, *exit_bb)
            .map_err(|e| BackendError::CodeGenError(e.to_string()))?;

        Ok(())
    }

    /// Generate BEGIN/UNTIL loop
    ///
    /// ```forth
    /// BEGIN
    ///   ... body ...
    ///   condition
    /// UNTIL
    /// ```
    ///
    /// LLVM IR:
    /// ```llvm
    /// entry:
    ///   br label %loop_body
    ///
    /// loop_body:
    ///   ... body ...
    ///   %cond = ...
    ///   br i1 %cond, label %loop_exit, label %loop_body
    ///
    /// loop_exit:
    ///   ...
    /// ```
    pub fn generate_begin_until(
        &self,
        builder: &Builder<'ctx>,
        values: &HashMap<Register, BasicValueEnum<'ctx>>,
        blocks: &HashMap<BlockId, BasicBlock<'ctx>>,
        body_block: BlockId,
        exit_block: BlockId,
        condition: Register,
    ) -> Result<()> {
        let body_bb = blocks
            .get(&body_block)
            .ok_or_else(|| BackendError::InvalidIR(format!("Undefined body block: {}", body_block.0)))?;

        let exit_bb = blocks
            .get(&exit_block)
            .ok_or_else(|| BackendError::InvalidIR(format!("Undefined exit block: {}", exit_block.0)))?;

        // Jump to body
        builder.build_unconditional_branch(*body_bb)
            .map_err(|e| BackendError::CodeGenError(e.to_string()))?;

        // At end of body, check condition
        builder.position_at_end(*body_bb);

        let cond_val = values
            .get(&condition)
            .ok_or_else(|| BackendError::InvalidIR(format!("Undefined condition register: {}", condition)))?;

        let cond_bool = if cond_val.is_int_value() {
            let int_val = cond_val.into_int_value();
            if int_val.get_type().get_bit_width() == 1 {
                int_val
            } else {
                let zero = int_val.get_type().const_zero();
                builder.build_int_compare(
                    inkwell::IntPredicate::NE,
                    int_val,
                    zero,
                    "until_cond"
                ).map_err(|e| BackendError::CodeGenError(e.to_string()))?
            }
        } else {
            return Err(BackendError::CodeGenError("Condition must be integer value".to_string()));
        };

        // Branch: if condition true, exit; else repeat
        builder.build_conditional_branch(cond_bool, *exit_bb, *body_bb)
            .map_err(|e| BackendError::CodeGenError(e.to_string()))?;

        Ok(())
    }

    /// Generate BEGIN/WHILE/REPEAT loop
    ///
    /// ```forth
    /// BEGIN
    ///   condition
    /// WHILE
    ///   ... body ...
    /// REPEAT
    /// ```
    ///
    /// LLVM IR:
    /// ```llvm
    /// entry:
    ///   br label %loop_cond
    ///
    /// loop_cond:
    ///   %cond = ...
    ///   br i1 %cond, label %loop_body, label %loop_exit
    ///
    /// loop_body:
    ///   ... body ...
    ///   br label %loop_cond
    ///
    /// loop_exit:
    ///   ...
    /// ```
    pub fn generate_begin_while_repeat(
        &self,
        builder: &Builder<'ctx>,
        values: &HashMap<Register, BasicValueEnum<'ctx>>,
        blocks: &HashMap<BlockId, BasicBlock<'ctx>>,
        cond_block: BlockId,
        body_block: BlockId,
        exit_block: BlockId,
        condition: Register,
    ) -> Result<()> {
        let cond_bb = blocks
            .get(&cond_block)
            .ok_or_else(|| BackendError::InvalidIR(format!("Undefined condition block: {}", cond_block.0)))?;

        let body_bb = blocks
            .get(&body_block)
            .ok_or_else(|| BackendError::InvalidIR(format!("Undefined body block: {}", body_block.0)))?;

        let exit_bb = blocks
            .get(&exit_block)
            .ok_or_else(|| BackendError::InvalidIR(format!("Undefined exit block: {}", exit_block.0)))?;

        // Jump to condition check
        builder.build_unconditional_branch(*cond_bb)
            .map_err(|e| BackendError::CodeGenError(e.to_string()))?;

        // Position at condition block
        builder.position_at_end(*cond_bb);

        let cond_val = values
            .get(&condition)
            .ok_or_else(|| BackendError::InvalidIR(format!("Undefined condition register: {}", condition)))?;

        let cond_bool = if cond_val.is_int_value() {
            let int_val = cond_val.into_int_value();
            if int_val.get_type().get_bit_width() == 1 {
                int_val
            } else {
                let zero = int_val.get_type().const_zero();
                builder.build_int_compare(
                    inkwell::IntPredicate::NE,
                    int_val,
                    zero,
                    "while_cond"
                ).map_err(|e| BackendError::CodeGenError(e.to_string()))?
            }
        } else {
            return Err(BackendError::CodeGenError("Condition must be integer value".to_string()));
        };

        // Branch: if condition true, execute body; else exit
        builder.build_conditional_branch(cond_bool, *body_bb, *exit_bb)
            .map_err(|e| BackendError::CodeGenError(e.to_string()))?;

        Ok(())
    }

    /// Generate tail call optimization
    ///
    /// Replaces function call at tail position with jump to avoid stack growth
    pub fn generate_tail_call(
        &self,
        builder: &Builder<'ctx>,
        function: inkwell::values::FunctionValue<'ctx>,
        args: &[BasicValueEnum<'ctx>],
    ) -> Result<BasicValueEnum<'ctx>> {
        let call = builder.build_call(function, args, "tail_call")
            .map_err(|e| BackendError::CodeGenError(e.to_string()))?;

        // Set tail call attribute
        call.set_tail_call(true);

        Ok(call.try_as_basic_value().left().unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_control_flow_codegen_creation() {
        let context = Context::create();
        let _cf_codegen = ControlFlowCodegen::new(&context);
    }
}
