//! Specification Validator
//!
//! Validates specifications for completeness and correctness

use super::{SpecError, SpecResult, Specification, TestValue};

/// Validator for specifications
pub struct SpecValidator {
    /// Enable strict validation (requires all optional fields)
    strict: bool,
}

impl SpecValidator {
    /// Create a new validator with default settings
    pub fn new() -> Self {
        Self { strict: false }
    }

    /// Create a validator with strict mode enabled
    pub fn strict() -> Self {
        Self { strict: true }
    }

    /// Validate a specification
    pub fn validate(&self, spec: &Specification) -> SpecResult<()> {
        self.validate_word_name(&spec.word)?;
        self.validate_stack_effect(spec)?;
        self.validate_test_cases(spec)?;
        self.validate_constraints(spec)?;

        if self.strict {
            self.validate_strict(spec)?;
        }

        Ok(())
    }

    /// Validate word name
    fn validate_word_name(&self, word: &str) -> SpecResult<()> {
        if word.is_empty() {
            return Err(SpecError::ValidationError(
                "Word name cannot be empty".to_string(),
            ));
        }

        // Check for valid Forth word characters
        let valid_chars = word.chars().all(|c| {
            c.is_alphanumeric()
                || matches!(c, '_' | '-' | '+' | '*' | '/' | '<' | '>' | '=' | '!' | '?')
        });

        if !valid_chars {
            return Err(SpecError::ValidationError(format!(
                "Word name '{}' contains invalid characters. \
                 Use only alphanumeric, _, -, +, *, /, <, >, =, !, ?",
                word
            )));
        }

        Ok(())
    }

    /// Validate stack effect
    fn validate_stack_effect(&self, spec: &Specification) -> SpecResult<()> {
        // Stack effect must have at least inputs or outputs
        if spec.stack_effect.inputs.is_empty() && spec.stack_effect.outputs.is_empty() {
            return Err(SpecError::StackEffectError(
                "Stack effect must have at least one input or output".to_string(),
            ));
        }

        // Validate constraints on inputs
        for (i, input) in spec.stack_effect.inputs.iter().enumerate() {
            if let Some(constraint) = &input.constraint {
                self.validate_constraint(constraint, i)?;
            }
        }

        Ok(())
    }

    /// Validate a constraint expression
    fn validate_constraint(&self, constraint: &str, _param_index: usize) -> SpecResult<()> {
        // Basic constraint validation
        // In a full implementation, this would parse and validate the constraint expression
        if constraint.is_empty() {
            return Err(SpecError::ConstraintError(
                "Constraint cannot be empty".to_string(),
            ));
        }

        // Check for common constraint patterns
        let has_operator = constraint.contains(">=")
            || constraint.contains("<=")
            || constraint.contains('>')
            || constraint.contains('<')
            || constraint.contains("==")
            || constraint.contains("!=");

        if !has_operator {
            return Err(SpecError::ConstraintError(format!(
                "Constraint '{}' should contain a comparison operator (>=, <=, >, <, ==, !=)",
                constraint
            )));
        }

        Ok(())
    }

    /// Validate test cases
    fn validate_test_cases(&self, spec: &Specification) -> SpecResult<()> {
        if let Some(test_cases) = &spec.test_cases {
            for (i, test) in test_cases.iter().enumerate() {
                // Validate input count matches stack effect
                if test.input.len() != spec.stack_effect.inputs.len() {
                    return Err(SpecError::ValidationError(format!(
                        "Test case {}: Expected {} inputs, got {}",
                        i,
                        spec.stack_effect.inputs.len(),
                        test.input.len()
                    )));
                }

                // Validate output count matches stack effect
                if test.output.len() != spec.stack_effect.outputs.len() {
                    return Err(SpecError::ValidationError(format!(
                        "Test case {}: Expected {} outputs, got {}",
                        i,
                        spec.stack_effect.outputs.len(),
                        test.output.len()
                    )));
                }

                // Validate type compatibility
                self.validate_test_types(spec, test, i)?;
            }

            // Warn if no base cases
            let has_base_case = test_cases.iter().any(|tc| {
                tc.tags
                    .as_ref()
                    .map(|tags| tags.iter().any(|t| matches!(t, super::TestTag::BaseCase)))
                    .unwrap_or(false)
            });

            if !has_base_case && self.strict {
                return Err(SpecError::ValidationError(
                    "No test cases marked as base_case".to_string(),
                ));
            }
        } else if self.strict {
            return Err(SpecError::ValidationError(
                "Strict mode requires test cases".to_string(),
            ));
        }

        Ok(())
    }

