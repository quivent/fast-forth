//! Intermediate Representation for Forth code
//!
//! This module defines the IR used throughout the optimization pipeline.

use crate::{OptimizerError, Result};
use smallvec::SmallVec;
use std::collections::HashMap;
use std::fmt;

/// Stack effect notation: (before -- after)
/// Example: (a b -- c) means: takes 2 items, produces 1 item
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StackEffect {
    /// Number of items consumed from stack
    pub consumed: u8,
    /// Number of items produced to stack
    pub produced: u8,
}

impl StackEffect {
    pub fn new(consumed: u8, produced: u8) -> Self {
        Self { consumed, produced }
    }

    /// Net stack change (negative = shrinks, positive = grows)
    pub fn net_change(&self) -> i32 {
        self.produced as i32 - self.consumed as i32
    }

    /// Compose two stack effects
    pub fn compose(&self, other: &StackEffect) -> Self {
        Self {
            consumed: self.consumed + other.consumed.saturating_sub(self.produced),
            produced: self.produced.saturating_sub(other.consumed) + other.produced,
        }
    }
}

impl fmt::Display for StackEffect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({} -- {})", self.consumed, self.produced)
    }
}

/// Forth instruction in IR form
#[derive(Debug, Clone, PartialEq)]
pub enum Instruction {
    // Literals
    Literal(i64),
    FloatLiteral(f64),

    // Stack operations
    Dup,       // ( a -- a a )
    Drop,      // ( a -- )
    Swap,      // ( a b -- b a )
    Over,      // ( a b -- a b a )
    Rot,       // ( a b c -- b c a )
    Nip,       // ( a b -- b )
    Tuck,      // ( a b -- b a b )
    Pick(u8),  // ( ... n -- ... n-th )
    Roll(u8),  // ( ... n -- ... rotate n items )

    // Arithmetic
    Add,       // ( a b -- a+b )
    Sub,       // ( a b -- a-b )
    Mul,       // ( a b -- a*b )
    Div,       // ( a b -- a/b )
    Mod,       // ( a b -- a%b )
    Neg,       // ( a -- -a )
    Abs,       // ( a -- |a| )

    // Bitwise
    And,       // ( a b -- a&b )
    Or,        // ( a b -- a|b )
    Xor,       // ( a b -- a^b )
    Not,       // ( a -- ~a )
    Shl,       // ( a n -- a<<n )
    Shr,       // ( a n -- a>>n )

    // Comparison
    Eq,        // ( a b -- a==b )
    Ne,        // ( a b -- a!=b )
    Lt,        // ( a b -- a<b )
    Le,        // ( a b -- a<=b )
    Gt,        // ( a b -- a>b )
    Ge,        // ( a b -- a>=b )
    ZeroEq,    // ( a -- a==0 )
    ZeroLt,    // ( a -- a<0 )
    ZeroGt,    // ( a -- a>0 )

    // Control flow
    Call(String),              // Call word by name
    Return,                    // Return from word
    Branch(usize),             // Unconditional branch to instruction
    BranchIf(usize),          // Branch if TOS is true
    BranchIfNot(usize),       // Branch if TOS is false

    // Memory operations
    Load,      // ( addr -- value )
    Store,     // ( value addr -- )
    Load8,     // ( addr -- byte )
    Store8,    // ( byte addr -- )

    // Return stack
    ToR,       // ( a -- ) (R: -- a)
    FromR,     // ( -- a ) (R: a -- )
    RFetch,    // ( -- a ) (R: a -- a)

    // Superinstructions (fused operations)
    DupAdd,           // dup + -> ( a -- a+a ) aka 2*
    DupMul,           // dup * -> ( a -- a*a ) aka square
    OverAdd,          // over + -> ( a b -- a a+b )
    SwapSub,          // swap - -> ( a b -- b-a )
    LiteralAdd(i64),  // Literal followed by +
    LiteralMul(i64),  // Literal followed by *
    IncOne,           // 1 + -> increment
    DecOne,           // 1 - -> decrement
    MulTwo,           // 2 * -> shift left 1
    DivTwo,           // 2 / -> shift right 1

