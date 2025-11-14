//! Pattern ID System and Pattern Library for Fast Forth
//!
//! This module provides:
//! - Canonical pattern identifiers (e.g., DUP_TRANSFORM_001, RECURSIVE_004)
//! - Pattern metadata and validation
//! - SQLite-based pattern database
//! - CLI and HTTP API for pattern queries
//! - Pattern template instantiation

pub mod registry;
pub mod database;
pub mod templates;
pub mod template_jit;
pub mod http;
pub mod validation;
pub mod cli;
pub mod integration;

pub use registry::{PatternRegistry, Pattern, PatternCategory};
pub use database::{PatternDatabase, PatternQuery};
pub use templates::{PatternTemplate, TemplateVariable, instantiate_pattern};
pub use template_jit::{instantiate_compiled, compile_and_cache};
pub use http::{PatternServer, PatternApiConfig};
pub use validation::{validate_pattern_metadata, PatternValidationError};
pub use cli::{PatternCli, PatternCommand, execute_pattern_command};
pub use integration::PatternValidator;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Pattern identifier (e.g., DUP_TRANSFORM_001)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PatternId(pub String);

impl PatternId {
    pub fn new(category: &str, number: u32) -> Self {
        Self(format!("{}_{:03}", category.to_uppercase(), number))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for PatternId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Performance class for patterns
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PerformanceClass {
    Constant,      // O(1)
    Logarithmic,   // O(log n)
    Linear,        // O(n)
    Linearithmic,  // O(n log n)
    Quadratic,     // O(n²)
    Exponential,   // O(2^n)
}

impl std::fmt::Display for PerformanceClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Constant => write!(f, "O(1)"),
            Self::Logarithmic => write!(f, "O(log n)"),
            Self::Linear => write!(f, "O(n)"),
            Self::Linearithmic => write!(f, "O(n log n)"),
            Self::Quadratic => write!(f, "O(n²)"),
            Self::Exponential => write!(f, "O(2^n)"),
        }
    }
}

/// Test case for pattern validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCase {
    pub input: Vec<i64>,
    pub output: Vec<i64>,
    pub description: Option<String>,
}

/// Pattern metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternMetadata {
    pub id: PatternId,
    pub category: String,
    pub stack_effect: String,
    pub code_template: String,
    pub performance_class: PerformanceClass,
    pub test_cases: Vec<TestCase>,
    pub description: String,
    pub tags: Vec<String>,
    pub template_variables: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Result type for pattern operations
pub type Result<T> = std::result::Result<T, PatternError>;

/// Pattern system errors
#[derive(Debug, thiserror::Error)]
pub enum PatternError {
    #[error("Pattern not found: {0}")]
    NotFound(String),

    #[error("Invalid pattern ID: {0}")]
    InvalidId(String),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Template error: {0}")]
    TemplateError(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("HTTP server error: {0}")]
    HttpError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pattern_id_creation() {
        let id = PatternId::new("DUP_TRANSFORM", 1);
        assert_eq!(id.as_str(), "DUP_TRANSFORM_001");
    }

    #[test]
    fn test_performance_class_display() {
        assert_eq!(PerformanceClass::Constant.to_string(), "O(1)");
        assert_eq!(PerformanceClass::Linear.to_string(), "O(n)");
        assert_eq!(PerformanceClass::Quadratic.to_string(), "O(n²)");
    }
}
