//! Fix patterns for common Forth errors
//!
//! Each pattern represents a common error and its fix strategy.

use serde::{Serialize, Deserialize};
use std::collections::HashMap;

/// A fix pattern with matching criteria and fix template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixPattern {
    pub id: String,
    pub name: String,
    pub description: String,
    pub error_code: String,
    pub pattern_match: PatternMatch,
    pub fix_template: String,
    pub base_confidence: f64,
    pub examples: Vec<FixExample>,
}

/// Pattern matching criteria
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternMatch {
    pub error_contains: Option<Vec<String>>,
    pub stack_effect_pattern: Option<String>,
    pub word_pattern: Option<String>,
}

/// Example of pattern application
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixExample {
    pub before: String,
    pub after: String,
    pub explanation: String,
}

impl FixPattern {
    pub fn matches(&self, error_msg: &str, stack_effect: Option<&str>) -> bool {
        // Check error message patterns
        if let Some(contains) = &self.pattern_match.error_contains {
            if !contains.iter().any(|pattern| error_msg.contains(pattern)) {
                return false;
            }
        }

        // Check stack effect pattern
        if let Some(pattern) = &self.pattern_match.stack_effect_pattern {
            if let Some(effect) = stack_effect {
                if !effect.contains(pattern) {
                    return false;
                }
            }
        }

        true
    }

    pub fn apply(&self, code: &str) -> Option<String> {
        // Simple template application - can be enhanced with more sophisticated logic
        Some(self.fix_template.replace("{CODE}", code))
    }
}

/// Registry of all fix patterns
pub struct PatternRegistry {
    patterns: HashMap<String, FixPattern>,
}

