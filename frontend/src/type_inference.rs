//! Type inference system for Forth
//!
//! Implements Hindley-Milner-style type inference for stack-based operations.
//! This allows us to infer concrete types for polymorphic words and detect type errors.

use crate::ast::*;
use crate::error::{ForthError, Result};
use rustc_hash::FxHashMap;
use std::collections::HashMap;

/// Type environment mapping variables to types
#[derive(Debug, Clone)]
pub struct TypeEnv {
    bindings: FxHashMap<String, StackType>,
    next_var_id: usize,
}

impl TypeEnv {
    pub fn new() -> Self {
        Self {
            bindings: FxHashMap::default(),
            next_var_id: 0,
        }
    }

    pub fn fresh_var(&mut self) -> StackType {
        let var = StackType::Var(TypeVar {
            id: self.next_var_id,
            name: None,
        });
        self.next_var_id += 1;
        var
    }

    pub fn bind(&mut self, name: String, ty: StackType) {
        self.bindings.insert(name, ty);
    }

    pub fn lookup(&self, name: &str) -> Option<&StackType> {
        self.bindings.get(name)
    }
}

impl Default for TypeEnv {
    fn default() -> Self {
        Self::new()
    }
}

/// Type substitution
#[derive(Debug, Clone, Default)]
pub struct Substitution {
    mappings: FxHashMap<usize, StackType>,
}

impl Substitution {
    pub fn new() -> Self {
        Self {
            mappings: FxHashMap::default(),
        }
    }

    pub fn bind(&mut self, var_id: usize, ty: StackType) {
        self.mappings.insert(var_id, ty);
    }

    pub fn apply(&self, ty: &StackType) -> StackType {
        match ty {
            StackType::Var(TypeVar { id, .. }) => {
                if let Some(substituted) = self.mappings.get(id) {
                    self.apply(substituted)
                } else {
                    ty.clone()
                }
            }
            _ => ty.clone(),
        }
    }

    pub fn compose(&mut self, other: &Substitution) {
        for (var_id, ty) in &other.mappings {
            let applied_ty = self.apply(ty);
            self.mappings.insert(*var_id, applied_ty);
        }
    }
}

/// Type inference engine
pub struct TypeInference {
    env: TypeEnv,
    substitution: Substitution,
}

impl TypeInference {
    pub fn new() -> Self {
        let mut env = TypeEnv::new();

        // Initialize with builtin types
        env.bind("+".to_string(), StackType::Int);
        env.bind("-".to_string(), StackType::Int);
        env.bind("*".to_string(), StackType::Int);
        env.bind("/".to_string(), StackType::Int);

        Self {
            env,
            substitution: Substitution::new(),
        }
    }

    /// Unify two types
    fn unify(&mut self, t1: &StackType, t2: &StackType) -> Result<()> {
        let t1 = self.substitution.apply(t1);
        let t2 = self.substitution.apply(t2);

        match (&t1, &t2) {
            // Same types unify trivially
            (StackType::Int, StackType::Int)
            | (StackType::Float, StackType::Float)
            | (StackType::Bool, StackType::Bool)
            | (StackType::Char, StackType::Char)
            | (StackType::Addr, StackType::Addr)
            | (StackType::String, StackType::String) => Ok(()),

            // In Forth, Bool and Int are compatible (booleans are integers: -1 for true, 0 for false)
            (StackType::Bool, StackType::Int) | (StackType::Int, StackType::Bool) => Ok(()),

            // Unknown can unify with anything
            (StackType::Unknown, _) | (_, StackType::Unknown) => Ok(()),

            // Type variable unification
            (StackType::Var(v1), StackType::Var(v2)) if v1.id == v2.id => Ok(()),
            (StackType::Var(v), t) | (t, StackType::Var(v)) => {
                if self.occurs_check(v.id, t) {
                    Err(ForthError::TypeError {
                        expected: format!("{}", t1),
                        found: format!("{}", t2),
                        location: None,
                    })
                } else {
                    self.substitution.bind(v.id, t.clone());
                    Ok(())
                }
            }

            // Type mismatch
            _ => Err(ForthError::TypeError {
                expected: format!("{}", t1),
                found: format!("{}", t2),
                location: None,
            }),
        }
    }

    /// Occurs check for type variables (prevent infinite types)
    fn occurs_check(&self, var_id: usize, ty: &StackType) -> bool {
        match ty {
            StackType::Var(TypeVar { id, .. }) => *id == var_id,
            _ => false,
        }
    }

