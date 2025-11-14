//! Compiler integration for pattern validation

use super::{PatternId, Result, PatternError};
use super::validation::{extract_pattern_id_from_code, validate_pattern_in_code};

/// Pattern validation during compilation
pub struct PatternValidator {
    strict_mode: bool,
}

impl PatternValidator {
    /// Create a new pattern validator
    pub fn new(strict_mode: bool) -> Self {
        Self { strict_mode }
    }

    /// Validate pattern metadata in code
    pub fn validate_code(&self, code: &str) -> Result<Option<PatternId>> {
        // Extract pattern ID from code comments
        let pattern_id = extract_pattern_id_from_code(code);

        if self.strict_mode && pattern_id.is_none() {
            return Err(PatternError::ValidationError(
                "Pattern ID required in strict mode but not found".to_string()
            ));
        }

        Ok(pattern_id)
    }

    /// Validate that code matches expected pattern
    pub fn validate_pattern_match(&self, code: &str, expected_id: &PatternId) -> Result<()> {
        validate_pattern_in_code(code, expected_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validator_extracts_pattern() {
        let validator = PatternValidator::new(false);
        let code = r#"
\ PATTERN: DUP_TRANSFORM_001
: square ( n -- n² )
  dup * ;
"#;
        let result = validator.validate_code(code).unwrap();
        assert_eq!(result, Some(PatternId("DUP_TRANSFORM_001".to_string())));
    }

    #[test]
    fn test_strict_mode_requires_pattern() {
        let validator = PatternValidator::new(true);
        let code = ": square dup * ;";
        assert!(validator.validate_code(code).is_err());
    }
}
