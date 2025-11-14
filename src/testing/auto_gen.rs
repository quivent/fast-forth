//! Automatic Test Generation
//!
//! Generates comprehensive test suites from specifications

use crate::spec::{SpecError, SpecResult, Specification, StackType, TestCase, TestTag, TestValue};
use std::collections::HashSet;

/// Test generator configuration
pub struct TestGenerator {
    /// Generate base case tests
    generate_base_cases: bool,

    /// Generate edge case tests
    generate_edge_cases: bool,

    /// Generate boundary tests
    generate_boundary_tests: bool,

    /// Generate property-based tests
    generate_property_tests: bool,

    /// Number of random property tests to generate
    random_test_count: usize,
}

impl TestGenerator {
    /// Create a new test generator with default settings
    pub fn new() -> Self {
        Self {
            generate_base_cases: true,
            generate_edge_cases: true,
            generate_boundary_tests: true,
            generate_property_tests: true,
            random_test_count: 5,
        }
    }

    /// Set whether to generate base case tests
    pub fn with_base_cases(mut self, generate: bool) -> Self {
        self.generate_base_cases = generate;
        self
    }

    /// Set whether to generate edge case tests
    pub fn with_edge_cases(mut self, generate: bool) -> Self {
        self.generate_edge_cases = generate;
        self
    }

    /// Set whether to generate boundary tests
    pub fn with_boundary_tests(mut self, generate: bool) -> Self {
        self.generate_boundary_tests = generate;
        self
    }

    /// Set whether to generate property tests
    pub fn with_property_tests(mut self, generate: bool) -> Self {
        self.generate_property_tests = generate;
        self
    }

    /// Set number of random property tests
    pub fn with_random_count(mut self, count: usize) -> Self {
        self.random_test_count = count;
        self
    }

    /// Generate comprehensive test suite from specification
    pub fn generate(&self, spec: &Specification) -> SpecResult<Vec<TestCase>> {
        let mut tests = Vec::new();

        // Include existing test cases from spec
        if let Some(existing_tests) = &spec.test_cases {
            tests.extend(existing_tests.clone());
        }

        // Track which inputs we've already tested to avoid duplicates
        let mut tested_inputs: HashSet<Vec<i64>> = HashSet::new();

        for test in &tests {
            if let Some(inputs) = self.extract_int_inputs(&test.input) {
                tested_inputs.insert(inputs);
            }
        }

        // Generate base cases
        if self.generate_base_cases {
            let base_cases = self.generate_base_cases_from_properties(spec);
            for test in base_cases {
                if let Some(inputs) = self.extract_int_inputs(&test.input) {
                    if !tested_inputs.contains(&inputs) {
                        tested_inputs.insert(inputs);
                        tests.push(test);
                    }
                }
            }
        }

        // Generate edge cases
        if self.generate_edge_cases {
            let edge_cases = self.generate_edge_cases_from_types(spec);
            for test in edge_cases {
                if let Some(inputs) = self.extract_int_inputs(&test.input) {
                    if !tested_inputs.contains(&inputs) {
                        tested_inputs.insert(inputs);
                        tests.push(test);
                    }
                }
            }
        }

        // Generate boundary tests
        if self.generate_boundary_tests {
            let boundary_tests = self.generate_boundary_tests_from_constraints(spec);
            for test in boundary_tests {
                if let Some(inputs) = self.extract_int_inputs(&test.input) {
                    if !tested_inputs.contains(&inputs) {
                        tested_inputs.insert(inputs);
                        tests.push(test);
                    }
                }
            }
        }

        // Generate property-based tests
        if self.generate_property_tests {
            let property_tests = self.generate_property_tests_from_spec(spec);
            for test in property_tests {
                if let Some(inputs) = self.extract_int_inputs(&test.input) {
                    if !tested_inputs.contains(&inputs) {
                        tested_inputs.insert(inputs);
                        tests.push(test);
                    }
                }
            }
        }

        Ok(tests)
    }

    /// Extract integer inputs from test values (for deduplication)
    fn extract_int_inputs(&self, inputs: &[TestValue]) -> Option<Vec<i64>> {
        inputs
            .iter()
            .map(|v| match v {
                TestValue::Int(n) => Some(*n),
                _ => None,
            })
            .collect()
    }

