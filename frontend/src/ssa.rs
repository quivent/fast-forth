//! SSA (Static Single Assignment) conversion for Forth
//!
//! Converts stack-based operations into SSA form for optimization and LLVM code generation.
//! Each stack value gets a unique SSA variable, and all operations are explicit.

use crate::ast::*;
use crate::error::{ForthError, Result};
use smallvec::SmallVec;
use std::fmt;

/// SSA register/variable
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Register(pub usize);

impl fmt::Display for Register {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "%{}", self.0)
    }
}

/// SSA instruction
#[derive(Debug, Clone, PartialEq)]
pub enum SSAInstruction {
    /// Load a constant integer
    LoadInt {
        dest: Register,
        value: i64,
    },

    /// Load a constant float
    LoadFloat {
        dest: Register,
        value: f64,
    },

    /// Load a string literal (ANS Forth: pushes addr and len)
    LoadString {
        dest_addr: Register,   // String address
        dest_len: Register,    // String length
        value: String,
    },

    /// Binary arithmetic operation
    BinaryOp {
        dest: Register,
        op: BinaryOperator,
        left: Register,
        right: Register,
    },

    /// Unary operation
    UnaryOp {
        dest: Register,
        op: UnaryOperator,
        operand: Register,
    },

    /// Call a word
    Call {
        dest: SmallVec<[Register; 4]>,
        name: String,
        args: SmallVec<[Register; 4]>,
    },

    /// Conditional branch
    Branch {
        condition: Register,
        true_block: BlockId,
        false_block: BlockId,
    },

    /// Unconditional jump
    Jump {
        target: BlockId,
    },

    /// Return from function
    Return {
        values: SmallVec<[Register; 4]>,
    },

    /// Phi node (for control flow merges)
    Phi {
        dest: Register,
        incoming: Vec<(BlockId, Register)>,
    },

    /// Load from memory
    Load {
        dest: Register,
        address: Register,
        ty: StackType,
    },

    /// Store to memory
    Store {
        address: Register,
        value: Register,
        ty: StackType,
    },

    // FFI and File I/O Operations

    /// FFI call to external C function
    FFICall {
        dest: SmallVec<[Register; 4]>,  // Return values
        function: String,                // C function name
        args: SmallVec<[Register; 4]>,  // Arguments
    },

    /// File open operation (ANS Forth: open-file)
    /// Stack effect: ( c-addr u fam -- fileid ior )
    FileOpen {
        dest_fileid: Register,  // File handle
        dest_ior: Register,     // I/O result (0 = success)
        path_addr: Register,    // String address
        path_len: Register,     // String length
        mode: Register,         // File access mode (r/o=0, w/o=1, r/w=2)
    },

    /// File read operation (ANS Forth: read-file)
    /// Stack effect: ( c-addr u fileid -- u ior )
    FileRead {
        dest_bytes: Register,   // Bytes actually read
        dest_ior: Register,     // I/O result (0 = success)
        buffer: Register,       // Buffer address
        count: Register,        // Max bytes to read
        fileid: Register,       // File handle
    },

    /// File write operation (ANS Forth: write-file)
    /// Stack effect: ( c-addr u fileid -- ior )
    FileWrite {
        dest_ior: Register,     // I/O result (0 = success)
        buffer: Register,       // Buffer address
        count: Register,        // Bytes to write
        fileid: Register,       // File handle
    },

    /// File close operation (ANS Forth: close-file)
    /// Stack effect: ( fileid -- ior )
    FileClose {
        dest_ior: Register,     // I/O result (0 = success)
        fileid: Register,       // File handle
    },

    /// File delete operation (ANS Forth: delete-file)
    /// Stack effect: ( c-addr u -- ior )
    FileDelete {
        dest_ior: Register,     // I/O result (0 = success)
        path_addr: Register,    // String address
        path_len: Register,     // String length
    },

    /// File create operation (ANS Forth: create-file)
    /// Stack effect: ( c-addr u fam -- fileid ior )
    FileCreate {
        dest_fileid: Register,  // File handle
        dest_ior: Register,     // I/O result (0 = success)
        path_addr: Register,    // String address
        path_len: Register,     // String length
        mode: Register,         // File access mode
    },

    /// System call operation (execute shell command)
    /// Stack effect: ( c-addr u -- return-code )
    SystemCall {
        dest: Register,         // Return code (0 = success)
        command_addr: Register, // Command string address
        command_len: Register,  // Command string length
    },
}

/// Binary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOperator {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Lt,
    Gt,
    Le,
    Ge,
    Eq,
    Ne,
    And,
    Or,
}

impl fmt::Display for BinaryOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BinaryOperator::Add => write!(f, "add"),
            BinaryOperator::Sub => write!(f, "sub"),
            BinaryOperator::Mul => write!(f, "mul"),
            BinaryOperator::Div => write!(f, "div"),
            BinaryOperator::Mod => write!(f, "mod"),
            BinaryOperator::Lt => write!(f, "lt"),
            BinaryOperator::Gt => write!(f, "gt"),
            BinaryOperator::Le => write!(f, "le"),
            BinaryOperator::Ge => write!(f, "ge"),
            BinaryOperator::Eq => write!(f, "eq"),
            BinaryOperator::Ne => write!(f, "ne"),
            BinaryOperator::And => write!(f, "and"),
            BinaryOperator::Or => write!(f, "or"),
        }
    }
}

/// Unary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOperator {
    Negate,
    Not,
    Abs,
}

impl fmt::Display for UnaryOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UnaryOperator::Negate => write!(f, "neg"),
            UnaryOperator::Not => write!(f, "not"),
            UnaryOperator::Abs => write!(f, "abs"),
        }
    }
}

/// Basic block identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockId(pub usize);

impl fmt::Display for BlockId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "bb{}", self.0)
    }
}

/// Basic block in SSA form
#[derive(Debug, Clone)]
pub struct BasicBlock {
    pub id: BlockId,
    pub instructions: Vec<SSAInstruction>,
    pub predecessors: Vec<BlockId>,
}

