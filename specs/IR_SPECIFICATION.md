# Fast Forth IR Specification
**Version**: 1.0
**Date**: 2025-11-14

## Overview

This document provides the complete specification for Fast Forth's three-tier IR system.

---

## 1. High-Level IR (HIR)

### 1.1 Design Philosophy

HIR preserves Forth's concatenative semantics while enabling high-level optimizations. It represents code as a sequence of word calls with implicit stack operations.

### 1.2 Complete HIR Instruction Set

```rust
/// High-level IR instruction
#[derive(Debug, Clone, PartialEq)]
pub enum HIRInstruction {
    /// Word call
    Call {
        word_id: WordId,
        immediate: bool,
        inferred_type: Option<TypeSignature>,
    },

    /// Literal value
    Literal(Literal),

    /// Stack manipulation patterns
    StackOp(StackOp),

    /// Control flow
    If {
        condition_value: Option<ValueId>, // None = implicit TOS
        then_block: BlockId,
        else_block: Option<BlockId>,
        stack_effect: StackEffect,
    },

    /// Loops
    BeginUntil {
        body: BlockId,
        exit_on_true: bool,
        stack_effect: StackEffect,
    },

    BeginWhileRepeat {
        condition: BlockId,
        body: BlockId,
        stack_effect: StackEffect,
    },

    DoLoop {
        body: BlockId,
        counter_var: ValueId,
        limit_var: ValueId,
        step: i64, // 1 for DO...LOOP, -1 for DO...-LOOP
        stack_effect: StackEffect,
    },

    /// Case/switch
    Case {
        scrutinee: Option<ValueId>, // None = implicit TOS
        branches: Vec<(Literal, BlockId)>,
        default: Option<BlockId>,
    },

    /// Early return
    Exit,

    /// Quotation (higher-order)
    Quotation {
        body: Vec<HIRInstruction>,
        captured: Vec<ValueId>,
        stack_effect: StackEffect,
    },

    /// Memory operations
    Fetch { addr_type: Option<Type> },
    Store { addr_type: Option<Type> },

    /// Return stack operations
    ToR,
    FromR,
    RFetch,

    /// Meta operations
    Comment(String),
    SourceLocation(SourceLocation),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Int32(i32),
    Int64(i64),
    Float32(f32),
    Float64(f64),
    Char(char),
    String(String),
    Bool(bool),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StackOp {
    Dup,      // ( a -- a a )
    Drop,     // ( a -- )
    Swap,     // ( a b -- b a )
    Over,     // ( a b -- a b a )
    Rot,      // ( a b c -- b c a )
    NegRot,   // ( a b c -- c a b ) aka -ROT
    Nip,      // ( a b -- b )
    Tuck,     // ( a b -- b a b )
    Pick(u8), // ( ... n -- ... an ) where n is depth
    Roll(u8), // ( ... n -- ... ) rotate n items
    TwoDup,   // ( a b -- a b a b )
    TwoDrop,  // ( a b -- )
    TwoSwap,  // ( a b c d -- c d a b )
    TwoOver,  // ( a b c d -- a b c d a b )
}

/// HIR function representation
#[derive(Debug, Clone)]
pub struct HIRFunction {
    pub id: FunctionId,
    pub name: String,
    pub stack_effect: StackEffect,
    pub type_signature: TypeSignature,
    pub body: Vec<HIRInstruction>,
    pub immediacy: Immediacy,
    pub visibility: Visibility,
    pub source_location: SourceLocation,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Immediacy {
    Normal,
    Immediate, // Executed at compile time
    CompileOnly, // Can only be used in definitions
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Visibility {
    Public,
    Private,
    Module,
}
```

### 1.3 HIR Examples

**Example 1: Simple arithmetic**
```forth
: QUADRATIC ( a b c x -- result )
  >R           \ x to return stack
  SWAP ROT     \ rearrange coefficients
  R@ * +       \ c*x + b
  R@ * +       \ result*x + a
  R> DROP ;
```

