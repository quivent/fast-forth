# Destructive Test Inventory

Complete listing of all destructive tests with descriptions and expected behaviors.

## Test Categories

### 1. Out-of-Memory (OOM) Tests
**File**: `test_oom.rs` (215 lines)
**Total Tests**: 7

#### test_small_allocation_failure
- **Purpose**: Test raw allocator failure handling
- **Method**: Allocate 1MB chunks until OOM
- **Expected**: Graceful NULL pointer handling
- **Validates**: Raw `alloc()`/`dealloc()` error paths

#### test_vec_allocation_failure
- **Purpose**: Test Vec allocation with try_reserve
- **Method**: Allocate 10M element vectors until failure
- **Expected**: `Err` return from `try_reserve`
- **Validates**: Vec::try_reserve error handling

#### test_string_allocation_failure
- **Purpose**: Test String allocation failures
- **Method**: Allocate 1MB strings until OOM
- **Expected**: `Err` from `try_reserve`, no panic
- **Validates**: String memory management

#### test_boxed_allocation_failure
- **Purpose**: Test Box allocation for large arrays
- **Method**: Box::new([0u8; 10_000_000]) until failure
- **Expected**: Caught panic or allocation failure
- **Validates**: Box allocation error handling

#### test_oom_recovery
- **Purpose**: Test recovery after OOM
- **Method**: Allocate until failure, free all, allocate again
- **Expected**: Successful allocation after recovery
- **Validates**: Memory recovery mechanisms

#### test_fastforth_oom_handling
- **Purpose**: Test FastForth compiler OOM scenarios
- **Method**: Simulate compiler buffer allocations
- **Expected**: Graceful degradation
- **Validates**: Compiler-specific OOM paths

#### Safety Verification Test
- **Purpose**: Verify safety guards work
- **Method**: Check is_safe_to_run_destructive_tests()
- **Expected**: Always passes
- **Validates**: Safety infrastructure

---

### 2. Disk Full Tests
**File**: `test_disk_full.rs` (269 lines)
**Total Tests**: 6

#### test_disk_full_write_handling
- **Purpose**: Test write operations until ENOSPC
- **Method**: Write 1MB chunks until disk full
- **Expected**: io::Error with ENOSPC
- **Validates**: Disk full error detection

#### test_disk_full_append_handling
- **Purpose**: Test append operations under disk pressure
- **Method**: Append to file until disk full
- **Expected**: Write error, error kind detection
- **Validates**: Append operation error handling

#### test_disk_full_temp_file_handling
- **Purpose**: Test temp file creation when disk full
- **Method**: Create 512KB temp files until failure
- **Expected**: File creation or write failure
- **Validates**: Temporary file error paths

#### test_disk_full_recovery
- **Purpose**: Test recovery after freeing space
- **Method**: Fill disk, delete half files, write again
- **Expected**: Successful write after cleanup
- **Validates**: Disk space recovery

#### test_disk_full_compilation
- **Purpose**: Test compilation output under disk pressure
- **Method**: Simulate compiler output files
- **Expected**: Graceful handling of write failures
- **Validates**: Compiler output error handling

#### test_disk_space_monitoring
- **Purpose**: Test disk space awareness
- **Method**: Use get_available_disk_space()
- **Expected**: Accurate space reporting
- **Validates**: Disk monitoring utilities

---

### 3. Stack Overflow Tests
**File**: `test_stack_overflow.rs` (242 lines)
**Total Tests**: 6

#### test_deep_recursion_handling
- **Purpose**: Test deep recursion with stack limit
- **Method**: Recursive calls with 1KB local arrays
- **Expected**: Caught panic or completion
- **Validates**: Stack overflow detection

#### test_mutual_recursion_overflow
- **Purpose**: Test mutual recursion patterns
- **Method**: Functions A and B call each other
- **Expected**: Stack overflow caught
- **Validates**: Mutual recursion handling

