//! Compositional Type Algebra
//!
//! Formal type composition and unification for Forth stack effects.
//! Enables verification of word compositions before code generation.

pub mod composition;
pub mod unification;
pub mod simplification;

pub use composition::{TypeComposer, CompositionResult};
pub use unification::{Unifier, UnificationError};
pub use simplification::{SimplificationRules, simplify_effect};

use fastforth_frontend::StackEffect as FrontendStackEffect;
use fastforth_frontend::ast::StackType;
use std::fmt;

/// Extended stack effect with algebraic properties
#[derive(Debug, Clone, PartialEq)]
pub struct AlgebraicStackEffect {
    pub inputs: Vec<AlgebraicType>,
    pub outputs: Vec<AlgebraicType>,
}

impl AlgebraicStackEffect {
    pub fn new(inputs: Vec<AlgebraicType>, outputs: Vec<AlgebraicType>) -> Self {
        Self { inputs, outputs }
    }

    pub fn from_frontend(effect: &FrontendStackEffect) -> Self {
        Self {
            inputs: effect.inputs.iter().map(AlgebraicType::from_stack_type).collect(),
            outputs: effect.outputs.iter().map(AlgebraicType::from_stack_type).collect(),
        }
    }

    pub fn net_effect(&self) -> i32 {
        self.outputs.len() as i32 - self.inputs.len() as i32
    }
}

impl fmt::Display for AlgebraicStackEffect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "( ")?;
        for (i, input) in self.inputs.iter().enumerate() {
            if i > 0 {
                write!(f, " ")?;
            }
            write!(f, "{}", input)?;
        }
        write!(f, " -- ")?;
        for (i, output) in self.outputs.iter().enumerate() {
            if i > 0 {
                write!(f, " ")?;
            }
            write!(f, "{}", output)?;
        }
        write!(f, " )")
    }
}

/// Algebraic type with enhanced unification capabilities
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AlgebraicType {
    /// Concrete type
    Concrete(ConcreteType),
    /// Type variable (can unify with anything)
    Var(TypeVariable),
    /// Compound type (for complex operations)
    Compound {
        base: Box<AlgebraicType>,
        operation: TypeOperation,
    },
}

impl AlgebraicType {
    pub fn from_stack_type(stack_type: &StackType) -> Self {
        match stack_type {
            StackType::Int => AlgebraicType::Concrete(ConcreteType::Int),
            StackType::Float => AlgebraicType::Concrete(ConcreteType::Float),
            StackType::Addr => AlgebraicType::Concrete(ConcreteType::Addr),
            StackType::Bool => AlgebraicType::Concrete(ConcreteType::Bool),
            StackType::Char => AlgebraicType::Concrete(ConcreteType::Char),
            StackType::String => AlgebraicType::Concrete(ConcreteType::String),
            StackType::Var(tv) => AlgebraicType::Var(TypeVariable {
                id: tv.id,
                name: tv.name.clone(),
            }),
            StackType::Unknown => AlgebraicType::Var(TypeVariable {
                id: 9999,
                name: Some("unknown".to_string()),
            }),
        }
    }
}

impl fmt::Display for AlgebraicType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AlgebraicType::Concrete(c) => write!(f, "{}", c),
            AlgebraicType::Var(v) => write!(f, "{}", v),
            AlgebraicType::Compound { base, operation } => {
                write!(f, "{}({})", operation, base)
            }
        }
    }
}

/// Concrete types
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ConcreteType {
    Int,
    Float,
    Addr,
    Bool,
    Char,
    String,
}

impl fmt::Display for ConcreteType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConcreteType::Int => write!(f, "int"),
            ConcreteType::Float => write!(f, "float"),
            ConcreteType::Addr => write!(f, "addr"),
            ConcreteType::Bool => write!(f, "bool"),
            ConcreteType::Char => write!(f, "char"),
            ConcreteType::String => write!(f, "string"),
        }
    }
}

/// Type variable for unification
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TypeVariable {
    pub id: usize,
    pub name: Option<String>,
}

impl fmt::Display for TypeVariable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(name) = &self.name {
            write!(f, "{}", name)
        } else {
            write!(f, "t{}", self.id)
        }
    }
}

/// Type operations for compound types
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TypeOperation {
    Square,      // n²
    Negate,      // -n
    Increment,   // n+1
    Decrement,   // n-1
}

impl fmt::Display for TypeOperation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TypeOperation::Square => write!(f, "square"),
            TypeOperation::Negate => write!(f, "negate"),
            TypeOperation::Increment => write!(f, "inc"),
            TypeOperation::Decrement => write!(f, "dec"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_algebraic_effect_display() {
        let effect = AlgebraicStackEffect {
            inputs: vec![AlgebraicType::Var(TypeVariable { id: 0, name: Some("a".to_string()) })],
            outputs: vec![
                AlgebraicType::Var(TypeVariable { id: 0, name: Some("a".to_string()) }),
                AlgebraicType::Compound {
                    base: Box::new(AlgebraicType::Var(TypeVariable { id: 0, name: Some("a".to_string()) })),
                    operation: TypeOperation::Square,
                },
            ],
        };

        let display = format!("{}", effect);
        assert!(display.contains("a"));
        assert!(display.contains("square"));
    }
}