```rust
HIRFunction {
    name: "QUADRATIC",
    stack_effect: StackEffect {
        inputs: vec![Int, Int, Int, Int],
        outputs: vec![Int],
    },
    body: vec![
        HIRInstruction::ToR,
        HIRInstruction::StackOp(StackOp::Swap),
        HIRInstruction::StackOp(StackOp::Rot),
        HIRInstruction::RFetch,
        HIRInstruction::Call { word_id: MULTIPLY, immediate: false, inferred_type: None },
        HIRInstruction::Call { word_id: ADD, immediate: false, inferred_type: None },
        HIRInstruction::RFetch,
        HIRInstruction::Call { word_id: MULTIPLY, immediate: false, inferred_type: None },
        HIRInstruction::Call { word_id: ADD, immediate: false, inferred_type: None },
        HIRInstruction::FromR,
        HIRInstruction::StackOp(StackOp::Drop),
    ],
    immediacy: Immediacy::Normal,
    visibility: Visibility::Public,
    source_location: SourceLocation { /* ... */ },
}
```

**Example 2: Control flow**
```forth
: ABS ( n -- |n| )
  DUP 0< IF NEGATE THEN ;
```

```rust
HIRFunction {
    name: "ABS",
    stack_effect: StackEffect {
        inputs: vec![Int],
        outputs: vec![Int],
    },
    body: vec![
        HIRInstruction::StackOp(StackOp::Dup),
        HIRInstruction::Literal(Literal::Int32(0)),
        HIRInstruction::Call { word_id: LESS_THAN, immediate: false, inferred_type: None },
        HIRInstruction::If {
            condition_value: None,
            then_block: BlockId(0),
            else_block: None,
            stack_effect: StackEffect {
                inputs: vec![Int],
                outputs: vec![Int],
            },
        },
    ],
    /* ... */
}

// Block 0 (then branch)
vec![
    HIRInstruction::Call { word_id: NEGATE, immediate: false, inferred_type: None },
]
```

---

## 2. Mid-Level IR (MIR)

### 2.1 Design Philosophy

MIR uses SSA form to make data flow explicit and enable standard compiler optimizations. Stack operations are converted to explicit value operations.

### 2.2 Complete MIR Instruction Set

