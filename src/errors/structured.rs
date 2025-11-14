//! Structured error representation for machine consumption

use super::error_code::ErrorCode;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

/// Error severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ErrorSeverity {
    Error,
    Warning,
    Info,
}

/// Precise location information
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Location {
    pub file: Option<String>,
    pub line: usize,
    pub column: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub word: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
}

impl Location {
    pub fn new(line: usize, column: usize) -> Self {
        Self {
            file: None,
            line,
            column,
            word: None,
            context: None,
        }
    }

    pub fn with_file(mut self, file: impl Into<String>) -> Self {
        self.file = Some(file.into());
        self
    }

    pub fn with_word(mut self, word: impl Into<String>) -> Self {
        self.word = Some(word.into());
        self
    }

    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        self.context = Some(context.into());
        self
    }
}

/// Code diff for auto-fix suggestions
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FixDiff {
    pub old: String,
    pub new: String,
}

/// Auto-fix suggestion with confidence
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Suggestion {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pattern: Option<String>,
    pub fix: String,
    pub confidence: f64,
    pub diff: FixDiff,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub explanation: Option<String>,
}

impl Suggestion {
    pub fn new(fix: impl Into<String>, old: impl Into<String>, new: impl Into<String>) -> Self {
        Self {
            pattern: None,
            fix: fix.into(),
            confidence: 0.5,
            diff: FixDiff {
                old: old.into(),
                new: new.into(),
            },
            explanation: None,
        }
    }

    pub fn with_pattern(mut self, pattern: impl Into<String>) -> Self {
        self.pattern = Some(pattern.into());
        self
    }

    pub fn with_confidence(mut self, confidence: f64) -> Self {
        self.confidence = confidence.clamp(0.0, 1.0);
        self
    }

    pub fn with_explanation(mut self, explanation: impl Into<String>) -> Self {
        self.explanation = Some(explanation.into());
        self
    }
}

/// Structured error message
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StructuredError {
    pub error: String,
    pub code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected_effect: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actual_effect: Option<String>,
    pub location: Location,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggestion: Option<Suggestion>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub alternatives: Vec<Suggestion>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub related_errors: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub severity: Option<ErrorSeverity>,
    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    pub metadata: HashMap<String, String>,
}

impl StructuredError {
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            error: message.into(),
            code: code.as_str(),
            expected_effect: None,
            actual_effect: None,
            location: Location::new(0, 0),
            suggestion: None,
            alternatives: Vec::new(),
            related_errors: Vec::new(),
            severity: Some(ErrorSeverity::Error),
            metadata: HashMap::new(),
        }
    }

    pub fn with_location(mut self, location: Location) -> Self {
        self.location = location;
        self
    }

    pub fn with_stack_effect(
        mut self,
        expected: impl Into<String>,
        actual: impl Into<String>,
    ) -> Self {
        self.expected_effect = Some(expected.into());
        self.actual_effect = Some(actual.into());
        self
    }

    pub fn with_suggestion(mut self, suggestion: Suggestion) -> Self {
        self.suggestion = Some(suggestion);
        self
    }

    pub fn with_alternatives(mut self, alternatives: Vec<Suggestion>) -> Self {
        self.alternatives = alternatives;
        self
    }

    pub fn with_severity(mut self, severity: ErrorSeverity) -> Self {
        self.severity = Some(severity);
        self
    }

    pub fn add_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Convert to JSON string
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Convert to compact JSON
    pub fn to_json_compact(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}

/// Convert from existing CompileError to StructuredError
pub fn convert_to_structured(
    error: &crate::error::CompileError,
    suggest_fixes: bool,
) -> StructuredError {
    use crate::error::CompileError;

    match error {
        CompileError::ParseError(msg) => {
            StructuredError::new(ErrorCode::UnexpectedToken, msg)
                .with_location(Location::new(0, 0))
        }

        CompileError::SemanticError(msg) => {
            if msg.contains("undefined") {
                StructuredError::new(ErrorCode::UndefinedWord, msg)
            } else if msg.contains("redefined") {
                StructuredError::new(ErrorCode::RedefinedWord, msg)
            } else {
                StructuredError::new(ErrorCode::InternalCompilerError, msg)
            }
        }

        CompileError::TypeError(msg) => {
            let mut err = StructuredError::new(ErrorCode::TypeMismatch, msg);

            if suggest_fixes {
                // Try to extract type information and suggest fix
                if msg.contains("stack depth") {
                    err = err.with_suggestion(
                        Suggestion::new(
                            "Add 'drop' to remove excess value",
                            "dup dup *",
                            "dup dup * drop",
                        )
                        .with_pattern("DROP_EXCESS_001")
                        .with_confidence(0.85)
                        .with_explanation("Stack has more items than expected by the declared effect"),
                    );
                }
            }

            err
        }

        CompileError::SSAError(msg) => {
            StructuredError::new(ErrorCode::SSAConversionError, msg)
        }

        CompileError::OptimizationError(msg) => {
            StructuredError::new(ErrorCode::OptimizationFailed, msg)
        }

        CompileError::CodeGenError(msg) => {
            StructuredError::new(ErrorCode::CodeGenFailed, msg)
        }

        CompileError::LLVMError(msg) => {
            StructuredError::new(ErrorCode::LLVMError, msg)
        }

        CompileError::IoError(path, _) => {
            StructuredError::new(
                ErrorCode::InternalCompilerError,
                format!("I/O error for file: {}", path.display()),
            )
        }

        CompileError::RuntimeError(msg) => {
            StructuredError::new(ErrorCode::InternalCompilerError, msg)
        }

        CompileError::InternalError(msg) => {
            StructuredError::new(ErrorCode::InternalCompilerError, msg)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_structured_error_creation() {
        let error = StructuredError::new(ErrorCode::StackDepthMismatch, "Test error")
            .with_location(Location::new(10, 5).with_word("test-word"))
            .with_stack_effect("( n -- n² )", "( n -- n n² )");

        assert_eq!(error.code, "E2234");
        assert_eq!(error.location.line, 10);
        assert_eq!(error.location.word, Some("test-word".to_string()));
    }

    #[test]
    fn test_json_serialization() {
        let error = StructuredError::new(ErrorCode::StackDepthMismatch, "Stack depth mismatch")
            .with_location(Location::new(5, 10))
            .with_suggestion(
                Suggestion::new("Add drop", "dup *", "dup * drop")
                    .with_confidence(0.95),
            );

        let json = error.to_json().unwrap();
        assert!(json.contains("E2234"));
        assert!(json.contains("0.95"));
    }

    #[test]
    fn test_suggestion_builder() {
        let suggestion = Suggestion::new("Add drop", "old code", "new code")
            .with_pattern("DROP_EXCESS_001")
            .with_confidence(0.9)
            .with_explanation("Removes excess stack item");

        assert_eq!(suggestion.confidence, 0.9);
        assert_eq!(suggestion.pattern, Some("DROP_EXCESS_001".to_string()));
    }
}
