//! Integration tests for Profile-Guided Optimization system
//!
//! Demonstrates real-world usage patterns and expected speedups

#[cfg(test)]
mod pgo_integration_tests {
    use fastforth_optimizer::{
        ForthIR, PGOOptimizer, PGOConfig, Instruction, Optimizer, OptimizationLevel,
    };
    use std::time::Duration;

    /// Test 1: Basic PGO workflow with balanced configuration
    #[test]
    fn test_pgo_balanced_workflow() {
        let mut pgo = PGOOptimizer::with_config(PGOConfig::balanced());
        pgo.enable_profiling();

        // Create simple IR with patterns
        let ir = ForthIR::parse("5 dup + 3 dup *").unwrap();

        // Profile the code (simulating 15K executions)
        for _ in 0..15_000 {
            pgo.profile_ir(&ir);
        }

        // Identify hot patterns (using balanced threshold)
        let hot = pgo.identify_hot_patterns(10_000);
        assert!(!hot.is_empty(), "Should find hot patterns");

        // Generate fusions
        let fusions = pgo.generate_fusions(&hot);
        assert!(!fusions.is_empty(), "Should generate fusions");

        println!("Balanced config: {} hot patterns, {} fusions", hot.len(), fusions.len());
    }

    /// Test 2: Aggressive PGO for maximum speedup
    #[test]
    fn test_pgo_aggressive_optimization() {
        let mut pgo = PGOOptimizer::with_config(PGOConfig::aggressive());
        pgo.enable_profiling();

        let ir = ForthIR::parse(
            ": square dup * ;
             : double dup + ;
             : test 1 + 2 - 1 +"
        ).unwrap();

        // Multiple profiling iterations for aggressive mode
        for _ in 0..20_000 {
            pgo.profile_ir(&ir);
        }

        // Adaptive threshold selection
        let hot = pgo.identify_hot_patterns_adaptive();
        let fusions = pgo.generate_fusions(&hot);

        // Optimize IR
        let (optimized, stats) = pgo.optimize(&ir, 5_000).unwrap();

        println!("Aggressive config results:");
        println!("  Hot patterns: {}", stats.hot_patterns_found);
        println!("  Fusions applied: {}", stats.fusions_applied);
        println!("  Estimated speedup: {:.1}%", stats.estimated_speedup_percent);

        // Verify optimization occurred
        assert!(stats.fusions_applied > 0, "Should apply fusions");
    }

    /// Test 3: Adaptive threshold calculation
    #[test]
    fn test_adaptive_threshold_selection() {
        let mut pgo = PGOOptimizer::new();
        pgo.enable_profiling();

        // Create IR with various pattern frequencies
        let patterns = vec![
            "dup +",          // Very hot
            "dup *",          // Very hot
            "1 +",            // Hot
            "2 *",            // Moderate
            "over +",         // Moderate
        ];

        let ir = ForthIR::parse(&patterns.join(" ")).unwrap();

        // Profile with different frequencies to create distribution
        for i in 0..20_000 {
            if i % 2 == 0 {
                pgo.profile_ir(&ir);  // Hot patterns
            }
        }

        // Use adaptive threshold (99th percentile)
        let adaptive_hot = pgo.identify_hot_patterns_adaptive();

        // Compare with fixed threshold
        let fixed_hot = pgo.identify_hot_patterns(10_000);

        println!("Threshold comparison:");
        println!("  Adaptive (99th percentile): {} patterns", adaptive_hot.len());
        println!("  Fixed (10K executions): {} patterns", fixed_hot.len());

        assert!(
            adaptive_hot.len() > 0,
            "Adaptive threshold should identify patterns"
        );
    }

