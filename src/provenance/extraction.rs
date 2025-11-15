//! Provenance metadata extraction
//!
//! Extract provenance metadata from Forth source code and compiled binaries

use crate::error::{CompileError, Result};
use crate::provenance::metadata::ProvenanceMetadata;
use std::collections::HashMap;
use std::path::Path;

/// Extract provenance metadata from Forth source code
pub fn extract_provenance(source: &str) -> Result<HashMap<String, ProvenanceMetadata>> {
    let mut metadata_map = HashMap::new();
    let mut current_word = None;
    let mut current_metadata = None;

    for line in source.lines() {
        let trimmed = line.trim();

        // Check for word definition start
        if trimmed.starts_with(": ") {
            // If we have a previous word without metadata, save it
            if let Some(word) = current_word.take() {
                if let Some(metadata) = current_metadata.take() {
                    metadata_map.insert(word, metadata);
                }
            }

            // Extract word name and keep any accumulated metadata
            if let Some(word_name) = trimmed.split_whitespace().nth(1) {
                current_word = Some(word_name.to_string());
                // Don't reset current_metadata - keep any metadata from preceding comments
            }
        }

        // Parse metadata comments
        if trimmed.starts_with("\\ GENERATED_BY: ") {
            let generated_by = trimmed.trim_start_matches("\\ GENERATED_BY: ").to_string();
            current_metadata = Some(ProvenanceMetadata::new(generated_by));
        } else if let Some(ref mut metadata) = current_metadata {
            if trimmed.starts_with("\\ PATTERN_ID: ") {
                let pattern_id = trimmed.trim_start_matches("\\ PATTERN_ID: ").to_string();
                metadata.pattern_id = Some(pattern_id);
            } else if trimmed.starts_with("\\ TIMESTAMP: ") {
                let timestamp = trimmed.trim_start_matches("\\ TIMESTAMP: ").to_string();
                metadata.timestamp = timestamp;
            } else if trimmed.starts_with("\\ SPEC_HASH: ") {
                let spec_hash = trimmed.trim_start_matches("\\ SPEC_HASH: ").to_string();
                metadata.spec_hash = Some(spec_hash);
            } else if trimmed.starts_with("\\ VERIFIED: ") {
                // Parse verification status
                let status_str = trimmed.trim_start_matches("\\ VERIFIED: ");
                metadata.verification = parse_verification_status(status_str);
            } else if trimmed.starts_with("\\ OPTIMIZATION_LEVEL: ") {
                let level = trimmed.trim_start_matches("\\ OPTIMIZATION_LEVEL: ").to_string();
                metadata.context.optimization_level = Some(level);
            } else if trimmed.starts_with("\\ PERFORMANCE_TARGET: ") {
                let target = trimmed.trim_start_matches("\\ PERFORMANCE_TARGET: ").to_string();
                metadata.context.performance_target = Some(target);
            }
        }

        // Check for word definition end
        if trimmed.ends_with(";") && current_word.is_some() {
            if let (Some(word), Some(metadata)) = (current_word.take(), current_metadata.take()) {
                metadata_map.insert(word, metadata);
            }
        }
    }

    // Handle any remaining metadata
    if let (Some(word), Some(metadata)) = (current_word, current_metadata) {
        metadata_map.insert(word, metadata);
    }

    Ok(metadata_map)
}

/// Extract provenance metadata from a compiled binary
pub fn extract_from_binary<P: AsRef<Path>>(binary_path: P) -> Result<HashMap<String, ProvenanceMetadata>> {
    // TODO: Implement binary metadata extraction from debug symbols
    // For now, return an error indicating this is not yet implemented
    Err(CompileError::InternalError(
        "Binary provenance extraction not yet implemented. Use extract_provenance() for source code.".to_string()
    ))
}

/// Parse verification status from string
fn parse_verification_status(status_str: &str) -> crate::provenance::metadata::VerificationStatus {
    use crate::provenance::metadata::VerificationStatus;

    let mut status = VerificationStatus::new();

    // Parse fields like "stack_balanced=true, tests_passed=3/3, type_checked=true, compiled=true"
    for part in status_str.split(", ") {
        let mut kv = part.split('=');
        if let (Some(key), Some(value)) = (kv.next(), kv.next()) {
            match key.trim() {
                "stack_balanced" => {
                    status.stack_balanced = value.trim() == "true";
                }
                "tests_passed" => {
                    if let Some((passed, total)) = parse_test_results(value.trim()) {
                        status.tests_passed = passed;
                        status.tests_total = total;
                    }
                }
                "type_checked" => {
                    status.type_checked = value.trim() == "true";
                }
                "compiled" => {
                    status.compiled = value.trim() == "true";
                }
                _ => {}
            }
        }
    }

    status
}

/// Parse test results from "3/3" format
fn parse_test_results(s: &str) -> Option<(usize, usize)> {
    let mut parts = s.split('/');
    let passed = parts.next()?.parse().ok()?;
    let total = parts.next()?.parse().ok()?;
    Some((passed, total))
}

/// Extract provenance from multiple source files
pub fn extract_from_files<P: AsRef<Path>>(paths: &[P]) -> Result<HashMap<String, ProvenanceMetadata>> {
    let mut all_metadata = HashMap::new();

    for path in paths {
        let source = std::fs::read_to_string(path)
            .map_err(|e| CompileError::IoError(path.as_ref().to_path_buf(), e))?;

        let file_metadata = extract_provenance(&source)?;
        all_metadata.extend(file_metadata);
    }

    Ok(all_metadata)
}

