\ Generated from specification: square
\ Calculates the square of a number

\ GENERATED METADATA
\   AUTHOR: FastForth Team
\   VERSION: 1.0.0
\   CREATED: 2025-01-14T00:00:00Z
\   PATTERN: DUP_TRANSFORM_001
\   TIME_COMPLEXITY: O(1)
\   SPACE_COMPLEXITY: O(1)

\ Properties:
\   square(n) = n * n
\   square(-n) = square(n)
\   square(0) = 0
\   square(1) = 1

: square ( n -- n² )  \ Calculates the square of a number
  dup *
;

\ Test harness for square
\ Run these tests to verify correctness

\ Test 1: Zero squared
T{ 0 square -> 0 }T
\ Test 2: One squared
T{ 1 square -> 1 }T
\ Test 3: Positive number
T{ 7 square -> 49 }T
\ Test 4: Negative number
T{ -5 square -> 25 }T
\ Test 5: Large number
T{ 100 square -> 10000 }T
