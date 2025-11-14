\ Sieve of Eratosthenes - Forth Implementation
\ Based on BENCHMARK_SUITE_SPECIFICATION.md
\ Target: < 200ms for Phase 1, < 100ms for Phase 2 (limit=8190)

\ Create a boolean array for sieve
: CREATE-SIEVE ( limit -- addr )
    1+ ALLOCATE THROW ;

\ Free the sieve array
: FREE-SIEVE ( addr -- )
    FREE THROW ;

\ Set all elements to TRUE (is_prime)
: INIT-SIEVE ( addr limit -- )
    0 DO
        TRUE OVER I + C!
    LOOP DROP ;

\ Mark number as composite
: MARK-COMPOSITE ( addr n -- )
    + FALSE SWAP C! ;

\ Check if number is prime
: IS-PRIME? ( addr n -- flag )
    + C@ ;

\ Main sieve algorithm
: SIEVE ( limit -- count )
    DUP CREATE-SIEVE >R     \ Create array, save address
    R@ OVER INIT-SIEVE       \ Initialize all to TRUE

    \ Mark 0 and 1 as not prime
    FALSE R@ C!
    FALSE R@ 1+ C!

    \ Mark composites
    DUP 0 DO
        I R@ IS-PRIME? IF
            I DUP * DUP ROT < IF
                \ Mark multiples starting from i*i
                DUP I + R@ OVER IS-PRIME? IF
                    BEGIN
                        DUP 3 PICK <
                    WHILE
                        FALSE OVER R@ + C!
                        OVER +
                    REPEAT
                THEN
                DROP DROP
            ELSE
                DROP
            THEN
        THEN
    LOOP

    \ Count primes
    0 SWAP 0 DO
        I R@ IS-PRIME? IF
            1+
        THEN
    LOOP

    R> FREE-SIEVE
;

\ Simpler sieve implementation (closer to spec)
: SIEVE-SIMPLE ( limit -- count )
    1+ DUP ALLOCATE THROW >R
    R@ OVER TRUE FILL
    0 SWAP 0
    DO
        I R@ + C@
        IF
            I DUP + 3 + DUP I +
            BEGIN
                DUP ROT <
            WHILE
                FALSE OVER R@ + C!
                OVER +
            REPEAT
            DROP DROP 1+
        THEN
    LOOP
    R> FREE THROW ;

\ Test and benchmark
: TEST-SIEVE
    CR ." Testing Sieve of Eratosthenes..." CR
    8190 SIEVE . ."  primes found (expected 1028)" CR
;

\ Benchmark with timing
: BENCHMARK-SIEVE ( limit iterations -- )
    CR ." Benchmarking Sieve(" OVER . ." ) for " DUP . ."  iterations..." CR
    UTIME >R            \ Start time
    0 DO
        DUP SIEVE DROP
    LOOP
    DROP
    UTIME R> -          \ Elapsed time in microseconds
    1000 /              \ Convert to milliseconds
    DUP . ."  ms total, "
    OVER / . ."  ms average" CR
;

\ Quick test
\ TEST-SIEVE

\ Standard benchmark
\ 8190 100 BENCHMARK-SIEVE
