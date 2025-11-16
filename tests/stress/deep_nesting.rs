//! Deep nesting stress tests
//!
//! Integration tests for deeply nested structures that stress the entire compilation pipeline

use fastforth_frontend::parser::parse_program;
use fastforth_frontend::ssa::convert_to_ssa;

#[cfg(feature = "llvm")]
use backend::{LLVMBackend, CodeGenerator, CompilationMode};
#[cfg(feature = "llvm")]
use inkwell::{context::Context, OptimizationLevel};

#[test]
fn test_deeply_nested_if_statements() {
    // Test parsing and SSA conversion of deeply nested IF statements
    let code = r#"
: deeply-nested ( n -- result )
  dup 0 > if
    dup 1 > if
      dup 2 > if
        dup 3 > if
          dup 4 > if
            dup 5 > if
              dup 6 > if
                dup 7 > if
                  dup 8 > if
                    dup 9 > if
                      100
                    else
                      90
                    then
                  else
                    80
                  then
                else
                  70
                then
              else
                60
              then
            else
              50
            then
          else
            40
          then
        else
          30
        then
      else
        20
      then
    else
      10
    then
  else
    0
  then
;
"#;

    let program = parse_program(code);
    assert!(program.is_ok(), "Failed to parse deeply nested IF statements");

    let program = program.unwrap();
    let ssa_result = convert_to_ssa(&program);
    assert!(ssa_result.is_ok(), "Failed to convert to SSA");

    let functions = ssa_result.unwrap();
    assert_eq!(functions.len(), 1);
    assert_eq!(functions[0].name, "deeply-nested");

    // Should have many basic blocks (one for each branch)
    assert!(functions[0].blocks.len() > 10, "Expected many basic blocks for nested IF");
}

#[test]
fn test_deeply_nested_loops() {
    // Test nested loops
    let code = r#"
: nested-loops ( n -- sum )
  0  ( accumulator )
  swap 0 do
    i 0 do
      1 +
    loop
  loop
;
"#;

    let program = parse_program(code);
    assert!(program.is_ok(), "Failed to parse nested loops");

    let program = program.unwrap();
    let ssa_result = convert_to_ssa(&program);
    assert!(ssa_result.is_ok(), "Failed to convert nested loops to SSA");
}

#[test]
fn test_deeply_nested_word_calls() {
    // Test deeply nested function calls
    let code = r#"
: level-10 ( n -- n ) 1 + ;
: level-9 ( n -- n ) level-10 level-10 ;
: level-8 ( n -- n ) level-9 level-9 ;
: level-7 ( n -- n ) level-8 level-8 ;
: level-6 ( n -- n ) level-7 level-7 ;
: level-5 ( n -- n ) level-6 level-6 ;
: level-4 ( n -- n ) level-5 level-5 ;
: level-3 ( n -- n ) level-4 level-4 ;
: level-2 ( n -- n ) level-3 level-3 ;
: level-1 ( n -- n ) level-2 level-2 ;
: top-level ( n -- n ) level-1 level-1 ;
"#;

    let program = parse_program(code);
    assert!(program.is_ok(), "Failed to parse deeply nested word definitions");

    let program = program.unwrap();
    assert_eq!(program.definitions.len(), 11);

    let ssa_result = convert_to_ssa(&program);
    assert!(ssa_result.is_ok(), "Failed to convert nested words to SSA");

    let functions = ssa_result.unwrap();
    assert_eq!(functions.len(), 11);
}

#[test]
fn test_complex_control_flow_graph() {
    // Test a function with complex control flow
    let code = r#"
: complex-cfg ( a b -- result )
  dup 0 > if
    dup 10 > if
      +
    else
      -
    then
  else
    dup 5 > if
      swap +
    else
      swap -
    then
  then
;
"#;

    let program = parse_program(code);
    assert!(program.is_ok(), "Failed to parse complex CFG");

    let program = program.unwrap();
    let ssa_result = convert_to_ssa(&program);
    assert!(ssa_result.is_ok(), "Failed to convert complex CFG to SSA");

    let functions = ssa_result.unwrap();
    assert_eq!(functions.len(), 1);

    // Complex CFG should have multiple blocks
    assert!(functions[0].blocks.len() >= 4);
}

