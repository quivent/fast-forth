/**
 * Sieve of Eratosthenes - C Reference Implementation
 *
 * Baseline for Fast Forth performance comparison
 * Target: ~50ms for n=8190 (gcc -O2)
 */

#include <stdio.h>
#include <stdlib.h>
#include <stdbool.h>
#include <time.h>
#include <string.h>

int sieve(int limit) {
    bool *is_prime = (bool *)malloc((limit + 1) * sizeof(bool));
    memset(is_prime, true, (limit + 1) * sizeof(bool));

    is_prime[0] = false;
    is_prime[1] = false;

    // Mark composites
    for (int i = 2; i * i <= limit; i++) {
        if (is_prime[i]) {
            for (int j = i * i; j <= limit; j += i) {
                is_prime[j] = false;
            }
        }
    }

    // Count primes
    int count = 0;
    for (int i = 2; i <= limit; i++) {
        if (is_prime[i]) {
            count++;
        }
    }

    free(is_prime);
    return count;
}

double benchmark_sieve(int limit, int iterations) {
    clock_t start = clock();

    int result = 0;
    for (int i = 0; i < iterations; i++) {
        result = sieve(limit);
    }

    clock_t end = clock();
    double elapsed = ((double)(end - start)) / CLOCKS_PER_SEC * 1000.0;

    printf("Sieve(%d): Found %d primes\n", limit, result);
    return elapsed / iterations;
}

int main(int argc, char **argv) {
    int limit = (argc > 1) ? atoi(argv[1]) : 8190;
    int iterations = (argc > 2) ? atoi(argv[2]) : 100;

    printf("C Sieve Benchmark (gcc -O2 baseline)\n");
    printf("=====================================\n");
    printf("Limit: %d\n", limit);
    printf("Iterations: %d\n\n", iterations);

    // Warmup
    for (int i = 0; i < 10; i++) {
        sieve(limit);
    }

    double avg_time = benchmark_sieve(limit, iterations);

    printf("Average time: %.3f ms\n", avg_time);

    // Expected results for validation
    if (limit == 8190) {
        int result = sieve(limit);
        printf("\nValidation: %s (expected 1027 primes)\n",
               result == 1027 ? "PASS" : "FAIL");
    }
    if (limit == 100) {
        int result = sieve(limit);
        printf("\nValidation: %s (expected 25 primes)\n",
               result == 25 ? "PASS" : "FAIL");
    }
    if (limit == 1000) {
        int result = sieve(limit);
        printf("\nValidation: %s (expected 168 primes)\n",
               result == 168 ? "PASS" : "FAIL");
    }

    return 0;
}
