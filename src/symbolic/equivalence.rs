//! Equivalence Checking
//!
//! Determines if two implementations are semantically equivalent

use super::{SymbolicExecutor, SymbolicValue};
use fastforth_frontend::{Program, Definition};
use serde::{Serialize, Deserialize};

/// Result of equivalence checking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EquivalenceResult {
    pub equivalent: bool,
    pub reason: String,
    pub left_output: Vec<String>,
    pub right_output: Vec<String>,
    pub differences: Vec<String>,
}

/// Equivalence checker using symbolic execution
pub struct EquivalenceChecker {
    max_inputs: usize,
}

impl EquivalenceChecker {
    pub fn new() -> Self {
        Self { max_inputs: 10 }
    }

    /// Check if two programs are equivalent
    pub fn check_programs(
        &self,
        left: &Program,
        right: &Program,
    ) -> EquivalenceResult {
        // For simplicity, assume both have same number of inputs
        let input_count = self.infer_input_count(left).max(self.infer_input_count(right));

        let mut left_executor = SymbolicExecutor::new();
        let mut right_executor = SymbolicExecutor::new();

        left_executor.initialize_inputs(input_count);
        right_executor.initialize_inputs(input_count);

        let left_result = left_executor.execute_program(left);
        let right_result = right_executor.execute_program(right);

        match (left_result, right_result) {
            (Ok(left_res), Ok(right_res)) => {
                let equivalent = self.compare_outputs(&left_res.final_stack, &right_res.final_stack);

                EquivalenceResult {
                    equivalent,
                    reason: if equivalent {
                        "Outputs are symbolically equivalent".to_string()
                    } else {
                        "Outputs differ".to_string()
                    },
                    left_output: left_res.final_stack.clone(),
                    right_output: right_res.final_stack.clone(),
                    differences: if equivalent {
                        vec![]
                    } else {
                        self.find_differences(&left_res.final_stack, &right_res.final_stack)
                    },
                }
            }
            (Err(e), _) => EquivalenceResult {
                equivalent: false,
                reason: format!("Left execution failed: {}", e),
                left_output: vec![],
                right_output: vec![],
                differences: vec![],
            },
            (_, Err(e)) => EquivalenceResult {
                equivalent: false,
                reason: format!("Right execution failed: {}", e),
                left_output: vec![],
                right_output: vec![],
                differences: vec![],
            },
        }
    }

    /// Check if two definitions are equivalent
    pub fn check_definitions(
        &self,
        left: &Definition,
        right: &Definition,
    ) -> EquivalenceResult {
        let left_prog = Program {
            definitions: vec![],
            top_level_code: left.body.clone(),
        };

        let right_prog = Program {
            definitions: vec![],
            top_level_code: right.body.clone(),
        };

        self.check_programs(&left_prog, &right_prog)
    }

    /// Compare output stacks
    fn compare_outputs(&self, left: &[String], right: &[String]) -> bool {
        if left.len() != right.len() {
            return false;
        }

        // For now, just do string comparison
        // In a full implementation, would do semantic comparison
        left == right
    }

    /// Find differences between outputs
    fn find_differences(&self, left: &[String], right: &[String]) -> Vec<String> {
        let mut differences = Vec::new();

        if left.len() != right.len() {
            differences.push(format!(
                "Stack depth differs: {} vs {}",
                left.len(),
                right.len()
            ));
        }

        for (i, (l, r)) in left.iter().zip(right.iter()).enumerate() {
            if l != r {
                differences.push(format!("Output {} differs: {} vs {}", i, l, r));
            }
        }

        differences
    }

    /// Infer the number of inputs a program needs
    fn infer_input_count(&self, program: &Program) -> usize {
        // Simplified heuristic
        // In a full implementation, would use stack effect inference
        3
    }
}

impl Default for EquivalenceChecker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fastforth_frontend::parse_program;

    #[test]
    fn test_equivalent_programs() {
        let left = parse_program(": square dup * ;").unwrap();
        let right = parse_program(": square dup * ;").unwrap();

        let checker = EquivalenceChecker::new();
        let result = checker.check_programs(&left, &right);

        assert!(result.equivalent);
    }

    #[test]
    fn test_different_programs() {
        let left = parse_program(": double 2 * ;").unwrap();
        let right = parse_program(": triple 3 * ;").unwrap();

        let checker = EquivalenceChecker::new();
        // Compare the actual definitions, not the programs
        let result = checker.check_definitions(&left.definitions[0], &right.definitions[0]);

        assert!(!result.equivalent);
    }
}
