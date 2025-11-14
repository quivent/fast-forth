//! Type Unification
//!
//! Implements Robinson's unification algorithm for type variables

use super::{AlgebraicType, TypeVariable, ConcreteType};
use rustc_hash::FxHashMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum UnificationError {
    #[error("Cannot unify concrete types {0} and {1}")]
    ConcreteTypeMismatch(ConcreteType, ConcreteType),

    #[error("Occurs check failed: variable {0} occurs in {1}")]
    OccursCheck(String, String),

    #[error("Compound type mismatch: {0} vs {1}")]
    CompoundMismatch(String, String),
}

/// Type unifier using substitution-based algorithm
pub struct Unifier {
    /// Substitution map: type variable -> type
    substitutions: FxHashMap<TypeVariable, AlgebraicType>,
}

impl Unifier {
    pub fn new() -> Self {
        Self {
            substitutions: FxHashMap::default(),
        }
    }

    /// Unify two types
    pub fn unify(&mut self, t1: &AlgebraicType, t2: &AlgebraicType) -> Result<(), UnificationError> {
        let t1 = self.resolve(t1);
        let t2 = self.resolve(t2);

        match (&t1, &t2) {
            // Two identical concrete types unify
            (AlgebraicType::Concrete(c1), AlgebraicType::Concrete(c2)) if c1 == c2 => Ok(()),

            // Different concrete types don't unify
            (AlgebraicType::Concrete(c1), AlgebraicType::Concrete(c2)) => {
                Err(UnificationError::ConcreteTypeMismatch(c1.clone(), c2.clone()))
            }

            // Variable unifies with anything (after occurs check)
            (AlgebraicType::Var(v), t) | (t, AlgebraicType::Var(v)) => {
                if let AlgebraicType::Var(v2) = t {
                    if v == v2 {
                        return Ok(()); // Same variable
                    }
                }

                // Occurs check
                if self.occurs(v, &t) {
                    return Err(UnificationError::OccursCheck(
                        format!("{}", v),
                        format!("{}", t),
                    ));
                }

                self.substitutions.insert(v.clone(), t.clone());
                Ok(())
            }

            // Compound types must have same operation and unify bases
            (
                AlgebraicType::Compound { base: b1, operation: op1 },
                AlgebraicType::Compound { base: b2, operation: op2 },
            ) => {
                if op1 != op2 {
                    return Err(UnificationError::CompoundMismatch(
                        format!("{}", t1),
                        format!("{}", t2),
                    ));
                }
                self.unify(b1, b2)
            }

            // Concrete and compound don't unify directly
            _ => Err(UnificationError::CompoundMismatch(
                format!("{}", t1),
                format!("{}", t2),
            )),
        }
    }

    /// Resolve a type through substitutions
    pub fn resolve(&self, t: &AlgebraicType) -> AlgebraicType {
        match t {
            AlgebraicType::Var(v) => {
                if let Some(subst) = self.substitutions.get(v) {
                    self.resolve(subst)
                } else {
                    t.clone()
                }
            }
            AlgebraicType::Compound { base, operation } => AlgebraicType::Compound {
                base: Box::new(self.resolve(base)),
                operation: operation.clone(),
            },
            _ => t.clone(),
        }
    }

    /// Occurs check: does variable v occur in type t?
    fn occurs(&self, v: &TypeVariable, t: &AlgebraicType) -> bool {
        match t {
            AlgebraicType::Var(v2) => {
                if v == v2 {
                    return true;
                }
                if let Some(subst) = self.substitutions.get(v2) {
                    return self.occurs(v, subst);
                }
                false
            }
            AlgebraicType::Compound { base, .. } => self.occurs(v, base),
            AlgebraicType::Concrete(_) => false,
        }
    }

    /// Clear all substitutions
    pub fn clear(&mut self) {
        self.substitutions.clear();
    }
}

impl Default for Unifier {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unify_variables() {
        let mut unifier = Unifier::new();

        let var_a = AlgebraicType::Var(TypeVariable { id: 0, name: Some("a".to_string()) });
        let var_b = AlgebraicType::Var(TypeVariable { id: 1, name: Some("b".to_string()) });

        unifier.unify(&var_a, &var_b).unwrap();

        let resolved = unifier.resolve(&var_a);
        assert_eq!(resolved, var_b);
    }

    #[test]
    fn test_unify_concrete() {
        let mut unifier = Unifier::new();

        let int1 = AlgebraicType::Concrete(ConcreteType::Int);
        let int2 = AlgebraicType::Concrete(ConcreteType::Int);

        unifier.unify(&int1, &int2).unwrap();
    }

    #[test]
    fn test_concrete_mismatch() {
        let mut unifier = Unifier::new();

        let int_type = AlgebraicType::Concrete(ConcreteType::Int);
        let float_type = AlgebraicType::Concrete(ConcreteType::Float);

        let result = unifier.unify(&int_type, &float_type);
        assert!(result.is_err());
    }

    #[test]
    fn test_occurs_check() {
        let mut unifier = Unifier::new();

        let var = TypeVariable { id: 0, name: Some("a".to_string()) };
        let recursive = AlgebraicType::Compound {
            base: Box::new(AlgebraicType::Var(var.clone())),
            operation: super::super::TypeOperation::Square,
        };

        let result = unifier.unify(&AlgebraicType::Var(var), &recursive);
        assert!(result.is_err());
    }
}