    /// Generate base case tests from properties
    fn generate_base_cases_from_properties(&self, spec: &Specification) -> Vec<TestCase> {
        let mut tests = Vec::new();

        if let Some(properties) = &spec.properties {
            for prop in properties {
                // Look for base case patterns like "f(0) = 1" or "f(1) = 1"
                if let Some(test) = self.parse_base_case_property(spec, prop) {
                    tests.push(test);
                }
            }
        }

        tests
    }

    /// Parse a base case property into a test case
    fn parse_base_case_property(&self, spec: &Specification, property: &str) -> Option<TestCase> {
        // Simple pattern matching for "word(n) = result"
        let word = &spec.word;

        // Try to match pattern: "word(n) = result"
        if let Some(pos) = property.find(&format!("{}(", word)) {
            let after_paren = &property[pos + word.len() + 1..];

            if let Some(close_paren) = after_paren.find(')') {
                let input_str = &after_paren[..close_paren].trim();

                if let Ok(input_val) = input_str.parse::<i64>() {
                    // Find the result after '='
                    if let Some(eq_pos) = after_paren.find('=') {
                        let result_str = after_paren[eq_pos + 1..].trim();

                        if let Ok(result_val) = result_str.parse::<i64>() {
                            return Some(TestCase {
                                description: Some(format!("Base case: {}", property)),
                                input: vec![TestValue::Int(input_val)],
                                output: vec![TestValue::Int(result_val)],
                                tags: Some(vec![TestTag::BaseCase]),
                            });
                        }
                    }
                }
            }
        }

        None
    }

    /// Generate edge case tests based on stack types
    fn generate_edge_cases_from_types(&self, spec: &Specification) -> Vec<TestCase> {
        let mut tests = Vec::new();
        let input_count = spec.stack_effect.inputs.len();

        // Generate tests for common edge values
        let edge_values: Vec<i64> = vec![0, 1, -1, 2, 10, 100];

        // Generate single-input edge cases
        if input_count == 1 {
            for &val in &edge_values {
                // Skip if constraint would be violated
                if !self.violates_constraints(spec, &[val]) {
                    if let Some(output) = self.calculate_output(spec, &[val]) {
                        tests.push(TestCase {
                            description: Some(format!("Edge case: input = {}", val)),
                            input: vec![TestValue::Int(val)],
                            output: vec![TestValue::Int(output)],
                            tags: Some(vec![TestTag::EdgeCase]),
                        });
                    }
                }
            }
        } else if input_count == 2 {
            // Generate two-input edge cases
            for &val1 in &[0, 1, 10] {
                for &val2 in &[0, 1, 10] {
                    if !self.violates_constraints(spec, &[val1, val2]) {
                        if let Some(output) = self.calculate_output(spec, &[val1, val2]) {
                            tests.push(TestCase {
                                description: Some(format!("Edge case: inputs = {}, {}", val1, val2)),
                                input: vec![TestValue::Int(val1), TestValue::Int(val2)],
                                output: vec![TestValue::Int(output)],
                                tags: Some(vec![TestTag::EdgeCase]),
                            });
                        }
                    }
                }
            }
        }

        tests
    }

    /// Generate boundary tests from constraints
    fn generate_boundary_tests_from_constraints(&self, spec: &Specification) -> Vec<TestCase> {
        let mut tests = Vec::new();

        for (i, input) in spec.stack_effect.inputs.iter().enumerate() {
            if let Some(constraint) = &input.constraint {
                // Parse constraint to find boundary values
                if let Some(boundary_values) = self.extract_boundary_values(constraint) {
                    for val in boundary_values {
                        // Create input vector with boundary value at position i
                        let mut inputs = vec![1i64; spec.stack_effect.inputs.len()];
                        inputs[i] = val;

                        if !self.violates_constraints(spec, &inputs) {
                            if let Some(output) = self.calculate_output(spec, &inputs) {
                                let input_values: Vec<TestValue> =
                                    inputs.iter().map(|&n| TestValue::Int(n)).collect();

                                tests.push(TestCase {
                                    description: Some(format!(
                                        "Boundary test: {} at constraint boundary",
                                        input.name.as_deref().unwrap_or("input")
                                    )),
                                    input: input_values,
                                    output: vec![TestValue::Int(output)],
                                    tags: Some(vec![TestTag::Boundary]),
                                });
                            }
                        }
                    }
                }
            }
        }

        tests
    }

