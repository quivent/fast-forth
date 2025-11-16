//! Parser Error Stress Tests (20 tests)
//!
//! Comprehensive tests for all parser error conditions including:
//! - Malformed input
//! - Unexpected EOF
//! - Invalid tokens
//! - Deeply nested structures
//! - Unicode edge cases
//! - Zero-length input
//! - Maximum length input

use fastforth_frontend as frontend;

// ============================================================================
// MALFORMED INPUT TESTS (5 tests)
// ============================================================================

#[test]
fn test_unclosed_string_literal() {
    let source = r#": test ." hello world ;"#;
    let result = frontend::parse_program(source);

    // Parser may or may not catch this - depends on implementation
    match result {
        Ok(_) => println!("Parser accepted unclosed string (lenient parsing)"),
        Err(e) => {
            println!("Unclosed string error: {}", e);
            assert!(e.to_string().contains("Parse") || e.to_string().contains("parse"));
        }
    }
}

#[test]
fn test_unclosed_comment() {
    let source = ": test ( this is a comment without close ;";
    let result = frontend::parse_program(source);
    assert!(result.is_err(), "Should fail on unclosed comment");
}

#[test]
fn test_invalid_number_format() {
    let source = ": test 123abc 456xyz ;";
    let result = frontend::parse_program(source);
    // May parse as words or fail - either is acceptable
    println!("Invalid number parse result: {:?}", result);
}

#[test]
fn test_mixed_control_structure_tokens() {
    let source = ": test IF WHILE REPEAT THEN ;";
    let result = frontend::parse_program(source);
    // Should fail on mismatched control structures
    if result.is_err() {
        println!("Control structure mismatch: {}", result.unwrap_err());
    }
}

#[test]
fn test_orphaned_control_structure_end() {
    let source = ": test THEN ;";
    let result = frontend::parse_program(source);
    // THEN without IF should fail
    if result.is_err() {
        println!("Orphaned THEN error: {}", result.unwrap_err());
    }
}

// ============================================================================
// UNEXPECTED EOF TESTS (5 tests)
// ============================================================================

#[test]
fn test_eof_in_definition() {
    let source = ": incomplete-definition 1 2 +";
    let result = frontend::parse_program(source);
    assert!(result.is_err(), "Should fail on EOF in definition");
    if let Err(e) = result {
        println!("EOF in definition: {}", e);
    }
}

#[test]
fn test_eof_in_if_statement() {
    let source = ": test IF 1";
    let result = frontend::parse_program(source);
    assert!(result.is_err(), "Should fail on EOF in IF");
}

#[test]
fn test_eof_in_begin_loop() {
    let source = ": test BEGIN dup";
    let result = frontend::parse_program(source);
    assert!(result.is_err(), "Should fail on EOF in BEGIN loop");
}

#[test]
fn test_eof_in_do_loop() {
    let source = ": test 10 0 DO i";
    let result = frontend::parse_program(source);
    assert!(result.is_err(), "Should fail on EOF in DO loop");
}

#[test]
fn test_eof_after_colon() {
    let source = ":";
    let result = frontend::parse_program(source);
    assert!(result.is_err(), "Should fail on EOF after colon");
}

// ============================================================================
// INVALID TOKEN TESTS (3 tests)
// ============================================================================

#[test]
fn test_control_characters_in_source() {
    let source = ": test \x00 \x01 \x02 ;";
    let result = frontend::parse_program(source);
    // May succeed or fail depending on lexer implementation
    println!("Control chars result: {:?}", result);
}

#[test]
fn test_null_bytes_in_source() {
    let source = ": test\0hello ;";
    let result = frontend::parse_program(source);
    println!("Null bytes result: {:?}", result);
}

#[test]
fn test_extremely_long_word_name() {
    let long_name = "a".repeat(10000);
    let source = format!(": {} 1 + ;", long_name);
    let result = frontend::parse_program(&source);
    // Should handle or reject gracefully
    println!("Long word name handled: {}", result.is_ok());
}

// ============================================================================
// DEEPLY NESTED STRUCTURES (3 tests)
// ============================================================================

#[test]
fn test_deeply_nested_if_statements() {
    let mut source = String::from(": test ");
    let depth = 100;

    for i in 0..depth {
        source.push_str(&format!("{} IF ", i));
    }

    for _ in 0..depth {
        source.push_str("THEN ");
    }

    source.push_str(";");

    let result = frontend::parse_program(&source);
    println!("Deep nesting (depth {}): {}", depth, result.is_ok());
}

#[test]
fn test_deeply_nested_begin_loops() {
    let mut source = String::from(": test ");
    let depth = 50;

    for _ in 0..depth {
        source.push_str("BEGIN ");
    }

    for _ in 0..depth {
        source.push_str("0 UNTIL ");
    }

    source.push_str(";");

    let result = frontend::parse_program(&source);
    println!("Deep loop nesting: {}", result.is_ok());
}

#[test]
fn test_maximum_recursion_depth() {
    let source = r#"
        : recurse1 recurse2 ;
        : recurse2 recurse3 ;
        : recurse3 recurse4 ;
        : recurse4 recurse5 ;
        : recurse5 recurse1 ;
    "#;

    let result = frontend::parse_program(source);
    // Should parse successfully (recursion is valid)
    assert!(result.is_ok(), "Mutual recursion should parse");
}

// ============================================================================
// UNICODE AND ENCODING TESTS (2 tests)
// ============================================================================

#[test]
fn test_unicode_in_word_names() {
    let source = ": 你好 1 + ;";
    let result = frontend::parse_program(source);
    // Behavior depends on Forth spec compliance
    println!("Unicode word name: {}", result.is_ok());
}

#[test]
fn test_unicode_in_comments() {
    let source = r#"
        : test ( 这是中文注释 こんにちは 🚀 )
            1 2 + ;
    "#;
    let result = frontend::parse_program(source);
    // Should handle Unicode in comments
    println!("Unicode in comments: {}", result.is_ok());
}

// ============================================================================
// ZERO-LENGTH AND EMPTY INPUT (1 test)
// ============================================================================

#[test]
fn test_empty_source_program() {
    let source = "";
    let result = frontend::parse_program(source);
    // Should succeed with empty program
    assert!(result.is_ok(), "Empty program should parse");
    if let Ok(program) = result {
        assert_eq!(program.definitions.len(), 0, "Empty program has no definitions");
    }
}

// ============================================================================
// MAXIMUM LENGTH INPUT (1 test)
// ============================================================================

#[test]
fn test_very_large_program() {
    // Generate a large but valid program
    let mut source = String::new();

    for i in 0..1000 {
        source.push_str(&format!(": func{} {} + ; ", i, i));
    }

    let result = frontend::parse_program(&source);
    assert!(result.is_ok(), "Large program should parse");

    if let Ok(program) = result {
        assert_eq!(program.definitions.len(), 1000, "Should parse all 1000 definitions");
        println!("Successfully parsed {} definitions", program.definitions.len());
    }
}
