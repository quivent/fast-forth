/**
 * Fast Forth Runtime Test Suite
 * Stream 6: Comprehensive tests for all primitives and features
 */

#include "../runtime/forth_runtime.h"
#include <stdio.h>
#include <stdlib.h>
#include <assert.h>
#include <string.h>

// Test framework
static int tests_passed = 0;
static int tests_failed = 0;

#define TEST(name) \
    static void test_##name(void); \
    static void run_##name(void) { \
        printf("Running test: %s...", #name); \
        fflush(stdout); \
        test_##name(); \
        tests_passed++; \
        printf(" PASSED\n"); \
    } \
    static void test_##name(void)

#define ASSERT_EQUAL(a, b) \
    do { \
        if ((a) != (b)) { \
            printf("\n  FAILED: %s:%d: %ld != %ld\n", __FILE__, __LINE__, (long)(a), (long)(b)); \
            tests_failed++; \
            return; \
        } \
    } while(0)

#define ASSERT_TRUE(expr) \
    do { \
        if (!(expr)) { \
            printf("\n  FAILED: %s:%d: %s is false\n", __FILE__, __LINE__, #expr); \
            tests_failed++; \
            return; \
        } \
    } while(0)

// ============================================================================
// ARITHMETIC TESTS
// ============================================================================

TEST(add) {
    forth_vm_t *vm = forth_create();
    push(vm, 5);
    push(vm, 3);
    forth_add(vm);
    ASSERT_EQUAL(pop(vm), 8);
    forth_destroy(vm);
}

TEST(sub) {
    forth_vm_t *vm = forth_create();
    push(vm, 10);
    push(vm, 3);
    forth_sub(vm);
    ASSERT_EQUAL(pop(vm), 7);
    forth_destroy(vm);
}

TEST(mul) {
    forth_vm_t *vm = forth_create();
    push(vm, 6);
    push(vm, 7);
    forth_mul(vm);
    ASSERT_EQUAL(pop(vm), 42);
    forth_destroy(vm);
}

TEST(div) {
    forth_vm_t *vm = forth_create();
    push(vm, 20);
    push(vm, 4);
    forth_div(vm);
    ASSERT_EQUAL(pop(vm), 5);
    forth_destroy(vm);
}

TEST(mod) {
    forth_vm_t *vm = forth_create();
    push(vm, 17);
    push(vm, 5);
    forth_mod(vm);
    ASSERT_EQUAL(pop(vm), 2);
    forth_destroy(vm);
}

TEST(divmod) {
    forth_vm_t *vm = forth_create();
    push(vm, 17);
    push(vm, 5);
    forth_divmod(vm);
    ASSERT_EQUAL(pop(vm), 3);  // quotient
    ASSERT_EQUAL(pop(vm), 2);  // remainder
    forth_destroy(vm);
}

TEST(negate) {
    forth_vm_t *vm = forth_create();
    push(vm, 42);
    forth_negate(vm);
    ASSERT_EQUAL(pop(vm), -42);
    forth_destroy(vm);
}

TEST(abs) {
    forth_vm_t *vm = forth_create();
    push(vm, -42);
    forth_abs(vm);
    ASSERT_EQUAL(pop(vm), 42);
    forth_destroy(vm);
}

TEST(min) {
    forth_vm_t *vm = forth_create();
    push(vm, 5);
    push(vm, 3);
    forth_min(vm);
    ASSERT_EQUAL(pop(vm), 3);
    forth_destroy(vm);
}

TEST(max) {
    forth_vm_t *vm = forth_create();
    push(vm, 5);
    push(vm, 3);
    forth_max(vm);
    ASSERT_EQUAL(pop(vm), 5);
    forth_destroy(vm);
}

// ============================================================================
// STACK MANIPULATION TESTS
// ============================================================================

TEST(dup) {
    forth_vm_t *vm = forth_create();
    push(vm, 42);
    forth_dup(vm);
    ASSERT_EQUAL(depth(vm), 2);
    ASSERT_EQUAL(pop(vm), 42);
    ASSERT_EQUAL(pop(vm), 42);
    forth_destroy(vm);
}

TEST(drop) {
    forth_vm_t *vm = forth_create();
    push(vm, 1);
    push(vm, 2);
    forth_drop(vm);
    ASSERT_EQUAL(depth(vm), 1);
    ASSERT_EQUAL(pop(vm), 1);
    forth_destroy(vm);
}

TEST(swap) {
    forth_vm_t *vm = forth_create();
    push(vm, 1);
    push(vm, 2);
    forth_swap(vm);
    ASSERT_EQUAL(pop(vm), 1);
    ASSERT_EQUAL(pop(vm), 2);
    forth_destroy(vm);
}

TEST(over) {
    forth_vm_t *vm = forth_create();
    push(vm, 1);
    push(vm, 2);
    forth_over(vm);
    ASSERT_EQUAL(depth(vm), 3);
    ASSERT_EQUAL(pop(vm), 1);
    ASSERT_EQUAL(pop(vm), 2);
    ASSERT_EQUAL(pop(vm), 1);
    forth_destroy(vm);
}

TEST(rot) {
    forth_vm_t *vm = forth_create();
    push(vm, 1);
    push(vm, 2);
    push(vm, 3);
    forth_rot(vm);
    ASSERT_EQUAL(pop(vm), 1);
    ASSERT_EQUAL(pop(vm), 3);
    ASSERT_EQUAL(pop(vm), 2);
    forth_destroy(vm);
}

TEST(nrot) {
    forth_vm_t *vm = forth_create();
    push(vm, 1);
    push(vm, 2);
    push(vm, 3);
    forth_nrot(vm);
    ASSERT_EQUAL(pop(vm), 2);
    ASSERT_EQUAL(pop(vm), 1);
    ASSERT_EQUAL(pop(vm), 3);
    forth_destroy(vm);
}

TEST(tuck) {
    forth_vm_t *vm = forth_create();
    push(vm, 1);
    push(vm, 2);
    forth_tuck(vm);
    ASSERT_EQUAL(depth(vm), 3);
    ASSERT_EQUAL(pop(vm), 2);
    ASSERT_EQUAL(pop(vm), 1);
    ASSERT_EQUAL(pop(vm), 2);
    forth_destroy(vm);
}

TEST(2dup) {
    forth_vm_t *vm = forth_create();
    push(vm, 1);
    push(vm, 2);
    forth_2dup(vm);
    ASSERT_EQUAL(depth(vm), 4);
    ASSERT_EQUAL(pop(vm), 2);
    ASSERT_EQUAL(pop(vm), 1);
    ASSERT_EQUAL(pop(vm), 2);
    ASSERT_EQUAL(pop(vm), 1);
    forth_destroy(vm);
}

// ============================================================================
// LOGICAL OPERATION TESTS
// ============================================================================

TEST(and) {
    forth_vm_t *vm = forth_create();
    push(vm, 0xFF);
    push(vm, 0x0F);
    forth_and(vm);
    ASSERT_EQUAL(pop(vm), 0x0F);
    forth_destroy(vm);
}

TEST(or) {
    forth_vm_t *vm = forth_create();
    push(vm, 0xF0);
    push(vm, 0x0F);
    forth_or(vm);
    ASSERT_EQUAL(pop(vm), 0xFF);
    forth_destroy(vm);
}

TEST(xor) {
    forth_vm_t *vm = forth_create();
    push(vm, 0xFF);
    push(vm, 0x0F);
    forth_xor(vm);
    ASSERT_EQUAL(pop(vm), 0xF0);
    forth_destroy(vm);
}

TEST(invert) {
    forth_vm_t *vm = forth_create();
    push(vm, 0);
    forth_invert(vm);
    ASSERT_EQUAL(pop(vm), -1);
    forth_destroy(vm);
}

TEST(lshift) {
    forth_vm_t *vm = forth_create();
    push(vm, 1);
    push(vm, 3);
    forth_lshift(vm);
    ASSERT_EQUAL(pop(vm), 8);
    forth_destroy(vm);
}

TEST(rshift) {
    forth_vm_t *vm = forth_create();
    push(vm, 16);
    push(vm, 2);
    forth_rshift(vm);
    ASSERT_EQUAL(pop(vm), 4);
    forth_destroy(vm);
}

// ============================================================================
// COMPARISON TESTS
// ============================================================================

TEST(eq) {
    forth_vm_t *vm = forth_create();
    push(vm, 5);
    push(vm, 5);
    forth_eq(vm);
    ASSERT_EQUAL(pop(vm), -1);  // Forth true is -1

    push(vm, 5);
    push(vm, 3);
    forth_eq(vm);
    ASSERT_EQUAL(pop(vm), 0);
    forth_destroy(vm);
}

TEST(lt) {
    forth_vm_t *vm = forth_create();
    push(vm, 3);
    push(vm, 5);
    forth_lt(vm);
    ASSERT_EQUAL(pop(vm), -1);

    push(vm, 5);
    push(vm, 3);
    forth_lt(vm);
    ASSERT_EQUAL(pop(vm), 0);
    forth_destroy(vm);
}

TEST(gt) {
    forth_vm_t *vm = forth_create();
    push(vm, 5);
    push(vm, 3);
    forth_gt(vm);
    ASSERT_EQUAL(pop(vm), -1);

    push(vm, 3);
    push(vm, 5);
    forth_gt(vm);
    ASSERT_EQUAL(pop(vm), 0);
    forth_destroy(vm);
}

// ============================================================================
// MEMORY OPERATION TESTS
// ============================================================================

TEST(fetch_store) {
    forth_vm_t *vm = forth_create();
    cell_t value = 42;
    cell_t *addr = &value;

    push(vm, (cell_t)addr);
    forth_fetch(vm);
    ASSERT_EQUAL(pop(vm), 42);

    push(vm, 99);
    push(vm, (cell_t)addr);
    forth_store(vm);
    ASSERT_EQUAL(value, 99);

    forth_destroy(vm);
}

TEST(cfetch_cstore) {
    forth_vm_t *vm = forth_create();
    byte_t buffer[10] = {0};

    push(vm, 65);  // 'A'
    push(vm, (cell_t)buffer);
    forth_cstore(vm);

    push(vm, (cell_t)buffer);
    forth_cfetch(vm);
    ASSERT_EQUAL(pop(vm), 65);

    forth_destroy(vm);
}

// ============================================================================
// RETURN STACK TESTS
// ============================================================================

TEST(return_stack) {
    forth_vm_t *vm = forth_create();

    push(vm, 42);
    forth_tor(vm);
    ASSERT_EQUAL(depth(vm), 0);
    ASSERT_EQUAL(rdepth(vm), 1);

    forth_fromr(vm);
    ASSERT_EQUAL(depth(vm), 1);
    ASSERT_EQUAL(rdepth(vm), 0);
    ASSERT_EQUAL(pop(vm), 42);

    forth_destroy(vm);
}

// ============================================================================
// DICTIONARY TESTS
// ============================================================================

TEST(here_allot) {
    forth_vm_t *vm = forth_create();

    forth_here(vm);
    cell_t here1 = pop(vm);

    push(vm, 64);
    forth_allot(vm);

    forth_here(vm);
    cell_t here2 = pop(vm);

    ASSERT_EQUAL(here2 - here1, 64);

    forth_destroy(vm);
}

TEST(comma) {
    forth_vm_t *vm = forth_create();

    forth_here(vm);
    cell_t addr = pop(vm);

    push(vm, 42);
    forth_comma(vm);

    push(vm, addr);
    forth_fetch(vm);
    ASSERT_EQUAL(pop(vm), 42);

    forth_destroy(vm);
}

// ============================================================================
// COMPLEX INTEGRATION TESTS
// ============================================================================

TEST(factorial) {
    // Test computing 5! = 120
    forth_vm_t *vm = forth_create();

    // Manual computation: 1 * 2 * 3 * 4 * 5
    push(vm, 1);
    push(vm, 2);
    forth_mul(vm);
    push(vm, 3);
    forth_mul(vm);
    push(vm, 4);
    forth_mul(vm);
    push(vm, 5);
    forth_mul(vm);

    ASSERT_EQUAL(pop(vm), 120);

    forth_destroy(vm);
}

TEST(fibonacci) {
    // Compute 10th Fibonacci number (0,1,1,2,3,5,8,13,21,34,55)
    forth_vm_t *vm = forth_create();

    push(vm, 0);  // fib(0)
    push(vm, 1);  // fib(1)

    for (int i = 0; i < 9; i++) {
        forth_2dup(vm);    // ( a b -- a b a b )
        forth_add(vm);     // ( a b a -- a b a+b )
        forth_nrot(vm);    // ( a b c -- c a b )
        forth_drop(vm);    // ( c a b -- c a )
        forth_swap(vm);    // ( c a -- a c )
    }

    forth_drop(vm);
    ASSERT_EQUAL(pop(vm), 55);

    forth_destroy(vm);
}

// ============================================================================
// TEST RUNNER
// ============================================================================

int main(void) {
    printf("Fast Forth Runtime Test Suite\n");
    printf("==============================\n\n");

    // Arithmetic tests
    run_add();
    run_sub();
    run_mul();
    run_div();
    run_mod();
    run_divmod();
    run_negate();
    run_abs();
    run_min();
    run_max();

    // Stack tests
    run_dup();
    run_drop();
    run_swap();
    run_over();
    run_rot();
    run_nrot();
    run_tuck();
    run_2dup();

    // Logical tests
    run_and();
    run_or();
    run_xor();
    run_invert();
    run_lshift();
    run_rshift();

    // Comparison tests
    run_eq();
    run_lt();
    run_gt();

    // Memory tests
    run_fetch_store();
    run_cfetch_cstore();

    // Return stack tests
    run_return_stack();

    // Dictionary tests
    run_here_allot();
    run_comma();

    // Integration tests
    run_factorial();
    run_fibonacci();

    printf("\n==============================\n");
    printf("Tests passed: %d\n", tests_passed);
    printf("Tests failed: %d\n", tests_failed);

    return tests_failed > 0 ? 1 : 0;
}
