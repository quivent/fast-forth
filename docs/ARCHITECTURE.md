# Fast Forth: System Architecture
**Version**: 1.0
**Date**: 2025-11-14
**Status**: Master Architecture Reference
**Agent**: Architect-SystemDesign-2025-09-04

---

## Executive Summary

Fast Forth is a modern optimizing Forth compiler targeting C-level performance (80-100%) through LLVM backend integration, sophisticated type inference, and stack-oriented optimizations. The architecture supports both AOT compilation and JIT execution with compile times under 100ms for typical programs.

**Key Innovations**:
- Stack effect type system with Hindley-Milner inference
- Multi-tier IR enabling both rapid threaded code and optimized native code
- Plugin-based optimization framework
- Incremental compilation for interactive development
- Static stack analysis preventing runtime stack errors

---

## 1. System Architecture Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                      FAST FORTH COMPILER                         │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌──────────────┐      ┌──────────────┐      ┌──────────────┐  │
│  │   Frontend   │─────▶│  Type System │─────▶│   IR Builder │  │
│  │              │      │   (HM Inf.)  │      │              │  │
│  │  Lexer       │      │              │      │  HIR → MIR   │  │
│  │  Parser      │      │  Stack       │      │  MIR → LIR   │  │
│  │  AST Builder │      │  Effect      │      │              │  │
│  └──────────────┘      │  Analysis    │      └──────────────┘  │
│                        └──────────────┘                         │
│                                                                  │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │            Optimization Pipeline                          │  │
│  │                                                            │  │
│  │  ┌─────────────┐  ┌─────────────┐  ┌──────────────┐     │  │
│  │  │   Stack     │─▶│ Constant    │─▶│ Inlining &   │     │  │
│  │  │   Caching   │  │ Folding     │  │ Specializ.   │     │  │
│  │  └─────────────┘  └─────────────┘  └──────────────┘     │  │
│  │                                                            │  │
│  │  ┌─────────────┐  ┌─────────────┐  ┌──────────────┐     │  │
│  │  │ Super-      │─▶│ Dead Code   │─▶│ Plugin       │     │  │
│  │  │ instructions│  │ Elimination │  │ Optimizers   │     │  │
│  │  └─────────────┘  └─────────────┘  └──────────────┘     │  │
│  └──────────────────────────────────────────────────────────┘  │
│                                                                  │
│  ┌──────────────┐      ┌──────────────┐      ┌──────────────┐  │
│  │   Backend    │      │   JIT        │      │   Runtime    │  │
│  │   Selector   │      │   Engine     │      │   Support    │  │
│  │              │      │              │      │              │  │
│  │  LLVM IR Gen │      │  OrcJIT v2   │      │  GC          │  │
│  │  Threaded    │      │  Lazy Comp.  │      │  FFI Bridge  │  │
│  │  Direct/Ind. │      │  Tiering     │      │  Stack Mgmt  │  │
│  └──────────────┘      └──────────────┘      └──────────────┘  │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘

              ┌────────────────────────────────┐
              │      Target Environments       │
              ├────────────────────────────────┤
              │  x86-64  │  ARM64  │  WASM    │
              │  Linux   │  macOS  │  Browser │
              │  Windows │  iOS    │  Embedded│
              └────────────────────────────────┘
```

---

## 2. Component Architecture

### 2.1 Frontend Components

```
┌──────────────────────────────────────────────────────────┐
│                     FRONTEND PIPELINE                     │
└──────────────────────────────────────────────────────────┘

Source Code
    │
    ▼
┌──────────┐
│  Lexer   │  Tokenization with source location tracking
└──────────┘
    │ Tokens
    ▼
┌──────────┐
│  Parser  │  Recursive descent with error recovery
└──────────┘
    │ Parse Tree
    ▼
┌──────────┐
│ AST      │  Abstract syntax tree with metadata
│ Builder  │  (definitions, stack comments, immediacy)
└──────────┘
    │ AST
    ▼
┌──────────────┐
│ Stack Effect │  Static analysis of stack behavior
│ Analyzer     │  ( a b -- c ) → Type constraints
└──────────────┘
    │ Effect Signatures
    ▼
┌──────────────┐
│ Type         │  Hindley-Milner inference
│ Inference    │  Unification & constraint solving
└──────────────┘
    │ Typed AST
    ▼
  HIR (High-Level IR)
```

**Key Data Structures**:

```rust
// Source location for error reporting
pub struct SourceLocation {
    pub file: FileId,
    pub line: u32,
    pub column: u32,
    pub span: Range<usize>,
}

// Token with metadata
pub struct Token {
    pub kind: TokenKind,
    pub lexeme: String,
    pub location: SourceLocation,
}

// Abstract Syntax Tree Node
pub struct ASTNode {
    pub kind: ASTKind,
    pub location: SourceLocation,
    pub stack_effect: Option<StackEffect>,
    pub type_signature: Option<TypeSignature>,
}

pub enum ASTKind {
    Definition(Definition),
    Literal(Literal),
    WordCall(WordCall),
    ControlFlow(ControlFlow),
    StackOp(StackOp),
}
```

### 2.2 Type System Architecture

The type system combines **stack effect tracking** with **Hindley-Milner type inference** to provide both safety and flexibility.

```
┌─────────────────────────────────────────────────────────┐
│               TYPE SYSTEM ARCHITECTURE                  │
└─────────────────────────────────────────────────────────┘

Stack Effect Analysis          Type Inference
       │                              │
       ▼                              ▼
┌─────────────┐              ┌──────────────┐
│ Effect Sig. │              │ Type Schema  │
│ ( a b -- c )│◀────────────▶│ ∀α. α α → α │
└─────────────┘              └──────────────┘
       │                              │
       └──────────┬───────────────────┘
                  ▼
         ┌─────────────────┐
         │  Unification    │
         │  Engine         │
         └─────────────────┘
                  │
                  ▼
         ┌─────────────────┐
         │ Concrete Types  │
         │ or Type Error   │
         └─────────────────┘
```

**Type System Components**:

```rust
// Stack effect signature
pub struct StackEffect {
    pub inputs: Vec<StackType>,
    pub outputs: Vec<StackType>,
    pub polymorphic: bool,
}

// Type variables for polymorphism
pub enum StackType {
    Concrete(ConcreteType),
    Variable(TypeVar),
    Constrained(TypeVar, Vec<Constraint>),
}

