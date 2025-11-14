//! Symbolic Execution Engine
//!
//! Enables equivalence checking and semantic comparison via symbolic execution

pub mod executor;
pub mod symbolic_value;
pub mod equivalence;

pub use executor::{SymbolicExecutor, ExecutionResult};
pub use symbolic_value::{SymbolicValue, SymbolicStack};
pub use equivalence::{EquivalenceChecker, EquivalenceResult};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum SymbolicError {
    #[error("Stack underflow: required {required}, available {available}")]
    StackUnderflow { required: usize, available: usize },

    #[error("Unknown word: {0}")]
    UnknownWord(String),

    #[error("Execution limit exceeded")]
    ExecutionLimitExceeded,

    #[error("Unsupported operation: {0}")]
    UnsupportedOperation(String),
}

pub type Result<T> = std::result::Result<T, SymbolicError>;
