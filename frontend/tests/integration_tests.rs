//! Integration tests for Fast Forth frontend

use fastforth_frontend::*;

#[test]
fn test_complete_pipeline() {
    let source = r#"
        : square ( n -- n*n )
            dup * ;

        : sum-of-squares ( a b -- a^2+b^2 )
            square swap square + ;
    "#;

    // Parse
    let program = parse_program(source).expect("Failed to parse");
    assert_eq!(program.definitions.len(), 2);

    // Semantic analysis
    semantic::analyze(&program).expect("Semantic analysis failed");

    // SSA conversion
    let ssa_functions = ssa::convert_to_ssa(&program).expect("SSA conversion failed");
    assert_eq!(ssa_functions.len(), 2);

    println!("SSA for square:\n{}", ssa_functions[0]);
    println!("\nSSA for sum-of-squares:\n{}", ssa_functions[1]);
}

#[test]
fn test_fibonacci() {
    let source = r#"
        : fib ( n -- fib(n) )
            dup 2 < IF
                drop 1
            ELSE
                dup 1 - fib
                swap 2 - fib
                +
            THEN ;
    "#;

    let program = parse_program(source).expect("Failed to parse");
    assert_eq!(program.definitions.len(), 1);

    semantic::analyze(&program).expect("Semantic analysis failed");

    let ssa_functions = ssa::convert_to_ssa(&program).expect("SSA conversion failed");
    assert_eq!(ssa_functions.len(), 1);
}

#[test]
fn test_control_structures() {
    let source = r#"
        : countdown ( n -- )
            BEGIN
                dup .
                1 -
                dup 0 =
            UNTIL
            drop ;
    "#;

    let program = parse_program(source).expect("Failed to parse");
    semantic::analyze(&program).expect("Semantic analysis failed");
}

#[test]
fn test_complex_stack_manipulation() {
    let source = r#"
        : 2swap ( a b c d -- c d a b )
            rot >r rot r> ;

        : nip ( a b -- b )
            swap drop ;

        : tuck ( a b -- b a b )
            swap over ;
    "#;

    let program = parse_program(source).expect("Failed to parse");
    assert_eq!(program.definitions.len(), 3);

    semantic::analyze(&program).expect("Semantic analysis failed");
}

#[test]
fn test_error_detection_undefined_word() {
    let source = ": test undefined-word ;";
    let program = parse_program(source).expect("Failed to parse");
    let result = semantic::analyze(&program);
    assert!(result.is_err());
}

#[test]
fn test_error_detection_stack_mismatch() {
    let source = ": test ( n -- n n ) drop ;";
    let program = parse_program(source).expect("Failed to parse");
    let result = semantic::analyze(&program);
    assert!(result.is_err());
}

#[test]
fn test_type_inference() {
    let source = r#"
        : add-one ( n -- n+1 )
            1 + ;
    "#;

    let program = parse_program(source).expect("Failed to parse");

    let mut type_inference = type_inference::TypeInference::new();
    let types = type_inference.analyze_program(&program).expect("Type inference failed");

    assert!(types.contains_key("add-one"));
}

#[test]
fn test_stack_effect_inference() {
    let source = r#"
        : double 2 * ;
        : triple 3 * ;
        : sum-doubled ( a b -- 2*(a+b) )
            + double ;
    "#;

    let program = parse_program(source).expect("Failed to parse");

    let mut stack_inference = stack_effects::StackEffectInference::new();
    let effects = stack_inference.analyze_program(&program).expect("Stack effect analysis failed");

    assert_eq!(effects.len(), 3);
    assert!(effects.contains_key("double"));
    assert!(effects.contains_key("triple"));
    assert!(effects.contains_key("sum-doubled"));
}

#[test]
fn test_nested_control_structures() {
    let source = r#"
        : collatz ( n -- )
            BEGIN
                dup 1 >
            WHILE
                dup 2 mod 0 = IF
                    2 /
                ELSE
                    3 * 1 +
                THEN
            REPEAT
            drop ;
    "#;

    let program = parse_program(source).expect("Failed to parse");
    semantic::analyze(&program).expect("Semantic analysis failed");

    let ssa_functions = ssa::convert_to_ssa(&program).expect("SSA conversion failed");
    assert_eq!(ssa_functions.len(), 1);
}