pub enum ConcreteType {
    Int32,
    Int64,
    Float64,
    Bool,
    Char,
    String,
    Array(Box<ConcreteType>),
    Struct(StructId),
    Pointer(Box<ConcreteType>),
    Effect(StackEffect), // Higher-order words
}

// Type constraints for inference
pub enum Constraint {
    Equal(StackType, StackType),
    Numeric(TypeVar),
    Integral(TypeVar),
    Comparable(TypeVar),
    HasMember(TypeVar, String, StackType),
}

// Polymorphic type scheme (Hindley-Milner)
pub struct TypeScheme {
    pub quantified_vars: Vec<TypeVar>,
    pub constraints: Vec<Constraint>,
    pub stack_effect: StackEffect,
}
```

**Type Inference Algorithm**:

1. **Constraint Generation**: Walk AST, generate constraints from:
   - Stack effect comments: `: DUP ( a -- a a )`
   - Primitive operations: `+` requires numeric types
   - Control flow: IF requires boolean on stack
   - Word calls: Instantiate polymorphic scheme

2. **Unification**: Solve constraints using Robinson's algorithm
   - Substitute concrete types for type variables
   - Detect conflicts → type errors
   - Generalize unconstrained variables

3. **Specialization**: For performance-critical paths
   - Monomorphize polymorphic words
   - Generate specialized versions for concrete types

**Example Type Inference**:

```forth
\ Definition with stack effect comment
: SQUARE ( n -- n² )  DUP * ;

\ Type inference steps:
\ 1. Parse effect: ( α -- β )
\ 2. DUP: ( α -- α α )      → β = ( α α )
\ 3. *:   ( num num -- num ) → α must be numeric
\ 4. Unify: α = β = num
\ 5. Result: ( num -- num )
```

---

## 3. Intermediate Representation (IR)

Fast Forth uses a **three-tier IR** strategy balancing compilation speed and optimization potential:

1. **HIR (High-level IR)**: Close to source, preserves Forth semantics
2. **MIR (Mid-level IR)**: Stack operations explicit, optimization-friendly
3. **LIR (Low-level IR)**: Register-based, maps to LLVM IR or threaded code

```
┌───────────────────────────────────────────────────────────┐
│                  IR TRANSFORMATION PIPELINE                │
└───────────────────────────────────────────────────────────┘

     Forth Source
           │
           ▼
    ┌──────────┐
    │   HIR    │  • Stack implicit in word composition
    │          │  • Control flow as Forth words (IF/ELSE/THEN)
    └──────────┘  • Preserves quotations and deferred words
           │
           │ Lowering (stack → explicit operations)
           ▼
    ┌──────────┐
    │   MIR    │  • Stack operations as SSA values
    │          │  • Control flow graph (CFG)
    └──────────┘  • Optimization passes work here
           │
           │ Backend selection
           ├─────────────┬─────────────┐
           ▼             ▼             ▼
    ┌──────────┐  ┌──────────┐  ┌──────────┐
    │   LIR    │  │ Threaded │  │ LLVM IR  │
    │ Register │  │  Code    │  │          │
    └──────────┘  └──────────┘  └──────────┘
     Native opt.   Fast compile   Full opt.
```

### 3.1 High-Level IR (HIR)

**Purpose**: Preserve Forth semantics, enable high-level optimizations

```rust
pub enum HIRInstruction {
    // Word invocation
    Call { word: WordId, immediate: bool },

    // Literals
    PushLiteral(Literal),

    // Stack manipulation (recognized patterns)
    StackShuffle(ShufflePattern),

    // Control flow (structured)
    If { then_block: BlockId, else_block: Option<BlockId> },
    Loop { body: BlockId, exit_check: ExitCheck },

    // Quotation (higher-order)
    Quotation(Vec<HIRInstruction>),

    // Definition
    Define { name: String, body: Vec<HIRInstruction>, effect: StackEffect },
}

pub enum ShufflePattern {
    Dup,      // ( a -- a a )
    Drop,     // ( a -- )
    Swap,     // ( a b -- b a )
    Over,     // ( a b -- a b a )
    Rot,      // ( a b c -- b c a )
    Custom(Vec<usize>), // Generic permutation
}
```

**Example HIR**:

```forth
: QUADRATIC ( a b c x -- y )
  >R           \ Save x
  SWAP ROT     \ Rearrange a b c → c b a
  R@ * +       \ c*x + b
  R@ * +       \ (c*x + b)*x + a
  R> DROP ;    \ Clean up
```

```rust
// HIR representation
HIRFunction {
    name: "QUADRATIC",
    effect: StackEffect {
        inputs: vec![Var(0), Var(1), Var(2), Var(3)],
        outputs: vec![Var(4)]
    },
    body: vec![
        Call { word: TO_R, immediate: false },
        StackShuffle(Swap),
        StackShuffle(Rot),
        Call { word: R_FETCH, immediate: false },
        Call { word: MULTIPLY, immediate: false },
        Call { word: ADD, immediate: false },
        Call { word: R_FETCH, immediate: false },
        Call { word: MULTIPLY, immediate: false },
        Call { word: ADD, immediate: false },
        Call { word: R_FROM, immediate: false },
        StackShuffle(Drop),
    ]
}
```

### 3.2 Mid-Level IR (MIR)

**Purpose**: Enable optimizations through SSA form and explicit stack operations

```rust
pub struct MIRFunction {
    pub name: String,
    pub params: Vec<ValueId>,
    pub blocks: Vec<BasicBlock>,
    pub stack_effect: StackEffect,
}

pub struct BasicBlock {
    pub id: BlockId,
    pub instructions: Vec<MIRInstruction>,
    pub terminator: Terminator,
}

pub enum MIRInstruction {
    // SSA operations
    BinOp { op: BinOpKind, lhs: ValueId, rhs: ValueId, result: ValueId },
    UnOp { op: UnOpKind, operand: ValueId, result: ValueId },

    // Memory operations
    Load { addr: ValueId, result: ValueId, ty: Type },
    Store { addr: ValueId, value: ValueId },

    // Stack operations (explicit)
    StackPush { value: ValueId },
    StackPop { result: ValueId },
    StackPeek { offset: usize, result: ValueId },

    // Calls
    DirectCall { target: FunctionId, args: Vec<ValueId>, results: Vec<ValueId> },
    IndirectCall { target: ValueId, args: Vec<ValueId>, results: Vec<ValueId> },

    // Literals
    Constant { value: ConstValue, result: ValueId },
}

