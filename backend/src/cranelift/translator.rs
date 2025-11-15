//! SSA to Cranelift IR Translation
//!
//! Translates Fast Forth SSA representation to Cranelift IR for compilation.

use crate::error::{BackendError, Result};
use fastforth_frontend::ssa::{
    SSAFunction, SSAInstruction, Register, BlockId, BinaryOperator, UnaryOperator, BasicBlock,
};
use fastforth_frontend::ast::StackType;

use cranelift_codegen::ir::{
    types, AbiParam, Block, Function, FuncRef, InstBuilder, Value,
};
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext, Variable};

use std::collections::HashMap;

/// Information about Phi nodes for a block
#[derive(Debug, Clone)]
struct PhiInfo {
    /// Register that receives the merged value
    dest: Register,
    /// Incoming values: (predecessor_block, source_register)
    incoming: Vec<(BlockId, Register)>,
}

/// Translator from Fast Forth SSA to Cranelift IR
pub struct SSATranslator<'a> {
    builder: FunctionBuilder<'a>,
    /// Map Fast Forth registers to Cranelift values (changed from Variables)
    register_values: HashMap<Register, Value>,
    /// Map Fast Forth blocks to Cranelift blocks
    block_map: HashMap<BlockId, Block>,
    /// Map of blocks to their Phi nodes
    phi_nodes: HashMap<BlockId, Vec<PhiInfo>>,
    /// Current block being translated
    current_block: Option<BlockId>,
    /// Map of function names to FuncRefs (pre-imported)
    func_refs: &'a HashMap<String, FuncRef>,
    /// Map of FFI function names to FuncRefs (pre-imported)
    ffi_refs: &'a HashMap<String, FuncRef>,
}

impl<'a> SSATranslator<'a> {
    /// Create new translator
    pub fn new(
        func: &'a mut Function,
        builder_ctx: &'a mut FunctionBuilderContext,
        func_refs: &'a HashMap<String, FuncRef>,
        ffi_refs: &'a HashMap<String, FuncRef>,
    ) -> Self {
        let builder = FunctionBuilder::new(func, builder_ctx);

        Self {
            builder,
            register_values: HashMap::new(),
            block_map: HashMap::new(),
            phi_nodes: HashMap::new(),
            current_block: None,
            func_refs,
            ffi_refs,
        }
    }

    /// Analyze Phi nodes in the SSA function
    fn analyze_phi_nodes(&mut self, ssa_func: &SSAFunction) {
        for block in &ssa_func.blocks {
            for inst in &block.instructions {
                if let SSAInstruction::Phi { dest, incoming } = inst {
                    let phi_info = PhiInfo {
                        dest: *dest,
                        incoming: incoming.clone(),
                    };
                    self.phi_nodes.entry(block.id)
                        .or_insert_with(Vec::new)
                        .push(phi_info);
                }
            }
        }
    }

    /// Translate entire SSA function to Cranelift IR
    pub fn translate(mut self, ssa_func: &SSAFunction) -> Result<()> {
        // First pass: analyze Phi nodes to determine block parameters
        self.analyze_phi_nodes(ssa_func);

        // Create Cranelift blocks for all SSA blocks
        for block in &ssa_func.blocks {
            let cl_block = self.builder.create_block();
            self.block_map.insert(block.id, cl_block);

            // First block is entry block - add parameters
            if block.id == ssa_func.entry_block {
                self.builder.append_block_params_for_function_params(cl_block);
            } else if let Some(phi_infos) = self.phi_nodes.get(&block.id) {
                // Add block parameters for Phi nodes
                for _ in phi_infos {
                    self.builder.append_block_param(cl_block, types::I64);
                }
            }
        }

        // Switch to entry block
        let entry_block = self.block_map[&ssa_func.entry_block];
        self.builder.switch_to_block(entry_block);

        // Map function parameters to registers
        for (i, &param_reg) in ssa_func.parameters.iter().enumerate() {
            let value = self.builder.block_params(entry_block)[i];
            self.register_values.insert(param_reg, value);
        }

        // Translate each block
        for block in &ssa_func.blocks {
            self.translate_block(block)?;
        }

        // Seal all blocks (required by Cranelift)
        for &cl_block in self.block_map.values() {
            self.builder.seal_block(cl_block);
        }

        // Finalize function
        self.builder.finalize();

        Ok(())
    }