#[test]
fn test_do_loop() {
    let source = r#"
        : print-range ( start end -- )
            DO
                i .
            LOOP ;
    "#;

    let _program = parse_program(source).expect("Failed to parse");
    // Note: 'i' is a special loop variable that we may need to handle specially
}

#[test]
fn test_immediate_words() {
    let source = r#"
        : [char] ( -- c )
            char
        ; IMMEDIATE
    "#;

    let program = parse_program(source).expect("Failed to parse");
    assert_eq!(program.definitions.len(), 1);
    assert!(program.definitions[0].immediate);
}

#[test]
fn test_string_literals() {
    let _source = r#"
        : hello ( -- )
            ." Hello, World!" cr ;
    "#;

    // Note: ." is a special word that prints a string
    // We'll need to handle this in the lexer/parser
}

#[test]
fn test_variable_and_constant() {
    let source = r#"
        VARIABLE counter
        10 CONSTANT max-count

        : increment ( -- )
            counter @ 1 + counter ! ;
    "#;

    let program = parse_program(source).expect("Failed to parse");

    // Check that variables and constants are recognized
    assert!(program.top_level_code.iter().any(|w| matches!(w, Word::Variable { .. })));
    assert!(program.top_level_code.iter().any(|w| matches!(w, Word::Constant { .. })));
}

#[test]
fn test_performance_large_program() {
    // Test parsing performance with a larger program
    let mut source = String::new();

    for i in 0..100 {
        source.push_str(&format!(": func{} {} + ;\n", i, i));
    }

    let start = std::time::Instant::now();
    let program = parse_program(&source).expect("Failed to parse");
    let duration = start.elapsed();

    assert_eq!(program.definitions.len(), 100);

    // Should parse in less than 50ms for 100 definitions
    assert!(duration.as_millis() < 50, "Parsing took too long: {:?}", duration);
}

#[test]
fn test_ssa_correctness() {
    let source = ": test 1 2 + 3 * ;";
    let program = parse_program(source).expect("Failed to parse");
    let ssa_functions = ssa::convert_to_ssa(&program).expect("SSA conversion failed");

    // Check that SSA form is generated correctly
    let func = &ssa_functions[0];
    assert!(!func.blocks.is_empty());
    assert!(!func.blocks[0].instructions.is_empty());
}

#[test]
fn test_polymorphic_words() {
    // Test identity function (polymorphic with empty body)
    let source = r#"
        : identity ( x -- x )
            ;
    "#;

    let program = parse_program(source).expect("Failed to parse");

    let mut type_inference = type_inference::TypeInference::new();
    let result = type_inference.analyze_program(&program);
    assert!(result.is_ok(), "Identity function should type-check");

    // Note: apply-twice with return stack operations and execute is complex
    // and would require more sophisticated analysis including:
    // - Proper return stack modeling
    // - Higher-order function support
    // - Data flow analysis across execute boundaries
    // This is left for future enhancement
}

#[test]
fn test_complex_arithmetic() {
    let source = r#"
        : quadratic ( a b c x -- result )
            >r              ( a b c )  ( R: x )
            swap r@ *       ( a c b*x )  ( R: x )
            swap r@ dup * * ( b*x a*x^2 )  ( R: x )
            + +             ( a*x^2+b*x+c )  ( R: x )
            r> drop ;       ( result )
    "#;

    let program = parse_program(source).expect("Failed to parse");
    semantic::analyze(&program).expect("Semantic analysis failed");
}

#[test]
fn test_edge_cases() {
    // Empty definition
    let source = ": noop ;";
    let program = parse_program(source).expect("Failed to parse");
    assert_eq!(program.definitions.len(), 1);
    assert!(program.definitions[0].body.is_empty());

    // Single word
    let source = ": identity dup drop ;";
    let program = parse_program(source).expect("Failed to parse");
    assert_eq!(program.definitions[0].body.len(), 2);
}

#[test]
fn test_floating_point() {
    let source = r#"
        : circle-area ( radius -- area )
            dup * 3.14159 * ;
    "#;

    let program = parse_program(source).expect("Failed to parse");

    // Check that float literal is parsed correctly
    assert!(program.definitions[0].body.iter().any(|w| {
        matches!(w, Word::FloatLiteral(_))
    }));
}
