//! FFI and C Runtime Coverage Tests
//!
//! Targets uncovered code paths in FFI integration, type marshaling,
//! error propagation, and memory safety at the FFI boundary.

use backend::cranelift::{FFIRegistry, FFISignature};
use backend::error::{BackendError, Result};
use cranelift_codegen::ir::types;
use cranelift_jit::{JITBuilder, JITModule};

#[test]
fn test_ffi_signature_creation() {
    let sig = FFISignature::new("test_func");
    assert_eq!(sig.name, "test_func");
    assert_eq!(sig.params.len(), 0);
    assert_eq!(sig.returns.len(), 0);
}

#[test]
fn test_ffi_signature_with_params() {
    let sig = FFISignature::new("add")
        .param(types::I64)
        .param(types::I64)
        .returns(types::I64);

    assert_eq!(sig.params.len(), 2);
    assert_eq!(sig.returns.len(), 1);
    assert_eq!(sig.params[0], types::I64);
}

#[test]
fn test_ffi_signature_multiple_returns() {
    let sig = FFISignature::new("multi_return")
        .param(types::I64)
        .returns(types::I64)
        .returns(types::I32);

    assert_eq!(sig.returns.len(), 2);
    assert_eq!(sig.returns[0], types::I64);
    assert_eq!(sig.returns[1], types::I32);
}

#[test]
fn test_ffi_signature_to_cranelift() {
    let sig = FFISignature::new("test")
        .param(types::I64)
        .returns(types::I32);

    let cranelift_sig = sig.to_cranelift_signature();
    assert_eq!(cranelift_sig.params.len(), 1);
    assert_eq!(cranelift_sig.returns.len(), 1);
}

#[test]
fn test_ffi_registry_creation() {
    let registry = FFIRegistry::new();
    // Registry should be empty initially
    assert!(registry.get_function("nonexistent").is_none());
}

#[test]
fn test_ffi_registry_register_libc() {
    let mut builder = JITBuilder::new(cranelift_module::default_libcall_names())
        .expect("Failed to create JIT builder");
    let mut module = JITModule::new(builder);
    let mut registry = FFIRegistry::new();

    // Should succeed in registering libc functions
    assert!(registry.register_libc_functions(&mut module).is_ok());
}

#[test]
fn test_ffi_lookup_fopen() {
    let mut builder = JITBuilder::new(cranelift_module::default_libcall_names())
        .expect("Failed to create JIT builder");
    let mut module = JITModule::new(builder);
    let mut registry = FFIRegistry::new();

    registry.register_libc_functions(&mut module).unwrap();

    // fopen should be registered
    assert!(registry.get_function("fopen").is_some());
}

#[test]
fn test_ffi_lookup_malloc() {
    let mut builder = JITBuilder::new(cranelift_module::default_libcall_names())
        .expect("Failed to create JIT builder");
    let mut module = JITModule::new(builder);
    let mut registry = FFIRegistry::new();

    registry.register_libc_functions(&mut module).unwrap();

    // malloc should be registered
    assert!(registry.get_function("malloc").is_some());
}

#[test]
fn test_ffi_signature_validation_empty_name() {
    let sig = FFISignature::new("");
    assert_eq!(sig.name, "");
}

#[test]
fn test_ffi_signature_float_types() {
    let sig = FFISignature::new("float_func")
        .param(types::F64)
        .param(types::F32)
        .returns(types::F64);

    assert_eq!(sig.params.len(), 2);
    assert_eq!(sig.params[0], types::F64);
    assert_eq!(sig.params[1], types::F32);
}

#[test]
fn test_ffi_signature_mixed_types() {
    let sig = FFISignature::new("mixed")
        .param(types::I64)
        .param(types::F64)
        .param(types::I32)
        .returns(types::I8);

    assert_eq!(sig.params.len(), 3);
    assert_eq!(sig.returns[0], types::I8);
}

#[test]
fn test_ffi_registry_duplicate_registration() {
    let mut builder = JITBuilder::new(cranelift_module::default_libcall_names())
        .expect("Failed to create JIT builder");
    let mut module = JITModule::new(builder);
    let mut registry = FFIRegistry::new();

    // First registration should succeed
    assert!(registry.register_libc_functions(&mut module).is_ok());

    // Second registration should handle duplicates gracefully
    assert!(registry.register_libc_functions(&mut module).is_ok());
}

#[test]
fn test_ffi_custom_function_registration() {
    let mut builder = JITBuilder::new(cranelift_module::default_libcall_names())
        .expect("Failed to create JIT builder");
    let mut module = JITModule::new(builder);
    let mut registry = FFIRegistry::new();

    let custom_sig = FFISignature::new("custom_func")
        .param(types::I64)
        .returns(types::I64);

    // Should be able to register custom function
    assert!(registry.register_function(&mut module, custom_sig).is_ok());
}

#[test]
fn test_ffi_type_conversion_i8() {
    let sig = FFISignature::new("i8_func")
        .param(types::I8)
        .returns(types::I8);

    let cranelift_sig = sig.to_cranelift_signature();
    assert_eq!(cranelift_sig.params[0].value_type, types::I8);
}

#[test]
fn test_ffi_type_conversion_pointer() {
    // Pointers are represented as I64 on 64-bit systems
    let sig = FFISignature::new("ptr_func")
        .param(types::I64) // void*
        .returns(types::I64); // void*

    let cranelift_sig = sig.to_cranelift_signature();
    assert_eq!(cranelift_sig.params[0].value_type, types::I64);
    assert_eq!(cranelift_sig.returns[0].value_type, types::I64);
}
