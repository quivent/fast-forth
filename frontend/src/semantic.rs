//! Semantic analysis and validation
//!
//! Performs comprehensive semantic checks on Forth programs:
//! - Undefined word detection
//! - Stack underflow detection
//! - Control structure validation
//! - Redefinition checks

use crate::ast::*;
use crate::error::{ForthError, Result};
use crate::stack_effects::StackEffectInference;
use rustc_hash::FxHashSet;
use std::collections::HashMap;

/// Semantic analyzer
pub struct SemanticAnalyzer {
    /// Known word definitions
    defined_words: FxHashSet<String>,
    /// Stack effect inference engine
    stack_inference: StackEffectInference,
    /// Variables
    variables: FxHashSet<String>,
    /// Constants
    constants: HashMap<String, i64>,
    /// Errors collected during analysis
    errors: Vec<ForthError>,
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        let mut defined_words = FxHashSet::default();

        // Add all builtin words
        for word in &[
            // Arithmetic
            "+", "-", "*", "/", "mod", "/mod", "negate", "abs", "min", "max",
            "1+", "1-", "2+", "2-", "2*", "2/", "*/", "*/mod",
            // Stack manipulation
            "dup", "drop", "swap", "over", "rot", "2dup", "2drop", "2swap", "2over",
            "pick", "roll", "depth", "?dup",
            // Comparison
            "<", ">", "=", "<=", ">=", "<>", "0<", "0>", "0=", "0<>",
            "u<", "u>", "u<=", "u>=",
            "d=", "d<", "d0=", "d0<",
            // Logical
            "and", "or", "xor", "not", "invert", "true", "false",
            // Memory
            "@", "!", "c@", "c!", "+!", "?",
            "cell", "cells", "cell+", "char+", "chars", "align", "aligned",
            "move", "fill", "erase", "compare", "search", "count",
            // I/O
            ".", "emit", "cr", "space", "spaces", "type",
            ".\"", ".(", ".r", ".s",
            // Control (these are special but should be recognized)
            "if", "then", "else", "begin", "until", "while", "repeat",
            "do", "loop", "+loop", "leave", "exit",
            // Return stack
            ">r", "r>", "r@",
            // Other
            "here", "allot", "execute", "char",
            "within", "sm/rem", "fm/mod",
            "d+", "d-", "dnegate", "dabs", "d2*", "d2/",
        ] {
            defined_words.insert(word.to_string());
        }

