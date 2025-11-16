//! Cranelift Compiler Implementation
//!
//! Fast compilation backend using Cranelift code generator.

use crate::error::{BackendError, Result};
use crate::cranelift::{CraneliftSettings, SSATranslator, FFIRegistry};
use fastforth_frontend::ssa::SSAFunction;

use cranelift_codegen::ir::types;

use cranelift_codegen::ir::{AbiParam, Function, FuncRef, Signature};
use cranelift_codegen::isa::CallConv;
use cranelift_codegen::settings::{self, Configurable, Flags};
use cranelift_codegen::Context;
use cranelift_codegen::isa::TargetIsa;
use cranelift_frontend::FunctionBuilderContext;
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{DataDescription, FuncId, Linkage, Module};
use target_lexicon::Triple;

use std::collections::HashMap;
use std::sync::Arc;

/// Cranelift backend for Fast Forth
pub struct CraneliftBackend {
    module: JITModule,
    ctx: Context,
    builder_ctx: FunctionBuilderContext,
    settings: CraneliftSettings,
    functions: HashMap<String, FuncId>,
    /// Cached function references for calls (populated during compilation)
    func_refs: HashMap<String, FuncRef>,
    /// FFI registry for external C function calls
    ffi_registry: FFIRegistry,
    /// Target ISA for verification
    isa: Arc<dyn TargetIsa>,
}

impl CraneliftBackend {
    /// Create a new Cranelift backend with given settings
    pub fn new(settings: CraneliftSettings) -> Result<Self> {
        // Get target triple (host or specified)
        let triple = if let Some(triple_str) = settings.target_triple {
            triple_str.parse().map_err(|e| {
                BackendError::Initialization(format!("Invalid target triple: {}", e))
            })?
        } else {
            Triple::host()
        };

        // Create Cranelift settings
        let mut flag_builder = settings::builder();

        // Set optimization level
        match settings.opt_level {
            0 => {
                flag_builder.set("opt_level", "none")
                    .map_err(|e| BackendError::Initialization(format!("Failed to set opt_level: {}", e)))?;
            }
            1 => {
                flag_builder.set("opt_level", "speed")
                    .map_err(|e| BackendError::Initialization(format!("Failed to set opt_level: {}", e)))?;
            }
            2 => {
                flag_builder.set("opt_level", "speed_and_size")
                    .map_err(|e| BackendError::Initialization(format!("Failed to set opt_level: {}", e)))?;
            }
            _ => {
                return Err(BackendError::Initialization(
                    "Cranelift supports opt_level 0-2. Use LLVM for -O3.".to_string()
                ));
            }
        }

        let flags = Flags::new(flag_builder);

        // Create ISA (returns Arc<dyn TargetIsa>)
        let isa = cranelift_codegen::isa::lookup(triple)
            .map_err(|e| BackendError::Initialization(format!("ISA lookup failed: {}", e)))?
            .finish(flags)
            .map_err(|e| BackendError::Initialization(format!("ISA creation failed: {}", e)))?;

        // Create JIT module (JITBuilder::with_isa takes Arc<dyn TargetIsa>)
        let builder = JITBuilder::with_isa(isa.clone(), cranelift_module::default_libcall_names());
        let mut module = JITModule::new(builder);

        // Initialize FFI registry and register libc functions
        let mut ffi_registry = FFIRegistry::new();
        ffi_registry.register_libc_functions(&mut module)?;

        Ok(Self {
            module,
            ctx: Context::new(),
            builder_ctx: FunctionBuilderContext::new(),
            settings,
            functions: HashMap::new(),
            func_refs: HashMap::new(),
            ffi_registry,
            isa,
        })
    }

    /// Declare all functions upfront (for recursion/inter-function calls)
    pub fn declare_all_functions(&mut self, functions: &[(String, &SSAFunction)]) -> Result<()> {
        for (name, ssa_func) in functions {
            // Create signature based on SSA function's parameters and return count
            let param_count = ssa_func.parameters.len();
            let return_count = 1; // All Forth functions return 1 value (top of stack)
            let sig = self.create_signature(param_count, return_count);

            let func_id = self.module
                .declare_function(name, Linkage::Export, &sig)
                .map_err(|e| BackendError::CodeGeneration(format!("Failed to declare function '{}': {}", name, e)))?;
            self.functions.insert(name.clone(), func_id);
        }
        Ok(())
    }

