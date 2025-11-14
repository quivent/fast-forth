//! Calling Convention Tests
//!
//! This module tests the custom Forth calling convention implementation,
//! including Forth-to-Forth calls and FFI bridges.

use backend::codegen::{
    CallingConvention, CallingConventionType, ForthCallingConvention,
    FFIBridge, ForthRegister, RegisterAllocator, LLVMBackend, CompilationMode,
};
use fastforth_frontend::ssa::{SSAFunction, SSAInstruction, Register, BlockId, BasicBlock};
use inkwell::context::Context;
use inkwell::OptimizationLevel;

#[test]
fn test_forth_internal_calling_convention() {
    let context = Context::create();
    let module = context.create_module("test");
    let builder = context.create_builder();

    let convention = ForthCallingConvention::internal();
    assert_eq!(convention.convention_type(), CallingConventionType::ForthInternal);

    // Create a simple function
    let i64_type = context.i64_type();
    let fn_type = i64_type.fn_type(&[i64_type.into()], false);
    let function = module.add_function("test_func", fn_type, None);

    let entry = context.append_basic_block(function, "entry");
    builder.position_at_end(entry);

    // Test prologue (should be no-op for internal calls)
    let result = convention.generate_prologue(&builder, function);
    assert!(result.is_ok());

    // Test epilogue (should be no-op for internal calls)
    let result = convention.generate_epilogue(&builder, function);
    assert!(result.is_ok());
}

#[test]
fn test_forth_to_c_calling_convention() {
    let convention = ForthCallingConvention::forth_to_c();
    assert_eq!(convention.convention_type(), CallingConventionType::ForthToC);
}

#[test]
fn test_c_to_forth_calling_convention() {
    let convention = ForthCallingConvention::c_to_forth();
    assert_eq!(convention.convention_type(), CallingConventionType::CToForth);
}

#[test]
fn test_forth_register_allocation() {
    // Test that dedicated registers have correct names
    assert_eq!(ForthRegister::DSP.llvm_name(), "r15");
    assert_eq!(ForthRegister::TOS.llvm_name(), "r12");
    assert_eq!(ForthRegister::NOS.llvm_name(), "r13");
    assert_eq!(ForthRegister::ThirdOS.llvm_name(), "r14");
    assert_eq!(ForthRegister::RSP.llvm_name(), "r11");

    // Test scratch registers
    for i in 0..9 {
        let reg = ForthRegister::Scratch(i);
        assert!(!reg.llvm_name().is_empty());
        assert!(!reg.constraint().is_empty());
    }
}

#[test]
fn test_register_allocator_basic() {
    let mut allocator = RegisterAllocator::new();

    // Allocate a register
    let reg1 = allocator.allocate("temp1".to_string());
    assert!(reg1.is_ok());
    assert!(matches!(reg1.unwrap(), ForthRegister::Scratch(_)));

    // Allocate another
    let reg2 = allocator.allocate("temp2".to_string());
    assert!(reg2.is_ok());

    // Check they're different
    assert_ne!(reg1.unwrap(), reg2.unwrap());

    // Free one
    allocator.free("temp1");

    // Should be able to allocate again
    let reg3 = allocator.allocate("temp3".to_string());
    assert!(reg3.is_ok());
}

#[test]
fn test_register_allocator_exhaustion() {
    let mut allocator = RegisterAllocator::new();

    // Allocate all 9 scratch registers
    let mut allocated = Vec::new();
    for i in 0..9 {
        let reg = allocator.allocate(format!("temp{}", i));
        assert!(reg.is_ok(), "Failed to allocate register {}", i);
        allocated.push(reg.unwrap());
    }

    // Next allocation should fail
    let overflow = allocator.allocate("overflow".to_string());
    assert!(overflow.is_err());

    // Free one and try again
    allocator.free("temp0");
    let reg = allocator.allocate("new_temp".to_string());
    assert!(reg.is_ok());
}

#[test]
fn test_register_allocator_reset() {
    let mut allocator = RegisterAllocator::new();

    // Allocate several registers
    for i in 0..5 {
        allocator.allocate(format!("temp{}", i)).unwrap();
    }

    // Reset
    allocator.reset();

    // Should be able to allocate all 9 again
    for i in 0..9 {
        let reg = allocator.allocate(format!("new_temp{}", i));
        assert!(reg.is_ok());
    }
}