/// Provenance extractor with filtering options
pub struct ProvenanceExtractor {
    filter_agent: Option<String>,
    filter_pattern: Option<String>,
    filter_verified_only: bool,
}

impl ProvenanceExtractor {
    /// Create a new extractor
    pub fn new() -> Self {
        Self {
            filter_agent: None,
            filter_pattern: None,
            filter_verified_only: false,
        }
    }

    /// Filter by agent
    pub fn with_agent_filter(mut self, agent: String) -> Self {
        self.filter_agent = Some(agent);
        self
    }

    /// Filter by pattern
    pub fn with_pattern_filter(mut self, pattern: String) -> Self {
        self.filter_pattern = Some(pattern);
        self
    }

    /// Filter verified only
    pub fn verified_only(mut self) -> Self {
        self.filter_verified_only = true;
        self
    }

    /// Extract and filter metadata
    pub fn extract(&self, source: &str) -> Result<HashMap<String, ProvenanceMetadata>> {
        let mut metadata = extract_provenance(source)?;

        // Apply filters
        metadata.retain(|_, meta| {
            if let Some(ref agent) = self.filter_agent {
                if &meta.generated_by != agent {
                    return false;
                }
            }

            if let Some(ref pattern) = self.filter_pattern {
                if meta.pattern_id.as_ref() != Some(pattern) {
                    return false;
                }
            }

            if self.filter_verified_only && !meta.verification.is_verified() {
                return false;
            }

            true
        });

        Ok(metadata)
    }
}

impl Default for ProvenanceExtractor {
    fn default() -> Self {
        Self::new()
    }
}

/// Generate a provenance report from extracted metadata
pub fn generate_report(metadata: &HashMap<String, ProvenanceMetadata>) -> String {
    let mut report = String::new();
    report.push_str("Provenance Report\n");
    report.push_str("=================\n\n");

    if metadata.is_empty() {
        report.push_str("No provenance metadata found.\n");
        return report;
    }

    // Statistics
    let total = metadata.len();
    let verified = metadata.values().filter(|m| m.verification.is_verified()).count();
    let agents: std::collections::HashSet<_> = metadata.values()
        .map(|m| &m.generated_by)
        .collect();

    report.push_str(&format!("Total Definitions: {}\n", total));
    report.push_str(&format!("Verified: {} ({:.1}%)\n", verified, (verified as f64 / total as f64) * 100.0));
    report.push_str(&format!("Unique Agents: {}\n", agents.len()));
    report.push_str("\n");

    // Detailed list
    report.push_str("Detailed Metadata:\n");
    report.push_str("-----------------\n\n");

    for (word, meta) in metadata {
        report.push_str(&format!("Word: {}\n", word));
        report.push_str(&format!("  Generated By: {}\n", meta.generated_by));
        if let Some(pattern) = &meta.pattern_id {
            report.push_str(&format!("  Pattern: {}\n", pattern));
        }
        report.push_str(&format!("  Timestamp: {}\n", meta.timestamp));
        report.push_str(&format!("  Verified: {}\n", meta.verification.is_verified()));
        if let Some(spec) = &meta.spec_hash {
            report.push_str(&format!("  Spec Hash: {}\n", spec));
        }
        report.push_str("\n");
    }

    report
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_simple_metadata() {
        let source = r#"
\ GENERATED_BY: claude-sonnet-4
\ PATTERN_ID: RECURSIVE_004
\ TIMESTAMP: 2025-01-15T10:23:45Z
: factorial ( n -- n! )
  dup 2 < if drop 1 else dup 1- recurse * then ;
"#;

        let metadata = extract_provenance(source).unwrap();
        assert_eq!(metadata.len(), 1);
        assert!(metadata.contains_key("factorial"));

        let meta = &metadata["factorial"];
        assert_eq!(meta.generated_by, "claude-sonnet-4");
        assert_eq!(meta.pattern_id, Some("RECURSIVE_004".to_string()));
    }

    #[test]
    fn test_extract_multiple_words() {
        let source = r#"
\ GENERATED_BY: claude-sonnet-4
: double ( n -- 2n )
  2 * ;

\ GENERATED_BY: other-agent
\ PATTERN_ID: SIMPLE_001
: triple ( n -- 3n )
  3 * ;
"#;

        let metadata = extract_provenance(source).unwrap();
        assert_eq!(metadata.len(), 2);
        assert!(metadata.contains_key("double"));
        assert!(metadata.contains_key("triple"));
    }

    #[test]
    fn test_extractor_with_filters() {
        let source = r#"
\ GENERATED_BY: claude-sonnet-4
: word1 ;

\ GENERATED_BY: other-agent
: word2 ;
"#;

        let extractor = ProvenanceExtractor::new()
            .with_agent_filter("claude-sonnet-4".to_string());

        let metadata = extractor.extract(source).unwrap();
        assert_eq!(metadata.len(), 1);
        assert!(metadata.contains_key("word1"));
    }

    #[test]
    fn test_parse_test_results() {
        assert_eq!(parse_test_results("3/5"), Some((3, 5)));
        assert_eq!(parse_test_results("10/10"), Some((10, 10)));
        assert_eq!(parse_test_results("0/3"), Some((0, 3)));
        assert_eq!(parse_test_results("invalid"), None);
    }

    #[test]
    fn test_generate_report() {
        let mut metadata = HashMap::new();
        metadata.insert(
            "factorial".to_string(),
            ProvenanceMetadata::new("claude-sonnet-4".to_string()),
        );

        let report = generate_report(&metadata);
        assert!(report.contains("Provenance Report"));
        assert!(report.contains("Word: factorial"));
    }
}
