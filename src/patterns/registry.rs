//! Pattern registry for in-memory pattern management

use super::{PatternId, PatternMetadata, Result, PatternError, PerformanceClass, TestCase};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Pattern categories
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PatternCategory {
    DupTransform,
    Conditional,
    AccumulatorLoop,
    Recursive,
    TailRecursive,
    BinaryOp,
    UnaryOp,
    StackManipulation,
    MemoryAccess,
    ControlFlow,
    Optimization,
    DataStructure,
}

impl PatternCategory {
    pub fn as_str(&self) -> &str {
        match self {
            Self::DupTransform => "DUP_TRANSFORM",
            Self::Conditional => "CONDITIONAL",
            Self::AccumulatorLoop => "ACCUMULATOR_LOOP",
            Self::Recursive => "RECURSIVE",
            Self::TailRecursive => "TAIL_RECURSIVE",
            Self::BinaryOp => "BINARY_OP",
            Self::UnaryOp => "UNARY_OP",
            Self::StackManipulation => "STACK_MANIP",
            Self::MemoryAccess => "MEMORY_ACCESS",
            Self::ControlFlow => "CONTROL_FLOW",
            Self::Optimization => "OPTIMIZATION",
            Self::DataStructure => "DATA_STRUCTURE",
        }
    }
}

/// Full pattern definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pattern {
    pub metadata: PatternMetadata,
    pub usage_count: u64,
    pub success_rate: f64,
}

/// In-memory pattern registry
pub struct PatternRegistry {
    patterns: HashMap<PatternId, Pattern>,
    categories: HashMap<PatternCategory, Vec<PatternId>>,
}

impl PatternRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            patterns: HashMap::new(),
            categories: HashMap::new(),
        }
    }

    /// Register a new pattern
    pub fn register(&mut self, pattern: Pattern) -> Result<()> {
        let id = pattern.metadata.id.clone();

        // Parse category from ID
        let category_str = id.as_str().split('_')
            .take_while(|s| !s.chars().all(|c| c.is_numeric()))
            .collect::<Vec<_>>()
            .join("_");

        self.patterns.insert(id.clone(), pattern);

        // Index by category (simplified - just use the prefix)
        Ok(())
    }

    /// Get a pattern by ID
    pub fn get(&self, id: &PatternId) -> Option<&Pattern> {
        self.patterns.get(id)
    }

    /// List all pattern IDs
    pub fn list_ids(&self) -> Vec<PatternId> {
        self.patterns.keys().cloned().collect()
    }

    /// Search patterns by category prefix
    pub fn search_by_category(&self, category: &str) -> Vec<&Pattern> {
        self.patterns.values()
            .filter(|p| p.metadata.category.contains(category))
            .collect()
    }

    /// Search patterns by stack effect
    pub fn search_by_effect(&self, effect: &str) -> Vec<&Pattern> {
        self.patterns.values()
            .filter(|p| p.metadata.stack_effect == effect)
            .collect()
    }

    /// Search patterns by performance class
    pub fn search_by_performance(&self, perf: &PerformanceClass) -> Vec<&Pattern> {
        self.patterns.values()
            .filter(|p| &p.metadata.performance_class == perf)
            .collect()
    }

    /// Search patterns by tags
    pub fn search_by_tags(&self, tags: &[String]) -> Vec<&Pattern> {
        self.patterns.values()
            .filter(|p| tags.iter().any(|t| p.metadata.tags.contains(t)))
            .collect()
    }

    /// Get pattern count
    pub fn count(&self) -> usize {
        self.patterns.len()
    }

    /// Load default patterns
    pub fn load_defaults(&mut self) -> Result<()> {
        // This will be populated by the database seed
        Ok(())
    }
}

impl Default for PatternRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_pattern(id_str: &str) -> Pattern {
        Pattern {
            metadata: PatternMetadata {
                id: PatternId(id_str.to_string()),
                category: "test".to_string(),
                stack_effect: "( a -- b )".to_string(),
                code_template: ": NAME OP ;".to_string(),
                performance_class: PerformanceClass::Constant,
                test_cases: vec![],
                description: "Test pattern".to_string(),
                tags: vec!["test".to_string()],
                template_variables: vec![],
                created_at: "2025-01-01".to_string(),
                updated_at: "2025-01-01".to_string(),
            },
            usage_count: 0,
            success_rate: 1.0,
        }
    }

    #[test]
    fn test_registry_register_and_get() {
        let mut registry = PatternRegistry::new();
        let pattern = create_test_pattern("TEST_001");
        let id = pattern.metadata.id.clone();

        registry.register(pattern).unwrap();
        assert!(registry.get(&id).is_some());
    }

    #[test]
    fn test_registry_count() {
        let mut registry = PatternRegistry::new();
        assert_eq!(registry.count(), 0);

        registry.register(create_test_pattern("TEST_001")).unwrap();
        assert_eq!(registry.count(), 1);
    }
}
