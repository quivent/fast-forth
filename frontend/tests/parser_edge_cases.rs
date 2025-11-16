//! Comprehensive edge case tests for the Forth parser
//!
//! This test suite targets untested code paths including:
//! - Whitespace handling edge cases
//! - Comment edge cases
//! - Literal parsing edge cases
//! - Control structure edge cases
//! - Word definition edge cases

use fastforth_frontend::*;

// ============================================================================
// WHITESPACE HANDLING EDGE CASES (5 tests)
// ============================================================================

#[test]
fn test_multiple_spaces_between_tokens() {
    let source = ":    double     2     *     ;";
    let program = parse_program(source).expect("Failed to parse");
    assert_eq!(program.definitions.len(), 1);
    assert_eq!(program.definitions[0].name, "double");
    assert_eq!(program.definitions[0].body.len(), 2);
}

#[test]
fn test_tabs_and_spaces_mixed() {
    let source = ":\tdouble\t2\t*\t;";
    let program = parse_program(source).expect("Failed to parse");
    assert_eq!(program.definitions.len(), 1);
    assert_eq!(program.definitions[0].name, "double");
}

#[test]
fn test_leading_trailing_whitespace() {
    let source = "   \n\t  : double 2 * ;  \n\t   ";
    let program = parse_program(source).expect("Failed to parse");
    assert_eq!(program.definitions.len(), 1);
    assert_eq!(program.definitions[0].name, "double");
}

#[test]
fn test_empty_lines_between_definitions() {
    let source = r#"
        : first 1 ;



        : second 2 ;

    "#;
    let program = parse_program(source).expect("Failed to parse");
    assert_eq!(program.definitions.len(), 2);
    assert_eq!(program.definitions[0].name, "first");
    assert_eq!(program.definitions[1].name, "second");
}

#[test]
fn test_newlines_in_definition() {
    let source = r#"
        : triple
            3
            *
        ;
    "#;
    let program = parse_program(source).expect("Failed to parse");
    assert_eq!(program.definitions.len(), 1);
    assert_eq!(program.definitions[0].body.len(), 2);
}

// ============================================================================
// COMMENT HANDLING EDGE CASES (3 tests)
// ============================================================================

#[test]
fn test_nested_parenthesized_comments() {
    let source = ": test ( outer ( inner ) still-comment ) 1 ;";
    let program = parse_program(source).expect("Failed to parse");
    assert_eq!(program.definitions.len(), 1);
    assert_eq!(program.definitions[0].body.len(), 1);
}

#[test]
fn test_comment_after_colon() {
    let source = r#"
        : ( this is a comment right after colon ) test 1 ;
    "#;
    let program = parse_program(source).expect("Failed to parse");
    assert_eq!(program.definitions.len(), 1);
    assert_eq!(program.definitions[0].name, "test");
}

#[test]
fn test_line_comment_with_backslash() {
    let source = r#"
        \ This is a line comment
        : test 1 ;
        \ Another comment
    "#;
    let program = parse_program(source).expect("Failed to parse");
    assert_eq!(program.definitions.len(), 1);
    assert_eq!(program.definitions[0].name, "test");
}

// ============================================================================
// LITERAL PARSING EDGE CASES (5 tests)
// ============================================================================

#[test]
fn test_very_large_positive_number() {
    let source = ": test 9223372036854775807 ;"; // i64::MAX
    let program = parse_program(source).expect("Failed to parse");
    assert_eq!(program.definitions.len(), 1);
    match &program.definitions[0].body[0] {
        Word::IntLiteral(n) => assert_eq!(*n, i64::MAX),
        _ => panic!("Expected IntLiteral"),
    }
}

#[test]
fn test_very_large_negative_number() {
    let source = ": test -9223372036854775808 ;"; // i64::MIN
    let program = parse_program(source).expect("Failed to parse");
    assert_eq!(program.definitions.len(), 1);
    match &program.definitions[0].body[0] {
        Word::IntLiteral(n) => assert_eq!(*n, i64::MIN),
        _ => panic!("Expected IntLiteral"),
    }
}

#[test]
fn test_negative_numbers_in_stack_effect() {
    let source = ": test ( n -- n-1 ) -1 + ;";
    let program = parse_program(source).expect("Failed to parse");
    assert_eq!(program.definitions.len(), 1);
    assert!(program.definitions[0].stack_effect.is_some());
}

#[test]
fn test_string_literal_with_escapes() {
    let source = r#": test "line1\nline2\ttabbed\r\n\"quoted\"\\backslash" ;"#;
    let program = parse_program(source).expect("Failed to parse");
    assert_eq!(program.definitions.len(), 1);
    match &program.definitions[0].body[0] {
        Word::StringLiteral(s) => {
            assert!(s.contains('\n'));
            assert!(s.contains('\t'));
            assert!(s.contains('"'));
            assert!(s.contains('\\'));
        }
        _ => panic!("Expected StringLiteral"),
    }
}

