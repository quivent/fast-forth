//! Fast Forth - High-performance Forth compiler with LLVM backend
//!
//! This is the main integration layer that connects:
//! - Frontend: Parsing, type inference, SSA conversion
//! - Optimizer: Five optimization passes (stack caching, superinstructions, etc.)
//! - Backend: LLVM code generation
//! - Runtime: C runtime library
//!
//! # Example
//!
//! ```rust,no_run
//! use fastforth::{Compiler, CompilationMode, OptimizationLevel};
//!
//! let compiler = Compiler::new(OptimizationLevel::Aggressive);
//! let result = compiler.compile_string(": square dup * ;", CompilationMode::JIT)?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

pub mod error;
pub mod compiler;
pub mod pipeline;
pub mod backend;

pub use error::{CompileError, Result};
pub use pipeline::{CompilationPipeline, CompilationMode, CompilationResult};

// Re-export commonly used types from components
pub use fastforth_frontend::{
    Program, Definition, Word, StackEffect as FrontendStackEffect,
    parse_program, analyze, convert_to_ssa,
};
pub use fastforth_optimizer::{
    ForthIR, Instruction, StackEffect, Optimizer, OptimizationLevel,
};

use std::path::Path;

/// Main Fast Forth compiler instance
///
/// This manages the entire compilation pipeline from source to executable/JIT.
pub struct Compiler {
    optimization_level: OptimizationLevel,
    optimizer: Optimizer,
}

impl Compiler {
    /// Create a new compiler with the specified optimization level
    pub fn new(optimization_level: OptimizationLevel) -> Self {
        Self {
            optimization_level,
            optimizer: Optimizer::new(optimization_level),
        }
    }

    /// Compile Forth source code from a string
    pub fn compile_string(&self, source: &str, mode: CompilationMode) -> Result<CompilationResult> {
        let pipeline = CompilationPipeline::new(self.optimization_level);
        pipeline.compile(source, mode)
    }

    /// Compile Forth source code from a file
    pub fn compile_file(&self, path: &Path, mode: CompilationMode) -> Result<CompilationResult> {
        let source = std::fs::read_to_string(path)
            .map_err(|e| CompileError::IoError(path.to_path_buf(), e))?;
        self.compile_string(&source, mode)
    }

    /// Get the optimization level
    pub fn optimization_level(&self) -> OptimizationLevel {
        self.optimization_level
    }

    /// Set the optimization level
    pub fn set_optimization_level(&mut self, level: OptimizationLevel) {
        self.optimization_level = level;
        self.optimizer = Optimizer::new(level);
    }
}

impl Default for Compiler {
    fn default() -> Self {
        Self::new(OptimizationLevel::Standard)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compiler_creation() {
        let compiler = Compiler::new(OptimizationLevel::Aggressive);
        assert_eq!(compiler.optimization_level(), OptimizationLevel::Aggressive);
    }

    #[test]
    fn test_compiler_default() {
        let compiler = Compiler::default();
        assert_eq!(compiler.optimization_level(), OptimizationLevel::Standard);
    }
}
