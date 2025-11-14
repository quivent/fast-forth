//! Error formatters for different output modes

use super::structured::{StructuredError, ErrorSeverity};
use colored::Colorize;

/// Output format selection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    /// Human-friendly colored output
    Human,
    /// Compact JSON for agents
    Json,
    /// Pretty-printed JSON
    JsonPretty,
    /// Plain text (no colors)
    Plain,
}

pub struct ErrorFormatter;

impl ErrorFormatter {
    /// Format error in specified output format
    pub fn format(error: &StructuredError, format: OutputFormat) -> String {
        match format {
            OutputFormat::Human => Self::format_human(error),
            OutputFormat::Json => Self::format_json(error, false),
            OutputFormat::JsonPretty => Self::format_json(error, true),
            OutputFormat::Plain => Self::format_plain(error),
        }
    }

    /// Format for human consumption with colors
    fn format_human(error: &StructuredError) -> String {
        let mut output = String::new();

        // Error header
        let severity = error.severity.unwrap_or(ErrorSeverity::Error);
        let severity_str = match severity {
            ErrorSeverity::Error => "Error".red().bold(),
            ErrorSeverity::Warning => "Warning".yellow().bold(),
            ErrorSeverity::Info => "Info".blue().bold(),
        };

        output.push_str(&format!("{}: {}\n", severity_str, error.error));

        // Error code and location
        output.push_str(&format!(
            "  {} {}\n",
            "Code:".cyan(),
            error.code.bright_white()
        ));

        let loc = &error.location;
        let location_str = if let Some(word) = &loc.word {
            format!(
                "{}:{}:{} in word '{}'",
                loc.file.as_deref().unwrap_or("<input>"),
                loc.line,
                loc.column,
                word
            )
        } else {
            format!(
                "{}:{}:{}",
                loc.file.as_deref().unwrap_or("<input>"),
                loc.line,
                loc.column
            )
        };
        output.push_str(&format!("  {} {}\n", "Location:".cyan(), location_str));

        // Stack effects if available
        if let (Some(expected), Some(actual)) = (&error.expected_effect, &error.actual_effect) {
            output.push_str(&format!("\n  {} {}\n", "Expected:".green(), expected));
            output.push_str(&format!("  {} {}\n", "Actual:".red(), actual));
        }

        // Context if available
        if let Some(context) = &loc.context {
            output.push_str(&format!("\n  {}\n", context.bright_black()));
            output.push_str(&format!(
                "  {}{}^ {}\n",
                " ".repeat(loc.column.saturating_sub(1)),
                "".red(),
                "Error here".red()
            ));
        }

        // Primary suggestion
        if let Some(suggestion) = &error.suggestion {
            output.push_str(&format!("\n  {} {}\n", "Suggestion:".yellow().bold(), suggestion.fix));

            if let Some(pattern) = &suggestion.pattern {
                output.push_str(&format!("  {} {}\n", "Pattern:".cyan(), pattern));
            }

            output.push_str(&format!(
                "  {} {:.0}%\n",
                "Confidence:".cyan(),
                suggestion.confidence * 100.0
            ));

            if let Some(explanation) = &suggestion.explanation {
                output.push_str(&format!("  {} {}\n", "Reason:".cyan(), explanation));
            }

            // Show diff
            output.push_str(&format!("\n  {} Diff:\n", "Code".cyan()));
            output.push_str(&format!("  {} {}\n", "-".red(), suggestion.diff.old.red()));
            output.push_str(&format!("  {} {}\n", "+".green(), suggestion.diff.new.green()));
        }

        // Alternative suggestions
        if !error.alternatives.is_empty() {
            output.push_str(&format!("\n  {}:\n", "Alternative Fixes".yellow()));
            for (i, alt) in error.alternatives.iter().enumerate() {
                output.push_str(&format!(
                    "  {}. {} (confidence: {:.0}%)\n",
                    i + 1,
                    alt.fix,
                    alt.confidence * 100.0
                ));
            }
        }

        // Related errors
        if !error.related_errors.is_empty() {
            output.push_str(&format!("\n  {}:\n", "Related".cyan()));
            for related in &error.related_errors {
                output.push_str(&format!("  - {}\n", related));
            }
        }

        output
    }

    /// Format as JSON
    fn format_json(error: &StructuredError, pretty: bool) -> String {
        if pretty {
            error.to_json().unwrap_or_else(|e| format!("{{\"error\": \"JSON serialization failed: {}\"}}", e))
        } else {
            error.to_json_compact().unwrap_or_else(|e| format!("{{\"error\": \"JSON serialization failed: {}\"}}", e))
        }
    }

    /// Format as plain text without colors
    fn format_plain(error: &StructuredError) -> String {
        let mut output = String::new();

        output.push_str(&format!("Error: {}\n", error.error));
        output.push_str(&format!("Code: {}\n", error.code));

        let loc = &error.location;
        output.push_str(&format!(
            "Location: {}:{}:{}\n",
            loc.file.as_deref().unwrap_or("<input>"),
            loc.line,
            loc.column
        ));

        if let Some(word) = &loc.word {
            output.push_str(&format!("Word: {}\n", word));
        }

        if let (Some(expected), Some(actual)) = (&error.expected_effect, &error.actual_effect) {
            output.push_str(&format!("Expected: {}\n", expected));
            output.push_str(&format!("Actual: {}\n", actual));
        }

        if let Some(suggestion) = &error.suggestion {
            output.push_str(&format!("\nSuggestion: {}\n", suggestion.fix));
            output.push_str(&format!("Confidence: {:.0}%\n", suggestion.confidence * 100.0));
        }

        output
    }
}

/// Convenience function to format error
pub fn format_error(error: &StructuredError, format: OutputFormat) -> String {
    ErrorFormatter::format(error, format)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::{ErrorCode, Location, Suggestion};

    #[test]
    fn test_json_format() {
        let error = StructuredError::new(ErrorCode::StackDepthMismatch, "Test")
            .with_location(Location::new(5, 10));

        let json = ErrorFormatter::format(&error, OutputFormat::Json);
        assert!(json.contains("E2234"));
    }

    #[test]
    fn test_human_format() {
        let error = StructuredError::new(ErrorCode::StackDepthMismatch, "Test")
            .with_location(Location::new(5, 10))
            .with_suggestion(Suggestion::new("Add drop", "old", "new"));

        let human = ErrorFormatter::format(&error, OutputFormat::Human);
        assert!(human.contains("Error:"));
        assert!(human.contains("Suggestion:"));
    }
}
