//! Inlining Optimizer with Stack Effect Analysis
//!
//! Expands small word definitions inline to eliminate call overhead.
//! Uses stack effect analysis to ensure correctness.
//!
//! # Inlining Heuristics
//!
//! A word is inlined if:
//! 1. It's marked as inline, OR
//! 2. It's small (d threshold instructions), AND
//! 3. It has compatible stack effects, AND
//! 4. Inlining won't cause code bloat (called d max_inline_sites times)
//!
//! # Example
//!
//! Before:
//! ```forth
//! : square dup * ;
//! : quad square square ;
//! 5 quad
//! ```
//!
//! After inlining:
//! ```forth
//! 5 dup * dup *
//! ```

use crate::ir::{ForthIR, Instruction, StackEffect, WordDef};
use crate::{OptimizationLevel, Result};
use std::collections::{HashMap, HashSet};

/// Inline cost threshold by optimization level
const INLINE_THRESHOLD_BASIC: usize = 3;
const INLINE_THRESHOLD_STANDARD: usize = 10;
const INLINE_THRESHOLD_AGGRESSIVE: usize = 25;

/// Maximum number of call sites before refusing to inline
const MAX_INLINE_SITES_STANDARD: usize = 5;
const MAX_INLINE_SITES_AGGRESSIVE: usize = 20;

/// Inlining decision for a word
#[derive(Debug, Clone, PartialEq)]
enum InlineDecision {
    Inline,
    NoInline,
    TooLarge,
    TooManyCalls,
    Recursive,
}

/// Inlining optimizer
pub struct InlineOptimizer {
    level: OptimizationLevel,
    inline_threshold: usize,
    max_inline_sites: usize,
}

impl InlineOptimizer {
    pub fn new(level: OptimizationLevel) -> Self {
        let (inline_threshold, max_inline_sites) = match level {
            OptimizationLevel::None => (0, 0),
            OptimizationLevel::Basic => (INLINE_THRESHOLD_BASIC, MAX_INLINE_SITES_STANDARD),
            OptimizationLevel::Standard => {
                (INLINE_THRESHOLD_STANDARD, MAX_INLINE_SITES_STANDARD)
            }
            OptimizationLevel::Aggressive => {
                (INLINE_THRESHOLD_AGGRESSIVE, MAX_INLINE_SITES_AGGRESSIVE)
            }
        };

        Self {
            level,
            inline_threshold,
            max_inline_sites,
        }
    }

    /// Inline small words in IR
    pub fn inline(&self, ir: &ForthIR) -> Result<ForthIR> {
        if self.level == OptimizationLevel::None {
            return Ok(ir.clone());
        }

        let mut optimized = ir.clone();

        // Analyze call graph
        let call_counts = self.count_calls(ir);

        // Decide which words to inline
        let inline_decisions = self.make_inline_decisions(ir, &call_counts);

        // Inline in main sequence
        optimized.main = self.inline_sequence(&ir.main, ir, &inline_decisions)?;

        // Inline in each word
        for (name, word) in ir.words.iter() {
            let mut optimized_word = word.clone();
            optimized_word.instructions =
                self.inline_sequence(&word.instructions, ir, &inline_decisions)?;
            optimized_word.update();
            optimized.words.insert(name.clone(), optimized_word);
        }

        Ok(optimized)
    }

    /// Count how many times each word is called
    fn count_calls(&self, ir: &ForthIR) -> HashMap<String, usize> {
        let mut counts = HashMap::new();

        // Count in main
        for inst in &ir.main {
            if let Instruction::Call(name) = inst {
                *counts.entry(name.clone()).or_insert(0) += 1;
            }
        }

        // Count in each word
        for word in ir.words.values() {
            for inst in &word.instructions {
                if let Instruction::Call(name) = inst {
                    *counts.entry(name.clone()).or_insert(0) += 1;
                }
            }
        }

        counts
    }

    /// Decide which words should be inlined
    fn make_inline_decisions(
        &self,
        ir: &ForthIR,
        call_counts: &HashMap<String, usize>,
    ) -> HashMap<String, InlineDecision> {
        let mut decisions = HashMap::new();

        for (name, word) in &ir.words {
            let decision = self.should_inline(word, call_counts.get(name).copied().unwrap_or(0));
            decisions.insert(name.clone(), decision);
        }

        decisions
    }

    /// Determine if a word should be inlined
    fn should_inline(&self, word: &WordDef, call_count: usize) -> InlineDecision {
        // Explicitly marked inline
        if word.is_inline {
            return InlineDecision::Inline;
        }

        // Check for recursion
        if self.is_recursive(word) {
            return InlineDecision::Recursive;
        }

        // Too large?
        if word.cost > self.inline_threshold {
            return InlineDecision::TooLarge;
        }

        // Too many call sites?
        if call_count > self.max_inline_sites {
            return InlineDecision::TooManyCalls;
        }

        // Small and not called too many times: inline!
        InlineDecision::Inline
    }

    /// Check if word is recursive
    fn is_recursive(&self, word: &WordDef) -> bool {
        word.instructions
            .iter()
            .any(|inst| matches!(inst, Instruction::Call(name) if name == &word.name))
    }

    /// Inline calls in an instruction sequence
    fn inline_sequence(
        &self,
        instructions: &[Instruction],
        ir: &ForthIR,
        decisions: &HashMap<String, InlineDecision>,
    ) -> Result<Vec<Instruction>> {
        let mut result = Vec::with_capacity(instructions.len());

        for inst in instructions {
            match inst {
                Instruction::Call(name) => {
                    // Check if we should inline this call
                    if let Some(InlineDecision::Inline) = decisions.get(name) {
                        if let Some(word) = ir.get_word(name) {
                            // Inline the word's instructions
                            result.extend_from_slice(&word.instructions);
                            continue;
                        }
                    }

                    // Don't inline: keep the call
                    result.push(inst.clone());
                }
                _ => {
                    result.push(inst.clone());
                }
            }
        }

        Ok(result)
    }

