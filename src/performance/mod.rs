//! Performance modeling and benchmark-driven generation
//!
//! This module provides performance targeting, modeling, and prediction
//! to enable agents to generate optimized code that meets performance targets.

pub mod modeling;
pub mod metrics;
pub mod benchmarks;

pub use modeling::{PerformanceModel, PerformancePrediction, PerformanceTarget};
pub use metrics::{PerformanceMetrics, ExecutionProfile};
pub use benchmarks::{BenchmarkSuite, BenchmarkResult};

use crate::error::{CompileError, Result};
use fastforth_optimizer::ForthIR;

/// Performance-driven code generation
pub struct PerformanceOptimizer {
    model: PerformanceModel,
    target: Option<PerformanceTarget>,
}

impl PerformanceOptimizer {
    /// Create a new performance optimizer
    pub fn new() -> Self {
        Self {
            model: PerformanceModel::new(),
            target: None,
        }
    }

    /// Set a performance target
    pub fn with_target(mut self, target: PerformanceTarget) -> Self {
        self.target = Some(target);
        self
    }

    /// Predict performance for the given IR
    pub fn predict(&self, ir: &ForthIR) -> Result<PerformancePrediction> {
        self.model.predict(ir)
    }

    /// Check if IR meets the performance target
    pub fn meets_target(&self, ir: &ForthIR) -> Result<bool> {
        if let Some(target) = &self.target {
            let prediction = self.predict(ir)?;
            Ok(prediction.meets_target(target))
        } else {
            Ok(true) // No target set, always passes
        }
    }

    /// Try alternative patterns if target is not met
    pub fn suggest_alternatives(&self, ir: &ForthIR) -> Result<Vec<String>> {
        let prediction = self.predict(ir)?;

        if let Some(target) = &self.target {
            if !prediction.meets_target(target) {
                return Ok(vec![
                    "SIMD_LOOP_005".to_string(),
                    "TAIL_RECURSIVE_008".to_string(),
                    "LOOP_UNROLL_004".to_string(),
                ]);
            }
        }

        Ok(vec![])
    }
}

impl Default for PerformanceOptimizer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_performance_optimizer_creation() {
        let optimizer = PerformanceOptimizer::new();
        assert!(optimizer.target.is_none());
    }

    #[test]
    fn test_performance_target_setting() {
        let target = PerformanceTarget::new(0.9);
        let optimizer = PerformanceOptimizer::new().with_target(target);
        assert!(optimizer.target.is_some());
    }
}
