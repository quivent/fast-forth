//! Type Composition Engine
//!
//! Formal composition verification: ( a -- b ) ∘ ( c -- d )

use super::{AlgebraicStackEffect, AlgebraicType, TypeVariable};
use super::unification::Unifier;
use thiserror::Error;
use serde::{Serialize, Deserialize};

/// Composition result with detailed information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompositionResult {
    pub valid: bool,
    pub composed_effect: Option<String>,
    pub unification_steps: Vec<UnificationStep>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnificationStep {
    pub description: String,
    pub before: String,
    pub after: String,
}

/// Type composer for formal composition verification
pub struct TypeComposer {
    unifier: Unifier,
}

#[derive(Debug, Error)]
pub enum CompositionError {
    #[error("Stack underflow: {word} requires {required} items, but only {available} available")]
    StackUnderflow {
        word: String,
        required: usize,
        available: usize,
    },

    #[error("Type mismatch: cannot compose {first} with {second}")]
    TypeMismatch { first: String, second: String },

    #[error("Unification failed: {0}")]
    UnificationFailed(String),
}

impl TypeComposer {
    pub fn new() -> Self {
        Self {
            unifier: Unifier::new(),
        }
    }

    /// Compose two stack effects: f ∘ g
    ///
    /// Given:
    ///   f: ( a -- b )
    ///   g: ( c -- d )
    ///
    /// Compose to produce: ( inputs -- outputs )
    pub fn compose(
        &mut self,
        first: &AlgebraicStackEffect,
        second: &AlgebraicStackEffect,
    ) -> Result<AlgebraicStackEffect, CompositionError> {
        let mut steps = Vec::new();

        // Step 1: Check if first's outputs can satisfy second's inputs
        if first.outputs.len() < second.inputs.len() {
            return Err(CompositionError::StackUnderflow {
                word: "second".to_string(),
                required: second.inputs.len(),
                available: first.outputs.len(),
            });
        }

        // Step 2: Unify the connecting types
        // The last N outputs of first must unify with the N inputs of second
        let connect_count = second.inputs.len();
        let first_connect = &first.outputs[first.outputs.len() - connect_count..];

        for (i, (out_type, in_type)) in first_connect.iter().zip(second.inputs.iter()).enumerate() {
            self.unifier.unify(out_type, in_type).map_err(|e| {
                CompositionError::UnificationFailed(format!(
                    "Cannot unify output {} of first with input {} of second: {}",
                    i, i, e
                ))
            })?;

            steps.push(UnificationStep {
                description: format!("Unify output {} with input {}", i, i),
                before: format!("{} ~ {}", out_type, in_type),
                after: format!("unified to {}", self.unifier.resolve(out_type)),
            });
        }

        // Step 3: Build the composed effect
        // Inputs: first's inputs + any unused inputs from the stack
        let mut composed_inputs = first.inputs.clone();

        // Outputs: unconsumed outputs from first + all outputs from second
        let mut composed_outputs = Vec::new();

        // Add unconsumed outputs from first
        if first.outputs.len() > connect_count {
            for out in &first.outputs[..first.outputs.len() - connect_count] {
                composed_outputs.push(self.unifier.resolve(out));
            }
        }

        // Add all outputs from second (resolved through unification)
        for out in &second.outputs {
            composed_outputs.push(self.unifier.resolve(out));
        }

        Ok(AlgebraicStackEffect {
            inputs: composed_inputs,
            outputs: composed_outputs,
        })
    }

    /// Compose multiple effects in sequence
    pub fn compose_sequence(
        &mut self,
        effects: &[AlgebraicStackEffect],
    ) -> Result<AlgebraicStackEffect, CompositionError> {
        if effects.is_empty() {
            return Ok(AlgebraicStackEffect::new(vec![], vec![]));
        }

        let mut result = effects[0].clone();
        for effect in &effects[1..] {
            result = self.compose(&result, effect)?;
        }

        Ok(result)
    }

    /// Verify composition without returning the result
    pub fn verify_composition(
        &mut self,
        first: &AlgebraicStackEffect,
        second: &AlgebraicStackEffect,
    ) -> CompositionResult {
        match self.compose(first, second) {
            Ok(effect) => CompositionResult {
                valid: true,
                composed_effect: Some(format!("{}", effect)),
                unification_steps: vec![],
                error: None,
            },
            Err(e) => CompositionResult {
                valid: false,
                composed_effect: None,
                unification_steps: vec![],
                error: Some(e.to_string()),
            },
        }
    }
}

impl Default for TypeComposer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::type_algebra::{ConcreteType, AlgebraicType};

    #[test]
    fn test_simple_composition() {
        let mut composer = TypeComposer::new();

        // dup: ( a -- a a )
        let dup = AlgebraicStackEffect {
            inputs: vec![AlgebraicType::Var(TypeVariable { id: 0, name: Some("a".to_string()) })],
            outputs: vec![
                AlgebraicType::Var(TypeVariable { id: 0, name: Some("a".to_string()) }),
                AlgebraicType::Var(TypeVariable { id: 0, name: Some("a".to_string()) }),
            ],
        };

        // *: ( a b -- c )
        let mult = AlgebraicStackEffect {
            inputs: vec![
                AlgebraicType::Concrete(ConcreteType::Int),
                AlgebraicType::Concrete(ConcreteType::Int),
            ],
            outputs: vec![AlgebraicType::Concrete(ConcreteType::Int)],
        };

        // dup *: ( a -- a² )
        let result = composer.compose(&dup, &mult).unwrap();
        assert_eq!(result.inputs.len(), 1);
        assert_eq!(result.outputs.len(), 1);
    }

    #[test]
    fn test_stack_underflow() {
        let mut composer = TypeComposer::new();

        // drop: ( a -- )
        let drop = AlgebraicStackEffect {
            inputs: vec![AlgebraicType::Var(TypeVariable { id: 0, name: Some("a".to_string()) })],
            outputs: vec![],
        };

        // +: ( a b -- c )
        let add = AlgebraicStackEffect {
            inputs: vec![
                AlgebraicType::Concrete(ConcreteType::Int),
                AlgebraicType::Concrete(ConcreteType::Int),
            ],
            outputs: vec![AlgebraicType::Concrete(ConcreteType::Int)],
        };

        // drop + should fail (not enough outputs from drop)
        let result = composer.compose(&drop, &add);
        assert!(result.is_err());
    }

    #[test]
    fn test_swap_composition() {
        let mut composer = TypeComposer::new();

        // swap: ( a b -- b a )
        let swap = AlgebraicStackEffect {
            inputs: vec![
                AlgebraicType::Var(TypeVariable { id: 0, name: Some("a".to_string()) }),
                AlgebraicType::Var(TypeVariable { id: 1, name: Some("b".to_string()) }),
            ],
            outputs: vec![
                AlgebraicType::Var(TypeVariable { id: 1, name: Some("b".to_string()) }),
                AlgebraicType::Var(TypeVariable { id: 0, name: Some("a".to_string()) }),
            ],
        };

        // swap swap: ( a b -- a b ) - identity
        let result = composer.compose(&swap, &swap.clone()).unwrap();
        assert_eq!(result.inputs.len(), 2);
        assert_eq!(result.outputs.len(), 2);
    }
}
