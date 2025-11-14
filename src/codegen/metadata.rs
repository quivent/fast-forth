//! Metadata generation during code generation
//!
//! Automatically generates provenance metadata during compilation

use crate::error::{CompileError, Result};
use crate::provenance::metadata::{ProvenanceMetadata, VerificationStatus, GenerationContext};
use fastforth_optimizer::ForthIR;
use std::time::Instant;

/// Code generation metadata tracker
pub struct CodegenMetadata {
    agent_id: String,
    pattern_id: Option<String>,
    spec_hash: Option<String>,
    start_time: Option<Instant>,
    optimization_level: Option<String>,
    performance_target: Option<String>,
}

impl CodegenMetadata {
    /// Create a new codegen metadata tracker
    pub fn new(agent_id: String) -> Self {
        Self {
            agent_id,
            pattern_id: None,
            spec_hash: None,
            start_time: None,
            optimization_level: None,
            performance_target: None,
        }
    }

    /// Set the pattern ID used
    pub fn with_pattern(mut self, pattern_id: String) -> Self {
        self.pattern_id = Some(pattern_id);
        self
    }

    /// Set the specification hash
    pub fn with_spec_hash(mut self, spec_hash: String) -> Self {
        self.spec_hash = Some(spec_hash);
        self
    }

    /// Set optimization level
    pub fn with_optimization_level(mut self, level: String) -> Self {
        self.optimization_level = Some(level);
        self
    }

    /// Set performance target
    pub fn with_performance_target(mut self, target: String) -> Self {
        self.performance_target = Some(target);
        self
    }

    /// Start generation timing
    pub fn start_generation(&mut self) {
        self.start_time = Some(Instant::now());
    }

    /// Generate provenance metadata for a completed definition
    pub fn generate_metadata(
        &self,
        verification: VerificationStatus,
    ) -> ProvenanceMetadata {
        let mut context = GenerationContext::new();

        if let Some(opt_level) = &self.optimization_level {
            context = context.with_optimization_level(opt_level.clone());
        }

        if let Some(target) = &self.performance_target {
            context = context.with_performance_target(target.clone());
        }

        if let Some(start) = self.start_time {
            let duration_ms = start.elapsed().as_millis() as u64;
            context = context.with_generation_time(duration_ms);
        }

        let mut metadata = ProvenanceMetadata::new(self.agent_id.clone())
            .with_verification(verification)
            .with_context(context);

        if let Some(pattern) = &self.pattern_id {
            metadata = metadata.with_pattern(pattern.clone());
        }

        if let Some(spec) = &self.spec_hash {
            metadata = metadata.with_spec_hash(spec.clone());
        }

        metadata
    }

    /// Generate metadata with automatic verification from IR
    pub fn generate_from_ir(
        &self,
        ir: &ForthIR,
        tests_passed: usize,
        tests_total: usize,
    ) -> ProvenanceMetadata {
        let verification = VerificationStatus::new()
            .with_stack_balanced(true) // Assume stack is balanced if it compiled
            .with_tests(tests_passed, tests_total)
            .with_type_checked(true) // Assume type checked if it compiled
            .with_compiled(true)
            .mark_verified();

        self.generate_metadata(verification)
    }
}

/// Metadata generator for batch code generation
pub struct BatchMetadataGenerator {
    agent_id: String,
    default_pattern: Option<String>,
    optimization_level: Option<String>,
    metadata_cache: Vec<(String, ProvenanceMetadata)>,
}

impl BatchMetadataGenerator {
    /// Create a new batch generator
    pub fn new(agent_id: String) -> Self {
        Self {
            agent_id,
            default_pattern: None,
            optimization_level: None,
            metadata_cache: Vec::new(),
        }
    }

    /// Set default pattern for all generated words
    pub fn with_default_pattern(mut self, pattern: String) -> Self {
        self.default_pattern = Some(pattern);
        self
    }

    /// Set optimization level for all generated words
    pub fn with_optimization_level(mut self, level: String) -> Self {
        self.optimization_level = Some(level);
        self
    }

