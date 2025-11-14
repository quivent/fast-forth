/// Comparison benchmarks: Fast Forth vs GForth vs C
///
/// Compare performance across implementations

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::process::{Command, Stdio};
use std::io::Write;

fn bench_fibonacci_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("fibonacci_comparison");

    // Fast Forth (when implemented)
    group.bench_function("fast_forth_fib_20", |b| {
        b.iter(|| {
            // TODO: Implement when Forth engine supports this
            black_box(6765)
        });
    });

    // GForth (if available)
    if gforth_available() {
        group.bench_function("gforth_fib_20", |b| {
            b.iter(|| {
                run_gforth_code(black_box("
                    : FIB ( n -- fib[n] )
                        DUP 2 < IF EXIT THEN
                        DUP 1- RECURSE
                        SWAP 2 - RECURSE
                        + ;
                    20 FIB .
                    bye
                "));
            });
        });
    }

    group.finish();
}

fn bench_sieve_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("sieve_comparison");

    // Rust reference implementation
    group.bench_function("rust_sieve_1000", |b| {
        b.iter(|| {
            sieve_of_eratosthenes(black_box(1000))
        });
    });

    group.finish();
}

// Helper functions
fn gforth_available() -> bool {
    Command::new("gforth")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok()
}

fn run_gforth_code(code: &str) {
    let mut child = Command::new("gforth")
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to spawn gforth");

    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(code.as_bytes());
    }

    let _ = child.wait();
}

fn sieve_of_eratosthenes(limit: usize) -> Vec<usize> {
    let mut is_prime = vec![true; limit + 1];
    is_prime[0] = false;
    is_prime[1] = false;

    for i in 2..=((limit as f64).sqrt() as usize) {
        if is_prime[i] {
            for j in (i * i..=limit).step_by(i) {
                is_prime[j] = false;
            }
        }
    }

    is_prime.iter()
        .enumerate()
        .filter_map(|(i, &prime)| if prime { Some(i) } else { None })
        .collect()
}

criterion_group!(benches, bench_fibonacci_comparison, bench_sieve_comparison);
criterion_main!(benches);
