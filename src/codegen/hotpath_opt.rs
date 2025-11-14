//! Hot Path Optimizations - Phase 3
//!
//! Optimized versions of code generation functions with:
//! - Inline annotations for critical paths
//! - Reduced allocations
//! - Pre-allocated buffers
//! - Loop optimizations

use crate::spec::{Specification, TestCase, TestValue};
use std::fmt::Write;

/// Generate word definition (optimized hot path)
#[inline(always)]
pub fn generate_word_definition_fast(spec: &Specification) -> Result<String, std::fmt::Error> {
    // Pre-allocate with typical size (150-200 chars)
    let mut output = String::with_capacity(200);

    // Stack effect comment - use write! for efficient formatting
    write!(
        &mut output,
        ": {} {}  \\ {}\n",
        spec.word,
        spec.stack_comment(),
        spec.description.as_deref().unwrap_or("")
    )?;

    // Generate body based on pattern or heuristics
    let body = if let Some(implementation) = &spec.implementation {
        if let Some(pattern) = &implementation.pattern {
            generate_from_pattern_fast(pattern)
        } else {
            generate_heuristic_fast(&spec.word, spec.properties.as_deref())
        }
    } else {
        generate_heuristic_fast(&spec.word, spec.properties.as_deref())
    };

    write!(&mut output, "  {}\n;\n", body)?;

    Ok(output)
}

/// Generate from pattern (optimized)
#[inline(always)]
fn generate_from_pattern_fast(pattern: &str) -> &'static str {
    // Use match for O(1) lookup vs HashMap
    match pattern {
        "DUP_TRANSFORM_001" => "dup *",
        "CONDITIONAL_NEGATE_002" => "dup 0 < if negate then",
        "ACCUMULATOR_LOOP_003" => "0 swap 1+ 1 do i + loop",
        "RECURSIVE_004" => "dup 2 < if drop 1 else dup 1- recurse * then",
        "TAIL_RECURSIVE_008" => "begin dup while swap over mod repeat drop",
        _ => "( TODO: implement )",
    }
}

/// Generate heuristic (optimized)
#[inline(always)]
fn generate_heuristic_fast(word: &str, properties: Option<&[String]>) -> &'static str {
    // Fast path: common patterns
    if word.contains("square") || word.ends_with("²") {
        return "dup *";
    }
    if word.contains("abs") {
        return "dup 0 < if negate then";
    }
    if word.contains("double") || word == "2*" {
        return "2 *";
    }
    if word.contains("half") || word == "2/" {
        return "2 /";
    }

    // Check properties for hints
    if let Some(props) = properties {
        for prop in props {
            if prop.contains("factorial") || prop.contains("recurse") {
                return "dup 2 < if drop 1 else dup 1- recurse * then";
            }
            if prop.contains("mod") {
                return "begin dup while swap over mod repeat drop";
            }
        }
    }

    "( TODO: implement )"
}

/// Generate test harness (optimized)
#[inline]
pub fn generate_test_harness_fast(
    word: &str,
    test_cases: &[TestCase]
) -> String {
    // Pre-allocate: ~50 chars per test case
    let mut output = String::with_capacity(test_cases.len() * 50 + 100);

    write!(&mut output, "\\ Test harness for {}\n", word).unwrap();
    output.push_str("\\ Run these tests to verify correctness\n\n");

    for (i, test) in test_cases.iter().enumerate() {
        // Add description if present
        if let Some(desc) = &test.description {
            write!(&mut output, "\\ Test {}: {}\n", i + 1, desc).unwrap();
        } else {
            write!(&mut output, "\\ Test {}\n", i + 1).unwrap();
        }

        // Generate input string without intermediate allocation
        let input_str = format_test_values(&test.input);
        let output_str = format_test_values(&test.output);

        write!(&mut output, "T{{ {} {} -> {} }}T\n", input_str, word, output_str).unwrap();
    }

    output
}

/// Format test values (optimized)
#[inline(always)]
fn format_test_values(values: &[TestValue]) -> String {
    // Pre-allocate: ~8 chars per value
    let mut result = String::with_capacity(values.len() * 8);

    for (i, v) in values.iter().enumerate() {
        if i > 0 {
            result.push(' ');
        }
        match v {
            TestValue::Int(n) => write!(&mut result, "{}", n).unwrap(),
            TestValue::Bool(b) => result.push_str(if *b { "true" } else { "false" }),
            TestValue::String(s) => write!(&mut result, "\"{}\"", s).unwrap(),
        }
    }

    result
}

/// Generate provenance metadata (optimized)
#[inline]
pub fn generate_provenance_fast(spec: &Specification) -> String {
    let mut output = String::with_capacity(256);

    output.push_str("\\ GENERATED METADATA\n");

    if let Some(metadata) = &spec.metadata {
        if let Some(author) = &metadata.author {
            write!(&mut output, "\\   AUTHOR: {}\n", author).unwrap();
        }
        if let Some(version) = &metadata.version {
            write!(&mut output, "\\   VERSION: {}\n", version).unwrap();
        }
        if let Some(created) = &metadata.created {
            write!(&mut output, "\\   CREATED: {}\n", created).unwrap();
        }
    }

    if let Some(implementation) = &spec.implementation {
        if let Some(pattern) = &implementation.pattern {
            write!(&mut output, "\\   PATTERN: {}\n", pattern).unwrap();
        }
    }

    if let Some(complexity) = &spec.complexity {
        if let Some(time) = &complexity.time {
            write!(&mut output, "\\   TIME_COMPLEXITY: {}\n", time).unwrap();
        }
        if let Some(space) = &complexity.space {
            write!(&mut output, "\\   SPACE_COMPLEXITY: {}\n", space).unwrap();
        }
    }

    output.push('\n');
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spec::{
        Implementation, StackEffect, StackParameter, StackResult, StackType,
    };

    #[test]
    fn test_generate_from_pattern_fast() {
        assert_eq!(generate_from_pattern_fast("DUP_TRANSFORM_001"), "dup *");
        assert_eq!(
            generate_from_pattern_fast("CONDITIONAL_NEGATE_002"),
            "dup 0 < if negate then"
        );
    }

    #[test]
    fn test_generate_heuristic_fast() {
        assert_eq!(generate_heuristic_fast("square", None), "dup *");
        assert_eq!(generate_heuristic_fast("abs", None), "dup 0 < if negate then");
        assert_eq!(generate_heuristic_fast("double", None), "2 *");
    }

    #[test]
    fn test_format_test_values() {
        let values = vec![TestValue::Int(5), TestValue::Int(10)];
        assert_eq!(format_test_values(&values), "5 10");
    }
}
