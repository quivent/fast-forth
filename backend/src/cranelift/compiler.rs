//! Cranelift Compiler Implementation
//!
//! Fast compilation backend using Cranelift code generator.

use crate::error::{BackendError, Result};
use crate::cranelift::{CraneliftSettings, SSATranslator};
use fastforth_frontend::ssa::SSAFunction;

use cranelift_codegen::ir::{AbiParam, Function, Signature};
use cranelift_codegen::isa::CallConv;
use cranelift_codegen::settings::{self, Configurable, Flags};
use cranelift_codegen::Context;
use cranelift_frontend::FunctionBuilderContext;
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{DataDescription, FuncId, Linkage, Module};
use target_lexicon::Triple;

use std::collections::HashMap;

/// Cranelift backend for Fast Forth
pub struct CraneliftBackend {
    module: JITModule,
    ctx: Context,
    builder_ctx: FunctionBuilderContext,
    settings: CraneliftSettings,
    functions: HashMap<String, FuncId>,
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
            _ => {
                return Err(BackendError::Initialization(
                    "Cranelift only supports opt_level 0 or 1. Use LLVM for -O2/-O3.".to_string()
                ));
            }
        }

        let flags = Flags::new(flag_builder);

        // Create ISA
        let isa = cranelift_codegen::isa::lookup(triple)
            .map_err(|e| BackendError::Initialization(format!("ISA lookup failed: {}", e)))?
            .finish(flags)
            .map_err(|e| BackendError::Initialization(format!("ISA creation failed: {}", e)))?;

        // Create JIT module
        let builder = JITBuilder::with_isa(isa, cranelift_module::default_libcall_names());
        let module = JITModule::new(builder);

        Ok(Self {
            module,
            ctx: Context::new(),
            builder_ctx: FunctionBuilderContext::new(),
            settings,
            functions: HashMap::new(),
        })
    }

    /// Compile an SSA function to native code
    pub fn compile_function(&mut self, ssa_func: &SSAFunction, name: &str) -> Result<*const u8> {
        // Create function signature
        let sig = self.create_signature();

        // Declare function in module
        let func_id = self.module
            .declare_function(name, Linkage::Export, &sig)
            .map_err(|e| BackendError::CodeGeneration(format!("Failed to declare function: {}", e)))?;

        // Translate SSA to Cranelift IR
        self.ctx.func.signature = sig;
        let translator = SSATranslator::new(&mut self.ctx.func, &mut self.builder_ctx);
        translator.translate(ssa_func)?;

        // Compile function
        self.module
            .define_function(func_id, &mut self.ctx)
            .map_err(|e| BackendError::CodeGeneration(format!("Failed to define function: {}", e)))?;

        // Clear context for next function
        self.module.clear_context(&mut self.ctx);

        // Finalize and get function pointer
        self.module.finalize_definitions()
            .map_err(|e| BackendError::CodeGeneration(format!("Failed to finalize: {}", e)))?;

        let code_ptr = self.module.get_finalized_function(func_id);

        // Store function ID
        self.functions.insert(name.to_string(), func_id);

        Ok(code_ptr)
    }

    /// Create standard Forth function signature (stack-based)
    fn create_signature(&self) -> Signature {
        let mut sig = Signature::new(CallConv::SystemV);
        // Fast Forth functions take stack pointer as argument
        sig.params.push(AbiParam::new(
            self.module.target_config().pointer_type()
        ));
        // Return updated stack pointer
        sig.returns.push(AbiParam::new(
            self.module.target_config().pointer_type()
        ));
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

    /// Compile SSA function to native code
    pub fn compile(&mut self, ssa_func: &SSAFunction, name: &str) -> Result<*const u8> {
        self.backend.compile_function(ssa_func, name)
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