    /// Compile an SSA function to native code (function must already be declared)
    /// Note: Call finalize_all() after compiling all functions
    pub fn compile_function(&mut self, ssa_func: &SSAFunction, name: &str) -> Result<()> {
        // Get the function ID (must have been declared first)
        let func_id = self.functions.get(name)
            .copied()
            .ok_or_else(|| BackendError::CodeGeneration(format!("Function '{}' not declared", name)))?;

        // Create function signature based on SSA function's parameters and return count
        let param_count = ssa_func.parameters.len();
        let return_count = 1; // All Forth functions return 1 value
        let sig = self.create_signature(param_count, return_count);
        self.ctx.func.signature = sig;

        // Import all declared functions into this function's context (for calls)
        // This must be done BEFORE translation begins
        self.func_refs.clear();
        for (func_name, &fid) in &self.functions {
            let func_ref = self.module.declare_func_in_func(fid, &mut self.ctx.func);
            self.func_refs.insert(func_name.clone(), func_ref);
        }

        // Import FFI functions as well
        let mut ffi_refs = HashMap::new();
        for ffi_name in self.ffi_registry.function_names() {
            if let Some(ffi_id) = self.ffi_registry.get_function(ffi_name) {
                let ffi_ref = self.module.declare_func_in_func(ffi_id, &mut self.ctx.func);
                ffi_refs.insert(ffi_name.to_string(), ffi_ref);
            }
        }

        // Clone func_refs to avoid borrow checker issues
        let func_refs_copy = self.func_refs.clone();

        // Translate SSA to Cranelift IR
        let translator = SSATranslator::new(
            &mut self.ctx.func,
            &mut self.builder_ctx,
            &func_refs_copy,
            &ffi_refs,
            &self.isa,
            self.settings.enable_verification,
        );
        translator.translate(ssa_func)?;

        // Define function (but don't finalize yet - allows recursion)
        self.module
            .define_function(func_id, &mut self.ctx)
            .map_err(|e| BackendError::CodeGeneration(format!("Failed to define function '{}': {}", name, e)))?;

        // Clear context for next function
        self.module.clear_context(&mut self.ctx);

        Ok(())
    }

    /// Finalize all compiled functions (call after compiling all functions)
    pub fn finalize_all(&mut self) -> Result<()> {
        self.module.finalize_definitions()
            .map_err(|e| BackendError::CodeGeneration(format!("Failed to finalize: {}", e)))?;
        Ok(())
    }

    /// Create standard Forth function signature (register-based SSA calling)
    /// Functions take their SSA parameters directly and return SSA results
    fn create_signature(&self, param_count: usize, return_count: usize) -> Signature {
        let mut sig = Signature::new(CallConv::SystemV);

        // Add parameters (all i64 for now)
        for _ in 0..param_count {
            sig.params.push(AbiParam::new(types::I64));
        }

        // Add returns (all i64 for now)
        for _ in 0..return_count {
            sig.returns.push(AbiParam::new(types::I64));
        }

        sig
    }

    /// Get pointer to compiled function by name
    pub fn get_function(&self, name: &str) -> Option<*const u8> {
        self.functions.get(name).map(|&func_id| {
            self.module.get_finalized_function(func_id)
        })
    }
}

/// High-level compiler interface
pub struct CraneliftCompiler {
    backend: CraneliftBackend,
}

impl CraneliftCompiler {
    /// Create new compiler with default settings
    pub fn new() -> Result<Self> {
        Self::with_settings(CraneliftSettings::default())
    }

    /// Create compiler with custom settings
    pub fn with_settings(settings: CraneliftSettings) -> Result<Self> {
        Ok(Self {
            backend: CraneliftBackend::new(settings)?,
        })
    }

    /// Get mutable reference to backend for two-pass compilation
    /// (declare_all_functions, compile_function for each, then finalize_all)
    pub fn backend_mut(&mut self) -> &mut CraneliftBackend {
        &mut self.backend
    }

    /// Get compiled function by name
    pub fn get_function(&self, name: &str) -> Option<*const u8> {
        self.backend.get_function(name)
    }
}

impl Default for CraneliftCompiler {
    fn default() -> Self {
        Self::new().expect("Failed to create Cranelift compiler")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_compiler() {
        let compiler = CraneliftCompiler::new();
        assert!(compiler.is_ok());
    }

    #[test]
    fn test_development_settings() {
        let settings = CraneliftSettings::development();
        let compiler = CraneliftCompiler::with_settings(settings);
        assert!(compiler.is_ok());
    }
}
