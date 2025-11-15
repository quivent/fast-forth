//! Stack effect inference engine
//!
//! This module infers stack effects for Forth words based on their definitions.
//! It tracks stack depth and computes the net effect of each operation.

use crate::ast::*;
use crate::error::Result;
use rustc_hash::FxHashMap;
use std::collections::HashMap;

/// Stack effect inference engine
pub struct StackEffectInference {
    /// Known word effects
    builtins: FxHashMap<String, StackEffect>,
    /// User-defined word effects
    user_words: FxHashMap<String, StackEffect>,
}

impl StackEffectInference {
    pub fn new() -> Self {
        let mut builtins = FxHashMap::default();

        // Arithmetic operations
        builtins.insert(
            "+".to_string(),
            StackEffect::new(vec![StackType::Int, StackType::Int], vec![StackType::Int]),
        );
        builtins.insert(
            "-".to_string(),
            StackEffect::new(vec![StackType::Int, StackType::Int], vec![StackType::Int]),
        );
        builtins.insert(
            "*".to_string(),
            StackEffect::new(vec![StackType::Int, StackType::Int], vec![StackType::Int]),
        );
        builtins.insert(
            "/".to_string(),
            StackEffect::new(vec![StackType::Int, StackType::Int], vec![StackType::Int]),
        );
        builtins.insert(
            "mod".to_string(),
            StackEffect::new(vec![StackType::Int, StackType::Int], vec![StackType::Int]),
        );
        builtins.insert(
            "/mod".to_string(),
            StackEffect::new(
                vec![StackType::Int, StackType::Int],
                vec![StackType::Int, StackType::Int],
            ),
        );

        // Stack manipulation
        builtins.insert(
            "dup".to_string(),
            StackEffect::new(
                vec![StackType::Var(TypeVar { id: 0, name: Some("a".to_string()) })],
                vec![
                    StackType::Var(TypeVar { id: 0, name: Some("a".to_string()) }),
                    StackType::Var(TypeVar { id: 0, name: Some("a".to_string()) }),
                ],
            ),
        );
        builtins.insert(
            "drop".to_string(),
            StackEffect::new(
                vec![StackType::Var(TypeVar { id: 0, name: Some("a".to_string()) })],
                vec![],
            ),
        );
        builtins.insert(
            "swap".to_string(),
            StackEffect::new(
                vec![
                    StackType::Var(TypeVar { id: 0, name: Some("a".to_string()) }),
                    StackType::Var(TypeVar { id: 1, name: Some("b".to_string()) }),
                ],
                vec![
                    StackType::Var(TypeVar { id: 1, name: Some("b".to_string()) }),
                    StackType::Var(TypeVar { id: 0, name: Some("a".to_string()) }),
                ],
            ),
        );
        builtins.insert(
            "over".to_string(),
            StackEffect::new(
                vec![
                    StackType::Var(TypeVar { id: 0, name: Some("a".to_string()) }),
                    StackType::Var(TypeVar { id: 1, name: Some("b".to_string()) }),
                ],
                vec![
                    StackType::Var(TypeVar { id: 0, name: Some("a".to_string()) }),
                    StackType::Var(TypeVar { id: 1, name: Some("b".to_string()) }),
                    StackType::Var(TypeVar { id: 0, name: Some("a".to_string()) }),
                ],
            ),
        );
        builtins.insert(
            "rot".to_string(),
            StackEffect::new(
                vec![
                    StackType::Var(TypeVar { id: 0, name: Some("a".to_string()) }),
                    StackType::Var(TypeVar { id: 1, name: Some("b".to_string()) }),
                    StackType::Var(TypeVar { id: 2, name: Some("c".to_string()) }),
                ],
                vec![
                    StackType::Var(TypeVar { id: 1, name: Some("b".to_string()) }),
                    StackType::Var(TypeVar { id: 2, name: Some("c".to_string()) }),
                    StackType::Var(TypeVar { id: 0, name: Some("a".to_string()) }),
                ],
            ),
        );

        // Comparison operations
        builtins.insert(
            "<".to_string(),
            StackEffect::new(vec![StackType::Int, StackType::Int], vec![StackType::Bool]),
        );
        builtins.insert(
            ">".to_string(),
            StackEffect::new(vec![StackType::Int, StackType::Int], vec![StackType::Bool]),
        );
        builtins.insert(
            "=".to_string(),
            StackEffect::new(vec![StackType::Int, StackType::Int], vec![StackType::Bool]),
        );
        builtins.insert(
            "<=".to_string(),
            StackEffect::new(vec![StackType::Int, StackType::Int], vec![StackType::Bool]),
        );
        builtins.insert(
            ">=".to_string(),
            StackEffect::new(vec![StackType::Int, StackType::Int], vec![StackType::Bool]),
        );
        builtins.insert(
            "<>".to_string(),
            StackEffect::new(vec![StackType::Int, StackType::Int], vec![StackType::Bool]),
        );

        // Logical operations
        builtins.insert(
            "and".to_string(),
            StackEffect::new(vec![StackType::Bool, StackType::Bool], vec![StackType::Bool]),
        );
        builtins.insert(
            "or".to_string(),
            StackEffect::new(vec![StackType::Bool, StackType::Bool], vec![StackType::Bool]),
        );
        builtins.insert(
            "not".to_string(),
            StackEffect::new(vec![StackType::Bool], vec![StackType::Bool]),
        );
        builtins.insert(
            "invert".to_string(),
            StackEffect::new(vec![StackType::Int], vec![StackType::Int]),
        );

        // I/O operations
        builtins.insert(
            ".".to_string(),
            StackEffect::new(vec![StackType::Int], vec![]),
        );
        builtins.insert(
            "emit".to_string(),
            StackEffect::new(vec![StackType::Char], vec![]),
        );
        builtins.insert(
            "cr".to_string(),
            StackEffect::new(vec![], vec![]),
        );

        // Memory operations
        builtins.insert(
            "@".to_string(),
            StackEffect::new(vec![StackType::Addr], vec![StackType::Int]),
        );
        builtins.insert(
            "!".to_string(),
            StackEffect::new(vec![StackType::Int, StackType::Addr], vec![]),
        );
        builtins.insert(
            "c@".to_string(),
            StackEffect::new(vec![StackType::Addr], vec![StackType::Char]),
        );
        builtins.insert(
            "c!".to_string(),
            StackEffect::new(vec![StackType::Char, StackType::Addr], vec![]),
        );

        // Additional common words
        builtins.insert(
            "negate".to_string(),
            StackEffect::new(vec![StackType::Int], vec![StackType::Int]),
        );
        builtins.insert(
            "abs".to_string(),
            StackEffect::new(vec![StackType::Int], vec![StackType::Int]),
        );
        builtins.insert(
            "min".to_string(),
            StackEffect::new(vec![StackType::Int, StackType::Int], vec![StackType::Int]),
        );
        builtins.insert(
            "max".to_string(),
            StackEffect::new(vec![StackType::Int, StackType::Int], vec![StackType::Int]),
        );

        Self {
            builtins,
            user_words: FxHashMap::default(),
        }
    }

