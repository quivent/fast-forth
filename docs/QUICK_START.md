# Fast Forth Quick Start Guide
**Stream 6: Get Started in 5 Minutes**

## Installation

### Prerequisites

- C compiler (GCC or Clang)
- Make
- Standard C library with math support

### Build from Source

```bash
cd /Users/joshkornreich/Documents/Projects/FastForth
make
make test
```

### Install System-Wide

```bash
sudo make install
```

This installs:
- `/usr/local/lib/libforth.a` - Runtime library
- `/usr/local/include/forth/` - Header files
- `/usr/local/bin/forth` - Standalone REPL

## Quick Examples

### 1. Interactive REPL

```bash
$ forth

Fast Forth Runtime v1.0
  60 primitives loaded
  Dictionary: 1048576 bytes
  Stack: 256 cells

Type 'WORDS' to see available words

ok> 2 3 + .
5  ok>

ok> : SQUARE DUP * ;
ok> 7 SQUARE .
49  ok>

ok> : FACTORIAL  DUP 2 < IF DROP 1 ELSE DUP 1- FACTORIAL * THEN ;
ok> 5 FACTORIAL .
120  ok>
```

### 2. Run Forth File

Create `hello.forth`:

```forth
\ Hello World in Forth
: HELLO  CR ." Hello, Fast Forth!" CR ;

HELLO
```

Run it:

```bash
$ forth hello.forth
Hello, Fast Forth!
```

### 3. Embed in C Program

Create `embed_example.c`:

```c
#include <forth/forth_runtime.h>
#include <stdio.h>

extern int forth_bootstrap(forth_vm_t*);
extern int forth_interpret(forth_vm_t*, const char*);

int main(void) {
    // Create Forth VM
    forth_vm_t *vm = forth_create();
    forth_bootstrap(vm);

    // Define and use Forth words
    forth_interpret(vm, ": DOUBLE DUP + ;");
    forth_interpret(vm, ": QUAD DOUBLE DOUBLE ;");

    // Push value and compute
    push(vm, 5);
    forth_interpret(vm, "QUAD");

    // Get result
    printf("Result: %ld\n", pop(vm));

    forth_destroy(vm);
    return 0;
}
```

Compile and run:

```bash
$ gcc embed_example.c -L/usr/local/lib -lforth -ldl -lm -o embed_example
$ ./embed_example
Result: 20
```

### 4. FFI Example (Call C from Forth)

Create `ffi_demo.c`:

```c
#include <forth/forth_runtime.h>
#include <stdio.h>
#include <math.h>

extern int forth_bootstrap(forth_vm_t*);
extern int forth_interpret(forth_vm_t*, const char*);
extern void forth_ffi_init_stdlib(forth_vm_t*);

// Custom C function
cell_t celsius_to_fahrenheit(cell_t celsius) {
    return celsius * 9 / 5 + 32;
}

int main(void) {
    forth_vm_t *vm = forth_create();
    forth_bootstrap(vm);
    forth_ffi_init_stdlib(vm);

    // Call C function from Forth
    push(vm, (cell_t)celsius_to_fahrenheit);
    push(vm, 25);  // 25°C
    push(vm, 1);   // 1 argument

    extern void forth_ffi_call_c(forth_vm_t*);
    forth_ffi_call_c(vm);

    printf("25°C = %ld°F\n", pop(vm));

    forth_destroy(vm);
    return 0;
}
```

Compile and run:

```bash
$ gcc ffi_demo.c -L/usr/local/lib -lforth -ldl -lm -o ffi_demo
$ ./ffi_demo
25°C = 77°F
```

## Common Operations

### Stack Manipulation

```forth
\ DUP - Duplicate top
5 DUP .S        \ <2> 5 5

\ SWAP - Swap top two
1 2 SWAP .S     \ <2> 2 1

\ OVER - Copy second to top
1 2 OVER .S     \ <3> 1 2 1

\ ROT - Rotate three
1 2 3 ROT .S    \ <3> 2 3 1
```

### Arithmetic

```forth
\ Basic operations
10 3 + .        \ 13
10 3 - .        \ 7
10 3 * .        \ 30
10 3 / .        \ 3
10 3 MOD .      \ 1

\ Combined
10 3 /MOD .S    \ <2> 3 1  (quotient, remainder)

\ Advanced
-5 ABS .        \ 5
3 5 MIN .       \ 3
3 5 MAX .       \ 5
```

### Defining Words

```forth
\ Constants
42 CONSTANT ANSWER
ANSWER .        \ 42

\ Variables
VARIABLE COUNTER
10 COUNTER !
COUNTER @ .     \ 10

\ Custom words
: TRIPLE  3 * ;
7 TRIPLE .      \ 21

\ Conditional
: ABS-DIFF  ( a b -- |a-b| )
    - ABS ;

5 10 ABS-DIFF . \ 5
```

### Control Flow

```forth
\ IF-THEN
: POSITIVE?  ( n -- )
    0 > IF
        ." Positive"
    ELSE
        ." Not positive"
    THEN ;

\ Loops
: COUNT-TO-10  ( -- )
    11 1 DO
        I .
    LOOP ;

\ BEGIN-UNTIL
: COUNTDOWN  ( n -- )
    BEGIN
        DUP .
        1-
        DUP 0=
    UNTIL
    DROP ;
```

