# Fast Forth Visual Design Mockups

**Design System**: Fast Forth Developer Tools
**Version**: 1.0
**Last Updated**: 2025-11-14

---

## 1. CLI Error Message Mockup (Terminal)

```
┌──────────────────────────────────────────────────────────────────────┐
│ $ fastforth compile math.fth                                         │
└──────────────────────────────────────────────────────────────────────┘

error: Stack underflow in word 'AVERAGE'

  Context: File 'math.fth', line 15, column 8

  Expected: 2 items on stack
  Actual:   1 item on stack

  Code:
    14 | : AVERAGE ( a b -- avg )
    15 |     + 2 / ;
              ^
              Stack underflow here

  Stack State:
    Before +: [ 42 13 ]  ✓ OK (2 items)
    After +:  [ 55 ]     ✓ OK (1 item)
    Before 2: [ 55 ]     ✓ OK (1 item)
    Before /: [ 55 2 ]   ✓ OK (2 items)
    After /:  [ 27 ]     ⚠ Expects 2 items, found 1

  Tip: The stack effect comment says ( a b -- avg ), meaning AVERAGE
       should consume 2 values and produce 1. Your implementation
       consumes 2 values but tries to divide the result by 2, which
       requires an additional value on the stack.

       Did you mean: + 2.0 /  (floating point division)
       Or perhaps:   + 2 RSHIFT  (for integers)

  Learn more: https://fastforth.dev/docs/stack-effects

┌──────────────────────────────────────────────────────────────────────┐
│ Color Scheme:                                                        │
│   error:   Red (#E74C3C)                                            │
│   Code:    White/Default                                            │
│   Pointer: Bright Red (^)                                           │
│   Success: Green (#2ECC71) for ✓                                    │
│   Warning: Yellow (#F39C12) for ⚠                                    │
└──────────────────────────────────────────────────────────────────────┘
```

---

## 2. REPL Interface Mockup

```
┌─────────────────────────────────────────────────────────────────────┐
│ Fast Forth REPL v1.0.0                                              │
│ Type 'help' for help, 'quit' to exit                               │
└─────────────────────────────────────────────────────────────────────┘

Stack: [ ]  (empty)                                        Depth: 0

forth> 5 3 +

  ✓ OK (0.4ms)

Stack: [ 8 ]                                               Depth: 1

forth> DUP

  ✓ OK (0.2ms)

Stack: [ 8 8 ]                                             Depth: 2

forth> *

  ✓ OK (0.3ms)

Stack: [ 64 ]                                              Depth: 1

forth> .

64 ✓ OK (0.5ms)

Stack: [ ]                                                 Depth: 0

forth> : SQUARE ( n -- n² )
...    DUP * ;

  ✓ Defined SQUARE (1.2ms)

  Stack Effect: ( n -- n² )
  Implementation: DUP *
  Inlined: Yes (simple)

Stack: [ ]                                                 Depth: 0

forth> 7 SQUARE .

49 ✓ OK (0.8ms)

Stack: [ ]                                                 Depth: 0

forth> _

┌─────────────────────────────────────────────────────────────────────┐
│ Design Elements:                                                    │
│   • Minimal, clean interface                                       │
│   • Clear visual hierarchy (Stack → Prompt → Output)              │
│   • Success indicators (✓) in green                               │
│   • Timing information for performance awareness                   │
│   • Stack visualization after each command                         │
│   • Multi-line editing with ... continuation prompt               │
└─────────────────────────────────────────────────────────────────────┘
```

---

## 3. Profiler Output Mockup

