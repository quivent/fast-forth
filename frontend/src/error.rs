//! Error types for the Fast Forth compiler

use thiserror::Error;

pub type Result<T> = std::result::Result<T, ForthError>;

#[derive(Error, Debug, Clone, PartialEq)]
pub enum ForthError {
    #[error("Parse error at line {line}, column {column}: {message}")]
    ParseError {
        line: usize,
        column: usize,
        message: String,
    },

    #[error("Lexical error at position {position}: {message}")]
    LexError {
        position: usize,
        message: String,
    },

    #[error("Undefined word: {word}")]
    UndefinedWord {
        word: String,
        line: Option<usize>,
    },

    #[error("Stack underflow in word '{word}': expected {expected} items, found {found}")]
    StackUnderflow {
        word: String,
        expected: usize,
        found: usize,
    },

    #[error("Stack depth mismatch in {word}: {message}")]
    StackMismatch {
        word: String,
        then_depth: usize,
        else_depth: usize,
        message: String,
    },

    #[error("Stack overflow: maximum depth {max} exceeded")]
    StackOverflow {
        max: usize,
    },

    #[error("Type error: expected {expected}, found {found}")]
    TypeError {
        expected: String,
        found: String,
        location: Option<String>,
    },

    #[error("Invalid stack effect declaration: {declaration}")]
    InvalidStackEffect {
        declaration: String,
    },

    #[error("Redefinition of word: {word}")]
    RedefinitionError {
        word: String,
    },

    #[error("Control structure mismatch: expected {expected}, found {found}")]
    ControlStructureMismatch {
        expected: String,
        found: String,
    },

    #[error("Invalid immediate word usage: {word}")]
    InvalidImmediateWord {
        word: String,
    },

    #[error("SSA conversion error: {message}")]
    SSAConversionError {
        message: String,
    },

    #[error("Internal compiler error: {message}")]
    InternalError {
        message: String,
    },
}

impl ForthError {
    pub fn parse_error(line: usize, column: usize, message: impl Into<String>) -> Self {
        ForthError::ParseError {
            line,
            column,
            message: message.into(),
        }
    }

    pub fn undefined_word(word: impl Into<String>) -> Self {
        ForthError::UndefinedWord {
            word: word.into(),
            line: None,
        }
    }

    pub fn type_error(expected: impl Into<String>, found: impl Into<String>) -> Self {
        ForthError::TypeError {
            expected: expected.into(),
            found: found.into(),
            location: None,
        }
    }
}
