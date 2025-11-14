/**
 * Fast Forth FFI Example
 * Demonstrates calling C functions from Forth
 */

#include "../runtime/forth_runtime.h"
#include <stdio.h>
#include <math.h>

// Example C functions to call from Forth
cell_t add_numbers(cell_t a, cell_t b) {
    return a + b;
}

cell_t factorial(cell_t n) {
    if (n <= 1) return 1;
    return n * factorial(n - 1);
}

void print_message(cell_t str_addr, cell_t str_len) {
    printf("C function says: %.*s\n", (int)str_len, (char*)str_addr);
}

double compute_sqrt(double x) {
    return sqrt(x);
}

// External FFI declarations
extern void forth_ffi_init(void);
extern int forth_ffi_register_function(const char*, void*, int, const int*, int);
extern int forth_bootstrap(forth_vm_t*);
extern int forth_interpret(forth_vm_t*, const char*);

int main(void) {
    // Create Forth VM
    forth_vm_t *vm = forth_create();
    if (!vm) {
        fprintf(stderr, "Failed to create VM\n");
        return 1;
    }

    // Bootstrap runtime
    forth_bootstrap(vm);

    printf("\n=== Fast Forth FFI Example ===\n\n");

    // Register C functions with FFI
    forth_ffi_init();

    // Simple example: Call C function from Forth
    printf("1. Simple C function call:\n");
    printf("   Forth: 10 15 add_numbers call-c\n");

    // Push arguments and function pointer
    push(vm, (cell_t)add_numbers);
    push(vm, 10);
    push(vm, 15);
    push(vm, 2);  // 2 arguments

    extern void forth_ffi_call_c(forth_vm_t*);
    forth_ffi_call_c(vm);

    printf("   Result: %ld\n\n", pop(vm));

    // Factorial example
    printf("2. Factorial function:\n");
    printf("   Forth: 6 factorial call-c\n");

    push(vm, (cell_t)factorial);
    push(vm, 6);
    push(vm, 1);  // 1 argument

    forth_ffi_call_c(vm);

    printf("   Result: %ld! = %ld\n\n", 6L, pop(vm));

    // String passing example
    printf("3. String passing to C:\n");
    const char *message = "Hello from Forth!";
    printf("   Forth: S\" %s\" print_message call-c\n", message);

    push(vm, (cell_t)print_message);
    push(vm, (cell_t)message);
    push(vm, strlen(message));
    push(vm, 2);  // 2 arguments

    forth_ffi_call_c(vm);

    // Dynamic library loading example
    printf("\n4. Dynamic library loading:\n");
    printf("   Loading libm.so for math functions...\n");

    extern void *forth_ffi_load_library(const char*);
    extern void *forth_ffi_get_symbol(void*, const char*);

#ifdef __APPLE__
    void *libm = forth_ffi_load_library("libm.dylib");
#else
    void *libm = forth_ffi_load_library("libm.so.6");
#endif

    if (libm) {
        void *sqrt_func = forth_ffi_get_symbol(libm, "sqrt");
        if (sqrt_func) {
            printf("   Found sqrt function!\n");

            // Call sqrt(16.0)
            double input = 16.0;
            double (*sqrt_ptr)(double) = (double(*)(double))sqrt_func;
            double result = sqrt_ptr(input);

            printf("   sqrt(%.1f) = %.1f\n", input, result);
        }
    }

    // Demonstrate FFI with arrays
    printf("\n5. Array processing:\n");
    cell_t array[5] = {1, 2, 3, 4, 5};
    cell_t sum = 0;

    printf("   Array: [");
    for (int i = 0; i < 5; i++) {
        printf("%ld%s", array[i], i < 4 ? ", " : "");
        sum += array[i];
    }
    printf("]\n");
    printf("   Sum: %ld\n", sum);

    // Complete example: Forth program using FFI
    printf("\n6. Complete Forth program with FFI:\n");
    printf("-----------------------------------\n");

    const char *forth_code =
        ": SQUARED  DUP * ;\n"
        ": CUBED    DUP SQUARED * ;\n"
        "5 SQUARED .\n"
        "3 CUBED .\n";

    printf("Forth code:\n%s\n", forth_code);
    printf("Output: ");

    forth_interpret(vm, forth_code);

    printf("\n");

    // Cleanup
    forth_destroy(vm);
    extern void forth_ffi_cleanup(void);
    forth_ffi_cleanup();

    printf("\nFFI example complete!\n");

    return 0;
}
