/// ANS Forth Extended Word Set Compliance Tests
///
/// Tests additional ANS Forth words beyond the core set
/// Including: Memory operations, Control structures, Word definitions, Return stack

use crate::test_utils::ForthEngine;

// ============================================================================
// DOUBLE-CELL ARITHMETIC TESTS
// ============================================================================
// Note: These tests verify the behavior of double-cell stack operations
// which operate on pairs of cells as a single value

#[test]
fn test_double_2dup() {
    let mut engine = ForthEngine::new();
    engine.eval("1 2 2DUP").unwrap();
    assert_eq!(engine.stack(), &[1, 2, 1, 2], "2DUP: ( d -- d d )");
}

#[test]
fn test_double_2drop() {
    let mut engine = ForthEngine::new();
    engine.eval("1 2 3 4 2DROP").unwrap();
    assert_eq!(engine.stack(), &[1, 2], "2DROP: ( d1 d2 -- d1 )");
}

#[test]
fn test_double_2swap() {
    let mut engine = ForthEngine::new();
    engine.eval("1 2 3 4 2SWAP").unwrap();
    assert_eq!(engine.stack(), &[3, 4, 1, 2], "2SWAP: ( d1 d2 -- d2 d1 )");
}

#[test]
fn test_double_2over() {
    let mut engine = ForthEngine::new();
    engine.eval("1 2 3 4 2OVER").unwrap();
    assert_eq!(engine.stack(), &[1, 2, 3, 4, 1, 2], "2OVER: ( d1 d2 -- d1 d2 d1 )");
}

// ============================================================================
// ADVANCED STACK OPERATIONS
// ============================================================================

// Note: PICK and ROLL are extended stack operations
// PICK: Copy the nth item (0-indexed from top) to the top
// ROLL: Move the nth item (0-indexed from top) to the top

// TODO: Implement PICK when available in ForthEngine
// #[test]
// fn test_stack_pick() {
//     let mut engine = ForthEngine::new();
//     // PICK: ( xu ... x0 u -- xu ... x0 xu )
//     engine.eval("10 20 30 2 PICK").unwrap();
//     assert_eq!(engine.stack(), &[10, 20, 30, 10], "2 PICK should copy 3rd item");
// }

// TODO: Implement ROLL when available in ForthEngine
// #[test]
// fn test_stack_roll() {
//     let mut engine = ForthEngine::new();
//     // ROLL: ( xu ... x0 u -- xu-1 ... x0 xu )
//     engine.eval("10 20 30 2 ROLL").unwrap();
//     assert_eq!(engine.stack(), &[20, 30, 10], "2 ROLL should move 3rd item to top");
// }

// ============================================================================
// MEMORY OPERATIONS (Placeholder Tests)
// ============================================================================
// Note: These require memory allocation which is not yet implemented
// in the test ForthEngine

// TODO: Implement memory operations when available
// #[test]
// fn test_memory_variable() {
//     let mut engine = ForthEngine::new();
//     // VARIABLE X creates a variable
//     // X pushes address
//     // ! stores value at address
//     // @ fetches value from address
//     engine.eval("VARIABLE X").unwrap();
//     engine.eval("42 X !").unwrap();
//     engine.eval("X @").unwrap();
//     assert_eq!(engine.stack(), &[42], "Variable store and fetch");
// }

// TODO: Implement constant operations
// #[test]
// fn test_memory_constant() {
//     let mut engine = ForthEngine::new();
//     engine.eval("42 CONSTANT ANSWER").unwrap();
//     engine.eval("ANSWER").unwrap();
//     assert_eq!(engine.stack(), &[42], "CONSTANT should push its value");
// }

// TODO: Implement VALUE operations
// #[test]
// fn test_memory_value() {
//     let mut engine = ForthEngine::new();
//     engine.eval("100 VALUE SCORE").unwrap();
//     engine.eval("SCORE").unwrap();
//     assert_eq!(engine.stack(), &[100], "VALUE should push its value");
// }

// TODO: Implement +! (add to memory)
// #[test]
// fn test_memory_plus_store() {
//     let mut engine = ForthEngine::new();
//     engine.eval("VARIABLE COUNTER").unwrap();
//     engine.eval("10 COUNTER !").unwrap();
//     engine.eval("5 COUNTER +!").unwrap();
//     engine.eval("COUNTER @").unwrap();
//     assert_eq!(engine.stack(), &[15], "+! should add to stored value");
// }