#[test]
fn test_long_computation_chain() {
    // Test a very long chain of computations
    let code = r#"
: long-chain ( n -- result )
  1 + 2 * 3 - 4 + 5 * 6 - 7 + 8 * 9 - 10 +
  1 + 2 * 3 - 4 + 5 * 6 - 7 + 8 * 9 - 10 +
  1 + 2 * 3 - 4 + 5 * 6 - 7 + 8 * 9 - 10 +
  1 + 2 * 3 - 4 + 5 * 6 - 7 + 8 * 9 - 10 +
  1 + 2 * 3 - 4 + 5 * 6 - 7 + 8 * 9 - 10 +
;
"#;

    let program = parse_program(code);
    assert!(program.is_ok(), "Failed to parse long computation chain");

    let program = program.unwrap();
    let ssa_result = convert_to_ssa(&program);
    assert!(ssa_result.is_ok(), "Failed to convert long chain to SSA");

    let functions = ssa_result.unwrap();
    assert_eq!(functions.len(), 1);

    // Should have many SSA instructions
    let total_instructions: usize = functions[0]
        .blocks
        .iter()
        .map(|b| b.instructions.len())
        .sum();
    assert!(total_instructions > 50, "Expected many instructions in long chain");
}

#[cfg(feature = "llvm")]
#[test]
fn test_codegen_deeply_nested_if() {
    // Test full compilation of deeply nested IF
    let code = r#"
: nested-if ( n -- result )
  dup 5 > if
    dup 10 > if
      100
    else
      50
    then
  else
    dup 2 > if
      25
    else
      10
    then
  then
;
"#;

    let program = parse_program(code).expect("Failed to parse");
    let functions = convert_to_ssa(&program).expect("Failed to convert to SSA");

    let context = Context::create();
    let mut backend = LLVMBackend::new(
        &context,
        "nested_if_test",
        CompilationMode::AOT,
        OptimizationLevel::None,
    );

    for func in &functions {
        assert!(
            backend.generate(func).is_ok(),
            "Failed to generate code for nested IF"
        );
    }

    let llvm_ir = backend.print_to_string();
    assert!(llvm_ir.contains("define"));
    assert!(llvm_ir.contains("br i1"));
}

#[cfg(feature = "llvm")]
#[test]
fn test_codegen_recursive_fibonacci() {
    // Test compilation of recursive fibonacci
    let code = r#"
: fib ( n -- result )
  dup 2 < if
    drop 1
  else
    dup 1 - fib
    swap 2 - fib
    +
  then
;
"#;

    let program = parse_program(code).expect("Failed to parse fibonacci");
    let functions = convert_to_ssa(&program).expect("Failed to convert fibonacci to SSA");

    let context = Context::create();
    let mut backend = LLVMBackend::new(
        &context,
        "fibonacci_test",
        CompilationMode::AOT,
        OptimizationLevel::Default,
    );

    for func in &functions {
        assert!(
            backend.generate(func).is_ok(),
            "Failed to generate code for fibonacci"
        );
    }

    let llvm_ir = backend.print_to_string();
    assert!(llvm_ir.contains("define"));
    assert!(llvm_ir.contains("call"));
}

#[test]
fn test_massive_stack_depth() {
    // Test handling of operations requiring very deep stack
    let code = r#"
: deep-stack ( -- result )
  1 2 3 4 5 6 7 8 9 10
  11 12 13 14 15 16 17 18 19 20
  + + + + + + + + +
  + + + + + + + + +
;
"#;

    let program = parse_program(code);
    assert!(program.is_ok(), "Failed to parse deep stack code");

    let program = program.unwrap();
    let ssa_result = convert_to_ssa(&program);
    assert!(ssa_result.is_ok(), "Failed to convert deep stack to SSA");
}

#[test]
fn test_many_local_definitions() {
    // Test many word definitions in one program
    let mut code = String::from("");
    for i in 0..100 {
        code.push_str(&format!(": word-{} ( n -- n ) {} + ;\n", i, i));
    }
    code.push_str(": main ( n -- result ) ");
    for i in 0..100 {
        code.push_str(&format!("word-{} ", i));
    }
    code.push_str(";\n");

    let program = parse_program(&code);
    assert!(program.is_ok(), "Failed to parse many definitions");

    let program = program.unwrap();
    assert_eq!(program.definitions.len(), 101);

    let ssa_result = convert_to_ssa(&program);
    assert!(ssa_result.is_ok(), "Failed to convert many definitions to SSA");
}

#[test]
fn test_nested_begin_until() {
    // Test nested BEGIN...UNTIL loops
    let code = r#"
: nested-until ( n -- )
  begin
    dup 0 >
  while
    dup
    begin
      dup 0 >
    while
      1 -
    repeat
    drop
    1 -
  repeat
  drop
;
"#;

    let program = parse_program(code);
    assert!(program.is_ok(), "Failed to parse nested BEGIN...UNTIL");

    let program = program.unwrap();
    let ssa_result = convert_to_ssa(&program);
    assert!(ssa_result.is_ok(), "Failed to convert nested loops to SSA");
}

