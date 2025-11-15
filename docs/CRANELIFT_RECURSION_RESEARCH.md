# Cranelift JIT Recursive Function Implementation Research

**Research Date:** 2025-11-15
**Target Version:** Cranelift 0.102.x
**Researcher:** Researcher Agent

## Executive Summary

Cranelift fully supports recursive function calls without requiring special trampolines or indirect call mechanisms. The key requirement is proper function declaration order using the Module API's `declare_function` and `declare_func_in_func` methods.

## Key Findings

### 1. Recursive Functions Are Fully Supported

Cranelift handles recursive function calls through direct call instructions. No special handling, trampolines, or indirect calls are required for recursion.

**Evidence:** The official cranelift-jit-demo repository includes a working recursive Fibonacci implementation:

```
fn recursive_fib(n) -> (r) {
    r = if n == 0 {
                0
        } else {
            if n == 1 {
                1
            } else {
                recursive_fib(n - 1) + recursive_fib(n - 2)
            }
        }
}
```

### 2. Implementation Pattern: Declare Before Define

The critical pattern for recursive functions in Cranelift:

```rust
// Step 1: Declare the function in the module
let func_id = module.declare_function(
    "my_recursive_func",
    Linkage::Local,
    &signature
).unwrap();

// Step 2: Create function context
let mut ctx = module.make_context();
ctx.func.signature = signature.clone();

// Step 3: Build function body
let mut builder_context = FunctionBuilderContext::new();
let mut builder = FunctionBuilder::new(&mut ctx.func, &mut builder_context);

// Step 4: Declare function reference within itself for recursive calls
let local_func_ref = module.declare_func_in_func(func_id, &mut ctx.func);

// Step 5: Use the local reference in call instructions
let call_inst = builder.ins().call(local_func_ref, &[arg1, arg2]);

// Step 6: Complete and define the function
builder.seal_all_blocks();
builder.finalize();
module.define_function(func_id, &mut ctx).unwrap();
```

### 3. Module API Methods

**`declare_function`**
- Purpose: Register a function signature with the module
- Returns: `FuncId` - a module-level function identifier
- When to call: Before building the function body
- Usage for recursion: Must be called before the function can reference itself

**`declare_func_in_func`**
- Signature: `fn declare_func_in_func(func_id: FuncId, func: &mut Function) -> FuncRef`
- Purpose: Create a function-local reference to a module-declared function
- Returns: `FuncRef` - usable with `builder.ins().call()`
- Critical for recursion: Allows a function to reference itself during construction

**`define_function`**
- Purpose: Finalize the function with compiled IR
- When to call: After building the complete function body
- Requirement: Must be called before `finalize_definitions()` or `finish()`

### 4. No Special Recursion Handling Required

Cranelift does NOT require:
- ❌ Trampolines for recursive calls
- ❌ Indirect calls for self-reference
- ❌ Special calling conventions
- ❌ Stack management code (unless sandboxed)

Cranelift DOES use:
- ✅ Direct call instructions (`call`)
- ✅ Standard calling conventions
- ✅ Regular function preambles/epilogues

### 5. Call Instruction Details

From Cranelift IR documentation:

```rust
// Direct call to a function reference
let result = builder.ins().call(func_ref, &[args]);

// For functions too far away (>±2GB on x86_64)
// Cranelift handles this automatically - no user action needed
```

**Important:** Cranelift emits 32-bit relocations for calls on x86_64, limiting addressable range to ±2GB. For functions beyond this range, Cranelift automatically handles the situation, but this is transparent to the user.

### 6. Working Example from cranelift-jit-demo

The canonical example repository is: https://github.com/bytecodealliance/cranelift-jit-demo

**Key implementation file:** `src/jit.rs`

Pattern used for function calls (including recursive):

```rust
// In the JIT compiler's translate_call method:

// 1. Ensure the callee is declared in the module
let callee_func_id = /* lookup or declare the function */;

// 2. Declare the function reference within the current function
let local_callee = self.module
    .declare_func_in_func(callee_func_id, &mut self.builder.func);

// 3. Prepare arguments
let arg_values = /* translate argument expressions */;

// 4. Emit the call instruction
let call_inst = self.builder.ins().call(local_callee, &arg_values);

// 5. Extract return value(s)
let return_values = self.builder.inst_results(call_inst);
```