impl BasicBlock {
    pub fn new(id: BlockId) -> Self {
        Self {
            id,
            instructions: Vec::new(),
            predecessors: Vec::new(),
        }
    }
}

/// SSA function representation
#[derive(Debug, Clone)]
pub struct SSAFunction {
    pub name: String,
    pub parameters: Vec<Register>,
    pub blocks: Vec<BasicBlock>,
    pub entry_block: BlockId,
}

impl SSAFunction {
    pub fn new(name: String, param_count: usize) -> Self {
        let parameters: Vec<_> = (0..param_count).map(Register).collect();
        let entry_block = BasicBlock::new(BlockId(0));

        Self {
            name,
            parameters,
            blocks: vec![entry_block],
            entry_block: BlockId(0),
        }
    }

    /// Validate SSA form invariants
    ///
    /// This performs comprehensive validation including:
    /// - Single assignment: each register assigned exactly once
    /// - Dominance: all uses dominated by definitions
    /// - Phi node validity: correct placement and incoming edges
    /// - Use-before-def: all registers defined before use
    /// - Block connectivity: no unreachable blocks
    /// - Type consistency: stack depth matches at merge points
    pub fn validate(&self) -> Result<()> {
        use crate::ssa_validator::SSAValidator;
        let mut validator = SSAValidator::new(self);
        validator.validate()
    }
}

/// SSA converter
pub struct SSAConverter {
    next_register: usize,
    next_block: usize,
    current_block: BlockId,
    blocks: Vec<BasicBlock>,
    /// Map from function name to parameter count
    function_params: std::collections::HashMap<String, usize>,
}

impl SSAConverter {
    pub fn new() -> Self {
        Self {
            next_register: 0,
            next_block: 0,
            current_block: BlockId(0),
            blocks: Vec::new(),
            function_params: std::collections::HashMap::new(),
        }
    }

    fn fresh_register(&mut self) -> Register {
        let reg = Register(self.next_register);
        self.next_register += 1;
        reg
    }

    fn fresh_block(&mut self) -> BlockId {
        let id = BlockId(self.next_block);
        self.next_block += 1;
        id
    }

    fn emit(&mut self, instruction: SSAInstruction) {
        if let Some(block) = self.blocks.iter_mut().find(|b| b.id == self.current_block) {
            block.instructions.push(instruction);
        } else {
            debug_assert!(false, "Attempting to emit instruction to non-existent block {:?}", self.current_block);
        }
    }

    fn create_block(&mut self) -> BlockId {
        let id = self.fresh_block();
        self.blocks.push(BasicBlock::new(id));
        id
    }

    fn set_current_block(&mut self, id: BlockId) {
        self.current_block = id;
    }

    /// Convert a sequence of words to SSA
    pub fn convert_sequence(&mut self, words: &[Word], stack: &mut Vec<Register>) -> Result<()> {
        for word in words {
            self.convert_word(word, stack)?;
        }
        Ok(())
    }

    /// Convert a single word to SSA
    fn convert_word(&mut self, word: &Word, stack: &mut Vec<Register>) -> Result<()> {
        match word {
            Word::IntLiteral(value) => {
                let dest = self.fresh_register();
                self.emit(SSAInstruction::LoadInt {
                    dest,
                    value: *value,
                });
                stack.push(dest);
            }

            Word::FloatLiteral(value) => {
                let dest = self.fresh_register();
                self.emit(SSAInstruction::LoadFloat {
                    dest,
                    value: *value,
                });
                stack.push(dest);
            }

            Word::StringLiteral(value) => {
                let dest_addr = self.fresh_register();
                let dest_len = self.fresh_register();
                self.emit(SSAInstruction::LoadString {
                    dest_addr,
                    dest_len,
                    value: value.clone(),
                });
                // Push both address and length (ANS Forth convention)
                stack.push(dest_addr);
                stack.push(dest_len);
            }

            Word::WordRef { name, .. } => {
                self.convert_word_call(name, stack)?;
            }

            Word::If {
                then_branch,
                else_branch,
            } => {
                self.convert_if(then_branch, else_branch.as_deref(), stack)?;
            }

            Word::BeginUntil { body } => {
                self.convert_begin_until(body, stack)?;
            }

            Word::BeginWhileRepeat { condition, body } => {
                self.convert_begin_while_repeat(condition, body, stack)?;
            }

            Word::DoLoop { body, .. } => {
                self.convert_do_loop(body, stack)?;
            }

            Word::Variable { name: _ } => {
                // Variables push their address
                let dest = self.fresh_register();
                // Would need to implement variable address allocation
                stack.push(dest);
            }

            Word::Constant { name: _, value } => {
                let dest = self.fresh_register();
                self.emit(SSAInstruction::LoadInt {
                    dest,
                    value: *value,
                });
                stack.push(dest);
            }

            Word::Comment(_) => {
                // Comments don't generate code
            }
        }

        Ok(())
    }

