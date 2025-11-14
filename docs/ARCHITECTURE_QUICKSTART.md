# Fast Forth Architecture - Quick Start Guide
**Version**: 1.0
**Date**: 2025-11-14

## Overview

This quick start guide provides a condensed reference for developers implementing Fast Forth. For detailed specifications, see the full architecture documents.

---

## 1. Core Design Decisions

### Key Architectural Choices

| Aspect | Decision | Rationale |
|--------|----------|-----------|
| **IR Strategy** | Three-tier (HIR/MIR/LIR) | Balance compilation speed & optimization |
| **Type System** | Hindley-Milner + Stack Effects | Static safety with polymorphism |
| **Backend** | LLVM + Threaded Code | Performance range: 60-100% of C |
| **Compilation** | AOT + JIT with tiering | Interactive dev + production performance |
| **Optimization** | Stack caching + specialization | Forth-specific + standard passes |
| **Plugin System** | Trait-based, IR hooks | Extensibility without core changes |

### Performance Targets

```
Compile Time: <100ms for typical programs
  - Frontend: 10-15ms
  - Type inference: 10-20ms
  - Optimization: 30-50ms
  - LLVM codegen: 30-50ms

Runtime: 80-100% of C
  - Threaded code: 60-70% (fast compile)
  - LLVM -O1: 85% (balanced)
  - LLVM -O3: 95-100% (aggressive)

Memory: Minimal overhead
  - Stack cache: 64 bytes
  - Dictionary: ~100 bytes/word
  - JIT cache: 64MB default
```

---

## 2. Component Overview

### Compilation Pipeline

```
Source → [Frontend] → AST → [Type Checker] → Typed AST
                                                  ↓
                                            [IR Builder]
                                                  ↓
                    HIR (Forth-like, implicit stack)
                                                  ↓
                          [HIR Optimizations]
                    Inlining, constant propagation
                                                  ↓
                    MIR (SSA form, explicit values)
                                                  ↓
                          [MIR Optimizations]
       Stack caching, CSE, DCE, specialization, plugins
                                                  ↓
                    LIR (Register-based, target-aware)
                                                  ↓
                          [Backend Selection]
                    ┌──────────┬──────────┐
                    ▼          ▼          ▼
              Threaded    LLVM IR    Native
```

### Module Responsibilities

```
src/
├── frontend/
│   ├── lexer.rs          # Tokenization
│   ├── parser.rs         # Recursive descent parser
│   └── ast.rs            # AST data structures
│
├── types/
│   ├── inference.rs      # HM type inference
│   ├── constraints.rs    # Constraint solving
│   └── effects.rs        # Stack effect analysis
│
├── ir/
│   ├── hir.rs            # High-level IR
│   ├── mir.rs            # Mid-level IR (SSA)
│   ├── lir.rs            # Low-level IR
│   └── transform.rs      # IR lowering
│
├── optimize/
│   ├── stack_cache.rs    # Stack → register optimization
│   ├── superinst.rs      # Instruction fusion
│   ├── inline.rs         # Function inlining
│   ├── specialize.rs     # Type specialization
│   └── passes.rs         # Pass manager
│
├── backend/
│   ├── llvm/             # LLVM integration
│   ├── threaded/         # Threaded code
│   └── jit/              # JIT engine (OrcJIT)
│
├── runtime/
│   ├── stack.rs          # Stack management
│   ├── memory.rs         # Heap & GC
│   └── ffi.rs            # C interop
│
└── plugin/
    ├── api.rs            # Plugin trait definitions
    └── loader.rs         # Dynamic loading
```

---

## 3. Key Data Structures

### Type System

```rust
// Polymorphic type scheme
TypeScheme {
    quantified: Vec<TypeVar>,      // ∀α β
    constraints: Vec<Constraint>,   // Numeric(α)
    ty: Type,                       // ( α α -- α )
}

// Stack effect
StackEffect {
    inputs: Vec<Type>,    // ( a b c
    outputs: Vec<Type>,   //       -- d e )
}

// Type constraints
enum Constraint {
    Equal(Type, Type),
    Numeric(TypeVar),
    Ordered(TypeVar),
    // ...
}
```

