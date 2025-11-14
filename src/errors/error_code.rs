//! Error code definitions (E0001-E9999)
//!
//! Error codes are organized by category:
//! - E0001-E0999: Lexical and parsing errors
//! - E1000-E1999: Semantic and type errors
//! - E2000-E2999: Stack effect errors
//! - E3000-E3999: Control flow errors
//! - E4000-E4999: Optimization errors
//! - E5000-E5999: Code generation errors
//! - E9000-E9999: Internal compiler errors

use serde::{Serialize, Deserialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCode {
    // Lexical/Parsing Errors (E0001-E0999)
    UnexpectedToken = 1,
    UnexpectedEof = 2,
    InvalidNumber = 3,
    InvalidStringLiteral = 4,
    UnterminatedComment = 5,
    InvalidCharacterLiteral = 6,

    // Semantic Errors (E1000-E1999)
    UndefinedWord = 1000,
    RedefinedWord = 1001,
    InvalidStackEffect = 1002,
    InvalidImmediate = 1003,
    RecursionWithoutBaseCase = 1004,

    // Stack Effect Errors (E2000-E2999)
    StackUnderflow = 2000,
    StackOverflow = 2001,
    StackDepthMismatch = 2234,
    TypeMismatch = 2300,
    InsufficientInputs = 2400,
    ExcessOutputs = 2401,

    // Control Flow Errors (E3000-E3999)
    UnmatchedIf = 3000,
    UnmatchedThen = 3001,
    UnmatchedElse = 3002,
    UnmatchedDo = 3010,
    UnmatchedLoop = 3011,
    UnmatchedBegin = 3020,
    UnmatchedUntil = 3021,
    UnmatchedWhile = 3022,
    UnmatchedRepeat = 3023,
    InvalidControlStructure = 3100,

    // Optimization Errors (E4000-E4999)
    OptimizationFailed = 4000,
    InliningError = 4001,
    ConstantFoldingError = 4002,
    DeadCodeEliminationError = 4003,

    // Code Generation Errors (E5000-E5999)
    CodeGenFailed = 5000,
    LLVMError = 5001,
    LinkingError = 5002,

    // Internal Errors (E9000-E9999)
    InternalCompilerError = 9000,
    SSAConversionError = 9001,
    UnexpectedState = 9002,
}

impl ErrorCode {
    /// Get the error code as a string (e.g., "E0234")
    pub fn as_str(&self) -> String {
        format!("E{:04}", *self as u32)
    }

    /// Get the category name
    pub fn category(&self) -> &'static str {
        let code = *self as u32;
        match code {
            0..=999 => "Lexical/Parsing",
            1000..=1999 => "Semantic",
            2000..=2999 => "Stack Effects",
            3000..=3999 => "Control Flow",
            4000..=4999 => "Optimization",
            5000..=5999 => "Code Generation",
            9000..=9999 => "Internal",
            _ => "Unknown",
        }
    }

    /// Get detailed description
    pub fn description(&self) -> &'static str {
        match self {
            ErrorCode::UnexpectedToken => "Unexpected token encountered during parsing",
            ErrorCode::UnexpectedEof => "Unexpected end of file",
            ErrorCode::InvalidNumber => "Invalid number format",
            ErrorCode::InvalidStringLiteral => "Invalid string literal",
            ErrorCode::UnterminatedComment => "Unterminated comment",
            ErrorCode::InvalidCharacterLiteral => "Invalid character literal",

            ErrorCode::UndefinedWord => "Reference to undefined word",
            ErrorCode::RedefinedWord => "Attempt to redefine existing word",
            ErrorCode::InvalidStackEffect => "Invalid stack effect declaration",
            ErrorCode::InvalidImmediate => "Invalid use of immediate word",
            ErrorCode::RecursionWithoutBaseCase => "Recursive definition without base case",

            ErrorCode::StackUnderflow => "Stack underflow - insufficient items on stack",
            ErrorCode::StackOverflow => "Stack overflow - too many items on stack",
            ErrorCode::StackDepthMismatch => "Stack depth doesn't match declared effect",
            ErrorCode::TypeMismatch => "Type mismatch in stack operation",
            ErrorCode::InsufficientInputs => "Insufficient inputs for operation",
            ErrorCode::ExcessOutputs => "More outputs than expected",

            ErrorCode::UnmatchedIf => "IF without matching THEN",
            ErrorCode::UnmatchedThen => "THEN without matching IF",
            ErrorCode::UnmatchedElse => "ELSE without matching IF",
            ErrorCode::UnmatchedDo => "DO without matching LOOP",
            ErrorCode::UnmatchedLoop => "LOOP without matching DO",
            ErrorCode::UnmatchedBegin => "BEGIN without matching UNTIL/WHILE/REPEAT",
            ErrorCode::UnmatchedUntil => "UNTIL without matching BEGIN",
            ErrorCode::UnmatchedWhile => "WHILE without matching BEGIN",
            ErrorCode::UnmatchedRepeat => "REPEAT without matching BEGIN",
            ErrorCode::InvalidControlStructure => "Invalid control structure",

            ErrorCode::OptimizationFailed => "Optimization pass failed",
            ErrorCode::InliningError => "Error during function inlining",
            ErrorCode::ConstantFoldingError => "Error during constant folding",
            ErrorCode::DeadCodeEliminationError => "Error during dead code elimination",

            ErrorCode::CodeGenFailed => "Code generation failed",
            ErrorCode::LLVMError => "LLVM backend error",
            ErrorCode::LinkingError => "Linking error",

            ErrorCode::InternalCompilerError => "Internal compiler error",
            ErrorCode::SSAConversionError => "SSA conversion error",
            ErrorCode::UnexpectedState => "Unexpected compiler state",
        }
    }

    /// Get suggested fix pattern if available
    pub fn fix_pattern(&self) -> Option<&'static str> {
        match self {
            ErrorCode::StackDepthMismatch => Some("DROP_EXCESS_001"),
            ErrorCode::StackUnderflow => Some("ADD_INPUTS_002"),
            ErrorCode::UnmatchedIf => Some("ADD_THEN_003"),
            ErrorCode::UnmatchedDo => Some("ADD_LOOP_004"),
            ErrorCode::UnmatchedBegin => Some("ADD_UNTIL_005"),
            _ => None,
        }
    }
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Error code registry for documentation generation
pub struct ErrorCodeRegistry;

