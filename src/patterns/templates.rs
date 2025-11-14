//! Pattern template instantiation system

use super::{Result, PatternError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Template variable definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateVariable {
    pub name: String,
    pub description: String,
    pub example: String,
    pub required: bool,
}

/// Pattern template with instantiation capability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternTemplate {
    pub template: String,
    pub variables: Vec<TemplateVariable>,
}

impl PatternTemplate {
    /// Create a new pattern template
    pub fn new(template: String, variables: Vec<TemplateVariable>) -> Self {
        Self { template, variables }
    }

    /// Instantiate the template with provided values
    pub fn instantiate(&self, values: &HashMap<String, String>) -> Result<String> {
        let mut result = self.template.clone();

        // Check all required variables are provided
        for var in &self.variables {
            if var.required && !values.contains_key(&var.name) {
                return Err(PatternError::TemplateError(
                    format!("Missing required variable: {}", var.name)
                ));
            }
        }

        // Replace variables
        for (name, value) in values {
            result = result.replace(name, value);
        }

        Ok(result)
    }

    /// Get list of required variables
    pub fn required_variables(&self) -> Vec<&TemplateVariable> {
        self.variables.iter()
            .filter(|v| v.required)
            .collect()
    }

    /// Get list of optional variables
    pub fn optional_variables(&self) -> Vec<&TemplateVariable> {
        self.variables.iter()
            .filter(|v| !v.required)
            .collect()
    }
}

/// Instantiate a pattern template (convenience function)
pub fn instantiate_pattern(
    template: &str,
    substitutions: &HashMap<String, String>
) -> Result<String> {
    let mut result = template.to_string();

    for (var, value) in substitutions {
        result = result.replace(var, value);
    }

    Ok(result)
}

/// Common template patterns
pub mod common {
    use super::*;

    /// Recursive pattern template
    pub fn recursive_template() -> PatternTemplate {
        PatternTemplate {
            template: r#": NAME ( n -- result )
  dup BASE_CASE if
    BASE_VALUE
  else
    RECURSIVE_STEP
  then ;"#.to_string(),
            variables: vec![
                TemplateVariable {
                    name: "NAME".to_string(),
                    description: "Function name".to_string(),
                    example: "factorial".to_string(),
                    required: true,
                },
                TemplateVariable {
                    name: "BASE_CASE".to_string(),
                    description: "Base case condition".to_string(),
                    example: "2 <".to_string(),
                    required: true,
                },
                TemplateVariable {
                    name: "BASE_VALUE".to_string(),
                    description: "Base case return value".to_string(),
                    example: "drop 1".to_string(),
                    required: true,
                },
                TemplateVariable {
                    name: "RECURSIVE_STEP".to_string(),
                    description: "Recursive computation".to_string(),
                    example: "dup 1- recurse *".to_string(),
                    required: true,
                },
            ],
        }
    }

    /// Loop accumulator template
    pub fn loop_accumulator_template() -> PatternTemplate {
        PatternTemplate {
            template: r#": NAME ( n -- result )
  INIT_VALUE swap LIMIT do
    LOOP_BODY
  loop ;"#.to_string(),
            variables: vec![
                TemplateVariable {
                    name: "NAME".to_string(),
                    description: "Function name".to_string(),
                    example: "sum-to-n".to_string(),
                    required: true,
                },
                TemplateVariable {
                    name: "INIT_VALUE".to_string(),
                    description: "Initial accumulator value".to_string(),
                    example: "0".to_string(),
                    required: true,
                },
                TemplateVariable {
                    name: "LIMIT".to_string(),
                    description: "Loop limit adjustment".to_string(),
                    example: "1+".to_string(),
                    required: true,
                },
                TemplateVariable {
                    name: "LOOP_BODY".to_string(),
                    description: "Operation in loop body".to_string(),
                    example: "i +".to_string(),
                    required: true,
                },
            ],
        }
    }

    /// Conditional template
    pub fn conditional_template() -> PatternTemplate {
        PatternTemplate {
            template: r#": NAME ( INPUTS -- OUTPUTS )
  CONDITION if
    TRUE_BRANCH
  then ;"#.to_string(),
            variables: vec![
                TemplateVariable {
                    name: "NAME".to_string(),
                    description: "Function name".to_string(),
                    example: "abs".to_string(),
                    required: true,
                },
                TemplateVariable {
                    name: "INPUTS".to_string(),
                    description: "Input stack effect".to_string(),
                    example: "n".to_string(),
                    required: true,
                },
                TemplateVariable {
                    name: "OUTPUTS".to_string(),
                    description: "Output stack effect".to_string(),
                    example: "|n|".to_string(),
                    required: true,
                },
                TemplateVariable {
                    name: "CONDITION".to_string(),
                    description: "Condition to test".to_string(),
                    example: "dup 0 <".to_string(),
                    required: true,
                },
                TemplateVariable {
                    name: "TRUE_BRANCH".to_string(),
                    description: "Action if condition is true".to_string(),
                    example: "negate".to_string(),
                    required: true,
                },
            ],
        }
    }

    /// Binary operation template
    pub fn binary_op_template() -> PatternTemplate {
        PatternTemplate {
            template: r#": NAME ( a b -- c )
  OP ;"#.to_string(),
            variables: vec![
                TemplateVariable {
                    name: "NAME".to_string(),
                    description: "Function name".to_string(),
                    example: "add".to_string(),
                    required: true,
                },
                TemplateVariable {
                    name: "OP".to_string(),
                    description: "Binary operation".to_string(),
                    example: "+".to_string(),
                    required: true,
                },
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_instantiation() {
        let template = PatternTemplate::new(
            ": NAME OP ;".to_string(),
            vec![
                TemplateVariable {
                    name: "NAME".to_string(),
                    description: "Function name".to_string(),
                    example: "test".to_string(),
                    required: true,
                },
                TemplateVariable {
                    name: "OP".to_string(),
                    description: "Operation".to_string(),
                    example: "+".to_string(),
                    required: true,
                },
            ],
        );

        let mut values = HashMap::new();
        values.insert("NAME".to_string(), "add".to_string());
        values.insert("OP".to_string(), "+".to_string());

        let result = template.instantiate(&values).unwrap();
        assert_eq!(result, ": add + ;");
    }

    #[test]
    fn test_missing_required_variable() {
        let template = PatternTemplate::new(
            ": NAME OP ;".to_string(),
            vec![
                TemplateVariable {
                    name: "NAME".to_string(),
                    description: "Function name".to_string(),
                    example: "test".to_string(),
                    required: true,
                },
            ],
        );

        let values = HashMap::new();
        let result = template.instantiate(&values);
        assert!(result.is_err());
    }

    #[test]
    fn test_recursive_template() {
        let template = common::recursive_template();
        assert_eq!(template.variables.len(), 4);

        let mut values = HashMap::new();
        values.insert("NAME".to_string(), "factorial".to_string());
        values.insert("BASE_CASE".to_string(), "2 <".to_string());
        values.insert("BASE_VALUE".to_string(), "drop 1".to_string());
        values.insert("RECURSIVE_STEP".to_string(), "dup 1- recurse *".to_string());

        let result = template.instantiate(&values).unwrap();
        assert!(result.contains("factorial"));
        assert!(result.contains("recurse"));
    }
}