    /// Extract boundary values from constraint expression
    fn extract_boundary_values(&self, constraint: &str) -> Option<Vec<i64>> {
        let mut values = Vec::new();

        // Look for numeric values in the constraint
        for part in constraint.split_whitespace() {
            if let Ok(num) = part.trim_matches(|c: char| !c.is_numeric() && c != '-').parse::<i64>()
            {
                values.push(num);
                values.push(num - 1); // Just below boundary
                values.push(num + 1); // Just above boundary
            }
        }

        if values.is_empty() {
            None
        } else {
            Some(values)
        }
    }

    /// Generate property-based tests
    fn generate_property_tests_from_spec(&self, spec: &Specification) -> Vec<TestCase> {
        let mut tests = Vec::new();

        // Generate random valid inputs and compute outputs
        for i in 0..self.random_test_count {
            let inputs = self.generate_random_inputs(spec, i);

            if !self.violates_constraints(spec, &inputs) {
                if let Some(output) = self.calculate_output(spec, &inputs) {
                    let input_values: Vec<TestValue> =
                        inputs.iter().map(|&n| TestValue::Int(n)).collect();

                    tests.push(TestCase {
                        description: Some(format!("Property test {}", i + 1)),
                        input: input_values,
                        output: vec![TestValue::Int(output)],
                        tags: Some(vec![TestTag::Property]),
                    });
                }
            }
        }

        tests
    }

    /// Generate random inputs for testing
    fn generate_random_inputs(&self, spec: &Specification, seed: usize) -> Vec<i64> {
        // Simple pseudo-random generation (not cryptographically secure)
        let mut values = Vec::new();

        for i in 0..spec.stack_effect.inputs.len() {
            // Use seed to generate deterministic "random" values
            let value = ((seed + i) * 7 + 3) as i64 % 20;
            values.push(value);
        }

        values
    }

    /// Check if inputs violate constraints
    fn violates_constraints(&self, spec: &Specification, inputs: &[i64]) -> bool {
        for (i, input) in spec.stack_effect.inputs.iter().enumerate() {
            if let Some(constraint) = &input.constraint {
                if i < inputs.len() {
                    let val = inputs[i];

                    // Simple constraint checking
                    if constraint.contains(">=") {
                        if let Some(min_str) = constraint.split(">=").nth(1) {
                            if let Ok(min) = min_str.trim().parse::<i64>() {
                                if val < min {
                                    return true;
                                }
                            }
                        }
                    }

                    if constraint.contains('>') && !constraint.contains(">=") {
                        if let Some(min_str) = constraint.split('>').nth(1) {
                            if let Ok(min) = min_str.trim().parse::<i64>() {
                                if val <= min {
                                    return true;
                                }
                            }
                        }
                    }
                }
            }
        }

        false
    }

    /// Calculate expected output from inputs (simplified)
    fn calculate_output(&self, spec: &Specification, inputs: &[i64]) -> Option<i64> {
        // This is a simplified calculation based on common patterns
        // In a full implementation, this would use symbolic execution or property analysis

        let word = spec.word.as_str();

        match word {
            "square" => inputs.first().map(|&n| n * n),
            "double" | "2*" => inputs.first().map(|&n| n * 2),
            "half" | "2/" => inputs.first().map(|&n| n / 2),
            "abs" => inputs.first().map(|&n| n.abs()),
            "negate" => inputs.first().map(|&n| -n),
            "factorial" => inputs.first().and_then(|&n| {
                if n < 0 {
                    None
                } else if n <= 1 {
                    Some(1)
                } else {
                    let mut result = 1i64;
                    for i in 2..=n {
                        result = result.checked_mul(i)?;
                    }
                    Some(result)
                }
            }),
            _ => {
                // For unknown words, try to infer from properties
                None
            }
        }
    }