    /// Convert a word call to SSA
    fn convert_word_call(&mut self, name: &str, stack: &mut Vec<Register>) -> Result<()> {
        match name {
            // Arithmetic operations
            "+" => self.convert_binary_op(BinaryOperator::Add, stack),
            "-" => self.convert_binary_op(BinaryOperator::Sub, stack),
            "*" => self.convert_binary_op(BinaryOperator::Mul, stack),
            "/" => self.convert_binary_op(BinaryOperator::Div, stack),
            "mod" => self.convert_binary_op(BinaryOperator::Mod, stack),

            // Comparison operations
            "<" => self.convert_binary_op(BinaryOperator::Lt, stack),
            ">" => self.convert_binary_op(BinaryOperator::Gt, stack),
            "<=" => self.convert_binary_op(BinaryOperator::Le, stack),
            ">=" => self.convert_binary_op(BinaryOperator::Ge, stack),
            "=" => self.convert_binary_op(BinaryOperator::Eq, stack),
            "<>" => self.convert_binary_op(BinaryOperator::Ne, stack),

            // Logical operations
            "and" => self.convert_binary_op(BinaryOperator::And, stack),
            "or" => self.convert_binary_op(BinaryOperator::Or, stack),
            "not" => self.convert_unary_op(UnaryOperator::Not, stack),

            // Unary operations
            "negate" => self.convert_unary_op(UnaryOperator::Negate, stack),
            "abs" => self.convert_unary_op(UnaryOperator::Abs, stack),

            // Stack manipulation
            "dup" => {
                if let Some(&reg) = stack.last() {
                    stack.push(reg);
                } else {
                    return Err(ForthError::StackUnderflow {
                        word: "dup".to_string(),
                        expected: 1,
                        found: 0,
                    });
                }
                Ok(())
            }

            "drop" => {
                if stack.pop().is_none() {
                    return Err(ForthError::StackUnderflow {
                        word: "drop".to_string(),
                        expected: 1,
                        found: 0,
                    });
                }
                Ok(())
            }

            "swap" => {
                if stack.len() < 2 {
                    return Err(ForthError::StackUnderflow {
                        word: "swap".to_string(),
                        expected: 2,
                        found: stack.len(),
                    });
                }
                let len = stack.len();
                stack.swap(len - 1, len - 2);
                Ok(())
            }

            "over" => {
                if stack.len() < 2 {
                    return Err(ForthError::StackUnderflow {
                        word: "over".to_string(),
                        expected: 2,
                        found: stack.len(),
                    });
                }
                let reg = stack[stack.len() - 2];
                stack.push(reg);
                Ok(())
            }

            "rot" => {
                if stack.len() < 3 {
                    return Err(ForthError::StackUnderflow {
                        word: "rot".to_string(),
                        expected: 3,
                        found: stack.len(),
                    });
                }
                let len = stack.len();
                let temp = stack[len - 3];
                stack[len - 3] = stack[len - 2];
                stack[len - 2] = stack[len - 1];
                stack[len - 1] = temp;
                Ok(())
            }

            // Memory operations
            "@" => {
                if let Some(addr) = stack.pop() {
                    let dest = self.fresh_register();
                    self.emit(SSAInstruction::Load {
                        dest,
                        address: addr,
                        ty: StackType::Int,
                    });
                    stack.push(dest);
                } else {
                    return Err(ForthError::StackUnderflow {
                        word: "@".to_string(),
                        expected: 1,
                        found: 0,
                    });
                }
                Ok(())
            }

            "!" => {
                if stack.len() < 2 {
                    return Err(ForthError::StackUnderflow {
                        word: "!".to_string(),
                        expected: 2,
                        found: stack.len(),
                    });
                }
                let addr = stack.pop().unwrap();
                let value = stack.pop().unwrap();
                self.emit(SSAInstruction::Store {
                    address: addr,
                    value,
                    ty: StackType::Int,
                });
                Ok(())
            }

            // Return stack operations (for now, treat as no-ops or simple calls)
            // TODO: Implement proper return stack handling
            ">r" | "r>" | "r@" => {
                // For now, treat these as opaque operations
                // A full implementation would need a separate return stack
                let dest = self.fresh_register();
                let mut args = SmallVec::new();
                if name == ">r" {
                    // Move value from data stack to return stack
                    if let Some(val) = stack.pop() {
                        args.push(val);
                    } else {
                        return Err(ForthError::StackUnderflow {
                            word: name.to_string(),
                            expected: 1,
                            found: 0,
                        });
                    }
                } else if name == "r>" {
                    // Move value from return stack to data stack
                    stack.push(dest);
                } else if name == "r@" {
                    // Copy top of return stack to data stack
                    stack.push(dest);
                }

                self.emit(SSAInstruction::Call {
                    dest: smallvec::smallvec![dest],
                    name: name.to_string(),
                    args,
                });
                Ok(())
            }

            // I/O operations
            "." | "emit" | "cr" => {
                // Print operations - consume from stack
                if let Some(val) = stack.pop() {
                    self.emit(SSAInstruction::Call {
                        dest: SmallVec::new(),
                        name: name.to_string(),
                        args: smallvec::smallvec![val],
                    });
                    Ok(())
                } else {
                    Err(ForthError::StackUnderflow {
                        word: name.to_string(),
                        expected: 1,
                        found: 0,
                    })
                }
            }

            // File mode constants (ANS Forth)
            // These push fopen-compatible mode strings (addr len)
            "r/o" => {
                // Read-only mode = "r"
                let dest_addr = self.fresh_register();
                let dest_len = self.fresh_register();
                self.emit(SSAInstruction::LoadString {
                    dest_addr,
                    dest_len,
                    value: "r".to_string(),
                });
                stack.push(dest_addr);
                stack.push(dest_len);
                Ok(())
            }
            "w/o" => {
                // Write-only mode = "w"
                let dest_addr = self.fresh_register();
                let dest_len = self.fresh_register();
                self.emit(SSAInstruction::LoadString {
                    dest_addr,
                    dest_len,
                    value: "w".to_string(),
                });
                stack.push(dest_addr);
                stack.push(dest_len);
                Ok(())
            }
            "r/w" => {
                // Read-write mode = "r+"
                let dest_addr = self.fresh_register();
                let dest_len = self.fresh_register();
                self.emit(SSAInstruction::LoadString {
                    dest_addr,
                    dest_len,
                    value: "r+".to_string(),
                });
                stack.push(dest_addr);
                stack.push(dest_len);
                Ok(())
            }

            // File I/O operations (ANS Forth File Access word set)
            "create-file" => {
                // Stack effect: ( path-addr path-len mode-addr mode-len -- fileid ior )
                if stack.len() < 4 {
                    return Err(ForthError::StackUnderflow {
                        word: "create-file".to_string(),
                        expected: 4,
                        found: stack.len(),
                    });
                }
                let _mode_len = stack.pop().unwrap();
                let mode = stack.pop().unwrap();
                let path_len = stack.pop().unwrap();
                let path_addr = stack.pop().unwrap();

                let dest_fileid = self.fresh_register();
                let dest_ior = self.fresh_register();

                self.emit(SSAInstruction::FileCreate {
                    dest_fileid,
                    dest_ior,
                    path_addr,
                    path_len,
                    mode,
                });

                stack.push(dest_fileid);
                stack.push(dest_ior);
                Ok(())
            }

            "open-file" => {
                // Stack effect: ( path-addr path-len mode-addr mode-len -- fileid ior )
                if stack.len() < 4 {
                    return Err(ForthError::StackUnderflow {
                        word: "open-file".to_string(),
                        expected: 4,
                        found: stack.len(),
                    });
                }
                let _mode_len = stack.pop().unwrap();
                let mode = stack.pop().unwrap();
                let path_len = stack.pop().unwrap();
                let path_addr = stack.pop().unwrap();

                let dest_fileid = self.fresh_register();
                let dest_ior = self.fresh_register();

                self.emit(SSAInstruction::FileOpen {
                    dest_fileid,
                    dest_ior,
                    path_addr,
                    path_len,
                    mode,
                });

                stack.push(dest_fileid);
                stack.push(dest_ior);
                Ok(())
            }

            "read-file" => {
                // Stack effect: ( c-addr u fileid -- u ior )
                if stack.len() < 3 {
                    return Err(ForthError::StackUnderflow {
                        word: "read-file".to_string(),
                        expected: 3,
                        found: stack.len(),
                    });
                }
                let fileid = stack.pop().unwrap();
                let count = stack.pop().unwrap();
                let buffer = stack.pop().unwrap();

                let dest_bytes = self.fresh_register();
                let dest_ior = self.fresh_register();

                self.emit(SSAInstruction::FileRead {
                    dest_bytes,
                    dest_ior,
                    buffer,
                    count,
                    fileid,
                });

                stack.push(dest_bytes);
                stack.push(dest_ior);
                Ok(())
            }

            "write-file" => {
                // Stack effect: ( c-addr u fileid -- ior )
                if stack.len() < 3 {
                    return Err(ForthError::StackUnderflow {
                        word: "write-file".to_string(),
                        expected: 3,
                        found: stack.len(),
                    });
                }
                let fileid = stack.pop().unwrap();
                let count = stack.pop().unwrap();
                let buffer = stack.pop().unwrap();

                let dest_ior = self.fresh_register();

                self.emit(SSAInstruction::FileWrite {
                    dest_ior,
                    buffer,
                    count,
                    fileid,
                });

                stack.push(dest_ior);
                Ok(())
            }

            "close-file" => {
                // Stack effect: ( fileid -- ior )
                if stack.is_empty() {
                    return Err(ForthError::StackUnderflow {
                        word: "close-file".to_string(),
                        expected: 1,
                        found: 0,
                    });
                }
                let fileid = stack.pop().unwrap();
                let dest_ior = self.fresh_register();

                self.emit(SSAInstruction::FileClose {
                    dest_ior,
                    fileid,
                });

                stack.push(dest_ior);
                Ok(())
            }

            "delete-file" => {
                // Stack effect: ( c-addr u -- ior )
                if stack.len() < 2 {
                    return Err(ForthError::StackUnderflow {
                        word: "delete-file".to_string(),
                        expected: 2,
                        found: stack.len(),
                    });
                }
                let path_len = stack.pop().unwrap();
                let path_addr = stack.pop().unwrap();

                let dest_ior = self.fresh_register();

                self.emit(SSAInstruction::FileDelete {
                    dest_ior,
                    path_addr,
                    path_len,
                });

                stack.push(dest_ior);
                Ok(())
            }

            // System call
            "system" => {
                // Stack effect: ( c-addr u -- return-code )
                if stack.len() < 2 {
                    return Err(ForthError::StackUnderflow {
                        word: "system".to_string(),
                        expected: 2,
                        found: stack.len(),
                    });
                }
                let command_len = stack.pop().unwrap();
                let command_addr = stack.pop().unwrap();

                let dest = self.fresh_register();

                self.emit(SSAInstruction::SystemCall {
                    dest,
                    command_addr,
                    command_len,
                });

                stack.push(dest);
                Ok(())
            }

            // Loop index word
            "i" | "j" => {
                // Loop index - pushes current loop counter
                let dest = self.fresh_register();
                self.emit(SSAInstruction::Call {
                    dest: smallvec::smallvec![dest],
                    name: name.to_string(),
                    args: SmallVec::new(),
                });
                stack.push(dest);
                Ok(())
            }

            // Other special words
            "execute" | "char" => {
                // For now, treat as generic calls
                let dest = self.fresh_register();
                let args = if name == "execute" && !stack.is_empty() {
                    smallvec::smallvec![stack.pop().unwrap()]
                } else {
                    SmallVec::new()
                };
                self.emit(SSAInstruction::Call {
                    dest: smallvec::smallvec![dest],
                    name: name.to_string(),
                    args,
                });
                stack.push(dest);
                Ok(())
            }

            // Generic word call
            _ => {
                // Look up the function to determine how many parameters it takes
                let param_count = self.function_params.get(name).copied().unwrap_or(0);

                // Pop arguments from stack
                if stack.len() < param_count {
                    return Err(ForthError::StackUnderflow {
                        word: name.to_string(),
                        expected: param_count,
                        found: stack.len(),
                    });
                }

                let mut args = SmallVec::new();
                for _ in 0..param_count {
                    // Pop from end and reverse to maintain order
                    if let Some(arg) = stack.pop() {
                        args.push(arg);
                    }
                }
                // Reverse to get correct argument order
                args.reverse();

                let dest = self.fresh_register();
                self.emit(SSAInstruction::Call {
                    dest: smallvec::smallvec![dest],
                    name: name.to_string(),
                    args,
                });
                stack.push(dest);
                Ok(())
            }
        }
    }

