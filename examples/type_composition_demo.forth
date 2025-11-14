\ Type Composition Examples for Stream 5

\ Example 1: Simple composition - square = dup *
: dup-example ( a -- a a ) dup ;
: mult-example ( a b -- c ) * ;

\ Example 2: Swap composition
: swap-example ( a b -- b a ) swap ;

\ Example 3: Complex composition
: over-add ( a b -- a b a+b ) over + ;

\ Example 4: Multiple stack operations
: rot-example ( a b c -- b c a ) rot ;

\ Example 5: Arithmetic sequence
: inc ( n -- n+1 ) 1 + ;
: double ( n -- 2n ) 2 * ;
: square ( n -- n² ) dup * ;
