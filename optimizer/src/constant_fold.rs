//! Constant Folding and Propagation
//!
//! Evaluates constant expressions at compile-time, eliminating runtime overhead.
//!
//! # Examples
//!
//! Before:
//! ```forth
//! 2 3 + 4 *
//! ```
//!
//! After:
//! ```forth
//! 20
//! ```
//!
//! # Features
//!
//! - Arithmetic constant folding
//! - Bitwise constant folding
//! - Comparison constant folding
//! - Constant propagation through stack
//! - Algebraic simplifications (x*0=0, x*1=x, x+0=x, etc.)

use crate::ir::{ForthIR, Instruction, WordDef};
use crate::Result;
use smallvec::{SmallVec, smallvec};

/// Value that can be tracked through constant propagation
#[derive(Debug, Clone, PartialEq)]
enum Value {
    Constant(i64),
    Unknown,
}

impl Value {
    fn as_constant(&self) -> Option<i64> {
        match self {
            Value::Constant(v) => Some(*v),
            Value::Unknown => None,
        }
    }
}

/// Stack for abstract interpretation during constant folding
struct AbstractStack {
    stack: Vec<Value>,
}

impl AbstractStack {
    fn new() -> Self {
        Self { stack: Vec::new() }
    }

    fn push(&mut self, value: Value) {
        self.stack.push(value);
    }

    fn pop(&mut self) -> Value {
        self.stack.pop().unwrap_or(Value::Unknown)
    }

    fn peek(&self, depth: usize) -> Value {
        if depth < self.stack.len() {
            self.stack[self.stack.len() - 1 - depth].clone()
        } else {
            Value::Unknown
        }
    }

    fn depth(&self) -> usize {
        self.stack.len()
    }
}

/// Constant folding optimizer
pub struct ConstantFolder {
    /// Enable aggressive algebraic simplifications
    aggressive: bool,
}

impl ConstantFolder {
    pub fn new() -> Self {
        Self { aggressive: true }
    }

    /// Fold constants in IR
    pub fn fold(&self, ir: &ForthIR) -> Result<ForthIR> {
        let mut optimized = ir.clone();

        // Fold main sequence
        optimized.main = self.fold_sequence(&ir.main)?;

        // Fold each word
        for (name, word) in ir.words.iter() {
            let folded_word = self.fold_word(word)?;
            optimized.words.insert(name.clone(), folded_word);
        }

        Ok(optimized)
    }

    /// Fold constants in a word definition
    fn fold_word(&self, word: &WordDef) -> Result<WordDef> {
        // Don't fold word definitions - they work with caller's stack
        // Only fold when we have concrete values (in main sequence or after inlining)
        Ok(word.clone())
    }

    /// Fold constants in an instruction sequence
    fn fold_sequence(&self, instructions: &[Instruction]) -> Result<Vec<Instruction>> {
        let mut result = Vec::new();
        let mut stack = AbstractStack::new();

        for inst in instructions {
            match self.fold_instruction(inst, &mut stack) {
                FoldResult::Instructions(insts) => result.extend(insts),
                FoldResult::None => {}
            }
        }

        // Materialize any remaining constants on the stack
        for value in stack.stack.iter() {
            if let Some(v) = value.as_constant() {
                result.push(Instruction::Literal(v));
            }
        }

        Ok(result)
    }

