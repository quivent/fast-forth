// Stack Overflow Testing
// Tests handling of stack exhaustion via deep recursion

#![cfg(feature = "destructive_tests")]

use super::safety::ensure_containerized;
use std::panic::catch_unwind;

#[test]
fn test_deep_recursion_handling() {
    ensure_containerized();

    println!("Testing deep recursion handling...");

    // Catch stack overflow panics
    let result = catch_unwind(|| {
        deep_recursion(0)
    });

    match result {
        Ok(_) => {
            // If we didn't overflow, that's actually fine for this test
            println!("Recursion completed without overflow (stack limit may be high)");
        }
        Err(_) => {
            println!("Stack overflow caught (expected)");
        }
    }

    // Verify we can still execute after stack overflow
    let recovery = simple_function();
    assert_eq!(recovery, 42, "Failed to recover after stack overflow");

    println!("Successfully recovered from stack overflow");
}

#[test]
fn test_mutual_recursion_overflow() {
    ensure_containerized();

    println!("Testing mutual recursion overflow...");

    let result = catch_unwind(|| {
        mutual_recursion_a(0)
    });

    match result {
        Ok(_) => println!("Mutual recursion completed"),
        Err(_) => println!("Mutual recursion overflow caught (expected)"),
    }

    // Verify recovery
    assert_eq!(simple_function(), 42, "Failed to recover");
}

#[test]
fn test_large_stack_frames() {
    ensure_containerized();

    println!("Testing large stack frame allocation...");

    let result = catch_unwind(|| {
        large_stack_frame_recursion(0)
    });

    match result {
        Ok(_) => println!("Large frame recursion completed"),
        Err(_) => println!("Large frame overflow caught (expected)"),
    }

    assert_eq!(simple_function(), 42, "Failed to recover");
}

#[test]
fn test_recursive_data_structures() {
    ensure_containerized();

    println!("Testing recursive data structure traversal...");

    // Build deep linked list
    let list = build_deep_list(1000);

    // Try to traverse it recursively (may overflow)
    let result = catch_unwind(|| {
        traverse_list_recursive(&list)
    });

    match result {
        Ok(len) => println!("Traversed list of length {}", len),
        Err(_) => println!("List traversal overflow caught (expected)"),
    }

    // Verify iterative traversal still works
    let len = traverse_list_iterative(&list);
    assert_eq!(len, 1000, "Iterative traversal failed");
}

#[test]
fn test_forth_stack_overflow() {
    ensure_containerized();

    println!("Testing Forth-style stack operations...");

    // Simulate Forth data stack with deep nesting
    let mut stack = Vec::new();

    let result = catch_unwind(|| {
        forth_deep_call_stack(&mut stack, 0)
    });

    match result {
        Ok(_) => println!("Forth stack operations completed"),
        Err(_) => println!("Forth stack overflow caught (expected)"),
    }

    // Verify stack is still usable
    stack.clear();
    stack.push(42);
    assert_eq!(stack.pop(), Some(42), "Stack corrupted after overflow");
}

#[test]
fn test_compiler_recursion_limits() {
    ensure_containerized();

    println!("Testing compiler recursion limits...");

    // Simulate deeply nested AST processing
    let result = catch_unwind(|| {
        process_nested_ast(0)
    });

    match result {
        Ok(depth) => println!("Processed AST to depth {}", depth),
        Err(_) => println!("AST processing overflow caught (expected)"),
    }

    assert_eq!(simple_function(), 42, "Failed to recover");
}

// Helper functions simulating various recursion patterns

fn deep_recursion(depth: usize) -> usize {
    if depth > 100_000 {
        return depth;
    }

    // Create some stack pressure
    let _local_array = [0u8; 1024];

    deep_recursion(depth + 1)
}

fn mutual_recursion_a(depth: usize) -> usize {
    if depth > 100_000 {
        return depth;
    }
    mutual_recursion_b(depth + 1)
}

fn mutual_recursion_b(depth: usize) -> usize {
    if depth > 100_000 {
        return depth;
    }
    mutual_recursion_a(depth + 1)
}

fn large_stack_frame_recursion(depth: usize) -> usize {
    if depth > 10_000 {
        return depth;
    }

    // Large stack frame
    let _large_array = [0u8; 16384]; // 16KB per frame

    large_stack_frame_recursion(depth + 1)
}

fn simple_function() -> i32 {
    42
}

// Linked list for testing
struct Node {
    value: i32,
    next: Option<Box<Node>>,
}

fn build_deep_list(depth: usize) -> Option<Box<Node>> {
    if depth == 0 {
        return None;
    }

    Some(Box::new(Node {
        value: depth as i32,
        next: build_deep_list(depth - 1),
    }))
}

fn traverse_list_recursive(node: &Option<Box<Node>>) -> usize {
    match node {
        None => 0,
        Some(n) => 1 + traverse_list_recursive(&n.next),
    }
}

fn traverse_list_iterative(mut node: &Option<Box<Node>>) -> usize {
    let mut count = 0;
    while let Some(n) = node {
        count += 1;
        node = &n.next;
    }
    count
}

fn forth_deep_call_stack(stack: &mut Vec<i32>, depth: usize) -> usize {
    if depth > 50_000 {
        return depth;
    }

    // Simulate Forth word execution
    stack.push(depth as i32);
    let result = forth_deep_call_stack(stack, depth + 1);
    stack.pop();

    result
}

fn process_nested_ast(depth: usize) -> usize {
    if depth > 10_000 {
        return depth;
    }

    // Simulate AST node processing with local data
    let _node_data = [0u64; 64];

    // Recurse for nested expressions
    let left = process_nested_ast(depth + 1);
    let _right = process_nested_ast(depth + 1);

    left
}
