//! Performance metrics collection and reporting
//!
//! Provides runtime performance metrics for actual execution profiling

use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

/// Runtime performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// Execution time
    pub execution_time: Duration,

    /// Number of operations executed
    pub operations_executed: u64,

    /// Operations per second
    pub ops_per_second: f64,

    /// Peak memory usage in bytes
    pub peak_memory_usage: usize,

    /// Average memory usage in bytes
    pub average_memory_usage: usize,

    /// Cache hit rate (0.0-1.0)
    pub cache_hit_rate: f64,

    /// Branch misprediction rate (0.0-1.0)
    pub branch_misprediction_rate: f64,
}

impl PerformanceMetrics {
    /// Create new metrics with execution time
    pub fn new(execution_time: Duration) -> Self {
        Self {
            execution_time,
            operations_executed: 0,
            ops_per_second: 0.0,
            peak_memory_usage: 0,
            average_memory_usage: 0,
            cache_hit_rate: 0.0,
            branch_misprediction_rate: 0.0,
        }
    }

    /// Set operations executed and calculate ops per second
    pub fn with_operations(mut self, ops: u64) -> Self {
        self.operations_executed = ops;
        if !self.execution_time.is_zero() {
            self.ops_per_second = ops as f64 / self.execution_time.as_secs_f64();
        }
        self
    }

    /// Set memory usage statistics
    pub fn with_memory_usage(mut self, peak: usize, average: usize) -> Self {
        self.peak_memory_usage = peak;
        self.average_memory_usage = average;
        self
    }

    /// Set cache performance metrics
    pub fn with_cache_metrics(mut self, hit_rate: f64, misprediction_rate: f64) -> Self {
        self.cache_hit_rate = hit_rate;
        self.branch_misprediction_rate = misprediction_rate;
        self
    }

    /// Get a formatted summary
    pub fn summary(&self) -> String {
        format!(
            "Execution: {:.2}ms | Ops: {} ({:.0} ops/sec) | Memory: {} bytes peak | Cache hit: {:.1}%",
            self.execution_time.as_secs_f64() * 1000.0,
            self.operations_executed,
            self.ops_per_second,
            self.peak_memory_usage,
            self.cache_hit_rate * 100.0
        )
    }
}

/// Execution profiler for runtime metrics collection
pub struct ExecutionProfile {
    start_time: Option<Instant>,
    metrics: PerformanceMetrics,
    operation_count: u64,
}

impl ExecutionProfile {
    /// Create a new profiler
    pub fn new() -> Self {
        Self {
            start_time: None,
            metrics: PerformanceMetrics::new(Duration::ZERO),
            operation_count: 0,
        }
    }

    /// Start profiling
    pub fn start(&mut self) {
        self.start_time = Some(Instant::now());
        self.operation_count = 0;
    }

    /// Record an operation
    pub fn record_operation(&mut self) {
        self.operation_count += 1;
    }

    /// Record multiple operations
    pub fn record_operations(&mut self, count: u64) {
        self.operation_count += count;
    }

    /// Stop profiling and return metrics
    pub fn stop(&mut self) -> PerformanceMetrics {
        let execution_time = if let Some(start) = self.start_time {
            start.elapsed()
        } else {
            Duration::ZERO
        };

        self.metrics = PerformanceMetrics::new(execution_time)
            .with_operations(self.operation_count);

        self.start_time = None;
        self.metrics.clone()
    }

    /// Get current metrics without stopping
    pub fn current_metrics(&self) -> PerformanceMetrics {
        let execution_time = if let Some(start) = self.start_time {
            start.elapsed()
        } else {
            Duration::ZERO
        };

        PerformanceMetrics::new(execution_time)
            .with_operations(self.operation_count)
    }
}

impl Default for ExecutionProfile {
    fn default() -> Self {
        Self::new()
    }
}

/// Comparison between predicted and actual performance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceComparison {
    /// Predicted execution time
    pub predicted_time_ms: u64,

    /// Actual execution time
    pub actual_time_ms: u64,

    /// Accuracy percentage (0.0-1.0)
    pub accuracy: f64,

    /// Prediction error
    pub error_ms: i64,
}

impl PerformanceComparison {
    /// Create a comparison
    pub fn new(predicted_ms: u64, actual_ms: u64) -> Self {
        let error_ms = actual_ms as i64 - predicted_ms as i64;
        let accuracy = 1.0 - (error_ms.abs() as f64 / predicted_ms.max(1) as f64).min(1.0);

        Self {
            predicted_time_ms: predicted_ms,
            actual_time_ms: actual_ms,
            accuracy,
            error_ms,
        }
    }

    /// Check if prediction is within acceptable range (within 20%)
    pub fn is_accurate(&self) -> bool {
        self.accuracy >= 0.8
    }

    /// Get a formatted summary
    pub fn summary(&self) -> String {
        format!(
            "Predicted: {}ms | Actual: {}ms | Accuracy: {:.1}% | Error: {}ms",
            self.predicted_time_ms,
            self.actual_time_ms,
            self.accuracy * 100.0,
            self.error_ms
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_performance_metrics_creation() {
        let metrics = PerformanceMetrics::new(Duration::from_millis(100));
        assert_eq!(metrics.execution_time.as_millis(), 100);
    }

    #[test]
    fn test_performance_metrics_with_operations() {
        let metrics = PerformanceMetrics::new(Duration::from_secs(1))
            .with_operations(1000);

        assert_eq!(metrics.operations_executed, 1000);
        assert_eq!(metrics.ops_per_second, 1000.0);
    }

    #[test]
    fn test_execution_profile() {
        let mut profile = ExecutionProfile::new();
        profile.start();
        profile.record_operations(100);

        let metrics = profile.stop();
        assert_eq!(metrics.operations_executed, 100);
    }

    #[test]
    fn test_performance_comparison() {
        let comparison = PerformanceComparison::new(100, 110);
        assert_eq!(comparison.predicted_time_ms, 100);
        assert_eq!(comparison.actual_time_ms, 110);
        assert_eq!(comparison.error_ms, 10);
        assert!(comparison.is_accurate()); // Within 20%
    }

    #[test]
    fn test_performance_comparison_inaccurate() {
        let comparison = PerformanceComparison::new(100, 200);
        assert!(!comparison.is_accurate()); // Outside 20%
    }
}
