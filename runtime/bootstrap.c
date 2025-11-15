/**
 * Fast Forth Bootstrap
 * Stream 6: System initialization and primitive registration
 *
 * Initializes the VM and registers all core primitives
 */

#include "forth_runtime.h"
#include "concurrency.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

// External declarations for FFI and memory
extern void forth_ffi_init_stdlib(forth_vm_t *vm);
extern void forth_ffi_init(void);

// ============================================================================
// PRIMITIVE REGISTRATION
// ============================================================================

typedef struct {
    const char *name;
    void (*code)(forth_vm_t*);
    uint8_t flags;
} primitive_def_t;

static const primitive_def_t primitives[] = {
    // Arithmetic
    {"+",       forth_add,      0},
    {"-",       forth_sub,      0},
    {"*",       forth_mul,      0},
    {"/",       forth_div,      0},
    {"MOD",     forth_mod,      0},
    {"/MOD",    forth_divmod,   0},
    {"NEGATE",  forth_negate,   0},
    {"ABS",     forth_abs,      0},
    {"MIN",     forth_min,      0},
    {"MAX",     forth_max,      0},

    // Stack manipulation
    {"DUP",     forth_dup,      0},
    {"DROP",    forth_drop,     0},
    {"SWAP",    forth_swap,     0},
    {"OVER",    forth_over,     0},
    {"ROT",     forth_rot,      0},
    {"-ROT",    forth_nrot,     0},
    {"NIP",     forth_nip,      0},
    {"TUCK",    forth_tuck,     0},
    {"PICK",    forth_pick,     0},
    {"ROLL",    forth_roll,     0},
    {"2DUP",    forth_2dup,     0},
    {"2DROP",   forth_2drop,    0},
    {"2SWAP",   forth_2swap,    0},
    {"2OVER",   forth_2over,    0},

    // Logical operations
    {"AND",     forth_and,      0},
    {"OR",      forth_or,       0},
    {"XOR",     forth_xor,      0},
    {"INVERT",  forth_invert,   0},
    {"LSHIFT",  forth_lshift,   0},
    {"RSHIFT",  forth_rshift,   0},

    // Comparison
    {"=",       forth_eq,       0},
    {"<>",      forth_neq,      0},
    {"<",       forth_lt,       0},
    {">",       forth_gt,       0},
    {"<=",      forth_le,       0},
    {">=",      forth_ge,       0},
    {"0=",      forth_0eq,      0},
    {"0<",      forth_0lt,      0},
    {"0>",      forth_0gt,      0},

    // Memory operations
    {"@",       forth_fetch,    0},
    {"!",       forth_store,    0},
    {"C@",      forth_cfetch,   0},
    {"C!",      forth_cstore,   0},
    {"+!",      forth_addstore, 0},
    {"2@",      forth_2fetch,   0},
    {"2!",      forth_2store,   0},

    // Return stack
    {">R",      forth_tor,      0},
    {"R>",      forth_fromr,    0},
    {"R@",      forth_rfetch,   0},

    // I/O primitives
    {"EMIT",    forth_emit,     0},
    {"KEY",     forth_key,      0},
    {"TYPE",    forth_type,     0},
    {"CR",      forth_cr,       0},
    {"SPACE",   forth_space,    0},
    {"SPACES",  forth_spaces,   0},

    // Dictionary operations
    {"HERE",    forth_here,     0},
    {"ALLOT",   forth_allot,    0},
    {",",       forth_comma,    0},
    {"C,",      forth_ccomma,   0},
    {"CREATE",  forth_create_word,   0},
    {"DOES>",   forth_does,     FLAG_COMPILE_ONLY},

    // Compilation (stubs - full implementation needs compiler)
    {":",       forth_colon,    0},
    {";",       forth_semicolon, FLAG_IMMEDIATE | FLAG_COMPILE_ONLY},
    {"IMMEDIATE", forth_immediate, 0},
    {"LITERAL", forth_literal, FLAG_IMMEDIATE | FLAG_COMPILE_ONLY},
    {"POSTPONE", forth_postpone, FLAG_IMMEDIATE | FLAG_COMPILE_ONLY},

    // Concurrency primitives (NEW)
    {"SPAWN",          forth_spawn_primitive,          0},
    {"JOIN",           forth_join_primitive,           0},
    {"CHANNEL",        forth_channel_primitive,        0},
    {"SEND",           forth_send_primitive,           0},
    {"RECV",           forth_recv_primitive,           0},
    {"CLOSE-CHANNEL",  forth_close_channel_primitive,  0},
    {"DESTROY-CHANNEL",forth_destroy_channel_primitive,0},

    // Terminator
    {NULL, NULL, 0}
};

// ============================================================================
// VM INITIALIZATION
// ============================================================================

int forth_bootstrap(forth_vm_t *vm) {
    if (!vm) return -1;

    // Register all primitives
    for (int i = 0; primitives[i].name != NULL; i++) {
        forth_define_word(vm, primitives[i].name, primitives[i].code, primitives[i].flags);
    }

    // Initialize FFI
    forth_ffi_init();
    forth_ffi_init_stdlib(vm);

    printf("Fast Forth Runtime v1.0\n");
    printf("  %d primitives loaded\n", (int)(sizeof(primitives) / sizeof(primitives[0]) - 1));
    printf("  Dictionary: %zu bytes\n", vm->dict_size);
    printf("  Stack: %d cells\n", DATA_STACK_SIZE);
    printf("\nType 'WORDS' to see available words\n\n");

    return FORTH_OK;
}

