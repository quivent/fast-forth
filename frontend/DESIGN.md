# Fast Forth Frontend Design Document

## Overview

This document describes the design and implementation of the Fast Forth frontend compiler. The frontend is responsible for transforming ANS Forth source code into optimized SSA form suitable for LLVM code generation.

## Goals

1. **Performance**: Parse typical Forth programs in <50ms
2. **Correctness**: Detect all semantic errors at compile time
3. **Optimization**: Produce SSA form that enables aggressive optimization
4. **Maintainability**: Clean, modular architecture
5. **Testing**: Comprehensive test coverage

## Architecture

### Compilation Pipeline

```
┌─────────────┐
│ Source Code │
└──────┬──────┘
       │
       ├─────────────────────────────┐
       │                             │
       v                             v
┌──────────┐                  ┌─────────┐
│  Lexer   │                  │ Lexer   │
└────┬─────┘                  └────┬────┘
     │                             │
     v                             v
┌──────────┐                  ┌─────────┐
│  Tokens  │                  │  Tokens │
└────┬─────┘                  └────┬────┘
     │                             │
     v                             v
┌──────────┐                  ┌─────────┐
│  Parser  │◄─────────────────┤  Cache  │
└────┬─────┘                  └─────────┘
     │
     v
┌──────────┐
│   AST    │
└────┬─────┘
     │
     ├──────────┬──────────┬──────────┐
     │          │          │          │
     v          v          v          v
┌─────────┐ ┌──────┐ ┌─────────┐ ┌────────┐
│ Stack   │ │ Type │ │Semantic │ │  SSA   │
│ Effects │ │Infer │ │ Analysis│ │Convert │
└─────────┘ └──────┘ └─────────┘ └────────┘
     │          │          │          │
     └──────────┴──────────┴──────────┘
                    │
                    v
              ┌──────────┐
              │ SSA IR   │
              └──────────┘
```

### Module Breakdown

#### 1. Lexer (`lexer.rs`)

**Purpose**: Transform source text into tokens

**Design Decisions:**
- Character-by-character processing for fine control
- Lookahead for number parsing (int vs float)
- Comment skipping at lexer level
- Case-insensitive keyword recognition

**Performance Optimizations:**
- Single pass through source
- Minimal allocations
- Direct string slicing where possible

**Token Types:**
```rust
pub enum Token {
    Colon, Semicolon,           // Word definition
    Word(String),               // Identifier
    Integer(i64), Float(f64),   // Literals
    String(String),             // String literal
    If, Then, Else,             // Control flow
    Begin, Until, While, Repeat,// Loops
    Do, Loop, PlusLoop,         // Counted loops
    Variable, Constant,         // Definitions
    LeftParen, RightParen,      // Comments/effects
    StackEffectSep,             // --
    Immediate,                  // Compilation mode
    Eof,                        // End of file
}
```

#### 2. Parser (`parser.rs`)

**Purpose**: Build Abstract Syntax Tree from tokens

**Design Decisions:**
- Recursive descent for clarity
- Early error detection
- Stack effect parsing integrated
- Immediate word tracking

**Grammar (simplified):**
```
program     := (definition | top-level)*
definition  := ':' WORD stack-effect? word* ';' 'IMMEDIATE'?
stack-effect := '(' type* '--' type* ')'
top-level   := VARIABLE WORD | INTEGER WORD CONSTANT | word
word        := literal | word-ref | if | begin | do | variable | constant
if          := 'IF' word* ('ELSE' word*)? 'THEN'
begin       := 'BEGIN' word* ('UNTIL' | 'WHILE' word* 'REPEAT')
do          := 'DO' word* ('LOOP' | '+LOOP')
```

**AST Structure:**
```rust
pub struct Program {
    pub definitions: Vec<Definition>,
    pub top_level_code: Vec<Word>,
}

pub struct Definition {
    pub name: String,
    pub body: Vec<Word>,
    pub immediate: bool,
    pub stack_effect: Option<StackEffect>,
    pub location: SourceLocation,
}

pub enum Word {
    IntLiteral(i64),
    FloatLiteral(f64),
    StringLiteral(String),
    WordRef { name: String, location: SourceLocation },
    If { then_branch: Vec<Word>, else_branch: Option<Vec<Word>> },
    BeginUntil { body: Vec<Word> },
    BeginWhileRepeat { condition: Vec<Word>, body: Vec<Word> },
    DoLoop { body: Vec<Word>, increment: i64 },
    Variable { name: String },
    Constant { name: String, value: i64 },
    Comment(String),
}
```