### IR Representation

```rust
// HIR: Forth-like
enum HIRInstruction {
    Call { word_id, immediate, type_sig },
    Literal(Literal),
    StackOp(StackOp),
    If { then_block, else_block },
    // ...
}

// MIR: SSA form
enum MIRInstruction {
    BinOp { op, lhs: ValueId, rhs: ValueId, result: ValueId },
    Call { callee, args: Vec<ValueId>, results: Vec<ValueId> },
    Load { addr: ValueId, result: ValueId },
    // ...
}

// LIR: Register-based
enum LIRInstruction {
    Move { src: Operand, dst: Operand },
    Add { src: Operand, dst: Operand },
    Jump { target: Label },
    // ...
}
```

---

## 4. Development Streams

### Stream Assignment Matrix

| Stream | Component | Dependencies | Key Files |
|--------|-----------|--------------|-----------|
| **STREAM 2** | Frontend | None | `frontend/*.rs` |
| **STREAM 3** | Type System | STREAM 2 | `types/*.rs` |
| **STREAM 4** | IR Builder | STREAM 2, 3 | `ir/*.rs` |
| **STREAM 5** | Optimizer | STREAM 4 | `optimize/*.rs` |
| **STREAM 6** | LLVM Backend | STREAM 4, 5 | `backend/llvm/*.rs` |
| **STREAM 7** | Runtime/JIT | STREAM 4, 6 | `runtime/*.rs`, `backend/jit/*.rs` |
| **STREAM 8** | Testing | ALL | `tests/*.rs` |

### Development Order

```
Phase 1: Foundation (Weeks 1-2)
  ├─ STREAM 2: Lexer + Parser
  └─ STREAM 3: Basic type checking

Phase 2: IR & Core Optimization (Weeks 3-4)
  ├─ STREAM 4: HIR + MIR implementation
  ├─ STREAM 5: Stack caching optimization
  └─ STREAM 6: Basic LLVM backend

Phase 3: Advanced Features (Weeks 5-6)
  ├─ STREAM 5: Specialization + plugins
  ├─ STREAM 7: JIT engine
  └─ STREAM 8: Integration tests

Phase 4: Optimization & Tuning (Weeks 7-8)
  ├─ All streams: Performance tuning
  └─ STREAM 8: Benchmark suite
```

---

## 5. Quick Implementation Guide

### Adding a New Word

1. **Define type signature** in `types/builtins.rs`:
```rust
types.insert("MY-WORD".to_string(), TypeScheme {
    quantified: vec![TypeVar(0)],
    constraints: vec![Constraint::Numeric(TypeVar(0))],
    ty: Type::Function(StackEffect {
        inputs: vec![Type::Unknown(TypeVar(0))],
        outputs: vec![Type::Unknown(TypeVar(0))],
    }),
});
```

2. **Implement HIR handling** in `ir/hir.rs`:
```rust
HIRInstruction::Call { word_id, .. } if word_id == MY_WORD => {
    // Custom HIR generation if needed
}
```

3. **Add MIR lowering** in `ir/transform.rs`:
```rust
fn lower_my_word(&mut self, ...) -> Vec<MIRInstruction> {
    // Generate MIR instructions
}
```

4. **Add backend support** in `backend/llvm/codegen.rs`:
```rust
fn emit_my_word(&self, ...) -> LLVMValueRef {
    // Generate LLVM IR
}
```

### Adding an Optimization Pass

1. **Implement OptimizationPass trait**:
```rust
pub struct MyOptimizationPass;

impl OptimizationPass for MyOptimizationPass {
    fn name(&self) -> &str { "my-optimization" }

    fn run(&mut self, mir: &mut MIRFunction) -> Result<bool> {
        let mut changed = false;

        for block in &mut mir.blocks {
            for inst in &mut block.instructions {
                // Transform instructions
                if self.should_optimize(inst) {
                    self.optimize(inst);
                    changed = true;
                }
            }
        }

        Ok(changed)
    }
}
```

