# C Baseline Benchmarks Makefile
# Provides reference performance targets for Fast Forth

CC = gcc
CFLAGS = -Wall -Wextra -O2 -std=c11 -march=native
DEBUG_CFLAGS = -Wall -Wextra -g -O0 -std=c11
PROF_CFLAGS = -Wall -Wextra -O2 -std=c11 -march=native -pg

TARGETS = sieve fibonacci matrix bubble_sort string_ops

.PHONY: all clean run benchmark compare debug

all: $(TARGETS)

# Compilation rules
sieve: sieve.c
	$(CC) $(CFLAGS) $< -o $@

fibonacci: fibonacci.c
	$(CC) $(CFLAGS) $< -o $@

matrix: matrix.c
	$(CC) $(CFLAGS) $< -o $@ -lm

bubble_sort: bubble_sort.c
	$(CC) $(CFLAGS) $< -o $@

string_ops: string_ops.c
	$(CC) $(CFLAGS) $< -o $@

# Run all benchmarks
run: all
	@echo "====================================="
	@echo "C BASELINE BENCHMARKS (gcc -O2)"
	@echo "====================================="
	@echo ""
	@echo "1. Sieve of Eratosthenes:"
	@./sieve 8190 100
	@echo ""
	@echo "2. Fibonacci:"
	@./fibonacci 35 40
	@echo ""
	@echo "3. Matrix Multiplication:"
	@./matrix 100 10
	@echo ""
	@echo "4. Bubble Sort:"
	@./bubble_sort 1000 10
	@echo ""
	@echo "5. String Operations:"
	@./string_ops 10000 10000
	@echo ""
	@echo "====================================="

# Quick benchmark (smaller iterations for CI)
benchmark: all
	@echo "Running quick benchmark suite..."
	@./sieve 8190 10
	@./fibonacci 30 35
	@./matrix 50 5
	@./bubble_sort 500 5
	@./string_ops 5000 5000

# Compare different optimization levels
compare: sieve.c fibonacci.c
	@echo "Comparing optimization levels for Sieve..."
	@gcc -O0 sieve.c -o sieve_O0
	@gcc -O1 sieve.c -o sieve_O1
	@gcc -O2 sieve.c -o sieve_O2
	@gcc -O3 sieve.c -o sieve_O3
	@echo "O0:"; @./sieve_O0 8190 10
	@echo "O1:"; @./sieve_O1 8190 10
	@echo "O2:"; @./sieve_O2 8190 10
	@echo "O3:"; @./sieve_O3 8190 10
	@rm sieve_O0 sieve_O1 sieve_O2 sieve_O3

# Debug builds
debug: CFLAGS = $(DEBUG_CFLAGS)
debug: all

# Profile builds
profile: CFLAGS = $(PROF_CFLAGS)
profile: all

clean:
	rm -f $(TARGETS)
	rm -f *.o
	rm -f gmon.out

# Individual test runs
test-sieve: sieve
	./sieve 8190 100

test-fibonacci: fibonacci
	./fibonacci 35 40

test-matrix: matrix
	./matrix 100 10

test-bubble: bubble_sort
	./bubble_sort 1000 10

test-string: string_ops
	./string_ops 10000 10000

# Help
help:
	@echo "C Baseline Benchmarks"
	@echo "====================="
	@echo ""
	@echo "Targets:"
	@echo "  all        - Build all benchmarks (default)"
	@echo "  run        - Build and run all benchmarks"
	@echo "  benchmark  - Quick benchmark suite"
	@echo "  compare    - Compare optimization levels"
	@echo "  debug      - Build with debug symbols"
	@echo "  profile    - Build with profiling"
	@echo "  clean      - Remove built files"
	@echo ""
	@echo "Individual tests:"
	@echo "  test-sieve"
	@echo "  test-fibonacci"
	@echo "  test-matrix"
	@echo "  test-bubble"
	@echo "  test-string"
