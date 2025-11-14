# Fast Forth Benchmark Report

**Date**: 2025-11-14 02:40:48

## Platform Information

- **OS**: Darwin
- **Machine**: arm64
- **GCC**: Apple clang version 16.0.0 (clang-1600.0.26.6)
- **GForth**: gforth 0.7.3

## C Baseline Results (gcc -O2)

| Benchmark | Time (ms) | Status |
|-----------|-----------|--------|
| sieve | 0.004 | FAIL |
| fibonacci_rec | - | error |
| matrix | - | error |
| bubble_sort | - | error |
| string_ops | - | error |

## Target Performance Comparison

Based on BENCHMARK_SUITE_SPECIFICATION.md targets:

| Benchmark | Target (ms) | Actual (ms) | Ratio | Status |
|-----------|-------------|-------------|-------|--------|
| sieve (Target for gcc -O2) | 50.0 | 0.004 | 0.00x | ✓ On target |
