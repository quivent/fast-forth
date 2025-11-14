/// Performance report generation
///
/// Creates comprehensive reports with charts and analysis

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use anyhow::Result;
use chrono::Local;
use serde::Serialize;

use crate::benchmarks::BenchmarkResult;
use crate::optimizations::OptimizationLevel;
use crate::regression::Regression;
use crate::{OptimizationComparison, ValidationResult};

/// Report generator
pub struct ReportGenerator {
    reports_dir: PathBuf,
}

impl ReportGenerator {
    pub fn new(reports_dir: PathBuf) -> Result<Self> {
        fs::create_dir_all(&reports_dir)?;
        Ok(Self { reports_dir })
    }

    /// Generate comprehensive performance report
    pub fn generate_comprehensive_report(
        &self,
        c_baselines: &HashMap<String, BenchmarkResult>,
        forth_results: &HashMap<String, HashMap<OptimizationLevel, BenchmarkResult>>,
        comparisons: &[OptimizationComparison],
        regressions: &[Regression],
    ) -> Result<()> {
        let timestamp = Local::now().format("%Y-%m-%d_%H-%M-%S");
        let report_path = self.reports_dir.join(format!("performance_report_{}.md", timestamp));

        let mut report = String::new();

        // Header
        report.push_str("# Fast Forth Performance Validation Report\n\n");
        report.push_str(&format!("**Generated**: {}\n\n", Local::now().format("%Y-%m-%d %H:%M:%S")));

        // Executive Summary
        report.push_str("## Executive Summary\n\n");
        self.add_executive_summary(&mut report, c_baselines, forth_results);

        // Detailed Results
        report.push_str("\n## Detailed Benchmark Results\n\n");
        self.add_detailed_results(&mut report, c_baselines, forth_results);

        // Optimization Impact Analysis
        report.push_str("\n## Optimization Impact Analysis\n\n");
        self.add_optimization_analysis(&mut report, comparisons);

        // Performance vs Targets
        report.push_str("\n## Performance vs Target Goals\n\n");
        self.add_target_comparison(&mut report, c_baselines, forth_results);

        // Regression Analysis
        report.push_str("\n## Regression Analysis\n\n");
        self.add_regression_analysis(&mut report, regressions);

        // Recommendations
        report.push_str("\n## Recommendations\n\n");
        self.add_recommendations(&mut report, c_baselines, forth_results);

        // Write report
        fs::write(&report_path, report)?;
        println!("  Report saved to: {}", report_path.display());

        // Generate JSON data
        self.generate_json_report(c_baselines, forth_results, comparisons, timestamp)?;

        Ok(())
    }

    fn add_executive_summary(
        &self,
        report: &mut String,
        c_baselines: &HashMap<String, BenchmarkResult>,
        forth_results: &HashMap<String, HashMap<OptimizationLevel, BenchmarkResult>>,
    ) {
        report.push_str("### Key Findings\n\n");

        let mut findings = Vec::new();

        // Calculate average speedup
        let mut speedups = Vec::new();
        for (bench, opts) in forth_results {
            if let (Some(baseline), Some(optimized)) = (
                opts.get(&OptimizationLevel::None),
                opts.get(&OptimizationLevel::Aggressive),
            ) {
                speedups.push(baseline.avg_time_ms / optimized.avg_time_ms);
            }
        }

        if !speedups.is_empty() {
            let avg_speedup = speedups.iter().sum::<f64>() / speedups.len() as f64;
            findings.push(format!(
                "- **Average Optimization Speedup**: {:.2}x ({:.0}% improvement)",
                avg_speedup,
                (avg_speedup - 1.0) * 100.0
            ));
        }

        // Calculate average vs GCC
        let mut gcc_ratios = Vec::new();
        for (bench, opts) in forth_results {
            if let (Some(optimized), Some(c_baseline)) = (
                opts.get(&OptimizationLevel::Aggressive),
                c_baselines.get(bench),
            ) {
                gcc_ratios.push(optimized.avg_time_ms / c_baseline.avg_time_ms);
            }
        }

        if !gcc_ratios.is_empty() {
            let avg_gcc_ratio = gcc_ratios.iter().sum::<f64>() / gcc_ratios.len() as f64;
            findings.push(format!(
                "- **Average Performance vs GCC**: {:.2}x ({} target of 1.0-1.2x)",
                avg_gcc_ratio,
                if avg_gcc_ratio <= 1.2 { "✓ Within" } else { "✗ Outside" }
            ));
        }

        // Best performing benchmark
        if let Some((best_bench, best_ratio)) = gcc_ratios.iter().enumerate()
            .min_by(|a, b| a.1.partial_cmp(b.1).unwrap())
        {
            let bench_name = forth_results.keys().nth(best_bench).unwrap();
            findings.push(format!(
                "- **Best Performance**: {} at {:.2}x gcc speed",
                bench_name, best_ratio
            ));
        }

        for finding in findings {
            report.push_str(&format!("{}\n", finding));
        }

        report.push('\n');
    }

