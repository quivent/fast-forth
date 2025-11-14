//! Confidence score calculation for fix suggestions
//!
//! Confidence scores are computed based on:
//! - Pattern match quality
//! - Error context
//! - Historical success rates
//! - Code complexity

use crate::errors::StructuredError;
use crate::diagnostics::patterns::FixPattern;

pub struct ConfidenceCalculator;

impl ConfidenceCalculator {
    /// Calculate confidence score for a fix suggestion
    ///
    /// Returns a value between 0.0 and 1.0 where:
    /// - 0.9-1.0: Very high confidence (almost certain)
    /// - 0.7-0.9: High confidence (likely correct)
    /// - 0.5-0.7: Medium confidence (possible fix)
    /// - 0.3-0.5: Low confidence (uncertain)
    /// - 0.0-0.3: Very low confidence (unlikely)
    pub fn calculate(
        pattern: &FixPattern,
        error: &StructuredError,
        context: &FixContext,
    ) -> f64 {
        let mut score = pattern.base_confidence;

        // Boost confidence for exact error code match
        if pattern.error_code == error.code {
            score *= 1.1;
        }

        // Boost for pattern match quality
        if let Some(contains) = &pattern.pattern_match.error_contains {
            let match_count = contains
                .iter()
                .filter(|p| error.error.to_lowercase().contains(&p.to_lowercase()))
                .count();

            if match_count == contains.len() {
                score *= 1.15; // All patterns match
            } else if match_count > 0 {
                score *= 1.0 + (0.15 * match_count as f64 / contains.len() as f64);
            }
        }

        // Adjust for code complexity
        score *= Self::complexity_factor(context.code_length, context.nesting_depth);

        // Adjust for location confidence
        if context.has_precise_location {
            score *= 1.05;
        }

        // Cap at 1.0
        score.min(1.0)
    }

    /// Calculate complexity factor
    ///
    /// More complex code reduces confidence as there's more room for error
    fn complexity_factor(code_length: usize, nesting_depth: usize) -> f64 {
        let length_factor = match code_length {
            0..=20 => 1.0,
            21..=50 => 0.95,
            51..=100 => 0.90,
            101..=200 => 0.85,
            _ => 0.80,
        };

        let nesting_factor = match nesting_depth {
            0..=1 => 1.0,
            2 => 0.95,
            3 => 0.90,
            4 => 0.85,
            _ => 0.80,
        };

        length_factor * nesting_factor
    }

    /// Rank multiple suggestions by confidence
    pub fn rank_suggestions(
        mut suggestions: Vec<(FixPattern, f64)>,
    ) -> Vec<(FixPattern, f64)> {
        suggestions.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        suggestions
    }
}

/// Context information for confidence calculation
#[derive(Debug, Clone)]
pub struct FixContext {
    pub code_length: usize,
    pub nesting_depth: usize,
    pub has_precise_location: bool,
    pub word_exists: bool,
}

impl Default for FixContext {
    fn default() -> Self {
        Self {
            code_length: 0,
            nesting_depth: 0,
            has_precise_location: false,
            word_exists: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::{ErrorCode, Location};

    #[test]
    fn test_complexity_factor() {
        assert_eq!(ConfidenceCalculator::complexity_factor(10, 0), 1.0);
        assert!(ConfidenceCalculator::complexity_factor(100, 3) < 1.0);
    }

    #[test]
    fn test_confidence_calculation() {
        use crate::diagnostics::patterns::*;

        let pattern = FixPattern {
            id: "TEST".to_string(),
            name: "Test".to_string(),
            description: "Test pattern".to_string(),
            error_code: "E2234".to_string(),
            pattern_match: PatternMatch {
                error_contains: Some(vec!["stack".to_string()]),
                stack_effect_pattern: None,
                word_pattern: None,
            },
            fix_template: "drop".to_string(),
            base_confidence: 0.8,
            examples: vec![],
        };

        let error = StructuredError::new(ErrorCode::StackDepthMismatch, "stack depth error")
            .with_location(Location::new(1, 1));

        let context = FixContext {
            code_length: 10,
            nesting_depth: 0,
            has_precise_location: true,
            word_exists: true,
        };

        let confidence = ConfidenceCalculator::calculate(&pattern, &error, &context);
        assert!(confidence >= 0.8);
        assert!(confidence <= 1.0);
    }
}