For recursive calls, `callee_func_id` is the same as the function currently being built.

### 7. Known Issues and Considerations

#### Stack Overflow
- Cranelift does not automatically insert stack overflow checks
- For sandboxed code or deep recursion, manual stack checks may be needed
- Reference: Issue #349 - "Stack limit checking"

#### Tail Call Optimization
- Tail call elimination is not fully supported in all backends
- Issue #1065 discusses tail call implementation
- Recursive functions using tail calls may still consume stack space
- Alternative: Use iteration or manually implement trampolines in your language runtime

#### Function Finalization Order
- Must call `module.finalize_definitions()` before accessing function pointers
- Issue #1288 documents this requirement
- All mutually recursive functions should be defined before finalization

#### Relocation Limits (x86_64)
- Direct calls limited to ±2GB range due to 32-bit relocations
- Cranelift handles this automatically
- May affect performance in very large codebases
- Reference: Issue #4000 - "JIT relocations depend on system allocator behaviour"

### 8. Version-Specific Information (0.102.x)

Cranelift 0.102.0 was released on November 20, 2023.

**No breaking changes specific to recursive functions were found between 0.102.x and adjacent versions.**

The recursive function implementation pattern has been stable across recent Cranelift versions.

Cranelift is not 1.0+, so breaking changes can occur on minor version bumps. However, the core Module API (`declare_function`, `declare_func_in_func`, `define_function`) has remained stable.

### 9. Alternative Examples and Tutorials

**Official Resources:**
- cranelift-jit-demo: https://github.com/bytecodealliance/cranelift-jit-demo
- Wasmtime documentation: https://docs.wasmtime.dev/
- Cranelift IR documentation: https://github.com/bytecodealliance/wasmtime/blob/main/cranelift/docs/ir.md

**Community Examples:**
- "Compiling Brainfuck code - Part 3: A Cranelift JIT Compiler" by Rodrigodd
  - URL: https://rodrigodd.github.io/2022/11/26/bf_compiler-part3.html
  - Shows practical JIT compilation patterns

- "A primer on code generation in Cranelift" by Benjamin Bouvier
  - URL: https://bouvier.cc/2021/02/17/cranelift-codegen-primer/
  - Covers Cranelift fundamentals

- "Building a Brainfuck Compiler using Cranelift" by Clemens Tiedt
  - URL: https://blog.tiedt.dev/article/brainfuck_compiler
  - Demonstrates function builder patterns

**Code Repositories Using Cranelift with Recursion:**
- rustc_codegen_cranelift: Rust compiler backend using Cranelift
  - https://github.com/rust-lang/rust/tree/master/compiler/rustc_codegen_cranelift
- wasmtime: WebAssembly runtime using Cranelift
  - Handles recursive WebAssembly functions

### 10. Complete Minimal Example

Here's a complete minimal example for implementing a recursive factorial function:

