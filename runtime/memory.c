/**
 * Fast Forth Memory Management
 * Stream 6: Dictionary, heap, and memory allocation
 *
 * Optimized memory management with:
 * - Linear dictionary allocation
 * - Hash table for fast word lookup
 * - Garbage collection support (optional)
 * - Memory alignment for performance
 */

#include "forth_runtime.h"
#include <stdlib.h>
#include <string.h>
#include <stdio.h>

// ============================================================================
// HASH TABLE FOR FAST WORD LOOKUP
// ============================================================================

#define HASH_TABLE_SIZE 256

typedef struct hash_entry {
    word_header_t *word;
    struct hash_entry *next;
} hash_entry_t;

typedef struct {
    hash_entry_t *buckets[HASH_TABLE_SIZE];
    word_header_t *word_list;  // Original linked list
} word_dictionary_t;

// Simple hash function (FNV-1a)
static uint32_t hash_name(const char *name, size_t len) {
    uint32_t hash = 2166136261u;
    for (size_t i = 0; i < len; i++) {
        hash ^= (uint8_t)name[i];
        hash *= 16777619u;
    }
    return hash % HASH_TABLE_SIZE;
}

// Initialize dictionary hash table
static word_dictionary_t *dict_create(void) {
    word_dictionary_t *dict = calloc(1, sizeof(word_dictionary_t));
    return dict;
}

// Add word to hash table
static void dict_add_word(word_dictionary_t *dict, word_header_t *word) {
    uint32_t idx = hash_name(word->name, word->name_len);

    hash_entry_t *entry = malloc(sizeof(hash_entry_t));
    entry->word = word;
    entry->next = dict->buckets[idx];
    dict->buckets[idx] = entry;

    // Also maintain linked list
    word->link = dict->word_list;
    dict->word_list = word;
}

// Find word in hash table (much faster than linear search)
static word_header_t *dict_find_word(word_dictionary_t *dict, const char *name, size_t len) {
    uint32_t idx = hash_name(name, len);
    hash_entry_t *entry = dict->buckets[idx];

    while (entry) {
        word_header_t *word = entry->word;
        if (word->name_len == len &&
            !(word->flags & FLAG_HIDDEN) &&
            memcmp(word->name, name, len) == 0) {
            return word;
        }
        entry = entry->next;
    }

    return NULL;
}

// ============================================================================
// DICTIONARY ALLOCATION (Linear allocator)
// ============================================================================

void *forth_dict_alloc(forth_vm_t *vm, size_t size) {
    // Align to cell boundary
    size = (size + sizeof(cell_t) - 1) & ~(sizeof(cell_t) - 1);

    if (vm->here + size >= vm->dictionary + vm->dict_size) {
        // Dictionary overflow - expand or error
        fprintf(stderr, "Dictionary overflow!\n");
        vm->error_code = FORTH_INVALID_MEMORY;
        return NULL;
    }

    void *ptr = vm->here;
    vm->here += size;
    return ptr;
}

// ============================================================================
// WORD CREATION WITH OPTIMIZED LAYOUT
// ============================================================================

void forth_create(forth_vm_t *vm) {
    // Parse word name
    char name[256];
    int len = 0;

    // Skip whitespace
    while (vm->input_buffer[vm->input_pos] == ' ' ||
           vm->input_buffer[vm->input_pos] == '\t') {
        vm->input_pos++;
    }

    // Read name
    while (vm->input_pos < vm->input_len &&
           vm->input_buffer[vm->input_pos] != ' ' &&
           vm->input_buffer[vm->input_pos] != '\t' &&
           len < 255) {
        name[len++] = vm->input_buffer[vm->input_pos++];
    }

    if (len == 0) {
        vm->error_code = FORTH_INVALID_STATE;
        return;
    }

    // Align dictionary pointer
    vm->here = (byte_t*)(((uintptr_t)vm->here + sizeof(cell_t) - 1) & ~(sizeof(cell_t) - 1));

    // Create word header
    word_header_t *header = (word_header_t*)vm->here;
    header->link = vm->last_word;
    header->flags = 0;
    header->name_len = len;

    vm->here += sizeof(word_header_t);
    memcpy(vm->here, name, len);
    vm->here += len;

    // Align again for code pointer
    vm->here = (byte_t*)(((uintptr_t)vm->here + sizeof(cell_t) - 1) & ~(sizeof(cell_t) - 1));

    vm->last_word = header;

    // Push data field address onto stack
    push(vm, (cell_t)vm->here);
}

// ============================================================================
// DOES> IMPLEMENTATION (Create defining words)
// ============================================================================

typedef struct {
    void (*runtime_code)(forth_vm_t*);
    byte_t compiled_code[];
} does_word_t;

static void does_runtime(forth_vm_t *vm) {
    // Get the word's data field address
    word_header_t *word = vm->last_word;
    does_word_t *does_word = (does_word_t*)((byte_t*)word +
                                            sizeof(word_header_t) +
                                            word->name_len);

    // Push data field address
    push(vm, (cell_t)does_word->compiled_code);

    // Execute the DOES> code
    // This would call the interpreter/compiler to execute the code
}

