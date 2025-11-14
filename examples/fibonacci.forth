\ Fibonacci sequence calculator
\ Calculate nth Fibonacci number

: fib ( n -- fib[n] )
  dup 2 <
  if
    drop 1
  else
    dup 1- recurse
    swap 2 - recurse
    +
  then
;

\ Calculate fib(10)
10 fib .