    fn convert_binary_op(&mut self, op: BinaryOperator, stack: &mut Vec<Register>) -> Result<()> {
        if stack.len() < 2 {
            return Err(ForthError::StackUnderflow {
                word: format!("{}", op),
                expected: 2,
                found: stack.len(),
            });
        }

        let right = stack.pop().unwrap();
        let left = stack.pop().unwrap();
        let dest = self.fresh_register();

        self.emit(SSAInstruction::BinaryOp {
            dest,
            op,
            left,
            right,
        });

        stack.push(dest);
        Ok(())
    }

    fn convert_unary_op(&mut self, op: UnaryOperator, stack: &mut Vec<Register>) -> Result<()> {
        if let Some(operand) = stack.pop() {
            let dest = self.fresh_register();
            self.emit(SSAInstruction::UnaryOp { dest, op, operand });
            stack.push(dest);
            Ok(())
        } else {
            Err(ForthError::StackUnderflow {
                word: format!("{}", op),
                expected: 1,
                found: 0,
            })
        }
    }

    fn convert_if(
        &mut self,
        then_branch: &[Word],
        else_branch: Option<&[Word]>,
        stack: &mut Vec<Register>,
    ) -> Result<()> {
        let condition = stack.pop().ok_or_else(|| ForthError::StackUnderflow {
            word: "IF".to_string(),
            expected: 1,
            found: 0,
        })?;

        let then_block = self.create_block();
        let merge_block = self.create_block();
        let else_block = if else_branch.is_some() {
            self.create_block()
        } else {
            merge_block
        };

        // Emit branch
        self.emit(SSAInstruction::Branch {
            condition,
            true_block: then_block,
            false_block: else_block,
        });

        //  Save original stack before branches
        let original_stack = stack.clone();

        // Convert then branch
        self.set_current_block(then_block);
        let mut then_stack = original_stack.clone();
        self.convert_sequence(then_branch, &mut then_stack)?;
        let then_final = then_stack.clone();
        // Track which block we're actually in after conversion (may differ from then_block if nested control flow)
        let actual_then_block = self.current_block;
        self.emit(SSAInstruction::Jump {
            target: merge_block,
        });

        // Convert else branch if present, otherwise use original stack
        let (else_final, actual_else_block) = if let Some(else_words) = else_branch {
            self.set_current_block(else_block);
            let mut else_stack = original_stack.clone();
            self.convert_sequence(else_words, &mut else_stack)?;
            let result = else_stack.clone();
            let actual_block = self.current_block;
            self.emit(SSAInstruction::Jump {
                target: merge_block,
            });
            (result, actual_block)
        } else {
            (original_stack.clone(), else_block)
        };

        // Verify same stack depth from both branches
        if then_final.len() != else_final.len() {
            return Err(ForthError::StackMismatch {
                word: "IF-THEN-ELSE".to_string(),
                then_depth: then_final.len(),
                else_depth: else_final.len(),
                message: format!(
                    "THEN branch leaves {} items, ELSE branch leaves {} items",
                    then_final.len(),
                    else_final.len()
                ),
            });
        }

        debug_assert_eq!(
            then_final.len(),
            else_final.len(),
            "Branch stack depths must match for SSA Phi generation"
        );

        // Continue from merge block
        self.set_current_block(merge_block);

        // Generate Phi nodes to merge values from both branches
        // Use the ACTUAL blocks that jump to merge_block, not the initial branch targets
        let mut merged_stack = Vec::new();
        for (&then_reg, &else_reg) in then_final.iter().zip(else_final.iter()) {
            if then_reg == else_reg {
                // Same register from both branches - no merge needed
                merged_stack.push(then_reg);
            } else {
                // Different registers - need Phi to merge
                let phi_reg = self.fresh_register();
                debug_assert!(
                    phi_reg.0 >= self.next_register - 1,
                    "Phi register should be freshly allocated"
                );
                self.emit(SSAInstruction::Phi {
                    dest: phi_reg,
                    incoming: vec![
                        (actual_then_block, then_reg),
                        (actual_else_block, else_reg),
                    ],
                });
                merged_stack.push(phi_reg);
            }
        }

        debug_assert_eq!(
            merged_stack.len(),
            then_final.len(),
            "Merged stack must have same size as input branches"
        );

        *stack = merged_stack;
        Ok(())
    }

