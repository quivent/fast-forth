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

    /// Load a string literal
    LoadString {
        dest: Register,
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
}

/// SSA converter
pub struct SSAConverter {
    next_register: usize,
    next_block: usize,
    current_block: BlockId,
    blocks: Vec<BasicBlock>,
}

impl SSAConverter {
    pub fn new() -> Self {
        Self {
            next_register: 0,
            next_block: 0,
            current_block: BlockId(0),
            blocks: Vec::new(),
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
                let dest = self.fresh_register();
                self.emit(SSAInstruction::LoadString {
                    dest,
                    value: value.clone(),
                });
                stack.push(dest);
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
                // For now, treat as opaque call
                let dest = self.fresh_register();
                let args = SmallVec::new();
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

        // Convert then branch
        self.set_current_block(then_block);
        let mut then_stack = stack.clone();
        self.convert_sequence(then_branch, &mut then_stack)?;
        self.emit(SSAInstruction::Jump {
            target: merge_block,
        });

        // Convert else branch if present
        if let Some(else_words) = else_branch {
            self.set_current_block(else_block);
            let mut else_stack = stack.clone();
            self.convert_sequence(else_words, &mut else_stack)?;
            self.emit(SSAInstruction::Jump {
                target: merge_block,
            });
        }

        // Continue from merge block
        self.set_current_block(merge_block);
        *stack = then_stack; // Use then_stack as representative

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
        // Determine number of parameters from stack effect
        let param_count = def
            .stack_effect
            .as_ref()
            .map(|e| e.inputs.len())
            .unwrap_or(0);

        let mut function = SSAFunction::new(def.name.clone(), param_count);

        // Initialize register counter with parameters
        self.next_register = param_count;

        // Create entry block
        let entry = self.create_block();
        self.set_current_block(entry);

        // Initialize stack with parameters
        let mut stack: Vec<Register> = function.parameters.clone();

        // Convert function body
        self.convert_sequence(&def.body, &mut stack)?;

        // Emit return
        let return_values = SmallVec::from_vec(stack);
        self.emit(SSAInstruction::Return {
            values: return_values,
        });

        // Move blocks to function
        function.blocks = std::mem::take(&mut self.blocks);

        Ok(function)
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

    for def in &program.definitions {
        let function = converter.convert_definition(def)?;
        functions.push(function);
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
        SSAInstruction::LoadString { dest, value } => format!("{} = load \"{}\"", dest, value),
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
}
