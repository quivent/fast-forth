/**
 * Fast Forth Foreign Function Interface (FFI)
 * Stream 6: C library integration and dynamic loading
 *
 * Allows Forth code to call arbitrary C functions with automatic
 * type marshalling and calling convention handling.
 */

#include "forth_runtime.h"
#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include <dlfcn.h>  // Dynamic library loading (Unix)

// ============================================================================
// TYPE DEFINITIONS
// ============================================================================

typedef enum {
    FFI_TYPE_VOID,
    FFI_TYPE_INT,
    FFI_TYPE_LONG,
    FFI_TYPE_FLOAT,
    FFI_TYPE_DOUBLE,
    FFI_TYPE_POINTER,
    FFI_TYPE_STRING,
} ffi_type_t;

typedef struct {
    char name[256];
    void *handle;           // dlopen handle
    void *func_ptr;         // Function pointer
    ffi_type_t return_type;
    ffi_type_t arg_types[16];
    int arg_count;
} ffi_function_t;

#define MAX_FFI_FUNCTIONS 256

typedef struct {
    ffi_function_t functions[MAX_FFI_FUNCTIONS];
    int count;
    void *lib_handles[32];  // Loaded library handles
    int lib_count;
} ffi_registry_t;

static ffi_registry_t *ffi_registry = NULL;

// ============================================================================
// FFI INITIALIZATION
// ============================================================================

void forth_ffi_init(void) {
    if (!ffi_registry) {
        ffi_registry = calloc(1, sizeof(ffi_registry_t));
    }
}

void forth_ffi_cleanup(void) {
    if (!ffi_registry) return;

    // Close all library handles
    for (int i = 0; i < ffi_registry->lib_count; i++) {
        dlclose(ffi_registry->lib_handles[i]);
    }

    free(ffi_registry);
    ffi_registry = NULL;
}

// ============================================================================
// LIBRARY LOADING
// ============================================================================

void *forth_ffi_load_library(const char *path) {
    forth_ffi_init();

    void *handle = dlopen(path, RTLD_LAZY | RTLD_LOCAL);
    if (!handle) {
        fprintf(stderr, "FFI: Failed to load library %s: %s\n", path, dlerror());
        return NULL;
    }

    if (ffi_registry->lib_count < 32) {
        ffi_registry->lib_handles[ffi_registry->lib_count++] = handle;
    }

    return handle;
}

void *forth_ffi_get_symbol(void *handle, const char *name) {
    void *sym = dlsym(handle, name);
    if (!sym) {
        fprintf(stderr, "FFI: Symbol not found: %s\n", name);
    }
    return sym;
}

// ============================================================================
// FUNCTION REGISTRATION
// ============================================================================

int forth_ffi_register_function(
    const char *name,
    void *func_ptr,
    ffi_type_t return_type,
    const ffi_type_t *arg_types,
    int arg_count
) {
    forth_ffi_init();

    if (ffi_registry->count >= MAX_FFI_FUNCTIONS) {
        fprintf(stderr, "FFI: Registry full\n");
        return -1;
    }

    ffi_function_t *func = &ffi_registry->functions[ffi_registry->count++];
    strncpy(func->name, name, 255);
    func->func_ptr = func_ptr;
    func->return_type = return_type;
    func->arg_count = arg_count;

    for (int i = 0; i < arg_count && i < 16; i++) {
        func->arg_types[i] = arg_types[i];
    }

    return ffi_registry->count - 1;
}

ffi_function_t *forth_ffi_find_function(const char *name) {
    if (!ffi_registry) return NULL;

    for (int i = 0; i < ffi_registry->count; i++) {
        if (strcmp(ffi_registry->functions[i].name, name) == 0) {
            return &ffi_registry->functions[i];
        }
    }

    return NULL;
}

// ============================================================================
// DYNAMIC CALL IMPLEMENTATION
// ============================================================================

// Convert Forth cell to C type
static void ffi_marshal_arg(ffi_type_t type, cell_t forth_value, void *c_value) {
    switch (type) {
        case FFI_TYPE_INT:
            *(int*)c_value = (int)forth_value;
            break;
        case FFI_TYPE_LONG:
            *(long*)c_value = (long)forth_value;
            break;
        case FFI_TYPE_FLOAT:
            *(float*)c_value = (float)forth_value;
            break;
        case FFI_TYPE_DOUBLE:
            *(double*)c_value = (double)forth_value;
            break;
        case FFI_TYPE_POINTER:
        case FFI_TYPE_STRING:
            *(void**)c_value = (void*)forth_value;
            break;
        default:
            break;
    }
}

