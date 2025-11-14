//! Semantic Differ
//!
//! Core logic for semantic comparison

use super::{SemanticDiff, PerformanceMetrics};
use super::analyzer::PerformanceAnalyzer;
use crate::symbolic::EquivalenceChecker;
use fastforth_frontend::{Program, Definition, parse_program};
use serde::{Serialize, Deserialize};
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DiffError {
    #[error("Parse error in old file: {0}")]
    OldParseError(String),

    #[error("Parse error in new file: {0}")]
    NewParseError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Definition not found: {0}")]
    DefinitionNotFound(String),
}

/// Result of semantic diff
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffResult {
    pub diffs: Vec<SemanticDiff>,
    pub total_words: usize,
    pub changed_words: usize,
    pub unchanged_words: usize,
}

impl DiffResult {
    pub fn new() -> Self {
        Self {
            diffs: Vec::new(),
            total_words: 0,
            changed_words: 0,
            unchanged_words: 0,
        }
    }

    pub fn add_diff(&mut self, diff: SemanticDiff) {
        self.total_words += 1;
        if diff.stack_effect_changed || diff.operations_changed || diff.performance_changed {
            self.changed_words += 1;
        } else {
            self.unchanged_words += 1;
        }
        self.diffs.push(diff);
    }
}

impl Default for DiffResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Semantic differ
pub struct SemanticDiffer {
    equivalence_checker: EquivalenceChecker,
    performance_analyzer: PerformanceAnalyzer,
}

impl SemanticDiffer {
    pub fn new() -> Self {
        Self {
            equivalence_checker: EquivalenceChecker::new(),
            performance_analyzer: PerformanceAnalyzer::new(),
        }
    }

    /// Compare two programs from file paths
    pub fn diff_files(&self, old_path: &Path, new_path: &Path) -> Result<DiffResult, DiffError> {
        let old_source = std::fs::read_to_string(old_path)?;
        let new_source = std::fs::read_to_string(new_path)?;

        self.diff_sources(&old_source, &new_source)
    }

    /// Compare two programs from source strings
    pub fn diff_sources(&self, old_source: &str, new_source: &str) -> Result<DiffResult, DiffError> {
        let old_program = parse_program(old_source)
            .map_err(|e| DiffError::OldParseError(format!("{:?}", e)))?;

        let new_program = parse_program(new_source)
            .map_err(|e| DiffError::NewParseError(format!("{:?}", e)))?;

        self.diff_programs(&old_program, &new_program)
    }

    /// Compare two programs
    pub fn diff_programs(&self, old: &Program, new: &Program) -> Result<DiffResult, DiffError> {
        let mut result = DiffResult::new();

        // Build maps of definitions
        let old_defs: std::collections::HashMap<_, _> = old
            .definitions
            .iter()
            .map(|d| (d.name.clone(), d))
            .collect();

        let new_defs: std::collections::HashMap<_, _> = new
            .definitions
            .iter()
            .map(|d| (d.name.clone(), d))
            .collect();

        // Find all unique word names
        let mut all_names: Vec<String> = old_defs
            .keys()
            .chain(new_defs.keys())
            .cloned()
            .collect();
        all_names.sort();
        all_names.dedup();

        // Compare each word
        for name in all_names {
            let diff = match (old_defs.get(&name), new_defs.get(&name)) {
                (Some(old_def), Some(new_def)) => {
                    self.diff_definitions(old_def, new_def)
                }
                (Some(old_def), None) => {
                    let mut diff = SemanticDiff::new(name.clone());
                    diff.operations_changed = true;
                    diff.stack_effect_old = format!("{:?}", old_def.stack_effect);
                    diff.stack_effect_new = "removed".to_string();
                    diff.semantically_equivalent = false;
                    diff.recommendation = "⚠ Word removed - check for usages".to_string();
                    diff
                }
                (None, Some(new_def)) => {
                    let mut diff = SemanticDiff::new(name.clone());
                    diff.operations_changed = true;
                    diff.stack_effect_old = "added".to_string();
                    diff.stack_effect_new = format!("{:?}", new_def.stack_effect);
                    diff.semantically_equivalent = false;
                    diff.recommendation = "✓ New word added".to_string();
                    diff
                }
                (None, None) => unreachable!(),
            };

            result.add_diff(diff);
        }

        Ok(result)
    }

    /// Compare two definitions
    pub fn diff_definitions(&self, old: &Definition, new: &Definition) -> SemanticDiff {
        let mut diff = SemanticDiff::new(old.name.clone());

        // Compare stack effects
        let old_effect = old.stack_effect.as_ref().map(|e| format!("{}", e));
        let new_effect = new.stack_effect.as_ref().map(|e| format!("{}", e));

        diff.stack_effect_old = old_effect.clone().unwrap_or_else(|| "unknown".to_string());
        diff.stack_effect_new = new_effect.clone().unwrap_or_else(|| "unknown".to_string());
        diff.stack_effect_changed = old_effect != new_effect;

        // Extract operations
        diff.operations_old = self.extract_operations(&old.body);
        diff.operations_new = self.extract_operations(&new.body);
        diff.operations_changed = diff.operations_old != diff.operations_new;

        // Analyze performance
        diff.performance_old = self.performance_analyzer.analyze_definition(old);
        diff.performance_new = self.performance_analyzer.analyze_definition(new);
        diff.performance_changed = diff.performance_old.operation_count != diff.performance_new.operation_count;

        // Check semantic equivalence
        let equiv_result = self.equivalence_checker.check_definitions(old, new);
        diff.semantically_equivalent = equiv_result.equivalent;

        // Generate recommendation
        diff.generate_recommendation();

        diff
    }

    /// Extract operation names from a word body
    fn extract_operations(&self, body: &[fastforth_frontend::Word]) -> Vec<String> {
        use fastforth_frontend::Word;

        let mut ops = Vec::new();
        for word in body {
            match word {
                Word::WordRef { name, .. } => ops.push(name.clone()),
                Word::IntLiteral(n) => ops.push(n.to_string()),
                Word::FloatLiteral(f) => ops.push(f.to_string()),
                Word::StringLiteral(s) => ops.push(format!("\"{}\"", s)),
                _ => ops.push("<complex>".to_string()),
            }
        }
        ops
    }
}

impl Default for SemanticDiffer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diff_identical() {
        let source = ": square dup * ;";
        let differ = SemanticDiffer::new();

        let result = differ.diff_sources(source, source).unwrap();
        assert_eq!(result.total_words, 1);
        assert_eq!(result.unchanged_words, 1);
    }

    #[test]
    fn test_diff_changed() {
        let old = ": double 2 * ;";
        let new = ": double dup + ;";
        let differ = SemanticDiffer::new();

        let result = differ.diff_sources(old, new).unwrap();
        assert_eq!(result.total_words, 1);
        assert!(result.diffs[0].operations_changed);
    }
}
