//! Fix suggestion engine
//!
//! This engine analyzes errors and generates ranked fix suggestions
//! using pattern matching and confidence scoring.

use crate::errors::{StructuredError, Suggestion, FixDiff};
use crate::diagnostics::patterns::{FixPattern, PATTERN_REGISTRY};
use crate::diagnostics::confidence::{ConfidenceCalculator, FixContext};

/// Fix suggestion with metadata
#[derive(Debug, Clone)]
pub struct FixSuggestion {
    pub pattern: FixPattern,
    pub suggestion: Suggestion,
    pub confidence: f64,
}

pub struct FixEngine {
    max_suggestions: usize,
}

impl FixEngine {
    pub fn new() -> Self {
        Self {
            max_suggestions: 5,
        }
    }

    pub fn with_max_suggestions(mut self, max: usize) -> Self {
        self.max_suggestions = max;
        self
    }

    /// Generate fix suggestions for an error
    pub fn suggest_fixes(
        &self,
        error: &StructuredError,
        max_alternatives: usize,
    ) -> Vec<Suggestion> {
        // Find matching patterns
        let patterns = PATTERN_REGISTRY.find_matching(&error.code, &error.error);

        if patterns.is_empty() {
            return vec![];
        }

        // Create context for confidence calculation
        let context = self.create_context(error);

        // Calculate confidence for each pattern
        let mut scored_patterns: Vec<_> = patterns
            .into_iter()
            .map(|pattern| {
                let confidence = ConfidenceCalculator::calculate(pattern, error, &context);
                (pattern, confidence)
            })
            .collect();

        // Sort by confidence
        scored_patterns.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        // Take top suggestions
        let top_suggestions: Vec<_> = scored_patterns
            .into_iter()
            .take(max_alternatives.min(self.max_suggestions))
            .collect();

        // Convert to Suggestion objects
        top_suggestions
            .into_iter()
            .map(|(pattern, confidence)| {
                self.create_suggestion(pattern, error, confidence)
            })
            .collect()
    }

    /// Create a suggestion from a pattern
    fn create_suggestion(
        &self,
        pattern: &FixPattern,
        error: &StructuredError,
        confidence: f64,
    ) -> Suggestion {
        // Extract code context if available
        let old_code = error.location.context.as_deref().unwrap_or("");

        // Apply pattern template
        let new_code = pattern.apply(old_code).unwrap_or_else(|| {
            pattern.fix_template.clone()
        });

        Suggestion {
            pattern: Some(pattern.id.clone()),
            fix: pattern.name.clone(),
            confidence,
            diff: FixDiff {
                old: old_code.to_string(),
                new: new_code,
            },
            explanation: Some(pattern.description.clone()),
        }
    }

    /// Create fix context from error
    fn create_context(&self, error: &StructuredError) -> FixContext {
        FixContext {
            code_length: error.location.context.as_ref().map_or(0, |c| c.len()),
            nesting_depth: self.estimate_nesting(error),
            has_precise_location: error.location.line > 0 && error.location.column > 0,
            word_exists: error.location.word.is_some(),
        }
    }

    /// Estimate nesting depth from error context
    fn estimate_nesting(&self, error: &StructuredError) -> usize {
        // Simple heuristic: count control structure keywords
        if let Some(context) = &error.location.context {
            let keywords = ["if", "begin", "do"];
            keywords.iter()
                .map(|kw| context.matches(kw).count())
                .sum()
        } else {
            0
        }
    }

    /// Get all available patterns
    pub fn patterns(&self) -> Vec<&FixPattern> {
        PATTERN_REGISTRY.all_patterns()
    }
}

impl Default for FixEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::{ErrorCode, Location};

    #[test]
    fn test_fix_engine_creation() {
        let engine = FixEngine::new();
        assert!(engine.max_suggestions > 0);
    }

    #[test]
    fn test_suggest_fixes() {
        let engine = FixEngine::new();

        let error = StructuredError::new(
            ErrorCode::StackDepthMismatch,
            "Stack depth mismatch: excess items"
        )
        .with_location(
            Location::new(5, 10)
                .with_context("dup dup *")
        );

        let suggestions = engine.suggest_fixes(&error, 3);
        assert!(!suggestions.is_empty());

        // First suggestion should have highest confidence
        if suggestions.len() > 1 {
            assert!(suggestions[0].confidence >= suggestions[1].confidence);
        }
    }

    #[test]
    fn test_pattern_listing() {
        let engine = FixEngine::new();
        let patterns = engine.patterns();
        assert!(!patterns.is_empty());
    }
}
