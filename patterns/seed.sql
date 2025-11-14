-- Seed data for Pattern Library Database
-- 20+ canonical Fast Forth patterns

-- DUP_TRANSFORM patterns
INSERT INTO patterns (id, category, stack_effect, code_template, performance_class, description, created_at, updated_at) VALUES
('DUP_TRANSFORM_001', 'dup_transform', '( n -- n² )', ': NAME ( n -- n² )\n  dup * ;', 'O(1)', 'Square a number using dup and multiply', datetime('now'), datetime('now')),
('DUP_TRANSFORM_002', 'dup_transform', '( n -- n n+1 )', ': NAME ( n -- n n+1 )\n  dup 1+ ;', 'O(1)', 'Duplicate and increment', datetime('now'), datetime('now'));

INSERT INTO pattern_tags VALUES
('DUP_TRANSFORM_001', 'arithmetic'),
('DUP_TRANSFORM_001', 'dup'),
('DUP_TRANSFORM_001', 'transform'),
('DUP_TRANSFORM_002', 'arithmetic'),
('DUP_TRANSFORM_002', 'dup');

-- CONDITIONAL patterns
INSERT INTO patterns (id, category, stack_effect, code_template, performance_class, description, created_at, updated_at) VALUES
('CONDITIONAL_001', 'conditional', '( n -- |n| )', ': NAME ( n -- |n| )\n  dup 0 < if negate then ;', 'O(1)', 'Absolute value using conditional', datetime('now'), datetime('now')),
('CONDITIONAL_002', 'conditional', '( a b -- max )', ': NAME ( a b -- max )\n  2dup < if swap then drop ;', 'O(1)', 'Maximum of two numbers', datetime('now'), datetime('now')),
('CONDITIONAL_003', 'conditional', '( a b -- min )', ': NAME ( a b -- min )\n  2dup > if swap then drop ;', 'O(1)', 'Minimum of two numbers', datetime('now'), datetime('now'));

INSERT INTO pattern_tags VALUES
('CONDITIONAL_001', 'arithmetic'),
('CONDITIONAL_001', 'conditional'),
('CONDITIONAL_001', 'abs'),
('CONDITIONAL_002', 'arithmetic'),
('CONDITIONAL_002', 'conditional'),
('CONDITIONAL_002', 'max'),
('CONDITIONAL_003', 'arithmetic'),
('CONDITIONAL_003', 'conditional'),
('CONDITIONAL_003', 'min');

-- ACCUMULATOR_LOOP patterns
INSERT INTO patterns (id, category, stack_effect, code_template, performance_class, description, created_at, updated_at) VALUES
('ACCUMULATOR_LOOP_001', 'accumulator_loop', '( n -- sum )', ': NAME ( n -- sum )\n  0 swap 1+ 1 do i + loop ;', 'O(n)', 'Sum from 1 to n', datetime('now'), datetime('now')),
('ACCUMULATOR_LOOP_002', 'accumulator_loop', '( n -- n! )', ': NAME ( n -- n! )\n  1 swap 1+ 1 do i * loop ;', 'O(n)', 'Factorial using loop', datetime('now'), datetime('now')),
('ACCUMULATOR_LOOP_003', 'accumulator_loop', '( n -- product )', ': NAME ( n -- product )\n  1 swap 0 do 2 * loop ;', 'O(n)', 'Power of 2 using loop', datetime('now'), datetime('now'));

INSERT INTO pattern_tags VALUES
('ACCUMULATOR_LOOP_001', 'loop'),
('ACCUMULATOR_LOOP_001', 'accumulator'),
('ACCUMULATOR_LOOP_001', 'sum'),
('ACCUMULATOR_LOOP_002', 'loop'),
('ACCUMULATOR_LOOP_002', 'accumulator'),
('ACCUMULATOR_LOOP_002', 'factorial'),
('ACCUMULATOR_LOOP_003', 'loop'),
('ACCUMULATOR_LOOP_003', 'accumulator'),
('ACCUMULATOR_LOOP_003', 'power');