// TODO: Implement C@ and C! (character/byte operations)
// #[test]
// fn test_memory_c_store_fetch() {
//     let mut engine = ForthEngine::new();
//     engine.eval("VARIABLE CBUF").unwrap();
//     engine.eval("65 CBUF C!").unwrap(); // Store 'A'
//     engine.eval("CBUF C@").unwrap();
//     assert_eq!(engine.stack(), &[65], "C@ and C! for byte operations");
// }

// TODO: Implement 2@ and 2! (double-cell memory operations)
// #[test]
// fn test_memory_2store_2fetch() {
//     let mut engine = ForthEngine::new();
//     engine.eval("VARIABLE BUF").unwrap();
//     engine.eval("10 20 BUF 2!").unwrap();
//     engine.eval("BUF 2@").unwrap();
//     assert_eq!(engine.stack(), &[10, 20], "2@ and 2! for double-cell");
// }

// ============================================================================
// CONTROL STRUCTURES (Placeholder Tests)
// ============================================================================
// Note: Control structures require compilation support

// TODO: Implement IF/THEN
// #[test]
// fn test_control_if_then_true() {
//     let mut engine = ForthEngine::new();
//     engine.eval(": TEST -1 IF 42 THEN ;").unwrap();
//     engine.eval("TEST").unwrap();
//     assert_eq!(engine.stack(), &[42], "IF/THEN should execute on true");
// }

// TODO: Implement IF/ELSE/THEN
// #[test]
// fn test_control_if_else_then() {
//     let mut engine = ForthEngine::new();
//     engine.eval(": TEST IF 1 ELSE 2 THEN ;").unwrap();
//     engine.eval("-1 TEST").unwrap();
//     assert_eq!(engine.stack(), &[1], "IF branch taken");
//     engine.clear_stack();
//     engine.eval("0 TEST").unwrap();
//     assert_eq!(engine.stack(), &[2], "ELSE branch taken");
// }

// TODO: Implement BEGIN/UNTIL
// #[test]
// fn test_control_begin_until() {
//     let mut engine = ForthEngine::new();
//     // Count down from 5 to 0
//     engine.eval(": COUNTDOWN 5 BEGIN 1 - DUP 0= UNTIL DROP ;").unwrap();
//     engine.eval("COUNTDOWN").unwrap();
//     assert_eq!(engine.stack(), &[], "BEGIN/UNTIL countdown");
// }

// TODO: Implement BEGIN/WHILE/REPEAT
// #[test]
// fn test_control_begin_while_repeat() {
//     let mut engine = ForthEngine::new();
//     engine.eval(": SUM 0 5 BEGIN DUP 0> WHILE SWAP OVER + SWAP 1- REPEAT DROP ;").unwrap();
//     engine.eval("SUM").unwrap();
//     assert_eq!(engine.stack(), &[15], "BEGIN/WHILE/REPEAT sum 1-5");
// }

// TODO: Implement DO/LOOP
// #[test]
// fn test_control_do_loop() {
//     let mut engine = ForthEngine::new();
//     // Sum from 0 to 4
//     engine.eval(": SUM 0 5 0 DO I + LOOP ;").unwrap();
//     engine.eval("SUM").unwrap();
//     assert_eq!(engine.stack(), &[10], "DO/LOOP sum 0-4");
// }

// TODO: Implement DO/+LOOP
// #[test]
// fn test_control_do_plus_loop() {
//     let mut engine = ForthEngine::new();
//     // Count by 2s from 0 to 8
//     engine.eval(": COUNT2 10 0 DO I 2 +LOOP ;").unwrap();
//     engine.eval("COUNT2").unwrap();
//     // This would leave 0 2 4 6 8 on stack
// }

// TODO: Implement LEAVE (exit loop early)
// #[test]
// fn test_control_leave() {
//     let mut engine = ForthEngine::new();
//     engine.eval(": EARLY 10 0 DO I DUP 5 = IF LEAVE THEN LOOP ;").unwrap();
//     engine.eval("EARLY").unwrap();
//     // Should leave loop when I=5
// }

// ============================================================================
// WORD DEFINITION TESTS (Placeholder)
// ============================================================================

// TODO: Implement colon definitions
// #[test]
// fn test_define_simple_word() {
//     let mut engine = ForthEngine::new();
//     engine.eval(": DOUBLE 2 * ;").unwrap();
//     engine.eval("5 DOUBLE").unwrap();
//     assert_eq!(engine.stack(), &[10], "User-defined word");
// }

