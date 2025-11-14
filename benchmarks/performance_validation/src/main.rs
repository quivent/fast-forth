/// Fast Forth Performance Validation Framework
///
/// Comprehensive benchmarking and optimization validation system
/// to prove we achieve 1.0-1.2x C speed (match/beat VFX Forth)

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use tabled::{Table, Tabled};

mod benchmarks;
mod optimizations;
mod reports;
mod regression;

use benchmarks::{BenchmarkSuite, BenchmarkResult};
use optimizations::OptimizationLevel;
use reports::ReportGenerator;
use regression::RegressionTester;

/// Performance validation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationConfig {
    /// Benchmarks directory
    pub benchmarks_dir: PathBuf,
    /// Results output directory
    pub results_dir: PathBuf,
    /// Number of iterations per benchmark
    pub iterations: usize,
    /// Enable warmup runs
    pub warmup: bool,
    /// Number of warmup iterations
    pub warmup_iterations: usize,
    /// Target performance vs gcc (1.0 = match gcc)
    pub target_gcc_ratio: f64,
    /// Performance regression threshold (5%)
    pub regression_threshold: f64,
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            benchmarks_dir: PathBuf::from("../"),
            results_dir: PathBuf::from("results"),
            iterations: 100,
            warmup: true,
            warmup_iterations: 10,
            target_gcc_ratio: 1.0,
            regression_threshold: 0.05,
        }
    }
}

/// Performance validation results
#[derive(Debug, Clone, Serialize, Deserialize, Tabled)]
pub struct ValidationResult {
    #[tabled(rename = "Benchmark")]
    pub name: String,
    #[tabled(rename = "Optimization")]
    pub optimization: String,
    #[tabled(rename = "Time (ms)")]
    pub time_ms: f64,
    #[tabled(rename = "vs Baseline")]
    pub speedup: f64,
    #[tabled(rename = "vs GCC")]
    pub gcc_ratio: f64,
    #[tabled(rename = "Status")]
    pub status: String,
}

/// Optimization comparison results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationComparison {
    pub benchmark: String,
    pub baseline_ms: f64,
    pub optimizations: HashMap<String, OptimizationImpact>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationImpact {
    pub time_ms: f64,
    pub speedup: f64,
    pub instruction_reduction: Option<f64>,
    pub code_size_change: Option<i64>,
}

/// Main performance validation orchestrator
pub struct PerformanceValidator {
    config: ValidationConfig,
    suite: BenchmarkSuite,
    regression_tester: RegressionTester,
    report_generator: ReportGenerator,
}

impl PerformanceValidator {
    pub fn new(config: ValidationConfig) -> Result<Self> {
        // Create output directories
        fs::create_dir_all(&config.results_dir)?;

        let suite = BenchmarkSuite::new(config.benchmarks_dir.clone())?;
        let regression_tester = RegressionTester::new(config.results_dir.join("history.json"))?;
        let report_generator = ReportGenerator::new(config.results_dir.join("reports"))?;

        Ok(Self {
            config,
            suite,
            regression_tester,
            report_generator,
        })
    }

    /// Run complete validation suite
    pub fn validate(&mut self) -> Result<()> {
        println!("{}", "Fast Forth Performance Validation".bold().green());
        println!("{}", "===================================".bold());
        println!();

        // Run baseline C benchmarks
        println!("{}", "Step 1: Running C baseline benchmarks...".bold());
        let c_baselines = self.run_c_baselines()?;
        println!("{}", "✓ C baselines complete".green());
        println!();

        // Run Fast Forth benchmarks with different optimization levels
        println!("{}", "Step 2: Running Fast Forth benchmarks...".bold());
        let forth_results = self.run_forth_benchmarks()?;
        println!("{}", "✓ Fast Forth benchmarks complete".green());
        println!();

        // Compare optimizations
        println!("{}", "Step 3: Analyzing optimization impact...".bold());
        let comparisons = self.compare_optimizations(&forth_results)?;
        println!("{}", "✓ Optimization analysis complete".green());
        println!();

        // Run regression tests
        println!("{}", "Step 4: Checking for regressions...".bold());

        // Convert to flat HashMap with aggressive results
        let mut aggressive_results = HashMap::new();
        for (bench_name, opt_results) in &forth_results {
            if let Some(aggressive) = opt_results.get(&OptimizationLevel::Aggressive) {
                aggressive_results.insert(bench_name.clone(), aggressive.clone());
            }
        }

        let regressions = self.regression_tester.check_regressions(&aggressive_results, self.config.regression_threshold)?;
        if regressions.is_empty() {
            println!("{}", "✓ No performance regressions detected".green());
        } else {
            println!("{}", format!("⚠ {} regressions detected", regressions.len()).yellow());
            for reg in &regressions {
                println!("  {} - {:.1}% slower", reg.benchmark, reg.degradation * 100.0);
            }
        }
        println!();

        // Generate validation report
        println!("{}", "Step 5: Generating performance report...".bold());
        self.report_generator.generate_comprehensive_report(
            &c_baselines,
            &forth_results,
            &comparisons,
            &regressions,
        )?;
        println!("{}", "✓ Report generated".green());
        println!();

        // Display summary
        self.display_summary(&c_baselines, &forth_results)?;

        Ok(())
    }

