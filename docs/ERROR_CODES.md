# Fast Forth Error Code Reference

This document provides comprehensive documentation for all Fast Forth error codes, organized by category with examples and fix suggestions.

## Table of Contents

- [Lexical/Parsing Errors (E0001-E0999)](#lexicalparsingerrors)
- [Semantic Errors (E1000-E1999)](#semanticerrors)
- [Stack Effect Errors (E2000-E2999)](#stackeffecterrors)
- [Control Flow Errors (E3000-E3999)](#controlflowerrors)
- [Optimization Errors (E4000-E4999)](#optimizationerrors)
- [Code Generation Errors (E5000-E5999)](#codegenerationerrors)
- [Internal Errors (E9000-E9999)](#internalerrors)

---

## Lexical/Parsing Errors

### E0001: Unexpected Token

**Description**: An unexpected token was encountered during parsing.

**Common Causes**:
- Mistyped word name
- Invalid syntax
- Missing whitespace

**Example**:
```forth
: square dup** ;  \ Error: ** is not a valid token
```

**Fix**:
```forth
: square dup * * ;  \ Correct: separate operations
```

---

### E0002: Unexpected End of File

**Description**: The parser reached the end of the file unexpectedly, usually indicating an unclosed definition or control structure.

**Common Causes**:
- Missing semicolon to close definition
- Unclosed control structure
- Missing closing parenthesis in comment

**Example**:
```forth
: factorial ( n -- n! )
  dup 2 < if drop 1 else dup 1- recurse * then
\ Error: Missing semicolon
```

**Fix**:
```forth
: factorial ( n -- n! )
  dup 2 < if drop 1 else dup 1- recurse * then ;
```

---

### E0003: Invalid Number

**Description**: A number literal could not be parsed.

**Common Causes**:
- Invalid digit for current base
- Malformed floating-point number
- Non-numeric characters in number

**Example**:
```forth
42G  \ Error: G is not a valid digit
```

---

### E0004: Invalid String Literal

**Description**: A string literal is malformed.

**Common Causes**:
- Unterminated string
- Invalid escape sequence

**Example**:
```forth
." Hello World  \ Error: Missing closing quote
```

**Fix**:
```forth
." Hello World"
```

---

## Semantic Errors

### E1000: Undefined Word

**Description**: Reference to a word that hasn't been defined.

**Common Causes**:
- Typo in word name
- Using a word before it's defined
- Forgot to load required library

**Example**:
```forth
: test-word
  undefined-operation 42 ;
\ Error: undefined-operation not found
```

**Fix Suggestions**:
1. Check spelling
2. Define the word before use
3. Ensure required libraries are loaded

**Pattern**: `SUGGEST_SIMILAR_WORD_001` - Suggests similarly named words

---

### E1001: Redefined Word

**Description**: Attempt to redefine an existing word without proper override.

**Example**:
```forth
: dup dup dup ;  \ Error: dup is already defined
```

**Note**: Some Forth systems allow redefinition; Fast Forth requires explicit override for safety.

---

### E1002: Invalid Stack Effect

**Description**: The declared stack effect doesn't match the actual effect.

**Example**:
```forth
: add-three ( a b -- sum )  \ Declares 2 inputs, 1 output
  + + ;                     \ Actually needs 3 inputs
\ Error: Stack effect mismatch
```

**Fix**:
```forth
: add-three ( a b c -- sum )
  + + ;
```

---

## Stack Effect Errors

### E2000: Stack Underflow

**Description**: Operation requires more items on the stack than are available.

**Common Causes**:
- Not enough inputs provided
- Consuming stack items without checking depth
- Incorrect word composition

**Example**:
```forth
*  \ Error: Multiply needs 2 items, stack is empty
```

**Fix Pattern**: `ADD_INPUTS_002`

**Suggestions**:
1. Add `dup` to duplicate existing value
2. Add literal value
3. Review stack effect of previous operations

---

### E2001: Stack Overflow

**Description**: Too many items on the stack (exceeded maximum depth).

**Common Causes**:
- Infinite loop generating values
- Not consuming intermediate results
- Excessive nesting

**Maximum Stack Depth**: 1024 items (configurable)

---

### E2234: Stack Depth Mismatch

**Description**: The final stack depth doesn't match the declared stack effect.

**Example**:
```forth
: square ( n -- n² )
  dup dup * ;  \ Leaves 2 items: n and n²
\ Error: Expected 1 output, got 2
```

**Fix Pattern**: `DROP_EXCESS_001`

**Fix**:
```forth
: square ( n -- n² )
  dup * ;  \ Correct: consumes both inputs
```

**Auto-Fix**: Add `drop` to remove excess items (confidence: 85%)

---

### E2300: Type Mismatch

**Description**: Type incompatibility in stack operation.

**Common Causes**:
- Using integer where address expected
- Mixing signed and unsigned incorrectly
- Incorrect operand order

**Fix Pattern**: `SWAP_BEFORE_OP_006`

**Example**:
```forth
: subtract ( a b -- a-b )
  swap - ;  \ May need swap depending on desired order
```

---

### E2400: Insufficient Inputs

**Description**: Operation doesn't have enough inputs.

**Fix Patterns**:
- `ADD_INPUTS_002` - Add DUP or literals
- `OVER_BEFORE_OP_007` - Use OVER to access second item

---

## Control Flow Errors

### E3000: Unmatched IF

**Description**: IF without matching THEN.

**Example**:
```forth
: abs ( n -- |n| )
  dup 0 < if negate  \ Missing THEN
;
```

**Fix Pattern**: `ADD_THEN_003` (confidence: 95%)

**Fix**:
```forth
: abs ( n -- |n| )
  dup 0 < if negate then ;
```

---

### E3001: Unmatched THEN

**Description**: THEN without matching IF.

**Example**:
```forth
: bad-word
  42 then ;  \ No IF before THEN
```

---

### E3002: Unmatched ELSE

**Description**: ELSE without matching IF.

---

### E3010: Unmatched DO

**Description**: DO without matching LOOP.

**Example**:
```forth
: count-to-10
  10 0 do i . ;  \ Missing LOOP
```

**Fix Pattern**: `ADD_LOOP_004` (confidence: 95%)

**Fix**:
```forth
: count-to-10
  10 0 do i . loop ;
```

---

### E3020: Unmatched BEGIN

**Description**: BEGIN without matching UNTIL, WHILE, or REPEAT.

**Example**:
```forth
: countdown ( n -- )
  begin dup 0 > ;  \ Missing UNTIL
```

**Fix Pattern**: `ADD_UNTIL_005` (confidence: 85%)

**Fix**:
```forth
: countdown ( n -- )
  begin dup 0 > while
    dup . 1-
  repeat drop ;
```

---

## Optimization Errors

### E4000: Optimization Failed

**Description**: An optimization pass encountered an error.

**Note**: This is typically an internal error. If encountered, try compiling with lower optimization level.

**Workaround**:
```bash
fastforth compile -O0 input.forth  # No optimization
```

---

### E4001: Inlining Error

**Description**: Function inlining failed.

**Common Causes**:
- Recursive function can't be inlined
- Function too large for inlining threshold
- Stack effect too complex

---

## Code Generation Errors

### E5000: Code Generation Failed

**Description**: Backend code generation encountered an error.

**Common Causes**:
- Unsupported operation
- Complex control flow
- Platform-specific limitation

---

### E5001: LLVM Error

**Description**: LLVM backend returned an error.

**Debug**: Compile with `--verbose` flag for detailed LLVM diagnostics.

---

## Internal Errors

### E9000: Internal Compiler Error

**Description**: An unexpected internal error occurred.

**Action**: Please report this as a bug with:
1. Source code that triggered the error
2. Compiler version
3. Full error message
4. Platform/OS information

---

### E9001: SSA Conversion Error

**Description**: Conversion to SSA form failed.

**Note**: This is typically a compiler bug. Please report if encountered.

---

## Using Error Codes with Agent Mode

When using Fast Forth in agent/automated mode, errors are returned as structured JSON:

### Command:
```bash
fastforth compile --agent-mode --suggest-fixes input.forth
```

### Example JSON Error Output:
```json
{
  "error": "Stack depth mismatch in 'square'",
  "code": "E2234",
  "expected_effect": "( n -- n² )",
  "actual_effect": "( n -- n n² )",
  "location": {
    "file": "input.forth",
    "line": 3,
    "column": 5,
    "word": "square"
  },
  "suggestion": {
    "pattern": "DROP_EXCESS_001",
    "fix": "Add 'drop' after 'dup dup *'",
    "confidence": 0.85,
    "diff": {
      "old": "dup dup *",
      "new": "dup dup * drop"
    },
    "explanation": "Stack has more items than expected by the declared effect"
  },
  "alternatives": [
    {
      "pattern": "CHANGE_EFFECT_002",
      "fix": "Update stack effect declaration to ( n -- n n² )",
      "confidence": 0.65,
      "diff": {
        "old": "( n -- n² )",
        "new": "( n -- n n² )"
      }
    }
  ],
  "severity": "error"
}
```

### Using Error Codes Programmatically

```python
import subprocess
import json

result = subprocess.run(
    ["fastforth", "compile", "--agent-mode", "--suggest-fixes", "input.forth"],
    capture_output=True,
    text=True
)

if result.returncode != 0:
    error = json.loads(result.stdout)
    print(f"Error {error['code']}: {error['error']}")

    if 'suggestion' in error:
        print(f"Suggestion: {error['suggestion']['fix']}")
        print(f"Confidence: {error['suggestion']['confidence']:.0%}")

        # Apply fix automatically
        if error['suggestion']['confidence'] > 0.8:
            # Apply the suggested fix
            apply_fix(error['suggestion']['diff'])
```

---

## Error Code Quick Reference

| Code | Category | Description | Fix Pattern | Confidence |
|------|----------|-------------|-------------|------------|
| E0001 | Parsing | Unexpected Token | - | - |
| E0002 | Parsing | Unexpected EOF | - | - |
| E1000 | Semantic | Undefined Word | SUGGEST_SIMILAR | 0.60 |
| E1001 | Semantic | Redefined Word | - | - |
| E2000 | Stack | Stack Underflow | ADD_INPUTS_002 | 0.70 |
| E2234 | Stack | Stack Depth Mismatch | DROP_EXCESS_001 | 0.85 |
| E2300 | Stack | Type Mismatch | SWAP_BEFORE_OP_006 | 0.75 |
| E3000 | Control | Unmatched IF | ADD_THEN_003 | 0.95 |
| E3010 | Control | Unmatched DO | ADD_LOOP_004 | 0.95 |
| E3020 | Control | Unmatched BEGIN | ADD_UNTIL_005 | 0.85 |
| E9000 | Internal | Internal Error | - | - |

---

## Compiler Flags Reference

### Agent-Specific Flags

| Flag | Description | Example |
|------|-------------|---------|
| `--agent-mode` | JSON output only, compact diagnostics | `fastforth compile --agent-mode file.forth` |
| `--error-format=json` | Structured JSON errors (pretty-printed) | `fastforth compile --error-format=json file.forth` |
| `--verify-only` | Type check without code generation | `fastforth compile --verify-only file.forth` |
| `--suggest-fixes` | Include auto-fix suggestions | `fastforth compile --suggest-fixes file.forth` |

### Output Formats

- `human` (default): Colored, human-friendly error messages
- `json`: Compact JSON (single line)
- `json-pretty`: Pretty-printed JSON with indentation
- `plain`: Plain text without colors

---

## Contributing

Found an error code that needs better documentation? Submit a PR or issue at:
https://github.com/fastforth/fastforth

---

**Last Updated**: 2025-01-15
**FastForth Version**: 0.1.0