        Self {
            defined_words,
            stack_inference: StackEffectInference::new(),
            variables: FxHashSet::default(),
            constants: HashMap::new(),
            errors: Vec::new(),
        }
    }

    /// Add an error to the list
    fn error(&mut self, error: ForthError) {
        self.errors.push(error);
    }

    /// Check if a word is defined
    fn is_defined(&self, word: &str) -> bool {
        self.defined_words.contains(word)
            || self.variables.contains(word)
            || self.constants.contains_key(word)
    }

    /// Analyze a complete program
    pub fn analyze(&mut self, program: &Program) -> Result<()> {
        // First pass: collect all definitions
        for def in &program.definitions {
            if self.defined_words.contains(&def.name) && !self.is_builtin(&def.name) {
                self.error(ForthError::RedefinitionError {
                    word: def.name.clone(),
                });
            }
            self.defined_words.insert(def.name.clone());

            // Add to stack inference
            if let Err(e) = self.stack_inference.add_definition(def) {
                self.error(e);
            }
        }

        // Collect variables and constants from top-level code
        for word in &program.top_level_code {
            match word {
                Word::Variable { name } => {
                    self.variables.insert(name.clone());
                }
                Word::Constant { name, value } => {
                    self.constants.insert(name.clone(), *value);
                }
                _ => {}
            }
        }

        // Second pass: validate definitions
        for def in &program.definitions {
            self.validate_definition(def)?;
        }

        // Validate top-level code
        for word in &program.top_level_code {
            self.validate_word(word)?;
        }

        // Return error if any were collected
        if !self.errors.is_empty() {
            return Err(self.errors[0].clone());
        }

        Ok(())
    }

    /// Check if a word is a builtin
    fn is_builtin(&self, word: &str) -> bool {
        matches!(
            word,
            // Arithmetic
            "+" | "-" | "*" | "/" | "mod" | "/mod" | "negate" | "abs" | "min" | "max"
            // Stack manipulation
            | "dup" | "drop" | "swap" | "over" | "rot" | "2dup" | "2drop" | "2swap" | "2over"
            | "pick" | "roll" | "depth"
            // Comparison
            | "<" | ">" | "=" | "<=" | ">=" | "<>" | "0<" | "0>" | "0="
            // Logical
            | "and" | "or" | "xor" | "not" | "invert"
            // Memory
            | "@" | "!" | "c@" | "c!" | "+!" | "?"
            // I/O
            | "." | "emit" | "cr" | "space" | "spaces" | "type"
            // Control
            | "if" | "then" | "else" | "begin" | "until" | "while" | "repeat"
            | "do" | "loop" | "+loop" | "leave" | "exit"
            // Return stack
            | ">r" | "r>" | "r@"
            // Other
            | "here" | "allot" | "cells" | "cell+" | "i" | "j" | "execute" | "char"
        )
    }

    /// Validate a definition
    fn validate_definition(&mut self, def: &Definition) -> Result<()> {
        // Check for control structure balance
        self.validate_control_structures(&def.body)?;

        // Check for undefined words
        for word in &def.body {
            self.validate_word(word)?;
        }

        // Validate stack effect if declared
        // Skip validation for definitions with loops or return stack operations,
        // as these are complex to analyze statically
        let has_complex_control_flow = self.has_complex_control_flow(&def.body);

        if let Some(declared_effect) = &def.stack_effect {
            if !has_complex_control_flow {
                // Infer actual stack effect
                match self.stack_inference.infer_sequence(&def.body) {
                    Ok(inferred_effect) => {
                        // Check if they match
                        if declared_effect.inputs.len() != inferred_effect.inputs.len()
                            || declared_effect.outputs.len() != inferred_effect.outputs.len()
                        {
                            self.error(ForthError::InvalidStackEffect {
                                declaration: format!(
                                    "Declared {} but inferred ( {} -- {} )",
                                    declared_effect,
                                    inferred_effect.inputs.len(),
                                    inferred_effect.outputs.len()
                                ),
                            });
                        }
                    }
                    Err(e) => {
                        self.error(e);
                    }
                }
            }
        }

        Ok(())
    }

    /// Validate a word
    fn validate_word(&mut self, word: &Word) -> Result<()> {
        match word {
            Word::WordRef { name, .. } => {
                if !self.is_defined(name) {
                    self.error(ForthError::UndefinedWord {
                        word: name.clone(),
                        line: None,
                    });
                }
            }
            Word::If {
                then_branch,
                else_branch,
            } => {
                for w in then_branch {
                    self.validate_word(w)?;
                }
                if let Some(else_words) = else_branch {
                    for w in else_words {
                        self.validate_word(w)?;
                    }
                }
            }
            Word::BeginUntil { body } => {
                for w in body {
                    self.validate_word(w)?;
                }
            }
            Word::BeginWhileRepeat { condition, body } => {
                for w in condition {
                    self.validate_word(w)?;
                }
                for w in body {
                    self.validate_word(w)?;
                }
            }
            Word::DoLoop { body, .. } => {
                for w in body {
                    self.validate_word(w)?;
                }
            }
            _ => {}
        }

        Ok(())
    }

    /// Check if a word sequence contains complex control flow (loops, return stack ops)
    fn has_complex_control_flow(&self, words: &[Word]) -> bool {
        for word in words {
            match word {
                Word::BeginUntil { .. } | Word::BeginWhileRepeat { .. } | Word::DoLoop { .. } => {
                    return true;
                }
                Word::WordRef { name, .. } if matches!(name.as_str(), ">r" | "r>" | "r@") => {
                    return true;
                }
                Word::If { then_branch, else_branch } => {
                    if self.has_complex_control_flow(then_branch) {
                        return true;
                    }
                    if let Some(else_words) = else_branch {
                        if self.has_complex_control_flow(else_words) {
                            return true;
                        }
                    }
                }
                _ => {}
            }
        }
        false
    }

    /// Validate control structure balance
    fn validate_control_structures(&mut self, words: &[Word]) -> Result<()> {
        // Note: Control structures are already validated during parsing.
        // Word::If, Word::BeginUntil, etc. are complete, balanced structures.
        // We only need to recursively validate nested structures.

        for word in words {
            match word {
                Word::If { then_branch, else_branch } => {
                    // Recursively validate branches
                    self.validate_control_structures(then_branch)?;
                    if let Some(else_words) = else_branch {
                        self.validate_control_structures(else_words)?;
                    }
                }
                Word::BeginUntil { body } => {
                    self.validate_control_structures(body)?;
                }
                Word::BeginWhileRepeat { condition, body } => {
                    self.validate_control_structures(condition)?;
                    self.validate_control_structures(body)?;
                }
                Word::DoLoop { body, .. } => {
                    self.validate_control_structures(body)?;
                }
                _ => {}
            }
        }

        Ok(())
    }

    /// Get collected errors
    pub fn errors(&self) -> &[ForthError] {
        &self.errors
    }

    /// Check if analysis passed
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }
}

