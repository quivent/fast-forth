/**
 * Matrix Multiplication - C Reference Implementation
 *
 * Dense matrix multiplication (NxN)
 * Target: ~80ms for 100x100 matrices (gcc -O2)
 */

#include <stdio.h>
#include <stdlib.h>
#include <time.h>
#include <string.h>

typedef struct {
    int rows;
    int cols;
    double *data;
} Matrix;

Matrix* create_matrix(int rows, int cols) {
    Matrix *m = (Matrix *)malloc(sizeof(Matrix));
    m->rows = rows;
    m->cols = cols;
    m->data = (double *)malloc(rows * cols * sizeof(double));
    memset(m->data, 0, rows * cols * sizeof(double));
    return m;
}

void free_matrix(Matrix *m) {
    free(m->data);
    free(m);
}

double matrix_get(Matrix *m, int i, int j) {
    return m->data[i * m->cols + j];
}

void matrix_set(Matrix *m, int i, int j, double value) {
    m->data[i * m->cols + j] = value;
}

void matrix_multiply(Matrix *a, Matrix *b, Matrix *c) {
    // c = a * b
    for (int i = 0; i < a->rows; i++) {
        for (int j = 0; j < b->cols; j++) {
            double sum = 0.0;
            for (int k = 0; k < a->cols; k++) {
                sum += matrix_get(a, i, k) * matrix_get(b, k, j);
            }
            matrix_set(c, i, j, sum);
        }
    }
}

void init_random_matrix(Matrix *m, int seed) {
    srand(seed);
    for (int i = 0; i < m->rows * m->cols; i++) {
        m->data[i] = (double)(rand() % 100) / 10.0;
    }
}

double benchmark_matrix_mult(int n, int iterations) {
    Matrix *a = create_matrix(n, n);
    Matrix *b = create_matrix(n, n);
    Matrix *c = create_matrix(n, n);

    init_random_matrix(a, 42);
    init_random_matrix(b, 43);

    clock_t start = clock();

    for (int i = 0; i < iterations; i++) {
        matrix_multiply(a, b, c);
    }

    clock_t end = clock();
    double elapsed = ((double)(end - start)) / CLOCKS_PER_SEC * 1000.0;

    // Print sample result for verification
    printf("Result[0][0] = %.2f\n", matrix_get(c, 0, 0));

    free_matrix(a);
    free_matrix(b);
    free_matrix(c);

    return elapsed / iterations;
}

int main(int argc, char **argv) {
    int n = (argc > 1) ? atoi(argv[1]) : 100;
    int iterations = (argc > 2) ? atoi(argv[2]) : 10;

    printf("C Matrix Multiplication Benchmark (gcc -O2 baseline)\n");
    printf("=====================================================\n");
    printf("Matrix size: %dx%d\n", n, n);
    printf("Iterations: %d\n\n", iterations);

    // Warmup with smaller matrix
    benchmark_matrix_mult(n / 2, 2);

    double avg_time = benchmark_matrix_mult(n, iterations);

    printf("Average time: %.3f ms\n", avg_time);

    return 0;
}
