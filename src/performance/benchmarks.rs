//! Benchmark suite integration for performance validation
//!
//! Provides integration with benchmark suites to validate performance predictions

use crate::error::{CompileError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// A single benchmark result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResult {
    /// Benchmark name
    pub name: String,

    /// Execution time
    pub execution_time: Duration,

    /// Operations executed
    pub operations: u64,

    /// Throughput (ops/sec)
    pub throughput: f64,

    /// Comparison to baseline (if available)
    pub speedup: Option<f64>,

    /// Success status
    pub success: bool,

    /// Error message (if failed)
    pub error: Option<String>,
}

impl BenchmarkResult {
    /// Create a successful benchmark result
    pub fn success(name: String, execution_time: Duration, operations: u64) -> Self {
        let throughput = operations as f64 / execution_time.as_secs_f64().max(0.001);

        Self {
            name,
            execution_time,
            operations,
            throughput,
            speedup: None,
            success: true,
            error: None,
        }
    }

    /// Create a failed benchmark result
    pub fn failure(name: String, error: String) -> Self {
        Self {
            name,
            execution_time: Duration::ZERO,
            operations: 0,
            throughput: 0.0,
            speedup: None,
            success: false,
            error: Some(error),
        }
    }

    /// Set speedup compared to baseline
    pub fn with_speedup(mut self, speedup: f64) -> Self {
        self.speedup = Some(speedup);
        self
    }

    /// Get formatted summary
    pub fn summary(&self) -> String {
        if self.success {
            let speedup_str = if let Some(speedup) = self.speedup {
                format!(" | {:.2}x speedup", speedup)
            } else {
                String::new()
            };

            format!(
                "{}: {:.2}ms | {} ops | {:.0} ops/sec{}",
                self.name,
                self.execution_time.as_secs_f64() * 1000.0,
                self.operations,
                self.throughput,
                speedup_str
            )
        } else {
            format!(
                "{}: FAILED - {}",
                self.name,
                self.error.as_ref().unwrap_or(&"Unknown error".to_string())
            )
        }
    }
}

/// Benchmark suite for performance validation
pub struct BenchmarkSuite {
    benchmarks: HashMap<String, BenchmarkFn>,
    baselines: HashMap<String, Duration>,
}

type BenchmarkFn = Box<dyn Fn() -> Result<(Duration, u64)> + Send + Sync>;

impl BenchmarkSuite {
    /// Create a new benchmark suite
    pub fn new() -> Self {
        Self {
            benchmarks: HashMap::new(),
            baselines: HashMap::new(),
        }
    }

    /// Add a benchmark to the suite
    pub fn add_benchmark<F>(&mut self, name: String, benchmark: F)
    where
        F: Fn() -> Result<(Duration, u64)> + Send + Sync + 'static,
    {
        self.benchmarks.insert(name, Box::new(benchmark));
    }

    /// Set baseline time for a benchmark
    pub fn set_baseline(&mut self, name: String, time: Duration) {
        self.baselines.insert(name, time);
    }

    /// Run a specific benchmark
    pub fn run_benchmark(&self, name: &str) -> Result<BenchmarkResult> {
        let benchmark = self.benchmarks.get(name)
            .ok_or_else(|| CompileError::InternalError(
                format!("Benchmark '{}' not found", name)
            ))?;

        match benchmark() {
            Ok((execution_time, operations)) => {
                let mut result = BenchmarkResult::success(
                    name.to_string(),
                    execution_time,
                    operations,
                );

                // Calculate speedup if baseline exists
                if let Some(baseline) = self.baselines.get(name) {
                    let speedup = baseline.as_secs_f64() / execution_time.as_secs_f64();
                    result = result.with_speedup(speedup);
                }

                Ok(result)
            }
            Err(e) => {
                Ok(BenchmarkResult::failure(name.to_string(), e.to_string()))
            }
        }
    }

    /// Run all benchmarks
    pub fn run_all(&self) -> Vec<BenchmarkResult> {
        let mut results = Vec::new();

        for name in self.benchmarks.keys() {
            match self.run_benchmark(name) {
                Ok(result) => results.push(result),
                Err(e) => {
                    results.push(BenchmarkResult::failure(
                        name.clone(),
                        e.to_string(),
                    ));
                }
            }
        }

        results
    }

    /// Get benchmark names
    pub fn benchmark_names(&self) -> Vec<String> {
        self.benchmarks.keys().cloned().collect()
    }

    /// Check if a benchmark exists
    pub fn has_benchmark(&self, name: &str) -> bool {
        self.benchmarks.contains_key(name)
    }
}

impl Default for BenchmarkSuite {
    fn default() -> Self {
        Self::new()
    }
}

/// Standard benchmark suite for Fast Forth
pub struct StandardBenchmarks;

