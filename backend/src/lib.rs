//! LLVM Backend for Fast Forth
//!
//! This module provides native code generation through LLVM IR,
//! supporting both AOT compilation and JIT execution.

#[cfg(feature = "llvm")]
pub mod codegen;
pub mod linker;
pub mod error;

#[cfg(feature = "llvm")]
pub use codegen::{CodeGenerator, LLVMBackend, CompilationMode};
pub use linker::{Linker, LinkMode};
pub use error::{BackendError, Result};

/// Backend version and compatibility
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const LLVM_VERSION: &str = "17.0";

/// Re-export types from frontend for convenience
#[cfg(feature = "llvm")]
pub use fastforth_frontend::ssa::{SSAFunction, SSAInstruction, Register, BlockId};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backend_version() {
        assert!(!VERSION.is_empty());
    }
}
