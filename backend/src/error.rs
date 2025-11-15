//! Error types for the backend (LLVM, Cranelift, etc.)

use thiserror::Error;

pub type Result<T> = std::result::Result<T, BackendError>;

#[derive(Error, Debug)]
pub enum BackendError {
    #[error("LLVM compilation failed: {0}")]
    CompilationFailed(String),

    #[error("Linking failed: {0}")]
    LinkingFailed(String),

    #[error("Invalid IR: {0}")]
    InvalidIR(String),

    #[error("Register allocation failed: {0}")]
    RegisterAllocationFailed(String),

    #[error("Code generation error: {0}")]
    CodeGenError(String),

    #[error("Code generation failed: {0}")]
    CodeGeneration(String),

    #[error("Backend initialization failed: {0}")]
    Initialization(String),

    #[error("Target machine creation failed: {0}")]
    TargetMachineError(String),

    #[error("Module verification failed: {0}")]
    VerificationFailed(String),

    #[error("Unsupported feature: {0}")]
    UnsupportedFeature(String),

    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
}
