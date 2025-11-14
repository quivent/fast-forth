// error_messages.rs - Fast Forth Error Message Formatting System
// Implements beautiful, helpful error messages with actionable suggestions

use std::fmt;

/// Error severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorSeverity {
    Error,   // Critical issue preventing compilation/execution
    Warning, // Potential problem or optimization opportunity
    Info,    // Helpful information or suggestion
    Hint,    // Style or best practice recommendation
}

impl fmt::Display for ErrorSeverity {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ErrorSeverity::Error => write!(f, "\x1b[31merror\x1b[0m"),   // Red
            ErrorSeverity::Warning => write!(f, "\x1b[33mwarning\x1b[0m"), // Yellow
            ErrorSeverity::Info => write!(f, "\x1b[34minfo\x1b[0m"),    // Blue
            ErrorSeverity::Hint => write!(f, "\x1b[90mhint\x1b[0m"),    // Gray
        }
    }
}

/// Source location information
#[derive(Debug, Clone)]
pub struct SourceLocation {
    pub file: String,
    pub line: usize,
    pub column: usize,
}

/// Code context for error display
#[derive(Debug, Clone)]
pub struct CodeContext {
    pub location: SourceLocation,
    pub source_line: String,
    pub pointer_column: usize,
    pub annotation: String,
}

/// A suggestion for fixing an error
#[derive(Debug, Clone)]
pub struct Suggestion {
    pub title: String,
    pub description: String,
    pub code_example: Option<String>,
}

/// Complete error message structure
#[derive(Debug, Clone)]
pub struct ErrorMessage {
    pub severity: ErrorSeverity,
    pub title: String,
    pub context: Option<CodeContext>,
    pub expected: Option<String>,
    pub actual: Option<String>,
    pub explanation: Option<String>,
    pub suggestions: Vec<Suggestion>,
    pub related_words: Vec<String>,
    pub documentation_link: Option<String>,
}

impl ErrorMessage {
    pub fn new(severity: ErrorSeverity, title: String) -> Self {
        ErrorMessage {
            severity,
            title,
            context: None,
            expected: None,
            actual: None,
            explanation: None,
            suggestions: Vec::new(),
            related_words: Vec::new(),
            documentation_link: None,
        }
    }

    /// Builder pattern methods
    pub fn with_context(mut self, context: CodeContext) -> Self {
        self.context = Some(context);
        self
    }

    pub fn with_expected(mut self, expected: String) -> Self {
        self.expected = Some(expected);
        self
    }

    pub fn with_actual(mut self, actual: String) -> Self {
        self.actual = Some(actual);
        self
    }

    pub fn with_explanation(mut self, explanation: String) -> Self {
        self.explanation = Some(explanation);
        self
    }

    pub fn add_suggestion(mut self, suggestion: Suggestion) -> Self {
        self.suggestions.push(suggestion);
        self
    }

    pub fn add_related_word(mut self, word: String) -> Self {
        self.related_words.push(word);
        self
    }

    pub fn with_documentation(mut self, url: String) -> Self {
        self.documentation_link = Some(url);
        self
    }

    /// Format the complete error message
    pub fn format(&self) -> String {
        let mut output = String::new();

        // Header: severity and title
        output.push_str(&format!("{}: {}\n\n", self.severity, self.title));

        // Context: file location
        if let Some(ctx) = &self.context {
            output.push_str(&format!(
                "  Context: File '{}', line {}, column {}\n\n",
                ctx.location.file, ctx.location.line, ctx.location.column
            ));
        }

        // Expected vs Actual
        if let Some(expected) = &self.expected {
            output.push_str(&format!("  Expected: {}\n", expected));
        }
        if let Some(actual) = &self.actual {
            output.push_str(&format!("  Actual:   {}\n\n", actual));
        }

        // Code context with pointer
        if let Some(ctx) = &self.context {
            output.push_str("  Code:\n");
            output.push_str(&format!(
                "    {} | {}\n",
                ctx.location.line, ctx.source_line
            ));
            output.push_str(&format!(
                "    {} |\n",
                " ".repeat(ctx.location.line.to_string().len())
            ));
            output.push_str(&format!(
                "    {} | {}\x1b[31m^\x1b[0m\n",
                " ".repeat(ctx.location.line.to_string().len()),
                " ".repeat(ctx.pointer_column)
            ));
            output.push_str(&format!(
                "    {} | {}\n\n",
                " ".repeat(ctx.location.line.to_string().len()),
                " ".repeat(ctx.pointer_column + 1) + &ctx.annotation
            ));
        }

        // Explanation
        if let Some(explanation) = &self.explanation {
            output.push_str(&format!("  Explanation:\n    {}\n\n", explanation));
        }

        // Suggestions
        if !self.suggestions.is_empty() {
            output.push_str("  Tip:\n");
            for (i, suggestion) in self.suggestions.iter().enumerate() {
                output.push_str(&format!("    {}. {}\n", i + 1, suggestion.title));
                if let Some(desc) = &suggestion.description.lines().next() {
                    output.push_str(&format!("       {}\n", desc));
                }
                if let Some(code) = &suggestion.code_example {
                    output.push_str(&format!("\n       {}\n\n", code));
                }
            }
        }

        // Related words
        if !self.related_words.is_empty() {
            output.push_str(&format!(
                "  Similar Words: {}\n\n",
                self.related_words.join(", ")
            ));
        }

        // Documentation link
        if let Some(url) = &self.documentation_link {
            output.push_str(&format!("  Learn more: {}\n", url));
        }

        output
    }
}

