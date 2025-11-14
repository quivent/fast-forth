/**
 * Fast Forth Runtime Kernel Implementation
 * Stream 6: Core primitives and VM implementation
 *
 * Performance-critical primitives in C for maximum speed
 */

#include "forth_runtime.h"
#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include <ctype.h>

// ============================================================================
// VM LIFECYCLE
// ============================================================================

forth_vm_t *forth_create(void) {
    forth_vm_t *vm = calloc(1, sizeof(forth_vm_t));
    if (!vm) return NULL;

    // Allocate dictionary
    vm->dictionary = malloc(DICTIONARY_SIZE);
    if (!vm->dictionary) {
        free(vm);
        return NULL;
    }

    vm->dict_size = DICTIONARY_SIZE;
    vm->here = vm->dictionary;

    // Initialize stacks (pointing to base - 1)
    vm->dsp = vm->data_stack - 1;
    vm->rsp = vm->return_stack - 1;

    vm->compiling = false;
    vm->last_word = NULL;
    vm->error_code = FORTH_OK;

    return vm;
}

void forth_destroy(forth_vm_t *vm) {
    if (!vm) return;
    if (vm->dictionary) free(vm->dictionary);
    free(vm);
}

int forth_reset(forth_vm_t *vm) {
    vm->dsp = vm->data_stack - 1;
    vm->rsp = vm->return_stack - 1;
    vm->here = vm->dictionary;
    vm->compiling = false;
    vm->error_code = FORTH_OK;
    return FORTH_OK;
}

// ============================================================================
// ARITHMETIC PRIMITIVES (Optimized)
// ============================================================================

void forth_add(forth_vm_t *vm) {
    cell_t b = pop(vm);
    cell_t a = pop(vm);
    push(vm, a + b);
}

void forth_sub(forth_vm_t *vm) {
    cell_t b = pop(vm);
    cell_t a = pop(vm);
    push(vm, a - b);
}

void forth_mul(forth_vm_t *vm) {
    cell_t b = pop(vm);
    cell_t a = pop(vm);
    push(vm, a * b);
}

void forth_div(forth_vm_t *vm) {
    cell_t b = pop(vm);
    cell_t a = pop(vm);
    if (b == 0) {
        vm->error_code = FORTH_DIVIDE_BY_ZERO;
        push(vm, 0);
        return;
    }
    push(vm, a / b);
}

void forth_mod(forth_vm_t *vm) {
    cell_t b = pop(vm);
    cell_t a = pop(vm);
    if (b == 0) {
        vm->error_code = FORTH_DIVIDE_BY_ZERO;
        push(vm, 0);
        return;
    }
    push(vm, a % b);
}

void forth_divmod(forth_vm_t *vm) {
    cell_t b = pop(vm);
    cell_t a = pop(vm);
    if (b == 0) {
        vm->error_code = FORTH_DIVIDE_BY_ZERO;
        push(vm, 0);
        push(vm, 0);
        return;
    }
    push(vm, a % b);  // Remainder
    push(vm, a / b);  // Quotient
}

void forth_negate(forth_vm_t *vm) {
    cell_t a = pop(vm);
    push(vm, -a);
}

void forth_abs(forth_vm_t *vm) {
    cell_t a = pop(vm);
    push(vm, a < 0 ? -a : a);
}

void forth_min(forth_vm_t *vm) {
    cell_t b = pop(vm);
    cell_t a = pop(vm);
    push(vm, a < b ? a : b);
}

void forth_max(forth_vm_t *vm) {
    cell_t b = pop(vm);
    cell_t a = pop(vm);
    push(vm, a > b ? a : b);
}

// ============================================================================
// STACK MANIPULATION (Highly optimized)
// ============================================================================

void forth_dup(forth_vm_t *vm) {
    cell_t a = peek(vm);
    push(vm, a);
}

void forth_drop(forth_vm_t *vm) {
    vm->dsp--;
}

void forth_swap(forth_vm_t *vm) {
    cell_t b = pop(vm);
    cell_t a = pop(vm);
    push(vm, b);
    push(vm, a);
}

void forth_over(forth_vm_t *vm) {
    cell_t b = pop(vm);
    cell_t a = peek(vm);
    push(vm, b);
    push(vm, a);
}