## Core Word Reference

### Stack Operations

| Word | Effect | Description |
|------|--------|-------------|
| DUP | ( a -- a a ) | Duplicate |
| DROP | ( a -- ) | Remove |
| SWAP | ( a b -- b a ) | Swap |
| OVER | ( a b -- a b a ) | Copy second |
| ROT | ( a b c -- b c a ) | Rotate |

### Arithmetic

| Word | Effect | Description |
|------|--------|-------------|
| + | ( a b -- a+b ) | Add |
| - | ( a b -- a-b ) | Subtract |
| * | ( a b -- a*b ) | Multiply |
| / | ( a b -- a/b ) | Divide |
| MOD | ( a b -- a%b ) | Modulo |

### Comparison

| Word | Effect | Description |
|------|--------|-------------|
| = | ( a b -- flag ) | Equal |
| < | ( a b -- flag ) | Less than |
| > | ( a b -- flag ) | Greater than |
| 0= | ( n -- flag ) | Zero? |

### Memory

| Word | Effect | Description |
|------|--------|-------------|
| @ | ( addr -- n ) | Fetch |
| ! | ( n addr -- ) | Store |
| C@ | ( addr -- byte ) | Fetch byte |
| C! | ( byte addr -- ) | Store byte |

### I/O

| Word | Effect | Description |
|------|--------|-------------|
| . | ( n -- ) | Print number |
| EMIT | ( char -- ) | Print char |
| CR | ( -- ) | Newline |
| .S | ( -- ) | Show stack |

## Performance Tips

### 1. Use Primitives

```forth
\ Fast - uses primitive
: FAST-SUM  + + + + ;

\ Slow - loop overhead
: SLOW-SUM  0 5 0 DO + LOOP ;
```

### 2. Minimize Stack Shuffling

```forth
\ Good - minimal shuffling
: AVERAGE  ( a b -- avg )
    + 2 / ;

\ Bad - excessive shuffling
: AVERAGE-BAD  ( a b -- avg )
    SWAP OVER + SWAP DROP 2 / ;
```

### 3. Inline Short Words

```forth
\ Consider inlining
: DOUBLE  2 * ;
: QUAD    DOUBLE DOUBLE ;  \ Inline DOUBLE calls
```

### 4. Use Locals Sparingly

```forth
\ Stack-based (faster)
: PYTHAGORAS  ( a b -- c )
    DUP * SWAP DUP * + ;

\ Locals (slower but clearer)
: PYTHAGORAS  { a b -- c }
    a a * b b * + ;
```

## Debugging

### Show Stack

```forth
ok> 1 2 3 .S
<3> 1 2 3  ok>
```

### See Word Definition

```forth
ok> SEE SQUARE
: SQUARE
  DUP
  *
; ok>
```

### Memory Dump

```forth
ok> HERE DUMP
00001000: 48 65 6C 6C 6F ...
```

### Error Handling

```forth
: SAFE-DIVIDE  ( a b -- a/b )
    DUP 0= IF
        ." Error: Division by zero!" CR
        DROP DROP 0
    ELSE
        /
    THEN ;
```

## Common Patterns

### Array Processing

```forth
\ Create array
CREATE MYARRAY 10 CELLS ALLOT

\ Set values
: INIT-ARRAY  ( -- )
    10 0 DO
        I MYARRAY I CELLS + !
    LOOP ;

\ Sum array
: SUM-ARRAY  ( -- sum )
    0 10 0 DO
        MYARRAY I CELLS + @ +
    LOOP ;
```

### String Operations

```forth
\ Create string
CREATE MSG 32 ALLOT

\ Copy string
: COPY-STRING  ( src len dest -- )
    SWAP CMOVE ;

\ Print string
S" Hello, World!" TYPE CR
```

### Data Structures

```forth
\ Linked list node
: NODE  ( data link -- node )
    CREATE , , ;

\ Stack structure
VARIABLE STACK-TOP

: PUSH  ( n -- )
    STACK-TOP @ , STACK-TOP ! ;

: POP  ( -- n )
    STACK-TOP @ @ ;
```

## Next Steps

1. **Read the full reference**: `docs/RUNTIME_REFERENCE.md`
2. **Explore examples**: `examples/`
3. **Run benchmarks**: `make benchmark`
4. **Study ANS standard**: https://forth-standard.org/
5. **Join community**: comp.lang.forth newsgroup

## Troubleshooting

### Stack Underflow

```forth
ok> +
Error -1: Stack underflow
```

**Solution**: Ensure enough values on stack before operation.

### Undefined Word

```forth
ok> UNDEFINED-WORD
Error -5: Undefined word: UNDEFINED-WORD
```

**Solution**: Check spelling or define the word first.

### Memory Issues

```forth
ok> 999999999 ALLOT
Error -4: Dictionary overflow
```

**Solution**: Reduce allocation or increase dictionary size.

## Resources

- **Documentation**: `/docs`
- **Examples**: `/examples`
- **Tests**: `/tests`
- **Source**: `/runtime`
- **ANS Forth Standard**: https://forth-standard.org/
- **Forth Tutorial**: https://www.forth.com/starting-forth/

---

**Happy Forth coding!**