    fn run_c_baselines(&self) -> Result<HashMap<String, BenchmarkResult>> {
        let mut results = HashMap::new();

        let c_dir = self.config.benchmarks_dir.join("c_baseline");
        let benchmarks = vec!["sieve", "fibonacci", "matrix"];

        for bench in benchmarks {
            print!("  Running {} (C)... ", bench);
            let executable = c_dir.join(bench);

            if !executable.exists() {
                // Try to compile
                let _ = Command::new("make")
                    .current_dir(&c_dir)
                    .arg(bench)
                    .output()?;
            }

            let result = self.suite.run_c_benchmark(&executable, self.config.iterations)?;
            println!("{:.3} ms", result.avg_time_ms);
            results.insert(bench.to_string(), result);
        }

        Ok(results)
    }

    fn run_forth_benchmarks(&self) -> Result<HashMap<String, HashMap<OptimizationLevel, BenchmarkResult>>> {
        let mut results = HashMap::new();

        let benchmarks = vec!["sieve", "fibonacci", "matrix"];
        let opt_levels = vec![
            OptimizationLevel::None,
            OptimizationLevel::Inlining,
            OptimizationLevel::PGO,
            OptimizationLevel::Aggressive,
        ];

        for bench in benchmarks {
            print!("  Running {} (Forth)... ", bench);
            let mut bench_results = HashMap::new();

            for opt in &opt_levels {
                let result = self.suite.run_forth_benchmark(bench, opt.clone(), self.config.iterations)?;
                bench_results.insert(opt.clone(), result);
            }

            println!("✓");
            results.insert(bench.to_string(), bench_results);
        }

        Ok(results)
    }

    fn compare_optimizations(
        &self,
        forth_results: &HashMap<String, HashMap<OptimizationLevel, BenchmarkResult>>
    ) -> Result<Vec<OptimizationComparison>> {
        let mut comparisons = Vec::new();

        for (bench_name, opt_results) in forth_results {
            let baseline = opt_results.get(&OptimizationLevel::None)
                .context("Missing baseline result")?;

            let mut optimizations = HashMap::new();

            for (opt_level, result) in opt_results {
                if *opt_level != OptimizationLevel::None {
                    let speedup = baseline.avg_time_ms / result.avg_time_ms;
                    optimizations.insert(
                        format!("{:?}", opt_level),
                        OptimizationImpact {
                            time_ms: result.avg_time_ms,
                            speedup,
                            instruction_reduction: None,
                            code_size_change: None,
                        }
                    );
                }
            }

            comparisons.push(OptimizationComparison {
                benchmark: bench_name.clone(),
                baseline_ms: baseline.avg_time_ms,
                optimizations,
            });
        }

        Ok(comparisons)
    }

    fn display_summary(
        &self,
        c_baselines: &HashMap<String, BenchmarkResult>,
        forth_results: &HashMap<String, HashMap<OptimizationLevel, BenchmarkResult>>
    ) -> Result<()> {
        println!("{}", "Performance Summary".bold().blue());
        println!("{}", "==================".bold());
        println!();

        let mut validation_results = Vec::new();

        for (bench_name, c_result) in c_baselines {
            if let Some(forth_opts) = forth_results.get(bench_name) {
                // Baseline (no optimizations)
                if let Some(baseline) = forth_opts.get(&OptimizationLevel::None) {
                    let gcc_ratio = baseline.avg_time_ms / c_result.avg_time_ms;
                    validation_results.push(ValidationResult {
                        name: bench_name.clone(),
                        optimization: "Baseline".to_string(),
                        time_ms: baseline.avg_time_ms,
                        speedup: 1.0,
                        gcc_ratio,
                        status: if gcc_ratio <= 1.2 { "✓".green().to_string() } else { "✗".red().to_string() },
                    });
                }

                // Aggressive optimizations
                if let Some(optimized) = forth_opts.get(&OptimizationLevel::Aggressive) {
                    let baseline = forth_opts.get(&OptimizationLevel::None).unwrap();
                    let speedup = baseline.avg_time_ms / optimized.avg_time_ms;
                    let gcc_ratio = optimized.avg_time_ms / c_result.avg_time_ms;
                    validation_results.push(ValidationResult {
                        name: bench_name.clone(),
                        optimization: "Aggressive".to_string(),
                        time_ms: optimized.avg_time_ms,
                        speedup,
                        gcc_ratio,
                        status: if gcc_ratio <= 1.2 { "✓".green().to_string() } else { "⚠".yellow().to_string() },
                    });
                }
            }
        }

        let table = Table::new(validation_results).to_string();
        println!("{}", table);
        println!();

        // Performance targets
        println!("{}", "Target Performance Goals:".bold());
        println!("  Sieve:  1.0-1.2x gcc speed (match VFX's 1.16x)");
        println!("  Fib:    1.0-1.2x gcc speed (match VFX's 1.09x)");
        println!("  Matrix: 0.8-1.0x gcc speed (beat VFX's 0.55x)");
        println!();

        println!("{}", "Report location:".bold());
        println!("  {}", self.config.results_dir.join("reports").display());

        Ok(())
    }
}

fn main() -> Result<()> {
    let config = ValidationConfig::default();
    let mut validator = PerformanceValidator::new(config)?;
    validator.validate()?;
    Ok(())
}
