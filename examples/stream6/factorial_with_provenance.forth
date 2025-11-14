\ GENERATED_BY: claude-sonnet-4
\ PATTERN_ID: RECURSIVE_004
\ TIMESTAMP: 2025-01-15T10:23:45Z
\ VERIFIED: stack_balanced=true, tests_passed=3/3, type_checked=true, compiled=true
\ SPEC_HASH: a3f7b2c9d1e4
\ OPTIMIZATION_LEVEL: Aggressive
\ PERFORMANCE_TARGET: 0.9
: factorial ( n -- n! )
  dup 2 < if drop 1 else dup 1- recurse * then ;

\ GENERATED_BY: claude-sonnet-4
\ PATTERN_ID: SIMPLE_001
\ TIMESTAMP: 2025-01-15T10:25:12Z
\ VERIFIED: stack_balanced=true, tests_passed=5/5, type_checked=true, compiled=true
\ SPEC_HASH: b4e8c3a2f5d6
: square ( n -- n² )
  dup * ;

\ GENERATED_BY: claude-sonnet-4
\ PATTERN_ID: ACCUMULATOR_LOOP_003
\ TIMESTAMP: 2025-01-15T10:27:33Z
\ VERIFIED: stack_balanced=true, tests_passed=4/4, type_checked=true, compiled=true
: sum-1-to-n ( n -- sum )
  0 swap 1+ 1 do i + loop ;