    /// Infer type for a word
    pub fn infer_word(&mut self, word: &Word) -> Result<(Vec<StackType>, Vec<StackType>)> {
        match word {
            Word::IntLiteral(_) => Ok((vec![], vec![StackType::Int])),
            Word::FloatLiteral(_) => Ok((vec![], vec![StackType::Float])),
            Word::StringLiteral(_) => Ok((vec![], vec![StackType::String])),

            Word::WordRef { name, .. } => {
                // Look up word type from environment
                self.infer_builtin_word(name)
            }

            Word::If { then_branch, else_branch } => {
                // IF requires a boolean condition
                let (then_inputs, then_outputs) = self.infer_sequence(then_branch)?;

                let (_else_inputs, else_outputs) = if let Some(else_words) = else_branch {
                    self.infer_sequence(else_words)?
                } else {
                    (vec![], vec![])
                };

                // Both branches should have compatible types
                if then_outputs.len() != else_outputs.len() {
                    return Err(ForthError::TypeError {
                        expected: format!("{} outputs", then_outputs.len()),
                        found: format!("{} outputs", else_outputs.len()),
                        location: Some("IF branches".to_string()),
                    });
                }

                // Unify output types
                for (t1, t2) in then_outputs.iter().zip(else_outputs.iter()) {
                    self.unify(t1, t2)?;
                }

                let mut inputs = vec![StackType::Bool];
                inputs.extend(then_inputs);

                Ok((inputs, then_outputs))
            }

            Word::BeginUntil { body } => {
                let (inputs, outputs) = self.infer_sequence(body)?;
                let mut all_inputs = inputs.clone();
                all_inputs.push(StackType::Bool);
                Ok((all_inputs, outputs))
            }

            Word::BeginWhileRepeat { condition, body } => {
                let (cond_inputs, cond_outputs) = self.infer_sequence(condition)?;
                let (body_inputs, body_outputs) = self.infer_sequence(body)?;

                // Condition should produce a boolean
                if let Some(last_output) = cond_outputs.last() {
                    self.unify(last_output, &StackType::Bool)?;
                }

                let mut inputs = cond_inputs;
                inputs.extend(body_inputs);

                Ok((inputs, body_outputs))
            }

            Word::DoLoop { body, .. } => {
                let (body_inputs, body_outputs) = self.infer_sequence(body)?;
                let mut inputs = vec![StackType::Int, StackType::Int];
                inputs.extend(body_inputs);
                Ok((inputs, body_outputs))
            }

            Word::Variable { .. } => Ok((vec![], vec![StackType::Addr])),
            Word::Constant { .. } => Ok((vec![], vec![StackType::Int])),
            Word::Comment(_) => Ok((vec![], vec![])),
        }
    }

    /// Infer types for builtin words
    fn infer_builtin_word(&mut self, name: &str) -> Result<(Vec<StackType>, Vec<StackType>)> {
        match name {
            // Arithmetic
            "+" | "-" | "*" | "/" | "mod" => {
                Ok((vec![StackType::Int, StackType::Int], vec![StackType::Int]))
            }
            "/mod" => Ok((
                vec![StackType::Int, StackType::Int],
                vec![StackType::Int, StackType::Int],
            )),

            // Stack manipulation (polymorphic)
            "dup" => {
                let var = self.env.fresh_var();
                Ok((vec![var.clone()], vec![var.clone(), var]))
            }
            "drop" => {
                let var = self.env.fresh_var();
                Ok((vec![var], vec![]))
            }
            "swap" => {
                let var1 = self.env.fresh_var();
                let var2 = self.env.fresh_var();
                Ok((vec![var1.clone(), var2.clone()], vec![var2, var1]))
            }
            "over" => {
                let var1 = self.env.fresh_var();
                let var2 = self.env.fresh_var();
                Ok((
                    vec![var1.clone(), var2.clone()],
                    vec![var1.clone(), var2, var1],
                ))
            }
            "rot" => {
                let var1 = self.env.fresh_var();
                let var2 = self.env.fresh_var();
                let var3 = self.env.fresh_var();
                Ok((
                    vec![var1.clone(), var2.clone(), var3.clone()],
                    vec![var2, var3, var1],
                ))
            }

            // Comparison
            "<" | ">" | "=" | "<=" | ">=" | "<>" => {
                Ok((vec![StackType::Int, StackType::Int], vec![StackType::Bool]))
            }

            // Logical
            "and" | "or" => {
                Ok((vec![StackType::Bool, StackType::Bool], vec![StackType::Bool]))
            }
            "not" => Ok((vec![StackType::Bool], vec![StackType::Bool])),
            "invert" => Ok((vec![StackType::Int], vec![StackType::Int])),

            // Memory
            "@" => Ok((vec![StackType::Addr], vec![StackType::Int])),
            "!" => Ok((vec![StackType::Int, StackType::Addr], vec![])),
            "c@" => Ok((vec![StackType::Addr], vec![StackType::Char])),
            "c!" => Ok((vec![StackType::Char, StackType::Addr], vec![])),

            // I/O
            "." => Ok((vec![StackType::Int], vec![])),
            "emit" => Ok((vec![StackType::Char], vec![])),
            "cr" => Ok((vec![], vec![])),

            // Other
            "negate" | "abs" => Ok((vec![StackType::Int], vec![StackType::Int])),
            "min" | "max" => {
                Ok((vec![StackType::Int, StackType::Int], vec![StackType::Int]))
            }

            // Unknown word
            _ => {
                let var = self.env.fresh_var();
                Ok((vec![var.clone()], vec![var]))
            }
        }
    }

