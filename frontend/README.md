# Fast Forth Frontend Compiler

A complete frontend compiler for ANS Forth, designed for speed and correctness. This compiler transforms Forth source code through multiple stages to produce optimized SSA (Static Single Assignment) form suitable for LLVM code generation.

## Features

- **Fast Parsing**: Target <50ms for typical programs using nom parser combinators
- **ANS Forth Compliance**: Supports standard ANS Forth syntax including:
  - Word definitions (`:` ... `;`)
  - Immediate words
  - Control structures (IF/THEN/ELSE, BEGIN/UNTIL, BEGIN/WHILE/REPEAT, DO/LOOP)
  - Comments (parenthesized and line comments)
  - String literals
  - Stack effect declarations

- **Stack Effect Inference**: Automatically infers stack effects for all word definitions
  - Tracks stack depth through sequences of operations
  - Validates against declared stack effects
  - Detects stack underflow at compile time

- **Type Inference**: Hindley-Milner-style type system for stack values
  - Polymorphic type variables for generic operations
  - Concrete types: int, float, addr, bool, char, string
  - Type unification and error reporting

- **SSA Conversion**: Transforms stack-based operations into SSA form
  - Each stack value becomes a unique SSA register
  - Explicit data flow for optimization
  - Basic block structure with control flow
  - Phi nodes for control flow merges

- **Semantic Analysis**: Comprehensive validation
  - Undefined word detection
  - Stack underflow checking
  - Control structure validation
  - Redefinition checking

## Architecture

```
Source Code
    ↓
Lexer (lexer.rs)
    ↓
Tokens
    ↓
Parser (parser.rs)
    ↓
AST (ast.rs)
    ↓
├─→ Stack Effect Inference (stack_effects.rs)
├─→ Type Inference (type_inference.rs)
└─→ Semantic Analysis (semantic.rs)
    ↓
SSA Conversion (ssa.rs)
    ↓
SSA IR (ready for LLVM)
```

## Usage

### Basic Parsing

```rust
use fastforth_frontend::*;

let source = ": square ( n -- n*n ) dup * ;";
let program = parse_program(source)?;
```

### Complete Pipeline

```rust
use fastforth_frontend::*;

let source = r#"
    : square ( n -- n*n )
        dup * ;

    : sum-of-squares ( a b -- a^2+b^2 )
        square swap square + ;
"#;

// Parse
let program = parse_program(source)?;

// Semantic analysis
semantic::analyze(&program)?;

// Stack effect inference
let mut stack_inference = stack_effects::StackEffectInference::new();
let effects = stack_inference.analyze_program(&program)?;

// Type inference
let mut type_inference = type_inference::TypeInference::new();
let types = type_inference.analyze_program(&program)?;

// SSA conversion
let ssa_functions = ssa::convert_to_ssa(&program)?;

// Display SSA
for func in &ssa_functions {
    println!("{}", func);
}
```

## Components

### Lexer (`lexer.rs`)

Tokenizes Forth source code into a stream of tokens.

**Features:**
- Handles all Forth token types
- Recognizes keywords (case-insensitive)
- Parses integer and floating-point literals
- Handles string literals with escape sequences
- Skips comments (both line and parenthesized)

### Parser (`parser.rs`)

Builds an Abstract Syntax Tree from tokens.

**Features:**
- Recursive descent parser
- Handles nested control structures
- Parses stack effect declarations
- Validates syntax structure
- Immediate word support

### Stack Effect Inference (`stack_effects.rs`)

Infers stack effects for word definitions.

**Algorithm:**
1. Track stack depth through each operation
2. Record inputs needed when stack underflows
3. Record outputs remaining on stack
4. Validate against declared effects

**Example:**
```forth
: double ( n -- n*2 ) 2 * ;
```
Inferred: `( int int -- int )` (takes 2 ints from declaring n and 2, produces 1 int)

### Type Inference (`type_inference.rs`)

Implements Hindley-Milner type inference for stack operations.

