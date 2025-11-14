\ math.fth - Mathematical operations demonstration

: SQUARE ( n -- n^2 )
  \ Computes the square of a number
  \ Example: 7 SQUARE . \ Prints 49
  DUP * ;

: CUBE ( n -- n^3 )
  \ Computes the cube of a number
  \ Example: 3 CUBE . \ Prints 27
  DUP DUP * * ;

: AVERAGE ( a b -- avg )
  \ Computes average of two numbers
  \ Example: 10 20 AVERAGE . \ Prints 15
  + 2 / ;

: ABS ( n -- |n| )
  \ Computes absolute value
  \ Example: -5 ABS . \ Prints 5
  DUP 0 < IF NEGATE THEN ;

\ Test the functions
7 SQUARE . CR
3 CUBE . CR
10 20 AVERAGE . CR
-5 ABS . CR