    /// Validate test value types match stack effect types
    fn validate_test_types(
        &self,
        spec: &Specification,
        test: &super::TestCase,
        test_index: usize,
    ) -> SpecResult<()> {
        // Validate input types
        for (i, (value, param)) in test.input.iter().zip(&spec.stack_effect.inputs).enumerate() {
            let compatible = match (&param.param_type, value) {
                (super::StackType::Int, TestValue::Int(_)) => true,
                (super::StackType::Uint, TestValue::Int(n)) if *n >= 0 => true,
                (super::StackType::Bool, TestValue::Bool(_)) => true,
                (super::StackType::Any, _) => true,
                _ => false,
            };

            if !compatible {
                return Err(SpecError::ValidationError(format!(
                    "Test case {}, input {}: Type mismatch. Expected {}, got {}",
                    test_index,
                    i,
                    param.param_type,
                    match value {
                        TestValue::Int(_) => "int",
                        TestValue::Bool(_) => "bool",
                        TestValue::String(_) => "string",
                    }
                )));
            }
        }

        // Validate output types
        for (i, (value, result)) in test.output.iter().zip(&spec.stack_effect.outputs).enumerate()
        {
            let compatible = match (&result.result_type, value) {
                (super::StackType::Int, TestValue::Int(_)) => true,
                (super::StackType::Uint, TestValue::Int(n)) if *n >= 0 => true,
                (super::StackType::Bool, TestValue::Bool(_)) => true,
                (super::StackType::Any, _) => true,
                _ => false,
            };

            if !compatible {
                return Err(SpecError::ValidationError(format!(
                    "Test case {}, output {}: Type mismatch. Expected {}, got {}",
                    test_index,
                    i,
                    result.result_type,
                    match value {
                        TestValue::Int(_) => "int",
                        TestValue::Bool(_) => "bool",
                        TestValue::String(_) => "string",
                    }
                )));
            }
        }

        Ok(())
    }

    /// Validate constraints on input values
    fn validate_constraints(&self, spec: &Specification) -> SpecResult<()> {
        if let Some(test_cases) = &spec.test_cases {
            for (tc_idx, test) in test_cases.iter().enumerate() {
                for (i, (value, param)) in
                    test.input.iter().zip(&spec.stack_effect.inputs).enumerate()
                {
                    if let Some(constraint) = &param.constraint {
                        if let TestValue::Int(n) = value {
                            // Basic constraint checking (would be more sophisticated in production)
                            let violated = if constraint.contains(">=") {
                                let parts: Vec<&str> = constraint.split(">=").collect();
                                if parts.len() == 2 {
                                    if let Ok(min) = parts[1].trim().parse::<i64>() {
                                        *n < min
                                    } else {
                                        false
                                    }
                                } else {
                                    false
                                }
                            } else if constraint.contains('>') && !constraint.contains(">=") {
                                let parts: Vec<&str> = constraint.split('>').collect();
                                if parts.len() == 2 {
                                    if let Ok(min) = parts[1].trim().parse::<i64>() {
                                        *n <= min
                                    } else {
                                        false
                                    }
                                } else {
                                    false
                                }
                            } else {
                                false
                            };

                            if violated {
                                return Err(SpecError::ValidationError(format!(
                                    "Test case {}, input {}: Value {} violates constraint '{}'",
                                    tc_idx, i, n, constraint
                                )));
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Strict validation (requires all optional fields)
    fn validate_strict(&self, spec: &Specification) -> SpecResult<()> {
        if spec.description.is_none() {
            return Err(SpecError::ValidationError(
                "Strict mode requires description".to_string(),
            ));
        }

        if spec.properties.is_none() || spec.properties.as_ref().unwrap().is_empty() {
            return Err(SpecError::ValidationError(
                "Strict mode requires properties".to_string(),
            ));
        }

        if spec.test_cases.is_none() || spec.test_cases.as_ref().unwrap().is_empty() {
            return Err(SpecError::ValidationError(
                "Strict mode requires test cases".to_string(),
            ));
        }

        Ok(())
    }
}

impl Default for SpecValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spec::{StackEffect, StackParameter, StackResult, StackType, TestCase, TestValue};

    #[test]
    fn test_validate_word_name() {
        let validator = SpecValidator::new();

        assert!(validator.validate_word_name("factorial").is_ok());
        assert!(validator.validate_word_name("gcd-fast").is_ok());
        assert!(validator.validate_word_name("2*").is_ok());
        assert!(validator.validate_word_name("<=").is_ok());
        assert!(validator.validate_word_name("").is_err());
        assert!(validator.validate_word_name("invalid space").is_err());
    }

    #[test]
    fn test_validate_test_case_count() {
        let validator = SpecValidator::new();

        let spec = Specification {
            word: "square".to_string(),
            description: None,
            stack_effect: StackEffect {
                inputs: vec![StackParameter {
                    name: Some("n".to_string()),
                    param_type: StackType::Int,
                    constraint: None,
                }],
                outputs: vec![StackResult {
                    name: Some("n²".to_string()),
                    result_type: StackType::Int,
                    value: None,
                }],
            },
            properties: None,
            test_cases: Some(vec![TestCase {
                description: None,
                input: vec![TestValue::Int(5)],
                output: vec![TestValue::Int(25)],
                tags: None,
            }]),
            complexity: None,
            implementation: None,
            metadata: None,
        };

        assert!(validator.validate(&spec).is_ok());

        // Test with wrong input count
        let mut bad_spec = spec.clone();
        bad_spec.test_cases = Some(vec![TestCase {
            description: None,
            input: vec![TestValue::Int(5), TestValue::Int(10)], // Too many inputs
            output: vec![TestValue::Int(25)],
            tags: None,
        }]);

        assert!(validator.validate(&bad_spec).is_err());
    }
}
