# Fast Forth Error Message Examples

This document demonstrates the error message formats in both human-friendly and agent-friendly (JSON) modes.

## Example 1: Stack Depth Mismatch (E2234)

### Source Code:
```forth
: square ( n -- n² )
  dup dup * ;
```

### Human Format (`--error-format=human`):

```
Error: Stack depth mismatch in 'square'
  Code: E2234
  Location: example.forth:2:5 in word 'square'

  Expected: ( n -- n² )
  Actual:   ( n -- n n² )

  dup dup *
      ^^^^^ Error here

  Suggestion: Add 'drop' after 'dup dup *'
  Pattern: DROP_EXCESS_001
  Confidence: 85%
  Reason: Stack has more items than expected by the declared effect

  Code Diff:
  - dup dup *
  + dup dup * drop

  Alternative Fixes:
  1. Update stack effect to ( n -- n n² ) (confidence: 65%)
```

### JSON Format (`--error-format=json-pretty`):

```json
{
  "error": "Stack depth mismatch in 'square'",
  "code": "E2234",
  "expected_effect": "( n -- n² )",
  "actual_effect": "( n -- n n² )",
  "location": {
    "file": "example.forth",
    "line": 2,
    "column": 5,
    "word": "square",
    "context": "dup dup *"
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
      "fix": "Update stack effect to ( n -- n n² )",
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

### Agent Mode (`--agent-mode`):

```json
{"error":"Stack depth mismatch in 'square'","code":"E2234","expected_effect":"( n -- n² )","actual_effect":"( n -- n n² )","location":{"file":"example.forth","line":2,"column":5,"word":"square"},"suggestion":{"pattern":"DROP_EXCESS_001","fix":"Add 'drop' after 'dup dup *'","confidence":0.85,"diff":{"old":"dup dup *","new":"dup dup * drop"}},"severity":"error"}
```

---

## Example 2: Unmatched IF (E3000)

### Source Code:
```forth
: abs ( n -- |n| )
  dup 0 < if negate ;
```

### Human Format:

```
Error: Unmatched IF - missing THEN
  Code: E3000
  Location: example.forth:2:15 in word 'abs'

  dup 0 < if negate
          ^^ IF without matching THEN

  Suggestion: Add 'then' before semicolon
  Pattern: ADD_THEN_003
  Confidence: 95%
  Reason: Every IF must have a matching THEN

  Code Diff:
  - dup 0 < if negate
  + dup 0 < if negate then
```

### JSON Format:

```json
{
  "error": "Unmatched IF - missing THEN",
  "code": "E3000",
  "location": {
    "file": "example.forth",
    "line": 2,
    "column": 15,
    "word": "abs",
    "context": "dup 0 < if negate"
  },
  "suggestion": {
    "pattern": "ADD_THEN_003",
    "fix": "Add 'then' before semicolon",
    "confidence": 0.95,
    "diff": {
      "old": "dup 0 < if negate",
      "new": "dup 0 < if negate then"
    },
    "explanation": "Every IF must have a matching THEN"
  },
  "severity": "error"
}
```

---

## Example 3: Stack Underflow (E2000)

### Source Code:
```forth
: broken-multiply ( -- result )
  * ;  \ Error: multiply needs 2 inputs
```

### Human Format:

```
Error: Stack underflow in 'broken-multiply'
  Code: E2000
  Location: example.forth:2:3 in word 'broken-multiply'

  *
  ^ Requires 2 items, stack has 0

  Suggestion: Add 'dup' before operation
  Pattern: ADD_INPUTS_002
  Confidence: 70%
  Reason: Operation needs more inputs than available

  Code Diff:
  - *
  + dup *

  Alternative Fixes:
  1. Add literal values: 5 10 * (confidence: 60%)
  2. Use OVER to access second item (confidence: 65%)
```

### JSON Format:

```json
{
  "error": "Stack underflow in 'broken-multiply'",
  "code": "E2000",
  "location": {
    "file": "example.forth",
    "line": 2,
    "column": 3,
    "word": "broken-multiply"
  },
  "suggestion": {
    "pattern": "ADD_INPUTS_002",
    "fix": "Add 'dup' before operation",
    "confidence": 0.70,
    "diff": {
      "old": "*",
      "new": "dup *"
    },
    "explanation": "Operation needs more inputs than available"
  },
  "alternatives": [
    {
      "fix": "Add literal values",
      "confidence": 0.60,
      "diff": {
        "old": "*",
        "new": "5 10 *"
      }
    },
    {
      "pattern": "OVER_BEFORE_OP_007",
      "fix": "Use OVER to access second item",
      "confidence": 0.65,
      "diff": {
        "old": "*",
        "new": "over *"
      }
    }
  ],
  "severity": "error"
}
```

---

## Example 4: Undefined Word (E1000)

### Source Code:
```forth
: test-word
  unknwon-operation 42 ;  \ Typo: unknown -> unknwon
```

### Human Format:

```
Error: Undefined word 'unknwon-operation'
  Code: E1000
  Location: example.forth:2:3 in word 'test-word'

  unknwon-operation 42
  ^^^^^^^^^^^^^^^^^

  Suggestion: Did you mean 'unknown-operation'?
  Pattern: SUGGEST_SIMILAR_WORD_001
  Confidence: 60%
  Reason: Similar word found in dictionary

  Related:
  - Check spelling
  - Ensure word is defined before use
  - Load required libraries