#### test_large_stack_frames
- **Purpose**: Test large local variable allocation
- **Method**: Recursive calls with 16KB arrays
- **Expected**: Stack overflow caught sooner
- **Validates**: Large frame handling

#### test_recursive_data_structures
- **Purpose**: Test deep linked list traversal
- **Method**: Build 1000-node list, traverse recursively
- **Expected**: Overflow caught, iterative works
- **Validates**: Data structure traversal

#### test_forth_stack_overflow
- **Purpose**: Test Forth-style stack operations
- **Method**: Deep call stack simulation
- **Expected**: Stack overflow with recovery
- **Validates**: Forth VM stack handling

#### test_compiler_recursion_limits
- **Purpose**: Test compiler AST processing depth
- **Method**: Simulate deeply nested AST processing
- **Expected**: Recursion limit detection
- **Validates**: Compiler recursion safety

---

### 4. File Descriptor Exhaustion Tests
**File**: `test_fd_exhaustion.rs` (231 lines)
**Total Tests**: 6

#### test_fd_exhaustion_handling
- **Purpose**: Test FD limit handling
- **Method**: Open /dev/null until EMFILE
- **Expected**: Error with errno 24 (EMFILE)
- **Validates**: FD limit detection

#### test_fd_recovery
- **Purpose**: Test recovery after closing FDs
- **Method**: Open many files, close all, open again
- **Expected**: Successful open after recovery
- **Validates**: FD recovery mechanisms

#### test_fd_leak_detection
- **Purpose**: Test FD leak detection
- **Method**: Compare FD count before/after operations
- **Expected**: No significant FD difference
- **Validates**: FD leak prevention

#### test_simultaneous_file_operations
- **Purpose**: Test multiple simultaneous file ops
- **Method**: Create/open many files simultaneously
- **Expected**: Graceful handling of limits
- **Validates**: Concurrent file operations

#### test_compiler_fd_usage
- **Purpose**: Test compiler file descriptor patterns
- **Method**: Simulate source/temp/output files
- **Expected**: Proper FD management
- **Validates**: Compiler FD usage

#### test_fd_limit_awareness
- **Purpose**: Test FD limit detection
- **Method**: Query ulimit, open safe number
- **Expected**: Limit-aware operation
- **Validates**: FD limit querying

---

## Test Execution Matrix

### Resource Limits by Test Category

| Test Category | Memory Limit | Disk Limit | Stack Limit | FD Limit | Duration |
|---------------|--------------|------------|-------------|----------|----------|
| OOM | 128MB | - | - | - | 5-10s |
| Disk Full | - | 100MB | - | - | 10-20s |
| Stack Overflow | - | - | 1MB | - | 2-5s |
| FD Exhaustion | - | - | - | 256 | 3-8s |
| **All Tests** | 256MB | - | 1MB | 512 | 30-60s |

### Expected Behaviors by Test

| Test | Success Indicator | Failure Indicator |
|------|-------------------|-------------------|
| OOM Tests | Graceful Err returns | Segfaults, panics |
| Disk Tests | io::Error detected | Silent data loss |
| Stack Tests | Caught panics | Unhandled segfaults |
| FD Tests | EMFILE detection | Process hangs |

## Test Coverage Map

### Error Paths Validated

```
Out-of-Memory:
  ✓ Vec::try_reserve
  ✓ String::try_reserve
  ✓ Box allocation
  ✓ Raw allocator
  ✓ Recovery mechanisms

Disk Full:
  ✓ File::create
  ✓ File::write_all
  ✓ OpenOptions::append
  ✓ Space recovery
  ✓ ENOSPC detection

Stack Overflow:
  ✓ Deep recursion
  ✓ Mutual recursion
  ✓ Large frames
  ✓ catch_unwind
  ✓ Recovery

File Descriptors:
  ✓ File::open
  ✓ EMFILE detection
  ✓ FD leak prevention
  ✓ Recovery
  ✓ Limit awareness
```

### FastForth Integration Points

