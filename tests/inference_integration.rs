//! Integration tests for inference API

#[cfg(feature = "inference")]
mod inference_tests {
    use fastforth::inference::InferenceAPI;
    use std::time::Instant;

    #[test]
    fn test_simple_inference() {
        let api = InferenceAPI::new();
        let result = api.infer("dup *").unwrap();
        assert!(result.valid);
        assert_eq!(result.stack_depth_delta, 0);
    }

    #[test]
    fn test_verify_effect() {
        let api = InferenceAPI::new();
        let result = api.verify_effect("dup *", "( n -- n² )").unwrap();
        assert!(result.valid);
    }

    #[test]
    fn test_composition() {
        let api = InferenceAPI::new();
        let result = api.compose(&["dup", "*"]).unwrap();
        assert!(result.valid);
    }

    #[test]
    fn test_subsecond_latency() {
        let api = InferenceAPI::new();
        let start = Instant::now();
        for _ in 0..1000 {
            let _ = api.infer("dup * swap +");
        }
        let total_ms = start.elapsed().as_secs_f64() * 1000.0;
        let avg_ms = total_ms / 1000.0;

        // Each inference should take less than 1ms on average
        assert!(avg_ms < 1.0, "Average latency {}ms exceeds 1ms target", avg_ms);
    }
}