    // Stack caching hints (for codegen)
    CachedDup { depth: u8 },      // Dup with known stack depth
    CachedSwap { depth: u8 },     // Swap with known stack depth
    CachedOver { depth: u8 },     // Over with known stack depth
    FlushCache,                    // Force stack cache to memory

    // Concurrency primitives (NEW)
    Spawn,         // ( xt -- thread-id ) Create OS thread
    Join,          // ( thread-id -- ) Wait for thread completion
    Channel(i64),  // ( size -- chan ) Create message queue (or literal size variant)
    Send,          // ( value chan -- ) Send to channel (blocking)
    Recv,          // ( chan -- value ) Receive from channel (blocking)
    CloseChannel,  // ( chan -- ) Close channel
    DestroyChannel, // ( chan -- ) Destroy channel

    // Metadata
    Comment(String),
    Label(String),
    Nop,
}

impl Instruction {
    /// Get the stack effect of this instruction
    pub fn stack_effect(&self) -> StackEffect {
        use Instruction::*;
        match self {
            Literal(_) | FloatLiteral(_) => StackEffect::new(0, 1),

            Dup => StackEffect::new(1, 2),
            Drop => StackEffect::new(1, 0),
            Swap | Rot | Nip | Tuck => StackEffect::new(2, 2),
            Over => StackEffect::new(2, 3),
            Pick(_) => StackEffect::new(1, 1), // Simplified
            Roll(_) => StackEffect::new(1, 0),

            Add | Sub | Mul | Div | Mod => StackEffect::new(2, 1),
            And | Or | Xor => StackEffect::new(2, 1),
            Eq | Ne | Lt | Le | Gt | Ge => StackEffect::new(2, 1),
            Shl | Shr => StackEffect::new(2, 1),

            Neg | Abs | Not => StackEffect::new(1, 1),
            ZeroEq | ZeroLt | ZeroGt => StackEffect::new(1, 1),

            Load => StackEffect::new(1, 1),
            Store => StackEffect::new(2, 0),
            Load8 => StackEffect::new(1, 1),
            Store8 => StackEffect::new(2, 0),

            ToR => StackEffect::new(1, 0),
            FromR => StackEffect::new(0, 1),
            RFetch => StackEffect::new(0, 1),

            // Superinstructions
            DupAdd | DupMul => StackEffect::new(1, 1),
            OverAdd => StackEffect::new(2, 3),
            SwapSub => StackEffect::new(2, 1),
            LiteralAdd(_) | LiteralMul(_) => StackEffect::new(1, 1),
            IncOne | DecOne | MulTwo | DivTwo => StackEffect::new(1, 1),

            // Stack caching
            CachedDup { .. } => StackEffect::new(1, 2),
            CachedSwap { .. } | CachedOver { .. } => StackEffect::new(2, 2),
            FlushCache => StackEffect::new(0, 0),

            Return | Branch(_) | BranchIf(_) | BranchIfNot(_) => StackEffect::new(0, 0),
            Call(_) => StackEffect::new(0, 0), // Depends on called word

            // Concurrency primitives
            Spawn => StackEffect::new(1, 1),          // ( xt -- thread-id )
            Join => StackEffect::new(1, 0),           // ( thread-id -- )
            Channel(_) => StackEffect::new(0, 1),     // ( -- chan ) size is literal
            Send => StackEffect::new(2, 0),           // ( value chan -- )
            Recv => StackEffect::new(1, 1),           // ( chan -- value )
            CloseChannel => StackEffect::new(1, 0),   // ( chan -- )
            DestroyChannel => StackEffect::new(1, 0), // ( chan -- )

            Comment(_) | Label(_) | Nop => StackEffect::new(0, 0),
        }
    }

