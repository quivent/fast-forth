# Destructive Testing Infrastructure

## Overview

This directory contains containerized destructive tests for validating error handling under extreme resource constraints. These tests are designed to run safely in isolated Docker containers with configurable resource limits.

## Test Categories

### 1. Out-of-Memory (OOM) Testing
- **File**: `test_oom.rs`
- **Container Limit**: 128MB memory
- **Tests**:
  - Small allocation failures
  - Vector allocation failures
  - String allocation failures
  - Boxed allocation failures
  - OOM recovery mechanisms
  - FastForth-specific OOM handling

### 2. Disk Full Testing
- **File**: `test_disk_full.rs`
- **Container Limit**: 100MB virtual filesystem
- **Tests**:
  - Write operations until disk full
  - Append operations under disk pressure
  - Temporary file handling
  - Disk space recovery
  - Compilation output handling

### 3. Stack Overflow Testing
- **File**: `test_stack_overflow.rs`
- **Container Limit**: 1MB stack size
- **Tests**:
  - Deep recursion handling
  - Mutual recursion overflow
  - Large stack frame allocation
  - Recursive data structure traversal
  - Forth-style stack operations
  - Compiler recursion limits

### 4. File Descriptor Exhaustion
- **File**: `test_fd_exhaustion.rs`
- **Container Limit**: 256 file descriptors
- **Tests**:
  - FD limit handling
  - FD recovery mechanisms
  - FD leak detection
  - Simultaneous file operations
  - Compiler FD usage patterns
  - FD limit awareness

## Safety Mechanisms

### Container Isolation
All destructive tests run in Docker containers with strict resource limits:
- Memory limits via `--memory` and `--memory-swap`
- Stack limits via `--ulimit stack`
- File descriptor limits via `--ulimit nofile`
- CPU limits via `--cpus`

### Safety Guards
The `safety.rs` module provides multiple layers of protection:

1. **Container Detection**: Verifies execution in Docker/containerd
2. **Environment Check**: Requires explicit opt-in via `ALLOW_DESTRUCTIVE_TESTS`
3. **Resource Verification**: Checks actual resource limits before testing
4. **Panic Prevention**: Tests won't run on host system

### Safety Checks
```rust
use tests::destructive::safety::ensure_containerized;

#[test]
fn my_destructive_test() {
    ensure_containerized(); // Panics if not in container
    // ... destructive test code
}
```

## Running Tests

### Quick Start
```bash
# Run all destructive tests
./scripts/run_destructive_tests.sh

# Run specific test category
./scripts/run_destructive_tests.sh oom
./scripts/run_destructive_tests.sh disk
./scripts/run_destructive_tests.sh stack
./scripts/run_destructive_tests.sh fd
```

### Manual Docker Execution

#### OOM Tests
```bash
docker build -t fastforth-destructive-tests -f tests/destructive/Dockerfile .

docker run --rm \
    --memory=128m \
    --memory-swap=128m \
    --env DESTRUCTIVE_TESTS_ENABLED=1 \
    --env ALLOW_DESTRUCTIVE_TESTS=1 \
    fastforth-destructive-tests \
    cargo test --release --features destructive_tests test_oom -- --test-threads=1 --nocapture
```

#### Disk Full Tests
```bash
docker run --rm \
    --env DESTRUCTIVE_TESTS_ENABLED=1 \
    --env ALLOW_DESTRUCTIVE_TESTS=1 \
    fastforth-destructive-tests \
    cargo test --release --features destructive_tests test_disk_full -- --test-threads=1 --nocapture
```

#### Stack Overflow Tests
```bash
docker run --rm \
    --ulimit stack=1048576:1048576 \
    --env DESTRUCTIVE_TESTS_ENABLED=1 \
    --env ALLOW_DESTRUCTIVE_TESTS=1 \
    fastforth-destructive-tests \
    cargo test --release --features destructive_tests test_stack_overflow -- --test-threads=1 --nocapture
```

#### FD Exhaustion Tests
```bash
docker run --rm \
    --ulimit nofile=256:256 \
    --env DESTRUCTIVE_TESTS_ENABLED=1 \
    --env ALLOW_DESTRUCTIVE_TESTS=1 \
    fastforth-destructive-tests \
    cargo test --release --features destructive_tests test_fd_exhaustion -- --test-threads=1 --nocapture
```

## CI Integration

### GitHub Actions Example
```yaml
name: Destructive Tests

on:
  schedule:
    - cron: '0 2 * * *'  # Run nightly at 2 AM
  workflow_dispatch:

jobs:
  destructive-tests:
    runs-on: ubuntu-latest

    steps:
      - uses: actions/checkout@v4

      - name: Build test container
        run: docker build -t fastforth-destructive-tests -f tests/destructive/Dockerfile .

      - name: Run OOM tests
        run: |
          docker run --rm \
            --memory=128m --memory-swap=128m \
            --env DESTRUCTIVE_TESTS_ENABLED=1 \
            --env ALLOW_DESTRUCTIVE_TESTS=1 \
            fastforth-destructive-tests \
            cargo test --release --features destructive_tests test_oom -- --test-threads=1

      - name: Run disk full tests
        run: |
          docker run --rm \
            --env DESTRUCTIVE_TESTS_ENABLED=1 \
            --env ALLOW_DESTRUCTIVE_TESTS=1 \
            fastforth-destructive-tests \
            cargo test --release --features destructive_tests test_disk_full -- --test-threads=1

      - name: Run stack overflow tests
        run: |
          docker run --rm \
            --ulimit stack=1048576:1048576 \
            --env DESTRUCTIVE_TESTS_ENABLED=1 \
            --env ALLOW_DESTRUCTIVE_TESTS=1 \
            fastforth-destructive-tests \
            cargo test --release --features destructive_tests test_stack_overflow -- --test-threads=1

      - name: Run FD exhaustion tests
        run: |
          docker run --rm \
            --ulimit nofile=256:256 \
            --env DESTRUCTIVE_TESTS_ENABLED=1 \
            --env ALLOW_DESTRUCTIVE_TESTS=1 \
            fastforth-destructive-tests \
            cargo test --release --features destructive_tests test_fd_exhaustion -- --test-threads=1
```

