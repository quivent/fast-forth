//! Symbolic Values
//!
//! Represents values symbolically for execution analysis

use std::fmt;

/// Symbolic value for abstract execution
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SymbolicValue {
    /// Concrete integer value
    Concrete(i64),

    /// Symbolic variable (input parameter)
    Variable {
        name: String,
        index: usize,
    },

    /// Binary operation
    BinaryOp {
        op: BinaryOperator,
        left: Box<SymbolicValue>,
        right: Box<SymbolicValue>,
    },

    /// Unary operation
    UnaryOp {
        op: UnaryOperator,
        value: Box<SymbolicValue>,
    },

    /// Conditional expression
    Conditional {
        condition: Box<SymbolicValue>,
        then_val: Box<SymbolicValue>,
        else_val: Box<SymbolicValue>,
    },
}

impl SymbolicValue {
    /// Create a concrete value
    pub fn concrete(val: i64) -> Self {
        SymbolicValue::Concrete(val)
    }

    /// Create a symbolic variable
    pub fn variable(name: String, index: usize) -> Self {
        SymbolicValue::Variable { name, index }
    }

    /// Create a binary operation
    pub fn binary_op(op: BinaryOperator, left: SymbolicValue, right: SymbolicValue) -> Self {
        SymbolicValue::BinaryOp {
            op,
            left: Box::new(left),
            right: Box::new(right),
        }
    }

    /// Create a unary operation
    pub fn unary_op(op: UnaryOperator, value: SymbolicValue) -> Self {
        SymbolicValue::UnaryOp {
            op,
            value: Box::new(value),
        }
    }

    /// Simplify the symbolic value
    pub fn simplify(&self) -> SymbolicValue {
        match self {
            SymbolicValue::BinaryOp { op, left, right } => {
                let left = left.simplify();
                let right = right.simplify();

                match (op, &left, &right) {
                    // Constant folding
                    (BinaryOperator::Add, SymbolicValue::Concrete(a), SymbolicValue::Concrete(b)) => {
                        SymbolicValue::Concrete(a + b)
                    }
                    (BinaryOperator::Sub, SymbolicValue::Concrete(a), SymbolicValue::Concrete(b)) => {
                        SymbolicValue::Concrete(a - b)
                    }
                    (BinaryOperator::Mul, SymbolicValue::Concrete(a), SymbolicValue::Concrete(b)) => {
                        SymbolicValue::Concrete(a * b)
                    }
                    (BinaryOperator::Div, SymbolicValue::Concrete(a), SymbolicValue::Concrete(b)) if *b != 0 => {
                        SymbolicValue::Concrete(a / b)
                    }

                    // Algebraic identities
                    (BinaryOperator::Add, _, SymbolicValue::Concrete(0)) => left,
                    (BinaryOperator::Add, SymbolicValue::Concrete(0), _) => right,
                    (BinaryOperator::Mul, _, SymbolicValue::Concrete(1)) => left,
                    (BinaryOperator::Mul, SymbolicValue::Concrete(1), _) => right,
                    (BinaryOperator::Mul, _, SymbolicValue::Concrete(0)) => SymbolicValue::Concrete(0),
                    (BinaryOperator::Mul, SymbolicValue::Concrete(0), _) => SymbolicValue::Concrete(0),

                    _ => SymbolicValue::BinaryOp {
                        op: *op,
                        left: Box::new(left),
                        right: Box::new(right),
                    },
                }
            }
            SymbolicValue::UnaryOp { op, value } => {
                let value = value.simplify();
                match (op, &value) {
                    (UnaryOperator::Negate, SymbolicValue::Concrete(v)) => {
                        SymbolicValue::Concrete(-v)
                    }
                    _ => SymbolicValue::UnaryOp {
                        op: *op,
                        value: Box::new(value),
                    },
                }
            }
            _ => self.clone(),
        }
    }
}

impl fmt::Display for SymbolicValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SymbolicValue::Concrete(v) => write!(f, "{}", v),
            SymbolicValue::Variable { name, index } => write!(f, "{}_{}", name, index),
            SymbolicValue::BinaryOp { op, left, right } => {
                write!(f, "({} {} {})", left, op, right)
            }
            SymbolicValue::UnaryOp { op, value } => {
                write!(f, "({}{})", op, value)
            }
            SymbolicValue::Conditional { condition, then_val, else_val } => {
                write!(f, "(if {} then {} else {})", condition, then_val, else_val)
            }
        }
    }
}