    /// Check if this is a pure operation (no side effects)
    pub fn is_pure(&self) -> bool {
        use Instruction::*;
        !matches!(
            self,
            Store | Store8 | ToR | Call(_) | Return | Branch(_) |
            BranchIf(_) | BranchIfNot(_) | FlushCache |
            // Concurrency primitives are NOT pure (side effects)
            Spawn | Join | Channel(_) | Send | Recv | CloseChannel | DestroyChannel
        )
    }

    /// Check if this is a constant value
    pub fn as_constant(&self) -> Option<i64> {
        match self {
            Instruction::Literal(v) => Some(*v),
            _ => None,
        }
    }
}

/// Word definition (like a function)
#[derive(Debug, Clone, PartialEq)]
pub struct WordDef {
    pub name: String,
    pub instructions: Vec<Instruction>,
    pub stack_effect: StackEffect,
    pub is_inline: bool,
    pub cost: usize, // Instruction count for inlining decisions
}

impl WordDef {
    pub fn new(name: String, instructions: Vec<Instruction>) -> Self {
        let stack_effect = Self::calculate_stack_effect(&instructions);
        let cost = instructions.len();
        Self {
            name,
            instructions,
            stack_effect,
            is_inline: false,
            cost,
        }
    }

    fn calculate_stack_effect(instructions: &[Instruction]) -> StackEffect {
        instructions
            .iter()
            .map(|i| i.stack_effect())
            .fold(StackEffect::new(0, 0), |acc, e| acc.compose(&e))
    }

    /// Update computed properties after modification
    pub fn update(&mut self) {
        self.stack_effect = Self::calculate_stack_effect(&self.instructions);
        self.cost = self.instructions.len();
    }
}

/// Complete Forth IR with all word definitions
#[derive(Debug, Clone, PartialEq)]
pub struct ForthIR {
    pub words: HashMap<String, WordDef>,
    pub main: Vec<Instruction>,
}

impl ForthIR {
    pub fn new() -> Self {
        Self {
            words: HashMap::new(),
            main: Vec::new(),
        }
    }

    /// Parse Forth source code into IR (basic parser)
    pub fn parse(source: &str) -> Result<Self> {
        // Very basic parser for demonstration
        // A real implementation would use a proper parser (nom, pest, etc.)
        let mut ir = Self::new();
        let tokens: Vec<&str> = source.split_whitespace().collect();
        let mut instructions = Vec::new();

        for token in tokens {
            let inst = match token {
                // Stack operations
                "dup" => Instruction::Dup,
                "drop" => Instruction::Drop,
                "swap" => Instruction::Swap,
                "over" => Instruction::Over,
                "rot" => Instruction::Rot,
                "nip" => Instruction::Nip,
                "tuck" => Instruction::Tuck,

                // Arithmetic
                "+" => Instruction::Add,
                "-" => Instruction::Sub,
                "*" => Instruction::Mul,
                "/" => Instruction::Div,
                "mod" => Instruction::Mod,
                "negate" => Instruction::Neg,
                "abs" => Instruction::Abs,

                // Bitwise
                "&" | "and" => Instruction::And,
                "|" | "or" => Instruction::Or,
                "^" | "xor" => Instruction::Xor,
                "~" | "not" | "invert" => Instruction::Not,
                "<<" | "lshift" => Instruction::Shl,
                ">>" | "rshift" => Instruction::Shr,

                // Comparison
                "=" => Instruction::Eq,
                "<>" => Instruction::Ne,
                "<" => Instruction::Lt,
                "<=" => Instruction::Le,
                ">" => Instruction::Gt,
                ">=" => Instruction::Ge,
                "0=" => Instruction::ZeroEq,
                "0<" => Instruction::ZeroLt,
                "0>" => Instruction::ZeroGt,

                // Control flow
                "return" => Instruction::Return,

                // Memory
                "@" | "fetch" => Instruction::Load,
                "!" | "store" => Instruction::Store,

                _ => {
                    if let Ok(n) = token.parse::<i64>() {
                        Instruction::Literal(n)
                    } else if token.starts_with(':') || token.starts_with(';') {
                        continue; // Skip word definition markers
                    } else {
                        Instruction::Call(token.to_string())
                    }
                }
            };
            instructions.push(inst);
        }

        ir.main = instructions;
        Ok(ir)
    }

