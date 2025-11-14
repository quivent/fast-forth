//! Benchmarks for stack effect inference
//!
//! Verifies sub-millisecond (<1ms) inference latency

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use fastforth::inference::InferenceAPI;

fn bench_simple_operations(c: &mut Criterion) {
    let api = InferenceAPI::new();

    let mut group = c.benchmark_group("simple_operations");

    let test_cases = vec![
        ("dup", "Duplicate"),
        ("dup *", "Square"),
        ("swap", "Swap"),
        ("+ -", "Add then subtract"),
        ("42 13 +", "Literal addition"),
    ];

    for (code, name) in test_cases {
        group.bench_with_input(BenchmarkId::new("infer", name), &code, |b, &code| {
            b.iter(|| {
                let result = api.infer(black_box(code));
                black_box(result)
            });
        });
    }

    group.finish();
}

fn bench_complex_compositions(c: &mut Criterion) {
    let api = InferenceAPI::new();

    let mut group = c.benchmark_group("complex_compositions");

    let test_cases = vec![
        ("dup * swap +", "Square and add"),
        ("dup dup * swap dup * +", "Sum of squares"),
        ("2dup + rot rot + +", "Complex stack manipulation"),
        ("over over * swap dup * + swap dup *", "Very complex"),
    ];

    for (code, name) in test_cases {
        group.bench_with_input(BenchmarkId::new("infer", name), &code, |b, &code| {
            b.iter(|| {
                let result = api.infer(black_box(code));
                black_box(result)
            });
        });
    }

    group.finish();
}

fn bench_verify_effect(c: &mut Criterion) {
    let api = InferenceAPI::new();

    let mut group = c.benchmark_group("verify_effect");

    let test_cases = vec![
        ("dup *", "( n -- n² )", "Square"),
        ("swap", "( a b -- b a )", "Swap"),
        ("2dup +", "( a b -- a b a+b )", "2dup add"),
    ];

    for (code, effect, name) in test_cases {
        group.bench_with_input(BenchmarkId::new("verify", name), &(code, effect), |b, &(code, effect)| {
            b.iter(|| {
                let result = api.verify_effect(black_box(code), black_box(effect));
                black_box(result)
            });
        });
    }

    group.finish();
}

fn bench_composition(c: &mut Criterion) {
    let api = InferenceAPI::new();

    let mut group = c.benchmark_group("composition");

    let test_cases = vec![
        (vec!["dup", "*"], "Square"),
        (vec!["dup", "dup", "*", "swap", "*"], "Cube"),
        (vec!["2dup", "+", "rot", "+"], "Sum three"),
    ];

    for (words, name) in test_cases {
        let words_ref: Vec<&str> = words.iter().map(|s| s.as_ref()).collect();
        group.bench_with_input(BenchmarkId::new("compose", name), &words_ref, |b, words| {
            b.iter(|| {
                let result = api.compose(black_box(words));
                black_box(result)
            });
        });
    }

    group.finish();
}

fn bench_throughput(c: &mut Criterion) {
    let api = InferenceAPI::new();

    c.bench_function("throughput_1000_inferences", |b| {
        b.iter(|| {
            for _ in 0..1000 {
                let _ = api.infer(black_box("dup * swap +"));
            }
        });
    });
}

criterion_group!(
    benches,
    bench_simple_operations,
    bench_complex_compositions,
    bench_verify_effect,
    bench_composition,
    bench_throughput
);
criterion_main!(benches);