    fn add_detailed_results(
        &self,
        report: &mut String,
        c_baselines: &HashMap<String, BenchmarkResult>,
        forth_results: &HashMap<String, HashMap<OptimizationLevel, BenchmarkResult>>,
    ) {
        report.push_str("| Benchmark | Optimization | Time (ms) | vs Baseline | vs GCC | Status |\n");
        report.push_str("|-----------|--------------|-----------|-------------|--------|--------|\n");

        for (bench_name, opts) in forth_results {
            let c_baseline = c_baselines.get(bench_name);
            let forth_baseline = opts.get(&OptimizationLevel::None);

            for opt_level in OptimizationLevel::all() {
                if let Some(result) = opts.get(&opt_level) {
                    let speedup = if let Some(baseline) = forth_baseline {
                        baseline.avg_time_ms / result.avg_time_ms
                    } else {
                        1.0
                    };

                    let gcc_ratio = if let Some(c_res) = c_baseline {
                        result.avg_time_ms / c_res.avg_time_ms
                    } else {
                        0.0
                    };

                    let status = if gcc_ratio > 0.0 && gcc_ratio <= 1.2 {
                        "✓"
                    } else if gcc_ratio <= 1.5 {
                        "⚠"
                    } else {
                        "✗"
                    };

                    report.push_str(&format!(
                        "| {} | {} | {:.3} | {:.2}x | {:.2}x | {} |\n",
                        bench_name, opt_level, result.avg_time_ms, speedup, gcc_ratio, status
                    ));
                }
            }

            // Add C baseline for reference
            if let Some(c_res) = c_baseline {
                report.push_str(&format!(
                    "| {} | gcc -O2 | {:.3} | - | 1.00x | baseline |\n",
                    bench_name, c_res.avg_time_ms
                ));
            }

            report.push_str("|-----------|--------------|-----------|-------------|--------|--------|\n");
        }

        report.push('\n');
    }

    fn add_optimization_analysis(
        &self,
        report: &mut String,
        comparisons: &[OptimizationComparison],
    ) {
        for comparison in comparisons {
            report.push_str(&format!("### {}\n\n", comparison.benchmark));
            report.push_str(&format!("**Baseline**: {:.3} ms\n\n", comparison.baseline_ms));

            report.push_str("| Optimization | Time (ms) | Speedup | Improvement |\n");
            report.push_str("|--------------|-----------|---------|-------------|\n");

            for (opt_name, impact) in &comparison.optimizations {
                let improvement = (impact.speedup - 1.0) * 100.0;
                report.push_str(&format!(
                    "| {} | {:.3} | {:.2}x | +{:.1}% |\n",
                    opt_name, impact.time_ms, impact.speedup, improvement
                ));
            }

            report.push_str("\n");
        }
    }

