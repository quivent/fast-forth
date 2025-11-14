//! Provenance metadata structures
//!
//! Defines the metadata format for tracking code generation provenance

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Complete provenance metadata for a code definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvenanceMetadata {
    /// Agent/model that generated this code
    pub generated_by: String,

    /// Pattern ID used (if applicable)
    pub pattern_id: Option<String>,

    /// Timestamp of generation (ISO 8601)
    pub timestamp: String,

    /// Verification status
    pub verification: VerificationStatus,

    /// Hash of the specification used
    pub spec_hash: Option<String>,

    /// Generation context
    pub context: GenerationContext,

    /// Additional custom metadata
    pub custom: HashMap<String, String>,
}

impl ProvenanceMetadata {
    /// Create new metadata with required fields
    pub fn new(generated_by: String) -> Self {
        Self {
            generated_by,
            pattern_id: None,
            timestamp: chrono::Utc::now().to_rfc3339(),
            verification: VerificationStatus::default(),
            spec_hash: None,
            context: GenerationContext::default(),
            custom: HashMap::new(),
        }
    }

    /// Set the pattern ID
    pub fn with_pattern(mut self, pattern_id: String) -> Self {
        self.pattern_id = Some(pattern_id);
        self
    }

    /// Set the specification hash
    pub fn with_spec_hash(mut self, spec_hash: String) -> Self {
        self.spec_hash = Some(spec_hash);
        self
    }

    /// Set the generation context
    pub fn with_context(mut self, context: GenerationContext) -> Self {
        self.context = context;
        self
    }

    /// Set verification status
    pub fn with_verification(mut self, verification: VerificationStatus) -> Self {
        self.verification = verification;
        self
    }

    /// Add custom metadata field
    pub fn add_custom(mut self, key: String, value: String) -> Self {
        self.custom.insert(key, value);
        self
    }

    /// Format as Forth comment block
    pub fn to_forth_comment(&self) -> String {
        let mut comment = String::new();
        comment.push_str(&format!("\\ GENERATED_BY: {}\n", self.generated_by));

        if let Some(pattern_id) = &self.pattern_id {
            comment.push_str(&format!("\\ PATTERN_ID: {}\n", pattern_id));
        }

        comment.push_str(&format!("\\ TIMESTAMP: {}\n", self.timestamp));
        comment.push_str(&format!("\\ VERIFIED: {}\n", self.verification.summary()));

        if let Some(spec_hash) = &self.spec_hash {
            comment.push_str(&format!("\\ SPEC_HASH: {}\n", spec_hash));
        }

        // Add context information
        if let Some(optimization_level) = &self.context.optimization_level {
            comment.push_str(&format!("\\ OPTIMIZATION_LEVEL: {}\n", optimization_level));
        }

        if let Some(target) = &self.context.performance_target {
            comment.push_str(&format!("\\ PERFORMANCE_TARGET: {}\n", target));
        }

        // Add custom fields
        for (key, value) in &self.custom {
            comment.push_str(&format!("\\ {}: {}\n", key.to_uppercase(), value));
        }

        comment
    }

    /// Format as JSON
    pub fn to_json(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }

    /// Parse from JSON
    pub fn from_json(json: &str) -> serde_json::Result<Self> {
        serde_json::from_str(json)
    }
}

/// Verification status of generated code
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VerificationStatus {
    /// Stack effect balanced
    pub stack_balanced: bool,

    /// Number of tests passed
    pub tests_passed: usize,

    /// Total number of tests
    pub tests_total: usize,

    /// Type checking passed
    pub type_checked: bool,

    /// Compilation succeeded
    pub compiled: bool,

    /// Performance target met (if applicable)
    pub performance_met: Option<bool>,

    /// Verification timestamp
    pub verified_at: Option<String>,
}

impl VerificationStatus {
    /// Create a new verification status
    pub fn new() -> Self {
        Self::default()
    }

    /// Set stack balance status
    pub fn with_stack_balanced(mut self, balanced: bool) -> Self {
        self.stack_balanced = balanced;
        self
    }

    /// Set test results
    pub fn with_tests(mut self, passed: usize, total: usize) -> Self {
        self.tests_passed = passed;
        self.tests_total = total;
        self
    }

    /// Set type checking status
    pub fn with_type_checked(mut self, checked: bool) -> Self {
        self.type_checked = checked;
        self
    }

    /// Set compilation status
    pub fn with_compiled(mut self, compiled: bool) -> Self {
        self.compiled = compiled;
        self
    }

    /// Set performance target status
    pub fn with_performance_met(mut self, met: bool) -> Self {
        self.performance_met = Some(met);
        self
    }

    /// Mark as verified with current timestamp
    pub fn mark_verified(mut self) -> Self {
        self.verified_at = Some(chrono::Utc::now().to_rfc3339());
        self
    }