/// Pre-built error message constructors
pub struct ErrorTemplates;

impl ErrorTemplates {
    /// Stack underflow error
    pub fn stack_underflow(
        word: &str,
        location: SourceLocation,
        source_line: String,
        column: usize,
        expected: usize,
        actual: usize,
    ) -> ErrorMessage {
        ErrorMessage::new(
            ErrorSeverity::Error,
            format!("Stack underflow in word '{}'", word),
        )
        .with_context(CodeContext {
            location,
            source_line,
            pointer_column: column,
            annotation: "Stack underflow here".to_string(),
        })
        .with_expected(format!("{} items on stack", expected))
        .with_actual(format!("{} item{} on stack", actual, if actual == 1 { "" } else { "s" }))
        .with_explanation(format!(
            "The '{}' word expects {} items on the stack, but only {} {} available. \
            Check your stack effect comment and ensure all required values are pushed.",
            word, expected, actual, if actual == 1 { "is" } else { "are" }
        ))
        .add_suggestion(Suggestion {
            title: "Push missing value(s) before calling this word".to_string(),
            description: "Ensure all required values are on the stack".to_string(),
            code_example: None,
        })
        .with_documentation("https://fastforth.dev/docs/stack-effects".to_string())
    }

    /// Type mismatch error
    pub fn type_mismatch(
        word: &str,
        location: SourceLocation,
        source_line: String,
        column: usize,
        expected_type: &str,
        actual_type: &str,
    ) -> ErrorMessage {
        ErrorMessage::new(
            ErrorSeverity::Error,
            format!("Type mismatch in word '{}'", word),
        )
        .with_context(CodeContext {
            location,
            source_line,
            pointer_column: column,
            annotation: format!("Expected {}, got {}", expected_type, actual_type),
        })
        .with_expected(expected_type.to_string())
        .with_actual(actual_type.to_string())
        .with_explanation(format!(
            "The '{}' operator requires both operands to be of type {}, \
            but you provided {}. Consider converting types or using a different operator.",
            word, expected_type, actual_type
        ))
        .with_documentation("https://fastforth.dev/docs/types".to_string())
    }

    /// Undefined word error with fuzzy matching
    pub fn undefined_word(
        word: &str,
        location: SourceLocation,
        source_line: String,
        column: usize,
        similar_words: Vec<(String, f32)>, // (word, similarity score)
    ) -> ErrorMessage {
        let mut error = ErrorMessage::new(
            ErrorSeverity::Error,
            format!("Undefined word '{}'", word),
        )
        .with_context(CodeContext {
            location,
            source_line,
            pointer_column: column,
            annotation: "Word not found".to_string(),
        });

        // Add similar words as suggestions
        if !similar_words.is_empty() {
            let best_match = &similar_words[0];
            error = error.add_suggestion(Suggestion {
                title: format!("Did you mean '{}'?", best_match.0),
                description: format!("{}% match", (best_match.1 * 100.0) as u32),
                code_example: Some(format!(
                    "Replace '{}' with '{}'",
                    word, best_match.0
                )),
            });

            // Add related words
            for (similar_word, _) in similar_words.iter().take(5) {
                error = error.add_related_word(similar_word.clone());
            }
        }

        error.with_documentation("https://fastforth.dev/docs/word-lookup".to_string())
    }

