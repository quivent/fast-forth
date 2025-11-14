/**
 * Bubble Sort - C Reference Implementation
 *
 * Sorts 1000 random integers
 * Target: ~50ms (gcc -O2)
 */

#include <stdio.h>
#include <stdlib.h>
#include <time.h>
#include <stdbool.h>
#include <string.h>

void bubble_sort(int *arr, int len) {
    for (int i = 0; i < len; i++) {
        for (int j = 0; j < len - i - 1; j++) {
            if (arr[j] > arr[j + 1]) {
                // Swap
                int temp = arr[j];
                arr[j] = arr[j + 1];
                arr[j + 1] = temp;
            }
        }
    }
}

bool is_sorted(int *arr, int len) {
    for (int i = 0; i < len - 1; i++) {
        if (arr[i] > arr[i + 1]) {
            return false;
        }
    }
    return true;
}

void init_random_array(int *arr, int len, int seed) {
    srand(seed);
    for (int i = 0; i < len; i++) {
        arr[i] = rand() % 10000;
    }
}

double benchmark_bubble_sort(int len, int iterations) {
    int *arr = (int *)malloc(len * sizeof(int));
    int *backup = (int *)malloc(len * sizeof(int));

    init_random_array(backup, len, 42);

    clock_t start = clock();

    for (int i = 0; i < iterations; i++) {
        // Copy array for each iteration
        memcpy(arr, backup, len * sizeof(int));
        bubble_sort(arr, len);
    }

    clock_t end = clock();
    double elapsed = ((double)(end - start)) / CLOCKS_PER_SEC * 1000.0;

    // Verify last sort
    printf("Sorted correctly: %s\n", is_sorted(arr, len) ? "YES" : "NO");
    printf("First 5 elements: %d %d %d %d %d\n",
           arr[0], arr[1], arr[2], arr[3], arr[4]);

    free(arr);
    free(backup);

    return elapsed / iterations;
}

int main(int argc, char **argv) {
    int len = (argc > 1) ? atoi(argv[1]) : 1000;
    int iterations = (argc > 2) ? atoi(argv[2]) : 10;

    printf("C Bubble Sort Benchmark (gcc -O2 baseline)\n");
    printf("===========================================\n");
    printf("Array size: %d\n", len);
    printf("Iterations: %d\n\n", iterations);

    // Warmup
    benchmark_bubble_sort(len / 2, 2);

    double avg_time = benchmark_bubble_sort(len, iterations);

    printf("Average time: %.3f ms\n", avg_time);

    return 0;
}