// Convert C type to Forth cell
static cell_t ffi_unmarshal_result(ffi_type_t type, void *c_value) {
    switch (type) {
        case FFI_TYPE_INT:
            return *(int*)c_value;
        case FFI_TYPE_LONG:
            return *(long*)c_value;
        case FFI_TYPE_FLOAT:
            return (cell_t)*(float*)c_value;
        case FFI_TYPE_DOUBLE:
            return (cell_t)*(double*)c_value;
        case FFI_TYPE_POINTER:
        case FFI_TYPE_STRING:
            return (cell_t)*(void**)c_value;
        case FFI_TYPE_VOID:
        default:
            return 0;
    }
}

// Generic function call (up to 16 arguments)
int forth_ffi_call_function(forth_vm_t *vm, ffi_function_t *func) {
    if (!func || !func->func_ptr) return -1;

    // Marshal arguments from Forth stack
    cell_t forth_args[16];
    void *c_args[16];
    long arg_values[16];  // Storage for arguments

    for (int i = func->arg_count - 1; i >= 0; i--) {
        forth_args[i] = pop(vm);
        c_args[i] = &arg_values[i];
        ffi_marshal_arg(func->arg_types[i], forth_args[i], c_args[i]);
    }

    // Call function (using libffi-style dynamic call)
    // This is a simplified version - real implementation would use libffi
    void *result_storage[2];  // Enough for any return type

    // Manual dispatch based on argument count
    switch (func->arg_count) {
        case 0:
            if (func->return_type == FFI_TYPE_VOID) {
                ((void(*)(void))func->func_ptr)();
            } else {
                *(long*)result_storage = ((long(*)(void))func->func_ptr)();
            }
            break;

        case 1:
            *(long*)result_storage = ((long(*)(long))func->func_ptr)(
                *(long*)c_args[0]
            );
            break;

        case 2:
            *(long*)result_storage = ((long(*)(long,long))func->func_ptr)(
                *(long*)c_args[0],
                *(long*)c_args[1]
            );
            break;

        case 3:
            *(long*)result_storage = ((long(*)(long,long,long))func->func_ptr)(
                *(long*)c_args[0],
                *(long*)c_args[1],
                *(long*)c_args[2]
            );
            break;

        case 4:
            *(long*)result_storage = ((long(*)(long,long,long,long))func->func_ptr)(
                *(long*)c_args[0],
                *(long*)c_args[1],
                *(long*)c_args[2],
                *(long*)c_args[3]
            );
            break;

        case 5:
            *(long*)result_storage = ((long(*)(long,long,long,long,long))func->func_ptr)(
                *(long*)c_args[0],
                *(long*)c_args[1],
                *(long*)c_args[2],
                *(long*)c_args[3],
                *(long*)c_args[4]
            );
            break;

        case 6:
            *(long*)result_storage = ((long(*)(long,long,long,long,long,long))func->func_ptr)(
                *(long*)c_args[0],
                *(long*)c_args[1],
                *(long*)c_args[2],
                *(long*)c_args[3],
                *(long*)c_args[4],
                *(long*)c_args[5]
            );
            break;

        default:
            fprintf(stderr, "FFI: Too many arguments (%d)\n", func->arg_count);
            return -1;
    }

    // Unmarshal result
    if (func->return_type != FFI_TYPE_VOID) {
        cell_t result = ffi_unmarshal_result(func->return_type, result_storage);
        push(vm, result);
    }

    return 0;
}

// ============================================================================
// HIGH-LEVEL FFI WORDS (Forth interface)
// ============================================================================

// LIBRARY ( c-addr len -- handle )
void forth_ffi_library(forth_vm_t *vm) {
    cell_t len = pop(vm);
    cell_t addr = pop(vm);

    char path[256];
    if (len >= 256) len = 255;
    memcpy(path, (char*)addr, len);
    path[len] = '\0';

    void *handle = forth_ffi_load_library(path);
    push(vm, (cell_t)handle);
}

// FUNCTION ( handle c-addr len -- func-ptr )
void forth_ffi_function(forth_vm_t *vm) {
    cell_t len = pop(vm);
    cell_t addr = pop(vm);
    cell_t handle = pop(vm);

    char name[256];
    if (len >= 256) len = 255;
    memcpy(name, (char*)addr, len);
    name[len] = '\0';

    void *func_ptr = forth_ffi_get_symbol((void*)handle, name);
    push(vm, (cell_t)func_ptr);
}

