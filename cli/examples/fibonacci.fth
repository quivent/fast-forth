\ fibonacci.fth - Fibonacci sequence implementations
\ Demonstrates multiple approaches: recursive, iterative, and optimized

: FIB-RECURSIVE ( n -- fib[n] )
  \ Recursive Fibonacci (slow for large n)
  \ Example: 10 FIB-RECURSIVE . \ Prints 55
  \ Complexity: O(2^n) - exponential!
  DUP 2 < IF
    \ Base cases: fib(0) = 0, fib(1) = 1
    EXIT
  THEN
  \ Recursive case: fib(n) = fib(n-1) + fib(n-2)
  DUP 1 - FIB-RECURSIVE
  SWAP 2 - FIB-RECURSIVE
  + ;

: FIB-ITERATIVE ( n -- fib[n] )
  \ Iterative Fibonacci (much faster)
  \ Complexity: O(n) - linear
  DUP 2 < IF EXIT THEN
  0 1 ROT 1 DO
    OVER + SWAP
  LOOP DROP ;

: FIB-SEQUENCE ( n -- )
  \ Print Fibonacci sequence from 0 to n
  DUP 1+ 0 DO
    I FIB-ITERATIVE .
  LOOP CR ;

\ Test recursive version (slow!)
." Recursive Fibonacci:" CR
." fib(0) = " 0 FIB-RECURSIVE . CR
." fib(1) = " 1 FIB-RECURSIVE . CR
." fib(5) = " 5 FIB-RECURSIVE . CR
." fib(10) = " 10 FIB-RECURSIVE . CR
." " CR

\ Test iterative version (fast!)
." Iterative Fibonacci:" CR
." fib(10) = " 10 FIB-ITERATIVE . CR
." fib(20) = " 20 FIB-ITERATIVE . CR
." fib(30) = " 30 FIB-ITERATIVE . CR
." " CR

\ Print sequence
." Fibonacci sequence (0-20):" CR
20 FIB-SEQUENCE

\ Matrix-based O(log n) version for advanced users
." " CR
." Note: Matrix-based O(log n) version coming soon!" CR
