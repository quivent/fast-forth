/**
 * FFI bindings to C runtime
 *
 * This module provides safe Rust bindings to the C runtime functions.
 * The C code is compiled by build.rs and linked as libforthruntime.a
 */

use std::os::raw::{c_char, c_int, c_void};

// Link to the static library compiled by build.rs
#[link(name = "forthruntime", kind = "static")]
extern "C" {}

// Re-export pthread which is required by concurrency.c
#[link(name = "pthread")]
extern "C" {}

pub type CellT = isize;

#[repr(C)]
pub struct ForthVM {
    _private: [u8; 0],
}

#[repr(C)]
pub struct WordHeader {
    pub link: *mut WordHeader,
    pub flags: u8,
    pub name_len: u8,
}

extern "C" {
    // VM lifecycle
    pub fn forth_create() -> *mut ForthVM;
    pub fn forth_destroy(vm: *mut ForthVM);
    pub fn forth_reset(vm: *mut ForthVM) -> c_int;

    // Stack operations (wrappers for inline functions)
    pub fn test_push(vm: *mut ForthVM, value: CellT);
    pub fn test_pop(vm: *mut ForthVM) -> CellT;
    pub fn test_peek(vm: *mut ForthVM) -> CellT;
    pub fn test_rpush(vm: *mut ForthVM, value: CellT);
    pub fn test_rpop(vm: *mut ForthVM) -> CellT;
    pub fn test_depth(vm: *mut ForthVM) -> c_int;
    pub fn test_rdepth(vm: *mut ForthVM) -> c_int;

    // Dictionary operations
    pub fn forth_find_word(vm: *mut ForthVM, name: *const c_char, len: usize) -> *mut WordHeader;
    pub fn forth_define_word(
        vm: *mut ForthVM,
        name: *const c_char,
        code: extern "C" fn(*mut ForthVM),
        flags: u8,
    );

    // Arithmetic
    pub fn forth_add(vm: *mut ForthVM);
    pub fn forth_sub(vm: *mut ForthVM);
    pub fn forth_mul(vm: *mut ForthVM);
    pub fn forth_div(vm: *mut ForthVM);
    pub fn forth_mod(vm: *mut ForthVM);
    pub fn forth_divmod(vm: *mut ForthVM);

    // Stack manipulation
    pub fn forth_dup(vm: *mut ForthVM);
    pub fn forth_drop(vm: *mut ForthVM);
    pub fn forth_swap(vm: *mut ForthVM);
    pub fn forth_over(vm: *mut ForthVM);
    pub fn forth_rot(vm: *mut ForthVM);
    pub fn forth_pick(vm: *mut ForthVM);
    pub fn forth_roll(vm: *mut ForthVM);

    // Memory operations
    pub fn forth_fetch(vm: *mut ForthVM);
    pub fn forth_store(vm: *mut ForthVM);
    pub fn forth_allot(vm: *mut ForthVM);
    pub fn forth_here(vm: *mut ForthVM);

    // Memory utilities
    pub fn forth_valid_address(vm: *mut ForthVM, addr: CellT, size: usize) -> bool;
    pub fn forth_move(vm: *mut ForthVM);
    pub fn forth_fill(vm: *mut ForthVM);
    pub fn forth_erase(vm: *mut ForthVM);

    // Concurrency primitives
    pub fn forth_spawn(vm: *mut ForthVM, xt: CellT) -> CellT;
    pub fn forth_join(vm: *mut ForthVM, thread_id: CellT);
    pub fn forth_channel_create(capacity: usize) -> CellT;
    pub fn forth_channel_send(value: CellT, chan: CellT);
    pub fn forth_channel_recv(chan: CellT) -> CellT;
    pub fn forth_channel_close(chan: CellT);
    pub fn forth_channel_destroy(chan: CellT);

    // FFI support
    pub fn forth_ffi_call(vm: *mut ForthVM, func_ptr: *mut c_void, arg_count: c_int) -> c_int;

    // Debugging
    pub fn forth_dump_stack(vm: *mut ForthVM);
    pub fn forth_dump_dictionary(vm: *mut ForthVM);
}