impl PatternRegistry {
    pub fn new() -> Self {
        let mut patterns = HashMap::new();

        // Pattern: DROP_EXCESS_001 - Remove excess stack items
        patterns.insert(
            "DROP_EXCESS_001".to_string(),
            FixPattern {
                id: "DROP_EXCESS_001".to_string(),
                name: "Drop Excess Items".to_string(),
                description: "Remove excess items from stack to match declared effect".to_string(),
                error_code: "E2234".to_string(),
                pattern_match: PatternMatch {
                    error_contains: Some(vec![
                        "stack depth".to_string(),
                        "excess".to_string(),
                    ]),
                    stack_effect_pattern: Some("--".to_string()),
                    word_pattern: None,
                },
                fix_template: "{CODE} drop".to_string(),
                base_confidence: 0.85,
                examples: vec![
                    FixExample {
                        before: "dup dup *".to_string(),
                        after: "dup dup * drop".to_string(),
                        explanation: "Stack had extra item after dup dup *, drop removes it".to_string(),
                    },
                ],
            },
        );

        // Pattern: ADD_INPUTS_002 - Add missing inputs
        patterns.insert(
            "ADD_INPUTS_002".to_string(),
            FixPattern {
                id: "ADD_INPUTS_002".to_string(),
                name: "Add Missing Inputs".to_string(),
                description: "Add DUP or literal to provide missing stack inputs".to_string(),
                error_code: "E2000".to_string(),
                pattern_match: PatternMatch {
                    error_contains: Some(vec![
                        "underflow".to_string(),
                        "insufficient".to_string(),
                    ]),
                    stack_effect_pattern: None,
                    word_pattern: None,
                },
                fix_template: "dup {CODE}".to_string(),
                base_confidence: 0.70,
                examples: vec![
                    FixExample {
                        before: "*".to_string(),
                        after: "dup *".to_string(),
                        explanation: "Multiply needs two inputs, dup provides the second".to_string(),
                    },
                ],
            },
        );

        // Pattern: ADD_THEN_003 - Close IF with THEN
        patterns.insert(
            "ADD_THEN_003".to_string(),
            FixPattern {
                id: "ADD_THEN_003".to_string(),
                name: "Add THEN".to_string(),
                description: "Close IF statement with matching THEN".to_string(),
                error_code: "E3000".to_string(),
                pattern_match: PatternMatch {
                    error_contains: Some(vec!["IF".to_string(), "unmatched".to_string()]),
                    stack_effect_pattern: None,
                    word_pattern: Some("if".to_string()),
                },
                fix_template: "{CODE} then".to_string(),
                base_confidence: 0.95,
                examples: vec![
                    FixExample {
                        before: "dup 0 < if negate".to_string(),
                        after: "dup 0 < if negate then".to_string(),
                        explanation: "Every IF must have a matching THEN".to_string(),
                    },
                ],
            },
        );

        // Pattern: ADD_LOOP_004 - Close DO with LOOP
        patterns.insert(
            "ADD_LOOP_004".to_string(),
            FixPattern {
                id: "ADD_LOOP_004".to_string(),
                name: "Add LOOP".to_string(),
                description: "Close DO statement with matching LOOP".to_string(),
                error_code: "E3010".to_string(),
                pattern_match: PatternMatch {
                    error_contains: Some(vec!["DO".to_string(), "unmatched".to_string()]),
                    stack_effect_pattern: None,
                    word_pattern: Some("do".to_string()),
                },
                fix_template: "{CODE} loop".to_string(),
                base_confidence: 0.95,
                examples: vec![
                    FixExample {
                        before: "10 0 do i .".to_string(),
                        after: "10 0 do i . loop".to_string(),
                        explanation: "Every DO must have a matching LOOP".to_string(),
                    },
                ],
            },
        );

        // Pattern: ADD_UNTIL_005 - Close BEGIN with UNTIL
        patterns.insert(
            "ADD_UNTIL_005".to_string(),
            FixPattern {
                id: "ADD_UNTIL_005".to_string(),
                name: "Add UNTIL".to_string(),
                description: "Close BEGIN statement with UNTIL, WHILE, or REPEAT".to_string(),
                error_code: "E3020".to_string(),
                pattern_match: PatternMatch {
                    error_contains: Some(vec!["BEGIN".to_string(), "unmatched".to_string()]),
                    stack_effect_pattern: None,
                    word_pattern: Some("begin".to_string()),
                },
                fix_template: "{CODE} until".to_string(),
                base_confidence: 0.85,
                examples: vec![
                    FixExample {
                        before: "begin dup 0 >".to_string(),
                        after: "begin dup 0 > until".to_string(),
                        explanation: "BEGIN loop needs termination condition with UNTIL".to_string(),
                    },
                ],
            },
        );

        // Pattern: SWAP_BEFORE_OP_006 - Add SWAP to fix operand order
        patterns.insert(
            "SWAP_BEFORE_OP_006".to_string(),
            FixPattern {
                id: "SWAP_BEFORE_OP_006".to_string(),
                name: "Swap Operands".to_string(),
                description: "Add SWAP to fix operand order for operation".to_string(),
                error_code: "E2300".to_string(),
                pattern_match: PatternMatch {
                    error_contains: Some(vec!["type".to_string(), "order".to_string()]),
                    stack_effect_pattern: None,
                    word_pattern: None,
                },
                fix_template: "swap {CODE}".to_string(),
                base_confidence: 0.75,
                examples: vec![
                    FixExample {
                        before: "5 10 -".to_string(),
                        after: "5 10 swap -".to_string(),
                        explanation: "Swaps operands to get correct order for subtraction".to_string(),
                    },
                ],
            },
        );

        // Pattern: OVER_BEFORE_OP_007 - Add OVER to duplicate second item
        patterns.insert(
            "OVER_BEFORE_OP_007".to_string(),
            FixPattern {
                id: "OVER_BEFORE_OP_007".to_string(),
                name: "Use OVER".to_string(),
                description: "Add OVER to access second stack item".to_string(),
                error_code: "E2400".to_string(),
                pattern_match: PatternMatch {
                    error_contains: Some(vec!["insufficient".to_string()]),
                    stack_effect_pattern: Some("a b".to_string()),
                    word_pattern: None,
                },
                fix_template: "over {CODE}".to_string(),
                base_confidence: 0.65,
                examples: vec![
                    FixExample {
                        before: "dup * +".to_string(),
                        after: "over dup * +".to_string(),
                        explanation: "OVER duplicates second item for use in expression".to_string(),
                    },
                ],
            },
        );

        Self { patterns }
    }

    pub fn get(&self, id: &str) -> Option<&FixPattern> {
        self.patterns.get(id)
    }

    pub fn find_matching(&self, error_code: &str, error_msg: &str) -> Vec<&FixPattern> {
        self.patterns
            .values()
            .filter(|p| p.error_code == error_code && p.matches(error_msg, None))
            .collect()
    }

    pub fn all_patterns(&self) -> Vec<&FixPattern> {
        self.patterns.values().collect()
    }
}

impl Default for PatternRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Global pattern registry
lazy_static::lazy_static! {
    pub static ref PATTERN_REGISTRY: PatternRegistry = PatternRegistry::new();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pattern_registry() {
        let registry = PatternRegistry::new();
        assert!(registry.get("DROP_EXCESS_001").is_some());
        assert!(registry.get("ADD_THEN_003").is_some());
    }

    #[test]
    fn test_pattern_matching() {
        let registry = PatternRegistry::new();
        let pattern = registry.get("DROP_EXCESS_001").unwrap();
        assert!(pattern.matches("stack depth mismatch", None));
        assert!(!pattern.matches("undefined word", None));
    }

    #[test]
    fn test_find_matching() {
        let registry = PatternRegistry::new();
        let matches = registry.find_matching("E2234", "stack depth excess");
        assert!(!matches.is_empty());
    }
}
