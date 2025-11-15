# Fast Forth Concurrency Tests

This directory contains comprehensive tests for the concurrency primitives.

## Test Structure

### 1. Unit Tests (C)
**File**: `runtime/tests/test_concurrency.c`

**Purpose**: Test individual primitives in isolation

**Tests**:
- Channel creation/destruction
- Send/recv operations
- FIFO ordering
- Channel close semantics
- Thread spawn/join
- Thread-channel communication
- Multi-thread scenarios
- Performance benchmarks
- Stress tests

**Build**:
```bash
cd runtime/tests
make test
```

**Expected Output**:
```
╔════════════════════════════════════════════════════════════╗
║  Fast Forth Concurrency Primitives - Unit Tests           ║
╚════════════════════════════════════════════════════════════╝

[TEST] test_channel_create_destroy...
  ✅ PASS

[TEST] test_channel_send_recv...
  ✅ PASS

...

╔════════════════════════════════════════════════════════════╗
║  Test Results                                              ║
╠════════════════════════════════════════════════════════════╣
║  Tests run:    11                                          ║
║  Tests passed: 11                                          ║
║  Tests failed: 0                                           ║
║  Success rate: 100.0%                                      ║
╚════════════════════════════════════════════════════════════╝
```

**Performance Tests**:
- Channel throughput: Expected ~100K-500K ops/sec
- Spawn latency: Expected ~50 μs average
- Channel send/recv: Expected ~50 ns unlocked, ~500 ns contended

### 2. Integration Tests (Forth)
**File**: `tests/concurrency_integration_test.forth`

**Purpose**: Test concurrency from Forth code (end-to-end)

**Tests**:
1. Basic channel send/recv
2. Channel FIFO order
3. Thread spawn and join
4. Thread communication via channels
5. Multi-agent pattern (10 agents)
6. Channel capacity handling
7. Pipeline pattern (3 stages)
8. Stress test (1000 messages)

**Run**:
```bash
fastforth run tests/concurrency_integration_test.forth
```

**Expected Output**:
```
╔════════════════════════════════════════════╗
║  Fast Forth Concurrency - Integration Tests   ║
╚════════════════════════════════════════════╝

[TEST] Basic channel send/recv...
✅ PASS

[TEST] Channel FIFO order...
✅ PASS

...

========================================
TEST SUMMARY
========================================
Tests run:    8
Tests passed: 8
Tests failed: 0
Success rate: 100%
========================================
```

### 3. Benchmarks (Forth)
**File**: `benchmarks/concurrency_bench.forth`

**Purpose**: Measure performance characteristics

**Benchmarks**:
1. Channel send/recv latency
2. Thread spawn latency
3. Multi-agent throughput (10 agents, 1000 specs)
4. Pipeline throughput (3 stages, 1000 items)
5. Channel contention (10 concurrent writers)
6. Memory overhead estimation
7. Scalability tests (1, 5, 10, 20 agents)

**Run**:
```bash
fastforth run benchmarks/concurrency_bench.forth
```

**Expected Output**:
```
╔════════════════════════════════════════════════╗
║  Fast Forth Concurrency - Performance Bench   ║
╚════════════════════════════════════════════════╝

[BENCH] Channel send/recv latency...
Benchmark: channel send/recv
Total time: 50 ms
Operations: 10000
Average:    0.005 ms/op
Throughput: 200000 ops/sec

[BENCH] Thread spawn latency...
Benchmark: thread spawn
Total time: 5000 ms
Operations: 100
Average:    50 ms/op
Throughput: 2 ops/sec

...
```

## Running All Tests

### Quick Test
```bash
# Run unit tests only
cd runtime/tests
make test
```

### Full Test Suite
```bash
# 1. Unit tests (C)
cd runtime/tests
make test

# 2. Integration tests (Forth)
cd ../..
fastforth run tests/concurrency_integration_test.forth

# 3. Benchmarks (Forth)
fastforth run benchmarks/concurrency_bench.forth
```

