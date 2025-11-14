\ Generated from specification: fibonacci
\ Calculates the nth Fibonacci number

\ GENERATED METADATA
\   AUTHOR: FastForth Team
\   VERSION: 1.0.0
\   CREATED: 2025-01-14T00:00:00Z
\   PATTERN: ACCUMULATOR_LOOP_003
\   TIME_COMPLEXITY: O(n)
\   SPACE_COMPLEXITY: O(1)

\ Properties:
\   fibonacci(0) = 0
\   fibonacci(1) = 1
\   fibonacci(n) = fibonacci(n-1) + fibonacci(n-2) for n > 1

: fibonacci ( n -- fib(n) )  \ Calculates the nth Fibonacci number
  0 swap 1+ 1 do i + loop
;

\ Test harness for fibonacci
\ Run these tests to verify correctness

\ Test 1: Base case: fib(0)
T{ 0 fibonacci -> 0 }T
\ Test 2: Base case: fib(1)
T{ 1 fibonacci -> 1 }T
\ Test 3: fib(2)
T{ 2 fibonacci -> 1 }T
\ Test 4: fib(7)
T{ 7 fibonacci -> 13 }T
\ Test 5: fib(10)
T{ 10 fibonacci -> 55 }T
\ Test 6: fib(15)
T{ 15 fibonacci -> 610 }T
