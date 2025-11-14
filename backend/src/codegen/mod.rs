//! LLVM IR Code Generation
//!
//! This module provides the main code generator that converts SSA IR
//! from the frontend into LLVM IR for native compilation.

pub mod stack_cache;
pub mod primitives;
pub mod control_flow;
pub mod calling_convention;

use crate::error::{BackendError, Result};
use fastforth_frontend::ssa::{SSAFunction, SSAInstruction, Register, BlockId, BinaryOperator, UnaryOperator};
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::types::{BasicTypeEnum, IntType, FloatType};
use inkwell::values::{BasicValueEnum, FunctionValue, IntValue, FloatValue, PointerValue, BasicValue};
use inkwell::IntPredicate;
use inkwell::FloatPredicate;
use inkwell::{OptimizationLevel, AddressSpace};
use inkwell::targets::{Target, TargetMachine, RelocMode, CodeModel, FileType, InitializationConfig};
use std::collections::HashMap;
use std::path::Path;

pub use stack_cache::StackCache;
pub use primitives::PrimitiveCodegen;
pub use control_flow::ControlFlowCodegen;
pub use calling_convention::{
    CallingConvention, CallingConventionType, ForthCallingConvention,
    FFIBridge, ForthRegister, RegisterAllocator,
};

/// Compilation mode (AOT or JIT)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompilationMode {
    /// Ahead-of-time compilation to object file
    AOT,
    /// Just-in-time compilation in memory
    JIT,
}

/// Code generator trait
pub trait CodeGenerator {
    /// Generate code from SSA function
    fn generate(&mut self, function: &SSAFunction) -> Result<()>;

    /// Finalize and emit code
    fn finalize(&self, path: &Path) -> Result<()>;
}

/// LLVM Backend implementation
pub struct LLVMBackend<'ctx> {
    pub context: &'ctx Context,
    pub module: Module<'ctx>,
    pub builder: Builder<'ctx>,

    /// Stack cache for register allocation
    pub stack_cache: StackCache<'ctx>,

    /// Primitive operation codegen
    pub primitives: PrimitiveCodegen<'ctx>,

    /// Control flow codegen
    pub control_flow: ControlFlowCodegen<'ctx>,

    /// Calling convention
    pub calling_convention: ForthCallingConvention,

    /// FFI bridge for C interop
    pub ffi_bridge: FFIBridge<'ctx>,

    /// Value mapping: SSA Register -> LLVM Value
    values: HashMap<Register, BasicValueEnum<'ctx>>,

    /// Basic block mapping
    blocks: HashMap<BlockId, inkwell::basic_block::BasicBlock<'ctx>>,

    /// Current function being compiled
    current_function: Option<FunctionValue<'ctx>>,

    /// Compilation mode
    mode: CompilationMode,

    /// Optimization level
    opt_level: OptimizationLevel,
}

impl<'ctx> LLVMBackend<'ctx> {
    /// Create a new LLVM backend
    pub fn new(
        context: &'ctx Context,
        module_name: &str,
        mode: CompilationMode,
        opt_level: OptimizationLevel,
    ) -> Self {
        let module = context.create_module(module_name);
        let builder = context.create_builder();

        let ffi_bridge = FFIBridge::new(context, &module);

        Self {
            context,
            module,
            builder,
            stack_cache: StackCache::new(context, 3), // Keep top 3 stack items in registers
            primitives: PrimitiveCodegen::new(context),
            control_flow: ControlFlowCodegen::new(context),
            calling_convention: ForthCallingConvention::internal(),
            ffi_bridge,
            values: HashMap::new(),
            blocks: HashMap::new(),
            current_function: None,
            mode,
            opt_level,
        }
    }

