/// Criterion benchmarks for Fast Forth
///
/// Comprehensive performance benchmarking

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use fast_forth::ForthEngine;

fn bench_simple_arithmetic(c: &mut Criterion) {
    let mut group = c.benchmark_group("arithmetic");

    group.bench_function("addition", |b| {
        let mut engine = ForthEngine::new();
        b.iter(|| {
            engine.clear_stack();
            engine.eval(black_box("5 10 +")).unwrap();
        });
    });

    group.bench_function("multiplication", |b| {
        let mut engine = ForthEngine::new();
        b.iter(|| {
            engine.clear_stack();
            engine.eval(black_box("5 10 *")).unwrap();
        });
    });

    group.bench_function("complex_expression", |b| {
        let mut engine = ForthEngine::new();
        b.iter(|| {
            engine.clear_stack();
            engine.eval(black_box("2 3 + 4 5 + *")).unwrap();
        });
    });

    group.finish();
}

fn bench_stack_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("stack");

    group.bench_function("dup", |b| {
        let mut engine = ForthEngine::new();
        engine.eval("5").unwrap();
        b.iter(|| {
            engine.eval(black_box("DUP DROP")).unwrap();
        });
    });

    group.bench_function("swap", |b| {
        let mut engine = ForthEngine::new();
        engine.eval("5 10").unwrap();
        b.iter(|| {
            engine.eval(black_box("SWAP SWAP")).unwrap();
        });
    });

    group.finish();
}

fn bench_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("parsing");

    let expressions = vec![
        "5 10 +",
        "1 2 3 4 5 + + + +",
        "10 DUP * DUP * DUP *",
    ];

    for expr in expressions {
        group.bench_with_input(BenchmarkId::from_parameter(expr), expr, |b, expr| {
            b.iter(|| {
                let mut engine = ForthEngine::new();
                engine.eval(black_box(expr)).unwrap();
            });
        });
    }

    group.finish();
}

criterion_group!(benches, bench_simple_arithmetic, bench_stack_operations, bench_parsing);
criterion_main!(benches);