    /// Check if code is fully verified
    pub fn is_verified(&self) -> bool {
        self.stack_balanced
            && self.type_checked
            && self.compiled
            && self.tests_passed == self.tests_total
            && self.tests_total > 0
    }

    /// Check if there are test failures
    pub fn has_failures(&self) -> bool {
        self.tests_total > 0 && self.tests_passed < self.tests_total
    }

    /// Get test pass rate (0.0-1.0)
    pub fn test_pass_rate(&self) -> f64 {
        if self.tests_total == 0 {
            0.0
        } else {
            self.tests_passed as f64 / self.tests_total as f64
        }
    }

    /// Get a summary string
    pub fn summary(&self) -> String {
        format!(
            "stack_balanced={}, tests_passed={}/{}, type_checked={}, compiled={}",
            self.stack_balanced,
            self.tests_passed,
            self.tests_total,
            self.type_checked,
            self.compiled
        )
    }
}

/// Generation context information
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GenerationContext {
    /// Optimization level used
    pub optimization_level: Option<String>,

    /// Performance target (if specified)
    pub performance_target: Option<String>,

    /// Source specification file
    pub spec_file: Option<String>,

    /// Generation iteration (for multi-attempt generation)
    pub iteration: Option<usize>,

    /// Total generation time in milliseconds
    pub generation_time_ms: Option<u64>,

    /// Model temperature (for LLM generation)
    pub temperature: Option<f64>,

    /// Additional context metadata
    pub metadata: HashMap<String, String>,
}

impl GenerationContext {
    /// Create a new context
    pub fn new() -> Self {
        Self::default()
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

    /// Set source specification file
    pub fn with_spec_file(mut self, file: String) -> Self {
        self.spec_file = Some(file);
        self
    }

    /// Set generation iteration
    pub fn with_iteration(mut self, iteration: usize) -> Self {
        self.iteration = Some(iteration);
        self
    }

    /// Set generation time
    pub fn with_generation_time(mut self, time_ms: u64) -> Self {
        self.generation_time_ms = Some(time_ms);
        self
    }

    /// Set model temperature
    pub fn with_temperature(mut self, temperature: f64) -> Self {
        self.temperature = Some(temperature);
        self
    }

    /// Add metadata field
    pub fn add_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

// Chrono is needed for timestamps, add a simple implementation
mod chrono {
    pub struct Utc;

    impl Utc {
        pub fn now() -> DateTime {
            DateTime
        }
    }

    pub struct DateTime;

    impl DateTime {
        pub fn to_rfc3339(&self) -> String {
            // Simple implementation using system time
            use std::time::{SystemTime, UNIX_EPOCH};
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap();
            format!("2025-01-15T{:02}:{:02}:{:02}Z",
                (now.as_secs() / 3600) % 24,
                (now.as_secs() / 60) % 60,
                now.as_secs() % 60)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provenance_metadata_creation() {
        let metadata = ProvenanceMetadata::new("claude-sonnet-4".to_string());
        assert_eq!(metadata.generated_by, "claude-sonnet-4");
        assert!(metadata.pattern_id.is_none());
    }

    #[test]
    fn test_provenance_metadata_builder() {
        let metadata = ProvenanceMetadata::new("claude-sonnet-4".to_string())
            .with_pattern("RECURSIVE_004".to_string())
            .with_spec_hash("a3f7b2c9d1e4".to_string());

        assert_eq!(metadata.pattern_id, Some("RECURSIVE_004".to_string()));
        assert_eq!(metadata.spec_hash, Some("a3f7b2c9d1e4".to_string()));
    }

    #[test]
    fn test_verification_status() {
        let status = VerificationStatus::new()
            .with_stack_balanced(true)
            .with_tests(3, 3)
            .with_type_checked(true)
            .with_compiled(true);

        assert!(status.is_verified());
        assert!(!status.has_failures());
        assert_eq!(status.test_pass_rate(), 1.0);
    }

    #[test]
    fn test_verification_status_failures() {
        let status = VerificationStatus::new()
            .with_tests(2, 5);

        assert!(status.has_failures());
        assert_eq!(status.test_pass_rate(), 0.4);
    }

    #[test]
    fn test_generation_context() {
        let context = GenerationContext::new()
            .with_optimization_level("Aggressive".to_string())
            .with_iteration(3);

        assert_eq!(context.optimization_level, Some("Aggressive".to_string()));
        assert_eq!(context.iteration, Some(3));
    }

    #[test]
    fn test_forth_comment_generation() {
        let metadata = ProvenanceMetadata::new("claude-sonnet-4".to_string())
            .with_pattern("RECURSIVE_004".to_string());

        let comment = metadata.to_forth_comment();
        assert!(comment.contains("GENERATED_BY: claude-sonnet-4"));
        assert!(comment.contains("PATTERN_ID: RECURSIVE_004"));
    }
}
