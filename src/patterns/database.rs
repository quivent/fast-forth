//! SQLite database for persistent pattern storage

use super::{PatternId, PatternMetadata, Pattern, Result, PatternError, PerformanceClass, TestCase};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Pattern database query parameters
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PatternQuery {
    pub category: Option<String>,
    pub stack_effect: Option<String>,
    pub performance_class: Option<String>,
    pub tags: Vec<String>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

/// SQLite-based pattern database
pub struct PatternDatabase {
    db_path: std::path::PathBuf,
    // In a real implementation, this would be a rusqlite::Connection
    // For now, we'll use a HashMap for demonstration
    patterns: std::collections::HashMap<PatternId, Pattern>,
}

impl PatternDatabase {
    /// Create or open a pattern database
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let db_path = path.as_ref().to_path_buf();

        // In a real implementation, we would:
        // 1. Open SQLite connection
        // 2. Run schema migrations
        // 3. Create indexes

        Ok(Self {
            db_path,
            patterns: std::collections::HashMap::new(),
        })
    }

    /// Initialize database schema
    pub fn init_schema(&self) -> Result<()> {
        // In a real implementation, this would execute:
        // CREATE TABLE patterns (...)
        // See seed.sql for the schema
        Ok(())
    }

    /// Insert a pattern
    pub fn insert(&mut self, pattern: Pattern) -> Result<()> {
        let id = pattern.metadata.id.clone();
        self.patterns.insert(id, pattern);
        Ok(())
    }

    /// Get a pattern by ID
    pub fn get(&self, id: &PatternId) -> Result<Option<Pattern>> {
        Ok(self.patterns.get(id).cloned())
    }

    /// Query patterns
    pub fn query(&self, query: &PatternQuery) -> Result<Vec<Pattern>> {
        let mut results: Vec<_> = self.patterns.values().cloned().collect();

        // Filter by category
        if let Some(ref category) = query.category {
            results.retain(|p| p.metadata.category == *category);
        }

        // Filter by stack effect
        if let Some(ref effect) = query.stack_effect {
            results.retain(|p| p.metadata.stack_effect == *effect);
        }

        // Filter by performance class
        if let Some(ref perf) = query.performance_class {
            results.retain(|p| p.metadata.performance_class.to_string() == *perf);
        }

        // Filter by tags
        if !query.tags.is_empty() {
            results.retain(|p| {
                query.tags.iter().any(|t| p.metadata.tags.contains(t))
            });
        }

        // Apply offset and limit
        if let Some(offset) = query.offset {
            results = results.into_iter().skip(offset).collect();
        }
        if let Some(limit) = query.limit {
            results.truncate(limit);
        }

        Ok(results)
    }

    /// List all patterns
    pub fn list_all(&self) -> Result<Vec<Pattern>> {
        Ok(self.patterns.values().cloned().collect())
    }

    /// Count patterns
    pub fn count(&self) -> Result<usize> {
        Ok(self.patterns.len())
    }

    /// Seed database with default patterns
    pub fn seed_defaults(&mut self) -> Result<()> {
        let patterns = create_default_patterns();
        for pattern in patterns {
            self.insert(pattern)?;
        }
        Ok(())
    }

    /// Export patterns to JSON
    pub fn export_json(&self) -> Result<String> {
        let patterns: Vec<_> = self.patterns.values().collect();
        serde_json::to_string_pretty(&patterns)
            .map_err(|e| PatternError::JsonError(e))
    }

    /// Import patterns from JSON
    pub fn import_json(&mut self, json: &str) -> Result<usize> {
        let patterns: Vec<Pattern> = serde_json::from_str(json)
            .map_err(|e| PatternError::JsonError(e))?;

        let count = patterns.len();
        for pattern in patterns {
            self.insert(pattern)?;
        }

        Ok(count)
    }
}