    /// Stack effect mismatch warning
    pub fn stack_effect_mismatch(
        word: &str,
        location: SourceLocation,
        source_line: String,
        declared: &str,
        actual: &str,
    ) -> ErrorMessage {
        ErrorMessage::new(
            ErrorSeverity::Error,
            format!("Stack effect mismatch in word '{}'", word),
        )
        .with_context(CodeContext {
            location,
            source_line,
            pointer_column: 0,
            annotation: "Definition ends here".to_string(),
        })
        .with_expected(format!("Stack effect: {}", declared))
        .with_actual(format!("Stack effect: {}", actual))
        .with_explanation(
            "The declared stack effect doesn't match the actual behavior of the word. \
            Either update the stack effect comment or fix the implementation."
                .to_string(),
        )
        .with_documentation("https://fastforth.dev/docs/word-definitions".to_string())
    }

    /// Performance warning
    pub fn performance_warning(
        word: &str,
        location: SourceLocation,
        source_line: String,
        column: usize,
        issue: &str,
        suggestion: &str,
        speedup: f32,
    ) -> ErrorMessage {
        ErrorMessage::new(
            ErrorSeverity::Warning,
            format!("Performance opportunity in word '{}'", word),
        )
        .with_context(CodeContext {
            location,
            source_line,
            pointer_column: column,
            annotation: issue.to_string(),
        })
        .add_suggestion(Suggestion {
            title: suggestion.to_string(),
            description: format!("Expected speedup: {:.0}%", speedup * 100.0),
            code_example: None,
        })
        .with_documentation("https://fastforth.dev/docs/optimization".to_string())
    }
}

/// Calculate fuzzy string similarity (Levenshtein-based)
pub fn calculate_similarity(s1: &str, s2: &str) -> f32 {
    let len1 = s1.len();
    let len2 = s2.len();

    if len1 == 0 {
        return if len2 == 0 { 1.0 } else { 0.0 };
    }

    let mut prev_row: Vec<usize> = (0..=len2).collect();
    let mut curr_row = vec![0; len2 + 1];

    for (i, c1) in s1.chars().enumerate() {
        curr_row[0] = i + 1;
        for (j, c2) in s2.chars().enumerate() {
            let cost = if c1 == c2 { 0 } else { 1 };
            curr_row[j + 1] = std::cmp::min(
                std::cmp::min(curr_row[j] + 1, prev_row[j + 1] + 1),
                prev_row[j] + cost,
            );
        }
        std::mem::swap(&mut prev_row, &mut curr_row);
    }

    let distance = prev_row[len2];
    let max_len = std::cmp::max(len1, len2);
    1.0 - (distance as f32 / max_len as f32)
}

/// Find similar words from dictionary
pub fn find_similar_words(word: &str, dictionary: &[String], limit: usize) -> Vec<(String, f32)> {
    let mut similarities: Vec<(String, f32)> = dictionary
        .iter()
        .map(|dict_word| (dict_word.clone(), calculate_similarity(word, dict_word)))
        .filter(|(_, score)| *score > 0.4) // Only show if >40% similar
        .collect();

    similarities.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    similarities.truncate(limit);
    similarities
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_formatting() {
        let error = ErrorTemplates::stack_underflow(
            "AVERAGE",
            SourceLocation {
                file: "test.fth".to_string(),
                line: 10,
                column: 5,
            },
            "+ 2 /".to_string(),
            0,
            2,
            1,
        );

        let formatted = error.format();
        assert!(formatted.contains("Stack underflow"));
        assert!(formatted.contains("AVERAGE"));
        assert!(formatted.contains("test.fth"));
    }

    #[test]
    fn test_similarity() {
        assert!(calculate_similarity("AVERAGE", "AVERAGE") > 0.99);
        assert!(calculate_similarity("AVERGE", "AVERAGE") > 0.85);
        assert!(calculate_similarity("AVG", "AVERAGE") < 0.5);
    }

    #[test]
    fn test_find_similar_words() {
        let dictionary = vec![
            "AVERAGE".to_string(),
            "MERGE".to_string(),
            "DIVERGE".to_string(),
            "COMPUTE".to_string(),
        ];

        let similar = find_similar_words("AVERGE", &dictionary, 3);
        assert_eq!(similar[0].0, "AVERAGE");
        assert!(similar[0].1 > 0.85);
    }
}
