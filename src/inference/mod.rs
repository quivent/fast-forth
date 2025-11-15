//! Stack Effect Inference API
//!
//! Pure type checker that infers stack effects without compilation.
//! Designed for sub-millisecond latency (<1ms typical).

pub mod engine;
pub mod types;

pub use engine::{InferenceEngine, InferenceResult};
pub use types::{StackEffect, StackType, OperationInfo};

use serde::{Deserialize, Serialize};
use std::time::Instant;

/// Main API for stack effect inference
#[derive(Clone)]
pub struct InferenceAPI {
    engine: InferenceEngine,
}

impl InferenceAPI {
    /// Create a new inference API instance
    pub fn new() -> Self {
        Self {
            engine: InferenceEngine::new(),
        }
    }

    /// Infer stack effect from Forth code
    ///
    /// # Example
    /// ```
    /// use fastforth::inference::InferenceAPI;
    ///
    /// let api = InferenceAPI::new();
    /// let result = api.infer("dup * swap +").unwrap();
    /// assert!(result.valid);
    /// assert!(result.latency_ms < 1.0);
    /// ```
    pub fn infer(&self, code: &str) -> Result<InferenceResult, String> {
        let start = Instant::now();
        let result = self.engine.infer(code)?;
        let latency_ms = start.elapsed().as_secs_f64() * 1000.0;

        Ok(InferenceResult {
            valid: true,
            inferred_effect: result.effect.to_string(),
            stack_depth_delta: result.stack_depth_delta,
            operations: result.operations,
            latency_ms,
            error: None,
        })
    }

    /// Verify that code matches expected stack effect
    pub fn verify_effect(&self, code: &str, expected_effect: &str) -> Result<VerifyResult, String> {
        let start = Instant::now();
        let result = self.engine.infer(code)?;
        let expected = self.engine.parse_effect(expected_effect)?;

        let matches = result.effect.compatible_with(&expected);
        let latency_ms = start.elapsed().as_secs_f64() * 1000.0;

        Ok(VerifyResult {
            valid: matches,
            inferred: result.effect.to_string(),
            expected: expected.to_string(),
            latency_ms,
            message: if matches {
                "Stack effects match".to_string()
            } else {
                format!(
                    "Stack effect mismatch: expected {}, got {}",
                    expected, result.effect
                )
            },
        })
    }

    /// Verify composition of multiple words
    pub fn compose(&self, words: &[&str]) -> Result<CompositionResult, String> {
        let start = Instant::now();
        let mut total_effect = StackEffect::identity();

        for word in words {
            let result = self.engine.infer(word)?;
            total_effect = total_effect.compose(&result.effect)?;
        }

        let latency_ms = start.elapsed().as_secs_f64() * 1000.0;

        Ok(CompositionResult {
            valid: true,
            effect: total_effect.to_string(),
            words: words.iter().map(|s| s.to_string()).collect(),
            latency_ms,
        })
    }
}

impl Default for InferenceAPI {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of stack effect verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyResult {
    pub valid: bool,
    pub inferred: String,
    pub expected: String,
    pub latency_ms: f64,
    pub message: String,
}

/// Result of composition verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompositionResult {
    pub valid: bool,
    pub effect: String,
    pub words: Vec<String>,
    pub latency_ms: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_infer_basic() {
        let api = InferenceAPI::new();
        let result = api.infer("dup *").unwrap();
        assert!(result.valid);
        assert!(result.latency_ms < 10.0);
    }

    #[test]
    fn test_verify_effect() {
        let api = InferenceAPI::new();
        let result = api.verify_effect("dup *", "( n -- n² )").unwrap();
        assert!(result.valid);
        assert!(result.latency_ms < 10.0);
    }

    #[test]
    fn test_compose() {
        let api = InferenceAPI::new();
        let result = api.compose(&["dup", "*", "swap"]).unwrap();
        assert!(result.valid);
        assert!(result.latency_ms < 1.0);
    }

    #[test]
    fn test_subsecond_performance() {
        let api = InferenceAPI::new();
        let start = Instant::now();
        for _ in 0..1000 {
            let _ = api.infer("dup * swap +");
        }
        let total_ms = start.elapsed().as_secs_f64() * 1000.0;
        assert!(total_ms < 1000.0, "1000 inferences should take <1s");
    }
}
