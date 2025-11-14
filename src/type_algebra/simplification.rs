//! Algebraic Simplification Rules
//!
//! Simplifies composed stack effects using algebraic identities

use super::{AlgebraicStackEffect, AlgebraicType, TypeVariable};

/// Simplification rules for stack effects
pub struct SimplificationRules;

impl SimplificationRules {
    /// Apply all simplification rules
    pub fn simplify(effect: &AlgebraicStackEffect) -> AlgebraicStackEffect {
        let mut simplified = effect.clone();

        // Apply various simplification passes
        simplified = Self::remove_identity_operations(&simplified);
        simplified = Self::collapse_duplicate_vars(&simplified);
        simplified = Self::normalize_var_ids(&simplified);

        simplified
    }

    /// Remove identity operations (e.g., swap swap)
    fn remove_identity_operations(effect: &AlgebraicStackEffect) -> AlgebraicStackEffect {
        // For now, just return the effect unchanged
        // More sophisticated pattern matching would go here
        effect.clone()
    }

    /// Collapse duplicate type variables
    fn collapse_duplicate_vars(effect: &AlgebraicStackEffect) -> AlgebraicStackEffect {
        effect.clone()
    }

    /// Normalize type variable IDs to canonical form
    fn normalize_var_ids(effect: &AlgebraicStackEffect) -> AlgebraicStackEffect {
        use rustc_hash::FxHashMap;

        fn normalize_type(
            t: &AlgebraicType,
            var_map: &mut FxHashMap<usize, usize>,
            next_id: &mut usize
        ) -> AlgebraicType {
            match t {
                AlgebraicType::Var(v) => {
                    let new_id = *var_map.entry(v.id).or_insert_with(|| {
                        let id = *next_id;
                        *next_id += 1;
                        id
                    });
                    AlgebraicType::Var(TypeVariable {
                        id: new_id,
                        name: v.name.clone(),
                    })
                }
                AlgebraicType::Compound { base, operation } => AlgebraicType::Compound {
                    base: Box::new(normalize_type(base, var_map, next_id)),
                    operation: operation.clone(),
                },
                _ => t.clone(),
            }
        }

        let mut var_map: FxHashMap<usize, usize> = FxHashMap::default();
        let mut next_id = 0;

        let inputs: Vec<AlgebraicType> = effect
            .inputs
            .iter()
            .map(|t| normalize_type(t, &mut var_map, &mut next_id))
            .collect();

        let outputs: Vec<AlgebraicType> = effect
            .outputs
            .iter()
            .map(|t| normalize_type(t, &mut var_map, &mut next_id))
            .collect();

        AlgebraicStackEffect { inputs, outputs }
    }
}

/// Simplify a stack effect
pub fn simplify_effect(effect: &AlgebraicStackEffect) -> AlgebraicStackEffect {
    SimplificationRules::simplify(effect)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_var_ids() {
        let effect = AlgebraicStackEffect {
            inputs: vec![
                AlgebraicType::Var(TypeVariable { id: 5, name: Some("a".to_string()) }),
                AlgebraicType::Var(TypeVariable { id: 10, name: Some("b".to_string()) }),
            ],
            outputs: vec![
                AlgebraicType::Var(TypeVariable { id: 10, name: Some("b".to_string()) }),
                AlgebraicType::Var(TypeVariable { id: 5, name: Some("a".to_string()) }),
            ],
        };

        let simplified = SimplificationRules::simplify(&effect);

        // Should normalize IDs to 0, 1
        if let AlgebraicType::Var(v) = &simplified.inputs[0] {
            assert_eq!(v.id, 0);
        } else {
            panic!("Expected variable");
        }

        if let AlgebraicType::Var(v) = &simplified.inputs[1] {
            assert_eq!(v.id, 1);
        } else {
            panic!("Expected variable");
        }
    }
}
