//! SSA to Cranelift IR Translation
//!
//! Translates Fast Forth SSA representation to Cranelift IR for compilation.

use crate::error::{BackendError, Result};
use fastforth_frontend::ssa::{
    SSAFunction, SSAInstruction, Register, BlockId, BinaryOperator, UnaryOperator, BasicBlock,
};
use fastforth_frontend::ast::StackType;

use cranelift_codegen::ir::{
    types, AbiParam, Block, Function, InstBuilder, Value,
};
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext, Variable};

use std::collections::HashMap;

/// Translator from Fast Forth SSA to Cranelift IR
pub struct SSATranslator<'a> {
    builder: FunctionBuilder<'a>,
    /// Map Fast Forth registers to Cranelift variables
    register_map: HashMap<Register, Variable>,
    /// Map Fast Forth blocks to Cranelift blocks
    block_map: HashMap<BlockId, Block>,
    /// Next variable ID
    next_var: u32,
}

impl<'a> SSATranslator<'a> {
    /// Create new translator
    pub fn new(func: &'a mut Function, builder_ctx: &'a mut FunctionBuilderContext) -> Self {
        let builder = FunctionBuilder::new(func, builder_ctx);

        Self {
            builder,
            register_map: HashMap::new(),
            block_map: HashMap::new(),
            next_var: 0,
        }
    }

    /// Translate entire SSA function to Cranelift IR
    pub fn translate(mut self, ssa_func: &SSAFunction) -> Result<()> {
        // Create Cranelift blocks for all SSA blocks
        for block in &ssa_func.blocks {
            let cl_block = self.builder.create_block();
            self.block_map.insert(block.id, cl_block);

            // First block is entry block - add parameters
            if block.id == ssa_func.entry_block {
                self.builder.append_block_params_for_function_params(cl_block);
            }
        }

        // Switch to entry block
        let entry_block = self.block_map[&ssa_func.entry_block];
        self.builder.switch_to_block(entry_block);

        // Map function parameters to registers
        for (i, &param_reg) in ssa_func.parameters.iter().enumerate() {
            let var = self.fresh_variable();
            self.register_map.insert(param_reg, var);
            let value = self.builder.block_params(entry_block)[i];
            self.builder.declare_var(var, types::I64);
            self.builder.def_var(var, value);
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

        for inst in &block.instructions {
            self.translate_instruction(inst)?;
        }

        Ok(())
    }

    /// Translate a single SSA instruction
    fn translate_instruction(&mut self, inst: &SSAInstruction) -> Result<()> {
        match inst {
            SSAInstruction::LoadInt { dest, value } => {
                let var = self.get_or_create_var(*dest);
                let val = self.builder.ins().iconst(types::I64, *value);
                self.builder.def_var(var, val);
            }

            SSAInstruction::LoadFloat { dest, value } => {
                let var = self.get_or_create_var(*dest);
                let val = self.builder.ins().f64const(*value);
                self.builder.def_var(var, val);
            }

            SSAInstruction::BinaryOp { dest, op, left, right } => {
                let var = self.get_or_create_var(*dest);
                let left_val = self.get_register(*left);
                let right_val = self.get_register(*right);

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

                self.builder.def_var(var, result);
            }

            SSAInstruction::UnaryOp { dest, op, operand } => {
                let var = self.get_or_create_var(*dest);
                let operand_val = self.get_register(*operand);

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

                self.builder.def_var(var, result);
            }

            SSAInstruction::Load { dest, address, ty } => {
                let var = self.get_or_create_var(*dest);
                let addr_val = self.get_register(*address);

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

                self.builder.def_var(var, result);
            }

            SSAInstruction::Store { address, value, ty } => {
                use cranelift_codegen::ir::MemFlags;

                let addr_val = self.get_register(*address);
                let val = self.get_register(*value);

                self.builder.ins().store(MemFlags::new(), val, addr_val, 0);
            }

            SSAInstruction::Branch { condition, true_block, false_block } => {
                let cond_val = self.get_register(*condition);
                let true_cl_block = self.block_map[true_block];
                let false_cl_block = self.block_map[false_block];

                // Convert i64 to i1 for branch condition
                let zero = self.builder.ins().iconst(types::I64, 0);
                let cond_bool = self.builder.ins().icmp(
                    cranelift_codegen::ir::condcodes::IntCC::NotEqual,
                    cond_val,
                    zero,
                );

                self.builder.ins().brif(cond_bool, true_cl_block, &[], false_cl_block, &[]);
            }

            SSAInstruction::Jump { target } => {
                let cl_block = self.block_map[target];
                self.builder.ins().jump(cl_block, &[]);
            }

            SSAInstruction::Return { values } => {
                let return_vals: Vec<Value> = values
                    .iter()
                    .map(|&reg| self.get_register(reg))
                    .collect();

                self.builder.ins().return_(&return_vals);
            }

            SSAInstruction::Call { dest, name, args } => {
                // For now, we'll treat calls as external function calls
                // This requires the function to be declared, which we'll handle later
                return Err(BackendError::CodeGeneration(
                    format!("Function calls not yet supported in Cranelift backend: {}", name)
                ));
            }

            SSAInstruction::Phi { dest, incoming } => {
                // Cranelift handles phi nodes automatically through variables
                // We just need to ensure the variable is declared
                let var = self.get_or_create_var(*dest);

                // The actual phi resolution happens in Cranelift's SSA reconstruction
                // We just need to make sure we use the same variable in all incoming blocks
                // This is already handled by our register_map

                // For now, use the first incoming value as a placeholder
                if let Some((_, reg)) = incoming.first() {
                    let val = self.get_register(*reg);
                    self.builder.def_var(var, val);
                }
            }

            SSAInstruction::LoadString { dest, value } => {
                // String handling requires more complex setup
                // For now, we'll return an error
                return Err(BackendError::CodeGeneration(
                    "String literals not yet supported in Cranelift backend".to_string()
                ));
            }
        }

        Ok(())
    }

    /// Get or create a Cranelift variable for a Fast Forth register
    fn get_or_create_var(&mut self, reg: Register) -> Variable {
        if let Some(&var) = self.register_map.get(&reg) {
            var
        } else {
            let var = self.fresh_variable();
            self.builder.declare_var(var, types::I64);
            self.register_map.insert(reg, var);
            var
        }
    }

    /// Get the Cranelift value for a Fast Forth register
    fn get_register(&mut self, reg: Register) -> Value {
        let var = self.register_map[&reg];
        self.builder.use_var(var)
    }

    /// Create a fresh Cranelift variable
    fn fresh_variable(&mut self) -> Variable {
        let var = Variable::with_u32(self.next_var);
        self.next_var += 1;
        var
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_translator_creation() {
        let mut func = Function::new();
        let mut builder_ctx = FunctionBuilderContext::new();
        let _translator = SSATranslator::new(&mut func, &mut builder_ctx);
    }
}
