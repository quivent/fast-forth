//! Benchmarks for type specialization performance

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use fastforth_optimizer::{
    ConcreteType, ForthIR, Instruction, OptimizationLevel, Optimizer,
    TypeInferenceResults, TypeSignature, TypeSpecializer, WordDef,
};

/// Benchmark: Basic type specialization overhead
fn bench_specialization_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("specialization_overhead");

    for word_count in [10, 50, 100].iter() {
        let mut ir = ForthIR::new();

        // Create multiple words
        for i in 0..*word_count {
            let word = WordDef::new(
                format!("word_{}", i),
                vec![Instruction::Dup, Instruction::Add],
            );
            ir.add_word(word);
        }

        let mut type_info = TypeInferenceResults::new();
        for i in 0..*word_count {
            type_info.add_word_signature(
                format!("word_{}", i),
                TypeSignature::new(vec![ConcreteType::Int], vec![ConcreteType::Int]),
            );
        }

        group.bench_with_input(
            BenchmarkId::from_parameter(word_count),
            word_count,
            |b, _| {
                b.iter(|| {
                    let mut specializer = TypeSpecializer::new();
                    let mut ir_clone = ir.clone();
                    specializer.specialize(&mut ir_clone, &type_info).unwrap()
                });
            },
        );
    }

    group.finish();
}

/// Benchmark: Polymorphic word analysis
fn bench_polymorphic_analysis(c: &mut Criterion) {
    let mut group = c.benchmark_group("polymorphic_analysis");

    for type_count in [2, 4, 6].iter() {
        let mut ir = ForthIR::new();
        let word = WordDef::new(
            "polymorphic".to_string(),
            vec![Instruction::Dup, Instruction::Swap, Instruction::Over],
        );
        ir.add_word(word);

        let mut type_info = TypeInferenceResults::new();
        let types = vec![
            ConcreteType::Int,
            ConcreteType::Float,
            ConcreteType::Addr,
            ConcreteType::Bool,
            ConcreteType::Char,
            ConcreteType::String,
        ];

        for i in 0..*type_count {
            type_info.add_word_signature(
                "polymorphic".to_string(),
                TypeSignature::new(
                    vec![types[i % types.len()].clone()],
                    vec![types[i % types.len()].clone()],
                ),
            );
        }

        group.bench_with_input(
            BenchmarkId::from_parameter(type_count),
            type_count,
            |b, _| {
                b.iter(|| {
                    let mut specializer = TypeSpecializer::new();
                    let mut ir_clone = ir.clone();
                    specializer.specialize(&mut ir_clone, &type_info).unwrap()
                });
            },
        );
    }

    group.finish();
}

/// Benchmark: Full optimization pipeline with vs without type specialization
fn bench_optimization_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("optimization_comparison");

    // Create a realistic IR
    let mut ir = ForthIR::new();

    // Square function
    let square = WordDef::new(
        "square".to_string(),
        vec![Instruction::Dup, Instruction::Mul],
    );
    ir.add_word(square);

    // Sum of squares
    let sum_squares = WordDef::new(
        "sum_squares".to_string(),
        vec![
            Instruction::Call("square".to_string()),
            Instruction::Swap,
            Instruction::Call("square".to_string()),
            Instruction::Add,
        ],
    );
    ir.add_word(sum_squares);

    // Main computation
    ir.main = vec![
        Instruction::Literal(3),
        Instruction::Literal(4),
        Instruction::Call("sum_squares".to_string()),
    ];

    let mut type_info = TypeInferenceResults::new();
    type_info.add_word_signature(
        "square".to_string(),
        TypeSignature::new(vec![ConcreteType::Int], vec![ConcreteType::Int]),
    );
    type_info.add_word_signature(
        "sum_squares".to_string(),
        TypeSignature::new(
            vec![ConcreteType::Int, ConcreteType::Int],
            vec![ConcreteType::Int],
        ),
    );

    // Without type specialization
    group.bench_function("without_specialization", |b| {
        b.iter(|| {
            let optimizer = Optimizer::new(OptimizationLevel::Aggressive);
            optimizer.optimize(black_box(ir.clone())).unwrap()
        });
    });

    // With type specialization
    group.bench_function("with_specialization", |b| {
        b.iter(|| {
            let mut optimizer = Optimizer::new(OptimizationLevel::Aggressive);
            optimizer
                .optimize_with_types(black_box(ir.clone()), &type_info)
                .unwrap()
        });
    });

    group.finish();
}