/// Create default pattern library (20+ patterns)
fn create_default_patterns() -> Vec<Pattern> {
    vec![
        // DUP_TRANSFORM patterns
        create_pattern(
            "DUP_TRANSFORM_001",
            "dup_transform",
            "( n -- n² )",
            ": NAME ( n -- n² )\n  dup * ;",
            PerformanceClass::Constant,
            "Square a number using dup and multiply",
            vec!["arithmetic", "dup", "transform"],
            vec![
                TestCase { input: vec![5], output: vec![25], description: Some("5² = 25".to_string()) },
                TestCase { input: vec![0], output: vec![0], description: Some("0² = 0".to_string()) },
                TestCase { input: vec![-3], output: vec![9], description: Some("(-3)² = 9".to_string()) },
            ],
        ),
        create_pattern(
            "DUP_TRANSFORM_002",
            "dup_transform",
            "( n -- n n+1 )",
            ": NAME ( n -- n n+1 )\n  dup 1+ ;",
            PerformanceClass::Constant,
            "Duplicate and increment",
            vec!["arithmetic", "dup"],
            vec![
                TestCase { input: vec![5], output: vec![5, 6], description: None },
            ],
        ),

        // CONDITIONAL patterns
        create_pattern(
            "CONDITIONAL_001",
            "conditional",
            "( n -- |n| )",
            ": NAME ( n -- |n| )\n  dup 0 < if negate then ;",
            PerformanceClass::Constant,
            "Absolute value using conditional",
            vec!["arithmetic", "conditional", "abs"],
            vec![
                TestCase { input: vec![5], output: vec![5], description: None },
                TestCase { input: vec![-5], output: vec![5], description: None },
                TestCase { input: vec![0], output: vec![0], description: None },
            ],
        ),
        create_pattern(
            "CONDITIONAL_002",
            "conditional",
            "( a b -- max )",
            ": NAME ( a b -- max )\n  2dup < if swap then drop ;",
            PerformanceClass::Constant,
            "Maximum of two numbers",
            vec!["arithmetic", "conditional", "max"],
            vec![
                TestCase { input: vec![3, 7], output: vec![7], description: None },
                TestCase { input: vec![7, 3], output: vec![7], description: None },
            ],
        ),

        // ACCUMULATOR_LOOP patterns
        create_pattern(
            "ACCUMULATOR_LOOP_001",
            "accumulator_loop",
            "( n -- sum )",
            ": NAME ( n -- sum )\n  0 swap 1+ 1 do i + loop ;",
            PerformanceClass::Linear,
            "Sum from 1 to n",
            vec!["loop", "accumulator", "sum"],
            vec![
                TestCase { input: vec![5], output: vec![15], description: Some("1+2+3+4+5=15".to_string()) },
                TestCase { input: vec![0], output: vec![0], description: None },
            ],
        ),
        create_pattern(
            "ACCUMULATOR_LOOP_002",
            "accumulator_loop",
            "( n -- product )",
            ": NAME ( n -- n! )\n  1 swap 1+ 1 do i * loop ;",
            PerformanceClass::Linear,
            "Factorial using loop",
            vec!["loop", "accumulator", "factorial"],
            vec![
                TestCase { input: vec![5], output: vec![120], description: None },
                TestCase { input: vec![0], output: vec![1], description: None },
            ],
        ),

        // RECURSIVE patterns
        create_pattern(
            "RECURSIVE_001",
            "recursive",
            "( n -- n! )",
            ": NAME ( n -- n! )\n  dup 2 < if drop 1 else dup 1- recurse * then ;",
            PerformanceClass::Linear,
            "Factorial using recursion",
            vec!["recursion", "factorial", "base-case"],
            vec![
                TestCase { input: vec![5], output: vec![120], description: None },
                TestCase { input: vec![0], output: vec![1], description: None },
                TestCase { input: vec![1], output: vec![1], description: None },
            ],
        ),
        create_pattern(
            "RECURSIVE_002",
            "recursive",
            "( n -- fib(n) )",
            ": NAME ( n -- fib )\n  dup 2 < if else dup 1- recurse swap 2 - recurse + then ;",
            PerformanceClass::Exponential,
            "Fibonacci using recursion (inefficient)",
            vec!["recursion", "fibonacci"],
            vec![
                TestCase { input: vec![0], output: vec![0], description: None },
                TestCase { input: vec![1], output: vec![1], description: None },
                TestCase { input: vec![6], output: vec![8], description: None },
            ],
        ),

        // TAIL_RECURSIVE patterns
        create_pattern(
            "TAIL_RECURSIVE_001",
            "tail_recursive",
            "( n acc -- n! )",
            ": NAME ( n acc -- result )\n  over 1 <= if nip else over * swap 1- swap recurse then ;",
            PerformanceClass::Linear,
            "Tail-recursive factorial",
            vec!["recursion", "tail-call", "factorial", "optimized"],
            vec![
                TestCase { input: vec![5, 1], output: vec![120], description: None },
            ],
        ),

        // BINARY_OP patterns
        create_pattern(
            "BINARY_OP_001",
            "binary_op",
            "( a b -- a+b )",
            ": NAME ( a b -- c )\n  + ;",
            PerformanceClass::Constant,
            "Simple binary operation template",
            vec!["arithmetic", "binary"],
            vec![
                TestCase { input: vec![3, 4], output: vec![7], description: None },
            ],
        ),
        create_pattern(
            "BINARY_OP_002",
            "binary_op",
            "( a b -- avg )",
            ": NAME ( a b -- avg )\n  + 2 / ;",
            PerformanceClass::Constant,
            "Average of two numbers",
            vec!["arithmetic", "average"],
            vec![
                TestCase { input: vec![4, 6], output: vec![5], description: None },
            ],
        ),

        // UNARY_OP patterns
        create_pattern(
            "UNARY_OP_001",
            "unary_op",
            "( n -- -n )",
            ": NAME ( n -- -n )\n  negate ;",
            PerformanceClass::Constant,
            "Negate a number",
            vec!["arithmetic", "unary"],
            vec![
                TestCase { input: vec![5], output: vec![-5], description: None },
            ],
        ),
        create_pattern(
            "UNARY_OP_002",
            "unary_op",
            "( n -- n*2 )",
            ": NAME ( n -- n*2 )\n  2 * ;",
            PerformanceClass::Constant,
            "Double a number",
            vec!["arithmetic", "double"],
            vec![
                TestCase { input: vec![5], output: vec![10], description: None },
            ],
        ),

        // STACK_MANIP patterns
        create_pattern(
            "STACK_MANIP_001",
            "stack_manipulation",
            "( a b c -- c b a )",
            ": NAME ( a b c -- c b a )\n  rot rot ;",
            PerformanceClass::Constant,
            "Reverse top 3 stack items",
            vec!["stack", "manipulation", "reverse"],
            vec![
                TestCase { input: vec![1, 2, 3], output: vec![3, 2, 1], description: None },
            ],
        ),
        create_pattern(
            "STACK_MANIP_002",
            "stack_manipulation",
            "( a b -- b a b )",
            ": NAME ( a b -- b a b )\n  tuck ;",
            PerformanceClass::Constant,
            "Tuck second item over top",
            vec!["stack", "manipulation", "tuck"],
            vec![
                TestCase { input: vec![1, 2], output: vec![2, 1, 2], description: None },
            ],
        ),

        // CONTROL_FLOW patterns
        create_pattern(
            "CONTROL_FLOW_001",
            "control_flow",
            "( n -- )",
            ": NAME ( n -- )\n  begin dup 0> while dup . 1- repeat drop ;",
            PerformanceClass::Linear,
            "Count down from n to 1",
            vec!["loop", "countdown", "control-flow"],
            vec![],
        ),
        create_pattern(
            "CONTROL_FLOW_002",
            "control_flow",
            "( n -- )",
            ": NAME ( n -- )\n  0 do i . loop ;",
            PerformanceClass::Linear,
            "Count from 0 to n-1",
            vec!["loop", "count", "control-flow"],
            vec![],
        ),

        // OPTIMIZATION patterns
        create_pattern(
            "OPTIMIZATION_001",
            "optimization",
            "( n -- n*8 )",
            ": NAME ( n -- n*8 )\n  3 lshift ;",
            PerformanceClass::Constant,
            "Multiply by 8 using bit shift",
            vec!["optimization", "bitwise", "multiply"],
            vec![
                TestCase { input: vec![5], output: vec![40], description: None },
            ],
        ),
        create_pattern(
            "OPTIMIZATION_002",
            "optimization",
            "( n -- bool )",
            ": NAME ( n -- bool )\n  1 and 0= ;",
            PerformanceClass::Constant,
            "Check if even using bitwise and",
            vec!["optimization", "bitwise", "even"],
            vec![
                TestCase { input: vec![4], output: vec![1], description: None },
                TestCase { input: vec![5], output: vec![0], description: None },
            ],
        ),

        // DATA_STRUCTURE patterns
        create_pattern(
            "DATA_STRUCTURE_001",
            "data_structure",
            "( addr n -- )",
            ": NAME ( addr n -- )\n  swap ! ;",
            PerformanceClass::Constant,
            "Store value at address",
            vec!["memory", "store", "data-structure"],
            vec![],
        ),
        create_pattern(
            "DATA_STRUCTURE_002",
            "data_structure",
            "( addr -- n )",
            ": NAME ( addr -- n )\n  @ ;",
            PerformanceClass::Constant,
            "Fetch value from address",
            vec!["memory", "fetch", "data-structure"],
            vec![],
        ),
    ]
}

