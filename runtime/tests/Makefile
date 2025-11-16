# Makefile for Fast Forth C Runtime Tests

CC = gcc
CFLAGS = -I.. -pthread -Wall -Wextra -std=c11 -O2
LDFLAGS = -lpthread -lm

# Platform detection
UNAME_S := $(shell uname -s)
UNAME_M := $(shell uname -m)

# Platform-specific flags
ifeq ($(UNAME_S),Linux)
    CFLAGS += -DPLATFORM_LINUX
    LDFLAGS += -lrt
endif
ifeq ($(UNAME_S),Darwin)
    CFLAGS += -DPLATFORM_MACOS
endif

# Architecture-specific optimizations (conditional)
ifeq ($(UNAME_M),x86_64)
    CFLAGS += -march=native
endif

# Source files
RUNTIME_SRCS = ../forth_runtime.c ../memory.c ../ffi.c ../bootstrap.c ../concurrency.c

# Test executables
TESTS = test_concurrency test_platform_optimizations

.PHONY: all clean test valgrind tsan bench help

all: $(TESTS)

# Build test executables
test_concurrency: test_concurrency.c $(RUNTIME_SRCS)
	$(CC) $(CFLAGS) -o $@ $^ $(LDFLAGS)

test_platform_optimizations: test_platform_optimizations.c
	$(CC) $(CFLAGS) -o $@ $< $(LDFLAGS)

# Run all tests
test: $(TESTS)
	@echo "================================"
	@echo "Running C Runtime Tests"
	@echo "================================"
	@echo "Platform: $(UNAME_S) $(UNAME_M)"
	@echo ""
	@for test in $(TESTS); do \
		echo "Running $$test..."; \
		./$$test || exit 1; \
		echo ""; \
	done
	@echo "================================"
	@echo "All C tests passed!"
	@echo "================================"

clean:
	rm -f $(TESTS) *_tsan *.o

# Run with valgrind for memory leak detection
valgrind: $(TESTS)
	@for test in $(TESTS); do \
		echo "Running $$test with valgrind..."; \
		valgrind --leak-check=full --show-leak-kinds=all ./$$test; \
	done

# Run with thread sanitizer
tsan: test_concurrency.c $(RUNTIME_SRCS)
	$(CC) $(CFLAGS) -fsanitize=thread -o test_concurrency_tsan $^ $(LDFLAGS)
	./test_concurrency_tsan
	rm -f test_concurrency_tsan

# Benchmark mode
bench: test_concurrency test_platform_optimizations
	@echo "Running benchmarks..."
	./test_concurrency 2>&1 | grep PERF || true
	./test_platform_optimizations

# Help target
help:
	@echo "Fast Forth C Runtime Test Makefile"
	@echo ""
	@echo "Targets:"
	@echo "  all                          - Build all test executables"
	@echo "  test                         - Build and run all tests"
	@echo "  clean                        - Remove build artifacts"
	@echo "  valgrind                     - Run tests with memory leak detection"
	@echo "  tsan                         - Run concurrency tests with thread sanitizer"
	@echo "  bench                        - Run performance benchmarks"
	@echo ""
	@echo "Individual tests:"
	@echo "  test_concurrency             - Thread and channel tests"
	@echo "  test_platform_optimizations  - x86_64 vs fallback optimization tests"
	@echo ""
	@echo "Platform: $(UNAME_S) $(UNAME_M)"