```rust
/// MIR Function with SSA values
#[derive(Debug, Clone)]
pub struct MIRFunction {
    pub id: FunctionId,
    pub name: String,
    pub parameters: Vec<ValueId>,
    pub return_values: Vec<ValueId>,
    pub blocks: Vec<BasicBlock>,
    pub value_types: HashMap<ValueId, Type>,
    pub stack_effect: StackEffect,
}

/// Basic block in MIR
#[derive(Debug, Clone)]
pub struct BasicBlock {
    pub id: BlockId,
    pub predecessors: Vec<BlockId>,
    pub instructions: Vec<MIRInstruction>,
    pub terminator: Terminator,
    pub phi_nodes: Vec<PhiNode>,
}

/// SSA phi node for control flow merges
#[derive(Debug, Clone)]
pub struct PhiNode {
    pub result: ValueId,
    pub incoming: Vec<(ValueId, BlockId)>,
    pub ty: Type,
}

/// MIR instruction (SSA form)
#[derive(Debug, Clone)]
pub enum MIRInstruction {
    /// Binary operations
    BinOp {
        op: BinOpKind,
        lhs: ValueId,
        rhs: ValueId,
        result: ValueId,
        ty: Type,
    },

    /// Unary operations
    UnOp {
        op: UnOpKind,
        operand: ValueId,
        result: ValueId,
        ty: Type,
    },

    /// Comparison
    Compare {
        op: CompareOp,
        lhs: ValueId,
        rhs: ValueId,
        result: ValueId,
    },

    /// Load from memory
    Load {
        addr: ValueId,
        result: ValueId,
        ty: Type,
        alignment: usize,
        volatile: bool,
    },

    /// Store to memory
    Store {
        addr: ValueId,
        value: ValueId,
        alignment: usize,
        volatile: bool,
    },

    /// Direct function call
    Call {
        callee: FunctionId,
        args: Vec<ValueId>,
        results: Vec<ValueId>,
        tail_call: bool,
    },

    /// Indirect call through function pointer
    IndirectCall {
        callee: ValueId,
        args: Vec<ValueId>,
        results: Vec<ValueId>,
    },

    /// Constant value
    Const {
        value: ConstValue,
        result: ValueId,
        ty: Type,
    },

    /// Cast between types
    Cast {
        value: ValueId,
        from_ty: Type,
        to_ty: Type,
        result: ValueId,
        kind: CastKind,
    },

    /// Get address of local variable
    GetLocal {
        local_id: LocalId,
        result: ValueId,
    },

    /// Allocate stack memory
    StackAlloc {
        size: usize,
        alignment: usize,
        result: ValueId,
    },

    /// Array/structure element access
    GetElementPtr {
        base: ValueId,
        indices: Vec<ValueId>,
        result: ValueId,
        ty: Type,
    },

    /// Stack operations (for non-optimized paths)
    StackPush { value: ValueId },
    StackPop { result: ValueId },
    StackPeek { offset: usize, result: ValueId },

    /// Debug/metadata
    Debug(DebugInfo),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BinOpKind {
    Add, Sub, Mul, Div, Mod,
    And, Or, Xor,
    Shl, Shr, // Logical shift
    AShr,     // Arithmetic shift right
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UnOpKind {
    Neg,      // Arithmetic negation
    Not,      // Bitwise NOT
    BoolNot,  // Logical NOT
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CompareOp {
    Eq, Ne,
    Lt, Le, Gt, Ge,
    ULt, ULe, UGt, UGe, // Unsigned comparisons
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CastKind {
    ZeroExtend,    // Unsigned extension
    SignExtend,    // Signed extension
    Truncate,      // Narrow to smaller type
    FloatToInt,
    IntToFloat,
    Bitcast,       // Reinterpret bits
}

/// Control flow terminators
#[derive(Debug, Clone)]
pub enum Terminator {
    /// Unconditional branch
    Br { target: BlockId },

    /// Conditional branch
    CondBr {
        cond: ValueId,
        then_target: BlockId,
        else_target: BlockId,
    },

    /// Multi-way branch (switch)
    Switch {
        scrutinee: ValueId,
        cases: Vec<(ConstValue, BlockId)>,
        default: BlockId,
    },

    /// Return from function
    Return { values: Vec<ValueId> },

    /// Unreachable (optimization hint)
    Unreachable,
}

/// Constant value in MIR
#[derive(Debug, Clone, PartialEq)]
pub enum ConstValue {
    Int(i64),
    Float(f64),
    Bool(bool),
    Null,
    Undef,
}
```

### 2.3 MIR Examples

**Example 1: QUADRATIC in MIR**

```
function QUADRATIC(v0: i64, v1: i64, v2: i64, v3: i64) -> i64 {
  bb0:
    ; v0 = a, v1 = b, v2 = c, v3 = x
    v4 = binop mul, v2, v3 : i64        ; c * x
    v5 = binop add, v4, v1 : i64        ; c*x + b
    v6 = binop mul, v5, v3 : i64        ; (c*x + b) * x
    v7 = binop add, v6, v0 : i64        ; result
    return [v7]
}
```

**Example 2: ABS with control flow**

```
function ABS(v0: i64) -> i64 {
  bb0:
    v1 = const 0 : i64
    v2 = compare lt, v0, v1             ; v0 < 0
    cond_br v2, bb1, bb2

  bb1:  ; negative case
    v3 = unop neg, v0 : i64
    br bb3

  bb2:  ; positive case
    br bb3

  bb3:  ; merge
    v4 = phi [(v3, bb1), (v0, bb2)] : i64
    return [v4]
}
```

**Example 3: Loop in MIR**

```forth
: SUM ( addr count -- sum )
  0 SWAP 0 DO
    OVER I CELLS + @ +
  LOOP
  NIP ;
```