```
═══════════════════════════════════════════════════════════════════════
  Fast Forth Profiler v1.0.0
  Target: compute.fth
  Runtime: 2.34 seconds (2,340ms)
═══════════════════════════════════════════════════════════════════════

TOP 10 HOT SPOTS (by exclusive time):

 #  Word            Time      %    Calls    Per Call  Notes
────────────────────────────────────────────────────────────────────────
 1  INNER-LOOP      1,057ms  45.2%  1.2M    0.88μs   🔥 HOT
 2  COMPUTE          541ms   23.1%  500K    1.08μs
 3  FORMAT-OUTPUT    288ms   12.3%  100K    2.88μs   I/O bound
 4  VALIDATE         176ms    7.5%  500K    0.35μs
 5  ALLOCATE          94ms    4.0%   50K    1.88μs
 6  /                 82ms    3.5%  1.2M    0.07μs   ⚡ Optimize
 7  SQRT              47ms    2.0%  500K    0.09μs
 8  DUP               23ms    1.0%  3.2M    0.01μs
 9  *                 18ms    0.8%  1.8M    0.01μs
10  +                 14ms    0.6%  2.1M    0.01μs

────────────────────────────────────────────────────────────────────────

CALL GRAPH (top 3 paths):

1. MAIN → INNER-LOOP → COMPUTE → VALIDATE
   1,057ms → 541ms → 176ms (75% of total)

2. MAIN → INNER-LOOP → FORMAT-OUTPUT
   1,057ms → 288ms (12% of total)

3. MAIN → INNER-LOOP → COMPUTE → SQRT
   1,057ms → 541ms → 47ms (2% of total)

────────────────────────────────────────────────────────────────────────

OPTIMIZATION OPPORTUNITIES:

🔥 CRITICAL: INNER-LOOP (1,057ms)

   Issue: Contains expensive division operation

   Code:
     : INNER-LOOP ( n -- )
         0 DO I 10 / COMPUTE LOOP ;
                  ^
                  Hot spot

   Recommendation:
     Replace division with bit shift:
     I 10 / → I 3 RSHIFT (65% faster)

   Expected Impact: -689ms (29% total speedup)

────────────────────────────────────────────────────────────────────────

SUMMARY:

✓ Program executed successfully
⚠ 3 optimization opportunities identified

Total Potential Speedup: -941ms (40% faster)

Run 'fastforth profile --flame-graph compute.fth' for visualization

═══════════════════════════════════════════════════════════════════════

┌─────────────────────────────────────────────────────────────────────┐
│ Visual Hierarchy:                                                   │
│   • Box drawing characters for structure                           │
│   • Emoji indicators for urgency (🔥 ⚡ 💡)                         │
│   • Percentage bars and visual alignment                           │
│   • Color coding: Red (hot), Yellow (warm), Gray (cool)           │
│   • Actionable recommendations with code snippets                  │
└─────────────────────────────────────────────────────────────────────┘
```

---

## 4. VSCode Extension Interface Mockup

```
┌─────────────────────────────────────────────────────────────────────┐
│ File: math.fth                                          [× Close]   │
├─────────────────────────────────────────────────────────────────────┤
│ 1  \ math.fth - Mathematical operations                             │
│ 2  \ Fast Forth standard library                                    │
│ 3                                                                    │
│ 4  : SQUARE ( n -- n² )                                             │
│ 5    \ Square a number by multiplying it with itself               │
│ 6    DUP * ;                                                        │
│ 7                                                                    │
│ 8  : AVERAGE ( a b -- avg )                                         │
│ 9    \ Compute the average of two numbers                          │
│10    + 2 / ;                                                        │
│        ^^^^                                                          │
│        └─ Hover: + (built-in)                                       │
│           ( a b -- sum )                                            │
│           Adds two numbers                                          │
│                                                                      │
│11                                                                    │
│12  : FACTORIAL ( n -- n! )                                          │
│13    \ Recursive factorial implementation                           │
│14    DUP 1 <= IF                                                    │
│15      DROP 1                                                       │
│16    ELSE                                                           │
│17      DUP 1 - FACTORIAL *                                          │
│18    THEN ;                                                         │
│19                                                                    │
│20  \ Test                                                           │
│21  5 AVERAGE .  \ ← Error: Stack underflow                          │
│      ~~~~~~~~                                                        │
│      └─ ✗ Stack underflow: AVERAGE expects 2 items, found 1        │
│                                                                      │
│22                                                                    │
└─────────────────────────────────────────────────────────────────────┘

┌─ Autocomplete ────────────────────────────────────────────┐
│ AV_                                                       │
│                                                           │
│ ◉ AVERAGE ( a b -- avg )                                  │
│   Compute average of two numbers                          │
│                                                           │
│ ○ AVERAGE-WEIGHTED ( a b weight -- weighted-avg )        │
│   Compute weighted average                                │
│                                                           │
│ [Press Tab to complete, ↑↓ to navigate]                  │
└───────────────────────────────────────────────────────────┘

┌─ Problems ────────────────────────────────────────────────┐
│ 1 Error, 0 Warnings                                       │
├───────────────────────────────────────────────────────────┤
│ ✗ Stack underflow in word AVERAGE                         │
│   math.fth:21:6                                           │
│   Expected 2 items on stack, found 1                      │
│                                                           │
│   💡 Quick Fix: Push another value before AVERAGE         │
└───────────────────────────────────────────────────────────┘

┌─ Outline ─────────────────────────────────────────────────┐
│ 📄 math.fth                                               │
│   ├─ ƒ SQUARE (line 4)                                    │
│   ├─ ƒ AVERAGE (line 8)                                   │
│   └─ ƒ FACTORIAL (line 12)                                │
└───────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────┐
│ Syntax Highlighting Color Scheme:                                  │
│   • Keywords (:, IF, THEN, ELSE, DO, LOOP): #d73a49 (red)         │
│   • Numbers (42, 3.14): #005cc5 (blue)                            │
│   • Strings ("Hello"): #032f62 (dark blue)                        │
│   • Comments (\ ...): #6a737d (gray italic)                       │
│   • Stack effects (( a b -- c )): #6f42c1 (purple)                │
│   • Operators (+, -, *, /): #d73a49 (red)                         │
│   • User words: default text color                                 │
└─────────────────────────────────────────────────────────────────────┘
```