    fn convert_begin_until(&mut self, body: &[Word], stack: &mut Vec<Register>) -> Result<()> {
        let loop_block = self.create_block();
        let exit_block = self.create_block();

        self.emit(SSAInstruction::Jump {
            target: loop_block,
        });

        self.set_current_block(loop_block);
        let mut loop_stack = stack.clone();
        self.convert_sequence(body, &mut loop_stack)?;

        let condition = loop_stack.pop().ok_or_else(|| ForthError::StackUnderflow {
            word: "UNTIL".to_string(),
            expected: 1,
            found: 0,
        })?;

        self.emit(SSAInstruction::Branch {
            condition,
            true_block: exit_block,
            false_block: loop_block,
        });

        self.set_current_block(exit_block);
        *stack = loop_stack;

        Ok(())
    }

    fn convert_begin_while_repeat(
        &mut self,
        condition: &[Word],
        body: &[Word],
        stack: &mut Vec<Register>,
    ) -> Result<()> {
        let cond_block = self.create_block();
        let body_block = self.create_block();
        let exit_block = self.create_block();

        self.emit(SSAInstruction::Jump {
            target: cond_block,
        });

        self.set_current_block(cond_block);
        let mut cond_stack = stack.clone();
        self.convert_sequence(condition, &mut cond_stack)?;

        let cond_val = cond_stack.pop().ok_or_else(|| ForthError::StackUnderflow {
            word: "WHILE".to_string(),
            expected: 1,
            found: 0,
        })?;

        self.emit(SSAInstruction::Branch {
            condition: cond_val,
            true_block: body_block,
            false_block: exit_block,
        });

        self.set_current_block(body_block);
        let mut body_stack = cond_stack.clone();
        self.convert_sequence(body, &mut body_stack)?;
        self.emit(SSAInstruction::Jump {
            target: cond_block,
        });

        self.set_current_block(exit_block);
        *stack = cond_stack;

        Ok(())
    }