**Features:**
- Type variables for polymorphic operations
- Unification algorithm
- Occurs check for infinite types
- Constraint solving

**Example:**
```forth
: dup ( a -- a a )  \ Polymorphic - works for any type
```

### SSA Conversion (`ssa.rs`)

Converts stack-based code to Static Single Assignment form.

**SSA Form Benefits:**
- Explicit data dependencies
- Enables optimization passes
- Natural fit for LLVM IR
- Clear control flow

**Example:**
```forth
: add-one 1 + ;
```

Converts to:
```
define add-one (%0) {
bb0:
  %1 = load 1
  %2 = add %0, %1
  ret %2
}
```

### Semantic Analysis (`semantic.rs`)

Validates program semantics.

**Checks:**
- Undefined words
- Stack underflow
- Control structure balance
- Type consistency
- Stack effect matching

## Performance

Target performance metrics:

- **Parsing**: <50ms for typical programs (100-1000 LOC)
- **Stack Inference**: <10ms per program
- **Type Inference**: <20ms per program
- **SSA Conversion**: <30ms per program
- **Total Pipeline**: <100ms for typical programs

Run benchmarks:
```bash
cargo bench
```

## Testing

### Unit Tests

Each module has comprehensive unit tests:
```bash
cargo test
```

### Integration Tests

Full pipeline tests in `tests/integration_tests.rs`:
```bash
cargo test --test integration_tests
```

### Test Coverage

Run with coverage:
```bash
cargo tarpaulin --out Html
```

## Examples

### Simple Arithmetic

```forth
: square ( n -- n*n )
    dup * ;

: cube ( n -- n^3 )
    dup square * ;
```

### Control Structures

```forth
: abs ( n -- |n| )
    dup 0 < IF
        negate
    THEN ;

: max ( a b -- max )
    2dup > IF
        drop
    ELSE
        swap drop
    THEN ;
```

### Loops

```forth
: countdown ( n -- )
    BEGIN
        dup .
        1 -
        dup 0 =
    UNTIL
    drop ;

: factorial ( n -- n! )
    1 swap
    1 + 1 DO
        i *
    LOOP ;
```

### Stack Manipulation

```forth
: 2swap ( a b c d -- c d a b )
    rot >r rot r> ;

: nip ( a b -- b )
    swap drop ;

: tuck ( a b -- b a b )
    swap over ;
```

## Error Handling

All errors use the `ForthError` enum from `error.rs`:

```rust
pub enum ForthError {
    ParseError { line: usize, column: usize, message: String },
    UndefinedWord { word: String, line: Option<usize> },
    StackUnderflow { word: String, expected: usize, found: usize },
    TypeError { expected: String, found: String, location: Option<String> },
    // ... more error types
}
```

## Design Decisions

### Why nom for parsing?

- Fast and composable parser combinators
- Excellent error messages
- Zero-copy parsing where possible
- Well-tested and maintained

### Why Hindley-Milner type inference?

- Powerful enough to handle Forth's polymorphism
- Well-understood algorithm
- Good error messages
- Enables optimization

### Why SSA form?

- Standard IR format
- Enables powerful optimizations
- Natural mapping to LLVM IR
- Clear data flow representation

## Future Enhancements

- [ ] Return stack operations (>R, R>, R@)
- [ ] Memory allocation (ALLOT, HERE)
- [ ] String printing (.")
- [ ] More sophisticated type inference
- [ ] Whole-program optimization
- [ ] Incremental compilation
- [ ] IDE support (LSP server)

## Contributing

See the main FastForth project README for contribution guidelines.

## License

MIT License - see LICENSE file for details.

## References

- ANS Forth Standard: https://forth-standard.org/
- Hindley-Milner Type Inference: https://en.wikipedia.org/wiki/Hindley%E2%80%93Milner_type_system
- SSA Form: https://en.wikipedia.org/wiki/Static_single_assignment_form
- nom Parser Combinators: https://github.com/rust-bakery/nom