    /// Get LLVM type for i64 (cell_t)
    pub fn cell_type(&self) -> IntType<'ctx> {
        self.context.i64_type()
    }

    /// Get LLVM type for f64
    pub fn float_type(&self) -> FloatType<'ctx> {
        self.context.f64_type()
    }

    /// Get LLVM type for boolean (i1)
    pub fn bool_type(&self) -> IntType<'ctx> {
        self.context.bool_type()
    }

    /// Get LLVM type for pointer
    pub fn ptr_type(&self) -> inkwell::types::PointerType<'ctx> {
        self.context.ptr_type(AddressSpace::default())
    }

    /// Create LLVM function from SSA function signature
    fn create_function(&mut self, ssa_func: &SSAFunction) -> Result<FunctionValue<'ctx>> {
        // Create parameter types (all i64 for now)
        let param_types: Vec<BasicTypeEnum> = ssa_func
            .parameters
            .iter()
            .map(|_| self.cell_type().into())
            .collect();

        // For now, assume single return value
        let ret_type = self.cell_type();

        // Create function type
        let fn_type = ret_type.fn_type(&param_types, false);

        // Add function to module
        let function = self.module.add_function(&ssa_func.name, fn_type, None);

        // Set parameter names
        for (i, param) in function.get_param_iter().enumerate() {
            if let Some(reg) = ssa_func.parameters.get(i) {
                param.set_name(&format!("param_{}", reg.0));
            }
        }

        Ok(function)
    }

    /// Create basic blocks for the function
    fn create_basic_blocks(
        &mut self,
        function: FunctionValue<'ctx>,
        ssa_func: &SSAFunction,
    ) -> Result<()> {
        self.blocks.clear();

        for block in &ssa_func.blocks {
            let bb = self.context.append_basic_block(function, &format!("bb{}", block.id.0));
            self.blocks.insert(block.id, bb);
        }

        Ok(())
    }

    /// Generate code for a single instruction
    fn generate_instruction(&mut self, inst: &SSAInstruction) -> Result<()> {
        match inst {
            SSAInstruction::LoadInt { dest, value } => {
                let val = self.cell_type().const_int(*value as u64, true);
                self.values.insert(*dest, val.into());
            }

            SSAInstruction::LoadFloat { dest, value } => {
                let val = self.float_type().const_float(*value);
                self.values.insert(*dest, val.into());
            }

            SSAInstruction::LoadString { dest, value } => {
                // Create global string constant
                let str_val = self.builder.build_global_string_ptr(value, "str")
                    .map_err(|e| BackendError::CodeGenError(e.to_string()))?;
                self.values.insert(*dest, str_val.as_basic_value_enum());
            }

            SSAInstruction::BinaryOp { dest, op, left, right } => {
                self.generate_binary_op(*dest, *op, *left, *right)?;
            }

            SSAInstruction::UnaryOp { dest, op, operand } => {
                self.generate_unary_op(*dest, *op, *operand)?;
            }

            SSAInstruction::Load { dest, address, .. } => {
                let addr = self.get_value(*address)?;
                let addr_ptr = addr.into_pointer_value();
                let loaded = self.builder.build_load(self.cell_type(), addr_ptr, "load")
                    .map_err(|e| BackendError::CodeGenError(e.to_string()))?;
                self.values.insert(*dest, loaded);
            }

            SSAInstruction::Store { address, value, .. } => {
                let addr = self.get_value(*address)?;
                let val = self.get_value(*value)?;
                let addr_ptr = addr.into_pointer_value();
                self.builder.build_store(addr_ptr, val)
                    .map_err(|e| BackendError::CodeGenError(e.to_string()))?;
            }

            SSAInstruction::Call { dest, name, args } => {
                self.generate_call(dest, name, args)?;
            }

            SSAInstruction::Branch { condition, true_block, false_block } => {
                self.control_flow.generate_branch(
                    &self.builder,
                    &mut self.values,
                    &self.blocks,
                    *condition,
                    *true_block,
                    *false_block,
                )?;
            }

            SSAInstruction::Jump { target } => {
                self.control_flow.generate_jump(
                    &self.builder,
                    &self.blocks,
                    *target,
                )?;
            }

            SSAInstruction::Return { values: ret_vals } => {
                if let Some(reg) = ret_vals.first() {
                    let val = self.get_value(*reg)?;
                    self.builder.build_return(Some(&val))
                        .map_err(|e| BackendError::CodeGenError(e.to_string()))?;
                } else {
                    self.builder.build_return(None)
                        .map_err(|e| BackendError::CodeGenError(e.to_string()))?;
                }
            }

            SSAInstruction::Phi { dest, incoming } => {
                // PHI nodes are handled separately
                self.generate_phi(*dest, incoming)?;
            }
        }

        Ok(())
    }

    /// Get LLVM value for SSA register
    fn get_value(&self, reg: Register) -> Result<BasicValueEnum<'ctx>> {
        self.values
            .get(&reg)
            .copied()
            .ok_or_else(|| BackendError::InvalidIR(format!("Undefined register: {}", reg)))
    }

    /// Generate binary operation
    fn generate_binary_op(
        &mut self,
        dest: Register,
        op: BinaryOperator,
        left: Register,
        right: Register,
    ) -> Result<()> {
        let lhs = self.get_value(left)?;
        let rhs = self.get_value(right)?;

        let result = self.primitives.generate_binary_op(
            &self.builder,
            op,
            lhs,
            rhs,
        )?;

        self.values.insert(dest, result);
        Ok(())
    }

    /// Generate unary operation
    fn generate_unary_op(
        &mut self,
        dest: Register,
        op: UnaryOperator,
        operand: Register,
    ) -> Result<()> {
        let val = self.get_value(operand)?;
        let result = self.primitives.generate_unary_op(&self.builder, op, val)?;
        self.values.insert(dest, result);
        Ok(())
    }

    /// Generate function call
    fn generate_call(
        &mut self,
        dest: &[Register],
        name: &str,
        args: &[Register],
    ) -> Result<()> {
        // Get or declare function
        let callee = self.module
            .get_function(name)
            .ok_or_else(|| BackendError::InvalidIR(format!("Undefined function: {}", name)))?;

        // Collect arguments
        let arg_values: Result<Vec<_>> = args
            .iter()
            .map(|&reg| self.get_value(reg).map(|v| v.into()))
            .collect();
        let arg_values = arg_values?;

        // Use custom calling convention for the call
        let result = self.calling_convention.generate_call(
            &self.builder,
            callee,
            &arg_values,
        )?;

        // Store result if present
        if let Some(&dest_reg) = dest.first() {
            self.values.insert(dest_reg, result);
        }

        Ok(())
    }

    /// Create FFI bridge for calling C function from Forth
    pub fn create_c_ffi_bridge(
        &mut self,
        c_function_name: &str,
        arg_count: usize,
    ) -> Result<FunctionValue<'ctx>> {
        // Get the C function
        let c_function = self.module
            .get_function(c_function_name)
            .ok_or_else(|| BackendError::InvalidIR(format!("C function not found: {}", c_function_name)))?;

        // Create the bridge
        self.ffi_bridge.create_forth_to_c_bridge(
            c_function_name,
            c_function,
            arg_count,
        )
    }

    /// Create FFI bridge for calling Forth function from C
    pub fn create_forth_ffi_bridge(
        &mut self,
        forth_function_name: &str,
        arg_count: usize,
    ) -> Result<FunctionValue<'ctx>> {
        // Get the Forth function
        let forth_function = self.module
            .get_function(forth_function_name)
            .ok_or_else(|| BackendError::InvalidIR(format!("Forth function not found: {}", forth_function_name)))?;

        // Create the bridge
        self.ffi_bridge.create_c_to_forth_bridge(
            forth_function_name,
            forth_function,
            arg_count,
        )
    }

    /// Set calling convention (for external calls)
    pub fn set_calling_convention(&mut self, convention: ForthCallingConvention) {
        self.calling_convention = convention;
    }

    /// Generate PHI node
    fn generate_phi(&mut self, dest: Register, incoming: &[(BlockId, Register)]) -> Result<()> {
        let phi_type = self.cell_type();
        let phi = self.builder.build_phi(phi_type, "phi")
            .map_err(|e| BackendError::CodeGenError(e.to_string()))?;

        for (block_id, reg) in incoming {
            let block = self.blocks.get(block_id)
                .ok_or_else(|| BackendError::InvalidIR(format!("Undefined block: {}", block_id.0)))?;
            let value = self.get_value(*reg)?;
            phi.add_incoming(&[(&value, *block)]);
        }

        self.values.insert(dest, phi.as_basic_value());
        Ok(())
    }

    /// Initialize function parameters
    fn init_parameters(&mut self, function: FunctionValue<'ctx>, params: &[Register]) {
        for (i, param) in function.get_param_iter().enumerate() {
            if let Some(&reg) = params.get(i) {
                self.values.insert(reg, param);
            }
        }
    }

    /// Verify the module
    fn verify_module(&self) -> Result<()> {
        if let Err(err) = self.module.verify() {
            return Err(BackendError::VerificationFailed(err.to_string()));
        }
        Ok(())
    }

    /// Run optimization passes
    fn optimize(&self) {
        use inkwell::passes::{PassManager, PassManagerBuilder};

        let pass_manager_builder = PassManagerBuilder::create();
        pass_manager_builder.set_optimization_level(self.opt_level);

        let fpm = PassManager::create(&self.module);
        pass_manager_builder.populate_function_pass_manager(&fpm);

        // Add specific passes
        fpm.add_instruction_combining_pass();
        fpm.add_reassociate_pass();
        fpm.add_gvn_pass();
        fpm.add_cfg_simplification_pass();
        fpm.add_basic_alias_analysis_pass();
        fpm.add_promote_memory_to_register_pass();
        fpm.add_tail_call_elimination_pass();

        fpm.initialize();

        // Run on all functions
        for function in self.module.get_functions() {
            fpm.run_on(&function);
        }

        fpm.finalize();
    }

    /// Print LLVM IR to string
    pub fn print_to_string(&self) -> String {
        self.module.print_to_string().to_string()
    }

    /// Write object file
    pub fn write_object_file(&self, path: &Path) -> Result<()> {
        // Initialize target
        Target::initialize_native(&InitializationConfig::default())
            .map_err(|e| BackendError::TargetMachineError(e.to_string()))?;

        let triple = TargetMachine::get_default_triple();
        let target = Target::from_triple(&triple)
            .map_err(|e| BackendError::TargetMachineError(e.to_string()))?;

        let cpu = TargetMachine::get_host_cpu_name().to_string();
        let features = TargetMachine::get_host_cpu_features().to_string();

        let target_machine = target
            .create_target_machine(
                &triple,
                &cpu,
                &features,
                self.opt_level,
                RelocMode::PIC,
                CodeModel::Default,
            )
            .ok_or_else(|| BackendError::TargetMachineError("Failed to create target machine".to_string()))?;

        target_machine
            .write_to_file(&self.module, FileType::Object, path)
            .map_err(|e| BackendError::CodeGenError(e.to_string()))?;

        Ok(())
    }
}

