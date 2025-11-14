/**
 * String Operations - C Reference Implementation
 *
 * String copy, reverse, and search operations
 * Target: 70-90% performance with LLVM optimizations
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>

void string_reverse(char *str, int len) {
    for (int i = 0; i < len / 2; i++) {
        char temp = str[i];
        str[i] = str[len - 1 - i];
        str[len - 1 - i] = temp;
    }
}

// Boyer-Moore-Horspool string search
int string_search(const char *haystack, int hlen, const char *needle, int nlen) {
    if (nlen > hlen) return -1;
    if (nlen == 0) return 0;

    // Build bad character table
    int bad_char[256];
    for (int i = 0; i < 256; i++) {
        bad_char[i] = nlen;
    }
    for (int i = 0; i < nlen - 1; i++) {
        bad_char[(unsigned char)needle[i]] = nlen - 1 - i;
    }

    // Search
    int pos = 0;
    while (pos <= hlen - nlen) {
        int i = nlen - 1;
        while (i >= 0 && needle[i] == haystack[pos + i]) {
            i--;
        }
        if (i < 0) {
            return pos;
        }
        pos += bad_char[(unsigned char)haystack[pos + nlen - 1]];
    }

    return -1;
}

double benchmark_string_copy(int len, int iterations) {
    char *src = (char *)malloc(len);
    char *dst = (char *)malloc(len);

    memset(src, 'A', len);

    clock_t start = clock();

    for (int i = 0; i < iterations; i++) {
        memcpy(dst, src, len);
    }

    clock_t end = clock();
    double elapsed = ((double)(end - start)) / CLOCKS_PER_SEC * 1000.0;

    free(src);
    free(dst);

    return elapsed / iterations;
}

double benchmark_string_reverse(int len, int iterations) {
    char *str = (char *)malloc(len + 1);
    char *backup = (char *)malloc(len + 1);

    for (int i = 0; i < len; i++) {
        backup[i] = 'A' + (i % 26);
    }
    backup[len] = '\0';

    clock_t start = clock();

    for (int i = 0; i < iterations; i++) {
        memcpy(str, backup, len);
        string_reverse(str, len);
    }

    clock_t end = clock();
    double elapsed = ((double)(end - start)) / CLOCKS_PER_SEC * 1000.0;

    free(str);
    free(backup);

    return elapsed / iterations;
}

double benchmark_string_search(int iterations) {
    const char *haystack = "The quick brown fox jumps over the lazy dog. "
                          "Pack my box with five dozen liquor jugs. "
                          "How vexingly quick daft zebras jump!";
    const char *needle = "quick";

    int hlen = strlen(haystack);
    int nlen = strlen(needle);

    clock_t start = clock();

    int result = 0;
    for (int i = 0; i < iterations; i++) {
        result = string_search(haystack, hlen, needle, nlen);
    }

    clock_t end = clock();
    double elapsed = ((double)(end - start)) / CLOCKS_PER_SEC * 1000.0;

    printf("Found '%s' at position: %d\n", needle, result);

    return elapsed / iterations;
}

int main(int argc, char **argv) {
    int len = (argc > 1) ? atoi(argv[1]) : 10000;
    int iterations = (argc > 2) ? atoi(argv[2]) : 10000;

    printf("C String Operations Benchmark (gcc -O2 baseline)\n");
    printf("=================================================\n\n");

    printf("STRING COPY (%d bytes)\n", len);
    printf("----------------------\n");
    double copy_time = benchmark_string_copy(len, iterations);
    printf("Average time: %.6f ms\n\n", copy_time);

    printf("STRING REVERSE (%d bytes)\n", len);
    printf("-------------------------\n");
    double reverse_time = benchmark_string_reverse(len, iterations);
    printf("Average time: %.6f ms\n\n", reverse_time);

    printf("STRING SEARCH (Boyer-Moore-Horspool)\n");
    printf("-------------------------------------\n");
    double search_time = benchmark_string_search(iterations);
    printf("Average time: %.6f ms\n\n", search_time);

    return 0;
}