impl ErrorCodeRegistry {
    pub fn all_codes() -> Vec<ErrorCode> {
        vec![
            // Lexical/Parsing
            ErrorCode::UnexpectedToken,
            ErrorCode::UnexpectedEof,
            ErrorCode::InvalidNumber,
            ErrorCode::InvalidStringLiteral,
            ErrorCode::UnterminatedComment,
            ErrorCode::InvalidCharacterLiteral,

            // Semantic
            ErrorCode::UndefinedWord,
            ErrorCode::RedefinedWord,
            ErrorCode::InvalidStackEffect,
            ErrorCode::InvalidImmediate,
            ErrorCode::RecursionWithoutBaseCase,

            // Stack Effects
            ErrorCode::StackUnderflow,
            ErrorCode::StackOverflow,
            ErrorCode::StackDepthMismatch,
            ErrorCode::TypeMismatch,
            ErrorCode::InsufficientInputs,
            ErrorCode::ExcessOutputs,

            // Control Flow
            ErrorCode::UnmatchedIf,
            ErrorCode::UnmatchedThen,
            ErrorCode::UnmatchedElse,
            ErrorCode::UnmatchedDo,
            ErrorCode::UnmatchedLoop,
            ErrorCode::UnmatchedBegin,
            ErrorCode::UnmatchedUntil,
            ErrorCode::UnmatchedWhile,
            ErrorCode::UnmatchedRepeat,
            ErrorCode::InvalidControlStructure,

            // Optimization
            ErrorCode::OptimizationFailed,
            ErrorCode::InliningError,
            ErrorCode::ConstantFoldingError,
            ErrorCode::DeadCodeEliminationError,

            // Code Generation
            ErrorCode::CodeGenFailed,
            ErrorCode::LLVMError,
            ErrorCode::LinkingError,

            // Internal
            ErrorCode::InternalCompilerError,
            ErrorCode::SSAConversionError,
            ErrorCode::UnexpectedState,
        ]
    }
}

/// Global error code registry
pub const ERROR_CODE_REGISTRY: ErrorCodeRegistry = ErrorCodeRegistry;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_code_format() {
        assert_eq!(ErrorCode::StackDepthMismatch.as_str(), "E2234");
        assert_eq!(ErrorCode::UndefinedWord.as_str(), "E1000");
    }

    #[test]
    fn test_error_category() {
        assert_eq!(ErrorCode::StackDepthMismatch.category(), "Stack Effects");
        assert_eq!(ErrorCode::UndefinedWord.category(), "Semantic");
    }

    #[test]
    fn test_all_codes() {
        let codes = ERROR_CODE_REGISTRY.all_codes();
        assert!(!codes.is_empty());
    }
}