// TODO: Implement word using other words
// #[test]
// fn test_define_composite_word() {
//     let mut engine = ForthEngine::new();
//     engine.eval(": DOUBLE 2 * ;").unwrap();
//     engine.eval(": QUAD DOUBLE DOUBLE ;").unwrap();
//     engine.eval("5 QUAD").unwrap();
//     assert_eq!(engine.stack(), &[20], "Composite word");
// }

// TODO: Implement RECURSE
// #[test]
// fn test_define_recursive_word() {
//     let mut engine = ForthEngine::new();
//     engine.eval(": FACTORIAL DUP 2 < IF DROP 1 ELSE DUP 1- RECURSE * THEN ;").unwrap();
//     engine.eval("5 FACTORIAL").unwrap();
//     assert_eq!(engine.stack(), &[120], "Recursive factorial");
// }

// TODO: Implement EXIT
// #[test]
// fn test_define_early_exit() {
//     let mut engine = ForthEngine::new();
//     engine.eval(": TEST DUP 0= IF DROP 99 EXIT THEN 1+ ;").unwrap();
//     engine.eval("0 TEST").unwrap();
//     assert_eq!(engine.stack(), &[99], "EXIT terminates word early");
// }

// ============================================================================
// IMMEDIATE WORDS (Placeholder)
// ============================================================================

// TODO: Implement IMMEDIATE
// #[test]
// fn test_immediate_word() {
//     let mut engine = ForthEngine::new();
//     // Define an immediate word that executes during compilation
//     engine.eval(": [SQUARE] DUP * ; IMMEDIATE").unwrap();
//     // This would execute [SQUARE] during compilation of the next word
// }

// TODO: Implement [ and ] (compilation mode toggle)
// #[test]
// fn test_bracket_compilation_mode() {
//     let mut engine = ForthEngine::new();
//     engine.eval(": TEST [ 2 3 + ] LITERAL ;").unwrap();
//     engine.eval("TEST").unwrap();
//     assert_eq!(engine.stack(), &[5], "[ ] for compile-time evaluation");
// }

// ============================================================================
// RETURN STACK OPERATIONS (Placeholder)
// ============================================================================

// TODO: Implement >R (to return stack)
// #[test]
// fn test_rstack_to_r() {
//     let mut engine = ForthEngine::new();
//     engine.eval(": TEST 5 >R 10 R> + ;").unwrap();
//     engine.eval("TEST").unwrap();
//     assert_eq!(engine.stack(), &[15], ">R moves to return stack");
// }

// TODO: Implement R> (from return stack)
// #[test]
// fn test_rstack_r_from() {
//     let mut engine = ForthEngine::new();
//     engine.eval(": TEST 5 >R 10 R> ;").unwrap();
//     engine.eval("TEST").unwrap();
//     assert_eq!(engine.stack(), &[10, 5], "R> retrieves from return stack");
// }

// TODO: Implement R@ (peek return stack)
// #[test]
// fn test_rstack_r_fetch() {
//     let mut engine = ForthEngine::new();
//     engine.eval(": TEST 5 >R R@ R> + ;").unwrap();
//     engine.eval("TEST").unwrap();
//     assert_eq!(engine.stack(), &[10], "R@ peeks return stack without removing");
// }

// TODO: Implement I and J (loop counters)
// #[test]
// fn test_rstack_loop_index() {
//     let mut engine = ForthEngine::new();
//     engine.eval(": SQUARES 5 0 DO I DUP * LOOP ;").unwrap();
//     engine.eval("SQUARES").unwrap();
//     // Should leave 0 1 4 9 16 on stack
//     assert_eq!(engine.stack(), &[0, 1, 4, 9, 16], "I provides loop index");
// }

// ============================================================================
// STRING AND I/O OPERATIONS (Placeholder)
// ============================================================================

// TODO: Implement TYPE (output string)
// #[test]
// fn test_io_type() {
//     let mut engine = ForthEngine::new();
//     // TYPE: ( c-addr u -- ) outputs u characters from address
//     // This requires string literals and memory
// }

// TODO: Implement ." (dot-quote for printing)
// #[test]
// fn test_io_dot_quote() {
//     let mut engine = ForthEngine::new();
//     engine.eval(": GREET .\" Hello, World!\" ;").unwrap();
//     // Should output "Hello, World!" when GREET is executed
// }

// TODO: Implement S" (string literal)
// #[test]
// fn test_io_s_quote() {
//     let mut engine = ForthEngine::new();
//     // S" Hello" leaves address and length on stack
// }

// TODO: Implement COUNT (string operations)
// #[test]
// fn test_string_count() {
//     let mut engine = ForthEngine::new();
//     // COUNT: ( c-addr -- c-addr+1 u )
//     // Converts counted string to address and length
// }

