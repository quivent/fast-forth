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
    /// Actual control flow graph: tracks which blocks jump to which blocks
    /// This is built during translation and may differ from SSA Phi predecessors
    block_predecessors: HashMap<BlockId, Vec<BlockId>>,
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
            block_predecessors: HashMap::new(),
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

                // Track control flow edges
                let from_block = self.current_block.ok_or_else(|| BackendError::CodeGeneration(
                    "Branch instruction outside of block context".to_string()
                ))?;
                self.block_predecessors.entry(*true_block).or_insert_with(Vec::new).push(from_block);
                self.block_predecessors.entry(*false_block).or_insert_with(Vec::new).push(from_block);

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

                // Track control flow edge
                self.block_predecessors.entry(*target).or_insert_with(Vec::new).push(from_block);

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

            SSAInstruction::LoadString { dest_addr, dest_len, value } => {
                // For now, implement a simplified version that uses runtime allocation
                // This calls malloc to allocate memory for the string

                let len = value.len() as i64;
                let len_val = self.builder.ins().iconst(types::I64, len);
                self.register_values.insert(*dest_len, len_val);

                // Allocate memory using malloc (len + 1 for null terminator)
                let malloc_ref = self.ffi_refs.get("malloc")
                    .copied()
                    .ok_or_else(|| BackendError::CodeGeneration(
                        "malloc not found (required for string literals)".to_string()
                    ))?;

                let alloc_size = self.builder.ins().iconst(types::I64, len + 1);
                let call = self.builder.ins().call(malloc_ref, &[alloc_size]);
                let addr = self.builder.inst_results(call)[0];
                self.register_values.insert(*dest_addr, addr);

                // Store each byte of the string
                for (i, &byte) in value.as_bytes().iter().enumerate() {
                    let byte_val = self.builder.ins().iconst(types::I8, byte as i64);
                    let offset = self.builder.ins().iconst(types::I64, i as i64);
                    let byte_addr = self.builder.ins().iadd(addr, offset);
                    self.builder.ins().store(
                        cranelift_codegen::ir::MemFlags::new(),
                        byte_val,
                        byte_addr,
                        0,
                    );
                }

                // Add null terminator
                let null = self.builder.ins().iconst(types::I8, 0);
                let null_offset = self.builder.ins().iconst(types::I64, len);
                let null_addr = self.builder.ins().iadd(addr, null_offset);
                self.builder.ins().store(
                    cranelift_codegen::ir::MemFlags::new(),
                    null,
                    null_addr,
                    0,
                );
            }

            // FFI and File I/O Operations
            SSAInstruction::FFICall { dest, function, args } => {
                // Look up the FFI function reference
                let ffi_ref = self.ffi_refs.get(function)
                    .copied()
                    .ok_or_else(|| BackendError::CodeGeneration(
                        format!("FFI function '{}' not registered", function)
                    ))?;

                // Convert arguments to Cranelift values
                let arg_values: Vec<Value> = args
                    .iter()
                    .map(|&reg| self.get_register(reg))
                    .collect::<Result<Vec<_>>>()?;

                // Emit the FFI call
                let call = self.builder.ins().call(ffi_ref, &arg_values);

                // Get return values
                let results: Vec<Value> = self.builder.inst_results(call).to_vec();

                // Map return values to destination registers
                for (i, &dest_reg) in dest.iter().enumerate() {
                    if i < results.len() {
                        self.register_values.insert(dest_reg, results[i]);
                    }
                }
            }

            SSAInstruction::FileOpen { dest_fileid, dest_ior, path_addr, path_len, mode } => {
                // Get fopen FFI function reference
                let fopen_ref = self.ffi_refs.get("fopen")
                    .copied()
                    .ok_or_else(|| BackendError::CodeGeneration(
                        "FFI function 'fopen' not registered".to_string()
                    ))?;

                // Get path pointer and mode
                let path_ptr = self.get_register(*path_addr)?;
                let mode_val = self.get_register(*mode)?;

                // Convert Forth mode to C mode string pointer
                // Mode: 0=r/o ("r"), 1=w/o ("w"), 2=r/w ("r+")
                // Note: This requires pre-allocated mode strings in memory
                // For now, we'll assume mode_val is already a pointer to C string
                let mode_ptr = mode_val;

                // Call fopen(path, mode)
                let call = self.builder.ins().call(fopen_ref, &[path_ptr, mode_ptr]);
                let file_handle = self.builder.inst_results(call)[0];

                // Check if NULL (error): file_handle == 0
                let null_ptr = self.builder.ins().iconst(types::I64, 0);
                let is_null = self.builder.ins().icmp(
                    cranelift_codegen::ir::condcodes::IntCC::Equal,
                    file_handle,
                    null_ptr,
                );

                // Set ior: 0 = success, -1 = error (following ANS Forth convention)
                let success = self.builder.ins().iconst(types::I64, 0);
                let error = self.builder.ins().iconst(types::I64, -1);
                let ior = self.builder.ins().select(is_null, error, success);

                // Store results
                self.register_values.insert(*dest_fileid, file_handle);
                self.register_values.insert(*dest_ior, ior);
            }

            SSAInstruction::FileRead { dest_bytes, dest_ior, buffer, count, fileid } => {
                // Get fread FFI function reference
                let fread_ref = self.ffi_refs.get("fread")
                    .copied()
                    .ok_or_else(|| BackendError::CodeGeneration(
                        "FFI function 'fread' not registered".to_string()
                    ))?;

                // Get arguments
                let buffer_ptr = self.get_register(*buffer)?;
                let count_val = self.get_register(*count)?;
                let file_handle = self.get_register(*fileid)?;

                // fread(buffer, 1, count, file) - read count bytes
                let size = self.builder.ins().iconst(types::I64, 1);

                // Call fread
                let call = self.builder.ins().call(
                    fread_ref,
                    &[buffer_ptr, size, count_val, file_handle]
                );
                let bytes_read = self.builder.inst_results(call)[0];

                // Check for error: bytes_read < count means error or EOF
                // For simplicity, ior = 0 (success), actual error checking done by user
                let success = self.builder.ins().iconst(types::I64, 0);

                // Store results
                self.register_values.insert(*dest_bytes, bytes_read);
                self.register_values.insert(*dest_ior, success);
            }

            SSAInstruction::FileWrite { dest_ior, buffer, count, fileid } => {
                // Get fwrite FFI function reference
                let fwrite_ref = self.ffi_refs.get("fwrite")
                    .copied()
                    .ok_or_else(|| BackendError::CodeGeneration(
                        "FFI function 'fwrite' not registered".to_string()
                    ))?;

                // Get arguments
                let buffer_ptr = self.get_register(*buffer)?;
                let count_val = self.get_register(*count)?;
                let file_handle = self.get_register(*fileid)?;

                // fwrite(buffer, 1, count, file) - write count bytes
                let size = self.builder.ins().iconst(types::I64, 1);

                // Call fwrite
                let call = self.builder.ins().call(
                    fwrite_ref,
                    &[buffer_ptr, size, count_val, file_handle]
                );
                let bytes_written = self.builder.inst_results(call)[0];

                // Check if all bytes were written
                let is_complete = self.builder.ins().icmp(
                    cranelift_codegen::ir::condcodes::IntCC::Equal,
                    bytes_written,
                    count_val,
                );

                // Set ior: 0 = success, -1 = error
                let success = self.builder.ins().iconst(types::I64, 0);
                let error = self.builder.ins().iconst(types::I64, -1);
                let ior = self.builder.ins().select(is_complete, success, error);

                // Store result
                self.register_values.insert(*dest_ior, ior);
            }

            SSAInstruction::FileClose { dest_ior, fileid } => {
                // Get fclose FFI function reference
                let fclose_ref = self.ffi_refs.get("fclose")
                    .copied()
                    .ok_or_else(|| BackendError::CodeGeneration(
                        "FFI function 'fclose' not registered".to_string()
                    ))?;

                // Get file handle
                let file_handle = self.get_register(*fileid)?;

                // Call fclose(file)
                let call = self.builder.ins().call(fclose_ref, &[file_handle]);
                let result = self.builder.inst_results(call)[0];

                // Convert i32 result to i64 for consistency
                let result_i64 = self.builder.ins().sextend(types::I64, result);

                // fclose returns 0 on success, EOF (-1) on error
                // We keep the same convention: 0 = success, non-zero = error
                self.register_values.insert(*dest_ior, result_i64);
            }

            SSAInstruction::FileDelete { dest_ior, path_addr, path_len } => {
                // Get remove FFI function reference
                let remove_ref = self.ffi_refs.get("remove")
                    .copied()
                    .ok_or_else(|| BackendError::CodeGeneration(
                        "FFI function 'remove' not registered".to_string()
                    ))?;

                // Get path pointer
                let path_ptr = self.get_register(*path_addr)?;

                // Call remove(path)
                let call = self.builder.ins().call(remove_ref, &[path_ptr]);
                let result = self.builder.inst_results(call)[0];

                // Convert i32 result to i64
                let result_i64 = self.builder.ins().sextend(types::I64, result);

                // remove returns 0 on success, non-zero on error
                self.register_values.insert(*dest_ior, result_i64);
            }

            SSAInstruction::FileCreate { dest_fileid, dest_ior, path_addr, path_len, mode } => {
                // FileCreate is the same as FileOpen with create mode
                // Get fopen FFI function reference
                let fopen_ref = self.ffi_refs.get("fopen")
                    .copied()
                    .ok_or_else(|| BackendError::CodeGeneration(
                        "FFI function 'fopen' not registered".to_string()
                    ))?;

                // Get path pointer and mode
                let path_ptr = self.get_register(*path_addr)?;
                let mode_val = self.get_register(*mode)?;

                // For create, mode should be "w" (write) or "w+" (read-write)
                // Assume mode_val is already a pointer to C string
                let mode_ptr = mode_val;

                // Call fopen(path, mode)
                let call = self.builder.ins().call(fopen_ref, &[path_ptr, mode_ptr]);
                let file_handle = self.builder.inst_results(call)[0];

                // Check if NULL (error)
                let null_ptr = self.builder.ins().iconst(types::I64, 0);
                let is_null = self.builder.ins().icmp(
                    cranelift_codegen::ir::condcodes::IntCC::Equal,
                    file_handle,
                    null_ptr,
                );

                // Set ior: 0 = success, -1 = error
                let success = self.builder.ins().iconst(types::I64, 0);
                let error = self.builder.ins().iconst(types::I64, -1);
                let ior = self.builder.ins().select(is_null, error, success);

                // Store results
                self.register_values.insert(*dest_fileid, file_handle);
                self.register_values.insert(*dest_ior, ior);
            }

            SSAInstruction::SystemCall { dest, command_addr, command_len } => {
                // Get system FFI function reference
                let system_ref = self.ffi_refs.get("system")
                    .copied()
                    .ok_or_else(|| BackendError::CodeGeneration(
                        "FFI function 'system' not registered".to_string()
                    ))?;

                // Get command pointer
                let command_ptr = self.get_register(*command_addr)?;

                // Call system(command)
                let call = self.builder.ins().call(system_ref, &[command_ptr]);
                let result = self.builder.inst_results(call)[0];

                // Convert i32 result to i64
                let result_i64 = self.builder.ins().sextend(types::I64, result);

                // system returns exit code of command
                // Store in destination register
                self.register_values.insert(*dest, result_i64);
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
                // Try to find the incoming value from our current block directly
                if let Some((_, reg)) = phi_info.incoming.iter()
                    .find(|(block_id, _)| block_id == from_block)
                {
                    let value = self.get_register(*reg)?;
                    args.push(value);
                } else {
                    // from_block is not a direct predecessor in the Phi node.
                    // This happens with nested if-then-else where an inner merge block
                    // jumps to an outer merge block.
                    //
                    // The SSA generation creates Phi nodes based on abstract control flow
                    // (e.g., "value from outer then" vs "value from outer else"), but the
                    // actual execution creates intermediate merge blocks (inner merge) that
                    // jump to the outer merge.
                    //
                    // Solution: Use the Phi destination from from_block. This represents
                    // the merged value that should be passed forward.

                    if let Some(from_phi_infos) = self.phi_nodes.get(from_block) {
                        // Use the first Phi's destination
                        // In the common case of nested if-then-else, there's typically one
                        // Phi per value being merged
                        if let Some(from_phi) = from_phi_infos.first() {
                            let value = self.get_register(from_phi.dest)?;
                            args.push(value);
                        } else {
                            return Err(BackendError::CodeGeneration(
                                format!(
                                    "Phi node in block {:?} missing incoming value from block {:?}. \
                                     Block {:?} has no Phi nodes.",
                                    target_block, from_block, from_block
                                )
                            ));
                        }
                    } else {
                        // from_block has no Phi nodes - this is an error
                        return Err(BackendError::CodeGeneration(
                            format!(
                                "Phi node in block {:?} missing incoming value from block {:?}. \
                                 Block {:?} is not a direct predecessor and has no Phi nodes to resolve the value. \
                                 Expected predecessors: {:?}",
                                target_block, from_block, from_block,
                                phi_info.incoming.iter().map(|(b, _)| b).collect::<Vec<_>>()
                            )
                        ));
                    }
                }
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