// ============================================================================
// COMPILATION STUBS (Simplified - full implementation in compiler)
// ============================================================================

void forth_colon(forth_vm_t *vm) {
    // Start compilation mode
    vm->compiling = true;

    // Create new word header
    forth_create_word(vm);
}

void forth_semicolon(forth_vm_t *vm) {
    // End compilation mode
    vm->compiling = false;

    // Compile EXIT instruction
    // (This would need actual compiler support)
}

void forth_immediate(forth_vm_t *vm) {
    // Mark last word as immediate
    if (vm->last_word) {
        word_header_t *header = (word_header_t *)vm->last_word;
        header->flags |= FLAG_IMMEDIATE;
    }
}

void forth_literal(forth_vm_t *vm) {
    // Compile a literal value
    cell_t value = pop(vm);

    // Compile literal opcode and value
    // (Needs compiler support)
    push(vm, value);  // For now, just push it
}

void forth_postpone(forth_vm_t *vm) {
    // Postpone compilation of next word
    // (Needs compiler support)
}

// ============================================================================
// REPL (Read-Eval-Print Loop)
// ============================================================================

int forth_repl(forth_vm_t *vm) {
    char input[1024];

    printf("ok> ");
    fflush(stdout);

    while (fgets(input, sizeof(input), stdin)) {
        // Remove trailing newline
        size_t len = strlen(input);
        if (len > 0 && input[len-1] == '\n') {
            input[len-1] = '\0';
            len--;
        }

        // Set input buffer
        vm->input_buffer = input;
        vm->input_pos = 0;
        vm->input_len = len;

        // Interpret line
        int result = forth_interpret(vm, input);

        if (result != FORTH_OK) {
            printf("Error %d: %s\n", result, vm->error_msg);
        }

        // Print stack
        forth_dump_stack(vm);

        printf("ok> ");
        fflush(stdout);
    }

    return FORTH_OK;
}

// ============================================================================
// SIMPLE INTERPRETER (For bootstrapping)
// ============================================================================

int forth_interpret(forth_vm_t *vm, const char *input) {
    vm->input_buffer = (char*)input;
    vm->input_pos = 0;
    vm->input_len = strlen(input);

    while (vm->input_pos < vm->input_len) {
        // Skip whitespace
        while (vm->input_pos < vm->input_len &&
               (vm->input_buffer[vm->input_pos] == ' ' ||
                vm->input_buffer[vm->input_pos] == '\t' ||
                vm->input_buffer[vm->input_pos] == '\n')) {
            vm->input_pos++;
        }

        if (vm->input_pos >= vm->input_len) break;

        // Parse word
        char word[256];
        int len = 0;

        while (vm->input_pos < vm->input_len &&
               vm->input_buffer[vm->input_pos] != ' ' &&
               vm->input_buffer[vm->input_pos] != '\t' &&
               vm->input_buffer[vm->input_pos] != '\n' &&
               len < 255) {
            word[len++] = vm->input_buffer[vm->input_pos++];
        }

        if (len == 0) continue;

        word[len] = '\0';

        // Try to find word in dictionary
        word_header_t *w = forth_find_word(vm, word, len);

        if (w) {
            // Execute word
            void (*code)(forth_vm_t*) = *(void(**)(forth_vm_t*))((byte_t*)w + sizeof(word_header_t) + w->name_len);

            // Align to cell boundary
            code = *(void(**)(forth_vm_t*))(((uintptr_t)w + sizeof(word_header_t) + w->name_len + sizeof(cell_t) - 1) & ~(sizeof(cell_t) - 1));

            code(vm);
        } else {
            // Try to parse as number
            char *endptr;
            long value = strtol(word, &endptr, 10);

            if (*endptr == '\0') {
                // Valid number
                push(vm, value);
            } else {
                // Unknown word
                snprintf(vm->error_msg, sizeof(vm->error_msg), "Undefined word: %s", word);
                return FORTH_UNDEFINED_WORD;
            }
        }

        if (vm->error_code != FORTH_OK) {
            return vm->error_code;
        }
    }

    return FORTH_OK;
}

// ============================================================================
// MAIN ENTRY POINT (Example)
// ============================================================================

#ifdef FORTH_STANDALONE
int main(int argc, char **argv) {
    forth_vm_t *vm = forth_create();
    if (!vm) {
        fprintf(stderr, "Failed to create VM\n");
        return 1;
    }

    forth_bootstrap(vm);

    if (argc > 1) {
        // Execute file
        FILE *f = fopen(argv[1], "r");
        if (!f) {
            fprintf(stderr, "Failed to open %s\n", argv[1]);
            forth_destroy(vm);
            return 1;
        }

        char line[1024];
        while (fgets(line, sizeof(line), f)) {
            forth_interpret(vm, line);
        }

        fclose(f);
    } else {
        // Interactive REPL
        forth_repl(vm);
    }

    forth_destroy(vm);
    return 0;
}
#endif
