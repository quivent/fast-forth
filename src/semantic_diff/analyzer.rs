//! Performance Analyzer
//!
//! Analyzes and predicts performance characteristics

use super::PerformanceMetrics;
use fastforth_frontend::{Definition, Word};

/// Performance analyzer
pub struct PerformanceAnalyzer;

impl PerformanceAnalyzer {
    pub fn new() -> Self {
        Self
    }

    /// Analyze a definition for performance metrics
    pub fn analyze_definition(&self, def: &Definition) -> PerformanceMetrics {
        let operation_count = self.count_operations(&def.body);
        let stack_depth_max = self.estimate_max_stack_depth(&def.body);
        let complexity_class = self.classify_complexity(&def.body);

        PerformanceMetrics {
            operation_count,
            stack_depth_max,
            complexity_class,
        }
    }

    /// Count the number of operations
    fn count_operations(&self, body: &[Word]) -> usize {
        let mut count = 0;
        for word in body {
            count += match word {
                Word::WordRef { .. } => 1,
                Word::IntLiteral(_) | Word::FloatLiteral(_) | Word::StringLiteral(_) => 1,
                Word::If { then_branch, else_branch } => {
                    let then_count = self.count_operations(then_branch);
                    let else_count = else_branch.as_ref()
                        .map(|b| self.count_operations(b))
                        .unwrap_or(0);
                    1 + then_count.max(else_count)
                }
                Word::BeginUntil { body } => {
                    1 + self.count_operations(body) * 5 // Estimate loop iterations
                }
                Word::BeginWhileRepeat { condition, body } => {
                    1 + (self.count_operations(condition) + self.count_operations(body)) * 5
                }
                Word::DoLoop { body, .. } => {
                    1 + self.count_operations(body) * 5
                }
                _ => 0,
            };
        }
        count
    }

    /// Estimate maximum stack depth
    fn estimate_max_stack_depth(&self, body: &[Word]) -> usize {
        let mut current_depth: usize = 0;
        let mut max_depth: usize = 0;

        for word in body {
            match word {
                Word::IntLiteral(_) | Word::FloatLiteral(_) | Word::StringLiteral(_) => {
                    current_depth += 1;
                }
                Word::WordRef { name, .. } => {
                    // Simple heuristic based on common words
                    current_depth = match name.as_str() {
                        "dup" => current_depth + 1,
                        "drop" => current_depth.saturating_sub(1),
                        "swap" | "over" | "rot" => current_depth,
                        "+" | "-" | "*" | "/" | "mod" => current_depth.saturating_sub(1),
                        _ => current_depth,
                    };
                }
                _ => {}
            }
            max_depth = max_depth.max(current_depth);
        }

        max_depth
    }

    /// Classify computational complexity
    fn classify_complexity(&self, body: &[Word]) -> String {
        let has_loop = body.iter().any(|w| matches!(
            w,
            Word::BeginUntil { .. } | Word::BeginWhileRepeat { .. } | Word::DoLoop { .. }
        ));

        let has_recursion = body.iter().any(|w| matches!(
            w,
            Word::WordRef { name, .. } if name == "recurse"
        ));

        if has_recursion {
            "O(n) recursive".to_string()
        } else if has_loop {
            "O(n) iterative".to_string()
        } else {
            format!("O(1) {} ops", self.count_operations(body))
        }
    }
}

impl Default for PerformanceAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fastforth_frontend::parse_program;

    #[test]
    fn test_count_operations() {
        let program = parse_program(": square dup * ;").unwrap();
        let analyzer = PerformanceAnalyzer::new();

        let metrics = analyzer.analyze_definition(&program.definitions[0]);
        assert_eq!(metrics.operation_count, 2); // dup + *
    }

    #[test]
    fn test_complexity_classification() {
        let program = parse_program(": square dup * ;").unwrap();
        let analyzer = PerformanceAnalyzer::new();

        let metrics = analyzer.analyze_definition(&program.definitions[0]);
        assert!(metrics.complexity_class.contains("O(1)"));
    }
}