---

## 5. Documentation Generator Output Mockup (HTML)

```html
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <title>Fast Forth Documentation - AVERAGE</title>
  <style>
    body {
      font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
      max-width: 900px;
      margin: 0 auto;
      padding: 40px;
      line-height: 1.6;
      color: #24292e;
      background: #ffffff;
    }

    .header {
      border-bottom: 2px solid #0366d6;
      padding-bottom: 20px;
      margin-bottom: 30px;
    }

    .word-name {
      font-size: 2.5em;
      font-weight: 600;
      color: #1a1a1a;
      margin: 0;
    }

    .stack-effect {
      font-family: 'Monaco', 'Menlo', monospace;
      font-size: 1.4em;
      color: #6f42c1;
      margin: 10px 0;
    }

    .description {
      font-size: 1.1em;
      margin: 20px 0;
      color: #586069;
    }

    .section {
      margin: 40px 0;
    }

    .section-title {
      font-size: 1.5em;
      font-weight: 600;
      color: #24292e;
      margin-bottom: 15px;
      border-bottom: 1px solid #e1e4e8;
      padding-bottom: 10px;
    }

    .code-example {
      background: #f6f8fa;
      border-left: 4px solid #0366d6;
      padding: 16px;
      margin: 20px 0;
      font-family: 'Monaco', monospace;
      font-size: 14px;
      border-radius: 4px;
      overflow-x: auto;
    }

    .performance-table {
      width: 100%;
      border-collapse: collapse;
      margin: 20px 0;
    }

    .performance-table td {
      padding: 10px;
      border: 1px solid #e1e4e8;
    }

    .performance-table td:first-child {
      font-weight: 600;
      width: 150px;
      background: #f6f8fa;
    }

    .see-also {
      margin-top: 40px;
      padding-top: 20px;
      border-top: 2px solid #e1e4e8;
    }

    .related-link {
      display: inline-block;
      margin: 5px 10px 5px 0;
      padding: 8px 16px;
      background: #0366d6;
      color: white;
      text-decoration: none;
      border-radius: 4px;
      font-size: 14px;
    }

    .related-link:hover {
      background: #0256c7;
    }
  </style>
</head>
<body>
  <div class="header">
    <h1 class="word-name">AVERAGE</h1>
    <div class="stack-effect">( a b -- avg )</div>
    <p class="description">
      Computes the average of two numbers by adding them together
      and dividing by 2.
    </p>
  </div>

  <div class="section">
    <h2 class="section-title">Examples</h2>
    <div class="code-example">10 20 AVERAGE .  \ Prints 15
5 15 AVERAGE .   \ Prints 10
0 100 AVERAGE .  \ Prints 50</div>
  </div>

  <div class="section">
    <h2 class="section-title">Implementation</h2>
    <div class="code-example">: AVERAGE ( a b -- avg )
  + 2 / ;</div>
  </div>

  <div class="section">
    <h2 class="section-title">Performance</h2>
    <table class="performance-table">
      <tr>
        <td>Complexity</td>
        <td>O(1)</td>
      </tr>
      <tr>
        <td>Inlined</td>
        <td>Yes</td>
      </tr>
      <tr>
        <td>CPU Cycles</td>
        <td>~3</td>
      </tr>
      <tr>
        <td>Memory Usage</td>
        <td>None (stack-based)</td>
      </tr>
    </table>
  </div>

  <div class="section">
    <h2 class="section-title">Notes</h2>
    <ul>
      <li>Uses integer division; result is truncated</li>
      <li>For floating-point average, use: + 2.0 /</li>
      <li>For more than 2 numbers, see <a href="mean.html">MEAN</a></li>
    </ul>
  </div>

  <div class="see-also">
    <h3>See Also</h3>
    <a href="plus.html" class="related-link">+</a>
    <a href="divide.html" class="related-link">/</a>
    <a href="median.html" class="related-link">MEDIAN</a>
    <a href="mean.html" class="related-link">MEAN</a>
    <a href="weighted-average.html" class="related-link">WEIGHTED-AVERAGE</a>
  </div>
</body>
</html>
```

