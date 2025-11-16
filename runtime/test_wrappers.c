/**
 * Test Wrappers for Inline Functions
 *
 * Provides non-inline exports of stack operations for FFI testing
 */

#include "forth_runtime.h"

// Export inline stack operations as regular functions for FFI

void test_push(forth_vm_t *vm, cell_t value) {
    push(vm, value);
}

cell_t test_pop(forth_vm_t *vm) {
    return pop(vm);
}

cell_t test_peek(forth_vm_t *vm) {
    return peek(vm);
}

void test_rpush(forth_vm_t *vm, cell_t value) {
    rpush(vm, value);
}

cell_t test_rpop(forth_vm_t *vm) {
    return rpop(vm);
}

int test_depth(forth_vm_t *vm) {
    return depth(vm);
}

int test_rdepth(forth_vm_t *vm) {
    return rdepth(vm);
}
