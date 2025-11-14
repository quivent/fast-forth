\ Generated from specification: factorial
\ Calculates the factorial of a non-negative integer

\ GENERATED METADATA
\   AUTHOR: FastForth Team
\   VERSION: 1.0.0
\   CREATED: 2025-01-14T00:00:00Z
\   PATTERN: RECURSIVE_004
\   TIME_COMPLEXITY: O(n)
\   SPACE_COMPLEXITY: O(n)

\ Properties:
\   factorial(0) = 1
\   factorial(1) = 1
\   factorial(n) = n * factorial(n-1) for n > 1

: factorial ( n -- n! )  \ Calculates the factorial of a non-negative integer
  dup 2 < if drop 1 else dup 1- recurse * then
;

\ Test harness for factorial
\ Run these tests to verify correctness

\ Test 1: Base case: factorial of 0
T{ 0 factorial -> 1 }T
\ Test 2: Base case: factorial of 1
T{ 1 factorial -> 1 }T
\ Test 3: Small value: factorial of 5
T{ 5 factorial -> 120 }T
\ Test 4: Medium value: factorial of 10
T{ 10 factorial -> 3628800 }T
\ Test 5: Edge case: factorial of 2
T{ 2 factorial -> 2 }T
