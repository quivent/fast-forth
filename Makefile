# Fast Forth Runtime Makefile
# Stream 6: Build system for runtime kernel and standard library

CC = clang
CFLAGS = -Wall -Wextra -O3 -std=c11 -march=native
DEBUG_CFLAGS = -Wall -Wextra -g -O0 -std=c11 -fsanitize=address
LDFLAGS = -ldl -lm

# Directories
RUNTIME_DIR = runtime
TEST_DIR = tests
BUILD_DIR = build
EXAMPLES_DIR = examples

# Source files
RUNTIME_SRCS = $(RUNTIME_DIR)/forth_runtime.c \
               $(RUNTIME_DIR)/memory.c \
               $(RUNTIME_DIR)/ffi.c \
               $(RUNTIME_DIR)/bootstrap.c

TEST_SRCS = $(TEST_DIR)/test_runtime.c

# Object files
RUNTIME_OBJS = $(patsubst $(RUNTIME_DIR)/%.c,$(BUILD_DIR)/%.o,$(RUNTIME_SRCS))
TEST_OBJS = $(patsubst $(TEST_DIR)/%.c,$(BUILD_DIR)/test_%.o,$(TEST_SRCS))

# Targets
RUNTIME_LIB = $(BUILD_DIR)/libforth.a
STANDALONE_BIN = $(BUILD_DIR)/forth
TEST_BIN = $(BUILD_DIR)/test_runtime

.PHONY: all clean test install benchmark docs

all: $(RUNTIME_LIB) $(STANDALONE_BIN)

# Build directories
$(BUILD_DIR):
	mkdir -p $(BUILD_DIR)

# Runtime library (static)
$(RUNTIME_LIB): $(BUILD_DIR) $(RUNTIME_OBJS)
	ar rcs $@ $(RUNTIME_OBJS)
	ranlib $@

# Compile runtime objects
$(BUILD_DIR)/%.o: $(RUNTIME_DIR)/%.c | $(BUILD_DIR)
	$(CC) $(CFLAGS) -c $< -o $@

# Standalone executable (with REPL)
$(STANDALONE_BIN): $(RUNTIME_LIB)
	$(CC) $(CFLAGS) -DFORTH_STANDALONE $(RUNTIME_DIR)/bootstrap.c \
		-L$(BUILD_DIR) -lforth $(LDFLAGS) -o $@

# Test suite
$(BUILD_DIR)/test_%.o: $(TEST_DIR)/test_%.c | $(BUILD_DIR)
	$(CC) $(DEBUG_CFLAGS) -I. -c $< -o $@

$(TEST_BIN): $(RUNTIME_LIB) $(BUILD_DIR)/test_runtime.o
	$(CC) $(DEBUG_CFLAGS) $(BUILD_DIR)/test_runtime.o \
		-L$(BUILD_DIR) -lforth $(LDFLAGS) -o $@

# Run tests
test: $(TEST_BIN)
	@echo "Running test suite..."
	@$(TEST_BIN)

# Clean build artifacts
clean:
	rm -rf $(BUILD_DIR)

# Install to system
install: $(RUNTIME_LIB) $(STANDALONE_BIN)
	@echo "Installing Fast Forth..."
	install -d /usr/local/lib
	install -d /usr/local/include/forth
	install -d /usr/local/bin
	install -m 644 $(RUNTIME_LIB) /usr/local/lib/
	install -m 644 $(RUNTIME_DIR)/*.h /usr/local/include/forth/
	install -m 755 $(STANDALONE_BIN) /usr/local/bin/
	@echo "Installed successfully!"

# Uninstall
uninstall:
	rm -f /usr/local/lib/libforth.a
	rm -rf /usr/local/include/forth
	rm -f /usr/local/bin/forth

# Benchmark
benchmark: $(STANDALONE_BIN)
	@echo "Running benchmarks..."
	@time $(STANDALONE_BIN) $(EXAMPLES_DIR)/benchmark.forth

# Generate documentation
docs:
	@echo "Generating documentation..."
	@mkdir -p docs/html
	@doxygen Doxyfile 2>/dev/null || echo "Doxygen not installed"

# Debug build
debug: CFLAGS = $(DEBUG_CFLAGS)
debug: clean all

# Profile build
profile: CFLAGS += -pg
profile: LDFLAGS += -pg
profile: clean all

# Size report
size: $(RUNTIME_LIB) $(STANDALONE_BIN)
	@echo "Library size:"
	@size $(RUNTIME_LIB)
	@echo ""
	@echo "Binary size:"
	@size $(STANDALONE_BIN)
	@echo ""
	@echo "Stripped binary size:"
	@strip -s $(STANDALONE_BIN) -o $(BUILD_DIR)/forth.stripped
	@size $(BUILD_DIR)/forth.stripped

# Coverage build
coverage: CFLAGS += -fprofile-arcs -ftest-coverage
coverage: LDFLAGS += -lgcov --coverage
coverage: clean test
	@echo "Generating coverage report..."
	@lcov --capture --directory $(BUILD_DIR) --output-file $(BUILD_DIR)/coverage.info
	@genhtml $(BUILD_DIR)/coverage.info --output-directory $(BUILD_DIR)/coverage

# Assembly output (for inspection)
asm: $(RUNTIME_SRCS)
	$(CC) $(CFLAGS) -S $(RUNTIME_DIR)/forth_runtime.c -o $(BUILD_DIR)/forth_runtime.s

# Help
help:
	@echo "Fast Forth Build System"
	@echo ""
	@echo "Targets:"
	@echo "  all        - Build runtime library and standalone binary"
	@echo "  test       - Run test suite"
	@echo "  install    - Install to /usr/local"
	@echo "  uninstall  - Remove from /usr/local"
	@echo "  clean      - Remove build artifacts"
	@echo "  benchmark  - Run performance benchmarks"
	@echo "  docs       - Generate documentation"
	@echo "  debug      - Build with debug symbols"
	@echo "  profile    - Build with profiling enabled"
	@echo "  size       - Show binary size information"
	@echo "  coverage   - Generate code coverage report"
	@echo "  asm        - Generate assembly output"
	@echo ""
	@echo "Usage:"
	@echo "  make              # Build everything"
	@echo "  make test         # Run tests"
	@echo "  make install      # Install system-wide"