pub enum Terminator {
    Return { values: Vec<ValueId> },
    Branch { target: BlockId },
    CondBranch { cond: ValueId, then_block: BlockId, else_block: BlockId },
    Unreachable,
}
```

**MIR Example** (from QUADRATIC):

```
function QUADRATIC(v0: i64, v1: i64, v2: i64, v3: i64) -> i64 {
bb0:
    v4 = v3              ; x
    v5 = v2              ; c
    v6 = v1              ; b
    v7 = v0              ; a
    v8 = mul v5, v4      ; c * x
    v9 = add v8, v6      ; c*x + b
    v10 = mul v9, v4     ; (c*x + b) * x
    v11 = add v10, v7    ; (c*x + b)*x + a
    return v11
}
```

### 3.3 Low-Level IR (LIR)

**Purpose**: Target-specific representation ready for code generation

```rust
pub enum LIRInstruction {
    // Register operations
    Move { src: Operand, dst: Register },
    BinOp { op: BinOpKind, src1: Operand, src2: Operand, dst: Register },

    // Memory
    Load { base: Register, offset: i32, dst: Register },
    Store { src: Register, base: Register, offset: i32 },

    // Control flow
    Jump { target: Label },
    CondJump { cond: Register, target: Label },
    Call { target: Symbol },
    Return,

    // Stack management (for non-optimized paths)
    Push { src: Register },
    Pop { dst: Register },
}

pub enum Operand {
    Register(Register),
    Immediate(i64),
    Memory(MemoryOperand),
}
```

---

## 4. Optimization Pipeline

The optimization pipeline operates primarily on MIR, with some optimizations at HIR and LIR levels.

```
┌─────────────────────────────────────────────────────────┐
│           OPTIMIZATION PASS ORDERING                    │
└─────────────────────────────────────────────────────────┘

HIR Optimizations
  │
  ├─▶ Word Inlining (high-level)
  ├─▶ Constant Propagation
  └─▶ Dead Definition Elimination
        │
        ▼
     HIR → MIR
        │
        ▼
MIR Optimizations (MAIN)
  │
  ├─▶ Stack Caching & SSA Conversion     [Core optimization]
  ├─▶ Superinstruction Formation         [Forth-specific]
  ├─▶ Constant Folding                   [Standard]
  ├─▶ Common Subexpression Elimination   [Standard]
  ├─▶ Dead Code Elimination              [Standard]
  ├─▶ Loop Invariant Code Motion         [Standard]
  ├─▶ Function Specialization            [Performance]
  ├─▶ Tail Call Optimization             [Performance]
  └─▶ Plugin Optimizations               [Extensible]
        │
        ▼
     MIR → LIR
        │
        ▼
LIR Optimizations
  │
  ├─▶ Register Allocation
  ├─▶ Instruction Selection
  ├─▶ Peephole Optimization
  └─▶ Instruction Scheduling
        │
        ▼
     Native Code / LLVM IR
```

### 4.1 Stack Caching Optimization

**Problem**: Stack operations are expensive (memory loads/stores)
**Solution**: Keep top N stack elements in registers

```rust
pub struct StackCacheOptimizer {
    cache_depth: usize, // Typically 4-8 elements
    register_allocation: HashMap<usize, Register>,
}

impl Optimizer for StackCacheOptimizer {
    fn optimize(&mut self, mir: &mut MIRFunction) {
        // Analysis: Track stack depth at each program point
        let stack_depths = self.analyze_stack_depths(mir);

        // Transformation: Replace stack ops with register ops
        for block in &mut mir.blocks {
            for inst in &mut block.instructions {
                match inst {
                    MIRInstruction::StackPush { value } => {
                        // Push to cache or spill to memory
                        *inst = self.cache_push(*value);
                    }
                    MIRInstruction::StackPop { result } => {
                        // Pop from cache or load from memory
                        *inst = self.cache_pop(*result);
                    }
                    _ => {}
                }
            }
        }
    }
}
```

**Example**:

```forth
: FOO  1 2 + 3 * ;

\ Before stack caching:
  push_stack #1
  push_stack #2
  pop_stack %tmp1
  pop_stack %tmp2
  add %tmp2, %tmp1 → %tmp3
  push_stack %tmp3
  push_stack #3
  pop_stack %tmp4
  pop_stack %tmp5
  mul %tmp5, %tmp4 → %tmp6

\ After stack caching (cache depth = 4):
  mov #1 → %cache0
  mov #2 → %cache1
  add %cache0, %cache1 → %cache1  ; TOS now holds result
  mov #3 → %cache0
  mul %cache1, %cache0 → %cache0  ; TOS now holds result
```

**Performance Impact**: 70-90% reduction in memory operations for typical Forth code

### 4.2 Superinstruction Formation

**Problem**: Sequences of simple operations have overhead
**Solution**: Combine frequent patterns into specialized instructions

```rust
pub struct SuperinstructionOptimizer {
    pattern_database: PatternDatabase,
}

// Common Forth patterns
const PATTERNS: &[(&str, &[HIRInstruction])] = &[
    ("DUP+", &[StackShuffle(Dup), Call(ADD)]),
    ("OVER+", &[StackShuffle(Over), Call(ADD)]),
    ("2DUP=", &[StackShuffle(TwoDup), Call(EQUALS)]),
    ("@+", &[Call(FETCH), Call(ADD)]),
    ("!0", &[PushLiteral(0), Call(STORE)]),
];

impl Optimizer for SuperinstructionOptimizer {
    fn optimize(&mut self, hir: &mut HIRFunction) {
        // Pattern matching on instruction sequences
        let mut i = 0;
        while i < hir.body.len() {
            if let Some((pattern_name, pattern_len)) =
                self.find_matching_pattern(&hir.body[i..])
            {
                // Replace pattern with superinstruction
                hir.body.splice(
                    i..i+pattern_len,
                    vec![HIRInstruction::Superinstruction(pattern_name)]
                );
            }
            i += 1;
        }
    }
}
```

**Performance Impact**: 10-20% speedup through reduced dispatch overhead

### 4.3 Function Specialization

**Problem**: Polymorphic words have runtime dispatch overhead
**Solution**: Generate specialized versions for common type combinations

```rust
pub struct SpecializationOptimizer {
    specialization_threshold: usize, // Min call count to specialize
    max_specializations: usize,      // Prevent code bloat
}

