//! Semantic Diff for Agents
//!
//! Compare two implementations semantically, showing stack effects,
//! operation changes, and performance differences

pub mod differ;
pub mod analyzer;
pub mod reporter;

pub use differ::{SemanticDiffer, DiffResult};
pub use analyzer::PerformanceAnalyzer;
pub use reporter::{DiffReporter, ReportFormat};

use serde::{Serialize, Deserialize};

/// Semantic difference between two implementations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticDiff {
    pub word_name: String,
    pub stack_effect_changed: bool,
    pub stack_effect_old: String,
    pub stack_effect_new: String,
    pub operations_changed: bool,
    pub operations_old: Vec<String>,
    pub operations_new: Vec<String>,
    pub performance_changed: bool,
    pub performance_old: PerformanceMetrics,
    pub performance_new: PerformanceMetrics,
    pub semantically_equivalent: bool,
    pub recommendation: String,
}

impl SemanticDiff {
    /// Create a new semantic diff
    pub fn new(word_name: String) -> Self {
        Self {
            word_name,
            stack_effect_changed: false,
            stack_effect_old: String::new(),
            stack_effect_new: String::new(),
            operations_changed: false,
            operations_old: Vec::new(),
            operations_new: Vec::new(),
            performance_changed: false,
            performance_old: PerformanceMetrics::default(),
            performance_new: PerformanceMetrics::default(),
            semantically_equivalent: true,
            recommendation: String::new(),
        }
    }

    /// Generate a recommendation based on the diff
    pub fn generate_recommendation(&mut self) {
        if !self.semantically_equivalent {
            self.recommendation = "⚠ Not semantically equivalent - verify correctness before deploying".to_string();
        } else if self.stack_effect_changed {
            self.recommendation = "⚠ Stack effect changed - update documentation and callers".to_string();
        } else if self.performance_changed {
            let perf_ratio = self.performance_new.operation_count as f64
                / self.performance_old.operation_count.max(1) as f64;

            if perf_ratio < 0.9 {
                self.recommendation = format!(
                    "✓ Performance improved ({:.1}x faster) - safe to deploy",
                    1.0 / perf_ratio
                );
            } else if perf_ratio > 1.1 {
                self.recommendation = format!(
                    "⚠ Performance degraded ({:.1}x slower) - consider reverting",
                    perf_ratio
                );
            } else {
                self.recommendation = "✓ Semantically equivalent - safe to deploy".to_string();
            }
        } else if !self.operations_changed {
            self.recommendation = "✓ No changes detected".to_string();
        } else {
            self.recommendation = "✓ Semantically equivalent with refactored code - safe to deploy".to_string();
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct PerformanceMetrics {
    pub operation_count: usize,
    pub stack_depth_max: usize,
    pub complexity_class: String,
}