```
Compiler:
  ✓ Memory allocation (OOM tests)
  ✓ Output file generation (Disk tests)
  ✓ AST processing (Stack tests)
  ✓ Source file handling (FD tests)

Runtime:
  ✓ Buffer management (OOM tests)
  ✓ Temp files (Disk tests)
  ✓ Call stack (Stack tests)
  ✓ File I/O (FD tests)

Parser:
  ✓ Token buffers (OOM tests)
  ✓ Expression depth (Stack tests)
```

## Running Individual Tests

### OOM Tests
```bash
docker run --rm --memory=128m --memory-swap=128m \
    --env DESTRUCTIVE_TESTS_ENABLED=1 --env ALLOW_DESTRUCTIVE_TESTS=1 \
    fastforth-destructive-tests \
    cargo test --release --features destructive_tests test_small_allocation_failure -- --nocapture
```

### Disk Full Tests
```bash
docker run --rm \
    --env DESTRUCTIVE_TESTS_ENABLED=1 --env ALLOW_DESTRUCTIVE_TESTS=1 \
    fastforth-destructive-tests \
    cargo test --release --features destructive_tests test_disk_full_write_handling -- --nocapture
```

### Stack Overflow Tests
```bash
docker run --rm --ulimit stack=1048576:1048576 \
    --env DESTRUCTIVE_TESTS_ENABLED=1 --env ALLOW_DESTRUCTIVE_TESTS=1 \
    fastforth-destructive-tests \
    cargo test --release --features destructive_tests test_deep_recursion_handling -- --nocapture
```

### FD Exhaustion Tests
```bash
docker run --rm --ulimit nofile=256:256 \
    --env DESTRUCTIVE_TESTS_ENABLED=1 --env ALLOW_DESTRUCTIVE_TESTS=1 \
    fastforth-destructive-tests \
    cargo test --release --features destructive_tests test_fd_exhaustion_handling -- --nocapture
```

## Test Development Guidelines

### Adding New Tests

1. **Choose appropriate category** (OOM, Disk, Stack, FD)
2. **Add test function**:
   ```rust
   #[test]
   fn test_new_scenario() {
       ensure_containerized();
       // Test implementation
   }
   ```
3. **Document expected behavior** in comments
4. **Update this inventory**
5. **Run locally** with test runner script
6. **Submit PR** (triggers CI)

### Test Naming Convention

- `test_<resource>_<scenario>`
- Examples:
  - `test_oom_recovery`
  - `test_disk_full_append`
  - `test_stack_overflow_mutual`
  - `test_fd_leak_detection`

### Test Structure Template

```rust
#[test]
fn test_<name>() {
    ensure_containerized();

    println!("Testing <scenario>...");

    // Setup
    let mut resources = Vec::new();

    // Execute until failure
    for i in 0..MAX_ITERATIONS {
        match try_operation() {
            Ok(r) => resources.push(r),
            Err(e) => {
                println!("Expected failure at iteration {}: {:?}", i, e);
                break;
            }
        }
    }

    // Verify graceful handling
    assert!(resources.len() > 0, "Should have some success before failure");

    // Cleanup
    drop(resources);

    println!("Test completed successfully");
}
```

## Test Maintenance

### Regular Tasks

- **Weekly**: Review CI results
- **Monthly**: Check for new error scenarios
- **Quarterly**: Update resource limits if needed
- **Annually**: Review test coverage

### Updating Tests

1. Modify test file
2. Update this inventory
3. Update README.md if behavior changes
4. Test locally: `./scripts/run_destructive_tests.sh`
5. Commit and push (triggers CI)

## References

- **Main Documentation**: `README.md`
- **Quick Reference**: `QUICKREF.md`
- **Implementation**: `IMPLEMENTATION_COMPLETE.md`
- **CI Workflow**: `../.github/workflows/destructive-tests.yml`
- **Test Runner**: `../../scripts/run_destructive_tests.sh`

---

**Total Tests**: 25
**Last Updated**: 2025-11-15
**Status**: Production Ready