```

### JSON Format:

```json
{
  "error": "Undefined word 'unknwon-operation'",
  "code": "E1000",
  "location": {
    "file": "example.forth",
    "line": 2,
    "column": 3,
    "word": "test-word"
  },
  "suggestion": {
    "pattern": "SUGGEST_SIMILAR_WORD_001",
    "fix": "Did you mean 'unknown-operation'?",
    "confidence": 0.60,
    "explanation": "Similar word found in dictionary"
  },
  "related_errors": [
    "Check spelling",
    "Ensure word is defined before use",
    "Load required libraries"
  ],
  "severity": "error"
}
```

---

## Example 5: Successful Compilation (Agent Mode)

### Source Code:
```forth
: square ( n -- n² )
  dup * ;

: cube ( n -- n³ )
  dup square * ;
```

### Agent Mode Success Output:

```json
{
  "status": "success",
  "mode": "AOT",
  "compile_time_ms": 23,
  "definitions_count": 2,
  "optimization_savings": 0.15,
  "output_path": "output.o"
}
```

---

## Command Line Usage Examples

### 1. Human-friendly errors (default):
```bash
$ fastforth compile example.forth
```

### 2. JSON errors for parsing:
```bash
$ fastforth compile --error-format=json example.forth
```

### 3. Pretty-printed JSON:
```bash
$ fastforth compile --error-format=json-pretty example.forth
```

### 4. Agent mode with auto-fix suggestions:
```bash
$ fastforth compile --agent-mode --suggest-fixes example.forth
```

### 5. Verify type checking only:
```bash
$ fastforth compile --verify-only example.forth
```

### 6. Plain text (no colors):
```bash
$ fastforth compile --error-format=plain example.forth
```

---

## Parsing Errors in Scripts

### Python Example:

```python
#!/usr/bin/env python3
import subprocess
import json

def compile_forth(source_file, auto_fix=True):
    """Compile Forth code with automatic error fixing."""

    cmd = [
        "fastforth", "compile",
        "--agent-mode",
        "--suggest-fixes",
        source_file
    ]

    result = subprocess.run(cmd, capture_output=True, text=True)

    if result.returncode == 0:
        # Success
        output = json.loads(result.stdout)
        print(f"✓ Compiled successfully in {output['compile_time_ms']}ms")
        print(f"  Definitions: {output['definitions_count']}")
        return True
    else:
        # Error with suggestions
        error = json.loads(result.stdout)
        print(f"✗ Error {error['code']}: {error['error']}")

        if 'suggestion' in error and error['suggestion']['confidence'] > 0.8:
            print(f"\n💡 Suggestion (confidence: {error['suggestion']['confidence']:.0%}):")
            print(f"   {error['suggestion']['fix']}")
            print(f"\n   Diff:")
            print(f"   - {error['suggestion']['diff']['old']}")
            print(f"   + {error['suggestion']['diff']['new']}")

            if auto_fix:
                # Apply the fix
                return apply_fix(source_file, error['suggestion'])

        return False

def apply_fix(source_file, suggestion):
    """Apply suggested fix to source file."""
    with open(source_file, 'r') as f:
        content = f.read()

    # Apply the diff
    fixed_content = content.replace(
        suggestion['diff']['old'],
        suggestion['diff']['new']
    )

    # Write back
    with open(source_file, 'w') as f:
        f.write(fixed_content)

    print(f"\n✓ Applied fix from pattern {suggestion['pattern']}")

    # Retry compilation
    return compile_forth(source_file, auto_fix=False)

if __name__ == "__main__":
    import sys
    if len(sys.argv) < 2:
        print("Usage: compile.py <source.forth>")
        sys.exit(1)

    success = compile_forth(sys.argv[1])
    sys.exit(0 if success else 1)
```

### Rust Example:

```rust
use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Deserialize)]
struct CompileError {
    error: String,
    code: String,
    suggestion: Option<Suggestion>,
}

#[derive(Deserialize)]
struct Suggestion {
    pattern: Option<String>,
    fix: String,
    confidence: f64,
    diff: Diff,
}

#[derive(Deserialize)]
struct Diff {
    old: String,
    new: String,
}

fn compile_with_auto_fix(source_file: &str) -> Result<(), String> {
    let output = Command::new("fastforth")
        .args(&["compile", "--agent-mode", "--suggest-fixes", source_file])
        .output()
        .map_err(|e| e.to_string())?;

    if output.status.success() {
        println!("✓ Compilation successful");
        Ok(())
    } else {
        let error: CompileError = serde_json::from_slice(&output.stdout)
            .map_err(|e| e.to_string())?;

        println!("✗ Error {}: {}", error.code, error.error);

        if let Some(suggestion) = error.suggestion {
            if suggestion.confidence > 0.8 {
                println!("💡 Applying suggested fix (confidence: {:.0}%)",
                         suggestion.confidence * 100.0);

                // Apply fix and recompile
                apply_fix(source_file, &suggestion.diff)?;
                return compile_with_auto_fix(source_file);
            }
        }

        Err(error.error)
    }
}
```

---

## Summary

Fast Forth provides three distinct error output modes optimized for different use cases:

1. **Human Mode** (default): Colored, detailed, with visual indicators
2. **JSON Mode**: Structured, parseable, with all metadata
3. **Agent Mode**: Compact JSON, optimized for automated tools

The `--suggest-fixes` flag enables auto-fix suggestions with confidence scores, allowing agents to automatically correct common errors with high reliability (85-95% confidence for structural errors like missing THEN, LOOP, etc.).
