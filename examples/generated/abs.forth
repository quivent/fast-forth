\ Generated from specification: abs
\ Calculates the absolute value of a number

\ GENERATED METADATA
\   AUTHOR: FastForth Team
\   VERSION: 1.0.0
\   CREATED: 2025-01-14T00:00:00Z
\   PATTERN: CONDITIONAL_NEGATE_002
\   TIME_COMPLEXITY: O(1)
\   SPACE_COMPLEXITY: O(1)

\ Properties:
\   abs(n) = n if n >= 0
\   abs(n) = -n if n < 0
\   abs(0) = 0
\   abs(-n) = abs(n)

: abs ( n -- |n| )  \ Calculates the absolute value of a number
  dup 0 < if negate then
;

\ Test harness for abs
\ Run these tests to verify correctness

\ Test 1: Zero
T{ 0 abs -> 0 }T
\ Test 2: Positive number
T{ 42 abs -> 42 }T
\ Test 3: Negative number
T{ -42 abs -> 42 }T
\ Test 4: Negative one
T{ -1 abs -> 1 }T
\ Test 5: Large negative
T{ -1000 abs -> 1000 }T