void forth_rot(forth_vm_t *vm) {
    // ( a b c -- b c a )
    cell_t c = pop(vm);
    cell_t b = pop(vm);
    cell_t a = pop(vm);
    push(vm, b);
    push(vm, c);
    push(vm, a);
}

void forth_nrot(forth_vm_t *vm) {
    // ( a b c -- c a b )
    cell_t c = pop(vm);
    cell_t b = pop(vm);
    cell_t a = pop(vm);
    push(vm, c);
    push(vm, a);
    push(vm, b);
}

void forth_nip(forth_vm_t *vm) {
    // ( a b -- b )
    cell_t b = pop(vm);
    vm->dsp--;  // Drop a
    push(vm, b);
}

void forth_tuck(forth_vm_t *vm) {
    // ( a b -- b a b )
    cell_t b = pop(vm);
    cell_t a = pop(vm);
    push(vm, b);
    push(vm, a);
    push(vm, b);
}

void forth_pick(forth_vm_t *vm) {
    // ( ... n -- ... x ) where x is the nth item
    cell_t n = pop(vm);
    push(vm, vm->dsp[-n]);
}

void forth_roll(forth_vm_t *vm) {
    // ( ... n -- ... x ) move nth item to top
    cell_t n = pop(vm);
    cell_t x = vm->dsp[-n];
    // Shift items down
    for (int i = -n; i < 0; i++) {
        vm->dsp[i] = vm->dsp[i + 1];
    }
    *vm->dsp = x;
}

void forth_2dup(forth_vm_t *vm) {
    // ( a b -- a b a b )
    cell_t b = vm->dsp[0];
    cell_t a = vm->dsp[-1];
    push(vm, a);
    push(vm, b);
}

void forth_2drop(forth_vm_t *vm) {
    vm->dsp -= 2;
}

void forth_2swap(forth_vm_t *vm) {
    // ( a b c d -- c d a b )
    cell_t d = vm->dsp[0];
    cell_t c = vm->dsp[-1];
    cell_t b = vm->dsp[-2];
    cell_t a = vm->dsp[-3];
    vm->dsp[-3] = c;
    vm->dsp[-2] = d;
    vm->dsp[-1] = a;
    vm->dsp[0] = b;
}

void forth_2over(forth_vm_t *vm) {
    // ( a b c d -- a b c d a b )
    cell_t b = vm->dsp[-2];
    cell_t a = vm->dsp[-3];
    push(vm, a);
    push(vm, b);
}

// ============================================================================
// LOGICAL OPERATIONS
// ============================================================================

void forth_and(forth_vm_t *vm) {
    cell_t b = pop(vm);
    cell_t a = pop(vm);
    push(vm, a & b);
}

void forth_or(forth_vm_t *vm) {
    cell_t b = pop(vm);
    cell_t a = pop(vm);
    push(vm, a | b);
}

void forth_xor(forth_vm_t *vm) {
    cell_t b = pop(vm);
    cell_t a = pop(vm);
    push(vm, a ^ b);
}

void forth_invert(forth_vm_t *vm) {
    cell_t a = pop(vm);
    push(vm, ~a);
}

void forth_lshift(forth_vm_t *vm) {
    cell_t n = pop(vm);
    cell_t x = pop(vm);
    push(vm, x << n);
}

void forth_rshift(forth_vm_t *vm) {
    cell_t n = pop(vm);
    cell_t x = pop(vm);
    push(vm, (ucell_t)x >> n);  // Logical shift
}

// ============================================================================
// COMPARISON OPERATIONS
// ============================================================================

void forth_eq(forth_vm_t *vm) {
    cell_t b = pop(vm);
    cell_t a = pop(vm);
    push(vm, a == b ? -1 : 0);  // Forth true is -1
}

void forth_neq(forth_vm_t *vm) {
    cell_t b = pop(vm);
    cell_t a = pop(vm);
    push(vm, a != b ? -1 : 0);
}

void forth_lt(forth_vm_t *vm) {
    cell_t b = pop(vm);
    cell_t a = pop(vm);
    push(vm, a < b ? -1 : 0);
}

void forth_gt(forth_vm_t *vm) {
    cell_t b = pop(vm);
    cell_t a = pop(vm);
    push(vm, a > b ? -1 : 0);
}