impl Optimizer for SpecializationOptimizer {
    fn optimize(&mut self, program: &mut Program) {
        // Profiling-guided optimization
        let call_profiles = self.analyze_call_sites(program);

        for (word_id, type_combinations) in call_profiles {
            let word = &program.words[word_id];

            // Skip if not polymorphic
            if !word.is_polymorphic() { continue; }

            // Specialize for hot type combinations
            for (types, call_count) in type_combinations {
                if call_count >= self.specialization_threshold {
                    let specialized = self.specialize_word(word, &types);
                    program.add_specialized_word(specialized);
                }
            }
        }
    }
}
```

**Example**:

```forth
: GENERIC-SORT ( addr len -- )  \ Polymorphic over element type
  ... comparison logic ...
;

\ After specialization:
: SORT-I32 ( addr len -- )  \ Specialized for i32 arrays
  ... optimized i32 comparison ...
;

: SORT-STRING ( addr len -- )  \ Specialized for strings
  ... optimized string comparison ...
;
```

### 4.4 Optimization Pass Configuration

```rust
pub struct OptimizationConfig {
    pub level: OptLevel,
    pub passes: Vec<Box<dyn OptimizationPass>>,
}

pub enum OptLevel {
    None,        // -O0: No optimization, fast compile
    Balanced,    // -O1: Basic opts, reasonable compile time
    Aggressive,  // -O2: All opts, longer compile time
    Maximum,     // -O3: Including profile-guided opts
}

impl OptimizationConfig {
    pub fn from_level(level: OptLevel) -> Self {
        let passes: Vec<Box<dyn OptimizationPass>> = match level {
            OptLevel::None => vec![],

            OptLevel::Balanced => vec![
                Box::new(StackCacheOptimizer::new(4)),
                Box::new(ConstantFoldingPass::new()),
                Box::new(DeadCodeEliminationPass::new()),
            ],

            OptLevel::Aggressive => vec![
                Box::new(InliningPass::new(/* aggressive */ true)),
                Box::new(StackCacheOptimizer::new(8)),
                Box::new(SuperinstructionOptimizer::new()),
                Box::new(ConstantFoldingPass::new()),
                Box::new(CSEPass::new()),
                Box::new(DeadCodeEliminationPass::new()),
                Box::new(LICMPass::new()),
            ],

            OptLevel::Maximum => vec![
                Box::new(InliningPass::new(/* aggressive */ true)),
                Box::new(StackCacheOptimizer::new(8)),
                Box::new(SuperinstructionOptimizer::new()),
                Box::new(SpecializationOptimizer::new()),
                Box::new(ConstantFoldingPass::new()),
                Box::new(CSEPass::new()),
                Box::new(DeadCodeEliminationPass::new()),
                Box::new(LICMPass::new()),
                Box::new(TailCallOptimizer::new()),
                // Plugin passes registered here
            ],
        };

        Self { level, passes }
    }
}
```

---

## 5. Backend Architecture

Fast Forth supports multiple backend strategies for different use cases:

```
┌──────────────────────────────────────────────────────────┐
│                  BACKEND ARCHITECTURE                     │
└──────────────────────────────────────────────────────────┘

                    MIR/LIR
                       │
                       ▼
              ┌────────────────┐
              │ Backend Router │
              └────────────────┘
                       │
        ┌──────────────┼──────────────┐
        ▼              ▼              ▼
┌──────────────┐ ┌──────────┐ ┌──────────────┐
│   Threaded   │ │   LLVM   │ │     JIT      │
│    Code      │ │  Backend │ │   (OrcJIT)   │
└──────────────┘ └──────────┘ └──────────────┘
   Fast compile   Full optim.   Interactive
   ~10ms          ~100ms        Incremental
        │              │              │
        └──────────────┼──────────────┘
                       ▼
                Native Execution
```

### 5.1 Threaded Code Backend

**Purpose**: Ultra-fast compilation for REPL and development

```rust
pub struct ThreadedCodeBackend {
    threading_model: ThreadingModel,
}

pub enum ThreadingModel {
    DirectThreaded,   // Jump table (fastest, non-portable)
    IndirectThreaded, // Function pointers (portable)
    TokenThreaded,    // Dispatch loop (slowest, smallest)
}

impl CodeGenerator for ThreadedCodeBackend {
    fn generate(&self, mir: &MIRFunction) -> CompiledCode {
        match self.threading_model {
            ThreadingModel::DirectThreaded => {
                self.generate_direct_threaded(mir)
            }
            ThreadingModel::IndirectThreaded => {
                self.generate_indirect_threaded(mir)
            }
            ThreadingModel::TokenThreaded => {
                self.generate_token_threaded(mir)
            }
        }
    }
}
```

**Direct Threading Example**:

```c
// Generated C code (for illustration)
void SQUARE() {
    goto *IP++;  // DUP
dup_impl:
    *SP = *(SP-1);
    SP++;
    goto *IP++;  // MULTIPLY
multiply_impl:
    SP--;
    *(SP-1) = *(SP-1) * *SP;
    goto *IP++;
}
```

**Performance**:
- Compile time: <10ms for typical functions
- Runtime: 60-70% of native code performance
- Use case: Interactive development, REPL

### 5.2 LLVM Backend

**Purpose**: Full optimization for production code

```rust
pub struct LLVMBackend<'ctx> {
    context: &'ctx Context,
    module: Module<'ctx>,
    builder: Builder<'ctx>,
    stack_type: StructType<'ctx>,
}

impl<'ctx> CodeGenerator for LLVMBackend<'ctx> {
    fn generate(&self, mir: &MIRFunction) -> CompiledCode {
        // Create LLVM function
        let fn_type = self.mir_to_llvm_type(&mir.stack_effect);
        let function = self.module.add_function(&mir.name, fn_type, None);

        // Generate basic blocks
        for mir_block in &mir.blocks {
            let llvm_block = self.context.append_basic_block(function, "");
            self.builder.position_at_end(llvm_block);

            // Generate instructions
            for inst in &mir_block.instructions {
                self.generate_instruction(inst);
            }

            // Generate terminator
            self.generate_terminator(&mir_block.terminator);
        }

        // Run LLVM optimization passes
        self.run_optimization_pipeline(function);

        // Compile to native code
        self.compile_to_native(function)
    }
}
```

**LLVM Optimization Passes**:

```rust
impl<'ctx> LLVMBackend<'ctx> {
    fn run_optimization_pipeline(&self, function: FunctionValue<'ctx>) {
        let pass_manager = PassManager::create(&self.module);

        // Add standard LLVM passes
        pass_manager.add_instruction_combining_pass();
        pass_manager.add_reassociate_pass();
        pass_manager.add_gvn_pass();
        pass_manager.add_cfg_simplification_pass();
        pass_manager.add_basic_alias_analysis_pass();
        pass_manager.add_promote_memory_to_register_pass();
        pass_manager.add_instruction_combining_pass();
        pass_manager.add_reassociate_pass();
        pass_manager.add_tail_call_elimination_pass();

        pass_manager.run_on(&function);
    }
}
```

**Performance**:
- Compile time: 50-100ms for typical functions
- Runtime: 80-100% of C performance
- Use case: Production builds, performance-critical code

### 5.3 JIT Engine (OrcJIT v2)

**Purpose**: Incremental compilation for interactive development with progressive optimization

```rust
pub struct JITEngine {
    execution_session: ExecutionSession,
    data_layout: DataLayout,
    mangler: Mangler,
    object_layer: RTDyldObjectLinkingLayer,
    compile_layer: IRCompileLayer,
    optimize_layer: IRTransformLayer,