    /// Infer stack effect for a sequence of words
    pub fn infer_sequence(&self, words: &[Word]) -> Result<StackEffect> {
        let mut stack_depth = 0;
        let mut inputs_needed = 0;

        for word in words {
            let effect = self.infer_word_effect(word)?;

            // Check if we have enough items on the stack
            if stack_depth < effect.inputs.len() {
                inputs_needed += effect.inputs.len() - stack_depth;
                stack_depth = 0;
            } else {
                stack_depth -= effect.inputs.len();
            }

            // Add outputs to stack
            stack_depth += effect.outputs.len();
        }

        let outputs_produced = stack_depth;

        Ok(StackEffect::new(
            vec![StackType::Unknown; inputs_needed],
            vec![StackType::Unknown; outputs_produced],
        ))
    }

    /// Infer stack effect for a single word
    fn infer_word_effect(&self, word: &Word) -> Result<StackEffect> {
        match word {
            Word::IntLiteral(_) | Word::FloatLiteral(_) | Word::StringLiteral(_) => {
                // Literals push one value
                Ok(StackEffect::new(vec![], vec![StackType::Unknown]))
            }
            Word::WordRef { name, .. } => {
                // Look up word effect
                if let Some(effect) = self.builtins.get(name) {
                    Ok(effect.clone())
                } else if let Some(effect) = self.user_words.get(name) {
                    Ok(effect.clone())
                } else {
                    // Unknown word - assume minimal effect
                    Ok(StackEffect::new(vec![], vec![]))
                }
            }
            Word::If { then_branch, else_branch } => {
                // IF consumes a boolean, branches should have same effect
                let then_effect = self.infer_sequence(then_branch)?;

                let else_effect = if let Some(else_words) = else_branch {
                    self.infer_sequence(else_words)?
                } else {
                    StackEffect::new(vec![], vec![])
                };

                // Both branches should produce same net effect
                // Take the maximum inputs needed by either branch
                let max_inputs = then_effect.inputs.len().max(else_effect.inputs.len());
                let inputs_vec: Vec<StackType> = (0..max_inputs).map(|_| StackType::Unknown).collect();

                // Both branches should produce the same number of outputs
                let output_count = then_effect.outputs.len();
                let outputs_vec: Vec<StackType> = (0..output_count).map(|_| StackType::Unknown).collect();

                // Add the boolean consumed by IF
                let mut inputs = vec![StackType::Bool];
                inputs.extend(inputs_vec);

                Ok(StackEffect::new(inputs, outputs_vec))
            }
            Word::BeginUntil { body } => {
                // BEGIN...UNTIL loops consume a boolean at the end
                let body_effect = self.infer_sequence(body)?;
                let mut inputs = body_effect.inputs.clone();
                inputs.push(StackType::Bool);

                Ok(StackEffect::new(inputs, body_effect.outputs))
            }
            Word::BeginWhileRepeat { condition, body } => {
                // BEGIN...WHILE...REPEAT
                let cond_effect = self.infer_sequence(condition)?;
                let body_effect = self.infer_sequence(body)?;

                let mut inputs = cond_effect.inputs.clone();
                inputs.extend(body_effect.inputs);
                let outputs = body_effect.outputs;

                Ok(StackEffect::new(inputs, outputs))
            }
            Word::DoLoop { body, .. } => {
                // DO...LOOP requires two loop bounds
                let body_effect = self.infer_sequence(body)?;
                let mut inputs = vec![StackType::Int, StackType::Int];
                inputs.extend(body_effect.inputs);

                Ok(StackEffect::new(inputs, body_effect.outputs))
            }
            Word::Variable { .. } | Word::Constant { .. } => {
                // Variable/constant push address or value
                Ok(StackEffect::new(vec![], vec![StackType::Addr]))
            }
            Word::Comment(_) => {
                // Comments have no effect
                Ok(StackEffect::new(vec![], vec![]))
            }
        }
    }

