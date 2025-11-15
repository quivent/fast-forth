//! Fast Forth Backend
//!
//! This module provides multiple backend options for native code generation:
//! - Cranelift: Fast compilation (50ms), good runtime (70-85% of C) - Default for -O0/-O1
//! - LLVM: Slow compilation (2-5min), excellent runtime (85-110% of C) - Default for -O2/-O3

#[cfg(feature = "llvm")]
pub mod codegen;
#[cfg(feature = "cranelift")]
pub mod cranelift;
pub mod linker;
pub mod error;

#[cfg(feature = "llvm")]
pub use codegen::{CodeGenerator, LLVMBackend, CompilationMode};
#[cfg(feature = "cranelift")]
pub use cranelift::{CraneliftBackend, CraneliftCompiler};
pub use linker::{Linker, LinkMode};
pub use error::{BackendError, Result};

/// Backend version and compatibility
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const LLVM_VERSION: &str = "17.0";
pub const CRANELIFT_VERSION: &str = "0.102";

/// Re-export types from frontend for convenience
#[cfg(any(feature = "llvm", feature = "cranelift"))]
pub use fastforth_frontend::ssa::{SSAFunction, SSAInstruction, Register, BlockId};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backend_version() {
        assert!(!VERSION.is_empty());
    }
}