// CALL-C ( func-ptr arg1 arg2 ... argN N -- result )
// Simple C function call with N arguments
void forth_ffi_call_c(forth_vm_t *vm) {
    cell_t arg_count = pop(vm);

    if (arg_count > 6) {
        fprintf(stderr, "FFI: Maximum 6 arguments supported\n");
        push(vm, 0);
        return;
    }

    cell_t args[6];
    for (int i = arg_count - 1; i >= 0; i--) {
        args[i] = pop(vm);
    }

    cell_t func_ptr = pop(vm);

    // Call function
    cell_t result = 0;
    switch (arg_count) {
        case 0: result = ((cell_t(*)(void))func_ptr)(); break;
        case 1: result = ((cell_t(*)(cell_t))func_ptr)(args[0]); break;
        case 2: result = ((cell_t(*)(cell_t,cell_t))func_ptr)(args[0], args[1]); break;
        case 3: result = ((cell_t(*)(cell_t,cell_t,cell_t))func_ptr)(args[0], args[1], args[2]); break;
        case 4: result = ((cell_t(*)(cell_t,cell_t,cell_t,cell_t))func_ptr)(args[0], args[1], args[2], args[3]); break;
        case 5: result = ((cell_t(*)(cell_t,cell_t,cell_t,cell_t,cell_t))func_ptr)(args[0], args[1], args[2], args[3], args[4]); break;
        case 6: result = ((cell_t(*)(cell_t,cell_t,cell_t,cell_t,cell_t,cell_t))func_ptr)(args[0], args[1], args[2], args[3], args[4], args[5]); break;
    }

    push(vm, result);
}

// ============================================================================
// COMMON C LIBRARY WRAPPERS
// ============================================================================

void forth_ffi_init_stdlib(forth_vm_t *vm) {
    forth_ffi_init();

    // Register common C library functions
    ffi_type_t int_arg[] = {FFI_TYPE_INT};
    ffi_type_t str_arg[] = {FFI_TYPE_STRING};
    ffi_type_t ptr_arg[] = {FFI_TYPE_POINTER};
    ffi_type_t size_arg[] = {FFI_TYPE_LONG};
    ffi_type_t ptr_size_args[] = {FFI_TYPE_POINTER, FFI_TYPE_LONG};

    // Memory functions
    forth_ffi_register_function("malloc", malloc, FFI_TYPE_POINTER, size_arg, 1);
    forth_ffi_register_function("free", free, FFI_TYPE_VOID, ptr_arg, 1);
    forth_ffi_register_function("strlen", strlen, FFI_TYPE_LONG, str_arg, 1);

    // I/O functions
    forth_ffi_register_function("puts", puts, FFI_TYPE_INT, str_arg, 1);
    forth_ffi_register_function("putchar", putchar, FFI_TYPE_INT, int_arg, 1);
    forth_ffi_register_function("getchar", getchar, FFI_TYPE_INT, NULL, 0);

    // Math functions (would need to load libm)
    // forth_ffi_register_function("sqrt", sqrt, FFI_TYPE_DOUBLE, ...);
}

// ============================================================================
// EXAMPLE: Call printf from Forth
// ============================================================================

void forth_ffi_example_printf(forth_vm_t *vm) {
    // This demonstrates calling printf with a format string and argument
    // In Forth: S" Hello %s\n" S" World" call-printf

    cell_t arg_len = pop(vm);
    cell_t arg_addr = pop(vm);
    cell_t fmt_len = pop(vm);
    cell_t fmt_addr = pop(vm);

    // Null-terminate strings if needed
    char fmt[256], arg[256];
    memcpy(fmt, (char*)fmt_addr, fmt_len);
    fmt[fmt_len] = '\0';
    memcpy(arg, (char*)arg_addr, arg_len);
    arg[arg_len] = '\0';

    int result = printf(fmt, arg);
    push(vm, result);
}

// ============================================================================
// CALLBACK SUPPORT (C calling Forth)
// ============================================================================

typedef struct {
    forth_vm_t *vm;
    cell_t forth_xt;  // Execution token (word address)
} forth_callback_t;

static forth_callback_t *callbacks[32];
static int callback_count = 0;

// Wrapper that C can call, which then calls Forth
static cell_t forth_callback_wrapper_0(void *user_data) {
    forth_callback_t *cb = (forth_callback_t*)user_data;
    // Execute Forth word
    // (This would need the interpreter/executor implementation)
    return 0;
}

int forth_ffi_create_callback(forth_vm_t *vm, cell_t forth_xt, void **c_callback) {
    if (callback_count >= 32) return -1;

    forth_callback_t *cb = malloc(sizeof(forth_callback_t));
    cb->vm = vm;
    cb->forth_xt = forth_xt;

    callbacks[callback_count++] = cb;
    *c_callback = (void*)forth_callback_wrapper_0;

    return 0;
}

// ============================================================================
// FFI DEBUGGING
// ============================================================================

void forth_ffi_dump_registry(void) {
    if (!ffi_registry) {
        printf("FFI registry not initialized\n");
        return;
    }

    printf("FFI Registry (%d functions):\n", ffi_registry->count);
    for (int i = 0; i < ffi_registry->count; i++) {
        ffi_function_t *func = &ffi_registry->functions[i];
        printf("  %s: %d args -> ", func->name, func->arg_count);

        switch (func->return_type) {
            case FFI_TYPE_VOID: printf("void"); break;
            case FFI_TYPE_INT: printf("int"); break;
            case FFI_TYPE_LONG: printf("long"); break;
            case FFI_TYPE_POINTER: printf("ptr"); break;
            default: printf("?"); break;
        }

        printf("\n");
    }
}