#[test]
fn test_empty_string_literal() {
    let source = r#": test "" ;"#;
    let program = parse_program(source).expect("Failed to parse");
    assert_eq!(program.definitions.len(), 1);
    match &program.definitions[0].body[0] {
        Word::StringLiteral(s) => assert_eq!(s, ""),
        _ => panic!("Expected StringLiteral"),
    }
}

// ============================================================================
// CONTROL STRUCTURE PARSING EDGE CASES (4 tests)
// ============================================================================

#[test]
fn test_deeply_nested_if_then_else() {
    let source = r#"
        : deep-nest
            0 < IF
                1 < IF
                    2 < IF
                        3
                    ELSE
                        4
                    THEN
                ELSE
                    5
                THEN
            ELSE
                6
            THEN
        ;
    "#;
    let program = parse_program(source).expect("Failed to parse");
    assert_eq!(program.definitions.len(), 1);

    // Verify nested structure - the IF is at index 2 after "0 <"
    match &program.definitions[0].body[2] {
        Word::If { then_branch, else_branch } => {
            assert!(!then_branch.is_empty());
            assert!(else_branch.is_some());

            // Check inner IF structure in then_branch
            let has_nested_if = then_branch.iter().any(|w| matches!(w, Word::If { .. }));
            assert!(has_nested_if, "Expected nested IF structure in then_branch");
        }
        _ => panic!("Expected If structure"),
    }
}

#[test]
fn test_empty_if_branches() {
    let source = ": test IF THEN ;";
    let program = parse_program(source).expect("Failed to parse");
    assert_eq!(program.definitions.len(), 1);

    match &program.definitions[0].body[0] {
        Word::If { then_branch, else_branch } => {
            assert!(then_branch.is_empty());
            assert!(else_branch.is_none());
        }
        _ => panic!("Expected If structure"),
    }
}

#[test]
fn test_multiple_consecutive_control_structures() {
    let source = r#"
        : test
            IF 1 THEN
            IF 2 THEN
            IF 3 THEN
        ;
    "#;
    let program = parse_program(source).expect("Failed to parse");
    assert_eq!(program.definitions.len(), 1);

    // Count IF structures in body
    let if_count = program.definitions[0].body.iter()
        .filter(|w| matches!(w, Word::If { .. }))
        .count();
    assert_eq!(if_count, 3);
}

#[test]
fn test_begin_while_repeat_empty_body() {
    let source = ": test BEGIN WHILE REPEAT ;";
    let program = parse_program(source).expect("Failed to parse");
    assert_eq!(program.definitions.len(), 1);

    match &program.definitions[0].body[0] {
        Word::BeginWhileRepeat { condition, body } => {
            assert!(condition.is_empty());
            assert!(body.is_empty());
        }
        _ => panic!("Expected BeginWhileRepeat structure"),
    }
}

// ============================================================================
// WORD DEFINITION PARSING EDGE CASES (3 tests)
// ============================================================================

#[test]
fn test_definition_with_only_literals() {
    let source = ": test 1 2 3 4 5 ;";
    let program = parse_program(source).expect("Failed to parse");
    assert_eq!(program.definitions.len(), 1);
    assert_eq!(program.definitions[0].body.len(), 5);

    for (i, word) in program.definitions[0].body.iter().enumerate() {
        match word {
            Word::IntLiteral(n) => assert_eq!(*n, (i + 1) as i64),
            _ => panic!("Expected all IntLiterals"),
        }
    }
}

#[test]
fn test_forward_reference_in_definition() {
    // Forth allows forward references to be compiled
    // even if the word doesn't exist yet
    let source = r#"
        : first second third ;
        : second 2 ;
        : third 3 ;
    "#;
    let program = parse_program(source).expect("Failed to parse");
    assert_eq!(program.definitions.len(), 3);

    // Semantic analysis should catch undefined words if called
}

#[test]
fn test_self_recursive_definition() {
    let source = r#"
        : factorial
            dup 1 > IF
                dup 1 - factorial *
            ELSE
                drop 1
            THEN
        ;
    "#;
    let program = parse_program(source).expect("Failed to parse");
    assert_eq!(program.definitions.len(), 1);

    // Should have at least one WordRef to "factorial"
    fn has_word_ref(words: &[Word], name: &str) -> bool {
        words.iter().any(|w| match w {
            Word::WordRef { name: n, .. } => n == name,
            Word::If { then_branch, else_branch } => {
                has_word_ref(then_branch, name) ||
                else_branch.as_ref().map_or(false, |b| has_word_ref(b, name))
            }
            _ => false,
        })
    }

    assert!(has_word_ref(&program.definitions[0].body, "factorial"));
}

// ============================================================================
// ADDITIONAL EDGE CASES (5 tests)
// ============================================================================

#[test]
fn test_word_names_with_special_characters() {
    // Forth allows various special characters in word names
    let source = ": 2dup dup dup ; : >r ; : r> ; : @ ; : ! ;";
    let program = parse_program(source).expect("Failed to parse");
    assert_eq!(program.definitions.len(), 5);
    assert_eq!(program.definitions[0].name, "2dup");
    assert_eq!(program.definitions[1].name, ">r");
    assert_eq!(program.definitions[2].name, "r>");
    assert_eq!(program.definitions[3].name, "@");
    assert_eq!(program.definitions[4].name, "!");
}