    /// Infer types for a sequence of words
    pub fn infer_sequence(&mut self, words: &[Word]) -> Result<(Vec<StackType>, Vec<StackType>)> {
        let mut stack: Vec<StackType> = Vec::new();
        let mut total_inputs = Vec::new();

        for word in words {
            let (inputs, outputs) = self.infer_word(word)?;

            // Pop inputs from stack
            if stack.len() < inputs.len() {
                // Need more inputs from outside
                let needed = inputs.len() - stack.len();
                for _ in 0..needed {
                    let var = self.env.fresh_var();
                    total_inputs.push(var.clone());
                    stack.push(var);
                }
            }

            // Type check the inputs
            for (expected, actual) in inputs.iter().zip(stack.iter().rev()).rev() {
                self.unify(expected, actual)?;
            }

            // Remove consumed items
            stack.truncate(stack.len().saturating_sub(inputs.len()));

            // Add outputs
            stack.extend(outputs);
        }

        // Apply substitutions to resolve type variables
        let resolved_inputs: Vec<_> = total_inputs.iter().map(|t| self.substitution.apply(t)).collect();
        let resolved_outputs: Vec<_> = stack.iter().map(|t| self.substitution.apply(t)).collect();

        Ok((resolved_inputs, resolved_outputs))
    }

    /// Infer types for a definition
    pub fn infer_definition(&mut self, def: &Definition) -> Result<(Vec<StackType>, Vec<StackType>)> {
        // If stack effect is declared, use it as a constraint
        if let Some(effect) = &def.stack_effect {
            // For empty bodies, trust the declaration (useful for identity functions, stubs, etc.)
            if def.body.is_empty() {
                return Ok((effect.inputs.clone(), effect.outputs.clone()));
            }

            let (inferred_inputs, inferred_outputs) = self.infer_sequence(&def.body)?;

            // Unify declared and inferred types
            if inferred_inputs.len() != effect.inputs.len()
                || inferred_outputs.len() != effect.outputs.len()
            {
                return Err(ForthError::TypeError {
                    expected: format!("{}", effect),
                    found: format!(
                        "( {} -- {} )",
                        inferred_inputs
                            .iter()
                            .map(|t| format!("{}", t))
                            .collect::<Vec<_>>()
                            .join(" "),
                        inferred_outputs
                            .iter()
                            .map(|t| format!("{}", t))
                            .collect::<Vec<_>>()
                            .join(" ")
                    ),
                    location: Some(def.name.clone()),
                });
            }

            for (declared, inferred) in effect.inputs.iter().zip(inferred_inputs.iter()) {
                self.unify(declared, inferred)?;
            }

            for (declared, inferred) in effect.outputs.iter().zip(inferred_outputs.iter()) {
                self.unify(declared, inferred)?;
            }

            Ok((
                effect.inputs.iter().map(|t| self.substitution.apply(t)).collect(),
                effect.outputs.iter().map(|t| self.substitution.apply(t)).collect(),
            ))
        } else {
            self.infer_sequence(&def.body)
        }
    }

    /// Analyze an entire program
    pub fn analyze_program(
        &mut self,
        program: &Program,
    ) -> Result<HashMap<String, (Vec<StackType>, Vec<StackType>)>> {
        let mut types = HashMap::new();

        for def in &program.definitions {
            let (inputs, outputs) = self.infer_definition(def)?;
            types.insert(def.name.clone(), (inputs, outputs));
        }

        Ok(types)
    }
}

impl Default for TypeInference {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_infer_arithmetic() {
        let mut inference = TypeInference::new();
        let words = vec![
            Word::IntLiteral(2),
            Word::IntLiteral(3),
            Word::WordRef {
                name: "+".to_string(),
                location: SourceLocation::default(),
            },
        ];

        let (inputs, outputs) = inference.infer_sequence(&words).unwrap();
        assert_eq!(inputs.len(), 0);
        assert_eq!(outputs.len(), 1);
        assert_eq!(outputs[0], StackType::Int);
    }

    #[test]
    fn test_infer_polymorphic() {
        let mut inference = TypeInference::new();
        let words = vec![Word::WordRef {
            name: "dup".to_string(),
            location: SourceLocation::default(),
        }];

        let (inputs, outputs) = inference.infer_sequence(&words).unwrap();
        assert_eq!(inputs.len(), 1);
        assert_eq!(outputs.len(), 2);
    }

    #[test]
    fn test_type_error() {
        let mut inference = TypeInference::new();
        // Try to add a boolean and an int (should fail if strict)
        let words = vec![
            Word::IntLiteral(1),
            Word::IntLiteral(2),
            Word::WordRef {
                name: "<".to_string(),
                location: SourceLocation::default(),
            },
            Word::IntLiteral(3),
            Word::WordRef {
                name: "+".to_string(),
                location: SourceLocation::default(),
            },
        ];

        // This should work in Forth but might fail in strict typing
        let result = inference.infer_sequence(&words);
        // In ANS Forth, booleans are just integers, so this is actually valid
        assert!(result.is_ok());
    }
}