2. **Register in pass manager** (`optimize/passes.rs`):
```rust
impl OptimizationConfig {
    pub fn from_level(level: OptLevel) -> Self {
        let passes = match level {
            OptLevel::Aggressive => vec![
                // ...
                Box::new(MyOptimizationPass::new()),
                // ...
            ],
        };
        // ...
    }
}
```

### Adding a Plugin

1. **Implement CompilerPlugin trait**:
```rust
pub struct MyPlugin;

impl CompilerPlugin for MyPlugin {
    fn name(&self) -> &str { "my-plugin" }

    fn register_optimizations(&self) -> Vec<Box<dyn OptimizationPass>> {
        vec![Box::new(MyOptimizationPass::new())]
    }

    fn on_mir_created(&mut self, mir: &mut MIRFunction) -> Result<()> {
        // Transform MIR
        Ok(())
    }
}
```

2. **Create plugin.toml**:
```toml
[plugin]
name = "my-plugin"
version = "1.0.0"
library = "libmy_plugin.so"

[dependencies]
fast-forth = "^1.0"
```

---

## 6. Testing Strategy

### Test Levels

```
Unit Tests (per module)
  └─ Test individual components in isolation

Integration Tests (per stream)
  └─ Test component interactions

End-to-End Tests (cross-stream)
  └─ Test complete compilation pipeline

Benchmark Tests
  └─ Measure performance vs. targets
```

### Example Tests

```rust
// Unit test (types/inference.rs)
#[test]
fn test_simple_inference() {
    let source = ": SQUARE DUP * ;";
    let ast = parse(source).unwrap();
    let ty = infer_type(ast).unwrap();

    assert_eq!(ty, TypeScheme {
        quantified: vec![TypeVar(0)],
        constraints: vec![Constraint::Numeric(TypeVar(0))],
        ty: Type::Function(StackEffect {
            inputs: vec![Type::Unknown(TypeVar(0))],
            outputs: vec![Type::Unknown(TypeVar(0))],
        }),
    });
}

// Integration test (tests/integration/)
#[test]
fn test_compile_and_run() {
    let source = r#"
        : FIB ( n -- fib[n] )
          DUP 2 < IF DROP 1 EXIT THEN
          DUP 1- RECURSE SWAP 2- RECURSE + ;
    "#;

    let compiler = Compiler::new();
    let compiled = compiler.compile(source).unwrap();
    let result = compiled.execute_word("FIB", &[10]).unwrap();

    assert_eq!(result, 55);
}

// Benchmark (benchmarks/)
#[bench]
fn bench_fibonacci_compile(b: &mut Bencher) {
    let source = include_str!("../fixtures/fibonacci.fth");

    b.iter(|| {
        let compiler = Compiler::new();
        compiler.compile(source).unwrap()
    });
}
```

---

## 7. Configuration

### Compiler Configuration

```rust
CompilerConfig {
    opt_level: OptLevel::Balanced,        // -O0, -O1, -O2, -O3
    backend: BackendKind::LLVM,           // LLVM, Threaded, JIT
    strict_stack_effects: true,           // Require annotations
    enable_plugins: true,                 // Load plugins
    plugin_dirs: vec!["./plugins"],       // Plugin search paths
    emit_debug_info: false,               // Debug symbols
    dump_ir: false,                       // Print IR for debugging
}
```

### Runtime Configuration

```rust
RuntimeConfig {
    data_stack_size: 64 * 1024,          // 64KB
    return_stack_size: 32 * 1024,        // 32KB
    heap_size: 16 * 1024 * 1024,         // 16MB
    stack_cache_depth: 8,                // Registers for stack
    jit_hot_threshold: 1000,             // Calls before recompile
}
```

---

## 8. Common Patterns

### Stack Effect Composition

```rust
// Compose two words
let word1_effect = StackEffect {
    inputs: vec![Type::Int32],
    outputs: vec![Type::Int32, Type::Int32],
};

let word2_effect = StackEffect {
    inputs: vec![Type::Int32, Type::Int32],
    outputs: vec![Type::Int32],
};

let composed = word1_effect.compose(&word2_effect)?;
// Result: ( Int32 -- Int32 )
```

### SSA Value Creation

