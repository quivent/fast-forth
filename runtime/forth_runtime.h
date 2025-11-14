/**
 * Fast Forth Runtime Kernel
 * Stream 6: Runtime & Standard Library
 *
 * High-performance ANS Forth runtime with optimized primitives
 */

#ifndef FORTH_RUNTIME_H
#define FORTH_RUNTIME_H

#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>

// ============================================================================
// CORE TYPE DEFINITIONS
// ============================================================================

typedef intptr_t cell_t;      // Native word size (64-bit on modern systems)
typedef uintptr_t ucell_t;    // Unsigned cell
typedef int32_t half_cell_t;  // Half-cell for compatibility
typedef uint8_t byte_t;       // Byte type

// Stack depth limits
#define DATA_STACK_SIZE 256
#define RETURN_STACK_SIZE 256
#define DICTIONARY_SIZE (1024 * 1024)  // 1MB initial dictionary

// ============================================================================
// FORTH VIRTUAL MACHINE STATE
// ============================================================================

typedef struct {
    // Data stack (parameter stack)
    cell_t data_stack[DATA_STACK_SIZE];
    cell_t *dsp;  // Data stack pointer

    // Return stack
    cell_t return_stack[RETURN_STACK_SIZE];
    cell_t *rsp;  // Return stack pointer

    // Dictionary (heap memory)
    byte_t *dictionary;
    byte_t *here;         // Dictionary pointer (next free location)
    size_t dict_size;     // Total dictionary size

    // Compilation state
    bool compiling;       // true when compiling, false when interpreting
    cell_t *last_word;    // Pointer to last defined word

    // I/O state
    char *input_buffer;
    size_t input_pos;
    size_t input_len;

    // Error handling
    int error_code;
    char error_msg[256];
} forth_vm_t;

// ============================================================================
// ERROR CODES
// ============================================================================

#define FORTH_OK 0
#define FORTH_STACK_UNDERFLOW -1
#define FORTH_STACK_OVERFLOW -2
#define FORTH_DIVIDE_BY_ZERO -3
#define FORTH_INVALID_MEMORY -4
#define FORTH_UNDEFINED_WORD -5
#define FORTH_COMPILE_ONLY -6
#define FORTH_INVALID_STATE -7

// ============================================================================
// WORD HEADER STRUCTURE
// ============================================================================

typedef struct word_header {
    struct word_header *link;  // Link to previous word
    uint8_t flags;             // Word flags
    uint8_t name_len;          // Name length
    char name[];               // Variable-length name (flexible array member)
} word_header_t;

// Word flags
#define FLAG_IMMEDIATE 0x01
#define FLAG_HIDDEN 0x02
#define FLAG_COMPILE_ONLY 0x04

// ============================================================================
// CORE VM FUNCTIONS
// ============================================================================

// VM lifecycle
forth_vm_t *forth_create(void);
void forth_destroy(forth_vm_t *vm);
int forth_reset(forth_vm_t *vm);

// Stack operations (inline for performance)
static inline void push(forth_vm_t *vm, cell_t value) {
    *++vm->dsp = value;
}

static inline cell_t pop(forth_vm_t *vm) {
    return *vm->dsp--;
}

static inline cell_t peek(forth_vm_t *vm) {
    return *vm->dsp;
}

static inline void rpush(forth_vm_t *vm, cell_t value) {
    *++vm->rsp = value;
}

static inline cell_t rpop(forth_vm_t *vm) {
    return *vm->rsp--;
}

// Stack depth checks
static inline int depth(forth_vm_t *vm) {
    return vm->dsp - vm->data_stack;
}

static inline int rdepth(forth_vm_t *vm) {
    return vm->rsp - vm->return_stack;
}

// ============================================================================
// PRIMITIVE OPERATIONS (C IMPLEMENTATIONS)
// ============================================================================

// Arithmetic primitives
void forth_add(forth_vm_t *vm);      // +
void forth_sub(forth_vm_t *vm);      // -
void forth_mul(forth_vm_t *vm);      // *
void forth_div(forth_vm_t *vm);      // /
void forth_mod(forth_vm_t *vm);      // MOD
void forth_divmod(forth_vm_t *vm);   // /MOD
void forth_negate(forth_vm_t *vm);   // NEGATE
void forth_abs(forth_vm_t *vm);      // ABS
void forth_min(forth_vm_t *vm);      // MIN
void forth_max(forth_vm_t *vm);      // MAX

// Stack manipulation
void forth_dup(forth_vm_t *vm);      // DUP
void forth_drop(forth_vm_t *vm);     // DROP
void forth_swap(forth_vm_t *vm);     // SWAP
void forth_over(forth_vm_t *vm);     // OVER
void forth_rot(forth_vm_t *vm);      // ROT
void forth_nrot(forth_vm_t *vm);     // -ROT
void forth_nip(forth_vm_t *vm);      // NIP
void forth_tuck(forth_vm_t *vm);     // TUCK
void forth_pick(forth_vm_t *vm);     // PICK
void forth_roll(forth_vm_t *vm);     // ROLL
void forth_2dup(forth_vm_t *vm);     // 2DUP
void forth_2drop(forth_vm_t *vm);    // 2DROP
void forth_2swap(forth_vm_t *vm);    // 2SWAP
void forth_2over(forth_vm_t *vm);    // 2OVER