    /// Add a word definition
    pub fn add_word(&mut self, word: WordDef) {
        self.words.insert(word.name.clone(), word);
    }

    /// Get a word definition
    pub fn get_word(&self, name: &str) -> Option<&WordDef> {
        self.words.get(name)
    }

    /// Get mutable word definition
    pub fn get_word_mut(&mut self, name: &str) -> Option<&mut WordDef> {
        self.words.get_mut(name)
    }

    /// Verify stack effects are valid
    pub fn verify(&self) -> Result<()> {
        // Check main sequence
        self.verify_sequence(&self.main)?;

        // Check each word
        for (name, word) in &self.words {
            self.verify_sequence(&word.instructions).map_err(|e| {
                OptimizerError::InvalidStackEffect(format!("In word '{}': {}", name, e))
            })?;
        }

        Ok(())
    }

    fn verify_sequence(&self, instructions: &[Instruction]) -> Result<()> {
        let mut depth = 0i32;

        for (i, inst) in instructions.iter().enumerate() {
            let effect = inst.stack_effect();
            depth -= effect.consumed as i32;

            if depth < 0 {
                return Err(OptimizerError::StackUnderflow(i));
            }

            depth += effect.produced as i32;

            if depth > 255 {
                return Err(OptimizerError::StackOverflow(i));
            }
        }

        Ok(())
    }

    /// Count total instructions
    pub fn instruction_count(&self) -> usize {
        self.main.len()
            + self.words.values().map(|w| w.instructions.len()).sum::<usize>()
    }
}

impl Default for ForthIR {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stack_effect_composition() {
        let dup = StackEffect::new(1, 2); // ( a -- a a )
        let add = StackEffect::new(2, 1); // ( a b -- c )
        let composed = dup.compose(&add);  // Should be ( a -- c )

        assert_eq!(composed.consumed, 1);
        assert_eq!(composed.produced, 1);
    }

    #[test]
    fn test_instruction_stack_effect() {
        assert_eq!(Instruction::Dup.stack_effect(), StackEffect::new(1, 2));
        assert_eq!(Instruction::Add.stack_effect(), StackEffect::new(2, 1));
        assert_eq!(Instruction::Literal(42).stack_effect(), StackEffect::new(0, 1));
    }

    #[test]
    fn test_parse_simple() {
        let ir = ForthIR::parse("1 2 + dup *").unwrap();
        assert_eq!(ir.main.len(), 5);
        assert!(matches!(ir.main[0], Instruction::Literal(1)));
        assert!(matches!(ir.main[1], Instruction::Literal(2)));
        assert!(matches!(ir.main[2], Instruction::Add));
    }

    #[test]
    fn test_word_def_stack_effect() {
        let word = WordDef::new(
            "square".to_string(),
            vec![Instruction::Dup, Instruction::Mul],
        );
        assert_eq!(word.stack_effect.consumed, 1);
        assert_eq!(word.stack_effect.produced, 1);
    }

    #[test]
    fn test_verify_valid_sequence() {
        let ir = ForthIR::parse("1 2 + 3 *").unwrap();
        assert!(ir.verify().is_ok());
    }

    #[test]
    fn test_verify_underflow() {
        let mut ir = ForthIR::new();
        ir.main = vec![Instruction::Add]; // Requires 2 items but stack is empty
        assert!(matches!(ir.verify(), Err(OptimizerError::StackUnderflow(_))));
    }
}
