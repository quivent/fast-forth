/**
 * Unit Tests for Fast Forth Concurrency Primitives
 *
 * Compile with:
 *   gcc -I../runtime -pthread -o test_concurrency test_concurrency.c \
 *       ../runtime/concurrency.c ../runtime/forth_runtime.c -lpthread
 */

#include "../concurrency.h"
#include <stdio.h>
#include <assert.h>
#include <unistd.h>
#include <time.h>

// Test counters
static int tests_run = 0;
static int tests_passed = 0;

#define TEST(name) \
    printf("\n[TEST] %s...\n", #name); \
    tests_run++; \
    if (name())

#define PASS() \
    do { \
        tests_passed++; \
        printf("  ✅ PASS\n"); \
        return 1; \
    } while(0)

#define FAIL(msg) \
    do { \
        printf("  ❌ FAIL: %s\n", msg); \
        return 0; \
    } while(0)

#define ASSERT(cond, msg) \
    if (!(cond)) FAIL(msg)

// ============================================================================
// CHANNEL TESTS
// ============================================================================

int test_channel_create_destroy() {
    cell_t chan = forth_channel_create(10);
    ASSERT(chan != 0, "Channel creation failed");

    forth_channel_destroy(chan);
    PASS();
}

int test_channel_send_recv() {
    cell_t chan = forth_channel_create(10);

    // Send value
    forth_channel_send(42, chan);

    // Receive value
    cell_t value = forth_channel_recv(chan);
    ASSERT(value == 42, "Received wrong value");

    forth_channel_destroy(chan);
    PASS();
}

int test_channel_multiple_values() {
    cell_t chan = forth_channel_create(100);

    // Send 100 values
    for (int i = 0; i < 100; i++) {
        forth_channel_send(i, chan);
    }

    // Receive 100 values
    for (int i = 0; i < 100; i++) {
        cell_t value = forth_channel_recv(chan);
        ASSERT(value == i, "Values out of order");
    }

    forth_channel_destroy(chan);
    PASS();
}

int test_channel_fifo_order() {
    cell_t chan = forth_channel_create(5);

    // Send in order: 10, 20, 30
    forth_channel_send(10, chan);
    forth_channel_send(20, chan);
    forth_channel_send(30, chan);

    // Receive in same order
    ASSERT(forth_channel_recv(chan) == 10, "FIFO order violated (1)");
    ASSERT(forth_channel_recv(chan) == 20, "FIFO order violated (2)");
    ASSERT(forth_channel_recv(chan) == 30, "FIFO order violated (3)");

    forth_channel_destroy(chan);
    PASS();
}

int test_channel_close() {
    cell_t chan = forth_channel_create(10);

    // Send and close
    forth_channel_send(100, chan);
    forth_channel_close(chan);

    // Can still receive buffered value
    cell_t value = forth_channel_recv(chan);
    ASSERT(value == 100, "Should receive buffered value after close");

    // Next receive should return 0 (closed + empty)
    value = forth_channel_recv(chan);
    ASSERT(value == 0, "Closed empty channel should return 0");

    forth_channel_destroy(chan);
    PASS();
}

// ============================================================================
// THREAD TESTS
// ============================================================================

// Worker function for thread test
static void simple_worker(forth_vm_t* vm) {
    // Just push a value to stack
    push(vm, 999);
}

int test_spawn_join() {
    forth_vm_t* vm = forth_create();

    // Spawn thread
    cell_t thread = forth_spawn(vm, (cell_t)simple_worker);
    ASSERT(thread != 0, "Thread spawn failed");

    // Join thread
    forth_join(vm, thread);

    forth_destroy(vm);
    PASS();
}

// Worker that sends to channel
static cell_t test_channel_global;
static void channel_sender_worker(forth_vm_t* vm) {
    for (int i = 0; i < 10; i++) {
        forth_channel_send(i, test_channel_global);
    }
}

int test_thread_channel_communication() {
    forth_vm_t* vm = forth_create();

    // Create channel
    test_channel_global = forth_channel_create(10);

    // Spawn sender thread
    cell_t thread = forth_spawn(vm, (cell_t)channel_sender_worker);

    // Receive 10 values
    for (int i = 0; i < 10; i++) {
        cell_t value = forth_channel_recv(test_channel_global);
        ASSERT(value == i, "Thread communication failed");
    }

    // Wait for thread
    forth_join(vm, thread);

    forth_channel_destroy(test_channel_global);
    forth_destroy(vm);
    PASS();
}

// ============================================================================
// MULTI-THREAD TESTS
// ============================================================================

static void worker_increment(forth_vm_t* vm) {
    for (int i = 0; i < 100; i++) {
        forth_channel_send(1, test_channel_global);
    }
}

int test_multiple_threads() {
    forth_vm_t* vm = forth_create();

    // Create result channel
    test_channel_global = forth_channel_create(1000);

    // Spawn 10 threads
    cell_t threads[10];
    for (int i = 0; i < 10; i++) {
        threads[i] = forth_spawn(vm, (cell_t)worker_increment);
    }

    // Collect 1000 results (10 threads × 100 sends)
    int sum = 0;
    for (int i = 0; i < 1000; i++) {
        sum += forth_channel_recv(test_channel_global);
    }

    // Wait for all threads
    for (int i = 0; i < 10; i++) {
        forth_join(vm, threads[i]);
    }

    ASSERT(sum == 1000, "Lost messages in multi-thread test");

    forth_channel_destroy(test_channel_global);
    forth_destroy(vm);
    PASS();
}

// ============================================================================
// PERFORMANCE TESTS
// ============================================================================

int test_channel_throughput() {
    printf("  [PERF] Testing channel throughput...\n");

    cell_t chan = forth_channel_create(1000);

    struct timespec start, end;
    clock_gettime(CLOCK_MONOTONIC, &start);

    // Send 100,000 messages
    const int COUNT = 100000;
    for (int i = 0; i < COUNT; i++) {
        forth_channel_send(i, chan);
        forth_channel_recv(chan);
    }

    clock_gettime(CLOCK_MONOTONIC, &end);

    double elapsed = (end.tv_sec - start.tv_sec) +
                     (end.tv_nsec - start.tv_nsec) / 1e9;
    double ops_per_sec = COUNT / elapsed;

    printf("  Channel throughput: %.0f ops/sec (%.2f sec for %d ops)\n",
           ops_per_sec, elapsed, COUNT);

    forth_channel_destroy(chan);
    PASS();
}

int test_spawn_latency() {
    printf("  [PERF] Testing spawn latency...\n");

    forth_vm_t* vm = forth_create();

    struct timespec start, end;
    clock_gettime(CLOCK_MONOTONIC, &start);

    // Spawn 100 threads
    const int COUNT = 100;
    cell_t threads[COUNT];
    for (int i = 0; i < COUNT; i++) {
        threads[i] = forth_spawn(vm, (cell_t)simple_worker);
    }

    clock_gettime(CLOCK_MONOTONIC, &end);

    double elapsed = (end.tv_sec - start.tv_sec) +
                     (end.tv_nsec - start.tv_nsec) / 1e9;
    double avg_latency_us = (elapsed / COUNT) * 1e6;

    printf("  Spawn latency: %.1f μs average (%d spawns in %.3f sec)\n",
           avg_latency_us, COUNT, elapsed);

    // Join all
    for (int i = 0; i < COUNT; i++) {
        forth_join(vm, threads[i]);
    }

    forth_destroy(vm);
    PASS();
}

// ============================================================================
// STRESS TESTS
// ============================================================================

int test_channel_stress() {
    printf("  [STRESS] Testing channel under load...\n");

    cell_t chan = forth_channel_create(10);

    // Rapid send/recv
    for (int i = 0; i < 10000; i++) {
        forth_channel_send(i, chan);
        cell_t val = forth_channel_recv(chan);
        ASSERT(val == i, "Channel stress test failed");
    }

    forth_channel_destroy(chan);
    PASS();
}

// ============================================================================
// MAIN TEST RUNNER
// ============================================================================

int main() {
    printf("╔════════════════════════════════════════════════════════════╗\n");
    printf("║  Fast Forth Concurrency Primitives - Unit Tests           ║\n");
    printf("╚════════════════════════════════════════════════════════════╝\n");

    // Channel tests
    TEST(test_channel_create_destroy);
    TEST(test_channel_send_recv);
    TEST(test_channel_multiple_values);
    TEST(test_channel_fifo_order);
    TEST(test_channel_close);

    // Thread tests
    TEST(test_spawn_join);
    TEST(test_thread_channel_communication);
    TEST(test_multiple_threads);

    // Performance tests
    TEST(test_channel_throughput);
    TEST(test_spawn_latency);

    // Stress tests
    TEST(test_channel_stress);

    // Summary
    printf("\n");
    printf("╔════════════════════════════════════════════════════════════╗\n");
    printf("║  Test Results                                              ║\n");
    printf("╠════════════════════════════════════════════════════════════╣\n");
    printf("║  Tests run:    %-3d                                         ║\n", tests_run);
    printf("║  Tests passed: %-3d                                         ║\n", tests_passed);
    printf("║  Tests failed: %-3d                                         ║\n", tests_run - tests_passed);
    printf("║  Success rate: %.1f%%                                       ║\n",
           (tests_passed * 100.0) / tests_run);
    printf("╚════════════════════════════════════════════════════════════╝\n");

    return (tests_passed == tests_run) ? 0 : 1;
}
