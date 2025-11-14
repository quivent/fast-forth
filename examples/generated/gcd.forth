\ Generated from specification: gcd
\ Calculates the greatest common divisor using Euclid's algorithm

\ GENERATED METADATA
\   AUTHOR: FastForth Team
\   VERSION: 1.0.0
\   CREATED: 2025-01-14T00:00:00Z
\   PATTERN: TAIL_RECURSIVE_008
\   TIME_COMPLEXITY: O(log min(a,b))
\   SPACE_COMPLEXITY: O(1)

\ Properties:
\   gcd(a, 0) = a
\   gcd(a, b) = gcd(b, a mod b)
\   gcd(a, b) = gcd(b, a)
\   gcd(a, a) = a

: gcd ( a b -- gcd(a,b) )  \ Calculates the greatest common divisor using Euclid's algorithm
  begin dup while swap over mod repeat drop
;

\ Test harness for gcd
\ Run these tests to verify correctness

\ Test 1: Same numbers
T{ 12 12 gcd -> 12 }T
\ Test 2: Coprime numbers
T{ 7 13 gcd -> 1 }T
\ Test 3: Common factor
T{ 48 18 gcd -> 6 }T
\ Test 4: One divides other
T{ 100 10 gcd -> 10 }T
\ Test 5: Large numbers
T{ 1071 462 gcd -> 21 }T
