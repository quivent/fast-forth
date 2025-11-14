\ Fast Forth Benchmark Suite
\ Tests performance of various operations

\ ============================================================================
\ UTILITY WORDS
\ ============================================================================

: BENCHMARK-START  UTIME ;
: BENCHMARK-END    UTIME SWAP - 1000 / ;
: REPORT  CR ." Result: " . ." iterations in " . ." ms" ;

\ ============================================================================
\ BENCHMARK 1: Stack Operations
\ ============================================================================

: STACK-OPS
    0 10000000 0 DO
        I DUP DUP DROP DROP +
    LOOP DROP ;

CR ." Running Stack Operations Benchmark..."
BENCHMARK-START
STACK-OPS
BENCHMARK-END
." Stack ops: " . ." ms"

\ ============================================================================
\ BENCHMARK 2: Arithmetic
\ ============================================================================

: ARITHMETIC
    0 1000000 0 DO
        I 2 * 3 + 5 / 7 MOD +
    LOOP DROP ;

CR ." Running Arithmetic Benchmark..."
BENCHMARK-START
ARITHMETIC
BENCHMARK-END
." Arithmetic: " . ." ms"

\ ============================================================================
\ BENCHMARK 3: Recursion (Fibonacci)
\ ============================================================================

: FIB  ( n -- fib[n] )
    DUP 2 < IF
        DROP 1
    ELSE
        DUP 1- RECURSE
        SWAP 2- RECURSE +
    THEN ;

CR ." Running Recursion Benchmark (Fib 20)..."
BENCHMARK-START
20 FIB
BENCHMARK-END
SWAP ." Fib(20) = " . ." in " . ." ms"

\ ============================================================================
\ BENCHMARK 4: Memory Operations
\ ============================================================================

: MEMORY-OPS
    HERE 1000 ALLOT
    DUP 100000 0 DO
        I OVER I + C!
    LOOP
    DROP ;

CR ." Running Memory Operations Benchmark..."
BENCHMARK-START
MEMORY-OPS
BENCHMARK-END
." Memory ops: " . ." ms"

\ ============================================================================
\ BENCHMARK 5: Word Calls
\ ============================================================================

: DUMMY ;

: WORD-CALLS
    1000000 0 DO
        DUMMY DUMMY DUMMY DUMMY
    LOOP ;

CR ." Running Word Call Benchmark..."
BENCHMARK-START
WORD-CALLS
BENCHMARK-END
." Word calls: " . ." ms"

\ ============================================================================
\ BENCHMARK 6: Logical Operations
\ ============================================================================

: LOGICAL-OPS
    0 1000000 0 DO
        I DUP AND DUP OR XOR INVERT +
    LOOP DROP ;

CR ." Running Logical Operations Benchmark..."
BENCHMARK-START
LOGICAL-OPS
BENCHMARK-END
." Logical ops: " . ." ms"

\ ============================================================================
\ BENCHMARK 7: Nested Loops
\ ============================================================================

: NESTED-LOOPS
    0
    100 0 DO
        1000 0 DO
            I J * +
        LOOP
    LOOP DROP ;

CR ." Running Nested Loops Benchmark..."
BENCHMARK-START
NESTED-LOOPS
BENCHMARK-END
." Nested loops: " . ." ms"

\ ============================================================================
\ BENCHMARK 8: String Operations
\ ============================================================================

: STRING-COPY  ( src dest len -- )
    0 DO
        OVER I + C@
        OVER I + C!
    LOOP 2DROP ;

: STRING-BENCH
    S" Hello, World! This is a test string." DROP
    DUP 100 ALLOT
    10000 0 DO
        2DUP 39 STRING-COPY
    LOOP
    2DROP ;

CR ." Running String Operations Benchmark..."
BENCHMARK-START
STRING-BENCH
BENCHMARK-END
." String ops: " . ." ms"

\ ============================================================================
\ BENCHMARK 9: Comparison Operations
\ ============================================================================

: COMPARISON-OPS
    0 1000000 0 DO
        I DUP < IF 1+ THEN
        I DUP > IF 1+ THEN
        I DUP = IF 1+ THEN
    LOOP DROP ;

CR ." Running Comparison Operations Benchmark..."
BENCHMARK-START
COMPARISON-OPS
BENCHMARK-END
." Comparison ops: " . ." ms"

\ ============================================================================
\ BENCHMARK 10: Real-world Algorithm (Sieve of Eratosthenes)
\ ============================================================================

: SIEVE  ( n -- count )
    HERE SWAP
    DUP 0 DO
        TRUE OVER I + C!
    LOOP
    0 SWAP
    DUP 2 DO
        DUP I + C@ IF
            I DUP * ROT DUP ROT DO
                FALSE OVER I + C!
            DUP +LOOP
            SWAP 1+ SWAP
        THEN
    LOOP
    DROP ;

CR ." Running Sieve of Eratosthenes (n=10000)..."
BENCHMARK-START
10000 SIEVE
BENCHMARK-END
SWAP ." Primes found: " . ." in " . ." ms"

CR
CR ." All benchmarks complete!"
CR