// ============================================================================
// BASE CONVERSION (Placeholder)
// ============================================================================

// TODO: Implement BASE variable
// #[test]
// fn test_base_variable() {
//     let mut engine = ForthEngine::new();
//     engine.eval("BASE @").unwrap();
//     assert_eq!(engine.stack(), &[10], "Default base should be 10");
// }

// TODO: Implement DECIMAL
// #[test]
// fn test_base_decimal() {
//     let mut engine = ForthEngine::new();
//     engine.eval("HEX 16 DECIMAL").unwrap();
//     // Number should be interpreted as hex 16 (22 decimal)
// }

// TODO: Implement HEX
// #[test]
// fn test_base_hex() {
//     let mut engine = ForthEngine::new();
//     engine.eval("HEX 10").unwrap();
//     assert_eq!(engine.stack(), &[16], "HEX 10 = decimal 16");
// }

// TODO: Implement BINARY
// #[test]
// fn test_base_binary() {
//     let mut engine = ForthEngine::new();
//     engine.eval("BINARY 1010").unwrap();
//     assert_eq!(engine.stack(), &[10], "Binary 1010 = decimal 10");
// }

// ============================================================================
// ADVANCED ARITHMETIC (Placeholder)
// ============================================================================

// TODO: Implement */ (multiply then divide)
// #[test]
// fn test_arith_star_slash() {
//     let mut engine = ForthEngine::new();
//     // */: ( n1 n2 n3 -- n1*n2/n3 )
//     // Uses intermediate double-cell to avoid overflow
//     engine.eval("100 50 10 */").unwrap();
//     assert_eq!(engine.stack(), &[500], "*/ multiply and divide");
// }

// TODO: Implement */MOD
// #[test]
// fn test_arith_star_slash_mod() {
//     let mut engine = ForthEngine::new();
//     // */MOD: ( n1 n2 n3 -- remainder quotient )
//     engine.eval("100 50 30 */MOD").unwrap();
//     assert_eq!(engine.stack(), &[20, 166], "*/MOD with intermediate precision");
// }

// TODO: Implement M* (mixed multiply)
// #[test]
// fn test_arith_m_star() {
//     let mut engine = ForthEngine::new();
//     // M*: ( n1 n2 -- d ) multiply to produce double-cell result
//     engine.eval("1000000 1000000 M*").unwrap();
//     // Result is 1000000000000 as double-cell
// }

// TODO: Implement FM/MOD (floored division)
// #[test]
// fn test_arith_fm_slash_mod() {
//     let mut engine = ForthEngine::new();
//     // FM/MOD: ( d n -- remainder quotient ) floored division
// }

// TODO: Implement SM/REM (symmetric division)
// #[test]
// fn test_arith_sm_slash_rem() {
//     let mut engine = ForthEngine::new();
//     // SM/REM: ( d n -- remainder quotient ) symmetric division
// }

// ============================================================================
// EXCEPTION HANDLING (Placeholder)
// ============================================================================

// TODO: Implement CATCH and THROW
// #[test]
// fn test_exception_catch_throw() {
//     let mut engine = ForthEngine::new();
//     // CATCH: ( ... xt -- ... 0 | ... n )
//     // THROW: ( ... n -- ... | ... n )
//     engine.eval(": RISKY 10 0 / ;").unwrap(); // Division by zero
//     engine.eval("' RISKY CATCH").unwrap();
//     // Should catch the exception and return error code
// }

// TODO: Implement ABORT
// #[test]
// fn test_exception_abort() {
//     let mut engine = ForthEngine::new();
//     // ABORT: ( -- ) clear stacks and abort
// }

// TODO: Implement ABORT"
// #[test]
// fn test_exception_abort_quote() {
//     let mut engine = ForthEngine::new();
//     // ABORT" message" aborts with error message if flag is true
//     engine.eval(": CHECK DUP 0< ABORT\" Negative number!\" ;").unwrap();
// }

// ============================================================================
// DICTIONARY AND COMPILATION (Placeholder)
// ============================================================================

// TODO: Implement FIND
// #[test]
// fn test_dict_find() {
//     let mut engine = ForthEngine::new();
//     // FIND: ( c-addr -- c-addr 0 | xt 1 | xt -1 )
//     // Look up word in dictionary
// }