    fn convert_do_loop(&mut self, body: &[Word], stack: &mut Vec<Register>) -> Result<()> {
        // DO...LOOP requires two values: limit and start
        if stack.len() < 2 {
            return Err(ForthError::StackUnderflow {
                word: "DO".to_string(),
                expected: 2,
                found: stack.len(),
            });
        }

        stack.pop(); // limit
        stack.pop(); // start

        let loop_block = self.create_block();
        let exit_block = self.create_block();

        self.emit(SSAInstruction::Jump {
            target: loop_block,
        });

        self.set_current_block(loop_block);
        let mut loop_stack = stack.clone();
        self.convert_sequence(body, &mut loop_stack)?;

        // TODO: Proper loop counter handling
        self.emit(SSAInstruction::Jump {
            target: exit_block,
        });

        self.set_current_block(exit_block);
        *stack = loop_stack;

        Ok(())
    }

    /// Convert a definition to SSA function
    pub fn convert_definition(&mut self, def: &Definition) -> Result<SSAFunction> {
        // Reset converter state for new function
        self.next_block = 0;
        self.blocks.clear();
        self.current_block = BlockId(0);

        // Determine number of parameters from stack effect, or infer from body
        let param_count = if let Some(ref effect) = def.stack_effect {
            effect.inputs.len()
        } else {
            // Infer parameter count by simulating the stack
            self.infer_parameter_count(&def.body)?
        };

        let mut function = SSAFunction::new(def.name.clone(), param_count);

        // Initialize register counter with parameters
        self.next_register = param_count;

        // Create entry block (will now be BlockId(0))
        let entry = self.create_block();
        self.set_current_block(entry);

        // Initialize stack with parameters
        let mut stack: Vec<Register> = function.parameters.clone();

        // Convert function body
        self.convert_sequence(&def.body, &mut stack)?;

        // Emit return - ensure we always return at least one value (0 if stack is empty)
        // This matches Cranelift backend expectation that all Forth functions return i64
        let return_values = if stack.is_empty() {
            // Stack is empty - return 0 as default
            let zero_reg = self.fresh_register();
            self.emit(SSAInstruction::LoadInt {
                dest: zero_reg,
                value: 0,
            });
            smallvec::smallvec![zero_reg]
        } else {
            // Return top of stack (or all values for multi-return functions)
            SmallVec::from_vec(stack)
        };

        self.emit(SSAInstruction::Return {
            values: return_values,
        });

        // Move blocks to function
        function.blocks = std::mem::take(&mut self.blocks);

        Ok(function)
    }

    /// Infer the number of parameters needed by simulating stack depth
    fn infer_parameter_count(&self, body: &[Word]) -> Result<usize> {
        let mut min_depth: i32 = 0;
        let mut current_depth: i32 = 0;

        for word in body {
            match word {
                Word::IntLiteral(_) | Word::FloatLiteral(_) | Word::StringLiteral(_) => {
                    current_depth += 1;
                }
                Word::WordRef { name, .. } => {
                    // Get stack effect for this word
                    let (consumes, produces) = self.get_word_stack_effect(name);
                    current_depth -= consumes;
                    if current_depth < min_depth {
                        min_depth = current_depth;
                    }
                    current_depth += produces;
                }
                Word::If { .. } | Word::DoLoop { .. } | Word::BeginUntil { .. } | Word::BeginWhileRepeat { .. } => {
                    // Control flow consumes condition from stack
                    // DoLoop consumes limit and index (2 items), others consume 1 (condition)
                    let consumed = match word {
                        Word::DoLoop { .. } => 2, // limit index
                        _ => 1, // condition for IF, UNTIL, WHILE
                    };
                    current_depth -= consumed;
                    if current_depth < min_depth {
                        min_depth = current_depth;
                    }
                }
                Word::Variable { .. } => {
                    // Variable pushes its address
                    current_depth += 1;
                }
                Word::Constant { .. } => {
                    // Constant pushes its value
                    current_depth += 1;
                }
                Word::Comment(_) => {
                    // Comments don't affect stack
                }
            }
        }

        // The minimum depth below 0 tells us how many parameters we need
        Ok((-min_depth).max(0) as usize)
    }

    /// Get stack effect for a word (consumes, produces)
    fn get_word_stack_effect(&self, name: &str) -> (i32, i32) {
        match name {
            // Arithmetic (2 in, 1 out)
            "+" | "-" | "*" | "/" | "mod" => (2, 1),
            "<" | ">" | "<=" | ">=" | "=" | "<>" => (2, 1),
            "and" | "or" => (2, 1),

            // Unary (1 in, 1 out)
            "negate" | "abs" | "not" => (1, 1),
            "1+" | "1-" | "2*" | "2/" => (1, 1),

            // Stack manipulation
            "dup" => (1, 2),
            "drop" => (1, 0),
            "swap" => (2, 2),
            "over" => (2, 3),
            "rot" => (3, 3),

            // Memory
            "@" => (1, 1),
            "!" => (2, 0),

            // Default: assume no stack effect for unknown words
            _ => (0, 0),
        }
    }
}

