\ Auto-generated tests for square
\ Generated 13 test cases

\ Edge Cases
\   Edge case: input = 0
T{ 0 square -> 0 }T
\   Edge case: input = 1
T{ 1 square -> 1 }T
\   Edge case: input = -1
T{ -1 square -> 1 }T
\   Edge case: input = 2
T{ 2 square -> 4 }T
\   Edge case: input = 10
T{ 10 square -> 100 }T
\   Edge case: input = 100
T{ 100 square -> 10000 }T

\ Property-Based Tests
\   Property test 1
T{ 3 square -> 9 }T
\   Property test 2
T{ 8 square -> 64 }T
\   Property test 3
T{ 13 square -> 169 }T
\   Property test 4
T{ 18 square -> 324 }T
\   Property test 5
T{ 23 square -> 529 }T