#[test]
fn test_ffi_bridge_creation() {
    let context = Context::create();
    let module = context.create_module("ffi_test");

    let mut bridge = FFIBridge::new(&context, &module);

    // Create a mock C function
    let i64_type = context.i64_type();
    let c_fn_type = i64_type.fn_type(&[i64_type.into(), i64_type.into()], false);
    let c_function = module.add_function("printf", c_fn_type, None);

    // Create Forth-to-C bridge
    let result = bridge.create_forth_to_c_bridge("printf", c_function, 2);
    assert!(result.is_ok());

    // Verify bridge was cached
    let cached = bridge.get_forth_to_c_bridge("printf");
    assert!(cached.is_some());
}

#[test]
fn test_ffi_bridge_caching() {
    let context = Context::create();
    let module = context.create_module("ffi_cache_test");

    let mut bridge = FFIBridge::new(&context, &module);

    // Create a C function
    let i64_type = context.i64_type();
    let c_fn_type = i64_type.fn_type(&[i64_type.into()], false);
    let c_function = module.add_function("test_c_func", c_fn_type, None);

    // Create bridge twice
    let bridge1 = bridge.create_forth_to_c_bridge("test_c_func", c_function, 1);
    let bridge2 = bridge.create_forth_to_c_bridge("test_c_func", c_function, 1);

    assert!(bridge1.is_ok());
    assert!(bridge2.is_ok());

    // Should be the same function
    assert_eq!(
        bridge1.unwrap().get_name().to_str().unwrap(),
        bridge2.unwrap().get_name().to_str().unwrap()
    );
}

#[test]
fn test_llvm_backend_with_calling_convention() {
    let context = Context::create();
    let mut backend = LLVMBackend::new(
        &context,
        "calling_convention_test",
        CompilationMode::AOT,
        OptimizationLevel::None,
    );

    // Verify initial calling convention is internal
    assert_eq!(
        backend.calling_convention.convention_type(),
        CallingConventionType::ForthInternal
    );

    // Change to Forth-to-C
    backend.set_calling_convention(ForthCallingConvention::forth_to_c());
    assert_eq!(
        backend.calling_convention.convention_type(),
        CallingConventionType::ForthToC
    );
}

#[test]
fn test_forth_to_forth_call_generation() {
    let context = Context::create();
    let mut backend = LLVMBackend::new(
        &context,
        "forth_call_test",
        CompilationMode::AOT,
        OptimizationLevel::None,
    );

    // Create two simple Forth functions
    let i64_type = context.i64_type();
    let fn_type = i64_type.fn_type(&[i64_type.into()], false);

    // Callee function
    let callee = backend.module.add_function("callee", fn_type, None);
    let callee_entry = context.append_basic_block(callee, "entry");
    backend.builder.position_at_end(callee_entry);
    let param = callee.get_nth_param(0).unwrap();
    backend.builder.build_return(Some(&param)).unwrap();

    // Caller function
    let caller = backend.module.add_function("caller", fn_type, None);
    let caller_entry = context.append_basic_block(caller, "entry");
    backend.builder.position_at_end(caller_entry);

    // Generate call using custom calling convention
    let arg = caller.get_nth_param(0).unwrap();
    let result = backend.calling_convention.generate_call(
        &backend.builder,
        callee,
        &[arg.into()],
    );

    assert!(result.is_ok());
}

#[test]
fn test_calling_convention_with_ssa_function() {
    let context = Context::create();
    let mut backend = LLVMBackend::new(
        &context,
        "ssa_convention_test",
        CompilationMode::AOT,
        OptimizationLevel::None,
    );

    // Create a simple SSA function
    let mut ssa_func = SSAFunction {
        name: "test_word".to_string(),
        parameters: vec![Register(0)],
        return_values: vec![Register(1)],
        blocks: vec![
            BasicBlock {
                id: BlockId(0),
                instructions: vec![
                    SSAInstruction::LoadInt {
                        dest: Register(1),
                        value: 42,
                    },
                    SSAInstruction::Return {
                        values: vec![Register(1)],
                    },
                ],
            },
        ],
    };

    // Generate code
    let result = backend.generate(&ssa_func);
    assert!(result.is_ok());

    // Verify the function was created
    let func = backend.module.get_function("test_word");
    assert!(func.is_some());
}

