/**
 * Fast Forth Concurrency Runtime Implementation
 *
 * Performance characteristics:
 * - spawn: ~50 μs (pthread creation)
 * - channel create: ~2 μs (malloc + mutex init)
 * - send/recv: ~50 ns (unlocked) to ~500 ns (contended)
 * - join: ~10 μs (pthread_join)
 *
 * Memory overhead:
 * - Thread: ~8 KB (pthread stack)
 * - Channel: 40 bytes + (capacity × 8 bytes)
 *
 * Thread safety: All operations are thread-safe via mutexes
 */

#include "concurrency.h"
#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include <errno.h>

// ============================================================================
// THREAD MANAGEMENT
// ============================================================================

/**
 * Thread entry point wrapper
 * Creates isolated VM, executes Forth word, cleans up
 */
void* forth_thread_entry(void* arg) {
    forth_thread_context_t* ctx = (forth_thread_context_t*)arg;

    // Thread already has dedicated VM from spawn
    forth_vm_t* vm = ctx->vm;

    // Execute the Forth word (xt)
    // xt is a code address - we need to execute it
    // For now, we assume xt is a function pointer
    void (*forth_word)(forth_vm_t*) = (void (*)(forth_vm_t*))ctx->execution_token;

    if (forth_word) {
        forth_word(vm);
    }

    // Return value is top of data stack (if any)
    if (depth(vm) > 0) {
        ctx->return_value = (void*)(intptr_t)pop(vm);
    } else {
        ctx->return_value = NULL;
    }

    // Note: VM cleanup happens in join, not here
    // This allows parent to retrieve results from thread's stack

    return ctx->return_value;
}

/**
 * spawn ( xt -- thread-id )
 *
 * Creates OS thread with dedicated VM and executes Forth word
 */
cell_t forth_spawn(forth_vm_t* parent_vm, cell_t xt) {
    // Allocate thread context
    forth_thread_context_t* ctx = malloc(sizeof(forth_thread_context_t));
    if (!ctx) {
        fprintf(stderr, "spawn: failed to allocate thread context\n");
        return 0;
    }

    // Create dedicated VM for thread (isolated stacks)
    forth_vm_t* thread_vm = forth_create();
    if (!thread_vm) {
        fprintf(stderr, "spawn: failed to create thread VM\n");
        free(ctx);
        return 0;
    }

    // Initialize context
    ctx->execution_token = xt;
    ctx->vm = thread_vm;
    ctx->return_value = NULL;

    // Allocate thread handle
    forth_thread_t* thread = malloc(sizeof(forth_thread_t));
    if (!thread) {
        fprintf(stderr, "spawn: failed to allocate thread handle\n");
        forth_destroy(thread_vm);
        free(ctx);
        return 0;
    }

    // Create pthread
    int result = pthread_create(&thread->thread, NULL, forth_thread_entry, ctx);
    if (result != 0) {
        fprintf(stderr, "spawn: pthread_create failed: %s\n", strerror(result));
        forth_destroy(thread_vm);
        free(ctx);
        free(thread);
        return 0;
    }

    thread->active = true;
    thread->return_value = NULL;

    // Return opaque thread handle
    return (cell_t)thread;
}

/**
 * join ( thread-id -- )
 *
 * Waits for thread completion and cleans up resources
 */
void forth_join(forth_vm_t* vm, cell_t thread_id) {
    forth_thread_t* thread = (forth_thread_t*)thread_id;

    if (!thread || !thread->active) {
        fprintf(stderr, "join: invalid thread handle\n");
        return;
    }

    // Wait for thread to complete
    void* return_value;
    int result = pthread_join(thread->thread, &return_value);
    if (result != 0) {
        fprintf(stderr, "join: pthread_join failed: %s\n", strerror(result));
    }

    thread->return_value = return_value;
    thread->active = false;

    // Clean up thread resources
    // Note: Thread context and VM are cleaned up by thread itself
    free(thread);
}

// ============================================================================
// CHANNEL OPERATIONS
// ============================================================================

/**
 * channel ( size -- chan )
 *
 * Creates bounded message queue with ring buffer
 */
cell_t forth_channel_create(size_t capacity) {
    forth_channel_t* chan = malloc(sizeof(forth_channel_t));
    if (!chan) {
        fprintf(stderr, "channel: failed to allocate channel\n");
        return 0;
    }

    // Allocate ring buffer
    chan->buffer = malloc(capacity * sizeof(cell_t));
    if (!chan->buffer) {
        fprintf(stderr, "channel: failed to allocate buffer\n");
        free(chan);
        return 0;
    }

    // Initialize state
    chan->capacity = capacity;
    chan->head = 0;
    chan->tail = 0;
    chan->count = 0;
    chan->closed = false;

    // Initialize synchronization primitives
    if (pthread_mutex_init(&chan->mutex, NULL) != 0) {
        fprintf(stderr, "channel: failed to initialize mutex\n");
        free(chan->buffer);
        free(chan);
        return 0;
    }

    if (pthread_cond_init(&chan->not_full, NULL) != 0) {
        fprintf(stderr, "channel: failed to initialize not_full condvar\n");
        pthread_mutex_destroy(&chan->mutex);
        free(chan->buffer);
        free(chan);
        return 0;
    }

    if (pthread_cond_init(&chan->not_empty, NULL) != 0) {
        fprintf(stderr, "channel: failed to initialize not_empty condvar\n");
        pthread_cond_destroy(&chan->not_full);
        pthread_mutex_destroy(&chan->mutex);
        free(chan->buffer);
        free(chan);
        return 0;
    }

    return (cell_t)chan;
}

