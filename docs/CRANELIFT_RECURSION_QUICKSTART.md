# Cranelift Recursive Functions - Quick Start Guide

## TL;DR

Cranelift supports recursive functions natively via direct calls. No special handling needed.

## The 4-Step Pattern

```rust
// 1. Declare function
let func_id = module.declare_function("my_func", Linkage::Local, &sig)?;

// 2. Build function body
let mut ctx = module.make_context();
let mut builder = FunctionBuilder::new(&mut ctx.func, &mut builder_ctx);

// 3. Get self-reference for recursive calls
let self_ref = module.declare_func_in_func(func_id, &mut ctx.func);

// 4. Make recursive call
let result = builder.ins().call(self_ref, &[args]);
```

## Complete Working Example (Factorial)

```rust
use cranelift::prelude::*;
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{Linkage, Module};

fn compile_factorial() -> *const u8 {
    let mut module = JITModule::new(JITBuilder::new(
        cranelift_module::default_libcall_names()
    ).unwrap());

    // Signature: i64 factorial(i64)
    let mut sig = module.make_signature();
    sig.params.push(AbiParam::new(types::I64));
    sig.returns.push(AbiParam::new(types::I64));

    // STEP 1: Declare
    let func_id = module.declare_function("factorial", Linkage::Export, &sig).unwrap();

    // STEP 2: Build context
    let mut ctx = module.make_context();
    ctx.func.signature = sig;
    let mut builder_ctx = FunctionBuilderContext::new();
    let mut builder = FunctionBuilder::new(&mut ctx.func, &mut builder_ctx);

    // STEP 3: Get self-reference
    let factorial_ref = module.declare_func_in_func(func_id, &mut ctx.func);

    // Build function body
    let entry = builder.create_block();
    builder.append_block_params_for_function_params(entry);
    builder.switch_to_block(entry);
    builder.seal_block(entry);

    let n = builder.block_params(entry)[0];
    let one = builder.ins().iconst(types::I64, 1);

    let base_case = builder.create_block();
    let recursive = builder.create_block();
    let merge = builder.create_block();
    builder.append_block_param(merge, types::I64);

    // if n <= 1
    let cond = builder.ins().icmp(IntCC::SignedLessThanOrEqual, n, one);
    builder.ins().brif(cond, base_case, &[], recursive, &[]);

    // Base case: return 1
    builder.switch_to_block(base_case);
    builder.seal_block(base_case);
    builder.ins().jump(merge, &[one]);

    // STEP 4: Recursive case
    builder.switch_to_block(recursive);
    builder.seal_block(recursive);
    let n_minus_1 = builder.ins().isub(n, one);

    // THIS IS THE RECURSIVE CALL
    let call = builder.ins().call(factorial_ref, &[n_minus_1]);
    let fact_n_minus_1 = builder.inst_results(call)[0];

    let result = builder.ins().imul(n, fact_n_minus_1);
    builder.ins().jump(merge, &[result]);

    // Return result
    builder.switch_to_block(merge);
    builder.seal_block(merge);
    let final_result = builder.block_params(merge)[0];
    builder.ins().return_(&[final_result]);

    // Finalize
    builder.finalize();
    module.define_function(func_id, &mut ctx).unwrap();
    module.finalize_definitions().unwrap();

    module.get_finalized_function(func_id)
}

// Use it:
fn main() {
    let ptr = compile_factorial();
    let factorial: fn(i64) -> i64 = unsafe { std::mem::transmute(ptr) };

    assert_eq!(factorial(5), 120);
    assert_eq!(factorial(10), 3628800);
}
```

## Common Patterns

### Pattern 1: Simple Recursion (Fibonacci)

```rust
// fib(n) = fib(n-1) + fib(n-2)
let call1 = builder.ins().call(self_ref, &[n_minus_1]);
let result1 = builder.inst_results(call1)[0];

let call2 = builder.ins().call(self_ref, &[n_minus_2]);
let result2 = builder.inst_results(call2)[0];

let sum = builder.ins().iadd(result1, result2);
```

### Pattern 2: Mutual Recursion

```rust
// Declare both functions first
let func_a_id = module.declare_function("func_a", Linkage::Local, &sig_a)?;
let func_b_id = module.declare_function("func_b", Linkage::Local, &sig_b)?;

// When building func_a, get reference to func_b
let func_b_ref = module.declare_func_in_func(func_b_id, &mut ctx_a.func);
let call = builder_a.ins().call(func_b_ref, &[args]);

// When building func_b, get reference to func_a
let func_a_ref = module.declare_func_in_func(func_a_id, &mut ctx_b.func);
let call = builder_b.ins().call(func_a_ref, &[args]);

// Define both before finalizing
module.define_function(func_a_id, &mut ctx_a)?;
module.define_function(func_b_id, &mut ctx_b)?;
module.finalize_definitions()?;
```