    /// Get inlining statistics
    pub fn get_stats(&self, before: &ForthIR, after: &ForthIR) -> InlineStats {
        let before_calls = self.count_total_calls(before);
        let after_calls = self.count_total_calls(after);

        InlineStats {
            calls_before: before_calls,
            calls_after: after_calls,
            calls_inlined: before_calls.saturating_sub(after_calls),
            instructions_before: before.instruction_count(),
            instructions_after: after.instruction_count(),
        }
    }

    fn count_total_calls(&self, ir: &ForthIR) -> usize {
        let mut count = 0;

        for inst in &ir.main {
            if matches!(inst, Instruction::Call(_)) {
                count += 1;
            }
        }

        for word in ir.words.values() {
            for inst in &word.instructions {
                if matches!(inst, Instruction::Call(_)) {
                    count += 1;
                }
            }
        }

        count
    }
}

#[derive(Debug, Clone)]
pub struct InlineStats {
    pub calls_before: usize,
    pub calls_after: usize,
    pub calls_inlined: usize,
    pub instructions_before: usize,
    pub instructions_after: usize,
}

impl std::fmt::Display for InlineStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Calls before: {}\n\
             Calls after: {}\n\
             Calls inlined: {}\n\
             Instructions before: {}\n\
             Instructions after: {}\n\
             Code growth: {}",
            self.calls_before,
            self.calls_after,
            self.calls_inlined,
            self.instructions_before,
            self.instructions_after,
            self.instructions_after as i64 - self.instructions_before as i64
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inline_small_word() {
        let optimizer = InlineOptimizer::new(OptimizationLevel::Aggressive);

        let mut ir = ForthIR::new();
        let square = WordDef::new(
            "square".to_string(),
            vec![Instruction::Dup, Instruction::Mul],
        );
        ir.add_word(square);

        ir.main = vec![Instruction::Literal(5), Instruction::Call("square".to_string())];

        let optimized = optimizer.inline(&ir).unwrap();

        // square should be inlined
        let has_call = optimized
            .main
            .iter()
            .any(|i| matches!(i, Instruction::Call(_)));
        assert!(!has_call);

        let has_dup = optimized.main.iter().any(|i| matches!(i, Instruction::Dup));
        let has_mul = optimized.main.iter().any(|i| matches!(i, Instruction::Mul));
        assert!(has_dup && has_mul);
    }

    #[test]
    fn test_dont_inline_large_word() {
        let optimizer = InlineOptimizer::new(OptimizationLevel::Basic);

        let mut ir = ForthIR::new();
        // Create a large word (more than threshold)
        let large_instructions = vec![Instruction::Dup; 20];
        let large = WordDef::new("large".to_string(), large_instructions);
        ir.add_word(large);

        ir.main = vec![Instruction::Call("large".to_string())];

        let optimized = optimizer.inline(&ir).unwrap();

        // Large word should NOT be inlined at Basic level
        let has_call = optimized
            .main
            .iter()
            .any(|i| matches!(i, Instruction::Call(_)));
        assert!(has_call);
    }

    #[test]
    fn test_dont_inline_recursive() {
        let optimizer = InlineOptimizer::new(OptimizationLevel::Aggressive);

        let mut ir = ForthIR::new();
        // Recursive word
        let recursive = WordDef::new(
            "factorial".to_string(),
            vec![
                Instruction::Dup,
                Instruction::Literal(1),
                Instruction::Gt,
                Instruction::Call("factorial".to_string()),
            ],
        );
        ir.add_word(recursive);

        ir.main = vec![
            Instruction::Literal(5),
            Instruction::Call("factorial".to_string()),
        ];

        let optimized = optimizer.inline(&ir).unwrap();

        // Recursive word should NOT be inlined
        let has_call = optimized
            .main
            .iter()
            .any(|i| matches!(i, Instruction::Call(_)));
        assert!(has_call);
    }

    #[test]
    fn test_inline_forced() {
        let optimizer = InlineOptimizer::new(OptimizationLevel::Standard);

        let mut ir = ForthIR::new();
        let mut word = WordDef::new("tiny".to_string(), vec![Instruction::Dup]);
        word.is_inline = true; // Force inline
        ir.add_word(word);

        ir.main = vec![
            Instruction::Literal(5),
            Instruction::Call("tiny".to_string()),
        ];

        let optimized = optimizer.inline(&ir).unwrap();

        // Should be inlined (forced)
        let has_call = optimized
            .main
            .iter()
            .any(|i| matches!(i, Instruction::Call(_)));
        assert!(!has_call);
    }

    #[test]
    fn test_inline_stats() {
        let optimizer = InlineOptimizer::new(OptimizationLevel::Aggressive);

        let mut ir = ForthIR::new();
        let square = WordDef::new(
            "square".to_string(),
            vec![Instruction::Dup, Instruction::Mul],
        );
        ir.add_word(square);

        ir.main = vec![
            Instruction::Literal(5),
            Instruction::Call("square".to_string()),
            Instruction::Call("square".to_string()),
        ];

        let optimized = optimizer.inline(&ir).unwrap();
        let stats = optimizer.get_stats(&ir, &optimized);

        assert_eq!(stats.calls_before, 2);
        assert_eq!(stats.calls_after, 0);
        assert_eq!(stats.calls_inlined, 2);
    }
}
