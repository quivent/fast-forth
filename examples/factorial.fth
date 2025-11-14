\ factorial.fth - Recursive factorial implementation
\ Example: Calculate factorial of 5

: FACTORIAL ( n -- n! )
  \ Computes the factorial of a number recursively
  \ Example: 5 FACTORIAL . \ Prints 120
  DUP 1 <= IF DROP 1 ELSE
    DUP 1 - FACTORIAL *
  THEN ;

\ Test cases
5 FACTORIAL . CR
10 FACTORIAL . CR