    /// Translate a single basic block
    fn translate_block(&mut self, block: &BasicBlock) -> Result<()> {
        let cl_block = self.block_map[&block.id];
        self.builder.switch_to_block(cl_block);

        // Set current block for branch/jump target resolution
        self.current_block = Some(block.id);

        // Handle block parameters for Phi nodes
        if let Some(phi_infos) = self.phi_nodes.get(&block.id).cloned() {
            let block_params = self.builder.block_params(cl_block).to_vec();
            for (i, phi_info) in phi_infos.iter().enumerate() {
                if i < block_params.len() {
                    self.register_values.insert(phi_info.dest, block_params[i]);
                }
            }
        }

        for inst in &block.instructions {
            self.translate_instruction(inst)?;
        }

        Ok(())
    }

    /// Translate a single SSA instruction
    fn translate_instruction(&mut self, inst: &SSAInstruction) -> Result<()> {
        match inst {
            SSAInstruction::LoadInt { dest, value } => {
                let val = self.builder.ins().iconst(types::I64, *value);
                self.register_values.insert(*dest, val);
            }

            SSAInstruction::LoadFloat { dest, value } => {
                let val = self.builder.ins().f64const(*value);
                self.register_values.insert(*dest, val);
            }

            SSAInstruction::BinaryOp { dest, op, left, right } => {
                let left_val = self.get_register(*left)?;
                let right_val = self.get_register(*right)?;

                let result = match op {
                    BinaryOperator::Add => self.builder.ins().iadd(left_val, right_val),
                    BinaryOperator::Sub => self.builder.ins().isub(left_val, right_val),
                    BinaryOperator::Mul => self.builder.ins().imul(left_val, right_val),
                    BinaryOperator::Div => self.builder.ins().sdiv(left_val, right_val),
                    BinaryOperator::Mod => self.builder.ins().srem(left_val, right_val),
                    BinaryOperator::Lt => {
                        let cmp = self.builder.ins().icmp(
                            cranelift_codegen::ir::condcodes::IntCC::SignedLessThan,
                            left_val,
                            right_val,
                        );
                        self.builder.ins().uextend(types::I64, cmp)
                    }
                    BinaryOperator::Gt => {
                        let cmp = self.builder.ins().icmp(
                            cranelift_codegen::ir::condcodes::IntCC::SignedGreaterThan,
                            left_val,
                            right_val,
                        );
                        self.builder.ins().uextend(types::I64, cmp)
                    }
                    BinaryOperator::Le => {
                        let cmp = self.builder.ins().icmp(
                            cranelift_codegen::ir::condcodes::IntCC::SignedLessThanOrEqual,
                            left_val,
                            right_val,
                        );
                        self.builder.ins().uextend(types::I64, cmp)
                    }
                    BinaryOperator::Ge => {
                        let cmp = self.builder.ins().icmp(
                            cranelift_codegen::ir::condcodes::IntCC::SignedGreaterThanOrEqual,
                            left_val,
                            right_val,
                        );
                        self.builder.ins().uextend(types::I64, cmp)
                    }
                    BinaryOperator::Eq => {
                        let cmp = self.builder.ins().icmp(
                            cranelift_codegen::ir::condcodes::IntCC::Equal,
                            left_val,
                            right_val,
                        );
                        self.builder.ins().uextend(types::I64, cmp)
                    }
                    BinaryOperator::Ne => {
                        let cmp = self.builder.ins().icmp(
                            cranelift_codegen::ir::condcodes::IntCC::NotEqual,
                            left_val,
                            right_val,
                        );
                        self.builder.ins().uextend(types::I64, cmp)
                    }
                    BinaryOperator::And => self.builder.ins().band(left_val, right_val),
                    BinaryOperator::Or => self.builder.ins().bor(left_val, right_val),
                };

                self.register_values.insert(*dest, result);
            }

            SSAInstruction::UnaryOp { dest, op, operand } => {
                let operand_val = self.get_register(*operand)?;

                let result = match op {
                    UnaryOperator::Negate => {
                        let zero = self.builder.ins().iconst(types::I64, 0);
                        self.builder.ins().isub(zero, operand_val)
                    }
                    UnaryOperator::Not => {
                        let all_ones = self.builder.ins().iconst(types::I64, -1);
                        self.builder.ins().bxor(operand_val, all_ones)
                    }
                    UnaryOperator::Abs => {
                        // abs(x) = (x < 0) ? -x : x
                        let zero = self.builder.ins().iconst(types::I64, 0);
                        let is_neg = self.builder.ins().icmp(
                            cranelift_codegen::ir::condcodes::IntCC::SignedLessThan,
                            operand_val,
                            zero,
                        );
                        let negated = self.builder.ins().isub(zero, operand_val);
                        self.builder.ins().select(is_neg, negated, operand_val)
                    }
                };

                self.register_values.insert(*dest, result);
            }

            SSAInstruction::Load { dest, address, ty } => {
                let addr_val = self.get_register(*address)?;

                use cranelift_codegen::ir::MemFlags;

                let result = match ty {
                    StackType::Int | StackType::Addr => {
                        self.builder.ins().load(types::I64, MemFlags::new(), addr_val, 0)
                    }
                    StackType::Float => {
                        self.builder.ins().load(types::F64, MemFlags::new(), addr_val, 0)
                    }
                    StackType::Bool | StackType::Char => {
                        self.builder.ins().load(types::I8, MemFlags::new(), addr_val, 0)
                    }
                    StackType::String | StackType::Var(_) | StackType::Unknown => {
                        // For unknown or complex types, default to I64
                        self.builder.ins().load(types::I64, MemFlags::new(), addr_val, 0)
                    }
                };

                self.register_values.insert(*dest, result);
            }

            SSAInstruction::Store { address, value, ty } => {
                use cranelift_codegen::ir::MemFlags;

                let addr_val = self.get_register(*address)?;
                let val = self.get_register(*value)?;

                self.builder.ins().store(MemFlags::new(), val, addr_val, 0);
            }

            SSAInstruction::Branch { condition, true_block, false_block } => {
                let cond_val = self.get_register(*condition)?;
                let true_cl_block = self.block_map[true_block];
                let false_cl_block = self.block_map[false_block];

                // Convert i64 to i1 for branch condition
                let zero = self.builder.ins().iconst(types::I64, 0);
                let cond_bool = self.builder.ins().icmp(
                    cranelift_codegen::ir::condcodes::IntCC::NotEqual,
                    cond_val,
                    zero,
                );

                // Branch instruction doesn't pass arguments - the then/else blocks
                // will jump to the merge block with the appropriate arguments
                self.builder.ins().brif(cond_bool, true_cl_block, &[], false_cl_block, &[]);
            }

            SSAInstruction::Jump { target } => {
                let cl_block = self.block_map[target];

                // Get current block for argument resolution
                let from_block = self.current_block.ok_or_else(|| BackendError::CodeGeneration(
                    "Jump instruction outside of block context".to_string()
                ))?;

                // Collect arguments based on target block's Phi nodes
                let args = self.collect_branch_args(*target, &from_block)?;

                self.builder.ins().jump(cl_block, &args);
            }

            SSAInstruction::Return { values } => {
                let return_vals: Vec<Value> = values
                    .iter()
                    .map(|&reg| self.get_register(reg))
                    .collect::<Result<Vec<_>>>()?;

                self.builder.ins().return_(&return_vals);
            }

            SSAInstruction::Call { dest, name, args } => {
                // Look up the pre-imported function reference
                let func_ref = self.func_refs.get(name)
                    .copied()
                    .ok_or_else(|| BackendError::CodeGeneration(
                        format!("Function '{}' not declared/imported", name)
                    ))?;

                // Convert arguments to Cranelift values
                let arg_values: Vec<Value> = args
                    .iter()
                    .map(|&reg| self.get_register(reg))
                    .collect::<Result<Vec<_>>>()?;

                // Emit the call instruction
                let call = self.builder.ins().call(func_ref, &arg_values);

                // Get the return values and convert to Vec to avoid borrow conflicts
                let results: Vec<Value> = self.builder.inst_results(call).to_vec();

                // Map return values to destination registers
                for (i, &dest_reg) in dest.iter().enumerate() {
                    if i < results.len() {
                        self.register_values.insert(dest_reg, results[i]);
                    } else {
                        return Err(BackendError::CodeGeneration(
                            format!("Function '{}' returned fewer values than expected", name)
                        ));
                    }
                }
            }

            SSAInstruction::Phi { dest, incoming } => {
                // Phi nodes are now handled via block parameters.
                // The destination register was already set when we entered the block.
                // Just skip this instruction - nothing to do here.
            }

            SSAInstruction::LoadString { dest, value } => {
                // String handling requires more complex setup
                // For now, we'll return an error
                return Err(BackendError::CodeGeneration(
                    "String literals not yet supported in Cranelift backend".to_string()
                ));
            }

            // FFI and File I/O Operations
            // These are placeholders for now - full implementation in Phase 5.2
            SSAInstruction::FFICall { dest, function, args } => {
                return Err(BackendError::CodeGeneration(
                    format!("FFI call to '{}' not yet implemented", function)
                ));
            }

            SSAInstruction::FileOpen { dest_fileid, dest_ior, path_addr, path_len, mode } => {
                return Err(BackendError::CodeGeneration(
                    "File open operation not yet implemented".to_string()
                ));
            }

            SSAInstruction::FileRead { dest_bytes, dest_ior, buffer, count, fileid } => {
                return Err(BackendError::CodeGeneration(
                    "File read operation not yet implemented".to_string()
                ));
            }

            SSAInstruction::FileWrite { dest_ior, buffer, count, fileid } => {
                return Err(BackendError::CodeGeneration(
                    "File write operation not yet implemented".to_string()
                ));
            }

            SSAInstruction::FileClose { dest_ior, fileid } => {
                return Err(BackendError::CodeGeneration(
                    "File close operation not yet implemented".to_string()
                ));
            }

            SSAInstruction::FileDelete { dest_ior, path_addr, path_len } => {
                return Err(BackendError::CodeGeneration(
                    "File delete operation not yet implemented".to_string()
                ));
            }

            SSAInstruction::FileCreate { dest_fileid, dest_ior, path_addr, path_len, mode } => {
                return Err(BackendError::CodeGeneration(
                    "File create operation not yet implemented".to_string()
                ));
            }

            SSAInstruction::SystemCall { dest, command_addr, command_len } => {
                return Err(BackendError::CodeGeneration(
                    "System call operation not yet implemented".to_string()
                ));
            }
        }

        Ok(())
    }