// Logical operations
void forth_and(forth_vm_t *vm);      // AND
void forth_or(forth_vm_t *vm);       // OR
void forth_xor(forth_vm_t *vm);      // XOR
void forth_invert(forth_vm_t *vm);   // INVERT
void forth_lshift(forth_vm_t *vm);   // LSHIFT
void forth_rshift(forth_vm_t *vm);   // RSHIFT

// Comparison
void forth_eq(forth_vm_t *vm);       // =
void forth_neq(forth_vm_t *vm);      // <>
void forth_lt(forth_vm_t *vm);       // <
void forth_gt(forth_vm_t *vm);       // >
void forth_le(forth_vm_t *vm);       // <=
void forth_ge(forth_vm_t *vm);       // >=
void forth_0eq(forth_vm_t *vm);      // 0=
void forth_0lt(forth_vm_t *vm);      // 0<
void forth_0gt(forth_vm_t *vm);      // 0>

// Memory operations
void forth_fetch(forth_vm_t *vm);    // @
void forth_store(forth_vm_t *vm);    // !
void forth_cfetch(forth_vm_t *vm);   // C@
void forth_cstore(forth_vm_t *vm);   // C!
void forth_addstore(forth_vm_t *vm); // +!
void forth_2fetch(forth_vm_t *vm);   // 2@
void forth_2store(forth_vm_t *vm);   // 2!

// Return stack
void forth_tor(forth_vm_t *vm);      // >R
void forth_fromr(forth_vm_t *vm);    // R>
void forth_rfetch(forth_vm_t *vm);   // R@

// I/O primitives
void forth_emit(forth_vm_t *vm);     // EMIT
void forth_key(forth_vm_t *vm);      // KEY
void forth_type(forth_vm_t *vm);     // TYPE
void forth_cr(forth_vm_t *vm);       // CR
void forth_space(forth_vm_t *vm);    // SPACE
void forth_spaces(forth_vm_t *vm);   // SPACES

// Dictionary operations
void forth_here(forth_vm_t *vm);     // HERE
void forth_allot(forth_vm_t *vm);    // ALLOT
void forth_comma(forth_vm_t *vm);    // ,
void forth_ccomma(forth_vm_t *vm);   // C,
void forth_create(forth_vm_t *vm);   // CREATE
void forth_does(forth_vm_t *vm);     // DOES>

// Compilation
void forth_colon(forth_vm_t *vm);    // :
void forth_semicolon(forth_vm_t *vm);// ;
void forth_immediate(forth_vm_t *vm);// IMMEDIATE
void forth_literal(forth_vm_t *vm);  // LITERAL
void forth_postpone(forth_vm_t *vm); // POSTPONE

// Word execution
int forth_execute(forth_vm_t *vm, void *code_addr);
int forth_interpret(forth_vm_t *vm, const char *input);

// ============================================================================
// DICTIONARY MANAGEMENT
// ============================================================================

word_header_t *forth_find_word(forth_vm_t *vm, const char *name, size_t len);
void forth_define_word(forth_vm_t *vm, const char *name, void (*code)(forth_vm_t*), uint8_t flags);
void forth_hide_word(forth_vm_t *vm);
void forth_reveal_word(forth_vm_t *vm);

// ============================================================================
// OPTIMIZED PRIMITIVES (PLATFORM-SPECIFIC)
// ============================================================================

#ifdef __x86_64__
// x86-64 optimized primitives using inline assembly
static inline cell_t fast_add(cell_t a, cell_t b) {
    cell_t result;
    __asm__ ("add %2, %0" : "=r"(result) : "0"(a), "r"(b));
    return result;
}

static inline cell_t fast_mul(cell_t a, cell_t b) {
    cell_t result;
    __asm__ ("imul %2, %0" : "=r"(result) : "0"(a), "r"(b));
    return result;
}
#else
// Fallback to standard C
#define fast_add(a, b) ((a) + (b))
#define fast_mul(a, b) ((a) * (b))
#endif

// ============================================================================
// FFI SUPPORT
// ============================================================================

typedef cell_t (*ffi_func_t)(cell_t*, int);

int forth_ffi_call(forth_vm_t *vm, void *func_ptr, int arg_count);
void forth_ffi_register(forth_vm_t *vm, const char *name, void *func_ptr);

// ============================================================================
// DEBUGGING & INTROSPECTION
// ============================================================================

void forth_dump_stack(forth_vm_t *vm);
void forth_dump_dictionary(forth_vm_t *vm);
void forth_dump_memory(forth_vm_t *vm, cell_t addr, size_t count);
void forth_see(forth_vm_t *vm, const char *word_name);

#endif // FORTH_RUNTIME_H