/// Binary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BinaryOperator {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    And,
    Or,
    Lt,
    Gt,
    Eq,
    Lte,
    Gte,
    Neq,
}

impl fmt::Display for BinaryOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BinaryOperator::Add => write!(f, "+"),
            BinaryOperator::Sub => write!(f, "-"),
            BinaryOperator::Mul => write!(f, "*"),
            BinaryOperator::Div => write!(f, "/"),
            BinaryOperator::Mod => write!(f, "mod"),
            BinaryOperator::And => write!(f, "and"),
            BinaryOperator::Or => write!(f, "or"),
            BinaryOperator::Lt => write!(f, "<"),
            BinaryOperator::Gt => write!(f, ">"),
            BinaryOperator::Eq => write!(f, "="),
            BinaryOperator::Lte => write!(f, "<="),
            BinaryOperator::Gte => write!(f, ">="),
            BinaryOperator::Neq => write!(f, "<>"),
        }
    }
}

/// Unary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UnaryOperator {
    Negate,
    Not,
    Abs,
}

impl fmt::Display for UnaryOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UnaryOperator::Negate => write!(f, "-"),
            UnaryOperator::Not => write!(f, "not "),
            UnaryOperator::Abs => write!(f, "abs "),
        }
    }
}

/// Symbolic stack for abstract execution
#[derive(Debug, Clone)]
pub struct SymbolicStack {
    stack: Vec<SymbolicValue>,
}

impl SymbolicStack {
    pub fn new() -> Self {
        Self { stack: Vec::new() }
    }

    pub fn push(&mut self, value: SymbolicValue) {
        self.stack.push(value);
    }

    pub fn pop(&mut self) -> Option<SymbolicValue> {
        self.stack.pop()
    }

    pub fn pop2(&mut self) -> Option<(SymbolicValue, SymbolicValue)> {
        let b = self.pop()?;
        let a = self.pop()?;
        Some((a, b))
    }

    pub fn dup(&mut self) -> Option<()> {
        let val = self.stack.last()?.clone();
        self.push(val);
        Some(())
    }

    pub fn swap(&mut self) -> Option<()> {
        let (a, b) = self.pop2()?;
        self.push(b);
        self.push(a);
        Some(())
    }

    pub fn over(&mut self) -> Option<()> {
        let len = self.stack.len();
        if len < 2 {
            return None;
        }
        let val = self.stack[len - 2].clone();
        self.push(val);
        Some(())
    }

    pub fn rot(&mut self) -> Option<()> {
        let c = self.pop()?;
        let b = self.pop()?;
        let a = self.pop()?;
        self.push(b);
        self.push(c);
        self.push(a);
        Some(())
    }

    pub fn depth(&self) -> usize {
        self.stack.len()
    }

    pub fn get_stack(&self) -> &[SymbolicValue] {
        &self.stack
    }

    pub fn clear(&mut self) {
        self.stack.clear();
    }
}

impl Default for SymbolicStack {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symbolic_value_display() {
        let val = SymbolicValue::binary_op(
            BinaryOperator::Add,
            SymbolicValue::variable("a".to_string(), 0),
            SymbolicValue::concrete(5),
        );

        let display = format!("{}", val);
        assert!(display.contains("a_0"));
        assert!(display.contains("5"));
    }

    #[test]
    fn test_simplification() {
        // a + 0 = a
        let val = SymbolicValue::binary_op(
            BinaryOperator::Add,
            SymbolicValue::variable("a".to_string(), 0),
            SymbolicValue::concrete(0),
        );

        let simplified = val.simplify();
        assert_eq!(simplified, SymbolicValue::variable("a".to_string(), 0));
    }

    #[test]
    fn test_constant_folding() {
        // 2 + 3 = 5
        let val = SymbolicValue::binary_op(
            BinaryOperator::Add,
            SymbolicValue::concrete(2),
            SymbolicValue::concrete(3),
        );

        let simplified = val.simplify();
        assert_eq!(simplified, SymbolicValue::concrete(5));
    }

    #[test]
    fn test_symbolic_stack() {
        let mut stack = SymbolicStack::new();
        stack.push(SymbolicValue::concrete(1));
        stack.push(SymbolicValue::concrete(2));

        assert_eq!(stack.depth(), 2);

        stack.dup().unwrap();
        assert_eq!(stack.depth(), 3);

        let val = stack.pop().unwrap();
        assert_eq!(val, SymbolicValue::concrete(2));
    }
}