fn create_pattern(
    id: &str,
    category: &str,
    stack_effect: &str,
    code_template: &str,
    performance_class: PerformanceClass,
    description: &str,
    tags: Vec<&str>,
    test_cases: Vec<TestCase>,
) -> Pattern {
    Pattern {
        metadata: PatternMetadata {
            id: PatternId(id.to_string()),
            category: category.to_string(),
            stack_effect: stack_effect.to_string(),
            code_template: code_template.to_string(),
            performance_class,
            test_cases,
            description: description.to_string(),
            tags: tags.into_iter().map(|s| s.to_string()).collect(),
            template_variables: extract_template_variables(code_template),
            created_at: "2025-11-14".to_string(),
            updated_at: "2025-11-14".to_string(),
        },
        usage_count: 0,
        success_rate: 1.0,
    }
}

fn extract_template_variables(template: &str) -> Vec<String> {
    let mut vars = Vec::new();
    if template.contains("NAME") {
        vars.push("NAME".to_string());
    }
    if template.contains("OP") {
        vars.push("OP".to_string());
    }
    if template.contains("BASE_CASE") {
        vars.push("BASE_CASE".to_string());
    }
    if template.contains("BASE_VALUE") {
        vars.push("BASE_VALUE".to_string());
    }
    if template.contains("RECURSIVE_STEP") {
        vars.push("RECURSIVE_STEP".to_string());
    }
    vars
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_creation() {
        let db = PatternDatabase::open("test.db").unwrap();
        assert_eq!(db.count().unwrap(), 0);
    }

    #[test]
    fn test_seed_defaults() {
        let mut db = PatternDatabase::open("test.db").unwrap();
        db.seed_defaults().unwrap();
        assert!(db.count().unwrap() >= 20);
    }

    #[test]
    fn test_query_by_category() {
        let mut db = PatternDatabase::open("test.db").unwrap();
        db.seed_defaults().unwrap();

        let query = PatternQuery {
            category: Some("recursive".to_string()),
            ..Default::default()
        };

        let results = db.query(&query).unwrap();
        assert!(results.len() > 0);
    }
}
