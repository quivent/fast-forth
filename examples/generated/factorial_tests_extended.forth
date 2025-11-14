\ Auto-generated tests for factorial
\ Generated 15 test cases

\ Base Cases
\   Base case: factorial(0) = 1
T{ 0 factorial -> 1 }T
\   Base case: factorial(1) = 1
T{ 1 factorial -> 1 }T

\ Edge Cases
\   Edge case: input = 2
T{ 2 factorial -> 2 }T
\   Edge case: input = 10
T{ 10 factorial -> 3628800 }T

\ Boundary Tests
\   Boundary test: n at constraint boundary
T{ 0 factorial -> 1 }T
\   Boundary test: n at constraint boundary
T{ 1 factorial -> 1 }T

\ Property-Based Tests
\   Property test 1
T{ 3 factorial -> 6 }T
\   Property test 2
T{ 4 factorial -> 24 }T
\   Property test 3
T{ 5 factorial -> 120 }T
\   Property test 4
T{ 6 factorial -> 720 }T
\   Property test 5
T{ 7 factorial -> 5040 }T

