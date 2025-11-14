//! Simple demonstration of inference API performance

use fastforth::inference::InferenceAPI;
use std::time::Instant;

fn main() {
    let api = InferenceAPI::new();

    println!("Fast Forth Inference API Performance Demo");
    println!("==========================================\n");

    // Test 1: Simple operation
    println!("Test 1: Simple operation");
    let start = Instant::now();
    match api.infer("dup *") {
        Ok(result) => {
            println!("  Code: dup *");
            println!("  Effect: {}", result.inferred_effect);
            println!("  Latency: {:.3}ms", result.latency_ms);
            println!("  Valid: {}", result.valid);
        }
        Err(e) => println!("  Error: {}", e),
    }
    println!();

    // Test 2: Complex composition
    println!("Test 2: Complex composition");
    match api.infer("dup * swap +") {
        Ok(result) => {
            println!("  Code: dup * swap +");
            println!("  Effect: {}", result.inferred_effect);
            println!("  Depth Delta: {}", result.stack_depth_delta);
            println!("  Operations: {}", result.operations.join(" "));
            println!("  Latency: {:.3}ms", result.latency_ms);
        }
        Err(e) => println!("  Error: {}", e),
    }
    println!();

    // Test 3: Verify effect
    println!("Test 3: Verify effect");
    match api.verify_effect("dup *", "( n -- n² )") {
        Ok(result) => {
            println!("  Code: dup *");
            println!("  Expected: {}", result.expected);
            println!("  Inferred: {}", result.inferred);
            println!("  Match: {}", result.valid);
            println!("  Latency: {:.3}ms", result.latency_ms);
        }
        Err(e) => println!("  Error: {}", e),
    }
    println!();

    // Test 4: Throughput test
    println!("Test 4: Throughput (1000 inferences)");
    let start = Instant::now();
    let mut success_count = 0;
    for _ in 0..1000 {
        if api.infer("dup * swap +").is_ok() {
            success_count += 1;
        }
    }
    let total_ms = start.elapsed().as_secs_f64() * 1000.0;
    let avg_ms = total_ms / 1000.0;
    println!("  Total time: {:.2}ms", total_ms);
    println!("  Average per inference: {:.3}ms", avg_ms);
    println!("  Successful: {}/1000", success_count);
    println!("  Throughput: {:.0} inferences/sec", 1000.0 / (total_ms / 1000.0));
    println!();

    // Summary
    println!("Summary");
    println!("=======");
    if avg_ms < 1.0 {
        println!("✓ Sub-millisecond performance achieved ({:.3}ms average)", avg_ms);
        println!("✓ Target: <1ms typical latency - PASSED");
    } else {
        println!("✗ Sub-millisecond performance NOT achieved ({:.3}ms average)", avg_ms);
        println!("✗ Target: <1ms typical latency - FAILED");
    }
}
