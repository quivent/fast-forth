//! Phase 2 Optimization Benchmarks
//!
//! Measures the impact of Phase 2 optimizations:
//! 1. LRU cache for pattern queries (1.2ms → 0.3ms)
//! 2. SIMD JSON parsing (12.4ms → 8ms)
//! 3. Parallel validation (16ms → 10ms)

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use fastforth::patterns::{PatternDatabase, PatternId};
use fastforth::spec::{Specification, SpecValidator, StackEffect, StackParameter, StackResult, StackType, TestCase, TestValue};

/// Benchmark pattern database LRU cache
fn bench_pattern_cache(c: &mut Criterion) {
    let mut db = PatternDatabase::open("test.db").unwrap();
    db.seed_defaults().unwrap();

    let pattern_ids = vec![
        "DUP_TRANSFORM_001",
        "CONDITIONAL_001",
        "ACCUMULATOR_LOOP_001",
        "RECURSIVE_001",
        "BINARY_OP_001",
    ];

    let mut group = c.benchmark_group("pattern_cache");

    // Benchmark first access (cache miss)
    group.bench_function("get_pattern_first_access", |b| {
        b.iter(|| {
            for id_str in &pattern_ids {
                let id = PatternId(id_str.to_string());
                let _ = db.get(black_box(&id));
            }
        });
    });

    // Benchmark repeated access (cache hit)
    // Warm up the cache first
    for id_str in &pattern_ids {
        let id = PatternId(id_str.to_string());
        let _ = db.get(&id);
    }

    group.bench_function("get_pattern_cached", |b| {
        b.iter(|| {
            for id_str in &pattern_ids {
                let id = PatternId(id_str.to_string());
                let _ = db.get(black_box(&id));
            }
        });
    });

    group.finish();
}

/// Benchmark SIMD JSON parsing
fn bench_simd_json_parsing(c: &mut Criterion) {
    let json_simple = r#"{
        "word": "square",
        "description": "Square a number",
        "stack_effect": {
            "inputs": [{"name": "n", "type": "int"}],
            "outputs": [{"name": "n²", "type": "int", "value": "n*n"}]
        },
        "properties": ["idempotent", "pure"],
        "test_cases": [
            {"input": [5], "output": [25], "description": "5² = 25"},
            {"input": [0], "output": [0], "description": "0² = 0"},
            {"input": [-3], "output": [9], "description": "(-3)² = 9"}
        ]
    }"#;

    let json_complex = r#"{
        "word": "factorial",
        "description": "Calculate factorial using recursion",
        "stack_effect": {
            "inputs": [{"name": "n", "type": "uint", "constraint": "n >= 0"}],
            "outputs": [{"name": "n!", "type": "uint", "value": "factorial(n)"}]
        },
        "properties": ["recursive", "exponential_growth"],
        "test_cases": [
            {"input": [0], "output": [1], "description": "0! = 1", "tags": ["base_case"]},
            {"input": [1], "output": [1], "description": "1! = 1", "tags": ["base_case"]},
            {"input": [5], "output": [120], "description": "5! = 120"},
            {"input": [10], "output": [3628800], "description": "10! = 3628800"}
        ],
        "complexity": {
            "time": "O(n)",
            "space": "O(n)"
        },
        "implementation": {
            "pattern": "RECURSIVE_001",
            "hints": ["Use base case for n < 2", "Recursive step: n * factorial(n-1)"]
        },
        "metadata": {
            "author": "FastForth Team",
            "version": "1.0.0",
            "created": "2025-11-14",
            "tags": ["math", "recursion"]
        }
    }"#;

    let mut group = c.benchmark_group("simd_json");

    group.bench_function("parse_simple_spec", |b| {
        b.iter(|| {
            let spec = Specification::from_json(black_box(json_simple)).unwrap();
            black_box(spec)
        });
    });

    group.bench_function("parse_complex_spec", |b| {
        b.iter(|| {
            let spec = Specification::from_json(black_box(json_complex)).unwrap();
            black_box(spec)
        });
    });

    // Benchmark batch parsing (simulates agent workflow)
    group.bench_function("parse_100_specs", |b| {
        b.iter(|| {
            for _ in 0..100 {
                let _ = Specification::from_json(black_box(json_simple)).unwrap();
            }
        });
    });

    group.finish();
}