#### 3. Stack Effect Inference (`stack_effects.rs`)

**Purpose**: Infer and validate stack effects

**Algorithm:**

```
function infer_sequence(words):
    stack_depth = 0
    inputs_needed = 0

    for word in words:
        effect = get_word_effect(word)

        # Check if stack has enough items
        if stack_depth < effect.inputs.len():
            inputs_needed += effect.inputs.len() - stack_depth
            stack_depth = 0
        else:
            stack_depth -= effect.inputs.len()

        # Add outputs
        stack_depth += effect.outputs.len()

    outputs_produced = stack_depth
    return StackEffect(inputs_needed, outputs_produced)
```

**Builtin Effects:**
```forth
+       ( int int -- int )
dup     ( a -- a a )
swap    ( a b -- b a )
drop    ( a -- )
over    ( a b -- a b a )
rot     ( a b c -- b c a )
```

**Validation:**
- Compare inferred vs declared effects
- Check stack underflow
- Verify control flow consistency

#### 4. Type Inference (`type_inference.rs`)

**Purpose**: Infer concrete types for stack values

**Algorithm**: Hindley-Milner with modifications for stack semantics

**Type System:**
```rust
pub enum StackType {
    Int,          // Cell-sized integer
    Float,        // Floating point
    Addr,         // Memory address
    Bool,         // Boolean flag
    Char,         // Character
    String,       // String reference
    Var(TypeVar), // Type variable
    Unknown,      // To be inferred
}
```

**Unification Rules:**
```
unify(Int, Int) = Ok
unify(Float, Float) = Ok
unify(Unknown, T) = Ok
unify(T, Unknown) = Ok
unify(Var(x), T) = bind(x, T) if !occurs(x, T)
unify(T1, T2) = Error(TypeError)
```

**Constraint Generation:**
```
For each word:
    - Generate constraints from builtin type
    - Propagate through stack operations
    - Unify with usage sites
```

**Example:**
```forth
: add-one 1 + ;

Generated constraints:
    %0 : int          (from literal 1)
    %1 : int          (from +)
    param : int       (unified with +)
    result : int      (from +)
```

#### 5. SSA Conversion (`ssa.rs`)

**Purpose**: Convert stack operations to SSA form

**Key Concepts:**

**SSA Properties:**
- Each variable assigned exactly once
- Phi nodes at control flow merges
- Explicit data dependencies
- Basic block structure

**Stack to SSA Mapping:**
```forth
Stack:    1 2 +
          │ │ │
          │ │ └─→ operation
          │ └───→ operand 2
          └─────→ operand 1

SSA:      %0 = load 1
          %1 = load 2
          %2 = add %0, %1
```

**Control Flow:**
```forth
IF:
          condition
             │
          branch
          ┌─────┐─────┐
          │           │
       then_bb     else_bb
          │           │
          └─────┬─────┘
             merge_bb
          (phi nodes)
```

**SSA Instructions:**
```rust
pub enum SSAInstruction {
    LoadInt { dest: Register, value: i64 },
    LoadFloat { dest: Register, value: f64 },
    BinaryOp { dest: Register, op: BinaryOp, left: Register, right: Register },
    UnaryOp { dest: Register, op: UnaryOp, operand: Register },
    Call { dest: Vec<Register>, name: String, args: Vec<Register> },
    Branch { condition: Register, true_block: BlockId, false_block: BlockId },
    Jump { target: BlockId },
    Return { values: Vec<Register> },
    Phi { dest: Register, incoming: Vec<(BlockId, Register)> },
    Load { dest: Register, address: Register, ty: StackType },
    Store { address: Register, value: Register, ty: StackType },
}
```

