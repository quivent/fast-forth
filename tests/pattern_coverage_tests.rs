//! Pattern System Coverage Tests
//!
//! Targets uncovered code paths in pattern matching, validation,
//! template system, and pattern optimization edge cases.

use fastforth::patterns::{
    Pattern, PatternDatabase, PatternId, PatternMetadata, PatternQuery,
    PatternRegistry, PerformanceClass, TestCase, validate_pattern_metadata,
};
use std::path::PathBuf;

fn create_test_metadata(id: &str, code: &str, is_valid: bool) -> PatternMetadata {
    PatternMetadata {
        id: PatternId(id.to_string()),
        description: if is_valid { "Test pattern".to_string() } else { "".to_string() },
        category: if is_valid { "test".to_string() } else { "".to_string() },
        stack_effect: if is_valid { "( a -- b )".to_string() } else { "invalid".to_string() },
        code_template: code.to_string(),
        performance_class: PerformanceClass::Constant,
        template_variables: vec![],
        test_cases: vec![],
        tags: vec![],
        created_at: "2025-11-15".to_string(),
        updated_at: "2025-11-15".to_string(),
    }
}

#[test]
fn test_pattern_id_creation() {
    let id = PatternId("DUP_TRANSFORM_001".to_string());
    assert_eq!(id.as_str(), "DUP_TRANSFORM_001");
}

#[test]
fn test_pattern_id_validation_valid() {
    let metadata = create_test_metadata("DUP_TRANSFORM_001", ": square dup * ;", true);
    assert!(validate_pattern_metadata(&metadata).is_ok());
}

#[test]
fn test_pattern_id_validation_invalid_format() {
    let metadata = create_test_metadata("invalid_id", ": test ;", true);
    assert!(validate_pattern_metadata(&metadata).is_err());
}

#[test]
fn test_pattern_empty_template_validation() {
    let mut metadata = create_test_metadata("TEST_PATTERN_001", "", true);
    metadata.code_template = "".to_string();
    assert!(validate_pattern_metadata(&metadata).is_err());
}

#[test]
fn test_pattern_empty_description_validation() {
    let mut metadata = create_test_metadata("TEST_PATTERN_002", ": test ;", true);
    metadata.description = "".to_string();
    assert!(validate_pattern_metadata(&metadata).is_err());
}

#[test]
fn test_pattern_empty_category_validation() {
    let mut metadata = create_test_metadata("TEST_PATTERN_003", ": test ;", true);
    metadata.category = "".to_string();
    assert!(validate_pattern_metadata(&metadata).is_err());
}

#[test]
fn test_stack_effect_validation_valid() {
    let metadata = create_test_metadata("STACK_TEST_001", ": test ;", true);
    assert!(validate_pattern_metadata(&metadata).is_ok());
}

#[test]
fn test_stack_effect_validation_missing_separator() {
    let mut metadata = create_test_metadata("STACK_TEST_002", ": test ;", true);
    metadata.stack_effect = "( a b c d e )".to_string(); // Missing --
    assert!(validate_pattern_metadata(&metadata).is_err());
}

#[test]
fn test_stack_effect_validation_missing_parens() {
    let mut metadata = create_test_metadata("STACK_TEST_003", ": test ;", true);
    metadata.stack_effect = "a -- b".to_string(); // Missing parentheses
    assert!(validate_pattern_metadata(&metadata).is_err());
}

#[test]
fn test_performance_class_constant() {
    let metadata = PatternMetadata {
        id: PatternId("PERF_TEST_001".to_string()),
        description: "O(1) operation".to_string(),
        category: "test".to_string(),
        stack_effect: "( n -- n )".to_string(),
        code_template: ": test ;".to_string(),
        performance_class: PerformanceClass::Constant,
        template_variables: vec![],
        test_cases: vec![],
        tags: vec![],
        created_at: "2025-11-15".to_string(),
        updated_at: "2025-11-15".to_string(),
    };

    assert_eq!(metadata.performance_class, PerformanceClass::Constant);
}

#[test]
fn test_performance_class_linear() {
    let metadata = PatternMetadata {
        id: PatternId("PERF_TEST_002".to_string()),
        description: "O(n) operation".to_string(),
        category: "test".to_string(),
        stack_effect: "( n -- result )".to_string(),
        code_template: ": test ;".to_string(),
        performance_class: PerformanceClass::Linear,
        template_variables: vec![],
        test_cases: vec![],
        tags: vec![],
        created_at: "2025-11-15".to_string(),
        updated_at: "2025-11-15".to_string(),
    };

    assert_eq!(metadata.performance_class, PerformanceClass::Linear);
}

#[test]
fn test_pattern_database_open() {
    let temp_path = PathBuf::from("/tmp/test_patterns.db");
    let result = PatternDatabase::open(&temp_path);
    assert!(result.is_ok());
}

#[test]
fn test_pattern_database_insert_and_get() {
    let temp_path = PathBuf::from("/tmp/test_patterns_2.db");
    let mut db = PatternDatabase::open(&temp_path).unwrap();

    let pattern = Pattern {
        metadata: create_test_metadata("INSERT_TEST_001", ": test ;", true),
        usage_count: 0,
        success_rate: 1.0,
    };

    let id = pattern.metadata.id.clone();
    assert!(db.insert(pattern).is_ok());

    let retrieved = db.get(&id).unwrap();
    assert!(retrieved.is_some());
}

#[test]
fn test_pattern_query_by_category() {
    let query = PatternQuery {
        category: Some("arithmetic".to_string()),
        stack_effect: None,
        performance_class: None,
        tags: vec![],
        limit: None,
        offset: None,
    };

    assert_eq!(query.category.unwrap(), "arithmetic");
}

#[test]
fn test_pattern_registry_empty() {
    let registry = PatternRegistry::new();
    assert_eq!(registry.count(), 0);
}
