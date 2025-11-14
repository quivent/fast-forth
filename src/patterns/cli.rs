//! CLI commands for pattern management

use super::{PatternDatabase, PatternQuery, PatternId, Result};
use clap::{Parser, Subcommand};
use serde_json;

/// Pattern CLI commands
#[derive(Debug, Parser)]
#[command(name = "fastforth patterns")]
#[command(about = "Manage Fast Forth pattern library")]
pub struct PatternCli {
    #[command(subcommand)]
    pub command: PatternCommand,
}

#[derive(Debug, Subcommand)]
pub enum PatternCommand {
    /// List all patterns
    List {
        /// Output format (table, json, yaml)
        #[arg(long, default_value = "table")]
        format: String,

        /// Filter by category
        #[arg(long)]
        category: Option<String>,

        /// Filter by performance class
        #[arg(long)]
        perf: Option<String>,

        /// Limit results
        #[arg(long)]
        limit: Option<usize>,
    },

    /// Show pattern details
    Show {
        /// Pattern ID
        id: String,

        /// Output format (pretty, json)
        #[arg(long, default_value = "pretty")]
        format: String,
    },

    /// Search patterns
    Search {
        /// Search query (searches in description, tags, category)
        query: String,

        /// Output format
        #[arg(long, default_value = "table")]
        format: String,
    },

    /// Query patterns with filters
    Query {
        /// Filter by category
        #[arg(long)]
        category: Option<String>,

        /// Filter by stack effect
        #[arg(long)]
        effect: Option<String>,

        /// Filter by performance class
        #[arg(long)]
        perf: Option<String>,

        /// Filter by tags (comma-separated)
        #[arg(long)]
        tags: Option<String>,

        /// Output format
        #[arg(long, default_value = "json")]
        format: String,

        /// Limit results
        #[arg(long)]
        limit: Option<usize>,
    },

    /// Initialize pattern database
    Init {
        /// Database path
        #[arg(long, default_value = "patterns.db")]
        db: String,

        /// Seed with default patterns
        #[arg(long)]
        seed: bool,
    },

    /// Export patterns to JSON
    Export {
        /// Output file
        #[arg(long)]
        output: String,
    },

    /// Import patterns from JSON
    Import {
        /// Input file
        #[arg(long)]
        input: String,
    },

    /// Show pattern statistics
    Stats {
        /// Output format
        #[arg(long, default_value = "table")]
        format: String,
    },
}

/// Execute pattern CLI command
pub fn execute_pattern_command(cmd: PatternCommand, db: &mut PatternDatabase) -> Result<()> {
    match cmd {
        PatternCommand::List { format, category, perf, limit } => {
            let query = PatternQuery {
                category,
                performance_class: perf,
                limit,
                ..Default::default()
            };

            let patterns = db.query(&query)?;

            match format.as_str() {
                "json" => {
                    let json = serde_json::to_string_pretty(&patterns)?;
                    println!("{}", json);
                }
                "table" => {
                    print_patterns_table(&patterns);
                }
                _ => {
                    eprintln!("Unknown format: {}", format);
                }
            }
        }

        PatternCommand::Show { id, format } => {
            let pattern_id = PatternId(id);
            match db.get(&pattern_id)? {
                Some(pattern) => {
                    match format.as_str() {
                        "json" => {
                            let json = serde_json::to_string_pretty(&pattern)?;
                            println!("{}", json);
                        }
                        "pretty" => {
                            print_pattern_details(&pattern);
                        }
                        _ => {
                            eprintln!("Unknown format: {}", format);
                        }
                    }
                }
                None => {
                    eprintln!("Pattern not found: {}", pattern_id);
                }
            }
        }

        PatternCommand::Query { category, effect, perf, tags, format, limit } => {
            let tag_vec = tags.map(|t| {
                t.split(',').map(|s| s.trim().to_string()).collect()
            }).unwrap_or_default();

            let query = PatternQuery {
                category,
                stack_effect: effect,
                performance_class: perf,
                tags: tag_vec,
                limit,
                ..Default::default()
            };

            let patterns = db.query(&query)?;

            match format.as_str() {
                "json" => {
                    let json = serde_json::to_string_pretty(&patterns)?;
                    println!("{}", json);
                }
                "table" => {
                    print_patterns_table(&patterns);
                }
                _ => {
                    eprintln!("Unknown format: {}", format);
                }
            }
        }

        PatternCommand::Init { db: db_path, seed } => {
            let mut new_db = PatternDatabase::open(&db_path)?;
            new_db.init_schema()?;

            if seed {
                new_db.seed_defaults()?;
                println!("Database initialized and seeded at: {}", db_path);
                println!("Total patterns: {}", new_db.count()?);
            } else {
                println!("Database initialized at: {}", db_path);
            }
        }

        PatternCommand::Export { output } => {
            let json = db.export_json()?;
            std::fs::write(&output, json)?;
            println!("Patterns exported to: {}", output);
        }

        PatternCommand::Import { input } => {
            let json = std::fs::read_to_string(&input)?;
            let count = db.import_json(&json)?;
            println!("Imported {} patterns from: {}", count, input);
        }

        PatternCommand::Stats { format } => {
            let stats = collect_stats(db)?;

            match format.as_str() {
                "json" => {
                    let json = serde_json::to_string_pretty(&stats)?;
                    println!("{}", json);
                }
                "table" => {
                    print_stats_table(&stats);
                }
                _ => {
                    eprintln!("Unknown format: {}", format);
                }
            }
        }

        PatternCommand::Search { query, format } => {
            let all_patterns = db.list_all()?;
            let results: Vec<_> = all_patterns.into_iter()
                .filter(|p| {
                    p.metadata.description.to_lowercase().contains(&query.to_lowercase()) ||
                    p.metadata.tags.iter().any(|t| t.to_lowercase().contains(&query.to_lowercase())) ||
                    p.metadata.category.to_lowercase().contains(&query.to_lowercase())
                })
                .collect();

            match format.as_str() {
                "json" => {
                    let json = serde_json::to_string_pretty(&results)?;
                    println!("{}", json);
                }
                "table" => {
                    print_patterns_table(&results);
                }
                _ => {
                    eprintln!("Unknown format: {}", format);
                }
            }
        }
    }

    Ok(())
}

