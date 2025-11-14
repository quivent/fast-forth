//! Provenance metadata embedding
//!
//! Embed provenance metadata into generated Forth code and compiled binaries

use crate::error::{CompileError, Result};
use crate::provenance::metadata::ProvenanceMetadata;
use std::path::Path;

/// Embed provenance metadata into Forth source code
pub fn embed_provenance(word_name: &str, word_body: &str, metadata: &ProvenanceMetadata) -> String {
    let mut output = String::new();

    // Add metadata as Forth comments
    output.push_str(&metadata.to_forth_comment());

    // Add the word definition
    // Extract stack effect comment if present
    let lines: Vec<&str> = word_body.lines().collect();
    if let Some(first_line) = lines.first() {
        if first_line.trim().starts_with(": ") {
            output.push_str(first_line);
            output.push('\n');

            // Add remaining lines
            for line in &lines[1..] {
                output.push_str(line);
                output.push('\n');
            }
        } else {
            output.push_str(word_body);
        }
    } else {
        output.push_str(word_body);
    }

    output
}

/// Embed provenance into multiple word definitions
pub fn embed_multiple(definitions: &[(String, String, ProvenanceMetadata)]) -> String {
    let mut output = String::new();

    for (word_name, word_body, metadata) in definitions {
        output.push_str(&embed_provenance(word_name, word_body, metadata));
        output.push('\n');
    }

    output
}

/// Embed provenance metadata into compiled binary debug symbols
pub fn embed_in_binary<P: AsRef<Path>>(
    binary_path: P,
    metadata: &ProvenanceMetadata,
) -> Result<()> {
    // TODO: Implement binary metadata embedding in debug symbols
    // This would involve:
    // 1. Creating custom debug sections
    // 2. Embedding JSON metadata
    // 3. Making it extractable by debuggers
    Err(CompileError::InternalError(
        "Binary provenance embedding not yet implemented. Use embed_provenance() for source code.".to_string()
    ))
}

/// Provenance embedder with customization options
pub struct ProvenanceEmbedder {
    include_timestamp: bool,
    include_pattern: bool,
    include_verification: bool,
    include_context: bool,
    compact_format: bool,
}

impl ProvenanceEmbedder {
    /// Create a new embedder with default settings
    pub fn new() -> Self {
        Self {
            include_timestamp: true,
            include_pattern: true,
            include_verification: true,
            include_context: true,
            compact_format: false,
        }
    }

    /// Create a compact embedder (minimal metadata)
    pub fn compact() -> Self {
        Self {
            include_timestamp: false,
            include_pattern: true,
            include_verification: false,
            include_context: false,
            compact_format: true,
        }
    }

    /// Create a full embedder (all metadata)
    pub fn full() -> Self {
        Self {
            include_timestamp: true,
            include_pattern: true,
            include_verification: true,
            include_context: true,
            compact_format: false,
        }
    }

    /// Set whether to include timestamp
    pub fn with_timestamp(mut self, include: bool) -> Self {
        self.include_timestamp = include;
        self
    }

    /// Set whether to include pattern
    pub fn with_pattern(mut self, include: bool) -> Self {
        self.include_pattern = include;
        self
    }

    /// Set whether to include verification
    pub fn with_verification(mut self, include: bool) -> Self {
        self.include_verification = include;
        self
    }

    /// Set whether to include context
    pub fn with_context(mut self, include: bool) -> Self {
        self.include_context = include;
        self
    }

    /// Embed metadata with current settings
    pub fn embed(&self, word_name: &str, word_body: &str, metadata: &ProvenanceMetadata) -> String {
        let mut output = String::new();

        // Always include generator
        output.push_str(&format!("\\ GENERATED_BY: {}\n", metadata.generated_by));

        // Conditional fields
        if self.include_pattern {
            if let Some(pattern_id) = &metadata.pattern_id {
                output.push_str(&format!("\\ PATTERN_ID: {}\n", pattern_id));
            }
        }

        if self.include_timestamp {
            output.push_str(&format!("\\ TIMESTAMP: {}\n", metadata.timestamp));
        }

        if self.include_verification {
            output.push_str(&format!("\\ VERIFIED: {}\n", metadata.verification.summary()));
        }

        if let Some(spec_hash) = &metadata.spec_hash {
            output.push_str(&format!("\\ SPEC_HASH: {}\n", spec_hash));
        }

        if self.include_context {
            if let Some(opt_level) = &metadata.context.optimization_level {
                output.push_str(&format!("\\ OPTIMIZATION_LEVEL: {}\n", opt_level));
            }
            if let Some(target) = &metadata.context.performance_target {
                output.push_str(&format!("\\ PERFORMANCE_TARGET: {}\n", target));
            }
        }

        // Add custom metadata
        for (key, value) in &metadata.custom {
            output.push_str(&format!("\\ {}: {}\n", key.to_uppercase(), value));
        }

        // Add word definition
        output.push_str(word_body);

        output
    }