## Local Development

### Prerequisites
- Docker installed and running
- Sufficient disk space (at least 2GB)
- Rust toolchain (handled by container)

### Development Workflow
1. Modify tests in `tests/destructive/*.rs`
2. Rebuild container: `docker build -t fastforth-destructive-tests -f tests/destructive/Dockerfile .`
3. Run specific test: `docker run ... cargo test --features destructive_tests <test_name>`
4. Iterate

### Debugging Failed Tests
```bash
# Run with full output
docker run --rm \
    --memory=128m --memory-swap=128m \
    --env DESTRUCTIVE_TESTS_ENABLED=1 \
    --env ALLOW_DESTRUCTIVE_TESTS=1 \
    --env RUST_BACKTRACE=1 \
    fastforth-destructive-tests \
    cargo test --release --features destructive_tests -- --nocapture

# Interactive container for debugging
docker run -it \
    --memory=128m --memory-swap=128m \
    --env DESTRUCTIVE_TESTS_ENABLED=1 \
    --env ALLOW_DESTRUCTIVE_TESTS=1 \
    fastforth-destructive-tests \
    /bin/bash
```

## Test Expectations

### Expected Behaviors

#### OOM Tests
- **Expected**: Graceful allocation failures with proper error returns
- **Unexpected**: Segfaults, panics without recovery, process kills

#### Disk Full Tests
- **Expected**: `std::io::Error` with `ErrorKind::Other` or similar
- **Unexpected**: Silent data loss, corrupted files, system instability

#### Stack Overflow Tests
- **Expected**: Caught panics, graceful stack unwinding
- **Unexpected**: Segfaults, process termination without cleanup

#### FD Exhaustion Tests
- **Expected**: `EMFILE` errors (error code 24), graceful degradation
- **Unexpected**: Process hangs, zombie processes, resource leaks

### Success Criteria
- All tests complete without crashes
- Resource exhaustion is detected and handled
- Recovery mechanisms work correctly
- No memory leaks after test completion
- Container cleanup is successful

## Performance Metrics

### Test Execution Times (approximate)
- OOM tests: 5-10 seconds
- Disk full tests: 10-20 seconds
- Stack overflow tests: 2-5 seconds
- FD exhaustion tests: 3-8 seconds
- Full suite: 30-60 seconds

### Resource Usage
- Container memory: 128-256MB
- Container disk: 100-500MB
- Container build time: 2-5 minutes

## Troubleshooting

### Docker Issues
```bash
# Check Docker is running
docker info

# Check container logs
docker logs <container_id>

# Remove old containers
docker ps -a | grep fastforth-destructive | awk '{print $1}' | xargs docker rm -f
```

### Test Failures
1. **Container won't start**: Check Docker daemon, rebuild image
2. **Tests skip**: Verify `DESTRUCTIVE_TESTS_ENABLED` environment variable
3. **Resource limits not applied**: Check Docker version supports resource limits
4. **Build failures**: Ensure Cargo.lock is up to date

### Common Errors

#### "SAFETY: Destructive tests can only run in containerized environments"
- **Cause**: Tests attempted to run on host system
- **Solution**: Use `./scripts/run_destructive_tests.sh` or Docker commands

#### "Failed to build container"
- **Cause**: Missing dependencies, outdated Dockerfile
- **Solution**: Update Dockerfile, check network connectivity

#### Tests hang indefinitely
- **Cause**: Resource limits too restrictive, deadlock
- **Solution**: Increase limits, add timeouts, check test logic

## Architecture

### Module Structure
```
tests/destructive/
├── Dockerfile              # Container configuration
├── README.md              # This file
├── mod.rs                 # Module root and exports
├── safety.rs              # Safety guards and container detection
├── test_oom.rs           # Out-of-memory tests
├── test_disk_full.rs     # Disk space exhaustion tests
├── test_stack_overflow.rs # Stack overflow tests
└── test_fd_exhaustion.rs # File descriptor tests
```

### Integration Points
- **Cargo features**: `destructive_tests` feature flag
- **CI/CD**: GitHub Actions, GitLab CI compatible
- **Safety layer**: Multi-tier protection against host execution
- **Resource limits**: Docker-native constraint enforcement

## Contributing

### Adding New Tests
1. Create test file in `tests/destructive/`
2. Use `#![cfg(feature = "destructive_tests")]` at module level
3. Add `ensure_containerized()` to each test
4. Update this README with test description
5. Add test category to `run_destructive_tests.sh`

### Best Practices
- Always use safety guards
- Document expected vs unexpected behaviors
- Test both failure and recovery paths
- Keep resource limits realistic
- Add logging for debugging
- Clean up resources in test teardown

## References

- [Rust Error Handling Best Practices](https://doc.rust-lang.org/book/ch09-00-error-handling.html)
- [Docker Resource Constraints](https://docs.docker.com/config/containers/resource_constraints/)
- [Unix Resource Limits](https://man7.org/linux/man-pages/man2/setrlimit.2.html)
- [FastForth Error Handling Design](/Users/joshkornreich/Documents/Projects/Ollama/llama/variants/fast-forth/docs/error_handling.md)

## License

MIT License - Same as parent project