    /// Get the Cranelift value for a Fast Forth register
    fn get_register(&self, reg: Register) -> Result<Value> {
        self.register_values.get(&reg)
            .copied()
            .ok_or_else(|| BackendError::CodeGeneration(
                format!("Register {:?} not defined", reg)
            ))
    }

    /// Collect arguments for a branch based on target block's Phi nodes
    fn collect_branch_args(&self, target_block: BlockId, from_block: &BlockId) -> Result<Vec<Value>> {
        if let Some(phi_infos) = self.phi_nodes.get(&target_block) {
            let mut args = Vec::new();
            for phi_info in phi_infos.iter() {
                // Find the incoming value from our current block
                let incoming_reg = phi_info.incoming.iter()
                    .find(|(block_id, _)| block_id == from_block)
                    .map(|(_, reg)| reg)
                    .ok_or_else(|| BackendError::CodeGeneration(
                        format!("Phi node in block {:?} missing incoming value from block {:?}",
                                target_block, from_block)
                    ))?;

                let value = self.get_register(*incoming_reg)?;
                args.push(value);
            }
            Ok(args)
        } else {
            // No Phi nodes, no arguments needed
            Ok(Vec::new())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test disabled - translator now requires module and functions map
    // These are tested through integration tests in cli/execute.rs
    /*
    #[test]
    fn test_translator_creation() {
        let mut func = Function::new();
        let mut builder_ctx = FunctionBuilderContext::new();
        let _translator = SSATranslator::new(&mut func, &mut builder_ctx);
    }
    */
}
