/**
 * Platform-Specific Optimization Tests
 *
 * Tests platform-specific code paths including:
 * - x86_64 inline assembly optimizations
 * - ARM64/other architecture fallbacks
 * - Performance validation
 *
 * Compile:
 *   gcc -I.. -o test_platform_optimizations test_platform_optimizations.c
 *
 * Run:
 *   ./test_platform_optimizations
 */

#include "../forth_runtime.h"
#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include <time.h>
#include <assert.h>

// Test counter
static int tests_passed = 0;
static int tests_failed = 0;

#define TEST(name) \
    printf("Testing: %s...", name); \
    fflush(stdout);

#define PASS() \
    printf(" PASS\n"); \
    tests_passed++;

#define FAIL(msg) \
    printf(" FAIL: %s\n", msg); \
    tests_failed++;

// ============================================================================
// FUNCTIONAL CORRECTNESS TESTS
// ============================================================================

void test_fast_add_correctness() {
    TEST("fast_add correctness");

    // Test basic addition
    assert(fast_add(5, 3) == 8);
    assert(fast_add(0, 0) == 0);
    assert(fast_add(-5, 5) == 0);
    assert(fast_add(100, 200) == 300);

    // Test edge cases
    assert(fast_add(INT64_MAX - 1, 1) == INT64_MAX);
    assert(fast_add(INT64_MIN + 1, -1) == INT64_MIN);

    // Test commutative property
    for (int i = -10; i <= 10; i++) {
        for (int j = -10; j <= 10; j++) {
            assert(fast_add(i, j) == fast_add(j, i));
        }
    }

    PASS();
}

void test_fast_mul_correctness() {
    TEST("fast_mul correctness");

    // Test basic multiplication
    assert(fast_mul(5, 3) == 15);
    assert(fast_mul(0, 100) == 0);
    assert(fast_mul(1, 42) == 42);
    assert(fast_mul(-5, 3) == -15);
    assert(fast_mul(-5, -3) == 15);

    // Test identity
    for (int i = -10; i <= 10; i++) {
        assert(fast_mul(i, 1) == i);
        assert(fast_mul(i, 0) == 0);
    }

    // Test commutative property
    for (int i = -10; i <= 10; i++) {
        for (int j = -10; j <= 10; j++) {
            assert(fast_mul(i, j) == fast_mul(j, i));
        }
    }

    PASS();
}

void test_fast_operations_match_standard() {
    TEST("fast operations match standard C");

    // Generate random test cases
    srand(time(NULL));

    for (int i = 0; i < 1000; i++) {
        int64_t a = (rand() % 10000) - 5000;
        int64_t b = (rand() % 10000) - 5000;

        // fast_add should match standard addition
        assert(fast_add(a, b) == (a + b));

        // fast_mul should match standard multiplication
        assert(fast_mul(a, b) == (a * b));
    }

    PASS();
}

// ============================================================================
// PLATFORM DETECTION TESTS
// ============================================================================

void test_platform_detection() {
    TEST("platform detection");

    printf("\n");
    printf("  Detected platform: ");

#ifdef __x86_64__
    printf("x86_64 (using inline assembly optimizations)\n");
#elif defined(__aarch64__) || defined(__arm64__)
    printf("ARM64 (using C fallback)\n");
#elif defined(__arm__)
    printf("ARM32 (using C fallback)\n");
#else
    printf("Unknown (using C fallback)\n");
#endif

    PASS();
}

// ============================================================================
// PERFORMANCE BENCHMARKS
// ============================================================================

#define BENCHMARK_ITERATIONS 10000000

void benchmark_fast_add() {
    TEST("fast_add performance benchmark");

    struct timespec start, end;
    volatile int64_t result = 0;

    // Benchmark fast_add
    clock_gettime(CLOCK_MONOTONIC, &start);
    for (int i = 0; i < BENCHMARK_ITERATIONS; i++) {
        result = fast_add(result, i);
    }
    clock_gettime(CLOCK_MONOTONIC, &end);

    double elapsed = (end.tv_sec - start.tv_sec) +
                     (end.tv_nsec - start.tv_nsec) / 1e9;
    double ns_per_op = (elapsed / BENCHMARK_ITERATIONS) * 1e9;

    printf("\n");
    printf("  Iterations: %d\n", BENCHMARK_ITERATIONS);
    printf("  Total time: %.3f seconds\n", elapsed);
    printf("  Time per operation: %.2f nanoseconds\n", ns_per_op);

#ifdef __x86_64__
    printf("  Expected: <5 ns (x86_64 inline asm)\n");
    if (ns_per_op > 10.0) {
        FAIL("Performance slower than expected for x86_64");
        return;
    }
#else
    printf("  Expected: <10 ns (C fallback)\n");
    if (ns_per_op > 20.0) {
        FAIL("Performance slower than expected for fallback");
        return;
    }
#endif

    PASS();
}

void benchmark_fast_mul() {
    TEST("fast_mul performance benchmark");

    struct timespec start, end;
    volatile int64_t result = 1;

    // Benchmark fast_mul
    clock_gettime(CLOCK_MONOTONIC, &start);
    for (int i = 1; i < BENCHMARK_ITERATIONS; i++) {
        result = fast_mul(result % 100, i % 100);
    }
    clock_gettime(CLOCK_MONOTONIC, &end);

    double elapsed = (end.tv_sec - start.tv_sec) +
                     (end.tv_nsec - start.tv_nsec) / 1e9;
    double ns_per_op = (elapsed / BENCHMARK_ITERATIONS) * 1e9;

    printf("\n");
    printf("  Iterations: %d\n", BENCHMARK_ITERATIONS);
    printf("  Total time: %.3f seconds\n", elapsed);
    printf("  Time per operation: %.2f nanoseconds\n", ns_per_op);

#ifdef __x86_64__
    printf("  Expected: <10 ns (x86_64 inline asm)\n");
    if (ns_per_op > 20.0) {
        FAIL("Performance slower than expected for x86_64");
        return;
    }
#else
    printf("  Expected: <15 ns (C fallback)\n");
    if (ns_per_op > 30.0) {
        FAIL("Performance slower than expected for fallback");
        return;
    }
#endif

    PASS();
}

// ============================================================================
// MAIN TEST RUNNER
// ============================================================================

int main() {
    printf("================================\n");
    printf("Platform-Specific Optimization Tests\n");
    printf("================================\n\n");

    // Correctness tests
    test_platform_detection();
    test_fast_add_correctness();
    test_fast_mul_correctness();
    test_fast_operations_match_standard();

    // Performance benchmarks
    printf("\n");
    printf("Performance Benchmarks\n");
    printf("----------------------\n");
    benchmark_fast_add();
    benchmark_fast_mul();

    // Summary
    printf("\n");
    printf("================================\n");
    printf("Test Results\n");
    printf("================================\n");
    printf("Passed: %d\n", tests_passed);
    printf("Failed: %d\n", tests_failed);
    printf("Total:  %d\n", tests_passed + tests_failed);
    printf("================================\n");

    return (tests_failed == 0) ? 0 : 1;
}