#[test]
fn test_interleaved_control_structures() {
    // Test mixing different control structures
    let code = r#"
: mixed-control ( n -- result )
  0 swap
  0 do
    i 2 mod 0 = if
      i +
    else
      i 3 mod 0 = if
        i 2 * +
      else
        drop i
      then
    then
  loop
;
"#;

    let program = parse_program(code);
    assert!(program.is_ok(), "Failed to parse mixed control structures");

    let program = program.unwrap();
    let ssa_result = convert_to_ssa(&program);
    assert!(ssa_result.is_ok(), "Failed to convert mixed control to SSA");
}

#[cfg(feature = "llvm")]
#[test]
fn test_optimization_on_nested_code() {
    // Test that optimization works on nested structures
    let code = r#"
: optimizable-nested ( n -- result )
  0 + ( no-op )
  1 * ( no-op )
  dup 0 > if
    2 *
  else
    2 *
  then
;
"#;

    let program = parse_program(code).expect("Failed to parse");
    let functions = convert_to_ssa(&program).expect("Failed to convert to SSA");

    let context = Context::create();
    let mut backend = LLVMBackend::new(
        &context,
        "optimizable_nested",
        CompilationMode::AOT,
        OptimizationLevel::Aggressive,
    );

    for func in &functions {
        assert!(
            backend.generate(func).is_ok(),
            "Failed to generate optimized code"
        );
    }

    let llvm_ir = backend.print_to_string();
    assert!(llvm_ir.contains("define"));
    // Optimizations should have simplified the code
}

#[test]
fn test_phi_nodes_in_complex_cfg() {
    // Test that phi nodes are correctly generated in complex CFG
    let code = r#"
: complex-phi ( a b c -- result )
  over over > if
    + ( a+b )
  else
    - ( a-b )
  then
  swap
  over over > if
    + ( prev+c )
  else
    * ( prev*c )
  then
;
"#;

    let program = parse_program(code);
    assert!(program.is_ok(), "Failed to parse complex phi test");

    let program = program.unwrap();
    let ssa_result = convert_to_ssa(&program);
    assert!(ssa_result.is_ok(), "Failed to convert to SSA with phi nodes");

    let functions = ssa_result.unwrap();
    assert_eq!(functions.len(), 1);

    // Check that we have phi nodes
    let has_phi = functions[0].blocks.iter().any(|block| {
        block.instructions.iter().any(|inst| {
            matches!(inst, fastforth_frontend::ssa::SSAInstruction::Phi { .. })
        })
    });

    assert!(has_phi, "Expected phi nodes in complex CFG");
}

#[test]
fn test_extreme_nesting_limit() {
    // Test behavior at extreme nesting levels
    let mut code = String::from(": extreme-nesting ( n -- result )\n");

    // Create 20 levels of nesting
    for i in 0..20 {
        code.push_str(&format!("  dup {} > if\n", i));
    }

    code.push_str("    1000\n");

    // Close all the IFs
    for _ in 0..20 {
        code.push_str("  else\n    0\n  then\n");
    }

    code.push_str(";\n");

    let program = parse_program(&code);
    assert!(program.is_ok(), "Failed to parse extreme nesting");

    let program = program.unwrap();
    let ssa_result = convert_to_ssa(&program);
    assert!(ssa_result.is_ok(), "Failed to convert extreme nesting to SSA");
}

#[cfg(feature = "llvm")]
#[test]
fn test_codegen_with_all_optimization_levels() {
    // Test that code generation works at all optimization levels
    let code = r#"
: test-opts ( n -- result )
  dup 0 > if
    dup *
  else
    negate
  then
;
"#;

    let program = parse_program(code).expect("Failed to parse");
    let functions = convert_to_ssa(&program).expect("Failed to convert to SSA");

    for opt_level in &[
        OptimizationLevel::None,
        OptimizationLevel::Less,
        OptimizationLevel::Default,
        OptimizationLevel::Aggressive,
    ] {
        let context = Context::create();
        let mut backend = LLVMBackend::new(
            &context,
            &format!("test_opt_{:?}", opt_level),
            CompilationMode::AOT,
            *opt_level,
        );

        for func in &functions {
            assert!(
                backend.generate(func).is_ok(),
                "Failed to generate code at optimization level {:?}",
                opt_level
            );
        }
    }
}