fn print_patterns_table(patterns: &[super::Pattern]) {
    println!("{:<25} {:<20} {:<25} {:<10}", "ID", "Category", "Stack Effect", "Performance");
    println!("{}", "-".repeat(80));

    for pattern in patterns {
        println!(
            "{:<25} {:<20} {:<25} {:<10}",
            pattern.metadata.id,
            pattern.metadata.category,
            pattern.metadata.stack_effect,
            pattern.metadata.performance_class
        );
    }

    println!("\nTotal: {} patterns", patterns.len());
}

fn print_pattern_details(pattern: &super::Pattern) {
    println!("Pattern ID: {}", pattern.metadata.id);
    println!("Category: {}", pattern.metadata.category);
    println!("Stack Effect: {}", pattern.metadata.stack_effect);
    println!("Performance: {}", pattern.metadata.performance_class);
    println!("Description: {}", pattern.metadata.description);
    println!("Tags: {}", pattern.metadata.tags.join(", "));
    println!("\nCode Template:");
    println!("{}", pattern.metadata.code_template);

    if !pattern.metadata.template_variables.is_empty() {
        println!("\nTemplate Variables:");
        for var in &pattern.metadata.template_variables {
            println!("  - {}", var);
        }
    }

    if !pattern.metadata.test_cases.is_empty() {
        println!("\nTest Cases:");
        for (i, test) in pattern.metadata.test_cases.iter().enumerate() {
            print!("  {}: {:?} -> {:?}", i + 1, test.input, test.output);
            if let Some(desc) = &test.description {
                print!(" ({})", desc);
            }
            println!();
        }
    }

    println!("\nUsage Count: {}", pattern.usage_count);
    println!("Success Rate: {:.1}%", pattern.success_rate * 100.0);
}

use serde::Serialize;

#[derive(Debug, Serialize)]
struct PatternStats {
    total_patterns: usize,
    categories: std::collections::HashMap<String, usize>,
    performance_classes: std::collections::HashMap<String, usize>,
    avg_usage_count: f64,
    avg_success_rate: f64,
}

fn collect_stats(db: &PatternDatabase) -> Result<PatternStats> {
    let patterns = db.list_all()?;
    let total = patterns.len();

    let mut categories = std::collections::HashMap::new();
    let mut performance_classes = std::collections::HashMap::new();
    let mut total_usage = 0u64;
    let mut total_success = 0.0;

    for pattern in &patterns {
        *categories.entry(pattern.metadata.category.clone()).or_insert(0) += 1;
        *performance_classes.entry(pattern.metadata.performance_class.to_string()).or_insert(0) += 1;
        total_usage += pattern.usage_count;
        total_success += pattern.success_rate;
    }

    Ok(PatternStats {
        total_patterns: total,
        categories,
        performance_classes,
        avg_usage_count: if total > 0 { total_usage as f64 / total as f64 } else { 0.0 },
        avg_success_rate: if total > 0 { total_success / total as f64 } else { 0.0 },
    })
}

fn print_stats_table(stats: &PatternStats) {
    println!("Pattern Library Statistics");
    println!("{}", "=".repeat(50));
    println!("Total Patterns: {}", stats.total_patterns);
    println!("Average Usage Count: {:.2}", stats.avg_usage_count);
    println!("Average Success Rate: {:.1}%", stats.avg_success_rate * 100.0);

    println!("\nPatterns by Category:");
    for (category, count) in &stats.categories {
        println!("  {:<30} {}", category, count);
    }

    println!("\nPatterns by Performance Class:");
    for (perf, count) in &stats.performance_classes {
        println!("  {:<30} {}", perf, count);
    }
}