/// Benchmark: Call site resolution
fn bench_call_site_resolution(c: &mut Criterion) {
    let mut group = c.benchmark_group("call_site_resolution");

    for call_count in [10, 100, 1000].iter() {
        let mut ir = ForthIR::new();

        let word = WordDef::new(
            "test".to_string(),
            vec![Instruction::Dup],
        );
        ir.add_word(word);

        // Create many call sites
        for _ in 0..*call_count {
            ir.main.push(Instruction::Call("test".to_string()));
        }

        let mut type_info = TypeInferenceResults::new();
        type_info.add_word_signature(
            "test".to_string(),
            TypeSignature::new(vec![ConcreteType::Int], vec![ConcreteType::Int]),
        );

        for i in 0..*call_count {
            type_info.add_call_site(
                i,
                TypeSignature::new(vec![ConcreteType::Int], vec![ConcreteType::Int]),
            );
        }

        group.bench_with_input(
            BenchmarkId::from_parameter(call_count),
            call_count,
            |b, _| {
                b.iter(|| {
                    let mut specializer = TypeSpecializer::new();
                    let mut ir_clone = ir.clone();
                    specializer.specialize(&mut ir_clone, &type_info).unwrap()
                });
            },
        );
    }

    group.finish();
}

/// Benchmark: Type signature mangling
fn bench_name_mangling(c: &mut Criterion) {
    let signatures = vec![
        TypeSignature::new(vec![ConcreteType::Int], vec![ConcreteType::Int]),
        TypeSignature::new(
            vec![ConcreteType::Int, ConcreteType::Float],
            vec![ConcreteType::Bool],
        ),
        TypeSignature::new(
            vec![ConcreteType::Addr, ConcreteType::Addr, ConcreteType::Int],
            vec![ConcreteType::Addr],
        ),
    ];

    c.bench_function("name_mangling", |b| {
        b.iter(|| {
            for sig in &signatures {
                black_box(sig.mangle_name("test_word"));
            }
        });
    });
}

/// Benchmark: Specialization stats calculation
fn bench_stats_calculation(c: &mut Criterion) {
    c.bench_function("stats_calculation", |b| {
        b.iter(|| {
            let mut stats = fastforth_optimizer::SpecializationStats {
                words_analyzed: 100,
                polymorphic_words: 25,
                specializations_created: 50,
                call_sites_rewritten: 200,
                estimated_speedup_percent: 0.0,
            };
            stats.calculate_speedup();
            black_box(stats);
        });
    });
}

/// Benchmark: Complex polymorphic scenario
fn bench_complex_polymorphic(c: &mut Criterion) {
    let mut ir = ForthIR::new();

    // Create a complex polymorphic word
    let complex = WordDef::new(
        "complex".to_string(),
        vec![
            Instruction::Dup,
            Instruction::Over,
            Instruction::Swap,
            Instruction::Add,
            Instruction::Swap,
            Instruction::Mul,
        ],
    );
    ir.add_word(complex);

    let mut type_info = TypeInferenceResults::new();

    // Add multiple type signatures
    for concrete_type in &[ConcreteType::Int, ConcreteType::Float] {
        type_info.add_word_signature(
            "complex".to_string(),
            TypeSignature::new(
                vec![concrete_type.clone(), concrete_type.clone()],
                vec![concrete_type.clone()],
            ),
        );
    }

    c.bench_function("complex_polymorphic", |b| {
        b.iter(|| {
            let mut specializer = TypeSpecializer::new();
            let mut ir_clone = ir.clone();
            specializer.specialize(&mut ir_clone, &type_info).unwrap()
        });
    });
}

criterion_group!(
    benches,
    bench_specialization_overhead,
    bench_polymorphic_analysis,
    bench_optimization_comparison,
    bench_call_site_resolution,
    bench_name_mangling,
    bench_stats_calculation,
    bench_complex_polymorphic,
);

criterion_main!(benches);