    // Tiering support
    hot_threshold: u64,
    call_counts: HashMap<FunctionId, u64>,
}

impl JITEngine {
    pub fn new() -> Self {
        let target_machine = Target::from_triple(&TargetTriple::create("native"))
            .unwrap()
            .create_target_machine(/* ... */);

        // Set up OrcJIT layers
        let object_layer = RTDyldObjectLinkingLayer::new(
            execution_session.clone(),
            || Box::new(SectionMemoryManager::new())
        );

        let compile_layer = IRCompileLayer::new(
            object_layer,
            SimpleCompiler::new(target_machine)
        );

        let optimize_layer = IRTransformLayer::new(
            compile_layer,
            |module| optimize_module(module)
        );

        Self {
            execution_session,
            /* ... */
            hot_threshold: 1000,
            call_counts: HashMap::new(),
        }
    }

    pub fn compile_function(&mut self, mir: &MIRFunction) -> JITSymbol {
        // Start with threaded code for fast startup
        let initial_code = ThreadedCodeBackend::new()
            .generate(mir);

        // Register for profiling
        self.call_counts.insert(mir.id, 0);

        // Install instrumentation to track calls
        let instrumented = self.add_call_counter(initial_code, mir.id);

        self.execution_session.lookup(instrumented.symbol)
    }

    pub fn maybe_recompile(&mut self, function_id: FunctionId) {
        let call_count = self.call_counts.get(&function_id).unwrap();

        if *call_count >= self.hot_threshold {
            // Function is hot, recompile with LLVM
            let mir = self.get_mir(function_id);
            let optimized = LLVMBackend::new().generate(&mir);

            // Replace old code with optimized version
            self.execution_session.replace(function_id, optimized);
        }
    }
}
```

**Tiering Strategy**:

```
   First Call         After 1000 Calls        After 10000 Calls
       │                     │                        │
       ▼                     ▼                        ▼
┌─────────────┐      ┌─────────────┐        ┌─────────────┐
│  Threaded   │─────▶│  LLVM -O1   │───────▶│  LLVM -O3   │
│   Code      │      │  (Basic)    │        │  + PGO      │
└─────────────┘      └─────────────┘        └─────────────┘
  ~10ms compile       ~50ms compile          ~200ms compile
  60% C perf          85% C perf             95-100% C perf
```

---

## 6. Plugin Architecture

Fast Forth provides a plugin system for custom optimizations and language extensions.

```
┌──────────────────────────────────────────────────────────┐
│                  PLUGIN ARCHITECTURE                      │
└──────────────────────────────────────────────────────────┘

    ┌────────────────────────────────────────┐
    │      Plugin Manager                    │
    │  • Registration                        │
    │  • Lifecycle management                │
    │  • Hook invocation                     │
    └────────────────────────────────────────┘
                    │
        ┌───────────┼───────────┐
        ▼           ▼           ▼
    ┌───────┐  ┌────────┐  ┌─────────┐
    │ HIR   │  │  MIR   │  │  LIR    │
    │Plugin │  │ Plugin │  │ Plugin  │
    └───────┘  └────────┘  └─────────┘
```

### 6.1 Plugin API

```rust
// Plugin trait
pub trait CompilerPlugin: Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> &str;

    // Registration hooks
    fn initialize(&mut self, compiler: &mut Compiler) -> Result<()>;
    fn shutdown(&mut self) -> Result<()>;

    // Compilation hooks (optional)
    fn on_hir_created(&mut self, hir: &mut HIRFunction) -> Result<()> {
        Ok(())
    }

    fn on_mir_created(&mut self, mir: &mut MIRFunction) -> Result<()> {
        Ok(())
    }

    fn on_lir_created(&mut self, lir: &mut LIRFunction) -> Result<()> {
        Ok(())
    }

    // Optimization hooks
    fn register_optimizations(&self) -> Vec<Box<dyn OptimizationPass>>;

    // Custom word definitions
    fn register_words(&self) -> Vec<WordDefinition>;
}

// Plugin manager
pub struct PluginManager {
    plugins: Vec<Box<dyn CompilerPlugin>>,
    optimization_registry: OptimizationRegistry,
}

impl PluginManager {
    pub fn register_plugin(&mut self, plugin: Box<dyn CompilerPlugin>) {
        // Initialize plugin
        plugin.initialize(&mut self.compiler);

        // Register optimizations
        let opts = plugin.register_optimizations();
        for opt in opts {
            self.optimization_registry.register(opt);
        }

        // Register custom words
        let words = plugin.register_words();
        for word in words {
            self.compiler.dictionary.add_word(word);
        }

        self.plugins.push(plugin);
    }

    pub fn invoke_hook<F>(&mut self, hook: F)
    where
        F: Fn(&mut dyn CompilerPlugin) -> Result<()>
    {
        for plugin in &mut self.plugins {
            hook(plugin.as_mut())?;
        }
    }
}
```

### 6.2 Example Plugin: SIMD Optimization

```rust
pub struct SIMDPlugin {
    target_features: TargetFeatures,
}

impl CompilerPlugin for SIMDPlugin {
    fn name(&self) -> &str { "simd-optimizer" }
    fn version(&self) -> &str { "1.0.0" }

    fn initialize(&mut self, compiler: &mut Compiler) -> Result<()> {
        // Detect CPU features
        self.target_features = compiler.target().features();
        Ok(())
    }

    fn register_optimizations(&self) -> Vec<Box<dyn OptimizationPass>> {
        vec![
            Box::new(VectorizeLoopsPass::new()),
            Box::new(SIMDIntrinsicsPass::new()),
        ]
    }

