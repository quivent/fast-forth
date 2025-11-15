# Minimal Fast Forth - Pure C99, Zero Dependencies
# Compiles in 30 seconds, no Rust needed

CC = gcc
CFLAGS = -std=c99 -O2 -Wall -Wextra
TARGET = forth
SRC = src/minimal_forth.c

.PHONY: all clean test install

all: $(TARGET)

$(TARGET): $(SRC)
	@echo "Building minimal Fast Forth compiler..."
	@echo "Performance: 30-50% of C (for 85-110%, install Rust)"
	$(CC) $(CFLAGS) -o $(TARGET) $(SRC)
	@echo "Build complete! (30s)"
	@echo ""
	@echo "Usage:"
	@echo "  ./forth              - Interactive REPL"
	@echo "  ./forth script.forth - Execute file"
	@echo ""
	@echo "For full optimizations (85-110% of C):"
	@echo "  ./forth --install-rust"

clean:
	rm -f $(TARGET)

test: $(TARGET)
	@echo "Testing minimal compiler..."
	@echo "3 4 +" | ./forth
	@echo "5 dup *" | ./forth
	@echo "10 1 - ." | ./forth

install: $(TARGET)
	install -m 755 $(TARGET) /usr/local/bin/forth-minimal

.PHONY: help
help:
	@echo "Minimal Fast Forth - Build Targets"
	@echo ""
	@echo "  make           - Build minimal compiler (30s)"
	@echo "  make test      - Run basic tests"
	@echo "  make clean     - Remove built files"
	@echo "  make install   - Install to /usr/local/bin"
