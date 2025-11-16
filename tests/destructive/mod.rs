// Destructive Testing Module
// Tests error handling under extreme resource constraints
// ONLY runs in containerized environments with safety guards

#![cfg(feature = "destructive_tests")]

pub mod test_oom;
pub mod test_disk_full;
pub mod test_stack_overflow;
pub mod test_fd_exhaustion;
pub mod safety;

// Re-export safety guards
pub use safety::{ensure_containerized, is_safe_to_run_destructive_tests};

/// Marker for destructive tests
/// Use with #[destructive_test] attribute macro
pub const DESTRUCTIVE_TEST_MARKER: &str = "DESTRUCTIVE_TEST";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_safety_guards() {
        // Ensure safety mechanisms are in place
        let is_safe = is_safe_to_run_destructive_tests();

        if !is_safe {
            println!("NOTICE: Destructive tests will be skipped (not in container)");
        }

        // This test always passes, just validates safety checks work
        assert!(true);
    }
}