impl StandardBenchmarks {
    /// Create a suite with standard Fast Forth benchmarks
    pub fn create_suite() -> BenchmarkSuite {
        let mut suite = BenchmarkSuite::new();

        // Factorial benchmark
        suite.add_benchmark("factorial".to_string(), || {
            let start = std::time::Instant::now();
            let mut result = 1u64;
            for i in 1..=20 {
                result = result.saturating_mul(i);
            }
            let duration = start.elapsed();
            Ok((duration, 20))
        });

        // Fibonacci benchmark
        suite.add_benchmark("fibonacci".to_string(), || {
            let start = std::time::Instant::now();
            let mut a = 0u64;
            let mut b = 1u64;
            for _ in 0..30 {
                let temp = a;
                a = b;
                b = temp.saturating_add(b);
            }
            let duration = start.elapsed();
            Ok((duration, 30))
        });

        // Stack operations benchmark
        suite.add_benchmark("stack_ops".to_string(), || {
            let start = std::time::Instant::now();
            let mut stack = Vec::with_capacity(100);
            for i in 0..1000 {
                stack.push(i);
                if stack.len() >= 2 {
                    let a = stack.pop().unwrap();
                    let b = stack.pop().unwrap();
                    stack.push(a + b);
                }
            }
            let duration = start.elapsed();
            Ok((duration, 2000))
        });

        // Set C baselines (example values)
        suite.set_baseline("factorial".to_string(), Duration::from_nanos(500));
        suite.set_baseline("fibonacci".to_string(), Duration::from_nanos(300));
        suite.set_baseline("stack_ops".to_string(), Duration::from_micros(10));

        suite
    }

    /// Run standard benchmarks and return results
    pub fn run() -> Vec<BenchmarkResult> {
        let suite = Self::create_suite();
        suite.run_all()
    }
}

/// Benchmark report generator
pub struct BenchmarkReport {
    results: Vec<BenchmarkResult>,
}

impl BenchmarkReport {
    /// Create a new report from results
    pub fn new(results: Vec<BenchmarkResult>) -> Self {
        Self { results }
    }

    /// Get total benchmarks run
    pub fn total_benchmarks(&self) -> usize {
        self.results.len()
    }

    /// Get number of successful benchmarks
    pub fn successful_benchmarks(&self) -> usize {
        self.results.iter().filter(|r| r.success).count()
    }

    /// Get number of failed benchmarks
    pub fn failed_benchmarks(&self) -> usize {
        self.results.iter().filter(|r| !r.success).count()
    }

    /// Get average speedup (if baselines available)
    pub fn average_speedup(&self) -> Option<f64> {
        let speedups: Vec<f64> = self.results
            .iter()
            .filter_map(|r| r.speedup)
            .collect();

        if speedups.is_empty() {
            None
        } else {
            Some(speedups.iter().sum::<f64>() / speedups.len() as f64)
        }
    }

    /// Generate formatted report
    pub fn format(&self) -> String {
        let mut report = String::new();
        report.push_str("Benchmark Results\n");
        report.push_str("=================\n\n");

        for result in &self.results {
            report.push_str(&format!("{}\n", result.summary()));
        }

        report.push_str(&format!(
            "\nTotal: {} | Success: {} | Failed: {}\n",
            self.total_benchmarks(),
            self.successful_benchmarks(),
            self.failed_benchmarks()
        ));

        if let Some(avg_speedup) = self.average_speedup() {
            report.push_str(&format!("Average Speedup: {:.2}x\n", avg_speedup));
        }

        report
    }

    /// Generate JSON report
    pub fn to_json(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(&self.results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_benchmark_result_success() {
        let result = BenchmarkResult::success(
            "test".to_string(),
            Duration::from_millis(100),
            1000,
        );

        assert!(result.success);
        assert_eq!(result.name, "test");
        assert_eq!(result.operations, 1000);
    }

    #[test]
    fn test_benchmark_result_failure() {
        let result = BenchmarkResult::failure(
            "test".to_string(),
            "Test error".to_string(),
        );

        assert!(!result.success);
        assert!(result.error.is_some());
    }

    #[test]
    fn test_benchmark_suite() {
        let mut suite = BenchmarkSuite::new();
        suite.add_benchmark("test".to_string(), || {
            Ok((Duration::from_millis(10), 100))
        });

        assert!(suite.has_benchmark("test"));
        assert!(!suite.has_benchmark("nonexistent"));
    }

    #[test]
    fn test_standard_benchmarks() {
        let suite = StandardBenchmarks::create_suite();
        assert!(suite.has_benchmark("factorial"));
        assert!(suite.has_benchmark("fibonacci"));
        assert!(suite.has_benchmark("stack_ops"));
    }

    #[test]
    fn test_benchmark_report() {
        let results = vec![
            BenchmarkResult::success("test1".to_string(), Duration::from_millis(10), 100),
            BenchmarkResult::success("test2".to_string(), Duration::from_millis(20), 200),
        ];

        let report = BenchmarkReport::new(results);
        assert_eq!(report.total_benchmarks(), 2);
        assert_eq!(report.successful_benchmarks(), 2);
        assert_eq!(report.failed_benchmarks(), 0);
    }
}