    /// Test 4: Pattern ranking by ROI (Return on Investment)
    #[test]
    fn test_roi_based_pattern_ranking() {
        let mut pgo = PGOOptimizer::new();
        pgo.enable_profiling();

        // Create code with patterns of varying lengths
        let ir = ForthIR::parse(
            "5 dup +            : Pattern A (short, high impact)
             5 dup + dup * 1 +  : Pattern B (long, high impact)
             1 +                : Pattern C (very short, low impact)"
        ).unwrap();

        // Profile with different frequencies
        for _ in 0..15_000 {
            pgo.profile_ir(&ir);
        }

        let hot = pgo.identify_hot_patterns(5_000);

        // Top patterns should be ranked by ROI
        if hot.len() > 1 {
            let first = &hot[0];
            let second = &hot[1];

            println!("Top patterns by ROI:");
            println!("  1st: {} (ROI: {:.2})", first.key, first.roi_score);
            println!("  2nd: {} (ROI: {:.2})", second.key, second.roi_score);

            // First should have higher or equal ROI
            assert!(
                first.roi_score >= second.roi_score,
                "Patterns should be ordered by ROI"
            );
        }
    }

    /// Test 5: Speedup estimation and measurement
    #[test]
    fn test_speedup_measurement() {
        let mut pgo = PGOOptimizer::new();

        // Simulate execution times
        let baseline = Duration::from_millis(100);
        let optimized = Duration::from_millis(75);

        pgo.set_baseline_time(baseline);
        pgo.set_optimized_time(optimized);

        let speedup = pgo.measure_speedup();
        assert!(speedup.is_some(), "Should measure speedup");

        let speedup_percent = speedup.unwrap();
        assert!(speedup_percent > 0.0, "Speedup should be positive");
        assert!(speedup_percent < 100.0, "Speedup should be reasonable");

        println!("Measured speedup: {:.1}%", speedup_percent);
        assert!(
            speedup_percent >= 20.0,
            "Should achieve at least 20% speedup in this example"
        );
    }

    /// Test 6: Full optimization pipeline with PGO
    #[test]
    fn test_full_pgo_pipeline() {
        // Create realistic Forth code
        let source = ": square dup * ;
                     : sum 0 >r for >r + r> next r> ;
                     : process square sum ;
                     10 square 20 square process";

        let ir = ForthIR::parse(source).unwrap();
        let original_size = ir.instruction_count();

        // Profile
        let mut pgo = PGOOptimizer::with_config(PGOConfig::aggressive());
        pgo.enable_profiling();

        for _ in 0..25_000 {
            pgo.profile_ir(&ir);
        }

        // Get statistics before optimization
        let db_stats = pgo.database().stats();
        println!("Profiling results:");
        println!("  Total patterns: {}", db_stats.total_patterns);
        println!("  Coverage: {:.1}%", db_stats.coverage_percent);

        // Optimize
        let hot = pgo.identify_hot_patterns_adaptive();
        let fusions = pgo.generate_fusions(&hot);
        let (optimized, stats) = pgo.optimize(&ir, 5_000).unwrap();

        let optimized_size = optimized.instruction_count();
        let reduction = ((original_size as f64 - optimized_size as f64) / original_size as f64) * 100.0;

        println!("Optimization results:");
        println!("  Original size: {} instructions", original_size);
        println!("  Optimized size: {} instructions", optimized_size);
        println!("  Code reduction: {:.1}%", reduction);
        println!("  Fusions applied: {}", stats.fusions_applied);
        println!("  Estimated speedup: {:.1}%", stats.estimated_speedup_percent);

        // Verify optimization
        assert!(stats.fusions_applied > 0, "Should apply fusions");
        assert!(reduction > 0.0, "Should reduce code size");
    }