```rust
// Create a new SSA value
let value_id = self.new_value();

// Emit MIR instruction
mir.add_instruction(
    block_id,
    MIRInstruction::BinOp {
        op: BinOpKind::Add,
        lhs: v1,
        rhs: v2,
        result: value_id,
        ty: Type::Int64,
    }
);
```

### Register Allocation

```rust
// Virtual registers in LIR
let v1 = Operand::Register(Register::Virtual(0));
let v2 = Operand::Register(Register::Virtual(1));

// After register allocation
let allocated = register_allocator.allocate(&lir);
let r1 = allocated.get(v1); // → Register::RAX
let r2 = allocated.get(v2); // → Register::RBX
```

---

## 9. Debugging Tips

### Enable IR Dumping

```bash
# Set environment variable
export FASTFORTH_DUMP_IR=1

# Compile with debug flag
fastforth compile --dump-ir program.fth
```

Output:
```
=== HIR ===
function SQUARE:
  Call(DUP)
  Call(MULTIPLY)

=== MIR ===
function SQUARE(v0: i64) -> i64 {
  bb0:
    v1 = binop mul, v0, v0
    return [v1]
}

=== LIR ===
function SQUARE:
  mov rdi, rax
  imul rax, rdi
  ret
```

### Type Inference Debugging

```rust
// Enable constraint logging
let mut ctx = TypeInferenceContext::new(env);
ctx.enable_logging(true);

let ty = ctx.infer_definition(&ast)?;

// Prints:
// [Constraint] Equal(Int32, Unknown(0))
// [Constraint] Numeric(Unknown(0))
// [Unification] Unknown(0) := Int32
```

---

## 10. External Resources

### Documentation References

- **Full Architecture**: `docs/ARCHITECTURE.md`
- **IR Specification**: `specs/IR_SPECIFICATION.md`
- **Type System**: `specs/TYPE_SYSTEM_SPECIFICATION.md`

### Related Projects

- **LLVM Documentation**: https://llvm.org/docs/
- **Inkwell (Rust LLVM)**: https://github.com/TheDan64/inkwell
- **OrcJIT**: https://llvm.org/docs/ORCv2.html

### Forth References

- **ANS Forth Standard**: https://forth-standard.org/
- **Factor Language**: https://factorcode.org/ (modern stack language)
- **gforth**: https://www.gnu.org/software/gforth/ (reference implementation)

---

## 11. FAQ

**Q: Why three IR levels?**
A: HIR preserves Forth semantics for Forth-specific optimizations, MIR enables standard compiler optimizations via SSA, LIR provides target-specific representation.

**Q: Why Hindley-Milner for Forth?**
A: Provides polymorphism without annotations while ensuring type safety. Natural fit for stack polymorphism.

**Q: How does stack caching work?**
A: Top N stack elements kept in registers, reducing memory operations by 70-90%.

**Q: What's the compilation latency budget?**
A: <100ms total, with most time in LLVM backend. Threaded code backend is <10ms for interactive use.

**Q: How are plugins loaded?**
A: Dynamic library loading via `libloading`. Plugins implement `CompilerPlugin` trait.

**Q: What's the difference between AOT and JIT?**
A: AOT compiles ahead-of-time for deployment. JIT compiles interactively with tiering: threaded → LLVM -O1 → LLVM -O3.

---

## 12. Next Steps

1. **Read full architecture**: `docs/ARCHITECTURE.md`
2. **Study IR spec**: `specs/IR_SPECIFICATION.md`
3. **Understand type system**: `specs/TYPE_SYSTEM_SPECIFICATION.md`
4. **Set up development environment**:
   ```bash
   # Install Rust toolchain
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

   # Install LLVM 17
   # macOS:
   brew install llvm@17

   # Linux:
   sudo apt install llvm-17-dev libclang-17-dev

   # Clone and build
   git clone https://github.com/fastforth/fastforth.git
   cd fastforth
   cargo build
   ```

5. **Choose your stream**: See section 4 for stream assignments
6. **Start implementing**: Follow the patterns in section 5

---

**Document Version**: 1.0
**Maintained By**: Architect Agent (STREAM 1)
**For Questions**: Open issue on GitHub or consult `docs/ARCHITECTURE.md`