```
function SUM(v0: ptr, v1: i64) -> i64 {
  bb0:
    v2 = const 0 : i64          ; accumulator
    v3 = const 0 : i64          ; loop index
    br bb1

  bb1:  ; loop header
    v4 = phi [(v2, bb0), (v9, bb2)] : i64    ; accumulator
    v5 = phi [(v3, bb0), (v10, bb2)] : i64   ; index
    v6 = compare lt, v5, v1                   ; i < count
    cond_br v6, bb2, bb3

  bb2:  ; loop body
    v7 = binop mul, v5, 8 : i64              ; i * sizeof(cell)
    v8 = get_element_ptr v0, [v7] : ptr
    v9_temp = load v8 : i64
    v9 = binop add, v4, v9_temp : i64        ; sum += array[i]
    v10 = binop add, v5, 1 : i64             ; i++
    br bb1

  bb3:  ; exit
    return [v4]
}
```

---

## 3. Low-Level IR (LIR)

### 3.1 Design Philosophy

LIR is a register-based representation close to assembly but platform-independent. It's the final IR before code generation.

### 3.2 Complete LIR Instruction Set

```rust
/// LIR Function with register allocation
#[derive(Debug, Clone)]
pub struct LIRFunction {
    pub id: FunctionId,
    pub name: String,
    pub calling_convention: CallingConvention,
    pub stack_frame_size: usize,
    pub blocks: Vec<LIRBasicBlock>,
    pub register_usage: RegisterUsage,
}

#[derive(Debug, Clone)]
pub struct LIRBasicBlock {
    pub id: BlockId,
    pub label: Label,
    pub instructions: Vec<LIRInstruction>,
}

/// Low-level instruction
#[derive(Debug, Clone)]
pub enum LIRInstruction {
    /// Move data
    Move {
        src: Operand,
        dst: Operand,
    },

    /// Binary arithmetic
    Add { src: Operand, dst: Operand },
    Sub { src: Operand, dst: Operand },
    Mul { src: Operand, dst: Operand },
    Div { src: Operand, dst: Operand },

    /// Bitwise operations
    And { src: Operand, dst: Operand },
    Or { src: Operand, dst: Operand },
    Xor { src: Operand, dst: Operand },
    Shl { src: Operand, dst: Operand },
    Shr { src: Operand, dst: Operand },

    /// Unary operations
    Neg { dst: Operand },
    Not { dst: Operand },

    /// Comparison
    Cmp {
        lhs: Operand,
        rhs: Operand,
    },

    /// Memory operations
    Load {
        addr: Operand,
        dst: Register,
        size: MemSize,
    },

    Store {
        src: Operand,
        addr: Operand,
        size: MemSize,
    },

    /// Stack operations
    Push { src: Operand },
    Pop { dst: Register },

    /// Control flow
    Jump { target: Label },

    JumpIf {
        condition: Condition,
        target: Label,
    },

    Call {
        target: CallTarget,
        args: Vec<Operand>,
    },

    Return,

    /// Special
    Nop,
    Label(Label),
}

/// Operand types
#[derive(Debug, Clone, PartialEq)]
pub enum Operand {
    /// Physical register
    Register(Register),

    /// Immediate constant
    Immediate(i64),

    /// Memory location
    Memory(MemoryOperand),

    /// Stack slot
    StackSlot(i32),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Register {
    // x86-64 general purpose
    RAX, RBX, RCX, RDX,
    RSI, RDI, RBP, RSP,
    R8, R9, R10, R11, R12, R13, R14, R15,

    // x86-64 SSE/AVX
    XMM0, XMM1, XMM2, XMM3, XMM4, XMM5, XMM6, XMM7,
    XMM8, XMM9, XMM10, XMM11, XMM12, XMM13, XMM14, XMM15,

    // Virtual register (before register allocation)
    Virtual(u32),
}

#[derive(Debug, Clone, PartialEq)]
pub struct MemoryOperand {
    pub base: Option<Register>,
    pub index: Option<Register>,
    pub scale: u8,  // 1, 2, 4, or 8
    pub displacement: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MemSize {
    Byte,   // 1 byte
    Word,   // 2 bytes
    Dword,  // 4 bytes
    Qword,  // 8 bytes
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Condition {
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    Below,       // Unsigned less
    BelowEqual,  // Unsigned less or equal
    Above,       // Unsigned greater
    AboveEqual,  // Unsigned greater or equal
}

#[derive(Debug, Clone)]
pub enum CallTarget {
    Direct(Symbol),
    Indirect(Register),
}

/// Calling convention
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CallingConvention {
    SystemV,     // Linux/macOS x86-64
    Win64,       // Windows x86-64
    AAPCS,       // ARM
    FastCall,    // Custom fast calling convention
}
```