    /// Generate ANS Forth test format output
    pub fn generate_forth_tests(&self, spec: &Specification, tests: &[TestCase]) -> String {
        let mut output = String::new();

        output.push_str(&format!(
            "\\ Auto-generated tests for {}\n",
            spec.word
        ));
        output.push_str(&format!("\\ Generated {} test cases\n\n", tests.len()));

        // Group tests by tag
        let base_cases: Vec<_> = tests
            .iter()
            .filter(|t| {
                t.tags
                    .as_ref()
                    .map(|tags| tags.contains(&TestTag::BaseCase))
                    .unwrap_or(false)
            })
            .collect();

        let edge_cases: Vec<_> = tests
            .iter()
            .filter(|t| {
                t.tags
                    .as_ref()
                    .map(|tags| tags.contains(&TestTag::EdgeCase))
                    .unwrap_or(false)
            })
            .collect();

        let boundary_cases: Vec<_> = tests
            .iter()
            .filter(|t| {
                t.tags
                    .as_ref()
                    .map(|tags| tags.contains(&TestTag::Boundary))
                    .unwrap_or(false)
            })
            .collect();

        let property_tests: Vec<_> = tests
            .iter()
            .filter(|t| {
                t.tags
                    .as_ref()
                    .map(|tags| tags.contains(&TestTag::Property))
                    .unwrap_or(false)
            })
            .collect();

        // Output base cases
        if !base_cases.is_empty() {
            output.push_str("\\ Base Cases\n");
            for test in base_cases {
                output.push_str(&self.format_test_case(spec, test));
            }
            output.push('\n');
        }

        // Output edge cases
        if !edge_cases.is_empty() {
            output.push_str("\\ Edge Cases\n");
            for test in edge_cases {
                output.push_str(&self.format_test_case(spec, test));
            }
            output.push('\n');
        }

        // Output boundary tests
        if !boundary_cases.is_empty() {
            output.push_str("\\ Boundary Tests\n");
            for test in boundary_cases {
                output.push_str(&self.format_test_case(spec, test));
            }
            output.push('\n');
        }

        // Output property tests
        if !property_tests.is_empty() {
            output.push_str("\\ Property-Based Tests\n");
            for test in property_tests {
                output.push_str(&self.format_test_case(spec, test));
            }
            output.push('\n');
        }

        output
    }

    /// Format a single test case in ANS Forth format
    fn format_test_case(&self, spec: &Specification, test: &TestCase) -> String {
        let mut output = String::new();

        if let Some(desc) = &test.description {
            output.push_str(&format!("\\   {}\n", desc));
        }

        let input_str = test
            .input
            .iter()
            .map(|v| v.to_string())
            .collect::<Vec<_>>()
            .join(" ");

        let output_str = test
            .output
            .iter()
            .map(|v| v.to_string())
            .collect::<Vec<_>>()
            .join(" ");

        output.push_str(&format!(
            "T{{ {} {} -> {} }}T\n",
            input_str, spec.word, output_str
        ));

        output
    }
}

impl Default for TestGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spec::{StackEffect, StackParameter, StackResult};

    #[test]
    fn test_generate_base_cases() {
        let spec = Specification {
            word: "factorial".to_string(),
            description: None,
            stack_effect: StackEffect {
                inputs: vec![StackParameter {
                    name: Some("n".to_string()),
                    param_type: StackType::Int,
                    constraint: Some("n >= 0".to_string()),
                }],
                outputs: vec![StackResult {
                    name: Some("n!".to_string()),
                    result_type: StackType::Int,
                    value: Some("n!".to_string()),
                }],
            },
            properties: Some(vec![
                "factorial(0) = 1".to_string(),
                "factorial(1) = 1".to_string(),
            ]),
            test_cases: None,
            complexity: None,
            implementation: None,
            metadata: None,
        };

        let generator = TestGenerator::new();
        let tests = generator.generate(&spec).unwrap();

        assert!(!tests.is_empty());

        // Check that base cases were generated
        let base_cases = tests
            .iter()
            .filter(|t| {
                t.tags
                    .as_ref()
                    .map(|tags| tags.contains(&TestTag::BaseCase))
                    .unwrap_or(false)
            })
            .collect::<Vec<_>>();

        assert!(!base_cases.is_empty());
    }
}