impl<'ctx> CodeGenerator for LLVMBackend<'ctx> {
    fn generate(&mut self, ssa_func: &SSAFunction) -> Result<()> {
        // Create LLVM function
        let function = self.create_function(ssa_func)?;
        self.current_function = Some(function);

        // Create basic blocks
        self.create_basic_blocks(function, ssa_func)?;

        // Initialize parameters
        self.init_parameters(function, &ssa_func.parameters);

        // Generate code for each basic block
        for block in &ssa_func.blocks {
            let bb = self.blocks.get(&block.id)
                .ok_or_else(|| BackendError::InvalidIR(format!("Block not found: {}", block.id.0)))?;

            self.builder.position_at_end(*bb);

            for inst in &block.instructions {
                self.generate_instruction(inst)?;
            }
        }

        Ok(())
    }

    fn finalize(&self, path: &Path) -> Result<()> {
        // Verify module
        self.verify_module()?;

        // Run optimizations
        if self.opt_level != OptimizationLevel::None {
            self.optimize();
        }

        // Write output
        match self.mode {
            CompilationMode::AOT => {
                self.write_object_file(path)?;
            }
            CompilationMode::JIT => {
                // JIT mode doesn't write to file
                // Code will be executed in memory
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backend_creation() {
        let context = Context::create();
        let backend = LLVMBackend::new(
            &context,
            "test_module",
            CompilationMode::AOT,
            OptimizationLevel::Default,
        );

        assert_eq!(backend.mode, CompilationMode::AOT);
    }

    #[test]
    fn test_cell_type() {
        let context = Context::create();
        let backend = LLVMBackend::new(
            &context,
            "test",
            CompilationMode::AOT,
            OptimizationLevel::None,
        );

        let cell = backend.cell_type();
        assert_eq!(cell.get_bit_width(), 64);
    }
}
