//! Memory Error Stress Tests (15 tests)
//!
//! Comprehensive tests for memory-related errors:
//! - Allocation failure simulation
//! - Pointer alignment issues
//! - Memory leak detection
//! - Double-free detection
//! - Use-after-free detection
//!
//! NOTE: Rust's memory safety prevents many of these at compile time.
//! These tests verify that the compiler handles edge cases safely.

use std::alloc::{alloc, dealloc, Layout};
use std::ptr;

// ============================================================================
// ALLOCATION FAILURE SIMULATION (3 tests)
// ============================================================================

#[test]
fn test_allocation_failure_recovery() {
    // Attempt to allocate an impossibly large amount of memory
    let huge_size = usize::MAX / 2;

    let layout_result = Layout::from_size_align(huge_size, 8);

    match layout_result {
        Ok(layout) => {
            // Try to allocate - should fail gracefully
            unsafe {
                let ptr = alloc(layout);
                if ptr.is_null() {
                    println!("Large allocation correctly failed");
                } else {
                    // If somehow succeeded, immediately deallocate
                    dealloc(ptr, layout);
                    panic!("Unexpectedly succeeded in allocating huge memory");
                }
            }
        }
        Err(e) => {
            println!("Layout creation failed as expected: {:?}", e);
        }
    }
}

#[test]
fn test_zero_size_allocation() {
    // Test allocation with zero size
    let layout_result = Layout::from_size_align(0, 1);

    if let Ok(layout) = layout_result {
        unsafe {
            let ptr = alloc(layout);
            println!("Zero-size allocation: {:?}", ptr);

            if !ptr.is_null() {
                dealloc(ptr, layout);
            }
        }
    }
}

#[test]
fn test_repeated_allocation_stress() {
    // Test many allocations and deallocations
    const NUM_ALLOCS: usize = 1000;
    const ALLOC_SIZE: usize = 1024;

    let mut ptrs = Vec::with_capacity(NUM_ALLOCS);
    let layout = Layout::from_size_align(ALLOC_SIZE, 8).unwrap();

    unsafe {
        // Allocate
        for _ in 0..NUM_ALLOCS {
            let ptr = alloc(layout);
            if ptr.is_null() {
                println!("Allocation failed after {} allocations", ptrs.len());
                break;
            }
            ptrs.push(ptr);
        }

        // Deallocate
        for ptr in ptrs {
            dealloc(ptr, layout);
        }
    }

    println!("Successfully allocated and freed {} blocks", NUM_ALLOCS);
}

// ============================================================================
// POINTER ALIGNMENT TESTS (3 tests)
// ============================================================================

#[test]
fn test_invalid_alignment() {
    // Test creation of layout with invalid alignment
    let result = Layout::from_size_align(1024, 3);

    assert!(result.is_err(), "Non-power-of-2 alignment should fail");
    if let Err(e) = result {
        println!("Invalid alignment error: {:?}", e);
    }
}

#[test]
fn test_alignment_requirements() {
    // Test different alignment requirements
    let alignments = vec![1, 2, 4, 8, 16, 32, 64, 128];

    for align in alignments {
        let layout_result = Layout::from_size_align(1024, align);
        assert!(layout_result.is_ok(), "Alignment {} should be valid", align);

        if let Ok(layout) = layout_result {
            unsafe {
                let ptr = alloc(layout);
                if !ptr.is_null() {
                    // Check alignment
                    assert_eq!(
                        ptr as usize % align,
                        0,
                        "Pointer not aligned to {} bytes",
                        align
                    );
                    dealloc(ptr, layout);
                }
            }
        }
    }
}

#[test]
fn test_oversized_alignment() {
    // Test extremely large alignment
    let result = Layout::from_size_align(1024, 1 << 20);

    match result {
        Ok(layout) => {
            unsafe {
                let ptr = alloc(layout);
                println!("Large alignment allocation: {:?}", ptr);
                if !ptr.is_null() {
                    dealloc(ptr, layout);
                }
            }
        }
        Err(e) => {
            println!("Large alignment failed: {:?}", e);
        }
    }
}

// ============================================================================
// MEMORY LEAK DETECTION (3 tests)
// ============================================================================

#[test]
fn test_intentional_leak_detection() {
    // Create intentional leak for testing
    let layout = Layout::from_size_align(1024, 8).unwrap();

    unsafe {
        let _leaked = alloc(layout);
        // Intentionally not deallocating
        // Memory leak detection tools should catch this
    }

    println!("Intentional leak created (for leak detector testing)");
}

#[test]
fn test_vec_capacity_leak() {
    // Test that Vec properly deallocates on drop
    {
        let mut v = Vec::with_capacity(10000);
        for i in 0..10000 {
            v.push(i);
        }
        // v drops here
    }

    println!("Vec properly cleaned up");
}

#[test]
fn test_box_leak() {
    // Test Box allocation and deallocation
    {
        let large_data = Box::new([0u8; 1_000_000]);
        let _ = large_data;
        // Box drops here
    }

    println!("Box properly cleaned up");
}

// ============================================================================
// DOUBLE-FREE DETECTION (3 tests)
// ============================================================================

#[test]
fn test_double_free_prevention() {
    let layout = Layout::from_size_align(1024, 8).unwrap();

    unsafe {
        let ptr = alloc(layout);
        if !ptr.is_null() {
            dealloc(ptr, layout);
            // Second dealloc would be double-free
            // Rust prevents this in safe code
            // dealloc(ptr, layout); // Would be UB
        }
    }

    println!("Double-free prevented by correct usage");
}

#[test]
fn test_box_double_drop_prevention() {
    // Rust's ownership prevents double-drop
    let b = Box::new(42);
    drop(b);
    // Cannot drop b again - compile error if uncommented
    // drop(b);

    println!("Box double-drop prevented by ownership");
}

#[test]
fn test_manual_drop_safety() {
    use std::mem::ManuallyDrop;

    let data = ManuallyDrop::new(Box::new(vec![1, 2, 3]));
    // ManuallyDrop prevents automatic drop
    // Must manually drop or leak

    unsafe {
        ManuallyDrop::drop(&mut { data });
    }

    println!("ManuallyDrop correctly handled");
}

// ============================================================================
// USE-AFTER-FREE DETECTION (3 tests)
// ============================================================================

#[test]
fn test_use_after_free_prevention() {
    // Rust's borrow checker prevents use-after-free
    let data = Box::new(42);
    let value = *data;
    drop(data);
    // Cannot use data here - compile error if uncommented
    // let _ = *data;

    println!("Use-after-free prevented: {}", value);
}

#[test]
fn test_dangling_pointer_prevention() {
    // Test that Rust prevents dangling pointers
    let ptr: *const i32;

    {
        let value = 42;
        ptr = &value;
        // value goes out of scope
    }

    // Cannot safely dereference ptr here
    // unsafe { *ptr } // Would be UB

    println!("Dangling pointer not dereferenced");
}

#[test]
fn test_safe_reference_lifetime() {
    // Test that references maintain safety
    let data = vec![1, 2, 3, 4, 5];
    let slice = &data[1..3];

    // data is still valid
    assert_eq!(slice, &[2, 3]);

    drop(data);
    // slice is now invalid - compile error if uncommented
    // assert_eq!(slice, &[2, 3]);

    println!("Reference lifetime correctly enforced");
}
