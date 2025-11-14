//! Structured error messages for agent consumption
//!
//! This module provides machine-readable error formats with:
//! - Unique error codes (E0001-E9999)
//! - Expected vs actual comparisons
//! - Precise location information
//! - Auto-fix suggestions with confidence scores
//! - JSON serialization for agent parsers

pub mod error_code;
pub mod structured;
pub mod formatter;

pub use error_code::{ErrorCode, ERROR_CODE_REGISTRY};
pub use structured::{StructuredError, Location, Suggestion, FixDiff, ErrorSeverity};
pub use formatter::{ErrorFormatter, OutputFormat};

use serde::{Serialize, Deserialize};

/// Convert a ForthError to a StructuredError with auto-fix suggestions
pub fn to_structured_error(
    error: &crate::error::CompileError,
    suggest_fixes: bool,
) -> StructuredError {
    structured::convert_to_structured(error, suggest_fixes)
}

/// Format error for specific output format
pub fn format_error(
    error: &StructuredError,
    format: OutputFormat,
) -> String {
    formatter::format_error(error, format)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_code_generation() {
        let code = ErrorCode::StackDepthMismatch;
        assert_eq!(code.as_str(), "E0234");
    }

    #[test]
    fn test_structured_error_json() {
        let error = StructuredError::new(
            ErrorCode::StackDepthMismatch,
            "Stack depth mismatch".to_string(),
        );
        let json = serde_json::to_string(&error).unwrap();
        assert!(json.contains("E0234"));
    }
}
