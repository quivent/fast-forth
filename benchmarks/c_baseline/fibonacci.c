/**
 * Fibonacci - C Reference Implementation
 *
 * Both recursive and iterative versions for comparison
 * Recursive target: ~35ms for fib(35) (gcc -O2)
 */

#include <stdio.h>
#include <stdlib.h>
#include <time.h>

// Recursive Fibonacci (exponential complexity)
long long fib_recursive(int n) {
    if (n < 2) {
        return n;
    }
    return fib_recursive(n - 1) + fib_recursive(n - 2);
}

// Iterative Fibonacci (linear complexity)
long long fib_iterative(int n) {
    if (n < 2) {
        return n;
    }

    long long a = 0, b = 1;
    for (int i = 2; i <= n; i++) {
        long long temp = a + b;
        a = b;
        b = temp;
    }
    return b;
}

double benchmark_recursive(int n, int iterations) {
    clock_t start = clock();

    long long result = 0;
    for (int i = 0; i < iterations; i++) {
        result = fib_recursive(n);
    }

    clock_t end = clock();
    double elapsed = ((double)(end - start)) / CLOCKS_PER_SEC * 1000.0;

    printf("Fib_recursive(%d): %lld\n", n, result);
    return elapsed / iterations;
}

double benchmark_iterative(int n, int iterations) {
    clock_t start = clock();

    long long result = 0;
    for (int i = 0; i < iterations; i++) {
        result = fib_iterative(n);
    }

    clock_t end = clock();
    double elapsed = ((double)(end - start)) / CLOCKS_PER_SEC * 1000.0;

    printf("Fib_iterative(%d): %lld\n", n, result);
    return elapsed / iterations;
}

int main(int argc, char **argv) {
    int n_recursive = (argc > 1) ? atoi(argv[1]) : 35;
    int n_iterative = (argc > 2) ? atoi(argv[2]) : 40;

    printf("C Fibonacci Benchmark (gcc -O2 baseline)\n");
    printf("=========================================\n\n");

    // Recursive benchmark
    printf("RECURSIVE VERSION\n");
    printf("-----------------\n");
    printf("Computing fib(%d)...\n", n_recursive);

    // Warmup
    for (int i = 0; i < 3; i++) {
        fib_recursive(n_recursive - 5);
    }

    double avg_time_rec = benchmark_recursive(n_recursive, 10);
    printf("Average time: %.3f ms\n\n", avg_time_rec);

    // Iterative benchmark
    printf("ITERATIVE VERSION\n");
    printf("-----------------\n");
    printf("Computing fib(%d)...\n", n_iterative);

    // Warmup
    for (int i = 0; i < 10; i++) {
        fib_iterative(n_iterative);
    }

    double avg_time_iter = benchmark_iterative(n_iterative, 1000);
    printf("Average time: %.6f ms\n\n", avg_time_iter);

    // Validation
    if (n_recursive == 35) {
        long long result = fib_recursive(35);
        printf("Validation (recursive): %s (expected 9227465)\n",
               result == 9227465 ? "PASS" : "FAIL");
    }

    if (n_iterative == 40) {
        long long result = fib_iterative(40);
        printf("Validation (iterative): %s (expected 102334155)\n",
               result == 102334155 ? "PASS" : "FAIL");
    }

    return 0;
}