// TODO: Implement ' (tick - get execution token)
// #[test]
// fn test_dict_tick() {
//     let mut engine = ForthEngine::new();
//     engine.eval(": DOUBLE 2 * ;").unwrap();
//     engine.eval("' DOUBLE").unwrap();
//     // Should leave execution token on stack
// }

// TODO: Implement EXECUTE
// #[test]
// fn test_dict_execute() {
//     let mut engine = ForthEngine::new();
//     engine.eval(": DOUBLE 2 * ;").unwrap();
//     engine.eval("5 ' DOUBLE EXECUTE").unwrap();
//     assert_eq!(engine.stack(), &[10], "EXECUTE runs execution token");
// }

// TODO: Implement CREATE...DOES>
// #[test]
// fn test_dict_create_does() {
//     let mut engine = ForthEngine::new();
//     // CREATE...DOES> for defining defining words
//     engine.eval(": CONSTANT CREATE , DOES> @ ;").unwrap();
//     engine.eval("42 CONSTANT ANSWER").unwrap();
//     engine.eval("ANSWER").unwrap();
//     assert_eq!(engine.stack(), &[42], "CREATE...DOES> defining word");
// }

// TODO: Implement ALLOT
// #[test]
// fn test_dict_allot() {
//     let mut engine = ForthEngine::new();
//     // ALLOT: ( n -- ) allocate n bytes in dictionary
//     engine.eval("CREATE BUFFER 100 ALLOT").unwrap();
//     // BUFFER now has 100 bytes of space
// }

// TODO: Implement HERE
// #[test]
// fn test_dict_here() {
//     let mut engine = ForthEngine::new();
//     // HERE: ( -- addr ) returns current dictionary pointer
//     engine.eval("HERE").unwrap();
//     // Should return an address
// }

// TODO: Implement , (comma - compile cell)
// #[test]
// fn test_dict_comma() {
//     let mut engine = ForthEngine::new();
//     // ,: ( n -- ) compile n into dictionary
//     engine.eval("CREATE DATA 1 , 2 , 3 ,").unwrap();
//     // DATA now contains three cells
// }

// ============================================================================
// NUMERIC OUTPUT FORMATTING (Placeholder)
// ============================================================================

// TODO: Implement U. (unsigned output)
// #[test]
// fn test_output_u_dot() {
//     let mut engine = ForthEngine::new();
//     // U.: ( u -- ) output unsigned number
// }

// TODO: Implement .R (right-justified output)
// #[test]
// fn test_output_dot_r() {
//     let mut engine = ForthEngine::new();
//     // .R: ( n width -- ) output number right-justified
// }

// TODO: Implement U.R (unsigned right-justified)
// #[test]
// fn test_output_u_dot_r() {
//     let mut engine = ForthEngine::new();
//     // U.R: ( u width -- ) output unsigned right-justified
// }

// TODO: Implement <# # #S #> (pictured numeric output)
// #[test]
// fn test_output_pictured_numeric() {
//     let mut engine = ForthEngine::new();
//     // <# starts numeric conversion
//     // # converts one digit
//     // #S converts remaining digits
//     // #> finishes conversion
// }

// ============================================================================
// SUMMARY STATISTICS
// ============================================================================

// Total Extended Word Categories Documented:
// - Double-cell operations: 4 words tested (2DUP, 2DROP, 2SWAP, 2OVER)
// - Advanced stack: 2 words (PICK, ROLL) - TODO
// - Memory operations: 8 words (VARIABLE, CONSTANT, VALUE, !, @, +!, C!, C@, 2!, 2@) - TODO
// - Control structures: 7 constructs (IF/THEN, IF/ELSE/THEN, BEGIN/UNTIL, BEGIN/WHILE/REPEAT, DO/LOOP, DO/+LOOP, LEAVE) - TODO
// - Word definition: 5 features (:, RECURSE, EXIT, IMMEDIATE, [ ]) - TODO
// - Return stack: 5 words (>R, R>, R@, I, J) - TODO
// - String/IO: 4 words (TYPE, .", S", COUNT) - TODO
// - Base conversion: 4 words (BASE, DECIMAL, HEX, BINARY) - TODO
// - Advanced arithmetic: 5 words (*/, */MOD, M*, FM/MOD, SM/REM) - TODO
// - Exception handling: 3 words (CATCH, THROW, ABORT, ABORT") - TODO
// - Dictionary: 8 words (FIND, ', EXECUTE, CREATE...DOES>, ALLOT, HERE, ,) - TODO
// - Numeric output: 8 words (U., .R, U.R, <#, #, #S, #>, HOLD) - TODO
//
// Total: 4 words currently tested, 59+ words documented for future implementation