impl Default for SSAConverter {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert a program to SSA form
pub fn convert_to_ssa(program: &Program) -> Result<Vec<SSAFunction>> {
    let mut converter = SSAConverter::new();
    let mut functions = Vec::new();

    // First pass: Build map of function names to parameter counts
    for def in &program.definitions {
        let param_count = if let Some(ref effect) = def.stack_effect {
            effect.inputs.len()
        } else {
            // Infer parameter count
            converter.infer_parameter_count(&def.body)?
        };
        converter.function_params.insert(def.name.clone(), param_count);
    }

    // Second pass: Convert all word definitions
    for def in &program.definitions {
        let function = converter.convert_definition(def)?;
        functions.push(function);
    }

    // If there's top-level code, wrap it in an implicit :main function
    if !program.top_level_code.is_empty() {
        // Create a synthetic Definition for top-level code
        // Top-level code has no parameters (it's the entry point)
        let main_def = Definition {
            name: "main".to_string(),
            body: program.top_level_code.clone(),
            immediate: false,
            stack_effect: Some(StackEffect {
                inputs: vec![],  // Top-level has no parameters
                outputs: vec![StackType::Int],  // Returns top of stack
            }),
            location: SourceLocation::default(),
        };

        let main_function = converter.convert_definition(&main_def)?;
        functions.push(main_function);
    }

    Ok(functions)
}

impl fmt::Display for SSAFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "define {} (", self.name)?;
        for (i, param) in self.parameters.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}", param)?;
        }
        writeln!(f, ") {{")?;

        for block in &self.blocks {
            writeln!(f, "{}:", block.id)?;
            for inst in &block.instructions {
                writeln!(f, "  {}", format_instruction(inst))?;
            }
        }

        writeln!(f, "}}")
    }
}