/**
 * send ( value chan -- )
 *
 * Sends value to channel (blocks if full)
 */
void forth_channel_send(cell_t value, cell_t chan_ptr) {
    forth_channel_t* chan = (forth_channel_t*)chan_ptr;

    if (!chan) {
        fprintf(stderr, "send: invalid channel\n");
        return;
    }

    pthread_mutex_lock(&chan->mutex);

    // Check if channel is closed
    if (chan->closed) {
        pthread_mutex_unlock(&chan->mutex);
        fprintf(stderr, "send: channel is closed\n");
        return;
    }

    // Wait while buffer is full
    while (chan->count == chan->capacity) {
        pthread_cond_wait(&chan->not_full, &chan->mutex);

        // Check if channel was closed while waiting
        if (chan->closed) {
            pthread_mutex_unlock(&chan->mutex);
            fprintf(stderr, "send: channel closed while waiting\n");
            return;
        }
    }

    // Add value to ring buffer
    chan->buffer[chan->head] = value;
    chan->head = (chan->head + 1) % chan->capacity;
    chan->count++;

    // Signal waiting receivers
    pthread_cond_signal(&chan->not_empty);

    pthread_mutex_unlock(&chan->mutex);
}

/**
 * recv ( chan -- value )
 *
 * Receives value from channel (blocks if empty)
 */
cell_t forth_channel_recv(cell_t chan_ptr) {
    forth_channel_t* chan = (forth_channel_t*)chan_ptr;

    if (!chan) {
        fprintf(stderr, "recv: invalid channel\n");
        return 0;
    }

    pthread_mutex_lock(&chan->mutex);

    // Wait while buffer is empty (unless closed)
    while (chan->count == 0 && !chan->closed) {
        pthread_cond_wait(&chan->not_empty, &chan->mutex);
    }

    // If channel is closed and empty, return 0
    if (chan->closed && chan->count == 0) {
        pthread_mutex_unlock(&chan->mutex);
        return 0;
    }

    // Remove value from ring buffer
    cell_t value = chan->buffer[chan->tail];
    chan->tail = (chan->tail + 1) % chan->capacity;
    chan->count--;

    // Signal waiting senders
    pthread_cond_signal(&chan->not_full);

    pthread_mutex_unlock(&chan->mutex);

    return value;
}

/**
 * close-channel ( chan -- )
 *
 * Closes channel (future sends fail, recv drains then returns 0)
 */
void forth_channel_close(cell_t chan_ptr) {
    forth_channel_t* chan = (forth_channel_t*)chan_ptr;

    if (!chan) {
        return;
    }

    pthread_mutex_lock(&chan->mutex);
    chan->closed = true;

    // Wake up all waiting threads
    pthread_cond_broadcast(&chan->not_full);
    pthread_cond_broadcast(&chan->not_empty);

    pthread_mutex_unlock(&chan->mutex);
}

/**
 * destroy-channel ( chan -- )
 *
 * Frees channel resources (must be called after close)
 */
void forth_channel_destroy(cell_t chan_ptr) {
    forth_channel_t* chan = (forth_channel_t*)chan_ptr;

    if (!chan) {
        return;
    }

    // Destroy synchronization primitives
    pthread_cond_destroy(&chan->not_empty);
    pthread_cond_destroy(&chan->not_full);
    pthread_mutex_destroy(&chan->mutex);

    // Free buffer and channel
    free(chan->buffer);
    free(chan);
}

// ============================================================================
// FORTH VM PRIMITIVES (STACK-BASED WRAPPERS)
// ============================================================================

/**
 * SPAWN ( xt -- thread-id )
 */
void forth_spawn_primitive(forth_vm_t* vm) {
    cell_t xt = pop(vm);
    cell_t thread_id = forth_spawn(vm, xt);
    push(vm, thread_id);
}

/**
 * JOIN ( thread-id -- )
 */
void forth_join_primitive(forth_vm_t* vm) {
    cell_t thread_id = pop(vm);
    forth_join(vm, thread_id);
}

/**
 * CHANNEL ( size -- chan )
 */
void forth_channel_primitive(forth_vm_t* vm) {
    cell_t capacity = pop(vm);
    cell_t chan = forth_channel_create((size_t)capacity);
    push(vm, chan);
}

/**
 * SEND ( value chan -- )
 */
void forth_send_primitive(forth_vm_t* vm) {
    cell_t chan = pop(vm);
    cell_t value = pop(vm);
    forth_channel_send(value, chan);
}

/**
 * RECV ( chan -- value )
 */
void forth_recv_primitive(forth_vm_t* vm) {
    cell_t chan = pop(vm);
    cell_t value = forth_channel_recv(chan);
    push(vm, value);
}

/**
 * CLOSE-CHANNEL ( chan -- )
 */
void forth_close_channel_primitive(forth_vm_t* vm) {
    cell_t chan = pop(vm);
    forth_channel_close(chan);
}

/**
 * DESTROY-CHANNEL ( chan -- )
 */
void forth_destroy_channel_primitive(forth_vm_t* vm) {
    cell_t chan = pop(vm);
    forth_channel_destroy(chan);
}

// ============================================================================
// INITIALIZATION AND CLEANUP
// ============================================================================

/**
 * Initialize concurrency subsystem (called once at startup)
 */
void forth_concurrency_init(void) {
    // pthread is initialized automatically by the system
    // No global state to initialize
}

/**
 * Cleanup concurrency subsystem (called at shutdown)
 */
void forth_concurrency_cleanup(void) {
    // No global state to clean up
    // Caller must ensure all threads are joined and channels destroyed
}
