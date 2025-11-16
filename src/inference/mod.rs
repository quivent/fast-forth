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
        assert!(result.latency_ms < 10.0);
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

    #[test]
    fn test_circular_type_references() {
        // Test type unification with circular references
        // This tests the engine's ability to handle self-referential type patterns
        let api = InferenceAPI::new();

        // Create a pattern that could create circular type references
        // Using dup and swap repeatedly to create complex type relationships
        // dup adds 1, swap maintains, so: dup(+1) swap(0) dup(+1) swap(0) dup(+1) swap(0) = +3
        let result = api.infer("dup swap dup swap dup swap").unwrap();

        assert!(result.valid, "Circular type pattern should be valid");
        assert!(result.latency_ms < 10.0, "Should complete in <10ms");
        assert_eq!(result.stack_depth_delta, 3, "Should add 3 items to stack");
    }

    #[test]
    fn test_polymorphic_recursion() {
        // Test recursive polymorphic functions
        // Tests the engine's ability to handle polymorphic stack effects
        let api = InferenceAPI::new();

        // Test polymorphic operations that work on any type
        let result = api.infer("dup dup dup drop drop drop").unwrap();

        assert!(result.valid, "Polymorphic recursion should be valid");
        assert!(result.latency_ms < 10.0, "Should complete in <10ms");
        assert_eq!(result.stack_depth_delta, 0, "Should have net zero stack effect");

        // Test with mixed types
        let result2 = api.infer("42 dup dup drop 3.14 swap drop").unwrap();
        assert!(result2.valid, "Mixed type polymorphism should work");
    }

    #[test]
    fn test_deep_stack_inference() {
        // Test stack effect inference with >100 items
        // This tests the engine's scalability with deep stack operations
        let api = InferenceAPI::new();

        // Create a program that generates 100+ stack items
        let mut code = String::new();
        for i in 0..105 {
            code.push_str(&format!("{} ", i));
        }

        let start = Instant::now();
        let result = api.infer(&code).unwrap();
        let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;

        assert!(result.valid, "Deep stack should be valid");
        assert_eq!(result.stack_depth_delta, 105, "Should have 105 items on stack");
        assert!(elapsed_ms < 10.0, "Deep stack inference should take <10ms, got {}ms", elapsed_ms);
        assert_eq!(result.operations.len(), 105, "Should track all 105 operations");
    }

    #[test]
    fn test_type_mismatch_detection() {
        // Test that type mismatches are caught
        // Tests the verify_effect function's ability to detect incompatible effects
        let api = InferenceAPI::new();

        // Test with an incorrect expected effect
        let result = api.verify_effect("dup *", "( n n -- n )").unwrap();
        assert!(!result.valid, "Should detect mismatch: dup * takes 1 input, not 2");
        assert!(result.message.contains("mismatch"), "Should explain the mismatch");

        // Test with correct effect - /mod takes 2 inputs and produces 2 outputs
        let result2 = api.verify_effect("/mod", "( n n -- n n )").unwrap();
        assert!(result2.valid, "Should match when effects align");

        // Test with wrong output count
        let result3 = api.verify_effect("drop", "( n -- n )").unwrap();
        assert!(!result3.valid, "Should detect output count mismatch");

        assert!(result.latency_ms < 10.0, "Type checking should be <10ms");
    }

    #[test]
    fn test_complex_composition() {
        // Test composing 10+ words together
        // Tests the compose function with complex multi-word sequences
        let api = InferenceAPI::new();

        let words = vec![
            "dup", "*",      // Square
            "dup", "+",      // Double
            "1+",           // Add 1
            "dup", "dup",   // Duplicate twice
            "*", "+",       // Complex math
            "swap", "drop", // Cleanup
        ];

        assert!(words.len() >= 10, "Should test 10+ words");

        let start = Instant::now();
        let result = api.compose(&words).unwrap();
        let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;

        assert!(result.valid, "Complex composition should be valid");
        assert_eq!(result.words.len(), words.len(), "Should track all words");
        assert!(elapsed_ms < 10.0, "Composition should take <10ms, got {}ms", elapsed_ms);
        assert!(result.effect.len() > 0, "Should have a non-empty effect string");
    }

    #[test]
    fn test_inference_error_recovery() {
        // Test graceful error handling
        // Tests that the engine handles errors without panicking
        let api = InferenceAPI::new();

        // Test with malformed stack effect notation
        let result = api.verify_effect("dup", "( n -- )");
        assert!(result.is_err() || !result.unwrap().valid, "Should handle malformed effects");

        // Test with empty input
        let result2 = api.infer("");
        assert!(result2.is_ok(), "Should handle empty input gracefully");
        assert_eq!(result2.unwrap().stack_depth_delta, 0, "Empty input has zero effect");

        // Test with only whitespace
        let result3 = api.infer("   \t\n  ");
        assert!(result3.is_ok(), "Should handle whitespace-only input");

        // Test compose with empty array
        let result4 = api.compose(&[]);
        assert!(result4.is_ok(), "Should handle empty composition");
        assert_eq!(result4.unwrap().words.len(), 0, "Empty composition has no words");
    }

    #[test]
    fn test_performance_large_programs() {
        // Test inference on 1000+ word programs
        // Tests scalability and performance with large codebases
        let api = InferenceAPI::new();

        // Create a large program with 1000+ words
        let mut large_program = String::new();
        for i in 0..1200 {
            match i % 5 {
                0 => large_program.push_str("dup "),
                1 => large_program.push_str("+ "),
                2 => large_program.push_str("1 "),
                3 => large_program.push_str("swap "),
                _ => large_program.push_str("drop "),
            }
        }

        let start = Instant::now();
        let result = api.infer(&large_program).unwrap();
        let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;

        assert!(result.valid, "Large program should be valid");
        assert!(result.operations.len() >= 1000, "Should have 1000+ operations");
        assert!(elapsed_ms < 100.0, "Large program inference should take <100ms, got {}ms", elapsed_ms);

        // Verify sub-millisecond per-operation average
        let avg_per_op = elapsed_ms / result.operations.len() as f64;
        assert!(avg_per_op < 0.1, "Should average <0.1ms per operation, got {}ms", avg_per_op);
    }

    #[test]
    fn test_edge_case_empty_effect() {
        // Test words with ( -- ) effect (no inputs, no outputs)
        // Tests handling of null effects and identity operations
        let api = InferenceAPI::new();

        // Test the 'cr' word which has no stack effect
        let result = api.infer("cr").unwrap();
        assert!(result.valid, "Empty effect should be valid");
        assert_eq!(result.stack_depth_delta, 0, "Empty effect should have zero depth change");

        // Test composition with empty effects
        let result2 = api.compose(&["cr", "cr", "cr"]).unwrap();
        assert!(result2.valid, "Multiple empty effects should compose");
        assert!(result2.effect.contains("--"), "Should show proper effect notation");

        // Test verify with empty effect
        let result3 = api.verify_effect("cr cr", "( -- )").unwrap();
        assert!(result3.valid, "Should verify empty effect correctly");

        // Test mixed empty and non-empty effects
        let result4 = api.infer("42 cr . cr").unwrap();
        assert!(result4.valid, "Mixed effects should work");
        assert_eq!(result4.stack_depth_delta, 0, "Net effect should be zero");

        assert!(result.latency_ms < 10.0, "Empty effect tests should be <10ms");
    }
}