void forth_le(forth_vm_t *vm) {
    cell_t b = pop(vm);
    cell_t a = pop(vm);
    push(vm, a <= b ? -1 : 0);
}

void forth_ge(forth_vm_t *vm) {
    cell_t b = pop(vm);
    cell_t a = pop(vm);
    push(vm, a >= b ? -1 : 0);
}

void forth_0eq(forth_vm_t *vm) {
    cell_t a = pop(vm);
    push(vm, a == 0 ? -1 : 0);
}

void forth_0lt(forth_vm_t *vm) {
    cell_t a = pop(vm);
    push(vm, a < 0 ? -1 : 0);
}

void forth_0gt(forth_vm_t *vm) {
    cell_t a = pop(vm);
    push(vm, a > 0 ? -1 : 0);
}

// ============================================================================
// MEMORY OPERATIONS
// ============================================================================

void forth_fetch(forth_vm_t *vm) {
    cell_t addr = pop(vm);
    push(vm, *(cell_t*)addr);
}

void forth_store(forth_vm_t *vm) {
    cell_t addr = pop(vm);
    cell_t value = pop(vm);
    *(cell_t*)addr = value;
}

void forth_cfetch(forth_vm_t *vm) {
    cell_t addr = pop(vm);
    push(vm, *(byte_t*)addr);
}

void forth_cstore(forth_vm_t *vm) {
    cell_t addr = pop(vm);
    cell_t value = pop(vm);
    *(byte_t*)addr = (byte_t)value;
}

void forth_addstore(forth_vm_t *vm) {
    cell_t addr = pop(vm);
    cell_t value = pop(vm);
    *(cell_t*)addr += value;
}

void forth_2fetch(forth_vm_t *vm) {
    cell_t addr = pop(vm);
    push(vm, ((cell_t*)addr)[0]);
    push(vm, ((cell_t*)addr)[1]);
}

void forth_2store(forth_vm_t *vm) {
    cell_t addr = pop(vm);
    cell_t b = pop(vm);
    cell_t a = pop(vm);
    ((cell_t*)addr)[0] = a;
    ((cell_t*)addr)[1] = b;
}

// ============================================================================
// RETURN STACK OPERATIONS
// ============================================================================

void forth_tor(forth_vm_t *vm) {
    rpush(vm, pop(vm));
}

void forth_fromr(forth_vm_t *vm) {
    push(vm, rpop(vm));
}

void forth_rfetch(forth_vm_t *vm) {
    push(vm, *vm->rsp);
}

// ============================================================================
// I/O PRIMITIVES
// ============================================================================

void forth_emit(forth_vm_t *vm) {
    cell_t c = pop(vm);
    putchar((char)c);
    fflush(stdout);
}

void forth_key(forth_vm_t *vm) {
    push(vm, getchar());
}

void forth_type(forth_vm_t *vm) {
    cell_t len = pop(vm);
    cell_t addr = pop(vm);
    fwrite((char*)addr, 1, len, stdout);
    fflush(stdout);
}

void forth_cr(forth_vm_t *vm) {
    putchar('\n');
    fflush(stdout);
}

void forth_space(forth_vm_t *vm) {
    putchar(' ');
}

void forth_spaces(forth_vm_t *vm) {
    cell_t n = pop(vm);
    for (cell_t i = 0; i < n; i++) {
        putchar(' ');
    }
}

// ============================================================================
// DICTIONARY OPERATIONS
// ============================================================================

void forth_here(forth_vm_t *vm) {
    push(vm, (cell_t)vm->here);
}

void forth_allot(forth_vm_t *vm) {
    cell_t n = pop(vm);
    vm->here += n;
}

void forth_comma(forth_vm_t *vm) {
    cell_t value = pop(vm);
    *(cell_t*)vm->here = value;
    vm->here += sizeof(cell_t);
}

void forth_ccomma(forth_vm_t *vm) {
    cell_t value = pop(vm);
    *vm->here++ = (byte_t)value;
}

// ============================================================================
// WORD FINDING (Optimized with length prefix)
// ============================================================================

