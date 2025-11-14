//! Example: Using the Pattern Library System

use fastforth::patterns::{
    PatternDatabase, PatternQuery, PatternId, PatternTemplate,
    instantiate_pattern,
};
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Fast Forth Pattern Library Examples\n");

    // Example 1: Initialize and seed database
    println!("=== Example 1: Initialize Pattern Database ===");
    let mut db = PatternDatabase::open("examples/patterns.db")?;
    db.seed_defaults()?;
    println!("Database initialized with {} patterns\n", db.count()?);

    // Example 2: Query patterns by category
    println!("=== Example 2: Query Recursive Patterns ===");
    let query = PatternQuery {
        category: Some("recursive".to_string()),
        ..Default::default()
    };
    let recursive_patterns = db.query(&query)?;
    println!("Found {} recursive patterns:", recursive_patterns.len());
    for pattern in &recursive_patterns {
        println!("  - {}: {}", pattern.metadata.id, pattern.metadata.description);
    }
    println!();

    // Example 3: Get specific pattern
    println!("=== Example 3: Get Specific Pattern ===");
    let pattern_id = PatternId("DUP_TRANSFORM_001".to_string());
    if let Some(pattern) = db.get(&pattern_id)? {
        println!("Pattern: {}", pattern.metadata.id);
        println!("Stack Effect: {}", pattern.metadata.stack_effect);
        println!("Description: {}", pattern.metadata.description);
        println!("Template:\n{}\n", pattern.metadata.code_template);
    }

    // Example 4: Instantiate a pattern template
    println!("=== Example 4: Instantiate Pattern Template ===");
    let template = ": NAME ( n -- n² )\n  dup * ;";
    let mut substitutions = HashMap::new();
    substitutions.insert("NAME".to_string(), "square".to_string());

    let instantiated = instantiate_pattern(template, &substitutions)?;
    println!("Template:\n{}", template);
    println!("\nInstantiated:\n{}\n", instantiated);

    // Example 5: Query by performance class
    println!("=== Example 5: Query O(1) Patterns ===");
    let query = PatternQuery {
        performance_class: Some("O(1)".to_string()),
        limit: Some(5),
        ..Default::default()
    };
    let fast_patterns = db.query(&query)?;
    println!("Found {} O(1) patterns (showing first 5):", fast_patterns.len());
    for pattern in &fast_patterns {
        println!("  - {}: {}", pattern.metadata.id, pattern.metadata.stack_effect);
    }
    println!();

    // Example 6: Query by tags
    println!("=== Example 6: Query Patterns by Tags ===");
    let query = PatternQuery {
        tags: vec!["factorial".to_string()],
        ..Default::default()
    };
    let factorial_patterns = db.query(&query)?;
    println!("Found {} factorial patterns:", factorial_patterns.len());
    for pattern in &factorial_patterns {
        println!("  - {}: {} ({})",
            pattern.metadata.id,
            pattern.metadata.description,
            pattern.metadata.performance_class
        );
    }
    println!();

    // Example 7: Complex recursive pattern instantiation
    println!("=== Example 7: Instantiate Recursive Pattern ===");
    if let Some(pattern) = db.get(&PatternId("RECURSIVE_001".to_string()))? {
        println!("Using pattern: {}", pattern.metadata.id);
        println!("Template variables: {:?}", pattern.metadata.template_variables);
        println!("\nOriginal template:\n{}\n", pattern.metadata.code_template);

        let mut values = HashMap::new();
        values.insert("NAME".to_string(), "factorial".to_string());

        // Note: This is a simplified example - the template already has the logic
        println!("Instantiated code would include pattern metadata:");
        println!("\\ PATTERN: RECURSIVE_001");
        println!("{}", pattern.metadata.code_template);
    }

    // Example 8: Export patterns
    println!("\n=== Example 8: Export Patterns ===");
    let json = db.export_json()?;
    println!("Exported {} bytes of JSON data", json.len());
    println!("(First 200 characters):");
    println!("{}\n", &json[..200.min(json.len())]);

    Ok(())
}
