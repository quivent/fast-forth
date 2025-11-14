# Fast Forth CLI - User Guide

**Version**: 1.0.0
**Last Updated**: 2025-11-14

---

## Table of Contents

1. [Installation](#installation)
2. [Quick Start](#quick-start)
3. [Command Reference](#command-reference)
4. [REPL Usage](#repl-usage)
5. [Compilation](#compilation)
6. [Type Checking](#type-checking)
7. [Documentation Generation](#documentation-generation)
8. [Profiling](#profiling)
9. [Language Server](#language-server)
10. [Examples](#examples)

---

## Installation

### Building from Source

```bash
cd /path/to/FastForth/cli
cargo build --release
```

The binary will be located at `target/release/fastforth`.

### Installing to PATH

```bash
cargo install --path .
```

Or manually copy the binary:

```bash
cp target/release/fastforth ~/.local/bin/
# or
sudo cp target/release/fastforth /usr/local/bin/
```

---

## Quick Start

### Hello World

```bash
# Create a simple Forth program
echo '. "Hello, World!" CR' > hello.fth

# Run it
fastforth run hello.fth
```

### Interactive REPL

```bash
# Start the REPL
fastforth

# Or explicitly
fastforth repl
```

In the REPL:

```forth
forth> 5 3 +
  ✓ OK (0.3ms)

Stack: [ 8 ]                                   Depth: 1

forth> .
8   ✓ OK (0.5ms)

Stack: [ ]                                     Depth: 0
```

---

## Command Reference

### Global Options

- `-v, --verbose` - Verbose output with detailed information
- `-q, --quiet` - Quiet mode (suppress non-error output)
- `--json` - JSON output for tool integration
- `-h, --help` - Show help information
- `-V, --version` - Show version

### Commands

#### `repl` - Interactive REPL

Start an interactive Read-Eval-Print Loop for Forth programming.

```bash
fastforth repl [OPTIONS]
```

**Options:**
- `--prompt <PROMPT>` - Custom prompt string
- `--no-stack-depth` - Don't show stack depth
- `--no-timing` - Don't show execution timing

**Examples:**

```bash
# Start REPL with default settings
fastforth repl

# Custom prompt
fastforth repl --prompt ">> "

# Minimal REPL
fastforth repl --no-stack-depth --no-timing
```

#### `compile` - Compile to Executable

Compile Forth source code to a native executable.

```bash
fastforth compile <INPUT> [OPTIONS]
```

**Options:**
- `-o, --output <FILE>` - Output file path
- `-O, --optimize <LEVEL>` - Optimization level (0-3, default: 1)
- `--target <TARGET>` - Target platform (native, wasm, js, llvm-ir, asm)
- `--debug` - Include debug symbols
- `--dump-ast` - Display Abstract Syntax Tree
- `--dump-ir` - Display Intermediate Representation
- `--time-passes` - Show timing for each compilation phase

**Examples:**

```bash
# Basic compilation
fastforth compile program.fth

# With optimization
fastforth compile program.fth -O3

# Specific output file
fastforth compile program.fth -o my-program

# Verbose with timing
fastforth compile program.fth -v --time-passes

# WebAssembly target
fastforth compile program.fth --target=wasm
```

#### `run` - JIT Compile and Execute

Just-In-Time compile and run Forth code.

```bash
fastforth run <INPUT> [OPTIONS]
```

**Options:**
- `--profile` - Enable profiling during execution
- `--debug` - Enable debug mode
- `--trace` - Trace execution

**Examples:**

```bash
# Run a program
fastforth run program.fth

# Run with profiling
fastforth run program.fth --profile

# Verbose execution
fastforth run program.fth -v
```

#### `check` - Type Check

Perform type checking without compilation or execution.

```bash
fastforth check <INPUT> [OPTIONS]
```

**Options:**
- `--strict` - Treat warnings as errors

**Examples:**

```bash
# Check a file
fastforth check program.fth

# Strict mode
fastforth check program.fth --strict

# Verbose checking
fastforth check program.fth -v
```

#### `profile` - Profile Execution

Profile program execution to identify performance bottlenecks.

```bash
fastforth profile <INPUT> [OPTIONS]
```

**Options:**
- `--flame-graph` - Generate flame graph visualization
- `--memory` - Profile memory usage
- `-o, --output <FILE>` - Output file for flame graph

**Examples:**

```bash
# Basic profiling
fastforth profile program.fth

# Generate flame graph
fastforth profile program.fth --flame-graph

# Memory profiling
fastforth profile program.fth --memory

# Save flame graph
fastforth profile program.fth --flame-graph -o profile.html
```

#### `doc` - Generate Documentation

Generate HTML or Markdown documentation from source code.

```bash
fastforth doc <INPUT> [OPTIONS]
```

**Options:**
- `--format <FORMAT>` - Output format (html, markdown)
- `-o, --output <DIR>` - Output directory (default: docs)

**Examples:**

```bash
# Generate HTML documentation
fastforth doc program.fth

# Generate Markdown
fastforth doc program.fth --format=markdown

# Custom output directory
fastforth doc program.fth -o documentation/

# Verbose generation
fastforth doc program.fth -v
```

#### `lsp` - Language Server

Start the Language Server Protocol server for IDE integration.

```bash
fastforth lsp
```

Use this with your editor's LSP client (VSCode, Vim, Emacs, etc.).

#### `format` - Auto-format

Auto-format Forth source code.

```bash
fastforth format <INPUT> [OPTIONS]
```

**Options:**
- `--check` - Check formatting without modifying files

**Examples:**

```bash
# Format a file
fastforth format program.fth

# Check formatting
fastforth format program.fth --check
```

#### `explain` - Explain Word

Get detailed information about a Forth word.

```bash
fastforth explain <WORD>
```

**Examples:**

```bash
# Explain a word
fastforth explain DUP

# Explain arithmetic
fastforth explain +
```

#### `benchmark` - Benchmark

Run performance benchmarks on Forth code.

```bash
fastforth benchmark <INPUT> [OPTIONS]
```

**Options:**
- `-n, --iterations <N>` - Number of iterations (default: 100)

**Examples:**

```bash
# Benchmark with default iterations
fastforth benchmark program.fth

# Custom iteration count
fastforth benchmark program.fth -n 1000
```

#### `new` - Create Project

Create a new Forth project with template structure.

```bash
fastforth new <NAME> [OPTIONS]
```

**Options:**
- `-t, --template <TEMPLATE>` - Project template (default, library, application)

**Examples:**

```bash
# Create a new project
fastforth new my-project

# Use library template
fastforth new my-lib --template=library
```

#### `init` - Initialize Project

Initialize current directory as a Forth project.

```bash
fastforth init
```

#### `test` - Run Tests

Run the test suite for a project.

```bash
fastforth test [PATTERN]
```

**Examples:**

```bash
# Run all tests
fastforth test

# Run specific tests
fastforth test math_tests
```

---

## REPL Usage

### Meta-Commands

The REPL supports special commands for interaction:

#### General Commands

- `help` - Show help message
- `help <word>` - Show help for specific word
- `quit` or `exit` - Exit the REPL
- `clear` or `cls` - Clear screen

#### Stack Operations

- `.S` - Show stack contents (non-destructive)
- `CLEAR-STACK` - Clear the stack
- `DEPTH` - Show stack depth

#### Word Inspection

- `SEE <word>` - Show word definition
- `WORDS` - List all defined words
- `WORDS <pattern>` - List words matching pattern

#### Other Commands

- `HISTORY` - Show command history
- `VERSION` - Show Fast Forth version
- `ENV` - Show environment information

### Defining Words

```forth
forth> : DOUBLE ( n -- 2n ) 2 * ;
  ✓ Defined DOUBLE (1.2ms)
  Stack Effect: ( n -- 2n )
  Implementation: 2 *

Stack: [ ]                                     Depth: 0

forth> 5 DOUBLE .
10   ✓ OK (0.8ms)
```

### Multi-line Definitions

```forth
forth> : FACTORIAL ( n -- n! )
...        DUP 1 <= IF DROP 1 ELSE
...          DUP 1 - FACTORIAL *
...        THEN ;
  ✓ Defined FACTORIAL (2.3ms)
```

---

## Compilation

### Basic Compilation

```bash
fastforth compile input.fth
```

This creates an executable named `input` in the same directory.

### Optimization Levels

- **-O0**: No optimization (fastest compilation, slowest execution)
- **-O1**: Basic optimization (default, balanced)
- **-O2**: Aggressive optimization
- **-O3**: Maximum optimization (slowest compilation, fastest execution)

### Compilation Pipeline

The compiler performs these phases:

1. **Lexical Analysis** - Tokenize source code
2. **Parsing** - Build Abstract Syntax Tree (AST)
3. **Type Checking** - Verify stack effects and types
4. **Optimization** - Apply optimization passes
5. **Code Generation** - Generate native code

View phase timings with `--time-passes -v`.

---

## Type Checking

Fast Forth includes a powerful type inference system based on Hindley-Milner type inference.

### Stack Effects

```forth
: SQUARE ( n -- n^2 )
  DUP * ;
```

The type checker verifies:
- Stack depth matches expectations
- Types are consistent
- Stack effects compose correctly

### Running Type Checks

```bash
# Check a file
fastforth check program.fth

# Verbose output
fastforth check program.fth -v
```

---

## Documentation Generation

Generate beautiful HTML or Markdown documentation from stack effect comments.

### Writing Documentable Code

```forth
\ math_ops.fth - Mathematical operations

: SQUARE ( n -- n^2 )
  \ Computes the square of a number
  \ Example: 7 SQUARE . \ Prints 49
  DUP * ;

: CUBE ( n -- n^3 )
  \ Computes the cube of a number
  \ Example: 3 CUBE . \ Prints 27
  DUP DUP * * ;
```

### Generating Documentation

```bash
# Generate HTML documentation
fastforth doc math_ops.fth

# This creates:
# - docs/square.html
# - docs/cube.html
# - docs/index.html
```

The generated documentation includes:
- Word signature and stack effect
- Description from comments
- Examples
- Implementation details

---

## Profiling

### Basic Profiling

```bash
fastforth profile program.fth
```

Output includes:
- **Hot Spots** - Where time is spent
- **Call Graph** - Function call hierarchy
- **Optimization Opportunities** - Suggested improvements
- **Memory Profile** - Allocation patterns

### Flame Graphs

Generate interactive flame graphs for visual analysis:

```bash
fastforth profile program.fth --flame-graph
```

This creates an HTML file with an interactive visualization.

### Interpreting Results

- **Exclusive Time** - Time spent in the function itself
- **Inclusive Time** - Time including called functions
- **Call Count** - Number of times function was called
- **Per Call** - Average time per invocation

---

## Language Server

The Fast Forth LSP provides:

- **Syntax Highlighting** - Semantic token highlighting
- **Autocomplete** - Context-aware completions
- **Hover Documentation** - Inline help
- **Go to Definition** - Jump to word definitions
- **Find References** - Find all usages
- **Diagnostics** - Real-time error checking
- **Code Actions** - Quick fixes

### VSCode Integration

Install the Fast Forth VSCode extension (when available) or configure manually:

```json
{
  "languageServerExample.trace.server": "verbose",
  "forth.lsp.command": "fastforth",
  "forth.lsp.args": ["lsp"]
}
```

---

## Examples

### Hello World

```forth
\ hello.fth
." Hello, World!" CR
```

```bash
fastforth run hello.fth
```

### Factorial

```forth
\ factorial.fth
: FACTORIAL ( n -- n! )
  DUP 1 <= IF DROP 1 ELSE
    DUP 1 - FACTORIAL *
  THEN ;

5 FACTORIAL . CR
```

### Mathematical Operations

```forth
\ math.fth
: SQUARE ( n -- n^2 ) DUP * ;
: CUBE ( n -- n^3 ) DUP DUP * * ;
: AVERAGE ( a b -- avg ) + 2 / ;

7 SQUARE . CR
3 CUBE . CR
10 20 AVERAGE . CR
```

### Complete Project

```bash
# Create project structure
fastforth new my-project
cd my-project

# Write code in src/main.fth

# Compile
fastforth compile src/main.fth -O3

# Run tests
fastforth test

# Generate documentation
fastforth doc src/

# Profile performance
fastforth profile src/main.fth --flame-graph
```

---

## Troubleshooting

### Common Issues

#### "Undefined word" Error

Make sure the word is defined before use. Check spelling.

#### "Stack underflow" Error

The stack doesn't have enough values for an operation. Check stack effects.

#### "Type mismatch" Error

Types don't match expectations. Verify stack effect annotations.

### Getting Help

1. Use `fastforth --help` for command reference
2. Use `fastforth <command> --help` for command-specific help
3. Use `fastforth explain <word>` for word documentation
4. Check the REPL with `help` command

---

## Performance Tips

1. **Use optimization flags**: `-O2` or `-O3` for production
2. **Profile first**: Use `fastforth profile` to find bottlenecks
3. **Minimize allocations**: Reuse stack space when possible
4. **Inline small words**: The optimizer does this automatically at `-O2+`
5. **Use stack caching**: Enabled automatically at optimization levels

---

## Contributing

See the main project README for contribution guidelines.

---

## License

MIT License - See LICENSE file in the project root.

---

**Fast Forth CLI** - Modern, Fast Forth Compiler with Excellent Developer Experience
Version 1.0.0 | Last Updated: 2025-11-14
