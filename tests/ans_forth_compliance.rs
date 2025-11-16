//! ANS Forth Compliance Test Suite
//!
//! Main test entry point for ANS Forth standard compliance tests.
//! This file exposes the compliance tests to cargo's test runner.

// Include test utilities
mod test_utils;

// Include compliance test modules
mod compliance {
    pub mod ans_forth_core;
    pub mod ans_forth_extended;
}

// Re-export for easy access
pub use compliance::*;
