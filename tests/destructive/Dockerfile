# Destructive Testing Container
# Provides isolated environment with configurable resource limits for testing
# failure scenarios: OOM, disk full, stack overflow, fd exhaustion

FROM rust:1.75-slim

# Install system utilities
RUN apt-get update && apt-get install -y \
    build-essential \
    strace \
    ulimit \
    util-linux \
    e2fsprogs \
    && rm -rf /var/lib/apt/lists/*

# Create test user with limited privileges
RUN useradd -m -s /bin/bash testuser

WORKDIR /app

# Copy project files
COPY . /app

# Set ownership
RUN chown -R testuser:testuser /app

# Switch to test user
USER testuser

# Build the project in release mode for more realistic testing
RUN cargo build --release --features destructive_tests

# Default command runs all destructive tests
CMD ["cargo", "test", "--release", "--features", "destructive_tests", "--", "--test-threads=1"]

# Resource limit configurations (override with docker run flags):
# Memory: --memory=128m --memory-swap=128m
# CPU: --cpus=1
# PIDs: --pids-limit=100
# Disk: Use volume mounts with size limits
