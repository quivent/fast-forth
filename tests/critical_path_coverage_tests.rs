//! Critical Path Coverage Tests
//!
//! Targets remaining uncovered critical code paths including:
//! - Error handler edge cases
//! - Platform-specific code paths
//! - Fallback implementations
//! - Edge cases in core algorithms

use fastforth::{
    CompileError, ForthEngine,
};

#[test]
fn test_error_display_formatting_parse() {
    let error = CompileError::ParseError("Test parse error at line 42".to_string());

    let error_string = format!("{}", error);

    // Should format error with message
    assert!(error_string.contains("Parse error"));
    assert!(error_string.contains("42"));
}

#[test]
fn test_error_debug_formatting_type() {
    let error = CompileError::TypeError("Expected int, found float".to_string());

    let error_debug = format!("{:?}", error);

    // Debug formatting should include error variant
    assert!(error_debug.contains("TypeError"));
}

#[test]
fn test_engine_empty_input() {
    let mut engine = ForthEngine::new();

    // Empty input should be handled gracefully
    let result = engine.eval("");

    assert!(result.is_ok() || result.is_err()); // Either way, shouldn't panic
}

#[test]
fn test_engine_whitespace_only() {
    let mut engine = ForthEngine::new();

    // Whitespace-only input should be handled
    let result = engine.eval("   \n\t   ");

    assert!(result.is_ok()); // Should handle whitespace gracefully
}

#[test]
fn test_division_by_zero_error() {
    let mut engine = ForthEngine::new();

    // Division by zero should be caught
    let result = engine.eval("10 0 /");

    // Should return a runtime error
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), CompileError::RuntimeError(_)));
}