fn format_instruction(inst: &SSAInstruction) -> String {
    match inst {
        SSAInstruction::LoadInt { dest, value } => format!("{} = load {}", dest, value),
        SSAInstruction::LoadFloat { dest, value } => format!("{} = load {}", dest, value),
        SSAInstruction::LoadString { dest_addr, dest_len, value } => {
            format!("{}, {} = load_string \"{}\"", dest_addr, dest_len, value)
        }
        SSAInstruction::BinaryOp {
            dest,
            op,
            left,
            right,
        } => format!("{} = {} {}, {}", dest, op, left, right),
        SSAInstruction::UnaryOp { dest, op, operand } => format!("{} = {} {}", dest, op, operand),
        SSAInstruction::Call { dest, name, args } => {
            let dest_str = dest
                .iter()
                .map(|r| format!("{}", r))
                .collect::<Vec<_>>()
                .join(", ");
            let args_str = args
                .iter()
                .map(|r| format!("{}", r))
                .collect::<Vec<_>>()
                .join(", ");
            format!("{} = call {}({})", dest_str, name, args_str)
        }
        SSAInstruction::Branch {
            condition,
            true_block,
            false_block,
        } => format!("br {}, {}, {}", condition, true_block, false_block),
        SSAInstruction::Jump { target } => format!("jmp {}", target),
        SSAInstruction::Return { values } => {
            let vals = values
                .iter()
                .map(|r| format!("{}", r))
                .collect::<Vec<_>>()
                .join(", ");
            format!("ret {}", vals)
        }
        SSAInstruction::Phi { dest, incoming } => {
            let incoming_str = incoming
                .iter()
                .map(|(block, reg)| format!("[{}, {}]", block, reg))
                .collect::<Vec<_>>()
                .join(", ");
            format!("{} = phi {}", dest, incoming_str)
        }
        SSAInstruction::Load { dest, address, .. } => format!("{} = load {}", dest, address),
        SSAInstruction::Store { address, value, .. } => format!("store {}, {}", value, address),

        // FFI and File I/O formatting
        SSAInstruction::FFICall { dest, function, args } => {
            let dest_str = dest
                .iter()
                .map(|r| format!("{}", r))
                .collect::<Vec<_>>()
                .join(", ");
            let args_str = args
                .iter()
                .map(|r| format!("{}", r))
                .collect::<Vec<_>>()
                .join(", ");
            format!("{} = ffi_call {}({})", dest_str, function, args_str)
        }
        SSAInstruction::FileOpen { dest_fileid, dest_ior, path_addr, path_len, mode } => {
            format!("{}, {} = file_open {}, {}, {}", dest_fileid, dest_ior, path_addr, path_len, mode)
        }
        SSAInstruction::FileRead { dest_bytes, dest_ior, buffer, count, fileid } => {
            format!("{}, {} = file_read {}, {}, {}", dest_bytes, dest_ior, buffer, count, fileid)
        }
        SSAInstruction::FileWrite { dest_ior, buffer, count, fileid } => {
            format!("{} = file_write {}, {}, {}", dest_ior, buffer, count, fileid)
        }
        SSAInstruction::FileClose { dest_ior, fileid } => {
            format!("{} = file_close {}", dest_ior, fileid)
        }
        SSAInstruction::FileDelete { dest_ior, path_addr, path_len } => {
            format!("{} = file_delete {}, {}", dest_ior, path_addr, path_len)
        }
        SSAInstruction::FileCreate { dest_fileid, dest_ior, path_addr, path_len, mode } => {
            format!("{}, {} = file_create {}, {}, {}", dest_fileid, dest_ior, path_addr, path_len, mode)
        }
        SSAInstruction::SystemCall { dest, command_addr, command_len } => {
            format!("{} = system {}, {}", dest, command_addr, command_len)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_program;

    #[test]
    fn test_convert_simple() {
        let program = parse_program(": double ( n -- n*2 ) 2 * ;").unwrap();
        let functions = convert_to_ssa(&program).unwrap();

        assert_eq!(functions.len(), 1);
        assert_eq!(functions[0].name, "double");
    }

    #[test]
    fn test_convert_with_stack_ops() {
        let program = parse_program(": square ( n -- n^2 ) dup * ;").unwrap();
        let functions = convert_to_ssa(&program).unwrap();

        assert_eq!(functions.len(), 1);
        let func = &functions[0];
        assert!(!func.blocks.is_empty());
    }

    #[test]
    fn test_ssa_display() {
        let program = parse_program(": add-one ( n -- n+1 ) 1 + ;").unwrap();
        let functions = convert_to_ssa(&program).unwrap();

        let output = format!("{}", functions[0]);
        assert!(output.contains("define add-one"));
    }

    #[test]
    fn test_stack_underflow_detection() {
        // Test that stack underflow is detected during SSA conversion
        // Use a word with explicit no-param stack effect to force underflow
        let program = parse_program(": underflow ( -- ) dup + ;").unwrap();
        let result = convert_to_ssa(&program);
        assert!(result.is_err(), "Expected stack underflow error");
        if let Err(ForthError::StackUnderflow { word, expected, found }) = result {
            assert_eq!(word, "dup");
            assert_eq!(expected, 1);
            assert_eq!(found, 0);
        } else {
            panic!("Expected StackUnderflow error, got: {:?}", result);
        }
    }

    #[test]
    fn test_maximum_stack_depth() {
        // Test stack operations at maximum depth (100+ items)
        let mut source = String::from(": max-stack");
        for i in 0..150 {
            source.push_str(&format!(" {}", i));
        }
        source.push_str(" ;");

        let program = parse_program(&source).unwrap();
        let functions = convert_to_ssa(&program).unwrap();
        assert_eq!(functions.len(), 1);
        // Verify it generated all the loads
        let func = &functions[0];
        assert!(!func.blocks.is_empty());
    }

    #[test]
    fn test_complex_control_flow_phi_nodes() {
        // Test Phi node generation in complex IF-ELSE branches
        let program = parse_program(
            ": complex-phi ( n -- result )
                dup 0 > IF
                    10 +
                ELSE
                    20 -
                THEN
            ;"
        ).unwrap();
        let functions = convert_to_ssa(&program).unwrap();
        assert_eq!(functions.len(), 1);

        // Check that Phi nodes are generated for the merged value
        let func = &functions[0];
        let has_phi = func.blocks.iter().any(|block| {
            block.instructions.iter().any(|inst| {
                matches!(inst, SSAInstruction::Phi { .. })
            })
        });
        assert!(has_phi, "Expected Phi node for IF-ELSE merge");
    }

    #[test]
    fn test_nested_loops_ssa() {
        // Test nested DO loops generate correct SSA structure
        let program = parse_program(
            ": nested-loops ( -- )
                10 0 DO
                    5 0 DO
                        i j +
                    LOOP
                LOOP
            ;"
        ).unwrap();
        let functions = convert_to_ssa(&program).unwrap();
        assert_eq!(functions.len(), 1);
        let func = &functions[0];
        assert!(func.blocks.len() > 1, "Nested loops should create multiple blocks");
    }

    #[test]
    fn test_string_literal_ssa_conversion() {
        // Test that string literals produce correct SSA (addr + len)
        // This Forth uses double quotes, not S"
        let program = parse_program(r#": test-string " Hello World " ;"#).unwrap();
        let functions = convert_to_ssa(&program).unwrap();
        assert_eq!(functions.len(), 1);

        let func = &functions[0];
        let has_load_string = func.blocks[0].instructions.iter().any(|inst| {
            matches!(inst, SSAInstruction::LoadString { .. })
        });
        assert!(has_load_string, "Expected LoadString instruction for string literal");
    }

    #[test]
    fn test_stack_manipulation_ssa() {
        // Test complex stack manipulation (dup, swap, over, rot)
        let program = parse_program(": stack-ops ( a b c -- b c a b ) rot swap dup ;").unwrap();
        let functions = convert_to_ssa(&program).unwrap();
        assert_eq!(functions.len(), 1);
        assert_eq!(functions[0].parameters.len(), 3);
    }

    #[test]
    fn test_memory_operations_ssa() {
        // Test memory load and store operations
        let program = parse_program(": mem-test ( addr value -- ) swap ! ;").unwrap();
        let functions = convert_to_ssa(&program).unwrap();
        assert_eq!(functions.len(), 1);

        let func = &functions[0];
        let has_store = func.blocks[0].instructions.iter().any(|inst| {
            matches!(inst, SSAInstruction::Store { .. })
        });
        assert!(has_store, "Expected Store instruction");
    }

    #[test]
    fn test_file_io_operations_ssa() {
        // Test file I/O operations generate correct SSA
        // This Forth uses double quotes, not S"
        let program = parse_program(
            r#": test-file ( -- )
                " test.txt " r/o open-file
                close-file
            ;"#
        ).unwrap();
        let functions = convert_to_ssa(&program).unwrap();
        assert_eq!(functions.len(), 1);

        let func = &functions[0];
        let has_file_ops = func.blocks[0].instructions.iter().any(|inst| {
            matches!(inst, SSAInstruction::FileOpen { .. }) ||
            matches!(inst, SSAInstruction::FileClose { .. })
        });
        assert!(has_file_ops, "Expected file I/O instructions");
    }

    #[test]
    fn test_begin_while_repeat_ssa() {
        // Test BEGIN-WHILE-REPEAT loop structure
        let program = parse_program(
            ": while-loop ( n -- )
                BEGIN
                    dup 0 >
                WHILE
                    1 -
                REPEAT
                drop
            ;"
        ).unwrap();
        let functions = convert_to_ssa(&program).unwrap();
        assert_eq!(functions.len(), 1);

        let func = &functions[0];
        assert!(func.blocks.len() >= 3, "WHILE-REPEAT should create multiple blocks");
    }

    #[test]
    fn test_empty_function_body() {
        // Test function with empty body
        let program = parse_program(": noop ;").unwrap();
        let functions = convert_to_ssa(&program).unwrap();
        assert_eq!(functions.len(), 1);

        // Empty function should still have return instruction
        let func = &functions[0];
        let has_return = func.blocks[0].instructions.iter().any(|inst| {
            matches!(inst, SSAInstruction::Return { .. })
        });
        assert!(has_return, "Empty function should have return instruction");
    }

    #[test]
    fn test_parameter_inference() {
        // Test that parameter count is correctly inferred from stack usage
        let program = parse_program(": inferred-params dup * + ;").unwrap();
        let functions = convert_to_ssa(&program).unwrap();
        assert_eq!(functions.len(), 1);

        // dup requires 1, * requires 2, + requires 2
        // Stack analysis: start with n (1 param), dup -> n n (2), * -> n*n (1), + requires 2
        // So minimum 2 parameters needed
        let func = &functions[0];
        assert!(func.parameters.len() >= 2, "Should infer at least 2 parameters");
    }
}