    fn register_words(&self) -> Vec<WordDefinition> {
        vec![
            // Vector operations
            WordDefinition {
                name: "V+".to_string(),
                stack_effect: StackEffect {
                    inputs: vec![Array(Float32), Array(Float32)],
                    outputs: vec![Array(Float32)],
                },
                implementation: WordImpl::Intrinsic(Intrinsic::VectorAdd),
            },
            // ... more SIMD words
        ]
    }
}

// Optimization pass
pub struct VectorizeLoopsPass {
    min_trip_count: usize,
}

impl OptimizationPass for VectorizeLoopsPass {
    fn run(&mut self, mir: &mut MIRFunction) -> Result<bool> {
        let mut changed = false;

        for block in &mut mir.blocks {
            if let Some(loop_info) = self.analyze_loop(block) {
                if loop_info.is_vectorizable() {
                    self.vectorize_loop(block, &loop_info)?;
                    changed = true;
                }
            }
        }

        Ok(changed)
    }
}
```

### 6.3 Plugin Discovery and Loading

```rust
pub struct PluginLoader {
    plugin_dirs: Vec<PathBuf>,
}

impl PluginLoader {
    pub fn discover_plugins(&self) -> Vec<PluginMetadata> {
        let mut plugins = Vec::new();

        for dir in &self.plugin_dirs {
            for entry in fs::read_dir(dir)? {
                let path = entry?.path();

                // Load plugin manifest
                if path.extension() == Some("toml") {
                    let metadata = self.load_metadata(&path)?;
                    plugins.push(metadata);
                }
            }
        }

        plugins
    }

    pub fn load_plugin(&self, metadata: &PluginMetadata) -> Result<Box<dyn CompilerPlugin>> {
        // Dynamic library loading
        unsafe {
            let lib = Library::new(&metadata.library_path)?;
            let constructor: Symbol<PluginConstructor> =
                lib.get(b"create_plugin\0")?;

            let plugin = constructor();
            Ok(plugin)
        }
    }
}

// Plugin manifest (plugin.toml)
/*
[plugin]
name = "simd-optimizer"
version = "1.0.0"
author = "Fast Forth Team"
library = "libsimd_plugin.so"

[dependencies]
fast-forth = "^1.0"

[configuration]
vectorize-threshold = 4
*/
```

---

## 7. Compilation Pipeline

### 7.1 AOT Compilation

```rust
pub struct AOTCompiler {
    frontend: Frontend,
    type_checker: TypeChecker,
    ir_builder: IRBuilder,
    optimizer: OptimizerPipeline,
    backend: Box<dyn CodeGenerator>,
    plugin_manager: PluginManager,
}

impl AOTCompiler {
    pub fn compile_file(&mut self, path: &Path) -> Result<CompiledModule> {
        // 1. Frontend: Parse source
        let source = fs::read_to_string(path)?;
        let ast = self.frontend.parse(&source)?;

        // 2. Type checking and inference
        let typed_ast = self.type_checker.check(ast)?;

        // 3. IR building
        let hir = self.ir_builder.build_hir(typed_ast)?;
        self.plugin_manager.invoke_hook(|p| p.on_hir_created(&mut hir))?;

        let mir = self.ir_builder.lower_to_mir(hir)?;
        self.plugin_manager.invoke_hook(|p| p.on_mir_created(&mut mir))?;

        // 4. Optimization
        let optimized_mir = self.optimizer.optimize(mir)?;

        // 5. Backend code generation
        let lir = self.ir_builder.lower_to_lir(optimized_mir)?;
        self.plugin_manager.invoke_hook(|p| p.on_lir_created(&mut lir))?;

        let compiled = self.backend.generate(lir)?;

        Ok(CompiledModule {
            name: path.file_stem().unwrap().to_string_lossy().into(),
            code: compiled,
            exports: self.extract_exports(&lir),
        })
    }

    pub fn link_modules(&self, modules: Vec<CompiledModule>) -> Result<Executable> {
        // Link compiled modules into executable
        self.backend.link(modules)
    }
}
```

### 7.2 Incremental Compilation

```rust
pub struct IncrementalCompiler {
    cache: CompilationCache,
    dependency_graph: DependencyGraph,
}

impl IncrementalCompiler {
    pub fn compile_incremental(&mut self, changes: Vec<Change>) -> Result<()> {
        // 1. Determine what needs recompilation
        let affected = self.dependency_graph.compute_affected_nodes(&changes);

        // 2. Invalidate cache entries
        for node in &affected {
            self.cache.invalidate(node);
        }

        // 3. Recompile affected modules
        for node in affected {
            if !self.cache.contains(&node) {
                let compiled = self.compile_module(&node)?;
                self.cache.insert(node, compiled);
            }
        }

        Ok(())
    }
}

pub struct CompilationCache {
    hir_cache: HashMap<ModuleId, HIRModule>,
    mir_cache: HashMap<FunctionId, MIRFunction>,
    object_cache: HashMap<FunctionId, CompiledObject>,
}
```

### 7.3 REPL Integration

```rust
pub struct REPLCompiler {
    jit: JITEngine,
    dictionary: Dictionary,
    incremental: IncrementalCompiler,
}

impl REPLCompiler {
    pub fn eval(&mut self, input: &str) -> Result<Value> {
        // Parse and type check
        let ast = self.parse_line(input)?;
        let typed_ast = self.type_check(ast)?;

        match typed_ast {
            ASTNode::Definition(def) => {
                // Compile definition
                let mir = self.build_mir(def)?;
                let symbol = self.jit.compile_function(&mir)?;

                // Add to dictionary
                self.dictionary.define(def.name, symbol);

                Ok(Value::Unit)
            }

            ASTNode::Expression(expr) => {
                // Compile and execute expression
                let mir = self.build_mir_expression(expr)?;
                let symbol = self.jit.compile_function(&mir)?;

                // Execute and return result
                let result = unsafe { self.jit.execute(symbol)? };
                Ok(result)
            }
        }
    }
}
```

---

## 8. Performance Targets and Benchmarking

### 8.1 Compilation Performance

```
Target: <100ms compile time for typical programs

Breakdown:
  Frontend (parsing):           5-10ms
  Type inference:               10-20ms
  HIR → MIR lowering:          5-10ms
  Optimization passes:          20-40ms
  LLVM backend:                 30-50ms
  ─────────────────────────────────
  Total:                        70-130ms