**Conversion Algorithm:**
```
function convert_word(word, stack):
    match word:
        IntLiteral(n):
            reg = fresh_register()
            emit(LoadInt(reg, n))
            stack.push(reg)

        WordRef("+"):
            right = stack.pop()
            left = stack.pop()
            dest = fresh_register()
            emit(BinaryOp(dest, Add, left, right))
            stack.push(dest)

        If(then, else):
            cond = stack.pop()
            then_bb = fresh_block()
            else_bb = fresh_block()
            merge_bb = fresh_block()

            emit(Branch(cond, then_bb, else_bb))

            set_block(then_bb)
            convert_sequence(then, stack)
            emit(Jump(merge_bb))

            set_block(else_bb)
            convert_sequence(else, stack)
            emit(Jump(merge_bb))

            set_block(merge_bb)
```

#### 6. Semantic Analysis (`semantic.rs`)

**Purpose**: Validate program semantics

**Checks Performed:**

1. **Undefined Words**
   - Check all word references
   - Build symbol table during first pass
   - Report location of undefined use

2. **Stack Underflow**
   - Track minimum stack depth
   - Detect compile-time underflow
   - Consider all control paths

3. **Control Structure Balance**
   - Track control structure nesting
   - Validate proper termination
   - Check branch compatibility

4. **Redefinition**
   - Track defined words
   - Warn on builtin redefinition
   - Error on duplicate definitions

5. **Type Consistency**
   - Check declared vs inferred types
   - Validate operator type requirements
   - Ensure branch type agreement

**Analysis Algorithm:**
```
function analyze(program):
    # First pass: collect definitions
    for def in program.definitions:
        check_redefinition(def.name)
        add_to_symbol_table(def.name)
        infer_stack_effect(def)

    # Second pass: validate definitions
    for def in program.definitions:
        validate_body(def.body)
        check_stack_effect(def)
        check_control_structures(def.body)

    # Validate top-level code
    validate_sequence(program.top_level_code)
```

## Performance Optimization Strategies

### 1. Parsing Performance

**Target: <50ms for typical programs**

Strategies:
- Single-pass lexing
- Minimal allocations
- String interning for identifiers
- Preallocated token buffer
- Direct string slicing

### 2. Analysis Performance

**Target: <100ms total pipeline**

Strategies:
- Incremental analysis
- Cache stack effects
- Reuse type inference results
- Parallel analysis of independent definitions

### 3. Memory Usage

Strategies:
- SmallVec for common cases
- FxHashMap (fast non-cryptographic hash)
- String interning for identifiers
- Compact AST representation

## Error Reporting

### Error Types

```rust
pub enum ForthError {
    ParseError { line: usize, column: usize, message: String },
    LexError { position: usize, message: String },
    UndefinedWord { word: String, line: Option<usize> },
    StackUnderflow { word: String, expected: usize, found: usize },
    StackOverflow { max: usize },
    TypeError { expected: String, found: String, location: Option<String> },
    InvalidStackEffect { declaration: String },
    RedefinitionError { word: String },
    ControlStructureMismatch { expected: String, found: String },
    InvalidImmediateWord { word: String },
    SSAConversionError { message: String },
    InternalError { message: String },
}
```

### Error Recovery

- Continue analysis after errors when possible
- Collect multiple errors in single pass
- Provide context for error messages
- Suggest fixes where appropriate

## Testing Strategy

### Unit Tests

Each module has comprehensive unit tests:
- Lexer: token recognition, edge cases
- Parser: grammar rules, error cases
- Stack inference: builtin effects, user definitions
- Type inference: unification, polymorphism
- SSA: conversion correctness, control flow
- Semantic: all validation rules

### Integration Tests

Full pipeline tests:
- Complete programs
- Error detection
- Performance benchmarks
- Edge cases

### Property-Based Tests

Using proptest:
- Generate random valid programs
- Verify invariants hold
- Test parser roundtripping

### Benchmarks

Using criterion:
- Lexer performance
- Parser performance
- Analysis performance
- SSA conversion performance
- Full pipeline performance

## Future Improvements

### Short Term
- [ ] Complete return stack operations
- [ ] Memory allocation words
- [ ] String printing
- [ ] More comprehensive builtins

### Medium Term
- [ ] Incremental compilation
- [ ] Better error messages
- [ ] Source maps for debugging
- [ ] Optimization passes on SSA

### Long Term
- [ ] Whole-program optimization
- [ ] IDE support (LSP)
- [ ] Parallel compilation
- [ ] JIT compilation mode

## Conclusion

The Fast Forth frontend provides a solid foundation for a modern, fast Forth compiler. The modular architecture allows for easy extension and optimization, while comprehensive testing ensures correctness.
