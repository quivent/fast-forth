//! Code Generation from Specifications
//!
//! Generates Forth code from machine-readable specifications

use crate::spec::{SpecError, SpecResult, Specification};

/// Code generator for specifications
pub struct SpecCodeGenerator {
    /// Include provenance comments in generated code
    include_provenance: bool,

    /// Include test harness
    include_tests: bool,
}

impl SpecCodeGenerator {
    /// Create a new code generator with default settings
    pub fn new() -> Self {
        Self {
            include_provenance: true,
            include_tests: true,
        }
    }

    /// Set whether to include provenance metadata
    pub fn with_provenance(mut self, include: bool) -> Self {
        self.include_provenance = include;
        self
    }

    /// Set whether to include test harness
    pub fn with_tests(mut self, include: bool) -> Self {
        self.include_tests = include;
        self
    }

    /// Generate Forth code from specification
    pub fn generate(&self, spec: &Specification) -> SpecResult<String> {
        // Validate specification first
        spec.validate()?;

        // Phase 1 optimization: Pre-allocate buffer with estimated capacity
        // Typical generated code is 200-500 chars, so 512 is a good default
        let mut output = String::with_capacity(512);

        // Add header comment using write! for more efficient string building
        use std::fmt::Write;
        write!(&mut output, "\\ Generated from specification: {}\n", spec.word)
            .map_err(|_| SpecError::ValidationError("Failed to write header".to_string()))?;

        if let Some(desc) = &spec.description {
            write!(&mut output, "\\ {}\n", desc)
                .map_err(|_| SpecError::ValidationError("Failed to write description".to_string()))?;
        }

        output.push('\n');

        // Add provenance metadata if requested
        if self.include_provenance {
            output.push_str(&self.generate_provenance(spec));
        }

        // Add properties as comments
        if let Some(properties) = &spec.properties {
            output.push_str("\\ Properties:\n");
            for prop in properties {
                output.push_str(&format!("\\   {}\n", prop));
            }
            output.push('\n');
        }

        // Generate word definition
        output.push_str(&self.generate_word_definition(spec)?);

        // Add test harness if requested
        if self.include_tests {
            if let Some(test_cases) = &spec.test_cases {
                if !test_cases.is_empty() {
                    output.push('\n');
                    output.push_str(&self.generate_test_harness(spec));
                }
            }
        }

        Ok(output)
    }

    /// Generate provenance metadata
    fn generate_provenance(&self, spec: &Specification) -> String {
        // Phase 1 optimization: Pre-allocate buffer for metadata (typically ~200 chars)
        let mut output = String::with_capacity(256);
        use std::fmt::Write;

        output.push_str("\\ GENERATED METADATA\n");

        if let Some(metadata) = &spec.metadata {
            if let Some(author) = &metadata.author {
                let _ = write!(&mut output, "\\   AUTHOR: {}\n", author);
            }
            if let Some(version) = &metadata.version {
                let _ = write!(&mut output, "\\   VERSION: {}\n", version);
            }
            if let Some(created) = &metadata.created {
                let _ = write!(&mut output, "\\   CREATED: {}\n", created);
            }
        }

        if let Some(implementation) = &spec.implementation {
            if let Some(pattern) = &implementation.pattern {
                let _ = write!(&mut output, "\\   PATTERN: {}\n", pattern);
            }
        }

        if let Some(complexity) = &spec.complexity {
            if let Some(time) = &complexity.time {
                let _ = write!(&mut output, "\\   TIME_COMPLEXITY: {}\n", time);
            }
            if let Some(space) = &complexity.space {
                let _ = write!(&mut output, "\\   SPACE_COMPLEXITY: {}\n", space);
            }
        }

        output.push('\n');
        output
    }

    /// Generate word definition based on pattern or heuristics
    fn generate_word_definition(&self, spec: &Specification) -> SpecResult<String> {
        // Phase 1 optimization: Pre-allocate buffer (typical word definition ~150 chars)
        let mut output = String::with_capacity(200);
        use std::fmt::Write;

        // Stack effect comment - use write! for efficient formatting
        write!(
            &mut output,
            ": {} {}  \\ {}\n",
            spec.word,
            spec.stack_comment(),
            spec.description.as_deref().unwrap_or("")
        ).map_err(|_| SpecError::ValidationError("Failed to write word definition".to_string()))?;

        // Generate body based on pattern or heuristics
        let body = if let Some(implementation) = &spec.implementation {
            if let Some(pattern) = &implementation.pattern {
                self.generate_from_pattern(spec, pattern)?
            } else {
                self.generate_heuristic(spec)?
            }
        } else {
            self.generate_heuristic(spec)?
        };

        write!(&mut output, "  {}\n;\n", body)
            .map_err(|_| SpecError::ValidationError("Failed to write body".to_string()))?;

        Ok(output)
    }