impl Default for SemanticAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience function to analyze a program
pub fn analyze(program: &Program) -> Result<()> {
    let mut analyzer = SemanticAnalyzer::new();
    analyzer.analyze(program)
}

/// Validation result with detailed information
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub passed: bool,
    pub errors: Vec<ForthError>,
    pub warnings: Vec<String>,
}

impl ValidationResult {
    pub fn success() -> Self {
        Self {
            passed: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    pub fn with_errors(errors: Vec<ForthError>) -> Self {
        Self {
            passed: errors.is_empty(),
            errors,
            warnings: Vec::new(),
        }
    }
}

/// Perform comprehensive validation
pub fn validate_program(program: &Program) -> ValidationResult {
    let mut analyzer = SemanticAnalyzer::new();

    match analyzer.analyze(program) {
        Ok(_) => ValidationResult::success(),
        Err(_) => ValidationResult::with_errors(analyzer.errors.clone()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_program;

    #[test]
    fn test_validate_simple() {
        let program = parse_program(": double 2 * ;").unwrap();
        assert!(analyze(&program).is_ok());
    }

    #[test]
    fn test_undefined_word() {
        let program = parse_program(": test undefined-word ;").unwrap();
        let result = analyze(&program);
        assert!(result.is_err());
        if let Err(ForthError::UndefinedWord { word, .. }) = result {
            assert_eq!(word, "undefined-word");
        } else {
            panic!("Expected UndefinedWord error");
        }
    }

    #[test]
    fn test_stack_effect_mismatch() {
        // Declared ( n -- n n ) but actually ( n -- n )
        let program = parse_program(": test ( n -- n n ) drop ;").unwrap();
        let result = analyze(&program);
        // Should detect mismatch
        assert!(result.is_err());
    }

    #[test]
    fn test_valid_control_structures() {
        let program = parse_program(": abs dup 0 < IF negate THEN ;").unwrap();
        assert!(analyze(&program).is_ok());
    }

    #[test]
    fn test_nested_words() {
        let program = parse_program(
            ": double 2 * ;
             : quadruple double double ;"
        ).unwrap();
        assert!(analyze(&program).is_ok());
    }

    #[test]
    fn test_validation_result() {
        let program = parse_program(": test 1 2 + ;").unwrap();
        let result = validate_program(&program);
        assert!(result.passed);
        assert!(result.errors.is_empty());
    }
}
