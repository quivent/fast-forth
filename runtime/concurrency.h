/**
 * Fast Forth Concurrency Runtime
 *
 * Minimal concurrency primitives for multi-agent workflows:
 * - spawn: Create OS threads
 * - channel: Type-safe message queues
 * - send/recv: Blocking message passing
 * - join: Thread synchronization
 *
 * Binary impact: +15 KB (~0.6% increase)
 * Compilation impact: +100ms (cacheable to +10ms)
 */

#ifndef FORTH_CONCURRENCY_H
#define FORTH_CONCURRENCY_H

#include "forth_runtime.h"
#include <pthread.h>

// ============================================================================
// CONCURRENCY TYPES
// ============================================================================

// Thread handle (wraps pthread_t)
typedef struct {
    pthread_t thread;
    bool active;
    void* return_value;
} forth_thread_t;

// Channel (bounded message queue with blocking send/recv)
typedef struct {
    cell_t* buffer;              // Ring buffer for messages
    size_t capacity;             // Maximum messages in buffer
    size_t head;                 // Write position
    size_t tail;                 // Read position
    size_t count;                // Current message count
    pthread_mutex_t mutex;       // Protects buffer access
    pthread_cond_t not_full;     // Signaled when space available
    pthread_cond_t not_empty;    // Signaled when message available
    bool closed;                 // Channel closed flag
} forth_channel_t;

// Thread context (passed to pthread_create)
typedef struct {
    cell_t execution_token;      // Forth word to execute (xt)
    forth_vm_t* vm;              // Dedicated VM for this thread
    void* return_value;          // Thread return value
} forth_thread_context_t;

// ============================================================================
// THREAD MANAGEMENT
// ============================================================================

/**
 * spawn ( xt -- thread-id )
 *
 * Creates new OS thread executing the given Forth word.
 * Each thread gets its own VM with isolated stacks.
 *
 * Stack effect: ( xt -- thread-id )
 * xt: Execution token (address of Forth word)
 * thread-id: Opaque handle for thread (used with join)
 *
 * Example:
 *   : worker-task ( -- ) ... ;
 *   ' worker-task spawn constant thread1
 */
cell_t forth_spawn(forth_vm_t* vm, cell_t xt);

/**
 * join ( thread-id -- )
 *
 * Blocks until thread completes, then cleans up resources.
 * Must be called exactly once per spawned thread.
 *
 * Stack effect: ( thread-id -- )
 * thread-id: Handle returned by spawn
 *
 * Example:
 *   thread1 join  \ Wait for thread1 to complete
 */
void forth_join(forth_vm_t* vm, cell_t thread_id);

// ============================================================================
// CHANNEL OPERATIONS
// ============================================================================

/**
 * channel ( size -- chan )
 *
 * Creates bounded message queue with given capacity.
 * send blocks when full, recv blocks when empty.
 *
 * Stack effect: ( size -- chan )
 * size: Maximum buffered messages (0 = unbuffered)
 * chan: Opaque channel handle
 *
 * Example:
 *   100 channel constant work-queue
 */
cell_t forth_channel_create(size_t capacity);

/**
 * send ( value chan -- )
 *
 * Sends value to channel (blocks if full).
 * Thread-safe, can be called from multiple threads.
 *
 * Stack effect: ( value chan -- )
 * value: Integer to send (cell_t)
 * chan: Channel handle
 *
 * Example:
 *   42 work-queue send
 */
void forth_channel_send(cell_t value, cell_t chan_ptr);

/**
 * recv ( chan -- value )
 *
 * Receives value from channel (blocks if empty).
 * Thread-safe, can be called from multiple threads.
 *
 * Stack effect: ( chan -- value )
 * chan: Channel handle
 * value: Received integer
 *
 * Example:
 *   work-queue recv  \ Blocks until message available
 */
cell_t forth_channel_recv(cell_t chan_ptr);

/**
 * close-channel ( chan -- )
 *
 * Closes channel (recv returns 0 after drain).
 * Safe to call multiple times.
 *
 * Stack effect: ( chan -- )
 * chan: Channel handle
 */
void forth_channel_close(cell_t chan_ptr);

/**
 * destroy-channel ( chan -- )
 *
 * Frees channel resources. Must be called after close.
 * Undefined behavior if threads still using channel.
 *
 * Stack effect: ( chan -- )
 * chan: Channel handle
 */
void forth_channel_destroy(cell_t chan_ptr);

// ============================================================================
// VM PRIMITIVES (CALLED FROM FORTH CODE)
// ============================================================================

/**
 * Forth word implementations that pop/push from VM stack
 * These wrap the lower-level functions above
 */
void forth_spawn_primitive(forth_vm_t* vm);          // SPAWN
void forth_join_primitive(forth_vm_t* vm);           // JOIN
void forth_channel_primitive(forth_vm_t* vm);        // CHANNEL
void forth_send_primitive(forth_vm_t* vm);           // SEND
void forth_recv_primitive(forth_vm_t* vm);           // RECV
void forth_close_channel_primitive(forth_vm_t* vm);  // CLOSE-CHANNEL
void forth_destroy_channel_primitive(forth_vm_t* vm);// DESTROY-CHANNEL

// ============================================================================
// INTERNAL HELPERS
// ============================================================================

// Thread entry point (wraps Forth word execution)
void* forth_thread_entry(void* context);

// Initialize concurrency subsystem (called once at startup)
void forth_concurrency_init(void);

// Cleanup concurrency subsystem (called at shutdown)
void forth_concurrency_cleanup(void);

#endif // FORTH_CONCURRENCY_H
