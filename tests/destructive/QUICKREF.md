# Destructive Tests Quick Reference

## Instant Usage

```bash
# Run all destructive tests in Docker (safest)
./scripts/run_destructive_tests.sh

# Run specific test category
./scripts/run_destructive_tests.sh oom      # Out-of-memory tests
./scripts/run_destructive_tests.sh disk     # Disk full tests
./scripts/run_destructive_tests.sh stack    # Stack overflow tests
./scripts/run_destructive_tests.sh fd       # File descriptor tests
```

## What Gets Tested

| Category | Tests | Container Limit | Expected Behavior |
|----------|-------|-----------------|-------------------|
| **OOM** | 7 | 128MB RAM | Graceful allocation failures |
| **Disk** | 6 | 100MB disk | ENOSPC error handling |
| **Stack** | 6 | 1MB stack | Caught panics, recovery |
| **FD** | 6 | 256 FDs | EMFILE handling |
| **Total** | **25** | Combined | All error paths validated |

## Safety Guarantees

- Multiple safety layers prevent host system damage
- Container-only execution enforced
- Resource limits prevent runaway tests
- Automatic cleanup on exit
- Will NOT run on host system (panics with clear error)

## CI Integration

Tests run automatically:
- Weekly on Sunday 2 AM UTC
- On manual trigger via GitHub Actions
- On PR modifying destructive test files

## Manual Docker Commands

### Build Container
```bash
docker build -t fastforth-destructive-tests -f tests/destructive/Dockerfile .
```

### Run OOM Tests
```bash
docker run --rm --memory=128m --memory-swap=128m \
    --env DESTRUCTIVE_TESTS_ENABLED=1 --env ALLOW_DESTRUCTIVE_TESTS=1 \
    fastforth-destructive-tests \
    cargo test --release --features destructive_tests test_oom -- --nocapture
```

### Run Disk Full Tests
```bash
docker run --rm \
    --env DESTRUCTIVE_TESTS_ENABLED=1 --env ALLOW_DESTRUCTIVE_TESTS=1 \
    fastforth-destructive-tests \
    cargo test --release --features destructive_tests test_disk_full -- --nocapture
```

### Run Stack Overflow Tests
```bash
docker run --rm --ulimit stack=1048576:1048576 \
    --env DESTRUCTIVE_TESTS_ENABLED=1 --env ALLOW_DESTRUCTIVE_TESTS=1 \
    fastforth-destructive-tests \
    cargo test --release --features destructive_tests test_stack_overflow -- --nocapture
```

### Run FD Exhaustion Tests
```bash
docker run --rm --ulimit nofile=256:256 \
    --env DESTRUCTIVE_TESTS_ENABLED=1 --env ALLOW_DESTRUCTIVE_TESTS=1 \
    fastforth-destructive-tests \
    cargo test --release --features destructive_tests test_fd_exhaustion -- --nocapture
```

### Run All Tests
```bash
docker run --rm \
    --memory=256m --memory-swap=256m \
    --ulimit stack=1048576:1048576 \
    --ulimit nofile=512:512 \
    --env DESTRUCTIVE_TESTS_ENABLED=1 --env ALLOW_DESTRUCTIVE_TESTS=1 \
    fastforth-destructive-tests \
    cargo test --release --features destructive_tests -- --test-threads=1 --nocapture
```

## Expected Timing

- OOM tests: 5-10 seconds
- Disk full tests: 10-20 seconds
- Stack overflow tests: 2-5 seconds
- FD exhaustion tests: 3-8 seconds
- **Full suite: 30-60 seconds**

## Troubleshooting

### "SAFETY: Destructive tests can only run in containerized environments"
**Solution**: Use `./scripts/run_destructive_tests.sh` or Docker commands above

### Tests skip/pass without running
**Solution**: Ensure `--features destructive_tests` flag is present

### Docker build fails
**Solution**: Check Docker is running (`docker info`), rebuild with `--no-cache`

### Resource limits not applied
**Solution**: Update Docker version, check `docker info` for capabilities

## Test Details

### OOM Tests (`test_oom.rs`)
1. `test_small_allocation_failure` - Raw allocator failures
2. `test_vec_allocation_failure` - Vector try_reserve
3. `test_string_allocation_failure` - String allocation
4. `test_boxed_allocation_failure` - Box allocation
5. `test_oom_recovery` - Recovery validation
6. `test_fastforth_oom_handling` - Compiler scenarios
7. Safety verification test

### Disk Full Tests (`test_disk_full.rs`)
1. `test_disk_full_write_handling` - Write until ENOSPC
2. `test_disk_full_append_handling` - Append operations
3. `test_disk_full_temp_file_handling` - Temp files
4. `test_disk_full_recovery` - Space recovery
5. `test_disk_full_compilation` - Compiler output
6. Space monitoring test

### Stack Overflow Tests (`test_stack_overflow.rs`)
1. `test_deep_recursion_handling` - Deep recursion
2. `test_mutual_recursion_overflow` - Mutual recursion
3. `test_large_stack_frames` - Large locals
4. `test_recursive_data_structures` - Linked lists
5. `test_forth_stack_overflow` - Forth patterns
6. `test_compiler_recursion_limits` - AST processing

### FD Exhaustion Tests (`test_fd_exhaustion.rs`)
1. `test_fd_exhaustion_handling` - Open until EMFILE
2. `test_fd_recovery` - FD recovery
3. `test_fd_leak_detection` - Leak detection
4. `test_simultaneous_file_operations` - Multiple files
5. `test_compiler_fd_usage` - Compiler patterns
6. `test_fd_limit_awareness` - Limit detection

## Files Structure

```
tests/destructive/
├── Dockerfile                 # Container config
├── README.md                  # Full documentation
├── QUICKREF.md               # This file
├── .dockerignore             # Build optimization
├── mod.rs                    # Module root
├── safety.rs                 # Safety guards
├── test_oom.rs              # 7 OOM tests
├── test_disk_full.rs        # 6 disk tests
├── test_stack_overflow.rs   # 6 stack tests
└── test_fd_exhaustion.rs    # 6 FD tests

scripts/
└── run_destructive_tests.sh  # Automated runner

.github/workflows/
└── destructive-tests.yml     # CI integration
```

## Environment Variables

- `DESTRUCTIVE_TESTS_ENABLED=1` - Enable tests (container check)
- `ALLOW_DESTRUCTIVE_TESTS=1` - Explicit permission
- `RUST_BACKTRACE=1` - Full backtraces on panic

## Cargo Feature

Add to test command:
```bash
cargo test --features destructive_tests
```

Or in Cargo.toml:
```toml
[features]
destructive_tests = []
```

## Contributing

1. Add tests to appropriate `test_*.rs` file
2. Use `ensure_containerized()` guard
3. Document expected behavior
4. Update README.md
5. Test locally with runner script
6. Submit PR

## Documentation

- Full docs: `tests/destructive/README.md`
- Implementation: `DESTRUCTIVE_TESTING_INFRASTRUCTURE.md`
- CI config: `.github/workflows/destructive-tests.yml`

---

**Remember**: These tests are designed to FAIL in controlled ways to validate error handling. The goal is graceful degradation, not preventing failures.