### 3.3 LIR Example

**QUADRATIC in LIR (x86-64 System V)**

```
function QUADRATIC:
  ; Arguments: rdi=a, rsi=b, rdx=c, rcx=x
  ; Result: rax

  ; c * x
  mov rax, rdx          ; rax = c
  imul rax, rcx         ; rax = c * x

  ; (c*x) + b
  add rax, rsi          ; rax = c*x + b

  ; result * x
  imul rax, rcx         ; rax = (c*x + b) * x

  ; final + a
  add rax, rdi          ; rax = result

  ret
```

---

## 4. IR Transformation Algorithms

### 4.1 HIR → MIR Lowering

```rust
pub struct HIRToMIRLowerer {
    value_counter: u32,
    block_counter: u32,
    stack_simulation: Vec<ValueId>,
}

impl HIRToMIRLowerer {
    pub fn lower(&mut self, hir: HIRFunction) -> MIRFunction {
        let mut mir = MIRFunction::new(hir.id, hir.name);

        // Create entry block
        let entry_block = self.new_block();

        // Allocate parameters as initial stack values
        for param_ty in &hir.stack_effect.inputs {
            let value = self.new_value(*param_ty);
            mir.parameters.push(value);
            self.stack_simulation.push(value);
        }

        // Lower each HIR instruction
        for inst in hir.body {
            self.lower_instruction(inst, &mut mir, entry_block);
        }

        mir
    }

    fn lower_instruction(
        &mut self,
        inst: HIRInstruction,
        mir: &mut MIRFunction,
        current_block: BlockId,
    ) {
        match inst {
            HIRInstruction::Call { word_id, .. } => {
                let word = self.lookup_word(word_id);
                let args = self.pop_values(word.stack_effect.inputs.len());
                let results = self.new_values(&word.stack_effect.outputs);

                mir.add_instruction(
                    current_block,
                    MIRInstruction::Call {
                        callee: word_id,
                        args,
                        results: results.clone(),
                        tail_call: false,
                    }
                );

                self.push_values(&results);
            }

            HIRInstruction::StackOp(StackOp::Dup) => {
                let top = self.peek_value(0);
                self.stack_simulation.push(top);
            }

            HIRInstruction::StackOp(StackOp::Drop) => {
                self.stack_simulation.pop();
            }

            HIRInstruction::StackOp(StackOp::Swap) => {
                let a = self.pop_value();
                let b = self.pop_value();
                self.stack_simulation.push(a);
                self.stack_simulation.push(b);
            }

            HIRInstruction::If { then_block, else_block, .. } => {
                let cond = self.pop_value();

                let then_bb = self.new_block();
                let else_bb = else_block.map(|_| self.new_block());
                let merge_bb = self.new_block();

                mir.add_terminator(
                    current_block,
                    Terminator::CondBr {
                        cond,
                        then_target: then_bb,
                        else_target: else_bb.unwrap_or(merge_bb),
                    }
                );

                // Lower then branch
                self.lower_block(then_block, mir, then_bb);
                mir.add_terminator(then_bb, Terminator::Br { target: merge_bb });

                // Lower else branch if present
                if let Some(else_bb_id) = else_bb {
                    self.lower_block(else_block.unwrap(), mir, else_bb_id);
                    mir.add_terminator(else_bb_id, Terminator::Br { target: merge_bb });
                }

                // Continue at merge block
                current_block = merge_bb;
            }

            // ... other instructions
        }
    }
}
```

### 4.2 MIR → LIR Lowering

