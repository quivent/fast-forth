//! Benchmarks for Fast Forth frontend

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use fastforth_frontend::*;

fn bench_lexer(c: &mut Criterion) {
    let source = r#"
        : square ( n -- n*n ) dup * ;
        : cube ( n -- n^3 ) dup square * ;
    "#;

    c.bench_function("lexer_simple", |b| {
        b.iter(|| {
            let mut lexer = lexer::Lexer::new(black_box(source));
            lexer.tokenize().unwrap()
        })
    });
}

fn bench_parser(c: &mut Criterion) {
    let samples = vec![
        ("simple", ": double 2 * ;"),
        ("with_stack_effect", ": square ( n -- n*n ) dup * ;"),
        (
            "control_structure",
            ": abs ( n -- |n| ) dup 0 < IF negate THEN ;",
        ),
        (
            "loop",
            ": countdown ( n -- ) BEGIN dup . 1 - dup 0 = UNTIL drop ;",
        ),
    ];

    let mut group = c.benchmark_group("parser");
    for (name, source) in samples {
        group.bench_with_input(BenchmarkId::from_parameter(name), source, |b, s| {
            b.iter(|| parse_program(black_box(s)).unwrap())
        });
    }
    group.finish();
}

fn bench_stack_inference(c: &mut Criterion) {
    let programs = vec![
        ("simple", ": double 2 * ;"),
        ("complex", ": sum-of-squares ( a b -- a^2+b^2 ) square swap square + ;"),
    ];

    let mut group = c.benchmark_group("stack_inference");
    for (name, source) in programs {
        let program = parse_program(source).unwrap();
        group.bench_with_input(BenchmarkId::from_parameter(name), &program, |b, prog| {
            b.iter(|| {
                let mut inference = stack_effects::StackEffectInference::new();
                inference.analyze_program(black_box(prog)).unwrap()
            })
        });
    }
    group.finish();
}

fn bench_type_inference(c: &mut Criterion) {
    let programs = vec![
        ("simple", ": add-one 1 + ;"),
        ("polymorphic", ": identity dup drop ;"),
    ];

    let mut group = c.benchmark_group("type_inference");
    for (name, source) in programs {
        let program = parse_program(source).unwrap();
        group.bench_with_input(BenchmarkId::from_parameter(name), &program, |b, prog| {
            b.iter(|| {
                let mut inference = type_inference::TypeInference::new();
                inference.analyze_program(black_box(prog)).unwrap()
            })
        });
    }
    group.finish();
}

fn bench_ssa_conversion(c: &mut Criterion) {
    let programs = vec![
        ("simple", ": double 2 * ;"),
        ("stack_ops", ": swap-and-add swap + ;"),
        ("control_flow", ": max ( a b -- max ) 2dup > IF drop ELSE swap drop THEN ;"),
    ];

    let mut group = c.benchmark_group("ssa_conversion");
    for (name, source) in programs {
        let program = parse_program(source).unwrap();
        group.bench_with_input(BenchmarkId::from_parameter(name), &program, |b, prog| {
            b.iter(|| ssa::convert_to_ssa(black_box(prog)).unwrap())
        });
    }
    group.finish();
}

fn bench_semantic_analysis(c: &mut Criterion) {
    let programs = vec![
        ("simple", ": double 2 * ;"),
        (
            "multiple_defs",
            r#"
            : double 2 * ;
            : triple 3 * ;
            : quadruple double double ;
            "#,
        ),
    ];

    let mut group = c.benchmark_group("semantic_analysis");
    for (name, source) in programs {
        let program = parse_program(source).unwrap();
        group.bench_with_input(BenchmarkId::from_parameter(name), &program, |b, prog| {
            b.iter(|| semantic::analyze(black_box(prog)).unwrap())
        });
    }
    group.finish();
}

fn bench_full_pipeline(c: &mut Criterion) {
    let source = r#"
        : square ( n -- n*n ) dup * ;
        : sum-of-squares ( a b -- a^2+b^2 )
            square swap square + ;
        : distance ( x1 y1 x2 y2 -- dist )
            rot - swap rot - sum-of-squares ;
    "#;

    c.bench_function("full_pipeline", |b| {
        b.iter(|| {
            let program = parse_program(black_box(source)).unwrap();
            semantic::analyze(&program).unwrap();
            ssa::convert_to_ssa(&program).unwrap()
        })
    });
}

fn bench_large_program(c: &mut Criterion) {
    let mut source = String::new();
    for i in 0..100 {
        source.push_str(&format!(": func{} {} + ;\n", i, i));
    }

    c.bench_function("large_program_100_defs", |b| {
        b.iter(|| {
            let program = parse_program(black_box(&source)).unwrap();
            semantic::analyze(&program).unwrap()
        })
    });
}

fn bench_deep_nesting(c: &mut Criterion) {
    let source = r#"
        : nested ( n -- )
            dup 0 > IF
                dup 1 - nested
            THEN
            drop ;
    "#;

    c.bench_function("deep_nesting", |b| {
        b.iter(|| {
            let program = parse_program(black_box(source)).unwrap();
            semantic::analyze(&program).unwrap();
            ssa::convert_to_ssa(&program).unwrap()
        })
    });
}

criterion_group!(
    benches,
    bench_lexer,
    bench_parser,
    bench_stack_inference,
    bench_type_inference,
    bench_ssa_conversion,
    bench_semantic_analysis,
    bench_full_pipeline,
    bench_large_program,
    bench_deep_nesting
);

criterion_main!(benches);