```

### 8.2 Runtime Performance

```
Target: 80-100% of equivalent C code

Benchmark Suite:
  - Fibonacci (recursion)
  - Sieve of Eratosthenes (loops, arrays)
  - JSON parser (strings, branching)
  - Matrix multiplication (numerical)
  - Hash table operations (memory)

Expected Results:
                      C      Fast Forth    Ratio
  Fibonacci:         10ms    11ms          90%
  Sieve:            150ms   165ms          91%
  JSON:             200ms   220ms          91%
  MatMul:           300ms   270ms         111% (SIMD)
  HashMap:          180ms   200ms          90%
  ──────────────────────────────────────────────
  Geomean:                                 93%
```

### 8.3 Memory Usage

```
Target: Minimal overhead over C

  Stack cache:           64 bytes (8 registers × 8 bytes)
  Dictionary overhead:   ~100 bytes per word definition
  Type metadata:         ~50 bytes per polymorphic word
  JIT code cache:        Configurable (default: 64MB)
```

---

## 9. Data Structures Reference

### 9.1 Core Structures

```rust
// Complete type system
pub struct TypeContext {
    pub type_vars: HashMap<TypeVarId, Type>,
    pub constraints: Vec<Constraint>,
    pub substitutions: Substitution,
}

// Dictionary (global symbol table)
pub struct Dictionary {
    pub words: HashMap<String, WordEntry>,
    pub immediate_words: HashSet<String>,
}

pub struct WordEntry {
    pub id: WordId,
    pub name: String,
    pub stack_effect: StackEffect,
    pub type_scheme: TypeScheme,
    pub implementation: WordImplementation,
    pub metadata: WordMetadata,
}

pub enum WordImplementation {
    Compiled(CompiledCode),
    Primitive(PrimitiveFn),
    Deferred(Option<WordId>),
    Quotation(Vec<HIRInstruction>),
}

// Compilation cache for incremental compilation
pub struct CompilationArtifacts {
    pub source_hash: u64,
    pub hir: HIRModule,
    pub mir: MIRModule,
    pub type_info: TypeInfo,
    pub dependencies: Vec<ModuleId>,
    pub timestamp: SystemTime,
}

// Dependency tracking
pub struct DependencyGraph {
    pub nodes: HashMap<ModuleId, DependencyNode>,
    pub edges: HashMap<ModuleId, Vec<ModuleId>>,
}

pub struct DependencyNode {
    pub module_id: ModuleId,
    pub definitions: Vec<WordId>,
    pub imports: Vec<ModuleId>,
}
```

### 9.2 Optimization Data Structures

```rust
// SSA value numbering
pub struct ValueNumbering {
    pub values: HashMap<ValueId, Value>,
    pub next_id: ValueId,
}

// Dominator tree for optimization
pub struct DominatorTree {
    pub immediate_dominators: HashMap<BlockId, BlockId>,
    pub dominance_frontier: HashMap<BlockId, HashSet<BlockId>>,
}

// Loop analysis
pub struct LoopInfo {
    pub header: BlockId,
    pub backedges: Vec<BlockId>,
    pub body: HashSet<BlockId>,
    pub trip_count: Option<usize>,
}

// Call graph for interprocedural optimization
pub struct CallGraph {
    pub nodes: HashMap<FunctionId, CallGraphNode>,
    pub edges: HashMap<FunctionId, Vec<FunctionId>>,
}

pub struct CallGraphNode {
    pub function_id: FunctionId,
    pub call_sites: Vec<CallSite>,
    pub is_recursive: bool,
}
```

---

## 10. System Configuration

### 10.1 Compiler Configuration

```rust
pub struct CompilerConfig {
    // Optimization
    pub opt_level: OptLevel,
    pub enable_plugins: bool,
    pub plugin_dirs: Vec<PathBuf>,

    // Backend selection
    pub backend: BackendKind,
    pub threading_model: ThreadingModel,

    // Type system
    pub strict_stack_effects: bool,
    pub allow_implicit_conversions: bool,

    // JIT
    pub jit_hot_threshold: u64,
    pub jit_cache_size: usize,

    // Debugging
    pub emit_debug_info: bool,
    pub dump_ir: bool,
    pub verbose: bool,
}

impl Default for CompilerConfig {
    fn default() -> Self {
        Self {
            opt_level: OptLevel::Balanced,
            enable_plugins: true,
            plugin_dirs: vec![PathBuf::from("./plugins")],
            backend: BackendKind::LLVM,
            threading_model: ThreadingModel::DirectThreaded,
            strict_stack_effects: true,
            allow_implicit_conversions: false,
            jit_hot_threshold: 1000,
            jit_cache_size: 64 * 1024 * 1024, // 64MB
            emit_debug_info: false,
            dump_ir: false,
            verbose: false,
        }
    }
}
```

### 10.2 Runtime Configuration

```rust
pub struct RuntimeConfig {
    // Stack sizes
    pub data_stack_size: usize,
    pub return_stack_size: usize,

    // Memory management
    pub heap_size: usize,
    pub gc_threshold: usize,

    // Performance
    pub stack_cache_depth: usize,
    pub enable_profiling: bool,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            data_stack_size: 64 * 1024,      // 64KB
            return_stack_size: 32 * 1024,    // 32KB
            heap_size: 16 * 1024 * 1024,     // 16MB
            gc_threshold: 8 * 1024 * 1024,   // 8MB
            stack_cache_depth: 8,
            enable_profiling: false,
        }
    }
}
```

---

## 11. Error Handling Architecture

```rust
pub enum CompilerError {
    // Frontend errors
    LexError { message: String, location: SourceLocation },
    ParseError { message: String, location: SourceLocation },

    // Type errors
    TypeMismatch { expected: Type, found: Type, location: SourceLocation },
    StackEffectMismatch { expected: StackEffect, found: StackEffect, location: SourceLocation },
    UnresolvedTypeVariable { var: TypeVar, location: SourceLocation },

    // Semantic errors
    UndefinedWord { name: String, location: SourceLocation },
    RedefinedWord { name: String, original: SourceLocation, new: SourceLocation },
    StackUnderflow { location: SourceLocation },

    // Optimization errors
    OptimizationFailed { pass: String, reason: String },

    // Backend errors
    CodeGenerationFailed { reason: String },
    LinkingFailed { reason: String },
}