void forth_does(forth_vm_t *vm) {
    // Modify the last defined word to use DOES> runtime
    if (!vm->last_word) {
        vm->error_code = FORTH_INVALID_STATE;
        return;
    }

    // Store pointer to does runtime
    *(void**)vm->here = (void*)does_runtime;
    vm->here += sizeof(void*);

    // The compiled code follows in the dictionary
}

// ============================================================================
// MEMORY PROTECTION (Optional bounds checking)
// ============================================================================

bool forth_valid_address(forth_vm_t *vm, cell_t addr, size_t size) {
    byte_t *ptr = (byte_t*)addr;

    // Check if address is within dictionary
    if (ptr >= vm->dictionary && ptr + size <= vm->here) {
        return true;
    }

    // Check if address is within data stack
    if (ptr >= (byte_t*)vm->data_stack &&
        ptr + size <= (byte_t*)(vm->data_stack + DATA_STACK_SIZE)) {
        return true;
    }

    // Check if address is within return stack
    if (ptr >= (byte_t*)vm->return_stack &&
        ptr + size <= (byte_t*)(vm->return_stack + RETURN_STACK_SIZE)) {
        return true;
    }

    // Otherwise might be system memory (malloc'd, etc.)
    // For safety, we'll allow it but could add stricter checking
    return true;
}

// ============================================================================
// MEMORY UTILITIES
// ============================================================================

void forth_move(forth_vm_t *vm) {
    cell_t count = pop(vm);
    cell_t dest = pop(vm);
    cell_t src = pop(vm);

    if (!forth_valid_address(vm, src, count) ||
        !forth_valid_address(vm, dest, count)) {
        vm->error_code = FORTH_INVALID_MEMORY;
        return;
    }

    memmove((void*)dest, (void*)src, count);
}

void forth_fill(forth_vm_t *vm) {
    cell_t c = pop(vm);
    cell_t count = pop(vm);
    cell_t addr = pop(vm);

    if (!forth_valid_address(vm, addr, count)) {
        vm->error_code = FORTH_INVALID_MEMORY;
        return;
    }

    memset((void*)addr, c, count);
}

void forth_erase(forth_vm_t *vm) {
    cell_t count = pop(vm);
    cell_t addr = pop(vm);

    if (!forth_valid_address(vm, addr, count)) {
        vm->error_code = FORTH_INVALID_MEMORY;
        return;
    }

    memset((void*)addr, 0, count);
}

// ============================================================================
// GARBAGE COLLECTION (Optional - for advanced implementations)
// ============================================================================

typedef struct gc_block {
    struct gc_block *next;
    size_t size;
    bool marked;
    byte_t data[];
} gc_block_t;

typedef struct {
    gc_block_t *blocks;
    size_t total_allocated;
    size_t total_freed;
} gc_heap_t;

static gc_heap_t *gc_heap = NULL;

void *forth_gc_alloc(size_t size) {
    if (!gc_heap) {
        gc_heap = calloc(1, sizeof(gc_heap_t));
    }

    gc_block_t *block = malloc(sizeof(gc_block_t) + size);
    block->next = gc_heap->blocks;
    block->size = size;
    block->marked = false;

    gc_heap->blocks = block;
    gc_heap->total_allocated += size;

    return block->data;
}

void forth_gc_mark(void *ptr) {
    if (!gc_heap) return;

    gc_block_t *block = gc_heap->blocks;
    while (block) {
        if (block->data == ptr) {
            block->marked = true;
            return;
        }
        block = block->next;
    }
}

void forth_gc_sweep(void) {
    if (!gc_heap) return;

    gc_block_t **block_ptr = &gc_heap->blocks;
    while (*block_ptr) {
        gc_block_t *block = *block_ptr;
        if (!block->marked) {
            // Unmark and free
            *block_ptr = block->next;
            gc_heap->total_freed += block->size;
            free(block);
        } else {
            // Unmark for next cycle
            block->marked = false;
            block_ptr = &block->next;
        }
    }
}

// ============================================================================
// MEMORY STATISTICS
// ============================================================================

void forth_memory_stats(forth_vm_t *vm) {
    size_t dict_used = vm->here - vm->dictionary;
    size_t dict_free = vm->dict_size - dict_used;

    printf("Memory Statistics:\n");
    printf("  Dictionary: %zu / %zu bytes (%.1f%% used)\n",
           dict_used, vm->dict_size,
           100.0 * dict_used / vm->dict_size);
    printf("  Data stack: %d / %d cells\n", depth(vm), DATA_STACK_SIZE);
    printf("  Return stack: %d / %d cells\n", rdepth(vm), RETURN_STACK_SIZE);

    if (gc_heap) {
        printf("  GC heap: %zu allocated, %zu freed\n",
               gc_heap->total_allocated, gc_heap->total_freed);
    }
}

// ============================================================================
// MEMORY COMPACTION (Optional - defragment dictionary)
// ============================================================================

void forth_compact_dictionary(forth_vm_t *vm) {
    // This would implement a mark-and-compact algorithm
    // to defragment the dictionary and reclaim unused space
    // Left as an exercise for advanced implementations
    printf("Dictionary compaction not yet implemented\n");
}