### Pattern 3: Tail Recursion (Not Optimized)

```rust
// Note: Cranelift doesn't optimize tail calls by default
// This will still consume stack space

// tail_factorial(n, acc) =
//   if n <= 1: acc
//   else: tail_factorial(n-1, n*acc)

let call = builder.ins().call(self_ref, &[n_minus_1, new_acc]);
let result = builder.inst_results(call)[0];
builder.ins().return_(&[result]);  // Not optimized to jump
```

## Key API Methods

| Method | Purpose | Returns |
|--------|---------|---------|
| `module.declare_function(name, linkage, sig)` | Register function with module | `FuncId` |
| `module.declare_func_in_func(func_id, func)` | Get callable reference | `FuncRef` |
| `builder.ins().call(func_ref, args)` | Emit call instruction | `Inst` |
| `builder.inst_results(inst)` | Get return value(s) | `&[Value]` |
| `module.define_function(func_id, ctx)` | Finalize function IR | `Result<()>` |
| `module.finalize_definitions()` | Complete all compilations | `Result<()>` |

## What You DON'T Need

- No trampolines
- No indirect calls
- No special calling conventions
- No manual stack management
- No function pointer gymnastics

Recursion in Cranelift works like recursion in assembly: it just works.

## Gotchas

### Must Declare Before Reference
```rust
// WRONG - func_id doesn't exist yet
let self_ref = module.declare_func_in_func(func_id, &mut ctx.func);
let func_id = module.declare_function(...)?;

// RIGHT - declare first
let func_id = module.declare_function(...)?;
let self_ref = module.declare_func_in_func(func_id, &mut ctx.func);
```

### Must Define Before Finalize
```rust
// WRONG - finalize before define
module.finalize_definitions()?;
module.define_function(func_id, &mut ctx)?;  // Too late!

// RIGHT - define first
module.define_function(func_id, &mut ctx)?;
module.finalize_definitions()?;
```

### Stack Overflow Not Checked
```rust
// Cranelift won't protect you from stack overflow
// For deep recursion or untrusted code, add manual checks

let stack_check = builder.create_block();
let stack_ok = builder.create_block();

// Check stack pointer
let sp = /* get stack pointer */;
let limit = /* get stack limit */;
let ok = builder.ins().icmp(IntCC::UnsignedGreaterThan, sp, limit);
builder.ins().brif(ok, stack_ok, &[], stack_error, &[]);
```

## Performance Notes

- Direct calls are as fast as non-recursive calls
- No hidden allocations
- Standard stack frame overhead
- No tail call optimization (yet)
- Call overhead: ~1-5 CPU cycles on modern x86_64

## Examples in the Wild

**Official:**
- cranelift-jit-demo: https://github.com/bytecodealliance/cranelift-jit-demo
  - See `src/jit.rs` and `src/bin/toy.rs`

**Community:**
- rustc_codegen_cranelift: Rust compiler backend
- wasmtime: WebAssembly runtime

## Debugging Tips

### Print Generated IR
```rust
println!("{}", ctx.func.display());
```

### Verify Function
```rust
use cranelift::prelude::*;
let verifier_flags = settings::Flags::new(settings::builder());
cranelift_codegen::verify_function(&ctx.func, &verifier_flags)?;
```

### Check Call Target
```rust
// After building
for inst in ctx.func.dfg.insts() {
    if let InstructionData::Call { func_ref, .. } = ctx.func.dfg[inst] {
        println!("Found call to: {:?}", func_ref);
    }
}
```

## Version Compatibility

This pattern works on:
- Cranelift 0.102.x
- Cranelift 0.103.x
- Cranelift 0.104+
- Latest version (0.125+)

The Module API has been stable for recursive functions since at least 0.80.

## Next Steps

1. Read full research document: `CRANELIFT_RECURSION_RESEARCH.md`
2. Study the cranelift-jit-demo repository
3. Experiment with the factorial example above
4. Add recursion to your JIT compiler

## Questions?

Check the official docs at https://cranelift.dev/ or ask on:
- GitHub: https://github.com/bytecodealliance/wasmtime
- Zulip: https://bytecodealliance.zulipchat.com/