impl CompilerError {
    pub fn report(&self) -> String {
        match self {
            Self::TypeMismatch { expected, found, location } => {
                format!(
                    "Type error at {}:{}\n  Expected: {}\n  Found: {}",
                    location.file, location.line, expected, found
                )
            }
            // ... other error formatting
        }
    }
}
```

---

## 12. Testing Strategy

### 12.1 Test Architecture

```
tests/
├── unit/
│   ├── frontend/
│   │   ├── lexer_tests.rs
│   │   └── parser_tests.rs
│   ├── type_system/
│   │   ├── inference_tests.rs
│   │   └── stack_effect_tests.rs
│   ├── optimizer/
│   │   ├── stack_cache_tests.rs
│   │   └── inlining_tests.rs
│   └── backend/
│       ├── llvm_tests.rs
│       └── threaded_tests.rs
├── integration/
│   ├── compile_tests.rs
│   ├── execution_tests.rs
│   └── incremental_tests.rs
├── benchmarks/
│   ├── compilation_bench.rs
│   └── runtime_bench.rs
└── fixtures/
    └── forth_programs/
```

### 12.2 Benchmark Suite

```rust
#[bench]
fn bench_compilation_fibonacci(b: &mut Bencher) {
    let source = r#"
        : FIB ( n -- fib[n] )
          DUP 2 < IF DROP 1 EXIT THEN
          DUP 1- RECURSE
          SWAP 2- RECURSE + ;
    "#;

    b.iter(|| {
        let mut compiler = AOTCompiler::new();
        compiler.compile(source)
    });
}

#[bench]
fn bench_runtime_sieve(b: &mut Bencher) {
    let compiled = compile_sieve();

    b.iter(|| {
        unsafe { compiled.execute(1000) }
    });
}
```

---

## 13. Documentation and Examples

### 13.1 Example: Complete Compilation Flow

```forth
\ fibonacci.fth
: FIB ( n -- fib[n] )
  DUP 2 < IF
    DROP 1
  ELSE
    DUP 1- RECURSE
    SWAP 2- RECURSE +
  THEN ;
```

**Step 1: Frontend (AST)**
```
Definition {
  name: "FIB",
  stack_effect: ( n -- fib[n] ),
  body: [
    StackOp(Dup),
    Literal(2),
    StackOp(Lt),
    If {
      then: [StackOp(Drop), Literal(1)],
      else: [
        StackOp(Dup), Literal(1), BinOp(Sub), WordCall(FIB),
        StackOp(Swap), Literal(2), BinOp(Sub), WordCall(FIB),
        BinOp(Add)
      ]
    }
  ]
}
```

**Step 2: Type Inference**
```
Constraints:
  - DUP:  α → α α
  - 2:    Int
  - <:    Int Int → Bool
  - IF:   Bool → ...

Unification:
  α = Int

Result: ( Int -- Int )
```

**Step 3: MIR**
```
function FIB(v0: i64) -> i64 {
bb0:
    v1 = v0
    v2 = const 2
    v3 = lt v1, v2
    cond_branch v3, bb1, bb2

bb1:  // then
    v4 = const 1
    return v4

bb2:  // else
    v5 = v0
    v6 = const 1
    v7 = sub v5, v6
    v8 = call FIB(v7)
    v9 = v0
    v10 = const 2
    v11 = sub v9, v10
    v12 = call FIB(v11)
    v13 = add v8, v12
    return v13
}
```

**Step 4: Optimization (Tail Call)**
```
function FIB(v0: i64) -> i64 {
bb0:
    v1 = const 2
    v2 = lt v0, v1
    cond_branch v2, bb1, bb2

bb1:  // base case
    v3 = const 1
    return v3

bb2:  // recursive (tail call optimized)
    v4 = const 1
    v5 = sub v0, v4
    v6 = call FIB(v5)  // Can't tail call (need result)
    v7 = const 2
    v8 = sub v0, v7
    v9 = tail_call FIB(v8)  // Tail position
    v10 = add v6, v9
    return v10
}
```

**Step 5: LLVM IR**
```llvm
define i64 @FIB(i64 %n) {
entry:
  %cmp = icmp slt i64 %n, 2
  br i1 %cmp, label %base, label %recurse

base:
  ret i64 1

recurse:
  %n_1 = sub i64 %n, 1
  %fib_n_1 = call i64 @FIB(i64 %n_1)
  %n_2 = sub i64 %n, 2
  %fib_n_2 = musttail call i64 @FIB(i64 %n_2)
  %result = add i64 %fib_n_1, %fib_n_2
  ret i64 %result
}
```

---

## 14. Future Extensions

### 14.1 Planned Features

1. **WASM Backend**: Compile to WebAssembly for browser deployment
2. **GPU Acceleration**: CUDA/OpenCL backend for parallel operations
3. **Distributed Runtime**: Multi-node execution for large-scale data processing
4. **Advanced Type System**: Dependent types, refinement types
5. **Verification**: Formal verification of stack effects and type safety
6. **IDE Integration**: LSP server for modern editor support

### 14.2 Research Directions

1. **Profile-Guided Optimization**: Use runtime profiles to guide specialization
2. **Adaptive Compilation**: Dynamic recompilation based on actual usage patterns
3. **Partial Evaluation**: Compile-time execution of pure computations
4. **Effect System**: Track side effects in type system

---

## 15. Conclusion

This architecture provides a solid foundation for Fast Forth to achieve:

- **Performance**: 80-100% of C through LLVM and stack caching
- **Interactivity**: <100ms compilation for rapid development
- **Safety**: Static type inference prevents stack errors
- **Extensibility**: Plugin system for custom optimizations
- **Flexibility**: Multiple backends for different use cases

The design balances modern compiler techniques (SSA, Hindley-Milner, LLVM) with Forth's unique stack-based semantics, creating a system that is both powerful and true to Forth's interactive, incremental development philosophy.

---

**Next Steps for Development Streams**:

1. **Stream 2 (Frontend)**: Implement lexer, parser, AST based on Section 2.1
2. **Stream 3 (Type System)**: Implement inference engine based on Section 2.2
3. **Stream 4 (IR)**: Implement HIR/MIR/LIR based on Section 3
4. **Stream 5 (Optimizer)**: Implement optimization passes based on Section 4
5. **Stream 6 (Backend)**: Implement LLVM backend based on Section 5
6. **Stream 7 (Runtime)**: Implement JIT and runtime support based on Section 5.3
7. **Stream 8 (Testing)**: Implement test suite based on Section 12

---

**Document Version**: 1.0
**Last Updated**: 2025-11-14
**Maintained By**: Architect Agent (STREAM 1)
