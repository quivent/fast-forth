//! Diff Reporter
//!
//! Formats semantic diff results for different output formats

use super::{SemanticDiff, DiffResult};
use serde_json;
use colored::Colorize;

/// Report format
#[derive(Debug, Clone, Copy)]
pub enum ReportFormat {
    Human,
    Json,
}

/// Diff reporter
pub struct DiffReporter;

impl DiffReporter {
    /// Generate a report from diff result
    pub fn report(result: &DiffResult, format: ReportFormat) -> String {
        match format {
            ReportFormat::Human => Self::report_human(result),
            ReportFormat::Json => Self::report_json(result),
        }
    }

    /// Generate human-readable report
    fn report_human(result: &DiffResult) -> String {
        let mut output = String::new();

        output.push_str(&format!("\n{}\n", "SEMANTIC DIFF REPORT".cyan().bold()));
        output.push_str(&"=".repeat(80));
        output.push_str("\n\n");

        output.push_str(&format!("Total words: {}\n", result.total_words));
        output.push_str(&format!("Changed: {}\n", result.changed_words.to_string().yellow()));
        output.push_str(&format!("Unchanged: {}\n", result.unchanged_words.to_string().green()));
        output.push_str("\n");

        for diff in &result.diffs {
            output.push_str(&Self::format_diff_human(diff));
            output.push_str("\n");
        }

        output
    }

    /// Format a single diff for human reading
    fn format_diff_human(diff: &SemanticDiff) -> String {
        let mut output = String::new();

        output.push_str(&format!("\n{}: {}\n", "Word".cyan().bold(), diff.word_name.yellow()));
        output.push_str(&"-".repeat(80));
        output.push_str("\n");

        // Stack effect
        output.push_str(&format!("{}\n", "Stack Effect:".green().bold()));
        if diff.stack_effect_changed {
            output.push_str(&format!("  - {}\n", diff.stack_effect_old.red()));
            output.push_str(&format!("  + {}\n", diff.stack_effect_new.green()));
        } else {
            output.push_str(&format!("  {} {}\n", "✓".green(), diff.stack_effect_new));
        }
        output.push_str("\n");

        // Operations
        output.push_str(&format!("{}\n", "Operations:".green().bold()));
        if diff.operations_changed {
            output.push_str(&format!("  - [{}]\n", diff.operations_old.join(", ").red()));
            output.push_str(&format!("  + [{}]\n", diff.operations_new.join(", ").green()));
        } else {
            output.push_str(&format!("  {} [{}]\n", "✓".green(), diff.operations_new.join(", ")));
        }
        output.push_str("\n");

        // Performance
        output.push_str(&format!("{}\n", "Performance:".green().bold()));
        if diff.performance_changed {
            output.push_str(&format!(
                "  - {} ops ({})\n",
                diff.performance_old.operation_count,
                diff.performance_old.complexity_class
            ).red().to_string());
            output.push_str(&format!(
                "  + {} ops ({})\n",
                diff.performance_new.operation_count,
                diff.performance_new.complexity_class
            ).green().to_string());

            let ratio = diff.performance_new.operation_count as f64
                / diff.performance_old.operation_count.max(1) as f64;

            if ratio < 1.0 {
                output.push_str(&format!(
                    "  {} {:.1}x faster\n",
                    "⚡".green(),
                    1.0 / ratio
                ).green().to_string());
            } else if ratio > 1.0 {
                output.push_str(&format!(
                    "  {} {:.1}x slower\n",
                    "⚠".yellow(),
                    ratio
                ).yellow().to_string());
            }
        } else {
            output.push_str(&format!(
                "  {} {} ops ({})\n",
                "✓".green(),
                diff.performance_new.operation_count,
                diff.performance_new.complexity_class
            ));
        }
        output.push_str("\n");

        // Equivalence
        output.push_str(&format!("{}\n", "Semantic Equivalence:".green().bold()));
        if diff.semantically_equivalent {
            output.push_str(&format!("  {} Semantically equivalent\n", "✓".green().bold()));
        } else {
            output.push_str(&format!("  {} NOT semantically equivalent\n", "✗".red().bold()));
        }
        output.push_str("\n");

        // Recommendation
        output.push_str(&format!("{}\n", "Recommendation:".green().bold()));
        output.push_str(&format!("  {}\n", diff.recommendation));

        output
    }

    /// Generate JSON report
    fn report_json(result: &DiffResult) -> String {
        serde_json::to_string_pretty(result).unwrap_or_else(|e| {
            format!("{{\"error\": \"Failed to serialize: {}\"}}", e)
        })
    }

    /// Generate summary line
    pub fn summary(result: &DiffResult) -> String {
        if result.changed_words == 0 {
            format!("{} No changes detected", "✓".green())
        } else {
            format!(
                "{} {} of {} words changed",
                "⚠".yellow(),
                result.changed_words,
                result.total_words
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_report_json() {
        let mut result = DiffResult::new();
        let mut diff = SemanticDiff::new("test".to_string());
        diff.stack_effect_old = "( a -- b )".to_string();
        diff.stack_effect_new = "( a -- b )".to_string();
        result.add_diff(diff);

        let report = DiffReporter::report(&result, ReportFormat::Json);
        assert!(report.contains("test"));
        assert!(report.contains("stack_effect"));
    }
}