    /// Generate implementation from pattern ID
    fn generate_from_pattern(&self, spec: &Specification, pattern: &str) -> SpecResult<String> {
        match pattern {
            "DUP_TRANSFORM_001" => {
                // Pattern: dup <operation>
                // Example: square = dup *
                Ok("dup *".to_string())
            }

            "CONDITIONAL_NEGATE_002" => {
                // Pattern: dup 0 < if negate then
                // Example: abs
                Ok("dup 0 < if negate then".to_string())
            }

            "ACCUMULATOR_LOOP_003" => {
                // Pattern: accumulator loop
                // This is a placeholder - would need more context
                Ok("0 swap 1+ 1 do i + loop".to_string())
            }

            "RECURSIVE_004" => {
                // Pattern: recursive with base case
                // Generate recursive template
                self.generate_recursive_template(spec)
            }

            "TAIL_RECURSIVE_008" => {
                // Pattern: tail recursive with loop
                Ok("begin dup while swap over mod repeat drop".to_string())
            }

            _ => Err(SpecError::ValidationError(format!(
                "Unknown pattern: {}",
                pattern
            ))),
        }
    }

    /// Generate recursive template
    fn generate_recursive_template(&self, spec: &Specification) -> SpecResult<String> {
        // Analyze properties to find base case
        let base_case = if let Some(properties) = &spec.properties {
            properties
                .iter()
                .find(|p| p.contains("(0)") || p.contains("(1)"))
                .and_then(|p| {
                    if p.contains("= 1") {
                        Some("dup 2 < if drop 1 else")
                    } else if p.contains("= 0") {
                        Some("dup 0 = if drop 0 else")
                    } else {
                        None
                    }
                })
                .unwrap_or("dup 2 < if drop 1 else")
        } else {
            "dup 2 < if drop 1 else"
        };

        // Generate recursive step
        let recursive_step = "dup 1- recurse *";

        Ok(format!("{} {} then", base_case, recursive_step))
    }

    /// Generate implementation using heuristics
    fn generate_heuristic(&self, spec: &Specification) -> SpecResult<String> {
        // Analyze word name and properties to generate reasonable implementation

        // Check for common patterns in word name
        if spec.word.contains("square") || spec.word.ends_with("²") {
            return Ok("dup *".to_string());
        }

        if spec.word.contains("abs") {
            return Ok("dup 0 < if negate then".to_string());
        }

        if spec.word.contains("double") || spec.word == "2*" {
            return Ok("2 *".to_string());
        }

        if spec.word.contains("half") || spec.word == "2/" {
            return Ok("2 /".to_string());
        }

        // Analyze properties for hints
        if let Some(properties) = &spec.properties {
            for prop in properties {
                if prop.contains("* factorial") || prop.contains("recurse") {
                    return self.generate_recursive_template(spec);
                }
                if prop.contains("mod") {
                    return Ok("begin dup while swap over mod repeat drop".to_string());
                }
            }
        }

        // Default placeholder implementation
        Ok("( TODO: implement )".to_string())
    }

    /// Generate test harness
    fn generate_test_harness(&self, spec: &Specification) -> String {
        let mut output = String::new();

        output.push_str(&format!(
            "\\ Test harness for {}\n",
            spec.word
        ));
        output.push_str("\\ Run these tests to verify correctness\n\n");

        if let Some(test_cases) = &spec.test_cases {
            for (i, test) in test_cases.iter().enumerate() {
                // Add description if present
                if let Some(desc) = &test.description {
                    output.push_str(&format!("\\ Test {}: {}\n", i + 1, desc));
                } else {
                    output.push_str(&format!("\\ Test {}\n", i + 1));
                }

                // Generate ANS Forth test format: T{ input -> output }T
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
            }
        }

        output
    }
}

impl Default for SpecCodeGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spec::{
        Implementation, StackEffect, StackParameter, StackResult, StackType, TestCase, TestValue,
    };

    #[test]
    fn test_generate_simple_word() {
        let spec = Specification {
            word: "square".to_string(),
            description: Some("Calculates the square of a number".to_string()),
            stack_effect: StackEffect {
                inputs: vec![StackParameter {
                    name: Some("n".to_string()),
                    param_type: StackType::Int,
                    constraint: None,
                }],
                outputs: vec![StackResult {
                    name: Some("n²".to_string()),
                    result_type: StackType::Int,
                    value: Some("n²".to_string()),
                }],
            },
            properties: Some(vec!["square(n) = n * n".to_string()]),
            test_cases: Some(vec![TestCase {
                description: Some("Square of 5".to_string()),
                input: vec![TestValue::Int(5)],
                output: vec![TestValue::Int(25)],
                tags: None,
            }]),
            complexity: None,
            implementation: Some(Implementation {
                pattern: Some("DUP_TRANSFORM_001".to_string()),
                hints: None,
            }),
            metadata: None,
        };

        let generator = SpecCodeGenerator::new();
        let code = generator.generate(&spec).unwrap();

        assert!(code.contains(": square"));
        assert!(code.contains("dup *"));
        assert!(code.contains("T{ 5 square -> 25 }T"));
    }
}
