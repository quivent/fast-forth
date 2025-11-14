//! Core inference engine for stack effect analysis

use super::types::{StackEffect, StackType};
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};

/// Result of stack effect inference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceResult {
    pub valid: bool,
    pub inferred_effect: String,
    pub stack_depth_delta: i32,
    pub operations: Vec<String>,
    pub latency_ms: f64,
    pub error: Option<String>,
}

/// Internal inference result
pub struct InferResult {
    pub effect: StackEffect,
    pub stack_depth_delta: i32,
    pub operations: Vec<String>,
}

/// Stack effect inference engine
#[derive(Clone)]
pub struct InferenceEngine {
    builtins: FxHashMap<String, StackEffect>,
}

impl InferenceEngine {
    pub fn new() -> Self {
        let mut builtins = FxHashMap::default();

        // Arithmetic operations (2 inputs, 1 output)
        for op in &["+", "-", "*", "/", "mod"] {
            builtins.insert(
                op.to_string(),
                StackEffect::new(
                    vec![StackType::Int, StackType::Int],
                    vec![StackType::Int],
                ),
            );
        }

        // /mod (2 inputs, 2 outputs)
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
                vec![StackType::Unknown],
                vec![StackType::Unknown, StackType::Unknown],
            ),
        );
        builtins.insert(
            "drop".to_string(),
            StackEffect::new(vec![StackType::Unknown], vec![]),
        );
        builtins.insert(
            "swap".to_string(),
            StackEffect::new(
                vec![StackType::Unknown, StackType::Unknown],
                vec![StackType::Unknown, StackType::Unknown],
            ),
        );
        builtins.insert(
            "over".to_string(),
            StackEffect::new(
                vec![StackType::Unknown, StackType::Unknown],
                vec![
                    StackType::Unknown,
                    StackType::Unknown,
                    StackType::Unknown,
                ],
            ),
        );
        builtins.insert(
            "rot".to_string(),
            StackEffect::new(
                vec![StackType::Unknown, StackType::Unknown, StackType::Unknown],
                vec![
                    StackType::Unknown,
                    StackType::Unknown,
                    StackType::Unknown,
                ],
            ),
        );
        builtins.insert(
            "2dup".to_string(),
            StackEffect::new(
                vec![StackType::Unknown, StackType::Unknown],
                vec![
                    StackType::Unknown,
                    StackType::Unknown,
                    StackType::Unknown,
                    StackType::Unknown,
                ],
            ),
        );
        builtins.insert(
            "2drop".to_string(),
            StackEffect::new(
                vec![StackType::Unknown, StackType::Unknown],
                vec![],
            ),
        );

        // Comparison operations
        for op in &["<", ">", "=", "<=", ">=", "<>"] {
            builtins.insert(
                op.to_string(),
                StackEffect::new(
                    vec![StackType::Int, StackType::Int],
                    vec![StackType::Bool],
                ),
            );
        }

        // Logical operations
        for op in &["and", "or"] {
            builtins.insert(
                op.to_string(),
                StackEffect::new(
                    vec![StackType::Bool, StackType::Bool],
                    vec![StackType::Bool],
                ),
            );
        }
        builtins.insert(
            "not".to_string(),
            StackEffect::new(vec![StackType::Bool], vec![StackType::Bool]),
        );
        builtins.insert(
            "invert".to_string(),
            StackEffect::new(vec![StackType::Int], vec![StackType::Int]),
        );

        // Unary operations
        for op in &["negate", "abs", "1+", "1-", "2*", "2/"] {
            builtins.insert(
                op.to_string(),
                StackEffect::new(vec![StackType::Int], vec![StackType::Int]),
            );
        }

        // Min/max
        for op in &["min", "max"] {
            builtins.insert(
                op.to_string(),
                StackEffect::new(
                    vec![StackType::Int, StackType::Int],
                    vec![StackType::Int],
                ),
            );
        }

        // I/O operations
        builtins.insert(
            ".".to_string(),
            StackEffect::new(vec![StackType::Int], vec![]),
        );
        builtins.insert(
            "emit".to_string(),
            StackEffect::new(vec![StackType::Char], vec![]),
        );
        builtins.insert("cr".to_string(), StackEffect::new(vec![], vec![]));

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

        Self { builtins }
    }

    /// Infer stack effect from code string
    pub fn infer(&self, code: &str) -> Result<InferResult, String> {
        let words = self.tokenize(code);
        let mut operations = Vec::new();
        let mut total_effect = StackEffect::identity();

        for word in words {
            let effect = self.infer_word(&word)?;
            total_effect = total_effect.compose(&effect)?;
            operations.push(word);
        }

        Ok(InferResult {
            stack_depth_delta: total_effect.depth_delta(),
            effect: total_effect,
            operations,
        })
    }

    /// Parse a stack effect string like "( n -- n² )"
    pub fn parse_effect(&self, effect_str: &str) -> Result<StackEffect, String> {
        let trimmed = effect_str.trim();
        if !trimmed.starts_with('(') || !trimmed.ends_with(')') {
            return Err("Stack effect must be in format: ( inputs -- outputs )".to_string());
        }

        let inner = &trimmed[1..trimmed.len() - 1];
        let parts: Vec<&str> = inner.split("--").collect();

        if parts.len() != 2 {
            return Err("Stack effect must contain '--' separator".to_string());
        }

        let inputs = self.parse_type_list(parts[0])?;
        let outputs = self.parse_type_list(parts[1])?;

        Ok(StackEffect::new(inputs, outputs))
    }

    fn parse_type_list(&self, s: &str) -> Result<Vec<StackType>, String> {
        let tokens: Vec<&str> = s.split_whitespace().collect();
        let mut types = Vec::new();

        for token in tokens {
            let ty = match token {
                "n" | "n1" | "n2" | "n3" => StackType::Int,
                "f" | "f1" | "f2" => StackType::Float,
                "b" | "flag" => StackType::Bool,
                "c" | "char" => StackType::Char,
                "a" | "addr" => StackType::Addr,
                "x" => StackType::Unknown,
                _ if token.contains('²') || token.contains('³') => StackType::Int,
                _ => StackType::Var(token.to_string()),
            };
            types.push(ty);
        }

        Ok(types)
    }

    fn tokenize(&self, code: &str) -> Vec<String> {
        code.split_whitespace()
            .map(|s| s.to_string())
            .collect()
    }

    fn infer_word(&self, word: &str) -> Result<StackEffect, String> {
        // Check if it's a number
        if word.parse::<i64>().is_ok() || word.parse::<f64>().is_ok() {
            return Ok(StackEffect::new(vec![], vec![StackType::Int]));
        }

        // Check builtins
        if let Some(effect) = self.builtins.get(word) {
            return Ok(effect.clone());
        }

        // Unknown word - assume it's a constant or variable
        Ok(StackEffect::new(vec![], vec![StackType::Unknown]))
    }
}

impl Default for InferenceEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_infer_dup_multiply() {
        let engine = InferenceEngine::new();
        let result = engine.infer("dup *").unwrap();
        assert_eq!(result.effect.inputs.len(), 1);
        assert_eq!(result.effect.outputs.len(), 1);
        assert_eq!(result.stack_depth_delta, 0);
    }

    #[test]
    fn test_infer_literals() {
        let engine = InferenceEngine::new();
        let result = engine.infer("42 13 +").unwrap();
        assert_eq!(result.effect.inputs.len(), 0);
        assert_eq!(result.effect.outputs.len(), 1);
    }

    #[test]
    fn test_parse_effect() {
        let engine = InferenceEngine::new();
        let effect = engine.parse_effect("( n -- n² )").unwrap();
        assert_eq!(effect.inputs.len(), 1);
        assert_eq!(effect.outputs.len(), 1);
    }

    #[test]
    fn test_stack_underflow_detection() {
        let engine = InferenceEngine::new();
        let result = engine.infer("swap");
        assert!(result.is_err());
    }
}