#[test]
fn test_multiple_forth_calls_no_overhead() {
    let context = Context::create();
    let mut backend = LLVMBackend::new(
        &context,
        "multi_call_test",
        CompilationMode::AOT,
        OptimizationLevel::None,
    );

    let i64_type = context.i64_type();
    let fn_type = i64_type.fn_type(&[i64_type.into()], false);

    // Create multiple functions
    let func_a = backend.module.add_function("func_a", fn_type, None);
    let func_b = backend.module.add_function("func_b", fn_type, None);
    let func_c = backend.module.add_function("func_c", fn_type, None);

    // Create a caller that calls all three
    let caller = backend.module.add_function("caller_multi", fn_type, None);
    let entry = context.append_basic_block(caller, "entry");
    backend.builder.position_at_end(entry);

    let arg = caller.get_nth_param(0).unwrap();

    // Call func_a
    let result_a = backend.calling_convention.generate_call(
        &backend.builder,
        func_a,
        &[arg.into()],
    );
    assert!(result_a.is_ok());

    // Call func_b with result from func_a
    let result_b = backend.calling_convention.generate_call(
        &backend.builder,
        func_b,
        &[result_a.unwrap()],
    );
    assert!(result_b.is_ok());

    // Call func_c with result from func_b
    let result_c = backend.calling_convention.generate_call(
        &backend.builder,
        func_c,
        &[result_b.unwrap()],
    );
    assert!(result_c.is_ok());

    // All calls should succeed with internal convention (zero overhead)
}

#[test]
fn test_register_constraints() {
    // Verify all register constraints are properly formatted
    let registers = vec![
        ForthRegister::DSP,
        ForthRegister::TOS,
        ForthRegister::NOS,
        ForthRegister::ThirdOS,
        ForthRegister::RSP,
    ];

    for reg in registers {
        let constraint = reg.constraint();
        assert!(constraint.starts_with('{'));
        assert!(constraint.ends_with('}'));
        assert!(constraint.len() > 2);
    }

    // Test scratch register constraints
    for i in 0..9 {
        let reg = ForthRegister::Scratch(i);
        let constraint = reg.constraint();
        assert!(constraint.starts_with('{'));
        assert!(constraint.ends_with('}'));
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    /// Test that demonstrates zero-overhead Forth-to-Forth calls
    #[test]
    fn test_zero_overhead_forth_chain() {
        let context = Context::create();
        let mut backend = LLVMBackend::new(
            &context,
            "zero_overhead_test",
            CompilationMode::AOT,
            OptimizationLevel::None,
        );

        // Create a chain of Forth functions: A -> B -> C
        let i64_type = context.i64_type();
        let fn_type = i64_type.fn_type(&[], false);

        // Function C (leaf)
        let func_c = backend.module.add_function("leaf", fn_type, None);
        let c_entry = context.append_basic_block(func_c, "entry");
        backend.builder.position_at_end(c_entry);
        let const_val = i64_type.const_int(42, false);
        backend.builder.build_return(Some(&const_val)).unwrap();

        // Function B (calls C)
        let func_b = backend.module.add_function("middle", fn_type, None);
        let b_entry = context.append_basic_block(func_b, "entry");
        backend.builder.position_at_end(b_entry);

        let b_result = backend.calling_convention.generate_call(
            &backend.builder,
            func_c,
            &[],
        ).unwrap();
        backend.builder.build_return(Some(&b_result)).unwrap();

        // Function A (calls B)
        let func_a = backend.module.add_function("root", fn_type, None);
        let a_entry = context.append_basic_block(func_a, "entry");
        backend.builder.position_at_end(a_entry);

        let a_result = backend.calling_convention.generate_call(
            &backend.builder,
            func_b,
            &[],
        ).unwrap();
        backend.builder.build_return(Some(&a_result)).unwrap();

        // Verify all functions were created
        assert!(backend.module.get_function("leaf").is_some());
        assert!(backend.module.get_function("middle").is_some());
        assert!(backend.module.get_function("root").is_some());

        // Verify module is valid
        assert!(backend.module.verify().is_ok());
    }

    /// Test FFI bridge for calling C printf
    #[test]
    fn test_printf_ffi_bridge() {
        let context = Context::create();
        let mut backend = LLVMBackend::new(
            &context,
            "printf_ffi_test",
            CompilationMode::AOT,
            OptimizationLevel::None,
        );

        // Declare printf (simplified: int printf(const char *format, ...))
        let i32_type = context.i32_type();
        let ptr_type = context.ptr_type(inkwell::AddressSpace::default());
        let printf_type = i32_type.fn_type(&[ptr_type.into()], true);
        let printf = backend.module.add_function("printf", printf_type, None);

        // Create FFI bridge
        let result = backend.create_c_ffi_bridge("printf", 1);
        assert!(result.is_ok());

        let bridge = result.unwrap();
        assert!(bridge.get_name().to_str().unwrap().contains("printf"));
    }
}
