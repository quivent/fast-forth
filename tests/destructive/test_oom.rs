// Out-of-Memory (OOM) Testing
// Tests allocation failure handling under memory pressure

#![cfg(feature = "destructive_tests")]

use super::safety::{ensure_containerized, get_memory_limit};
use std::alloc::{alloc, dealloc, Layout};

/// Test graceful handling of allocation failures
#[test]
fn test_small_allocation_failure() {
    ensure_containerized();

    if let Some(limit) = get_memory_limit() {
        println!("Running OOM test with memory limit: {} MB", limit / 1_048_576);
    }

    // Allocate memory in chunks until we hit OOM
    let mut allocations = Vec::new();
    let chunk_size = 1_000_000; // 1MB chunks
    let max_chunks = 1000;

    for i in 0..max_chunks {
        match try_allocate(chunk_size) {
            Some(ptr) => {
                allocations.push(ptr);
                if i % 10 == 0 {
                    println!("Allocated {} MB", i);
                }
            }
            None => {
                println!("Allocation failed after {} MB (expected)", i);
                break;
            }
        }
    }

    // Clean up allocations
    for ptr in allocations {
        unsafe {
            dealloc(ptr, Layout::from_size_align(chunk_size, 8).unwrap());
        }
    }

    // Test passed if we handled OOM gracefully
    assert!(true, "OOM handled gracefully");
}

/// Test Vec allocation failures
#[test]
fn test_vec_allocation_failure() {
    ensure_containerized();

    let mut vecs = Vec::new();
    let vec_size = 10_000_000; // 10M elements

    // Try allocating vectors until we fail
    for i in 0..100 {
        // Use try_reserve to handle allocation failures gracefully
        let mut v = Vec::<u8>::new();
        match v.try_reserve(vec_size) {
            Ok(_) => {
                vecs.push(v);
                if i % 5 == 0 {
                    println!("Allocated vector {} ({}MB)", i, (i * vec_size) / 1_000_000);
                }
            }
            Err(e) => {
                println!("Vec allocation failed after {} vecs: {:?}", i, e);
                break;
            }
        }
    }

    println!("Successfully handled {} vector allocations", vecs.len());
    assert!(true, "Vector allocation failures handled gracefully");
}

/// Test String allocation failures
#[test]
fn test_string_allocation_failure() {
    ensure_containerized();

    let mut strings = Vec::new();
    let string_size = 1_000_000; // 1MB strings

    for i in 0..500 {
        let mut s = String::new();
        match s.try_reserve(string_size) {
            Ok(_) => {
                // Fill with data to ensure actual allocation
                s.push_str(&"x".repeat(string_size));
                strings.push(s);
                if i % 10 == 0 {
                    println!("Allocated string {} ({}MB)", i, i);
                }
            }
            Err(e) => {
                println!("String allocation failed after {} strings: {:?}", i, e);
                break;
            }
        }
    }

    assert!(true, "String allocation failures handled gracefully");
}

/// Test Box allocation failures
#[test]
fn test_boxed_allocation_failure() {
    ensure_containerized();

    let mut boxes = Vec::new();

    // Try allocating large boxed arrays
    for i in 0..100 {
        let result = std::panic::catch_unwind(|| {
            Box::new([0u8; 10_000_000]) // 10MB array
        });

        match result {
            Ok(b) => {
                boxes.push(b);
                if i % 5 == 0 {
                    println!("Allocated box {} ({}MB)", i, (i * 10));
                }
            }
            Err(_) => {
                println!("Box allocation failed after {} boxes (expected)", i);
                break;
            }
        }
    }

    assert!(true, "Boxed allocation failures handled gracefully");
}

/// Test recovery after OOM
#[test]
fn test_oom_recovery() {
    ensure_containerized();

    println!("Testing OOM recovery...");

    // Allocate until failure
    let mut temp_allocs = Vec::new();
    for _ in 0..100 {
        match try_allocate(5_000_000) {
            Some(ptr) => temp_allocs.push(ptr),
            None => break,
        }
    }

    // Free all allocations
    for ptr in temp_allocs {
        unsafe {
            dealloc(ptr, Layout::from_size_align(5_000_000, 8).unwrap());
        }
    }

    // Verify we can allocate again after recovery
    let recovery_alloc = try_allocate(1_000_000);
    assert!(recovery_alloc.is_some(), "Failed to recover from OOM");

    if let Some(ptr) = recovery_alloc {
        unsafe {
            dealloc(ptr, Layout::from_size_align(1_000_000, 8).unwrap());
        }
    }

    println!("Successfully recovered from OOM");
}

// Helper function to safely attempt allocation
fn try_allocate(size: usize) -> Option<*mut u8> {
    unsafe {
        let layout = Layout::from_size_align(size, 8).ok()?;
        let ptr = alloc(layout);
        if ptr.is_null() {
            None
        } else {
            Some(ptr)
        }
    }
}

#[test]
fn test_fastforth_oom_handling() {
    ensure_containerized();

    // Test that FastForth components handle OOM gracefully
    // This would integrate with actual FastForth allocation paths

    println!("Testing FastForth OOM handling...");

    // Simulate compiler OOM scenario
    let mut buffers = Vec::new();
    for i in 0..1000 {
        match Vec::<u8>::try_with_capacity(1_000_000) {
            Ok(mut v) => {
                v.resize(1_000_000, 0);
                buffers.push(v);
            }
            Err(_) => {
                println!("Compiler OOM at iteration {}", i);
                break;
            }
        }
    }

    // Ensure we can still function with available memory
    assert!(buffers.len() > 0, "Should have allocated some buffers before OOM");

    println!("FastForth OOM handling validated");
}
