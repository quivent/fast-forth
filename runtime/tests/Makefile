# Makefile for Fast Forth Concurrency Tests

CC = gcc
CFLAGS = -I.. -pthread -Wall -Wextra -std=c11 -O2 -march=native
LDFLAGS = -lpthread

# Source files
RUNTIME_SRCS = ../forth_runtime.c ../memory.c ../ffi.c ../bootstrap.c ../concurrency.c
TEST_SRCS = test_concurrency.c

# Output binary
TEST_BIN = test_concurrency

.PHONY: all clean test

all: $(TEST_BIN)

$(TEST_BIN): $(TEST_SRCS) $(RUNTIME_SRCS)
	$(CC) $(CFLAGS) -o $@ $^ $(LDFLAGS)

test: $(TEST_BIN)
	./$(TEST_BIN)

clean:
	rm -f $(TEST_BIN)

# Run with valgrind for memory leak detection
valgrind: $(TEST_BIN)
	valgrind --leak-check=full --show-leak-kinds=all ./$(TEST_BIN)

# Run with thread sanitizer
tsan: $(TEST_SRCS) $(RUNTIME_SRCS)
	$(CC) $(CFLAGS) -fsanitize=thread -o $(TEST_BIN)_tsan $^ $(LDFLAGS)
	./$(TEST_BIN)_tsan
	rm -f $(TEST_BIN)_tsan

# Benchmark mode
bench: $(TEST_BIN)
	./$(TEST_BIN) 2>&1 | grep PERF