### Memory Leak Detection
```bash
cd runtime/tests
make valgrind
```

**Expected**: No memory leaks detected

### Thread Safety Testing
```bash
cd runtime/tests
make tsan
```

**Expected**: No data races detected

## Test Coverage

### Primitives Tested
- ✅ `spawn` - Thread creation
- ✅ `join` - Thread synchronization
- ✅ `channel` - Queue creation
- ✅ `send` - Blocking send
- ✅ `recv` - Blocking receive
- ✅ `close-channel` - Graceful shutdown
- ✅ `destroy-channel` - Resource cleanup

### Patterns Tested
- ✅ Single producer, single consumer
- ✅ Multiple producers, single consumer
- ✅ Single producer, multiple consumers
- ✅ Multiple producers, multiple consumers
- ✅ Pipeline (multi-stage)
- ✅ Worker pool (multi-agent)

### Edge Cases Tested
- ✅ Empty channel recv (blocks)
- ✅ Full channel send (blocks)
- ✅ Channel close with buffered messages
- ✅ Multiple threads joining
- ✅ FIFO ordering under load
- ✅ Thread-local VM isolation

### Performance Metrics
- ✅ Channel latency (send/recv)
- ✅ Thread spawn latency
- ✅ Throughput (ops/sec)
- ✅ Scalability (1-20 agents)
- ✅ Memory overhead
- ✅ Contention handling

## Expected Results

### Unit Tests (C)
- **11/11 tests pass**
- **Channel throughput**: 100K-500K ops/sec
- **Spawn latency**: ~50 μs
- **No memory leaks**
- **No data races**

### Integration Tests (Forth)
- **8/8 tests pass**
- **Multi-agent**: 10 agents process 100 specs
- **Pipeline**: 3 stages process 1000 items
- **Stress**: 1000 messages pass through channel

### Benchmarks (Forth)
- **Channel send/recv**: 200K+ ops/sec
- **Thread spawn**: ~50 ms/spawn (pthread overhead)
- **Multi-agent**: ~100 specs in ~10 seconds (10x speedup)
- **Pipeline**: ~1000 items in ~2 seconds
- **Scalability**: Linear up to 10 agents, diminishing returns after

## Troubleshooting

### Test Fails: "Channel creation failed"
- **Cause**: malloc failure (OOM)
- **Fix**: Reduce channel capacity or close system apps

### Test Fails: "Received wrong value"
- **Cause**: FIFO ordering violation
- **Fix**: Check for race conditions in channel implementation

### Test Hangs
- **Cause**: Deadlock (sender waiting on full channel, no receiver)
- **Fix**: Ensure receivers drain channels before shutdown

### Memory Leak Detected
- **Cause**: Missing `destroy-channel` or `join`
- **Fix**: Always pair `channel` with `destroy-channel`, `spawn` with `join`

### Data Race Detected
- **Cause**: Shared state without synchronization
- **Fix**: Use channels for all inter-thread communication

## Performance Optimization

If benchmarks show poor performance:

1. **Check contention**: `make bench` and look for mutex waits
2. **Increase buffer size**: Larger channels reduce blocking
3. **Use pipelines**: Split work across stages
4. **Profile**: `perf record ./test_concurrency`

## Continuous Integration

Add to CI pipeline:
```yaml
- name: Test Concurrency
  run: |
    cd runtime/tests
    make test
    make valgrind
    make tsan
```

## Future Tests

**TODO**:
- [ ] Benchmark against Go orchestrator
- [ ] Test on Windows (if pthread compat added)
- [ ] Fuzz testing for edge cases
- [ ] Load testing (1000+ agents)
- [ ] Latency distribution (p50, p95, p99)

## References

- [CONCURRENCY_IMPLEMENTATION_GUIDE.md](../CONCURRENCY_IMPLEMENTATION_GUIDE.md)
- [examples/forth_multi_agent.forth](../examples/forth_multi_agent.forth)
- [runtime/concurrency.h](../runtime/concurrency.h)
