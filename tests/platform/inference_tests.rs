/// Tests for inference feature
///
/// These tests only compile when the 'inference' feature is enabled.
/// Run with: cargo test --features inference

#[test]
fn test_inference_feature_enabled() {
    assert!(
        cfg!(feature = "inference"),
        "inference feature should be enabled"
    );
}

#[test]
fn test_inference_module_available() {
    // Verify inference module is accessible when feature is enabled
    use fastforth::inference::InferenceEngine;

    let _engine = InferenceEngine::new();
}

#[test]
fn test_type_inference_basic() {
    use fastforth::inference::InferenceEngine;

    let mut engine = InferenceEngine::new();

    // Test basic type inference
    // Example: : double dup + ;
    // Should infer: ( n -- n ) where both n's are the same type

    // Implementation depends on current inference API
    // This is a placeholder for actual inference tests
}

#[test]
fn test_inference_in_default_features() {
    // Inference should be in default features
    #[cfg(feature = "default")]
    {
        assert!(
            cfg!(feature = "inference"),
            "inference should be enabled in default features"
        );
    }
}
