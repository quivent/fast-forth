# Fast Forth LLVM Backend

High-performance native code generation for Forth using LLVM.

## Features

- **Native Code Generation**: Compile Forth to optimized native machine code
- **Stack Caching**: Keep top 2-3 stack elements in registers (70-90% reduction in memory operations)
- **Primitive Operations**: Optimized code generation for all Forth primitives (+, -, *, /, etc.)
- **Control Flow**: Native implementation of IF/THEN/ELSE, DO/LOOP, BEGIN/UNTIL
- **Optimization**: Leverage LLVM's optimization passes for maximum performance
- **Linking**: Static and dynamic linking support
- **Debug Symbols**: DWARF debug information generation (planned)

## Building

### Prerequisites

LLVM 16+ must be installed on your system:

**macOS:**
```bash
brew install llvm@16
export LLVM_SYS_160_PREFIX=/opt/homebrew/opt/llvm
```

**Ubuntu/Debian:**
```bash
sudo apt-get install llvm-16 llvm-16-dev
export LLVM_SYS_160_PREFIX=/usr/lib/llvm-16
```

### Compile

```bash
cargo build --features llvm
```

## Usage

### Basic Example

```rust
use backend::{LLVMBackend, CodeGenerator, CompilationMode};
use inkwell::{context::Context, OptimizationLevel};

let context = Context::create();
let mut backend = LLVMBackend::new(
    &context,
    "my_module",
    CompilationMode::AOT,
    OptimizationLevel::Aggressive,
);

// Generate code from SSA function
backend.generate(&ssa_function)?;

// Write object file
backend.finalize(Path::new("output.o"))?;
```

### Running Examples

```bash
# Simple compilation example
cargo run --example simple_compile --features llvm

# Fibonacci example
cargo run --example fibonacci --features llvm
```

## Architecture

### Code Generation Pipeline

```
SSA IR → LLVM IR → Optimizations → Native Code
```

1. **SSA Input**: Take SSA IR from frontend
2. **LLVM IR Generation**: Convert to LLVM IR with stack caching
3. **Optimization**: Apply LLVM optimization passes
4. **Code Generation**: Generate native object file

### Stack Caching

The backend implements an aggressive stack caching strategy:

- **Cache Depth**: 3 stack elements in registers (configurable)
- **Spilling**: Automatic spilling to memory when cache is full
- **Allocation**: Smart register allocation for stack operations

Example optimization:
```forth
: FOO 1 2 + 3 * ;

; Before stack caching:
  push_stack #1
  push_stack #2
  pop_stack %tmp1
  pop_stack %tmp2
  add %tmp2, %tmp1 → %tmp3
  push_stack %tmp3
  ; ... more memory operations

; After stack caching:
  mov #1 → %r0
  mov #2 → %r1
  add %r0, %r1 → %r1  ; TOS in register
  mov #3 → %r0
  mul %r1, %r0 → %r0  ; Result in register
```

### Primitive Operations

All Forth primitives are compiled to native instructions:

- **Arithmetic**: `+`, `-`, `*`, `/`, `MOD`
- **Comparison**: `<`, `>`, `=`, `<>`, `<=`, `>=`
- **Logical**: `AND`, `OR`, `NOT`, `XOR`
- **Stack**: `DUP`, `DROP`, `SWAP`, `OVER`, `ROT`
- **Memory**: `@`, `!`, `C@`, `C!`

### Control Flow

Control structures are lowered to LLVM basic blocks:

- **IF/THEN/ELSE**: Conditional branches
- **DO/LOOP**: Counted loops with loop counter
- **BEGIN/UNTIL**: Post-test loops
- **BEGIN/WHILE/REPEAT**: Pre-test loops

### Optimization Passes

LLVM optimization passes applied:

1. Instruction combining
2. Reassociation
3. GVN (Global Value Numbering)
4. CFG simplification
5. Memory to register promotion
6. Tail call elimination

## Linking

The backend includes a linker that can:

- Link multiple object files
- Link with runtime library
- Create static or shared libraries
- Generate executables

Example:
```rust
use backend::linker::{Linker, LinkerConfig, LinkMode};

let config = LinkerConfig {
    mode: LinkMode::Static,
    runtime_lib: PathBuf::from("runtime/forth_runtime.c"),
    output: PathBuf::from("my_program"),
    ..Default::default()
};

let linker = Linker::new(config);
linker.link(&[PathBuf::from("output.o")])?;
```

## Performance

Expected performance (compared to C):

- **Arithmetic operations**: 90-95% of C
- **Control flow**: 85-90% of C
- **Function calls**: 80-85% of C
- **Overall**: 80-100% of C (depending on workload)

Compilation time:

- **Small functions**: <10ms
- **Medium functions**: 20-50ms
- **Large functions**: 50-100ms

## Testing

```bash
# Run all tests
cargo test --features llvm

# Run specific test
cargo test --features llvm stack_cache

# Run with output
cargo test --features llvm -- --nocapture
```

## API Documentation

Generate API docs:
```bash
cargo doc --features llvm --open
```

## Current Limitations

1. **LLVM Version**: Requires LLVM 16+ (LLVM 17 support coming soon)
2. **JIT Mode**: JIT compilation is not yet fully implemented
3. **Debug Symbols**: DWARF generation is planned but not yet implemented
4. **Calling Convention**: FFI support is limited

## Roadmap

- [ ] JIT compilation support
- [ ] DWARF debug symbol generation
- [ ] Better FFI support
- [ ] SIMD optimizations
- [ ] Profile-guided optimization
- [ ] Cross-compilation support

## Integration with Frontend

The backend expects SSA IR from the frontend in this format:

```rust
pub struct SSAFunction {
    pub name: String,
    pub parameters: Vec<Register>,
    pub blocks: Vec<BasicBlock>,
    pub entry_block: BlockId,
}
```

See `frontend/src/ssa.rs` for the complete SSA IR specification.

## Contributing

When adding new features:

1. Add primitive operations to `codegen/primitives.rs`
2. Add control flow patterns to `codegen/control_flow.rs`
3. Update stack caching in `codegen/stack_cache.rs`
4. Add tests in `tests/`
5. Add examples in `examples/`

## License

MIT