    /// Test 7: Conservative vs Aggressive comparison
    #[test]
    fn test_configuration_comparison() {
        let ir = ForthIR::parse(
            "5 dup + 3 dup * 1 + 2 * 0 ="
        ).unwrap();

        // Aggressive profile
        let mut pgo_agg = PGOOptimizer::with_config(PGOConfig::aggressive());
        pgo_agg.enable_profiling();
        for _ in 0..15_000 {
            pgo_agg.profile_ir(&ir);
        }
        let hot_agg = pgo_agg.identify_hot_patterns_adaptive();

        // Conservative profile
        let mut pgo_cons = PGOOptimizer::with_config(PGOConfig::conservative());
        pgo_cons.enable_profiling();
        for _ in 0..15_000 {
            pgo_cons.profile_ir(&ir);
        }
        let hot_cons = pgo_cons.identify_hot_patterns(50_000);

        println!("Configuration comparison:");
        println!("  Aggressive: {} hot patterns", hot_agg.len());
        println!("  Conservative: {} hot patterns", hot_cons.len());

        // Aggressive should find more patterns
        assert!(
            hot_agg.len() >= hot_cons.len(),
            "Aggressive should find at least as many patterns"
        );
    }

    /// Test 8: Multi-iteration optimization (auto-tuning)
    #[test]
    fn test_multi_iteration_optimization() {
        let ir = ForthIR::parse(
            ": compute dup + dup * 1 + dup + 2 * 1 ;"
        ).unwrap();

        let mut pgo = PGOOptimizer::with_config(PGOConfig::aggressive());
        pgo.enable_profiling();

        for _ in 0..30_000 {
            pgo.profile_ir(&ir);
        }

        let mut total_fusions = 0;
        let iterations = 5;

        for i in 1..=iterations {
            let hot = pgo.identify_hot_patterns_adaptive();
            let fusions = pgo.generate_fusions(&hot);

            println!("Iteration {}: {} patterns, {} fusions", i, hot.len(), fusions.len());
            total_fusions += fusions.len();
        }

        println!("Total fusions across {} iterations: {}", iterations, total_fusions);
        assert!(total_fusions > 0, "Should generate fusions across iterations");
    }

    /// Test 9: Pattern coverage metrics
    #[test]
    fn test_pattern_coverage_analysis() {
        let mut pgo = PGOOptimizer::new();
        pgo.enable_profiling();

        let ir = ForthIR::parse(
            ": loop-test 100 0 for dup + next ;"
        ).unwrap();

        for _ in 0..50_000 {
            pgo.profile_ir(&ir);
        }

        let _hot = pgo.identify_hot_patterns_adaptive();
        let stats = pgo.database().stats();

        println!("Coverage analysis:");
        println!("  Total patterns: {}", stats.total_patterns);
        println!("  Hot patterns: {}", stats.hot_patterns);
        println!("  Coverage: {:.1}%", stats.coverage_percent);
        println!("  Total instructions: {}", stats.total_instructions);

        assert!(stats.coverage_percent > 50.0, "Should cover significant portion of execution");
    }

    /// Test 10: Real-world Fibonacci example
    #[test]
    fn test_fibonacci_optimization() {
        let fibonacci = ": fib ( n -- fib(n) )
                          dup 1 <= if drop 1 else
                            dup 1 - recurse >r
                            dup 2 - recurse
                            r> +
                          then ;";

        let ir = ForthIR::parse(fibonacci).ok();

        if let Some(ir) = ir {
            let mut pgo = PGOOptimizer::with_config(PGOConfig::aggressive());
            pgo.enable_profiling();

            // Simulate calling fib multiple times
            for _ in 0..10_000 {
                pgo.profile_ir(&ir);
            }

            let hot = pgo.identify_hot_patterns_adaptive();
            let (optimized, stats) = pgo.optimize(&ir, 5_000).unwrap();

            println!("Fibonacci optimization:");
            println!("  Hot patterns: {}", hot.len());
            println!("  Fusions applied: {}", stats.fusions_applied);
            println!("  Code reduction: {:.1}%", stats.code_reduction_percent);
            println!("  Estimated speedup: {:.1}%", stats.estimated_speedup_percent);

            if stats.fusions_applied > 0 {
                println!("Successfully optimized recursive Fibonacci");
            }
        }
    }
}
