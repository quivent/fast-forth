//! Machine-Readable Specification System
//!
//! This module provides machine-readable specifications for Forth words,
//! enabling AI agents to generate correct code from JSON specifications.

use serde::{Deserialize, Serialize};
use std::path::Path;
use thiserror::Error;

pub mod validator;
pub mod zero_copy;

pub use validator::SpecValidator;
pub use zero_copy::{ArchivedSpecification, ArchivedStackEffect, serialize_spec, deserialize_spec};

/// Errors that can occur during specification processing
#[derive(Error, Debug)]
pub enum SpecError {
    #[error("Failed to read specification file: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Failed to parse JSON: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Invalid stack effect: {0}")]
    StackEffectError(String),

    #[error("Invalid constraint: {0}")]
    ConstraintError(String),
}

pub type SpecResult<T> = Result<T, SpecError>;

/// Type for stack parameters
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum StackType {
    Int,
    Uint,
    Bool,
    Char,
    Addr,
    Any,
}

impl std::fmt::Display for StackType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StackType::Int => write!(f, "int"),
            StackType::Uint => write!(f, "uint"),
            StackType::Bool => write!(f, "bool"),
            StackType::Char => write!(f, "char"),
            StackType::Addr => write!(f, "addr"),
            StackType::Any => write!(f, "any"),
        }
    }
}

/// Stack parameter with optional constraints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StackParameter {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    #[serde(rename = "type")]
    pub param_type: StackType,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub constraint: Option<String>,
}

/// Stack result with optional value expression
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StackResult {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    #[serde(rename = "type")]
    pub result_type: StackType,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
}

/// Stack effect declaration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StackEffect {
    pub inputs: Vec<StackParameter>,
    pub outputs: Vec<StackResult>,
}

impl StackEffect {
    /// Format as Forth-style stack comment
    pub fn to_forth_comment(&self) -> String {
        let inputs = self.inputs
            .iter()
            .map(|p| p.name.as_deref().unwrap_or("x"))
            .collect::<Vec<_>>()
            .join(" ");

        let outputs = self.outputs
            .iter()
            .map(|r| r.name.as_deref().unwrap_or("y"))
            .collect::<Vec<_>>()
            .join(" ");

        format!("( {} -- {} )", inputs, outputs)
    }
}

/// Test case value (can be int, bool, or string)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum TestValue {
    Int(i64),
    Bool(bool),
    String(String),
}

impl std::fmt::Display for TestValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TestValue::Int(n) => write!(f, "{}", n),
            TestValue::Bool(b) => write!(f, "{}", if *b { "true" } else { "false" }),
            TestValue::String(s) => write!(f, "\"{}\"", s),
        }
    }
}

/// Test case tags for categorization
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TestTag {
    BaseCase,
    EdgeCase,
    Boundary,
    Error,
    Performance,
    Property,
}

/// Individual test case
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCase {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    pub input: Vec<TestValue>,
    pub output: Vec<TestValue>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<TestTag>>,
}

/// Complexity bounds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Complexity {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub space: Option<String>,
}

/// Implementation hints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Implementation {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pattern: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub hints: Option<Vec<String>>,
}

/// Specification metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub created: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
}

/// Complete machine-readable specification for a Forth word
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Specification {
    /// Name of the word
    pub word: String,

    /// Human-readable description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Stack effect declaration
    pub stack_effect: StackEffect,

    /// Mathematical/logical properties
    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties: Option<Vec<String>>,

    /// Test cases
    #[serde(skip_serializing_if = "Option::is_none")]
    pub test_cases: Option<Vec<TestCase>>,

    /// Complexity bounds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub complexity: Option<Complexity>,

    /// Implementation hints
    #[serde(skip_serializing_if = "Option::is_none")]
    pub implementation: Option<Implementation>,

    /// Metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Metadata>,
}

impl Specification {
    /// Load specification from JSON file
    pub fn from_file<P: AsRef<Path>>(path: P) -> SpecResult<Self> {
        let content = std::fs::read_to_string(path)?;
        Self::from_json(&content)
    }

    /// Parse specification from JSON string (with SIMD optimization)
    pub fn from_json(json: &str) -> SpecResult<Self> {
        // Try SIMD JSON parsing first (12.4ms → 8ms - Phase 2 optimization)
        let mut json_bytes = json.as_bytes().to_vec();

        match simd_json::from_slice::<Specification>(&mut json_bytes) {
            Ok(spec) => Ok(spec),
            Err(_) => {
                // Fallback to standard serde_json if SIMD fails
                // (e.g., on platforms without SIMD support)
                let spec: Specification = serde_json::from_str(json)?;
                Ok(spec)
            }
        }
    }

    /// Validate this specification
    pub fn validate(&self) -> SpecResult<()> {
        let validator = SpecValidator::new();
        validator.validate(self)
    }

    /// Convert to pretty-printed JSON
    pub fn to_json_pretty(&self) -> SpecResult<String> {
        Ok(serde_json::to_string_pretty(self)?)
    }

    /// Get the Forth-style stack effect comment
    pub fn stack_comment(&self) -> String {
        self.stack_effect.to_forth_comment()
    }

    /// Count total test cases
    pub fn test_count(&self) -> usize {
        self.test_cases.as_ref().map(|tc| tc.len()).unwrap_or(0)
    }

    /// Get test cases by tag
    pub fn tests_by_tag(&self, tag: TestTag) -> Vec<&TestCase> {
        self.test_cases
            .as_ref()
            .map(|cases| {
                cases
                    .iter()
                    .filter(|tc| {
                        tc.tags
                            .as_ref()
                            .map(|tags| tags.contains(&tag))
                            .unwrap_or(false)
                    })
                    .collect()
            })
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_spec() {
        let json = r#"{
            "word": "square",
            "stack_effect": {
                "inputs": [{"type": "int"}],
                "outputs": [{"type": "int"}]
            }
        }"#;

        let spec = Specification::from_json(json).unwrap();
        assert_eq!(spec.word, "square");
        assert_eq!(spec.stack_effect.inputs.len(), 1);
        assert_eq!(spec.stack_effect.outputs.len(), 1);
    }

    #[test]
    fn test_stack_comment() {
        let spec = Specification {
            word: "add".to_string(),
            description: None,
            stack_effect: StackEffect {
                inputs: vec![
                    StackParameter {
                        name: Some("a".to_string()),
                        param_type: StackType::Int,
                        constraint: None,
                    },
                    StackParameter {
                        name: Some("b".to_string()),
                        param_type: StackType::Int,
                        constraint: None,
                    },
                ],
                outputs: vec![StackResult {
                    name: Some("sum".to_string()),
                    result_type: StackType::Int,
                    value: Some("a+b".to_string()),
                }],
            },
            properties: None,
            test_cases: None,
            complexity: None,
            implementation: None,
            metadata: None,
        };

        assert_eq!(spec.stack_comment(), "( a b -- sum )");
    }
}