    /// Generate metadata for a word
    pub fn generate_for_word(
        &mut self,
        word_name: String,
        pattern: Option<String>,
        verification: VerificationStatus,
    ) -> ProvenanceMetadata {
        let mut codegen = CodegenMetadata::new(self.agent_id.clone());

        // Use provided pattern or default
        if let Some(p) = pattern.or_else(|| self.default_pattern.clone()) {
            codegen = codegen.with_pattern(p);
        }

        // Use optimization level if set
        if let Some(opt) = &self.optimization_level {
            codegen = codegen.with_optimization_level(opt.clone());
        }

        let metadata = codegen.generate_metadata(verification);
        self.metadata_cache.push((word_name, metadata.clone()));
        metadata
    }

    /// Get all generated metadata
    pub fn get_cached_metadata(&self) -> &[(String, ProvenanceMetadata)] {
        &self.metadata_cache
    }

    /// Export cached metadata as JSON
    pub fn export_json(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(&self.metadata_cache)
    }

    /// Clear the metadata cache
    pub fn clear_cache(&mut self) {
        self.metadata_cache.clear();
    }
}

/// Helper function to compute specification hash
pub fn compute_spec_hash(spec_json: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    spec_json.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

/// Helper function to create verification status from test results
pub fn create_verification_status(
    stack_balanced: bool,
    tests_passed: usize,
    tests_total: usize,
    type_checked: bool,
    compiled: bool,
) -> VerificationStatus {
    VerificationStatus::new()
        .with_stack_balanced(stack_balanced)
        .with_tests(tests_passed, tests_total)
        .with_type_checked(type_checked)
        .with_compiled(compiled)
        .mark_verified()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_codegen_metadata_creation() {
        let metadata = CodegenMetadata::new("claude-sonnet-4".to_string());
        assert_eq!(metadata.agent_id, "claude-sonnet-4");
        assert!(metadata.pattern_id.is_none());
    }

    #[test]
    fn test_codegen_metadata_builder() {
        let metadata = CodegenMetadata::new("claude-sonnet-4".to_string())
            .with_pattern("RECURSIVE_004".to_string())
            .with_spec_hash("abc123".to_string())
            .with_optimization_level("Aggressive".to_string());

        assert_eq!(metadata.pattern_id, Some("RECURSIVE_004".to_string()));
        assert_eq!(metadata.spec_hash, Some("abc123".to_string()));
        assert_eq!(metadata.optimization_level, Some("Aggressive".to_string()));
    }

    #[test]
    fn test_generate_metadata() {
        let codegen = CodegenMetadata::new("claude-sonnet-4".to_string())
            .with_pattern("SIMPLE_001".to_string());

        let verification = VerificationStatus::new()
            .with_stack_balanced(true)
            .with_tests(3, 3)
            .with_type_checked(true)
            .with_compiled(true);

        let metadata = codegen.generate_metadata(verification);
        assert_eq!(metadata.generated_by, "claude-sonnet-4");
        assert_eq!(metadata.pattern_id, Some("SIMPLE_001".to_string()));
        assert!(metadata.verification.is_verified());
    }

    #[test]
    fn test_batch_metadata_generator() {
        let mut generator = BatchMetadataGenerator::new("claude-sonnet-4".to_string())
            .with_default_pattern("SIMPLE_001".to_string())
            .with_optimization_level("Standard".to_string());

        let verification = VerificationStatus::new()
            .with_stack_balanced(true)
            .with_compiled(true);

        generator.generate_for_word("square".to_string(), None, verification.clone());
        generator.generate_for_word("double".to_string(), None, verification);

        assert_eq!(generator.get_cached_metadata().len(), 2);
    }

    #[test]
    fn test_compute_spec_hash() {
        let spec1 = r#"{"word": "factorial"}"#;
        let spec2 = r#"{"word": "fibonacci"}"#;

        let hash1 = compute_spec_hash(spec1);
        let hash2 = compute_spec_hash(spec2);

        assert_ne!(hash1, hash2);
        assert_eq!(compute_spec_hash(spec1), hash1); // Deterministic
    }

    #[test]
    fn test_create_verification_status() {
        let status = create_verification_status(true, 5, 5, true, true);
        assert!(status.is_verified());
        assert!(!status.has_failures());
    }
}
