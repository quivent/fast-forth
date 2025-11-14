//! Comprehensive benchmarks for FastForth optimizer
//!
//! Demonstrates the performance improvements from each optimization pass.

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use fastforth_optimizer::*;

/// Benchmark constant folding pass
fn bench_constant_folding(c: &mut Criterion) {
    let mut group = c.benchmark_group("constant_folding");

    // Simple arithmetic
    let simple = ForthIR::parse("2 3 + 4 *").unwrap();
    group.bench_function("simple_arithmetic", |b| {
        let folder = ConstantFolder::new();
        b.iter(|| {
            folder.fold(black_box(&simple)).unwrap()
        });
    });

    // Complex expression
    let complex = ForthIR::parse("10 20 + 30 40 + * 2 /").unwrap();
    group.bench_function("complex_expression", |b| {
        let folder = ConstantFolder::new();
        b.iter(|| {
            folder.fold(black_box(&complex)).unwrap()
        });
    });

    // Mixed constants and operations
    let mixed = ForthIR::parse("5 dup + 10 * 2 / 1 +").unwrap();
    group.bench_function("mixed_operations", |b| {
        let folder = ConstantFolder::new();
        b.iter(|| {
            folder.fold(black_box(&mixed)).unwrap()
        });
    });

    group.finish();
}

/// Benchmark superinstruction recognition
fn bench_superinstructions(c: &mut Criterion) {
    let mut group = c.benchmark_group("superinstructions");

    let patterns = vec![
        ("dup_add", "5 dup +"),
        ("dup_mul", "7 dup *"),
        ("inc_one", "10 1 +"),
        ("dec_one", "10 1 -"),
        ("swap_swap", "1 2 swap swap +"),
        ("multiple_patterns", "5 dup + 1 + dup * 2 /"),
    ];

    for (name, code) in patterns {
        let ir = ForthIR::parse(code).unwrap();
        group.bench_with_input(BenchmarkId::from_parameter(name), &ir, |b, ir| {
            let optimizer = SuperinstructionOptimizer::new();
            b.iter(|| {
                optimizer.recognize(black_box(ir)).unwrap()
            });
        });
    }

    group.finish();
}

/// Benchmark stack caching
fn bench_stack_caching(c: &mut Criterion) {
    let mut group = c.benchmark_group("stack_caching");

    for cache_size in [1, 2, 3, 4].iter() {
        let ir = ForthIR::parse("1 2 3 4 5 + + + +").unwrap();
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("cache_{}", cache_size)),
            cache_size,
            |b, &size| {
                let optimizer = StackCacheOptimizer::new(size);
                b.iter(|| {
                    optimizer.optimize(black_box(&ir)).unwrap()
                });
            },
        );
    }

    group.finish();
}

/// Benchmark dead code elimination
fn bench_dead_code_elimination(c: &mut Criterion) {
    let mut group = c.benchmark_group("dead_code_elimination");

    let cases = vec![
        ("dup_drop", "5 dup drop"),
        ("swap_swap", "1 2 swap swap"),
        ("unused_computation", "1 2 + drop 5"),
        ("complex_dead", "1 2 3 dup drop swap swap over drop drop"),
    ];

    for (name, code) in cases {
        let ir = ForthIR::parse(code).unwrap();
        group.bench_with_input(BenchmarkId::from_parameter(name), &ir, |b, ir| {
            let eliminator = DeadCodeEliminator::new();
            b.iter(|| {
                eliminator.eliminate(black_box(ir)).unwrap()
            });
        });
    }

    group.finish();
}

/// Benchmark inlining
fn bench_inlining(c: &mut Criterion) {
    let mut group = c.benchmark_group("inlining");

    // Create IR with small word definitions
    let mut ir = ForthIR::new();

    let square = WordDef::new(
        "square".to_string(),
        vec![Instruction::Dup, Instruction::Mul],
    );
    ir.add_word(square);

    let cube = WordDef::new(
        "cube".to_string(),
        vec![
            Instruction::Dup,
            Instruction::Call("square".to_string()),
            Instruction::Mul,
        ],
    );
    ir.add_word(cube);

    ir.main = vec![
        Instruction::Literal(5),
        Instruction::Call("cube".to_string()),
    ];

    for level in [OptimizationLevel::Basic, OptimizationLevel::Standard, OptimizationLevel::Aggressive].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{:?}", level)),
            level,
            |b, &level| {
                let optimizer = InlineOptimizer::new(level);
                b.iter(|| {
                    optimizer.inline(black_box(&ir)).unwrap()
                });
            },
        );
    }

    group.finish();
}

/// Benchmark full optimization pipeline
fn bench_full_pipeline(c: &mut Criterion) {
    let mut group = c.benchmark_group("full_pipeline");

    // Real-world-like Forth code
    let code = "
        : square dup * ;
        : quad square square ;
        5 quad 1 + 2 /
    ";

    let ir = ForthIR::parse(code).unwrap();

    for level in [
        OptimizationLevel::None,
        OptimizationLevel::Basic,
        OptimizationLevel::Standard,
        OptimizationLevel::Aggressive,
    ]
    .iter()
    {
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{:?}", level)),
            level,
            |b, &level| {
                let optimizer = Optimizer::new(level);
                b.iter(|| {
                    optimizer.optimize(black_box(ir.clone())).unwrap()
                });
            },
        );
    }

    group.finish();
}

/// Benchmark optimization until fixpoint
fn bench_fixpoint_optimization(c: &mut Criterion) {
    let mut group = c.benchmark_group("fixpoint_optimization");

    let code = "1 2 dup + swap swap 3 4 dup drop + *";
    let ir = ForthIR::parse(code).unwrap();

    group.bench_function("aggressive", |b| {
        let optimizer = Optimizer::new(OptimizationLevel::Aggressive);
        b.iter(|| {
            optimizer.optimize_until_fixpoint(black_box(ir.clone())).unwrap()
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_constant_folding,
    bench_superinstructions,
    bench_stack_caching,
    bench_dead_code_elimination,
    bench_inlining,
    bench_full_pipeline,
    bench_fixpoint_optimization,
);

criterion_main!(benches);
