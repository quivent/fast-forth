/// Property-based fuzzing library for Fast Forth
///
/// This library provides comprehensive property-based testing using proptest
/// to systematically explore the input space of Forth programs.

pub mod property_tests;
pub mod stress_tests;

// Re-export key types and functions
pub use property_tests::{
    gforth_available,
    run_gforth,
    run_fast_forth,
    CORPUS,
};