    /// Add a user-defined word and infer its effect
    pub fn add_definition(&mut self, def: &Definition) -> Result<()> {
        let effect = if let Some(declared_effect) = &def.stack_effect {
            // Use declared effect if available
            declared_effect.clone()
        } else {
            // Infer from body
            self.infer_sequence(&def.body)?
        };

        self.user_words.insert(def.name.clone(), effect);
        Ok(())
    }

    /// Get the stack effect for a word
    pub fn get_effect(&self, name: &str) -> Option<&StackEffect> {
        self.builtins.get(name).or_else(|| self.user_words.get(name))
    }

    /// Analyze a complete program and infer all effects
    pub fn analyze_program(&mut self, program: &Program) -> Result<HashMap<String, StackEffect>> {
        let mut effects = HashMap::new();

        for def in &program.definitions {
            self.add_definition(def)?;
            if let Some(effect) = self.user_words.get(&def.name) {
                effects.insert(def.name.clone(), effect.clone());
            }
        }

        Ok(effects)
    }
}

impl Default for StackEffectInference {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_program;

    #[test]
    fn test_infer_arithmetic() {
        let inference = StackEffectInference::new();
        let words = vec![
            Word::IntLiteral(2),
            Word::IntLiteral(3),
            Word::WordRef {
                name: "+".to_string(),
                location: SourceLocation::default(),
            },
        ];

        let effect = inference.infer_sequence(&words).unwrap();
        assert_eq!(effect.inputs.len(), 0);
        assert_eq!(effect.outputs.len(), 1);
    }

    #[test]
    fn test_infer_dup() {
        let inference = StackEffectInference::new();
        let words = vec![Word::WordRef {
            name: "dup".to_string(),
            location: SourceLocation::default(),
        }];

        let effect = inference.infer_sequence(&words).unwrap();
        assert_eq!(effect.inputs.len(), 1);
        assert_eq!(effect.outputs.len(), 2);
    }

    #[test]
    fn test_analyze_program() {
        let program = parse_program(": double ( n -- n*2 ) 2 * ;").unwrap();
        let mut inference = StackEffectInference::new();
        let effects = inference.analyze_program(&program).unwrap();

        assert!(effects.contains_key("double"));
        let effect = &effects["double"];
        assert_eq!(effect.inputs.len(), 1);
        assert_eq!(effect.outputs.len(), 1);
    }

    #[test]
    fn test_stack_manipulation() {
        let inference = StackEffectInference::new();
        let words = vec![
            Word::IntLiteral(1),
            Word::IntLiteral(2),
            Word::WordRef {
                name: "swap".to_string(),
                location: SourceLocation::default(),
            },
        ];

        let effect = inference.infer_sequence(&words).unwrap();
        assert_eq!(effect.outputs.len(), 2);
    }
}
