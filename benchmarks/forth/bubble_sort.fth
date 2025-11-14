\ Bubble Sort - Forth Implementation
\ Based on BENCHMARK_SUITE_SPECIFICATION.md
\ Target: < 150ms Phase 1, < 75ms Phase 2 (1000 elements)

\ Swap two array elements
: ARRAY-SWAP ( addr i j -- )
    CELLS OVER + >R         \ addr i, R: addr[j]
    CELLS + DUP @           \ addr[i] val[i]
    R@ @ SWAP               \ addr[i] val[j] val[i]
    ROT !                   \ val[j], store val[j] in addr[i]
    R> ! ;                  \ store val[i] in addr[j]

\ Get array element
: ARRAY@ ( addr i -- value )
    CELLS + @ ;

\ Set array element
: ARRAY! ( value addr i -- )
    CELLS + ! ;

\ Bubble sort implementation
: BUBBLE-SORT ( addr len -- )
    DUP 0 DO                \ Outer loop
        DUP I - 0 DO        \ Inner loop
            OVER I CELLS +  \ Get addr[i]
            DUP @ OVER CELL+ @ > IF
                \ Swap if arr[i] > arr[i+1]
                DUP @ OVER CELL+ @
                OVER ! SWAP CELL+ !
            ELSE
                DROP
            THEN
        LOOP
    LOOP
    2DROP ;

\ Initialize array with pseudo-random values
: INIT-ARRAY ( addr len seed -- )
    >R
    0 DO
        I R@ + 997 MOD
        OVER I CELLS + !
    LOOP
    DROP R> DROP ;

\ Check if array is sorted
: IS-SORTED? ( addr len -- flag )
    1- 0 DO
        DUP I CELLS + @
        OVER I 1+ CELLS + @
        > IF
            DROP FALSE EXIT
        THEN
    LOOP
    DROP TRUE ;

\ Print first N elements
: PRINT-ARRAY ( addr len -- )
    0 DO
        DUP I CELLS + @ .
    LOOP
    DROP ;

\ Test bubble sort
: TEST-BUBBLE
    CR ." Testing Bubble Sort..." CR

    \ Create array
    1000 CELLS ALLOCATE THROW DUP >R

    \ Initialize with random values
    R@ 1000 42 INIT-ARRAY

    \ Sort
    R@ 1000 BUBBLE-SORT

    \ Check if sorted
    ." Sorted: " R@ 1000 IS-SORTED? IF ." YES" ELSE ." NO" THEN CR

    \ Print first 5 elements
    ." First 5 elements: " R@ 5 PRINT-ARRAY CR

    \ Clean up
    R> FREE THROW
;

\ Benchmark bubble sort
: BENCHMARK-BUBBLE ( len iterations -- )
    CR ." Benchmarking Bubble Sort (len=" OVER . ." ) for " DUP . ."  iterations..." CR

    \ Create arrays
    OVER CELLS ALLOCATE THROW DUP >R      \ Working array
    OVER CELLS ALLOCATE THROW DUP >R      \ Backup array

    \ Initialize backup
    R@ 2 PICK 42 INIT-ARRAY

    \ Benchmark
    UTIME >R
    0 DO
        \ Copy backup to working array
        R> R> 2DUP >R >R
        2 PICK CELLS CMOVE

        \ Sort
        R@ 2 PICK BUBBLE-SORT
    LOOP
    UTIME R> -

    \ Verify
    ." Sorted: " R@ OVER IS-SORTED? IF ." YES" ELSE ." NO" THEN CR
    ." First 5 elements: " R@ 5 PRINT-ARRAY CR

    \ Report time
    SWAP . ."  ms total, "
    OVER / . ."  ms average" CR

    \ Clean up
    R> FREE THROW
    R> FREE THROW
    DROP
;

\ Quick test
\ TEST-BUBBLE

\ Standard benchmark
\ 1000 10 BENCHMARK-BUBBLE