**Visual Design Notes**:
- Clean, minimalist aesthetic inspired by Stripe documentation
- Clear visual hierarchy with typography scale
- Syntax-highlighted code examples with subtle background
- Color scheme: Blue (#0366d6) for primary actions, Purple (#6f42c1) for code
- Responsive design (works on mobile, tablet, desktop)
- Accessible (WCAG 2.1 AA compliant)

---

## 6. Command-Line Help Mockup

```
$ fastforth --help

┌─────────────────────────────────────────────────────────────────────┐
│ Fast Forth v1.0.0                                                   │
│ A modern, fast Forth compiler with excellent developer experience  │
└─────────────────────────────────────────────────────────────────────┘

USAGE:
    fastforth [COMMAND] [OPTIONS]

COMMANDS:
    repl        Start interactive REPL (default)
    compile     Compile source to executable
    run         JIT compile and execute
    check       Type check without execution
    profile     Profile execution performance
    doc         Generate documentation
    lsp         Start language server

    format      Auto-format source code
    explain     Explain word behavior
    benchmark   Benchmark performance

    new         Create new project
    init        Initialize current directory
    test        Run test suite

OPTIONS:
    -v, --verbose      Verbose output
    -q, --quiet        Quiet mode (suppress non-error output)
        --json         JSON output (for tooling integration)
    -h, --help         Print help
    -V, --version      Print version

EXAMPLES:
    fastforth                          # Start REPL
    fastforth run hello.fth            # Execute program
    fastforth compile -O3 app.fth      # Compile with optimization
    fastforth profile app.fth          # Profile performance

DOCUMENTATION:
    https://fastforth.dev/docs
    https://fastforth.dev/tutorial
    https://github.com/fastforth/fastforth

For command-specific help, use: fastforth <command> --help
```

---

## Design System Summary

### Typography
```
Font Stack:
  --font-mono: 'Monaco', 'Menlo', 'Consolas', monospace
  --font-sans: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif

Type Scale:
  --text-xs: 12px
  --text-sm: 14px
  --text-base: 16px
  --text-lg: 18px
  --text-xl: 24px
  --text-2xl: 32px
```

### Color Palette
```
Brand Colors:
  --primary: #0366d6       (Primary blue)
  --success: #28a745       (Success green)
  --warning: #ffd33d       (Warning yellow)
  --error: #d73a49         (Error red)
  --info: #6f42c1          (Info purple)

UI Colors:
  --background: #ffffff
  --surface: #f6f8fa
  --border: #e1e4e8
  --text: #24292e
  --text-secondary: #586069
```

### Component Patterns
```
Box Drawing:
  ┌─┬─┐  Top border
  ├─┼─┤  Middle separator
  └─┴─┘  Bottom border

Icons:
  ✓ Success
  ✗ Error
  ⚠ Warning
  💡 Hint
  🔥 Critical
  ⚡ Important
  📄 File
  ƒ Function
  𝑣 Variable
```

---

**Design Philosophy**: "Clarity through simplicity - every visual element serves a purpose, guiding users to understanding and action."

**Status**: Mockups Complete, Ready for Implementation
