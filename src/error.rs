//! Error types for the Fast Forth compiler

use std::path::PathBuf;
use thiserror::Error;

/// Result type for compilation operations
pub type Result<T> = std::result::Result<T, CompileError>;

/// Compilation error types
#[derive(Error, Debug)]
pub enum CompileError {
    /// Frontend parsing error
    #[error("Parse error: {0}")]
    ParseError(String),

    /// Semantic analysis error
    #[error("Semantic error: {0}")]
    SemanticError(String),

    /// Type inference error
    #[error("Type error: {0}")]
    TypeError(String),

    /// SSA conversion error
    #[error("SSA conversion error: {0}")]
    SSAError(String),

    /// Optimization error
    #[error("Optimization error: {0}")]
    OptimizationError(String),

    /// Code generation error
    #[error("Code generation error: {0}")]
    CodeGenError(String),

    /// LLVM error
    #[error("LLVM error: {0}")]
    LLVMError(String),

    /// I/O error
    #[error("I/O error for file {0}: {1}")]
    IoError(PathBuf, #[source] std::io::Error),

    /// Runtime error
    #[error("Runtime error: {0}")]
    RuntimeError(String),

    /// Internal compiler error
    #[error("Internal compiler error: {0}")]
    InternalError(String),
}

impl From<fastforth_frontend::ForthError> for CompileError {
    fn from(err: fastforth_frontend::ForthError) -> Self {
        CompileError::ParseError(err.to_string())
    }
}

impl From<fastforth_optimizer::OptimizerError> for CompileError {
    fn from(err: fastforth_optimizer::OptimizerError) -> Self {
        CompileError::OptimizationError(err.to_string())
    }
}
