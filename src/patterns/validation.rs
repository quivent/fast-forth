//! Pattern validation system

use super::{PatternMetadata, PatternId, Result, PatternError};
use regex::Regex;

/// Pattern validation error
#[derive(Debug, thiserror::Error)]
pub enum PatternValidationError {
    #[error("Invalid pattern ID format: {0}")]
    InvalidIdFormat(String),

    #[error("Invalid stack effect: {0}")]
    InvalidStackEffect(String),

    #[error("Missing required field: {0}")]
    MissingField(String),

    #[error("Invalid template: {0}")]
    InvalidTemplate(String),

    #[error("Test case validation failed: {0}")]
    TestCaseError(String),
}

/// Validate pattern metadata
pub fn validate_pattern_metadata(metadata: &PatternMetadata) -> Result<()> {
    // Validate pattern ID format
    validate_pattern_id(&metadata.id)?;

    // Validate stack effect format
    validate_stack_effect(&metadata.stack_effect)?;

    // Validate template is not empty
    if metadata.code_template.is_empty() {
        return Err(PatternError::ValidationError(
            "Code template cannot be empty".to_string()
        ));
    }

    // Validate description
    if metadata.description.is_empty() {
        return Err(PatternError::ValidationError(
            "Description cannot be empty".to_string()
        ));
    }

    // Validate category
    if metadata.category.is_empty() {
        return Err(PatternError::ValidationError(
            "Category cannot be empty".to_string()
        ));
    }

    Ok(())
}

/// Validate pattern ID format (e.g., DUP_TRANSFORM_001)
fn validate_pattern_id(id: &PatternId) -> Result<()> {
    let re = Regex::new(r"^[A-Z_]+_\d{3}$").unwrap();

    if !re.is_match(id.as_str()) {
        return Err(PatternError::ValidationError(
            format!("Invalid pattern ID format: {}. Expected format: CATEGORY_NNN", id)
        ));
    }

    Ok(())
}

/// Validate stack effect format (e.g., ( n -- n² ))
fn validate_stack_effect(effect: &str) -> Result<()> {
    // Basic validation: must contain ( -- )
    if !effect.contains("--") {
        return Err(PatternError::ValidationError(
            format!("Invalid stack effect: {}. Must contain '--'", effect)
        ));
    }

    if !effect.starts_with("(") || !effect.ends_with(")") {
        return Err(PatternError::ValidationError(
            format!("Invalid stack effect: {}. Must be enclosed in parentheses", effect)
        ));
    }

    Ok(())
}

/// Extract pattern ID from code comments
pub fn extract_pattern_id_from_code(code: &str) -> Option<PatternId> {
    let re = Regex::new(r"\\\ PATTERN:\s*([A-Z_]+_\d{3})").unwrap();

    re.captures(code)
        .and_then(|cap| cap.get(1))
        .map(|m| PatternId(m.as_str().to_string()))
}

/// Validate pattern is present in code
pub fn validate_pattern_in_code(code: &str, expected_id: &PatternId) -> Result<()> {
    match extract_pattern_id_from_code(code) {
        Some(found_id) if found_id == *expected_id => Ok(()),
        Some(found_id) => Err(PatternError::ValidationError(
            format!("Pattern ID mismatch: expected {}, found {}", expected_id, found_id)
        )),
        None => Err(PatternError::ValidationError(
            format!("Pattern ID {} not found in code", expected_id)
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_pattern_id_valid() {
        let id = PatternId("DUP_TRANSFORM_001".to_string());
        assert!(validate_pattern_id(&id).is_ok());
    }

    #[test]
    fn test_validate_pattern_id_invalid() {
        let id = PatternId("invalid".to_string());
        assert!(validate_pattern_id(&id).is_err());
    }

    #[test]
    fn test_validate_stack_effect_valid() {
        assert!(validate_stack_effect("( n -- n² )").is_ok());
        assert!(validate_stack_effect("( a b -- c )").is_ok());
    }

    #[test]
    fn test_validate_stack_effect_invalid() {
        assert!(validate_stack_effect("n -- n²").is_err());
        assert!(validate_stack_effect("( invalid )").is_err());
    }

    #[test]
    fn test_extract_pattern_id_from_code() {
        let code = r#"
\ PATTERN: DUP_TRANSFORM_001
: square ( n -- n² )
  dup * ;
"#;
        let id = extract_pattern_id_from_code(code);
        assert_eq!(id, Some(PatternId("DUP_TRANSFORM_001".to_string())));
    }

    #[test]
    fn test_extract_pattern_id_not_found() {
        let code = ": square dup * ;";
        assert_eq!(extract_pattern_id_from_code(code), None);
    }
}