    fn add_target_comparison(
        &self,
        report: &mut String,
        c_baselines: &HashMap<String, BenchmarkResult>,
        forth_results: &HashMap<String, HashMap<OptimizationLevel, BenchmarkResult>>,
    ) {
        report.push_str("Based on VFX Forth performance targets:\n\n");
        report.push_str("| Benchmark | Target | Actual | Status |\n");
        report.push_str("|-----------|--------|--------|--------|\n");

        let targets = [
            ("sieve", 1.0, 1.2),
            ("fibonacci", 1.0, 1.2),
            ("matrix", 0.8, 1.0),
        ];

        for (bench_name, min_target, max_target) in targets {
            if let (Some(opts), Some(c_res)) = (forth_results.get(bench_name), c_baselines.get(bench_name)) {
                if let Some(optimized) = opts.get(&OptimizationLevel::Aggressive) {
                    let ratio = optimized.avg_time_ms / c_res.avg_time_ms;
                    let status = if ratio >= min_target && ratio <= max_target {
                        "✓ On target"
                    } else if ratio < min_target {
                        "✓ Exceeded"
                    } else {
                        "✗ Below target"
                    };

                    report.push_str(&format!(
                        "| {} | {:.2}x-{:.2}x | {:.2}x | {} |\n",
                        bench_name, min_target, max_target, ratio, status
                    ));
                }
            }
        }

        report.push_str("\n");
    }

    fn add_regression_analysis(
        &self,
        report: &mut String,
        regressions: &[Regression],
    ) {
        if regressions.is_empty() {
            report.push_str("✓ No performance regressions detected.\n\n");
        } else {
            report.push_str(&format!("⚠ {} regression(s) detected:\n\n", regressions.len()));

            report.push_str("| Benchmark | Degradation | Previous | Current |\n");
            report.push_str("|-----------|-------------|----------|----------|\n");

            for reg in regressions {
                report.push_str(&format!(
                    "| {} | {:.1}% | {:.3} ms | {:.3} ms |\n",
                    reg.benchmark,
                    reg.degradation * 100.0,
                    reg.previous_time_ms,
                    reg.current_time_ms
                ));
            }

            report.push_str("\n");
        }
    }

    fn add_recommendations(
        &self,
        report: &mut String,
        _c_baselines: &HashMap<String, BenchmarkResult>,
        forth_results: &HashMap<String, HashMap<OptimizationLevel, BenchmarkResult>>,
    ) {
        report.push_str("Based on the performance analysis:\n\n");

        // Check which benchmarks need improvement
        let mut needs_improvement = Vec::new();
        for (bench, _opts) in forth_results {
            needs_improvement.push(bench.clone());
        }

        if needs_improvement.is_empty() {
            report.push_str("- ✓ All benchmarks meet or exceed target performance\n");
            report.push_str("- Continue monitoring for regressions\n");
        } else {
            report.push_str("- Focus optimization efforts on:\n");
            for bench in needs_improvement {
                report.push_str(&format!("  - {}\n", bench));
            }
        }

        report.push_str("\n");
    }

    fn generate_json_report(
        &self,
        c_baselines: &HashMap<String, BenchmarkResult>,
        forth_results: &HashMap<String, HashMap<OptimizationLevel, BenchmarkResult>>,
        comparisons: &[OptimizationComparison],
        timestamp: impl std::fmt::Display,
    ) -> Result<()> {
        #[derive(Serialize)]
        struct JsonReport<'a> {
            timestamp: String,
            c_baselines: &'a HashMap<String, BenchmarkResult>,
            forth_results: &'a HashMap<String, HashMap<OptimizationLevel, BenchmarkResult>>,
            comparisons: &'a [OptimizationComparison],
        }

        let report = JsonReport {
            timestamp: timestamp.to_string(),
            c_baselines,
            forth_results,
            comparisons,
        };

        let json_path = self.reports_dir.join(format!("performance_data_{}.json", timestamp));
        let json = serde_json::to_string_pretty(&report)?;
        fs::write(&json_path, json)?;

        println!("  JSON data saved to: {}", json_path.display());

        Ok(())
    }
}
