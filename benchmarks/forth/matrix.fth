\ Matrix Multiplication - Forth Implementation
\ Based on BENCHMARK_SUITE_SPECIFICATION.md
\ Target: < 400ms Phase 1, < 160ms Phase 2 (100x100)

100 CONSTANT N

\ Create a matrix (N x N)
: CREATE-MATRIX ( -- addr )
    N N * CELLS ALLOCATE THROW ;

\ Free a matrix
: FREE-MATRIX ( addr -- )
    FREE THROW ;

\ Get matrix element at (i, j)
: MATRIX@ ( addr i j -- value )
    SWAP N * + CELLS + @ ;

\ Set matrix element at (i, j)
: MATRIX! ( value addr i j -- )
    SWAP N * + CELLS + ! ;

\ Initialize matrix with sequential values
: INIT-MATRIX ( addr seed -- )
    >R
    N N * 0 DO
        I R@ + OVER I CELLS + !
    LOOP
    DROP R> DROP ;

\ Initialize matrix with random-like values
: INIT-MATRIX-RANDOM ( addr seed -- )
    >R
    N N * 0 DO
        I R@ + 997 MOD OVER I CELLS + !
    LOOP
    DROP R> DROP ;

\ Zero out a matrix
: ZERO-MATRIX ( addr -- )
    N N * CELLS 0 FILL ;

\ Matrix multiplication: C = A * B
: MATRIX-MULT ( a b c -- )
    >R >R >R
    N 0 DO
        N 0 DO
            0               \ Accumulator
            N 0 DO
                R@ I J 3 PICK MATRIX@
                R> 2 PICK >R J K MATRIX@
                * +
            LOOP
            R@ I J 2 PICK MATRIX!
        LOOP
    LOOP
    R> R> R> DROP DROP DROP ;

\ Simpler version for testing
: MATRIX-MULT-SIMPLE ( a b c -- )
    -ROT                    \ c a b
    N 0 DO
        N 0 DO
            0               \ Accumulator for c[i][j]
            N 0 DO
                \ Get a[i][k]
                2 PICK I K MATRIX@
                \ Get b[k][j]
                OVER K J MATRIX@
                * +
            LOOP
            \ Store in c[i][j]
            2 PICK I J ROT MATRIX!
        LOOP
    LOOP
    DROP DROP ;

\ Test matrix multiplication
: TEST-MATRIX
    CR ." Testing Matrix Multiplication..." CR
    CREATE-MATRIX DUP >R
    CREATE-MATRIX DUP >R
    CREATE-MATRIX DUP >R

    \ Initialize A and B
    R@ 2 PICK 42 INIT-MATRIX-RANDOM
    R> 2 PICK 43 INIT-MATRIX-RANDOM
    R>

    \ Multiply
    3DUP MATRIX-MULT-SIMPLE

    \ Show result
    ." Result[0][0] = " DUP 0 0 MATRIX@ . CR

    \ Clean up
    FREE-MATRIX
    FREE-MATRIX
    FREE-MATRIX
;

\ Benchmark matrix multiplication
: BENCHMARK-MATRIX ( iterations -- )
    CR ." Benchmarking Matrix Multiplication (" N . ." x" N . ." ) for " DUP . ."  iterations..." CR

    \ Create matrices
    CREATE-MATRIX DUP >R
    CREATE-MATRIX DUP >R
    CREATE-MATRIX DUP >R

    \ Initialize
    R@ 2 PICK 42 INIT-MATRIX-RANDOM
    R> 2 PICK 43 INIT-MATRIX-RANDOM
    R>

    \ Benchmark
    UTIME >R
    0 DO
        3DUP MATRIX-MULT-SIMPLE
    LOOP
    UTIME R> -

    \ Show result
    ." Result[0][0] = " DUP 0 0 MATRIX@ . CR

    \ Report time
    SWAP . ."  ms total, "
    OVER / . ."  ms average" CR

    \ Clean up
    FREE-MATRIX
    FREE-MATRIX
    FREE-MATRIX
;

\ Quick test
\ TEST-MATRIX

\ Standard benchmark
\ 10 BENCHMARK-MATRIX