```rust
use cranelift::prelude::*;
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{Linkage, Module};

fn create_recursive_factorial() -> Result<*const u8, String> {
    // Initialize JIT
    let mut builder = JITBuilder::new(cranelift_module::default_libcall_names())
        .map_err(|e| format!("JIT builder error: {}", e))?;
    let mut module = JITModule::new(builder);

    // Create function signature: i64 factorial(i64 n)
    let mut sig = module.make_signature();
    sig.params.push(AbiParam::new(types::I64));
    sig.returns.push(AbiParam::new(types::I64));

    // Step 1: Declare the function
    let func_id = module
        .declare_function("factorial", Linkage::Export, &sig)
        .map_err(|e| format!("Declare error: {}", e))?;

    // Step 2: Create context and builder
    let mut ctx = module.make_context();
    ctx.func.signature = sig.clone();

    let mut builder_ctx = FunctionBuilderContext::new();
    let mut builder = FunctionBuilder::new(&mut ctx.func, &mut builder_ctx);

    // Step 3: Declare function reference for recursive calls
    let factorial_ref = module.declare_func_in_func(func_id, &mut ctx.func);

    // Step 4: Build function body
    let entry_block = builder.create_block();
    builder.append_block_params_for_function_params(entry_block);
    builder.switch_to_block(entry_block);
    builder.seal_block(entry_block);

    let n = builder.block_params(entry_block)[0];

    // Create blocks for control flow
    let base_case_block = builder.create_block();
    let recursive_case_block = builder.create_block();
    let merge_block = builder.create_block();
    builder.append_block_param(merge_block, types::I64);

    // Check if n <= 1 (base case)
    let one = builder.ins().iconst(types::I64, 1);
    let is_base_case = builder.ins().icmp(IntCC::SignedLessThanOrEqual, n, one);
    builder.ins().brif(is_base_case, base_case_block, &[], recursive_case_block, &[]);

    // Base case: return 1
    builder.switch_to_block(base_case_block);
    builder.seal_block(base_case_block);
    builder.ins().jump(merge_block, &[one]);

    // Recursive case: return n * factorial(n - 1)
    builder.switch_to_block(recursive_case_block);
    builder.seal_block(recursive_case_block);

    let n_minus_one = builder.ins().isub(n, one);

    // RECURSIVE CALL HERE
    let call_inst = builder.ins().call(factorial_ref, &[n_minus_one]);
    let recursive_result = builder.inst_results(call_inst)[0];

    let result = builder.ins().imul(n, recursive_result);
    builder.ins().jump(merge_block, &[result]);

    // Merge block: return the result
    builder.switch_to_block(merge_block);
    builder.seal_block(merge_block);
    let final_result = builder.block_params(merge_block)[0];
    builder.ins().return_(&[final_result]);

    // Step 5: Finalize and define
    builder.finalize();
    module
        .define_function(func_id, &mut ctx)
        .map_err(|e| format!("Define error: {}", e))?;

    // Step 6: Finalize all definitions and get function pointer
    module.finalize_definitions().unwrap();
    let code_ptr = module.get_finalized_function(func_id);

    Ok(code_ptr)
}

// Usage:
fn main() {
    let factorial_ptr = create_recursive_factorial().unwrap();
    let factorial: fn(i64) -> i64 = unsafe { std::mem::transmute(factorial_ptr) };

    println!("factorial(5) = {}", factorial(5));  // Should print 120
    println!("factorial(10) = {}", factorial(10)); // Should print 3628800
}
```

### 11. Best Practices Summary

1. **Declaration Order:**
   - Always `declare_function()` before building the function body
   - Use `declare_func_in_func()` to get callable references

2. **Module Management:**
   - Use a single `Module` instance for all mutually recursive functions
   - Call `finalize_definitions()` after all functions are defined

3. **Error Handling:**
   - Check return values from `declare_function()` and `define_function()`
   - Validate function signatures match at call sites

4. **Performance:**
   - Cranelift uses direct calls (efficient)
   - No runtime overhead for recursion vs. normal calls
   - Stack consumption is standard (no hidden allocations)

5. **Testing:**
   - Test base cases first
   - Verify stack behavior with deep recursion
   - Consider adding stack overflow protection for untrusted code

## Conclusion

Recursive function calls in Cranelift are straightforward and efficient. The key requirement is proper use of the Module API to declare functions before they reference themselves. No special recursion handling, trampolines, or indirect calls are needed.

The pattern is:
1. `declare_function()` - register with module
2. `declare_func_in_func()` - get callable reference
3. `builder.ins().call()` - emit call instruction
4. `define_function()` - finalize the function

This pattern has been stable across Cranelift versions and is well-demonstrated in the official cranelift-jit-demo repository.

## References

- **cranelift-jit-demo:** https://github.com/bytecodealliance/cranelift-jit-demo
- **Cranelift Documentation:** https://cranelift.dev/
- **Wasmtime Repository:** https://github.com/bytecodealliance/wasmtime
- **Cranelift IR Docs:** https://github.com/bytecodealliance/wasmtime/blob/main/cranelift/docs/ir.md
- **Module API Docs:** https://docs.rs/cranelift-module/
- **JIT API Docs:** https://docs.rs/cranelift-jit/

## Related Issues

- Issue #675: "How to use cranelift build a function and call rust function?"
- Issue #1288: "`finalize_definitions` needs to be called before `finish`"
- Issue #349: "Stack limit checking"
- Issue #1065: "Implement support for tail calls in Wasmtime and Cranelift"
- Issue #4000: "JIT relocations depend on system allocator behaviour"
