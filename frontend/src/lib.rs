//! Fast Forth Frontend Compiler
//!
//! This module provides a complete frontend compiler for ANS Forth, including:
//! - Lexical analysis and parsing
//! - Stack effect inference
//! - Type inference (Hindley-Milner-style)
//! - SSA conversion
//! - Semantic analysis and validation

pub mod error;
pub mod ast;
pub mod lexer;
pub mod parser;
pub mod stack_effects;
pub mod type_inference;
pub mod ssa;
pub mod ssa_validator;
pub mod semantic;

pub use error::{ForthError, Result};
pub use ast::{Program, Definition, Word, StackEffect};
pub use parser::parse_program;
pub use semantic::analyze;
pub use ssa::{convert_to_ssa, SSAFunction};
pub use ssa_validator::SSAValidator;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_pipeline() {
        let source = ": double 2 * ;";
        let program = parse_program(source).expect("Failed to parse");
        assert_eq!(program.definitions.len(), 1);
    }
}