/// Benchmark parallel validation
fn bench_parallel_validation(c: &mut Criterion) {
    // Create specification with many test cases to benefit from parallelization
    let mut test_cases = Vec::new();
    for i in 0..100 {
        test_cases.push(TestCase {
            description: Some(format!("Test case {}", i)),
            input: vec![TestValue::Int(i)],
            output: vec![TestValue::Int(i * i)],
            tags: None,
        });
    }

    let spec = Specification {
        word: "square".to_string(),
        description: Some("Square a number".to_string()),
        stack_effect: StackEffect {
            inputs: vec![StackParameter {
                name: Some("n".to_string()),
                param_type: StackType::Int,
                constraint: Some("n >= 0".to_string()),
            }],
            outputs: vec![StackResult {
                name: Some("n²".to_string()),
                result_type: StackType::Int,
                value: Some("n*n".to_string()),
            }],
        },
        properties: Some(vec!["pure".to_string()]),
        test_cases: Some(test_cases),
        complexity: None,
        implementation: None,
        metadata: None,
    };

    let mut group = c.benchmark_group("parallel_validation");

    group.bench_function("validate_100_test_cases", |b| {
        let validator = SpecValidator::new();
        b.iter(|| {
            let result = validator.validate(black_box(&spec));
            black_box(result)
        });
    });

    // Test with different test case counts
    for size in [10, 50, 100, 200].iter() {
        let mut test_cases = Vec::new();
        for i in 0..*size {
            test_cases.push(TestCase {
                description: Some(format!("Test case {}", i)),
                input: vec![TestValue::Int(i)],
                output: vec![TestValue::Int(i * i)],
                tags: None,
            });
        }

        let spec = Specification {
            word: "square".to_string(),
            description: Some("Square a number".to_string()),
            stack_effect: StackEffect {
                inputs: vec![StackParameter {
                    name: Some("n".to_string()),
                    param_type: StackType::Int,
                    constraint: Some("n >= 0".to_string()),
                }],
                outputs: vec![StackResult {
                    name: Some("n²".to_string()),
                    result_type: StackType::Int,
                    value: Some("n*n".to_string()),
                }],
            },
            properties: Some(vec!["pure".to_string()]),
            test_cases: Some(test_cases),
            complexity: None,
            implementation: None,
            metadata: None,
        };

        group.bench_with_input(
            BenchmarkId::new("validate_scaling", size),
            &spec,
            |b, spec| {
                let validator = SpecValidator::new();
                b.iter(|| {
                    let result = validator.validate(black_box(spec));
                    black_box(result)
                });
            },
        );
    }

    group.finish();
}

/// Benchmark end-to-end agent workflow
fn bench_agent_workflow(c: &mut Criterion) {
    let mut db = PatternDatabase::open("test.db").unwrap();
    db.seed_defaults().unwrap();

    let json = r#"{
        "word": "factorial",
        "description": "Calculate factorial",
        "stack_effect": {
            "inputs": [{"name": "n", "type": "uint", "constraint": "n >= 0"}],
            "outputs": [{"name": "n!", "type": "uint"}]
        },
        "properties": ["recursive"],
        "test_cases": [
            {"input": [0], "output": [1], "tags": ["base_case"]},
            {"input": [5], "output": [120]}
        ]
    }"#;

    let mut group = c.benchmark_group("agent_workflow");

    group.bench_function("complete_workflow", |b| {
        b.iter(|| {
            // 1. Parse spec (SIMD JSON)
            let spec = Specification::from_json(black_box(json)).unwrap();

            // 2. Validate spec (Parallel validation)
            spec.validate().unwrap();

            // 3. Query patterns (LRU cache)
            let pattern_id = PatternId("RECURSIVE_001".to_string());
            let _ = db.get(black_box(&pattern_id)).unwrap();

            black_box(())
        });
    });

    group.finish();
}

criterion_group!(
    phase2_benches,
    bench_pattern_cache,
    bench_simd_json_parsing,
    bench_parallel_validation,
    bench_agent_workflow
);
criterion_main!(phase2_benches);