    /// Embed metadata for multiple definitions
    pub fn embed_multiple(&self, definitions: &[(String, String, ProvenanceMetadata)]) -> String {
        let mut output = String::new();

        for (word_name, word_body, metadata) in definitions {
            output.push_str(&self.embed(word_name, word_body, metadata));
            if !self.compact_format {
                output.push('\n');
            }
        }

        output
    }
}

impl Default for ProvenanceEmbedder {
    fn default() -> Self {
        Self::new()
    }
}

/// Write provenance-embedded code to a file
pub fn write_to_file<P: AsRef<Path>>(
    path: P,
    word_name: &str,
    word_body: &str,
    metadata: &ProvenanceMetadata,
) -> Result<()> {
    let embedded = embed_provenance(word_name, word_body, metadata);
    std::fs::write(&path, embedded)
        .map_err(|e| CompileError::IoError(path.as_ref().to_path_buf(), e))?;
    Ok(())
}

/// Append provenance-embedded code to an existing file
pub fn append_to_file<P: AsRef<Path>>(
    path: P,
    word_name: &str,
    word_body: &str,
    metadata: &ProvenanceMetadata,
) -> Result<()> {
    use std::fs::OpenOptions;
    use std::io::Write;

    let embedded = embed_provenance(word_name, word_body, metadata);
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .map_err(|e| CompileError::IoError(path.as_ref().to_path_buf(), e))?;

    writeln!(file, "{}", embedded)
        .map_err(|e| CompileError::IoError(path.as_ref().to_path_buf(), e))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embed_provenance() {
        let metadata = ProvenanceMetadata::new("claude-sonnet-4".to_string())
            .with_pattern("RECURSIVE_004".to_string());

        let word_body = ": factorial ( n -- n! )\n  dup 2 < if drop 1 else dup 1- recurse * then ;";
        let embedded = embed_provenance("factorial", word_body, &metadata);

        assert!(embedded.contains("GENERATED_BY: claude-sonnet-4"));
        assert!(embedded.contains("PATTERN_ID: RECURSIVE_004"));
        assert!(embedded.contains("factorial"));
    }

    #[test]
    fn test_embed_multiple() {
        let definitions = vec![
            (
                "double".to_string(),
                ": double 2 * ;".to_string(),
                ProvenanceMetadata::new("claude".to_string()),
            ),
            (
                "triple".to_string(),
                ": triple 3 * ;".to_string(),
                ProvenanceMetadata::new("claude".to_string()),
            ),
        ];

        let embedded = embed_multiple(&definitions);
        assert!(embedded.contains("double"));
        assert!(embedded.contains("triple"));
    }

    #[test]
    fn test_embedder_compact() {
        let embedder = ProvenanceEmbedder::compact();
        let metadata = ProvenanceMetadata::new("claude-sonnet-4".to_string())
            .with_pattern("SIMPLE_001".to_string());

        let embedded = embedder.embed("square", ": square dup * ;", &metadata);
        assert!(embedded.contains("GENERATED_BY: claude-sonnet-4"));
        assert!(embedded.contains("PATTERN_ID: SIMPLE_001"));
        assert!(!embedded.contains("TIMESTAMP")); // Compact format excludes timestamp
    }

    #[test]
    fn test_embedder_full() {
        let embedder = ProvenanceEmbedder::full();
        let metadata = ProvenanceMetadata::new("claude-sonnet-4".to_string())
            .with_pattern("SIMPLE_001".to_string());

        let embedded = embedder.embed("square", ": square dup * ;", &metadata);
        assert!(embedded.contains("GENERATED_BY"));
        assert!(embedded.contains("PATTERN_ID"));
        assert!(embedded.contains("TIMESTAMP"));
        assert!(embedded.contains("VERIFIED"));
    }

    #[test]
    fn test_embedder_custom() {
        let embedder = ProvenanceEmbedder::new()
            .with_timestamp(false)
            .with_verification(false);

        let metadata = ProvenanceMetadata::new("claude".to_string());
        let embedded = embedder.embed("test", ": test ;", &metadata);

        assert!(embedded.contains("GENERATED_BY"));
        assert!(!embedded.contains("TIMESTAMP"));
        assert!(!embedded.contains("VERIFIED"));
    }
}
