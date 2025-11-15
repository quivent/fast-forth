//! FFI (Foreign Function Interface) Infrastructure
//!
//! Provides mechanism for calling external C functions from FastForth code.
//! Supports standard libc functions for file I/O and system operations.

use crate::error::{BackendError, Result};
use cranelift_codegen::ir::{types, AbiParam, ExternalName, Signature};
use cranelift_codegen::isa::CallConv;
use cranelift_module::{FuncId, Linkage, Module};
use std::collections::HashMap;

/// FFI function metadata
#[derive(Debug, Clone)]
pub struct FFISignature {
    pub name: String,
    pub params: Vec<types::Type>,
    pub returns: Vec<types::Type>,
}

impl FFISignature {
    /// Create a new FFI signature
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            params: Vec::new(),
            returns: Vec::new(),
        }
    }

    /// Add a parameter type
    pub fn param(mut self, ty: types::Type) -> Self {
        self.params.push(ty);
        self
    }

    /// Add a return type
    pub fn returns(mut self, ty: types::Type) -> Self {
        self.returns.push(ty);
        self
    }

    /// Convert to Cranelift signature
    pub fn to_cranelift_signature(&self) -> Signature {
        let mut sig = Signature::new(CallConv::SystemV);

        for param_ty in &self.params {
            sig.params.push(AbiParam::new(*param_ty));
        }

        for return_ty in &self.returns {
            sig.returns.push(AbiParam::new(*return_ty));
        }

        sig
    }
}

/// Registry of external C functions
pub struct FFIRegistry {
    /// Map of function names to their Cranelift function IDs
    functions: HashMap<String, FuncId>,
    /// Function signatures for reference
    signatures: HashMap<String, FFISignature>,
}

impl FFIRegistry {
    /// Create a new empty FFI registry
    pub fn new() -> Self {
        Self {
            functions: HashMap::new(),
            signatures: HashMap::new(),
        }
    }

    /// Register standard libc functions required for file I/O and system operations
    pub fn register_libc_functions<M: Module>(&mut self, module: &mut M) -> Result<()> {
        // FILE* fopen(const char* path, const char* mode)
        self.register_function(
            module,
            FFISignature::new("fopen")
                .param(types::I64) // const char* path
                .param(types::I64) // const char* mode
                .returns(types::I64), // FILE* pointer
        )?;

        // size_t fread(void* ptr, size_t size, size_t count, FILE* stream)
        self.register_function(
            module,
            FFISignature::new("fread")
                .param(types::I64) // void* ptr
                .param(types::I64) // size_t size
                .param(types::I64) // size_t count
                .param(types::I64) // FILE* stream
                .returns(types::I64), // size_t (bytes read)
        )?;

        // size_t fwrite(const void* ptr, size_t size, size_t count, FILE* stream)
        self.register_function(
            module,
            FFISignature::new("fwrite")
                .param(types::I64) // const void* ptr
                .param(types::I64) // size_t size
                .param(types::I64) // size_t count
                .param(types::I64) // FILE* stream
                .returns(types::I64), // size_t (bytes written)
        )?;

        // int fclose(FILE* stream)
        self.register_function(
            module,
            FFISignature::new("fclose")
                .param(types::I64) // FILE* pointer
                .returns(types::I32), // int (0 = success)
        )?;

        // int remove(const char* path)
        self.register_function(
            module,
            FFISignature::new("remove")
                .param(types::I64) // const char* path
                .returns(types::I32), // int (0 = success)
        )?;

        // int system(const char* command)
        self.register_function(
            module,
            FFISignature::new("system")
                .param(types::I64) // const char* command
                .returns(types::I32), // int (exit code)
        )?;

        // void* malloc(size_t size)
        self.register_function(
            module,
            FFISignature::new("malloc")
                .param(types::I64) // size_t size
                .returns(types::I64), // void* pointer
        )?;

        // void free(void* ptr)
        self.register_function(
            module,
            FFISignature::new("free")
                .param(types::I64) // void* ptr
                .returns(types::I64), // void (returns dummy i64)
        )?;

        // void* memcpy(void* dest, const void* src, size_t n)
        self.register_function(
            module,
            FFISignature::new("memcpy")
                .param(types::I64) // void* dest
                .param(types::I64) // const void* src
                .param(types::I64) // size_t n
                .returns(types::I64), // void* (dest pointer)
        )?;

        // int printf(const char* format, ...)
        // Note: Variadic functions need special handling - this is simplified
        self.register_function(
            module,
            FFISignature::new("printf")
                .param(types::I64) // const char* format
                .returns(types::I32), // int (chars printed)
        )?;

        Ok(())
    }

    /// Register a single external function
    fn register_function<M: Module>(
        &mut self,
        module: &mut M,
        sig: FFISignature,
    ) -> Result<()> {
        let cranelift_sig = sig.to_cranelift_signature();

        let func_id = module
            .declare_function(&sig.name, Linkage::Import, &cranelift_sig)
            .map_err(|e| {
                BackendError::CodeGeneration(format!(
                    "Failed to declare FFI function '{}': {}",
                    sig.name, e
                ))
            })?;

        self.functions.insert(sig.name.clone(), func_id);
        self.signatures.insert(sig.name.clone(), sig);

        Ok(())
    }

    /// Get function ID for a registered C function
    pub fn get_function(&self, name: &str) -> Option<FuncId> {
        self.functions.get(name).copied()
    }

    /// Get signature for a registered function
    pub fn get_signature(&self, name: &str) -> Option<&FFISignature> {
        self.signatures.get(name)
    }

    /// Check if a function is registered
    pub fn has_function(&self, name: &str) -> bool {
        self.functions.contains_key(name)
    }

    /// Get all registered function names
    pub fn function_names(&self) -> Vec<&str> {
        self.functions.keys().map(|s| s.as_str()).collect()
    }
}

impl Default for FFIRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ffi_signature_builder() {
        let sig = FFISignature::new("test_func")
            .param(types::I64)
            .param(types::I32)
            .returns(types::I64);

        assert_eq!(sig.name, "test_func");
        assert_eq!(sig.params.len(), 2);
        assert_eq!(sig.returns.len(), 1);
    }

    #[test]
    fn test_ffi_signature_to_cranelift() {
        let sig = FFISignature::new("fopen")
            .param(types::I64)
            .param(types::I64)
            .returns(types::I64);

        let cranelift_sig = sig.to_cranelift_signature();
        assert_eq!(cranelift_sig.params.len(), 2);
        assert_eq!(cranelift_sig.returns.len(), 1);
    }

    #[test]
    fn test_ffi_registry_creation() {
        let registry = FFIRegistry::new();
        assert_eq!(registry.functions.len(), 0);
        assert_eq!(registry.signatures.len(), 0);
    }

    #[test]
    fn test_ffi_registry_has_function() {
        let mut registry = FFIRegistry::new();
        assert!(!registry.has_function("fopen"));

        // Note: Can't easily test register_function without a real Module
        // This would require integration tests
    }
}