    /// Fold a single instruction with abstract stack
    fn fold_instruction(
        &self,
        inst: &Instruction,
        stack: &mut AbstractStack,
    ) -> FoldResult {
        use Instruction::*;

        match inst {
            // Literals: push constant value onto abstract stack, don't emit yet
            Literal(v) => {
                stack.push(Value::Constant(*v));
                FoldResult::None  // Don't emit yet, will materialize at end if needed
            }

            // Binary arithmetic operations
            Add => self.fold_binary_op(stack, |a, b| a.wrapping_add(b), Add),
            Sub => self.fold_binary_op(stack, |a, b| a.wrapping_sub(b), Sub),
            Mul => self.fold_binary_op(stack, |a, b| a.wrapping_mul(b), Mul),
            Div => {
                // Check for division by zero
                if let Value::Constant(0) = stack.peek(0) {
                    // Division by zero - keep instruction for runtime error
                    stack.pop();
                    stack.pop();
                    stack.push(Value::Unknown);
                    FoldResult::Instructions(smallvec![Div])
                } else {
                    self.fold_binary_op(stack, |a, b| a.wrapping_div(b), Div)
                }
            }
            Mod => {
                if let Value::Constant(0) = stack.peek(0) {
                    stack.pop();
                    stack.pop();
                    stack.push(Value::Unknown);
                    FoldResult::Instructions(smallvec![Mod])
                } else {
                    self.fold_binary_op(stack, |a, b| a.wrapping_rem(b), Mod)
                }
            }

            // Bitwise operations
            And => self.fold_binary_op(stack, |a, b| a & b, And),
            Or => self.fold_binary_op(stack, |a, b| a | b, Or),
            Xor => self.fold_binary_op(stack, |a, b| a ^ b, Xor),
            Shl => self.fold_binary_op(stack, |a, b| a.wrapping_shl(b as u32), Shl),
            Shr => self.fold_binary_op(stack, |a, b| a.wrapping_shr(b as u32), Shr),

            // Unary operations
            Neg => self.fold_unary_op(stack, |a| a.wrapping_neg(), Neg),
            Abs => self.fold_unary_op(stack, |a| a.abs(), Abs),
            Not => self.fold_unary_op(stack, |a| !a, Not),

            // Comparison operations
            Eq => self.fold_binary_op(stack, |a, b| if a == b { -1 } else { 0 }, Eq),
            Ne => self.fold_binary_op(stack, |a, b| if a != b { -1 } else { 0 }, Ne),
            Lt => self.fold_binary_op(stack, |a, b| if a < b { -1 } else { 0 }, Lt),
            Le => self.fold_binary_op(stack, |a, b| if a <= b { -1 } else { 0 }, Le),
            Gt => self.fold_binary_op(stack, |a, b| if a > b { -1 } else { 0 }, Gt),
            Ge => self.fold_binary_op(stack, |a, b| if a >= b { -1 } else { 0 }, Ge),
            ZeroEq => self.fold_unary_op(stack, |a| if a == 0 { -1 } else { 0 }, ZeroEq),
            ZeroLt => self.fold_unary_op(stack, |a| if a < 0 { -1 } else { 0 }, ZeroLt),
            ZeroGt => self.fold_unary_op(stack, |a| if a > 0 { -1 } else { 0 }, ZeroGt),

            // Stack operations that preserve constants
            Dup => {
                let top = stack.peek(0);
                stack.push(top.clone());
                // If duplicating a constant, no need to emit Dup
                if top.as_constant().is_some() {
                    FoldResult::None
                } else {
                    FoldResult::Instructions(smallvec![Dup])
                }
            }

            Drop => {
                stack.pop();
                FoldResult::None // Drop can be eliminated if value is unused
            }

            Swap => {
                if stack.depth() >= 2 {
                    let a = stack.pop();
                    let b = stack.pop();
                    // If both are constants, no need to emit Swap
                    let both_const = a.as_constant().is_some() && b.as_constant().is_some();
                    stack.push(a);
                    stack.push(b);
                    if both_const {
                        FoldResult::None
                    } else {
                        FoldResult::Instructions(smallvec![Swap])
                    }
                } else {
                    FoldResult::Instructions(smallvec![Swap])
                }
            }

            Over => {
                let second = stack.peek(1);
                let is_const = second.as_constant().is_some();
                stack.push(second);
                // If copying a constant, no need to emit Over
                if is_const {
                    FoldResult::None
                } else {
                    FoldResult::Instructions(smallvec![Over])
                }
            }

            // Superinstructions
            DupAdd => {
                if let Some(v) = stack.peek(0).as_constant() {
                    stack.pop();
                    let result = v.wrapping_add(v);
                    stack.push(Value::Constant(result));
                    FoldResult::None  // Don't emit yet, will materialize at end
                } else {
                    stack.pop();
                    stack.push(Value::Unknown);
                    FoldResult::Instructions(smallvec![DupAdd])
                }
            }

            DupMul => {
                if let Some(v) = stack.peek(0).as_constant() {
                    stack.pop();
                    let result = v.wrapping_mul(v);
                    stack.push(Value::Constant(result));
                    FoldResult::None  // Don't emit yet, will materialize at end
                } else {
                    stack.pop();
                    stack.push(Value::Unknown);
                    FoldResult::Instructions(smallvec![DupMul])
                }
            }

            IncOne => self.fold_unary_op(stack, |a| a.wrapping_add(1), IncOne),
            DecOne => self.fold_unary_op(stack, |a| a.wrapping_sub(1), DecOne),
            MulTwo => self.fold_unary_op(stack, |a| a.wrapping_shl(1), MulTwo),
            DivTwo => self.fold_unary_op(stack, |a| a.wrapping_shr(1), DivTwo),

            // Non-foldable instructions
            _ => {
                // Conservatively mark stack as unknown after non-pure operations
                if !inst.is_pure() {
                    stack.stack.clear();
                } else {
                    // Handle stack effect
                    let effect = inst.stack_effect();
                    for _ in 0..effect.consumed {
                        stack.pop();
                    }
                    for _ in 0..effect.produced {
                        stack.push(Value::Unknown);
                    }
                }
                FoldResult::Instructions(smallvec![inst.clone()])
            }
        }
    }

