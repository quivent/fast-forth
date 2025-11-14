\ factorial.fth - Recursive factorial implementation
\ Demonstrates recursion and conditional logic

: FACTORIAL ( n -- n! )
  \ Computes factorial of n
  \ Example: 5 FACTORIAL . \ Prints 120
  \ Complexity: O(n)
  DUP 1 <= IF
    DROP 1
  ELSE
    DUP 1 - FACTORIAL *
  THEN ;

\ Test cases
." Testing FACTORIAL:" CR
." 0! = " 0 FACTORIAL . CR
." 1! = " 1 FACTORIAL . CR
." 5! = " 5 FACTORIAL . CR
." 10! = " 10 FACTORIAL . CR

\ Iterative version (more efficient)
: FACTORIAL-ITER ( n -- n! )
  \ Iterative factorial using a loop
  \ Generally faster than recursive version
  DUP 1 <= IF
    DROP 1
  ELSE
    1 SWAP 1+ 1 DO I * LOOP
  THEN ;

." " CR
." Testing FACTORIAL-ITER:" CR
." 5! = " 5 FACTORIAL-ITER . CR
." 10! = " 10 FACTORIAL-ITER . CR
