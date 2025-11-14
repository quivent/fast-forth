//! Diagnostic engine for auto-fix suggestions
//!
//! This module analyzes errors and generates fix suggestions with confidence scores.
//! It uses pattern matching and heuristics to suggest the most likely fixes.

pub mod fix_engine;
pub mod patterns;
pub mod confidence;

pub use fix_engine::{FixEngine, FixSuggestion};
pub use patterns::{FixPattern, PATTERN_REGISTRY};
pub use confidence::ConfidenceCalculator;

use crate::errors::{StructuredError, Suggestion};

/// Generate fix suggestions for an error
pub fn suggest_fixes(error: &StructuredError, max_alternatives: usize) -> Vec<Suggestion> {
    let engine = FixEngine::new();
    engine.suggest_fixes(error, max_alternatives)
}

/// Get the best fix suggestion for an error
pub fn suggest_best_fix(error: &StructuredError) -> Option<Suggestion> {
    suggest_fixes(error, 1).into_iter().next()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fix_suggestion() {
        // This will be tested with actual error patterns
        let engine = FixEngine::new();
        assert!(engine.patterns().len() > 0);
    }
}
