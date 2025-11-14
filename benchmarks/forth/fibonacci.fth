\ Fibonacci - Forth Implementation
\ Based on BENCHMARK_SUITE_SPECIFICATION.md
\ Recursive target: < 100ms Phase 1, < 35ms Phase 2 (n=35)

\ Recursive Fibonacci (exponential complexity)
: FIB-REC ( n -- fib[n] )
    DUP 2 < IF DROP 1 EXIT THEN
    DUP 1- RECURSE
    SWAP 2- RECURSE
    + ;

\ Iterative Fibonacci (linear complexity)
: FIB-ITER ( n -- fib[n] )
    DUP 2 < IF EXIT THEN
    0 1 ROT              \ a b n
    2 DO
        OVER +           \ a b (a+b)
        SWAP             \ (a+b) a
        DROP             \ (a+b) -- but we need to shift
    LOOP
    NIP ;

\ Better iterative version
: FIB-ITERATIVE ( n -- fib[n] )
    DUP 2 < IF EXIT THEN
    0 1                  \ a b
    ROT 2 - 0 DO
        OVER +           \ a b (a+b)
        SWAP             \ (a+b) b
    LOOP
    NIP ;

\ Test recursive Fibonacci
: TEST-FIB-REC
    CR ." Testing Recursive Fibonacci..." CR
    0 FIB-REC . ."  (expected 1)" CR
    1 FIB-REC . ."  (expected 1)" CR
    10 FIB-REC . ."  (expected 55)" CR
    20 FIB-REC . ."  (expected 6765)" CR
;

\ Test iterative Fibonacci
: TEST-FIB-ITER
    CR ." Testing Iterative Fibonacci..." CR
    0 FIB-ITERATIVE . ."  (expected 0)" CR
    1 FIB-ITERATIVE . ."  (expected 1)" CR
    10 FIB-ITERATIVE . ."  (expected 55)" CR
    20 FIB-ITERATIVE . ."  (expected 6765)" CR
    40 FIB-ITERATIVE . ."  (expected 102334155)" CR
;

\ Benchmark recursive Fibonacci
: BENCHMARK-FIB-REC ( n iterations -- )
    CR ." Benchmarking FIB-REC(" OVER . ." ) for " DUP . ."  iterations..." CR
    UTIME >R
    0 DO
        DUP FIB-REC DROP
    LOOP
    DUP FIB-REC
    UTIME R> -
    SWAP . ."  = fib(" SWAP . ." )" CR
    DUP . ."  ms total, "
    OVER / . ."  ms average" CR
;

\ Benchmark iterative Fibonacci
: BENCHMARK-FIB-ITER ( n iterations -- )
    CR ." Benchmarking FIB-ITERATIVE(" OVER . ." ) for " DUP . ."  iterations..." CR
    UTIME >R
    0 DO
        DUP FIB-ITERATIVE DROP
    LOOP
    DUP FIB-ITERATIVE
    UTIME R> -
    SWAP . ."  = fib(" SWAP . ." )" CR
    DUP . ."  ms total, "
    OVER / . ."  ms average" CR
;

\ Quick tests
\ TEST-FIB-REC
\ TEST-FIB-ITER

\ Standard benchmarks
\ 35 10 BENCHMARK-FIB-REC
\ 40 1000 BENCHMARK-FIB-ITER