-- RECURSIVE patterns
INSERT INTO patterns (id, category, stack_effect, code_template, performance_class, description, created_at, updated_at) VALUES
('RECURSIVE_001', 'recursive', '( n -- n! )', ': NAME ( n -- n! )\n  dup 2 < if drop 1 else dup 1- recurse * then ;', 'O(n)', 'Factorial using recursion', datetime('now'), datetime('now')),
('RECURSIVE_002', 'recursive', '( n -- fib(n) )', ': NAME ( n -- fib )\n  dup 2 < if else dup 1- recurse swap 2 - recurse + then ;', 'O(2^n)', 'Fibonacci using recursion (inefficient)', datetime('now'), datetime('now')),
('RECURSIVE_003', 'recursive', '( n -- sum )', ': NAME ( n -- sum )\n  dup 0 <= if drop 0 else dup 1- recurse + then ;', 'O(n)', 'Sum from 1 to n using recursion', datetime('now'), datetime('now'));

INSERT INTO pattern_tags VALUES
('RECURSIVE_001', 'recursion'),
('RECURSIVE_001', 'factorial'),
('RECURSIVE_001', 'base-case'),
('RECURSIVE_002', 'recursion'),
('RECURSIVE_002', 'fibonacci'),
('RECURSIVE_003', 'recursion'),
('RECURSIVE_003', 'sum');

-- TAIL_RECURSIVE patterns
INSERT INTO patterns (id, category, stack_effect, code_template, performance_class, description, created_at, updated_at) VALUES
('TAIL_RECURSIVE_001', 'tail_recursive', '( n acc -- n! )', ': NAME ( n acc -- result )\n  over 1 <= if nip else over * swap 1- swap recurse then ;', 'O(n)', 'Tail-recursive factorial', datetime('now'), datetime('now')),
('TAIL_RECURSIVE_002', 'tail_recursive', '( n a b -- fib(n) )', ': NAME ( n a b -- fib )\n  rot dup 0 <= if drop drop else 1- rot rot tuck + recurse then ;', 'O(n)', 'Tail-recursive fibonacci', datetime('now'), datetime('now'));

INSERT INTO pattern_tags VALUES
('TAIL_RECURSIVE_001', 'recursion'),
('TAIL_RECURSIVE_001', 'tail-call'),
('TAIL_RECURSIVE_001', 'factorial'),
('TAIL_RECURSIVE_001', 'optimized'),
('TAIL_RECURSIVE_002', 'recursion'),
('TAIL_RECURSIVE_002', 'tail-call'),
('TAIL_RECURSIVE_002', 'fibonacci'),
('TAIL_RECURSIVE_002', 'optimized');

-- BINARY_OP patterns
INSERT INTO patterns (id, category, stack_effect, code_template, performance_class, description, created_at, updated_at) VALUES
('BINARY_OP_001', 'binary_op', '( a b -- c )', ': NAME ( a b -- c )\n  OP ;', 'O(1)', 'Simple binary operation template', datetime('now'), datetime('now')),
('BINARY_OP_002', 'binary_op', '( a b -- avg )', ': NAME ( a b -- avg )\n  + 2 / ;', 'O(1)', 'Average of two numbers', datetime('now'), datetime('now')),
('BINARY_OP_003', 'binary_op', '( a b -- gcd )', ': NAME ( a b -- gcd )\n  begin dup while tuck mod repeat drop ;', 'O(log n)', 'Greatest common divisor', datetime('now'), datetime('now'));

INSERT INTO pattern_tags VALUES
('BINARY_OP_001', 'arithmetic'),
('BINARY_OP_001', 'binary'),
('BINARY_OP_002', 'arithmetic'),
('BINARY_OP_002', 'average'),
('BINARY_OP_003', 'arithmetic'),
('BINARY_OP_003', 'gcd');

-- UNARY_OP patterns
INSERT INTO patterns (id, category, stack_effect, code_template, performance_class, description, created_at, updated_at) VALUES
('UNARY_OP_001', 'unary_op', '( n -- -n )', ': NAME ( n -- -n )\n  negate ;', 'O(1)', 'Negate a number', datetime('now'), datetime('now')),
('UNARY_OP_002', 'unary_op', '( n -- n*2 )', ': NAME ( n -- n*2 )\n  2 * ;', 'O(1)', 'Double a number', datetime('now'), datetime('now')),
('UNARY_OP_003', 'unary_op', '( n -- n/2 )', ': NAME ( n -- n/2 )\n  2 / ;', 'O(1)', 'Halve a number', datetime('now'), datetime('now'));

