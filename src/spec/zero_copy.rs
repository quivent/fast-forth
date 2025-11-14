//! Zero-Copy Deserialization - Phase 3 Optimization
//!
//! Uses rkyv for zero-copy deserialization of specifications in hot paths.
//! Target: Reduce JSON parsing overhead from 12.4ms → 4ms (3x improvement)

use rkyv::{Archive, Deserialize, Serialize};
use super::{SpecError, SpecResult};

/// Archived-friendly stack type
#[derive(Archive, Deserialize, Serialize, Debug, Clone, PartialEq)]
#[archive(check_bytes)]
#[repr(u8)]
pub enum ArchivedStackType {
    Int = 0,
    Uint = 1,
    Bool = 2,
    Char = 3,
    Addr = 4,
    Any = 5,
}

/// Archived stack parameter
#[derive(Archive, Deserialize, Serialize, Debug, Clone)]
#[archive(check_bytes)]
pub struct ArchivedStackParameter {
    pub name: Option<String>,
    pub param_type: ArchivedStackType,
    pub constraint: Option<String>,
}

/// Archived stack result
#[derive(Archive, Deserialize, Serialize, Debug, Clone)]
#[archive(check_bytes)]
pub struct ArchivedStackResult {
    pub name: Option<String>,
    pub result_type: ArchivedStackType,
    pub value: Option<String>,
}

/// Archived stack effect
#[derive(Archive, Deserialize, Serialize, Debug, Clone)]
#[archive(check_bytes)]
pub struct ArchivedStackEffect {
    pub inputs: Vec<ArchivedStackParameter>,
    pub outputs: Vec<ArchivedStackResult>,
}

impl ArchivedStackEffect {
    /// Format as Forth-style stack comment (zero-copy when possible)
    #[inline]
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

/// Archived specification for zero-copy validation
#[derive(Archive, Deserialize, Serialize, Debug, Clone)]
#[archive(check_bytes)]
pub struct ArchivedSpecification {
    pub word: String,
    pub description: Option<String>,
    pub stack_effect: ArchivedStackEffect,
    pub properties: Option<Vec<String>>,
}

impl ArchivedSpecification {
    /// Validate this specification (zero-copy)
    #[inline]
    pub fn validate_fast(&self) -> SpecResult<()> {
        // Basic validation without deserialization
        if self.word.is_empty() {
            return Err(SpecError::ValidationError(
                "Word name cannot be empty".to_string()
            ));
        }

        if self.stack_effect.inputs.is_empty() && self.stack_effect.outputs.is_empty() {
            return Err(SpecError::ValidationError(
                "Stack effect must have at least one input or output".to_string()
            ));
        }

        Ok(())
    }

    /// Get the Forth-style stack effect comment (zero-copy)
    #[inline]
    pub fn stack_comment(&self) -> String {
        self.stack_effect.to_forth_comment()
    }
}

/// Serialize specification to rkyv format for fast loading
pub fn serialize_spec(spec: &ArchivedSpecification) -> Result<Vec<u8>, SpecError> {
    rkyv::to_bytes::<_, 256>(spec)
        .map(|bytes| bytes.to_vec())
        .map_err(|e| SpecError::ValidationError(format!("Serialization failed: {}", e)))
}

/// Deserialize specification from rkyv format (zero-copy)
pub fn deserialize_spec(bytes: &[u8]) -> Result<&rkyv::Archived<ArchivedSpecification>, SpecError> {
    rkyv::check_archived_root::<ArchivedSpecification>(bytes)
        .map_err(|e| SpecError::ValidationError(format!("Deserialization failed: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_archived_stack_effect() {
        let effect = ArchivedStackEffect {
            inputs: vec![ArchivedStackParameter {
                name: Some("n".to_string()),
                param_type: ArchivedStackType::Int,
                constraint: None,
            }],
            outputs: vec![ArchivedStackResult {
                name: Some("n²".to_string()),
                result_type: ArchivedStackType::Int,
                value: Some("n*n".to_string()),
            }],
        };

        let comment = effect.to_forth_comment();
        assert_eq!(comment, "( n -- n² )");
    }

    #[test]
    fn test_validate_fast() {
        let spec = ArchivedSpecification {
            word: "square".to_string(),
            description: Some("Square a number".to_string()),
            stack_effect: ArchivedStackEffect {
                inputs: vec![ArchivedStackParameter {
                    name: Some("n".to_string()),
                    param_type: ArchivedStackType::Int,
                    constraint: None,
                }],
                outputs: vec![ArchivedStackResult {
                    name: Some("n²".to_string()),
                    result_type: ArchivedStackType::Int,
                    value: Some("n*n".to_string()),
                }],
            },
            properties: None,
        };

        assert!(spec.validate_fast().is_ok());
    }

    #[test]
    fn test_serialize_deserialize() {
        let spec = ArchivedSpecification {
            word: "test".to_string(),
            description: None,
            stack_effect: ArchivedStackEffect {
                inputs: vec![],
                outputs: vec![],
            },
            properties: None,
        };

        let bytes = serialize_spec(&spec).unwrap();
        let archived = deserialize_spec(&bytes).unwrap();
        assert_eq!(archived.word, "test");
    }
}
