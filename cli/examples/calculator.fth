\ calculator.fth - Simple stack-based calculator
\ Demonstrates practical use of stack operations

\ Basic arithmetic words (already built-in, shown for reference)
\ : ADD + ;
\ : SUBTRACT - ;
\ : MULTIPLY * ;
\ : DIVIDE / ;

: SQUARE ( n -- n² )
  \ Square a number
  DUP * ;

: CUBE ( n -- n³ )
  \ Cube a number
  DUP DUP * * ;

: POWER ( base exp -- base^exp )
  \ Raise base to exponent
  \ Example: 2 8 POWER . \ Prints 256
  1 SWAP 0 DO
    OVER *
  LOOP SWAP DROP ;

: ABS ( n -- |n| )
  \ Absolute value
  DUP 0< IF NEGATE THEN ;

: MIN ( a b -- min )
  \ Minimum of two numbers
  2DUP < IF DROP ELSE SWAP DROP THEN ;

: MAX ( a b -- max )
  \ Maximum of two numbers
  2DUP > IF DROP ELSE SWAP DROP THEN ;

: AVERAGE ( a b -- avg )
  \ Average of two numbers
  + 2 / ;

: CLAMP ( n min max -- n' )
  \ Clamp n to range [min, max]
  \ Example: 15 0 10 CLAMP . \ Prints 10
  ROT
  ( min max n )
  OVER MAX   \ n = max(n, min)
  SWAP MIN ; \ n = min(n, max)

\ Test calculator functions
." Calculator Examples:" CR
." " CR

." Basic Arithmetic:" CR
." 5 + 3 = " 5 3 + . CR
." 10 - 4 = " 10 4 - . CR
." 7 * 6 = " 7 6 * . CR
." 20 / 4 = " 20 4 / . CR
." " CR

." Powers:" CR
." 5² = " 5 SQUARE . CR
." 3³ = " 3 CUBE . CR
." 2⁸ = " 2 8 POWER . CR
." " CR

." Utility:" CR
." |−7| = " -7 ABS . CR
." min(5, 3) = " 5 3 MIN . CR
." max(5, 3) = " 5 3 MAX . CR
." avg(10, 20) = " 10 20 AVERAGE . CR
." clamp(15, 0, 10) = " 15 0 10 CLAMP . CR
." " CR

\ Complex calculation example
." Complex Calculation:" CR
." ((5 + 3) * 2)² = "
5 3 + 2 * SQUARE . CR