INSERT INTO pattern_tags VALUES
('UNARY_OP_001', 'arithmetic'),
('UNARY_OP_001', 'unary'),
('UNARY_OP_002', 'arithmetic'),
('UNARY_OP_002', 'double'),
('UNARY_OP_003', 'arithmetic'),
('UNARY_OP_003', 'halve');

-- STACK_MANIP patterns
INSERT INTO patterns (id, category, stack_effect, code_template, performance_class, description, created_at, updated_at) VALUES
('STACK_MANIP_001', 'stack_manipulation', '( a b c -- c b a )', ': NAME ( a b c -- c b a )\n  rot rot ;', 'O(1)', 'Reverse top 3 stack items', datetime('now'), datetime('now')),
('STACK_MANIP_002', 'stack_manipulation', '( a b -- b a b )', ': NAME ( a b -- b a b )\n  tuck ;', 'O(1)', 'Tuck second item over top', datetime('now'), datetime('now')),
('STACK_MANIP_003', 'stack_manipulation', '( a b c -- b c a )', ': NAME ( a b c -- b c a )\n  rot ;', 'O(1)', 'Rotate top 3 stack items', datetime('now'), datetime('now'));

INSERT INTO pattern_tags VALUES
('STACK_MANIP_001', 'stack'),
('STACK_MANIP_001', 'manipulation'),
('STACK_MANIP_001', 'reverse'),
('STACK_MANIP_002', 'stack'),
('STACK_MANIP_002', 'manipulation'),
('STACK_MANIP_002', 'tuck'),
('STACK_MANIP_003', 'stack'),
('STACK_MANIP_003', 'manipulation'),
('STACK_MANIP_003', 'rotate');

-- OPTIMIZATION patterns
INSERT INTO patterns (id, category, stack_effect, code_template, performance_class, description, created_at, updated_at) VALUES
('OPTIMIZATION_001', 'optimization', '( n -- n*8 )', ': NAME ( n -- n*8 )\n  3 lshift ;', 'O(1)', 'Multiply by 8 using bit shift', datetime('now'), datetime('now')),
('OPTIMIZATION_002', 'optimization', '( n -- bool )', ': NAME ( n -- bool )\n  1 and 0= ;', 'O(1)', 'Check if even using bitwise and', datetime('now'), datetime('now')),
('OPTIMIZATION_003', 'optimization', '( n -- n*10 )', ': NAME ( n -- n*10 )\n  dup 2 lshift + dup + ;', 'O(1)', 'Multiply by 10 optimized', datetime('now'), datetime('now'));

INSERT INTO pattern_tags VALUES
('OPTIMIZATION_001', 'optimization'),
('OPTIMIZATION_001', 'bitwise'),
('OPTIMIZATION_001', 'multiply'),
('OPTIMIZATION_002', 'optimization'),
('OPTIMIZATION_002', 'bitwise'),
('OPTIMIZATION_002', 'even'),
('OPTIMIZATION_003', 'optimization'),
('OPTIMIZATION_003', 'multiply');

-- Test cases
INSERT INTO pattern_test_cases (pattern_id, input_values, output_values, description) VALUES
('DUP_TRANSFORM_001', '[5]', '[25]', '5² = 25'),
('DUP_TRANSFORM_001', '[0]', '[0]', '0² = 0'),
('DUP_TRANSFORM_001', '[-3]', '[9]', '(-3)² = 9'),
('CONDITIONAL_001', '[5]', '[5]', 'abs(5) = 5'),
('CONDITIONAL_001', '[-5]', '[5]', 'abs(-5) = 5'),
('CONDITIONAL_001', '[0]', '[0]', 'abs(0) = 0'),
('ACCUMULATOR_LOOP_001', '[5]', '[15]', '1+2+3+4+5 = 15'),
('ACCUMULATOR_LOOP_002', '[5]', '[120]', '5! = 120'),
('RECURSIVE_001', '[5]', '[120]', '5! = 120'),
('RECURSIVE_001', '[0]', '[1]', '0! = 1');

-- Template variables
INSERT INTO template_variables (pattern_id, variable_name, description, example, required) VALUES
('DUP_TRANSFORM_001', 'NAME', 'Function name', 'square', 1),
('BINARY_OP_001', 'NAME', 'Function name', 'add', 1),
('BINARY_OP_001', 'OP', 'Binary operation', '+', 1),
('RECURSIVE_001', 'NAME', 'Function name', 'factorial', 1);