```rust
pub struct MIRToLIRLowerer {
    virtual_reg_counter: u32,
    register_allocator: RegisterAllocator,
}

impl MIRToLIRLowerer {
    pub fn lower(&mut self, mir: MIRFunction) -> LIRFunction {
        let mut lir = LIRFunction::new(mir.id, mir.name);

        // Allocate registers
        let allocation = self.register_allocator.allocate(&mir);

        for block in mir.blocks {
            let mut lir_block = LIRBasicBlock::new(block.id);

            // Lower phi nodes to moves at end of predecessors
            for phi in block.phi_nodes {
                self.lower_phi(phi, &mut lir, &allocation);
            }

            // Lower instructions
            for inst in block.instructions {
                let lir_insts = self.lower_instruction(inst, &allocation);
                lir_block.instructions.extend(lir_insts);
            }

            // Lower terminator
            let term_insts = self.lower_terminator(block.terminator, &allocation);
            lir_block.instructions.extend(term_insts);

            lir.add_block(lir_block);
        }

        lir
    }

    fn lower_instruction(
        &self,
        inst: MIRInstruction,
        allocation: &RegisterAllocation,
    ) -> Vec<LIRInstruction> {
        match inst {
            MIRInstruction::BinOp { op, lhs, rhs, result, .. } => {
                let lhs_reg = allocation.get(lhs);
                let rhs_reg = allocation.get(rhs);
                let dst_reg = allocation.get(result);

                vec![
                    LIRInstruction::Move {
                        src: Operand::Register(lhs_reg),
                        dst: Operand::Register(dst_reg),
                    },
                    match op {
                        BinOpKind::Add => LIRInstruction::Add {
                            src: Operand::Register(rhs_reg),
                            dst: Operand::Register(dst_reg),
                        },
                        BinOpKind::Mul => LIRInstruction::Mul {
                            src: Operand::Register(rhs_reg),
                            dst: Operand::Register(dst_reg),
                        },
                        // ... other ops
                    }
                ]
            }

            // ... other instructions
        }
    }
}
```

---

## 5. IR Validation

Each IR level has validation checks to ensure correctness:

```rust
pub trait IRValidator {
    type IR;
    fn validate(&self, ir: &Self::IR) -> Result<(), ValidationError>;
}

pub struct HIRValidator;

impl IRValidator for HIRValidator {
    type IR = HIRFunction;

    fn validate(&self, hir: &HIRFunction) -> Result<(), ValidationError> {
        // Check stack effect consistency
        let computed_effect = self.compute_stack_effect(&hir.body)?;
        if computed_effect != hir.stack_effect {
            return Err(ValidationError::StackEffectMismatch {
                declared: hir.stack_effect.clone(),
                computed: computed_effect,
            });
        }

        // Check all referenced words exist
        for inst in &hir.body {
            if let HIRInstruction::Call { word_id, .. } = inst {
                if !self.word_exists(*word_id) {
                    return Err(ValidationError::UndefinedWord(*word_id));
                }
            }
        }

        Ok(())
    }
}

pub struct MIRValidator;

impl IRValidator for MIRValidator {
    type IR = MIRFunction;

    fn validate(&self, mir: &MIRFunction) -> Result<(), ValidationError> {
        // Check SSA property: each value defined exactly once
        let mut defined = HashSet::new();
        for block in &mir.blocks {
            for inst in &block.instructions {
                for def in inst.defined_values() {
                    if !defined.insert(def) {
                        return Err(ValidationError::MultipleDefinitions(def));
                    }
                }
            }
        }

        // Check all uses have definitions
        for block in &mir.blocks {
            for inst in &block.instructions {
                for used in inst.used_values() {
                    if !defined.contains(&used) {
                        return Err(ValidationError::UndefinedValue(used));
                    }
                }
            }
        }

        // Check CFG well-formedness
        self.validate_cfg(mir)?;

        Ok(())
    }
}
```

---

## 6. Performance Characteristics

| Operation | HIR | MIR | LIR |
|-----------|-----|-----|-----|
| Size | 100% | 150-200% | 120-150% |
| Analysis complexity | O(n) | O(n log n) | O(n) |
| Optimization potential | Low | High | Medium |
| Construction time | 1ms | 5-10ms | 3-5ms |

---

This specification provides the complete foundation for implementing Fast Forth's IR system. All development streams working on IR transformation should reference this document.