word_header_t *forth_find_word(forth_vm_t *vm, const char *name, size_t len) {
    word_header_t *word = vm->last_word;

    while (word) {
        // Quick length check first (optimization)
        if (word->name_len == len &&
            !(word->flags & FLAG_HIDDEN) &&
            strncmp(word->name, name, len) == 0) {
            return word;
        }
        word = word->link;
    }

    return NULL;
}

void forth_define_word(forth_vm_t *vm, const char *name, void (*code)(forth_vm_t*), uint8_t flags) {
    size_t name_len = strlen(name);

    // Align to cell boundary
    vm->here = (byte_t*)(((uintptr_t)vm->here + sizeof(cell_t) - 1) & ~(sizeof(cell_t) - 1));

    word_header_t *header = (word_header_t*)vm->here;
    header->link = vm->last_word;
    header->flags = flags;
    header->name_len = name_len;

    vm->here += sizeof(word_header_t);
    memcpy(vm->here, name, name_len);
    vm->here += name_len;

    // Align again
    vm->here = (byte_t*)(((uintptr_t)vm->here + sizeof(cell_t) - 1) & ~(sizeof(cell_t) - 1));

    // Store code pointer
    *(void**)vm->here = (void*)code;
    vm->here += sizeof(void*);

    vm->last_word = header;
}

// ============================================================================
// DEBUGGING & INTROSPECTION
// ============================================================================

void forth_dump_stack(forth_vm_t *vm) {
    int d = depth(vm);
    printf("Stack<%d>: ", d);
    for (int i = 0; i < d; i++) {
        printf("%ld ", vm->data_stack[i]);
    }
    printf("\n");
}

void forth_dump_dictionary(forth_vm_t *vm) {
    word_header_t *word = vm->last_word;
    printf("Dictionary:\n");
    while (word) {
        printf("  %.*s%s%s\n",
               word->name_len, word->name,
               (word->flags & FLAG_IMMEDIATE) ? " (IMMEDIATE)" : "",
               (word->flags & FLAG_HIDDEN) ? " (HIDDEN)" : "");
        word = word->link;
    }
}

void forth_dump_memory(forth_vm_t *vm, cell_t addr, size_t count) {
    printf("Memory dump at 0x%lx:\n", addr);
    byte_t *ptr = (byte_t*)addr;
    for (size_t i = 0; i < count; i += 16) {
        printf("%08lx: ", addr + i);
        for (size_t j = 0; j < 16 && i + j < count; j++) {
            printf("%02x ", ptr[i + j]);
        }
        printf("\n");
    }
}

// ============================================================================
// FFI SUPPORT (C function calling)
// ============================================================================

int forth_ffi_call(forth_vm_t *vm, void *func_ptr, int arg_count) {
    // Simple FFI for up to 6 arguments (typical C calling convention)
    cell_t args[6];

    if (arg_count > 6) return -1;

    // Pop arguments in reverse order
    for (int i = arg_count - 1; i >= 0; i--) {
        args[i] = pop(vm);
    }

    // Call function based on argument count
    cell_t result;
    switch (arg_count) {
        case 0: result = ((cell_t(*)(void))func_ptr)(); break;
        case 1: result = ((cell_t(*)(cell_t))func_ptr)(args[0]); break;
        case 2: result = ((cell_t(*)(cell_t,cell_t))func_ptr)(args[0], args[1]); break;
        case 3: result = ((cell_t(*)(cell_t,cell_t,cell_t))func_ptr)(args[0], args[1], args[2]); break;
        case 4: result = ((cell_t(*)(cell_t,cell_t,cell_t,cell_t))func_ptr)(args[0], args[1], args[2], args[3]); break;
        case 5: result = ((cell_t(*)(cell_t,cell_t,cell_t,cell_t,cell_t))func_ptr)(args[0], args[1], args[2], args[3], args[4]); break;
        case 6: result = ((cell_t(*)(cell_t,cell_t,cell_t,cell_t,cell_t,cell_t))func_ptr)(args[0], args[1], args[2], args[3], args[4], args[5]); break;
        default: return -1;
    }

    push(vm, result);
    return 0;
}

void forth_ffi_register(forth_vm_t *vm, const char *name, void *func_ptr) {
    // Create a wrapper word that calls the FFI function
    // This is simplified - real implementation would need argument count
    forth_define_word(vm, name, (void(*)(forth_vm_t*))func_ptr, 0);
}