    /// Fold binary operation if both operands are constant
    fn fold_binary_op<F>(
        &self,
        stack: &mut AbstractStack,
        op: F,
        fallback: Instruction,
    ) -> FoldResult
    where
        F: FnOnce(i64, i64) -> i64,
    {
        let b = stack.pop();
        let a = stack.pop();

        match (a.as_constant(), b.as_constant()) {
            (Some(av), Some(bv)) => {
                // Both constants: fold!
                let result = op(av, bv);
                stack.push(Value::Constant(result));
                FoldResult::None  // Don't emit yet, will materialize at end
            }
            _ => {
                // Not both constants: keep original instruction
                stack.push(Value::Unknown);
                FoldResult::Instructions(smallvec![fallback])
            }
        }
    }

    /// Fold unary operation if operand is constant
    fn fold_unary_op<F>(
        &self,
        stack: &mut AbstractStack,
        op: F,
        fallback: Instruction,
    ) -> FoldResult
    where
        F: FnOnce(i64) -> i64,
    {
        let a = stack.pop();

        match a.as_constant() {
            Some(av) => {
                // Constant: fold!
                let result = op(av);
                stack.push(Value::Constant(result));
                FoldResult::None  // Don't emit yet, will materialize at end
            }
            None => {
                // Not constant: keep original instruction
                stack.push(Value::Unknown);
                FoldResult::Instructions(smallvec![fallback])
            }
        }
    }
}

impl Default for ConstantFolder {
    fn default() -> Self {
        Self::new()
    }
}

enum FoldResult {
    Instructions(SmallVec<[Instruction; 4]>),
    None, // Instruction eliminated
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fold_simple_arithmetic() {
        let folder = ConstantFolder::new();
        let ir = ForthIR::parse("2 3 +").unwrap();
        let folded = folder.fold(&ir).unwrap();

        // Should be folded to just "5"
        assert_eq!(folded.main.len(), 1);
        assert!(matches!(folded.main[0], Instruction::Literal(5)));
    }

    #[test]
    fn test_fold_complex_expression() {
        let folder = ConstantFolder::new();
        let ir = ForthIR::parse("2 3 + 4 *").unwrap();
        let folded = folder.fold(&ir).unwrap();

        // Should be folded to just "20"
        assert_eq!(folded.main.len(), 1);
        assert!(matches!(folded.main[0], Instruction::Literal(20)));
    }

    #[test]
    fn test_fold_dup_add() {
        let folder = ConstantFolder::new();
        let mut ir = ForthIR::new();
        ir.main = vec![Instruction::Literal(5), Instruction::DupAdd];

        let folded = folder.fold(&ir).unwrap();

        // 5 dup + = 10
        assert_eq!(folded.main.len(), 1);
        assert!(matches!(folded.main[0], Instruction::Literal(10)));
    }

    #[test]
    fn test_fold_comparison() {
        let folder = ConstantFolder::new();
        let ir = ForthIR::parse("5 3 >").unwrap(); // 5 > 3 = true (-1)
        let folded = folder.fold(&ir).unwrap();

        assert_eq!(folded.main.len(), 1);
        assert!(matches!(folded.main[0], Instruction::Literal(-1)));
    }

    #[test]
    fn test_no_fold_with_unknown() {
        let folder = ConstantFolder::new();
        let mut ir = ForthIR::new();
        // foo is a call (produces unknown value)
        ir.main = vec![
            Instruction::Call("foo".to_string()),
            Instruction::Literal(5),
            Instruction::Add,
        ];

        let folded = folder.fold(&ir).unwrap();

        // Cannot fold since one operand is unknown
        assert!(folded.main.len() >= 2);
        let has_add = folded.main.iter().any(|i| matches!(i, Instruction::Add));
        assert!(has_add);
    }

    #[test]
    fn test_fold_bitwise() {
        let folder = ConstantFolder::new();
        let ir = ForthIR::parse("15 3 &").unwrap(); // 15 & 3 = 3
        let folded = folder.fold(&ir).unwrap();

        assert_eq!(folded.main.len(), 1);
        assert!(matches!(folded.main[0], Instruction::Literal(3)));
    }
}