#[test]
fn test_immediate_word_flag() {
    let source = ": test ; IMMEDIATE";
    let program = parse_program(source).expect("Failed to parse");
    assert_eq!(program.definitions.len(), 1);
    assert!(program.definitions[0].immediate);
}

#[test]
fn test_variable_and_constant_declarations() {
    let source = r#"
        VARIABLE x
        VARIABLE y
        42 CONSTANT answer
        -1 CONSTANT true
    "#;
    let program = parse_program(source).expect("Failed to parse");

    let var_count = program.top_level_code.iter()
        .filter(|w| matches!(w, Word::Variable { .. }))
        .count();
    assert_eq!(var_count, 2);

    let const_count = program.top_level_code.iter()
        .filter(|w| matches!(w, Word::Constant { .. }))
        .count();
    assert_eq!(const_count, 2);
}

#[test]
fn test_do_loop_with_empty_body() {
    let source = ": test 10 0 DO LOOP ;";
    let program = parse_program(source).expect("Failed to parse");
    assert_eq!(program.definitions.len(), 1);

    match &program.definitions[0].body[2] {
        Word::DoLoop { body, .. } => assert!(body.is_empty()),
        _ => panic!("Expected DoLoop structure"),
    }
}

#[test]
fn test_mixed_float_and_int_literals() {
    let source = ": test 1 2.0 3 4.5e-10 5 ;";
    let program = parse_program(source).expect("Failed to parse");
    assert_eq!(program.definitions.len(), 1);
    assert_eq!(program.definitions[0].body.len(), 5);

    // Check types
    assert!(matches!(program.definitions[0].body[0], Word::IntLiteral(_)));
    assert!(matches!(program.definitions[0].body[1], Word::FloatLiteral(_)));
    assert!(matches!(program.definitions[0].body[2], Word::IntLiteral(_)));
    assert!(matches!(program.definitions[0].body[3], Word::FloatLiteral(_)));
    assert!(matches!(program.definitions[0].body[4], Word::IntLiteral(_)));
}

// ============================================================================
// ERROR CASE TESTS (bonus tests for untested error paths)
// ============================================================================

#[test]
fn test_unterminated_definition() {
    let source = ": test 1 2 3";
    let result = parse_program(source);
    assert!(result.is_err());

    if let Err(ForthError::ParseError { message, .. }) = result {
        assert!(message.contains("Unterminated"));
    } else {
        panic!("Expected ParseError");
    }
}

#[test]
fn test_unterminated_if() {
    let source = ": test IF 1 2 3 ;";
    let result = parse_program(source);
    assert!(result.is_err());

    if let Err(ForthError::ParseError { message, .. }) = result {
        // Parser encounters semicolon before THEN, giving "Unexpected token" error
        assert!(message.contains("Unexpected token") || message.contains("Unterminated IF"));
    } else {
        panic!("Expected ParseError");
    }
}

#[test]
fn test_unterminated_begin_until() {
    let source = ": test BEGIN 1 2 3 ;";
    let result = parse_program(source);
    assert!(result.is_err());

    if let Err(ForthError::ParseError { message, .. }) = result {
        // Parser may encounter semicolon or reach unterminated check
        assert!(message.contains("Unexpected token") || message.contains("Unterminated BEGIN"));
    } else {
        panic!("Expected ParseError");
    }
}

#[test]
fn test_unterminated_do_loop() {
    let source = ": test 10 0 DO i ;";
    let result = parse_program(source);
    assert!(result.is_err());

    if let Err(ForthError::ParseError { message, .. }) = result {
        // Parser may encounter semicolon or reach unterminated check
        assert!(message.contains("Unexpected token") || message.contains("Unterminated DO"));
    } else {
        panic!("Expected ParseError");
    }
}

#[test]
fn test_constant_without_value() {
    let source = "CONSTANT x";
    let result = parse_program(source);
    assert!(result.is_err());

    if let Err(ForthError::ParseError { message, .. }) = result {
        assert!(message.contains("constant value"));
    } else {
        panic!("Expected ParseError");
    }
}

#[test]
fn test_stack_effect_with_no_separator() {
    let source = ": test ( a b c ) 1 ;";
    let program = parse_program(source).expect("Failed to parse");
    // Should parse but stack effect will have all inputs and no outputs
    assert_eq!(program.definitions.len(), 1);
}

#[test]
fn test_multiple_definitions_same_line() {
    let source = ": first 1 ; : second 2 ; : third 3 ;";
    let program = parse_program(source).expect("Failed to parse");
    assert_eq!(program.definitions.len(), 3);
    assert_eq!(program.definitions[0].name, "first");
    assert_eq!(program.definitions[1].name, "second");
    assert_eq!(program.definitions[2].name, "third");
}
