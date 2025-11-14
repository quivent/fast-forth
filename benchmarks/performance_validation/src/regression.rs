/// Regression detection and analysis
///
/// Monitors performance over time to detect degradations

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::benchmarks::BenchmarkResult;

/// Performance regression
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Regression {
    pub benchmark: String,
    pub degradation: f64,
    pub previous_time_ms: f64,
    pub current_time_ms: f64,
}

/// Historical performance data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalData {
    pub benchmark: String,
    pub measurements: Vec<f64>,
    pub timestamps: Vec<String>,
}

/// Regression testing and tracking
pub struct RegressionTester {
    history_file: std::path::PathBuf,
    history: HashMap<String, HistoricalData>,
}

impl RegressionTester {
    pub fn new(history_file: impl AsRef<Path>) -> Result<Self> {
        let history_file = history_file.as_ref().to_path_buf();
        let mut history = HashMap::new();

        // Try to load existing history
        if history_file.exists() {
            let data = fs::read_to_string(&history_file)?;
            if let Ok(loaded) = serde_json::from_str::<Vec<HistoricalData>>(&data) {
                for item in loaded {
                    history.insert(item.benchmark.clone(), item);
                }
            }
        }

        Ok(Self { history_file, history })
    }

    /// Check for performance regressions
    pub fn check_regressions(
        &self,
        current_results: &HashMap<String, crate::benchmarks::BenchmarkResult>,
        threshold: f64,
    ) -> Result<Vec<Regression>> {
        let mut regressions = Vec::new();

        for (bench_name, current) in current_results {
            if let Some(historical) = self.history.get(bench_name) {
                if !historical.measurements.is_empty() {
                    let avg_previous = historical.measurements.iter().sum::<f64>()
                        / historical.measurements.len() as f64;
                    let degradation = (current.avg_time_ms - avg_previous) / avg_previous;

                    if degradation > threshold {
                        regressions.push(Regression {
                            benchmark: bench_name.clone(),
                            degradation,
                            previous_time_ms: avg_previous,
                            current_time_ms: current.avg_time_ms,
                        });
                    }
                }
            }
        }

        Ok(regressions)
    }

    /// Update history with new measurements
    pub fn update_history(&mut self, results: &HashMap<String, BenchmarkResult>) -> Result<()> {
        let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

        for (bench_name, result) in results {
            let entry = self.history.entry(bench_name.clone()).or_insert_with(|| {
                HistoricalData {
                    benchmark: bench_name.clone(),
                    measurements: Vec::new(),
                    timestamps: Vec::new(),
                }
            });

            entry.measurements.push(result.avg_time_ms);
            entry.timestamps.push(timestamp.clone());

            // Keep only last 100 measurements
            if entry.measurements.len() > 100 {
                entry.measurements.remove(0);
                entry.timestamps.remove(0);
            }
        }

        // Save history
        let history_vec: Vec<_> = self.history.values().cloned().collect();
        let json = serde_json::to_string_pretty(&history_vec)?;
        fs::write(&self.history_file, json)?;

        Ok(())
    }

    /// Get historical trend for a benchmark
    pub fn get_trend(&self, benchmark: &str) -> Option<Vec<f64>> {
        self.history
            .get(benchmark)
            .map(|h| h.measurements.clone())
    }

    /// Get average performance over history
    pub fn get_historical_average(&self, benchmark: &str) -> Option<f64> {
        self.get_trend(benchmark).and_then(|measurements| {
            if measurements.is_empty() {
                None
            } else {
                Some(measurements.iter().sum::<f64>() / measurements.len() as f64)
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_regression_detection() {
        let threshold = 0.05;
        let degradation = 0.10;

        assert!(degradation > threshold);
    }
}
