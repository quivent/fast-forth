# Fast Forth Agent Programming Context

**For AI Agents**: Load this file into your context to optimize Fast Forth code generation.

---

## Fast Forth Agent Programming Context

You are generating code for Fast Forth, a stack-based compiled language with Hindley-Milner type inference.

### Core Principles

1. **Stack-First Thinking**: All operations manipulate an implicit data stack. No variables needed.
2. **Postfix Notation**: Arguments before operations. `3 4 +` not `3 + 4`
3. **Stack Effects Are Types**: Every word has signature `( inputs -- outputs )`
4. **Composition Over Variables**: Compose small words rather than storing intermediate results

### Essential Stack Words

```forth
\ Stack manipulation
dup    ( a -- a a )           \ Duplicate top
swap   ( a b -- b a )         \ Swap top two
over   ( a b -- a b a )       \ Copy second to top
rot    ( a b c -- b c a )     \ Rotate third to top
drop   ( a -- )               \ Discard top
2dup   ( a b -- a b a b )     \ Duplicate top two

\ Arithmetic
+      ( a b -- a+b )
-      ( a b -- a-b )
*      ( a b -- a*b )
/      ( a b -- a/b )
mod    ( a b -- a%b )

\ Comparison (produce flag: -1=true, 0=false)
<      ( a b -- flag )
>      ( a b -- flag )
=      ( a b -- flag )

\ Control flow
if ... then              \ ( flag -- ) Execute if true
if ... else ... then     \ ( flag -- ) Branch on flag
begin ... again          \ Infinite loop
begin ... until          \ ( flag -- ) Loop until true
do ... loop              \ ( limit start -- ) Count loop, use 'i' for counter

\ Memory
@      ( addr -- value )      \ Fetch from address
!      ( value addr -- )      \ Store to address
c@     ( addr -- byte )       \ Fetch byte
c!     ( byte addr -- )       \ Store byte
```

### Code Generation Protocol

**Step 1: Analyze Requirements**
```
Input: Task description
Output: Formal specification
  - Stack effect signature
  - Input types/constraints
  - Output types/constraints
  - Algorithm approach
```

**Step 2: Generate Code**
```forth
\ ALWAYS include stack effect comment
: word-name ( inputs -- outputs )
  \ implementation
;
```

**Step 3: Verify**
- Stack depth must balance (same depth in/out accounting for signature)
- Type constraints respected (Hindley-Milner inference will catch errors)
- Edge cases handled (zero, negative, empty, etc.)

### Common Patterns

**Pattern 1: Duplicate and Transform**
```forth
: square ( n -- n² )
  dup * ;                    \ Duplicate, then multiply
```

**Pattern 2: Conditional**
```forth
: abs ( n -- |n| )
  dup 0 < if negate then ;   \ Negate if negative
```

**Pattern 3: Accumulator Loop**
```forth
: sum-1-to-n ( n -- sum )
  0 swap                     \ ( 0 n -- ) accumulator start
  1+ 1 do                    \ Loop from 1 to n
    i +                      \ Add counter to accumulator
  loop ;
```

**Pattern 4: Recursive**
```forth
: factorial ( n -- n! )
  dup 2 < if                 \ Base case
    drop 1
  else                       \ Recursive case
    dup 1- recurse *
  then ;
```

**Pattern 5: Two Values Transform**
```forth
: average ( a b -- avg )
  + 2 / ;                    \ Add, then divide by 2
```

### Anti-Patterns (AVOID)

❌ **Unnecessary Stack Juggling**
```forth
\ Bad: Over-complex stack manipulation
: bad-average ( a b -- avg )
  swap over + swap drop 2 / ;

\ Good: Simple composition
: good-average ( a b -- avg )
  + 2 / ;
```

❌ **Premature Optimization**
```forth
\ Bad: Trying to be clever
: bad-square ( n -- n² )
  dup + dup + ;              \ n+n+n+n (WRONG for n=3!)

\ Good: Correct and clear
: good-square ( n -- n² )
  dup * ;
```

❌ **Ignoring Stack Effects**
```forth
\ Bad: Unbalanced stack
: bad ( n -- )
  dup dup * ;                \ Signature says 1 out, actually 2!

\ Good: Accurate signature
: good ( n -- n n² )
  dup dup * ;
```

### Verification Checklist

Before outputting code, verify:
- [ ] Stack effect comment present and accurate
- [ ] Stack depth balanced (accounting for declared effect)
- [ ] Base cases handled (recursion, loops)
- [ ] Edge cases covered (0, negative, empty)
- [ ] No undefined words used
- [ ] Types compatible (no mixing addresses/integers incorrectly)

### Example Generation Workflow

**Task**: Generate code to compute sum of squares

**Analysis**:
```
Input: Two integers a, b
Output: a² + b²
Stack effect: ( a b -- sum )
Algorithm: Square each, sum results
Dependencies: Need 'square' helper
```

**Implementation**:
```forth
: square ( n -- n² )
  dup * ;

: sum-of-squares ( a b -- sum )
  square swap square + ;
```

**Verification**:
- `square`: ( n -- n² ) ✓ 1 in, 1 out, balanced
- `sum-of-squares`: ( a b -- sum ) ✓ 2 in, 1 out, balanced
- Test: `3 4 sum-of-squares` => 25 ✓

### Quick Reference: Stack Transformations

```
( a -- a a )           dup
( a b -- b )           nip
( a b -- a b a )       over
( a b -- b a )         swap
( a b c -- c a b )     -rot
( a b c -- b c a )     rot
( a b -- )             2drop
( a b c -- c )         nip nip
```

### Type Inference Tips

Fast Forth uses Hindley-Milner inference. The compiler infers:
- Numeric types (works for int/float/etc. polymorphically)
- Address types (pointers)
- Stack depth consistency

You don't specify types - the compiler verifies correctness.

### When Stuck

1. **Sketch stack states**: Write comments showing stack contents
   ```forth
   : mystery ( a b c -- result )
     +        \ ( a [b+c] -- )
     swap     \ ( [b+c] a -- )
     * ;      \ ( result -- )
   ```

2. **Build incrementally**: Define helper words, test each
   ```forth
   : helper1 ... ;   \ Test this
   : helper2 ... ;   \ Test this
   : main ... ;      \ Use helpers
   ```

3. **Use the REPL**: Fast Forth compiles in 50-100ms - test interactively

---

**End of Agent Context. Use the patterns above to generate efficient, correct Fast Forth code.**
