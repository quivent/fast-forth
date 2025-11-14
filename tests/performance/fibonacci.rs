/// Fibonacci benchmark
///
/// Tests both iterative and recursive implementations
/// Good test of:
/// - Recursion
/// - Stack management
/// - Arithmetic operations

use fast_forth::ForthEngine;

/// Iterative Fibonacci in Rust
pub fn fib_iterative_rust(n: u64) -> u64 {
    if n <= 1 {
        return n;
    }
    let mut a = 0;
    let mut b = 1;
    for _ in 2..=n {
        let temp = a + b;
        a = b;
        b = temp;
    }
    b
}

/// Recursive Fibonacci in Rust (slow for large n)
pub fn fib_recursive_rust(n: u64) -> u64 {
    match n {
        0 => 0,
        1 => 1,
        n => fib_recursive_rust(n - 1) + fib_recursive_rust(n - 2),
    }
}

/// Iterative Forth implementation
pub const FIB_ITERATIVE_FORTH: &str = r#"
\ Iterative Fibonacci
\ Usage: n FIB-ITER
\ Returns nth Fibonacci number

: FIB-ITER ( n -- fib[n] )
    DUP 2 < IF EXIT THEN
    0 1 ROT              \ a b n
    2 DO
        OVER OVER +      \ a b (a+b)
        ROT DROP         \ b (a+b)
    LOOP
    NIP
;
"#;

/// Recursive Forth implementation
pub const FIB_RECURSIVE_FORTH: &str = r#"
\ Recursive Fibonacci
\ Usage: n FIB-REC
\ Returns nth Fibonacci number (slow!)

: FIB-REC ( n -- fib[n] )
    DUP 2 < IF EXIT THEN
    DUP 1- RECURSE
    SWAP 2 - RECURSE
    +
;
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fib_iterative_rust() {
        assert_eq!(fib_iterative_rust(0), 0);
        assert_eq!(fib_iterative_rust(1), 1);
        assert_eq!(fib_iterative_rust(2), 1);
        assert_eq!(fib_iterative_rust(3), 2);
        assert_eq!(fib_iterative_rust(4), 3);
        assert_eq!(fib_iterative_rust(5), 5);
        assert_eq!(fib_iterative_rust(10), 55);
        assert_eq!(fib_iterative_rust(20), 6765);
    }

    #[test]
    fn test_fib_recursive_rust() {
        assert_eq!(fib_recursive_rust(0), 0);
        assert_eq!(fib_recursive_rust(1), 1);
        assert_eq!(fib_recursive_rust(2), 1);
        assert_eq!(fib_recursive_rust(10), 55);
        // Don't test large values - too slow
    }

    #[test]
    fn test_fib_forth_iterative() {
        // TODO: Test Forth iterative implementation
        // let mut engine = ForthEngine::new();
        // engine.eval(FIB_ITERATIVE_FORTH).unwrap();
        // engine.eval("10 FIB-ITER").unwrap();
        // assert_eq!(engine.stack(), &[55]);
    }

    #[test]
    fn test_fib_forth_recursive() {
        // TODO: Test Forth recursive implementation
        // let mut engine = ForthEngine::new();
        // engine.eval(FIB_RECURSIVE_FORTH).unwrap();
        // engine.eval("10 FIB-REC").unwrap();
        // assert_eq!(engine.stack(), &[55]);
    }

    #[test]
    fn bench_fib_iterative() {
        let (result, duration) = super::super::measure(|| fib_iterative_rust(40));
        println!("Rust iterative fib(40): {} in {:?}", result, duration);
        assert_eq!(result, 102334155);
    }

    #[test]
    fn bench_fib_recursive_small() {
        let (result, duration) = super::super::measure(|| fib_recursive_rust(20));
        println!("Rust recursive fib(20): {} in {:?}", result, duration);
        assert_eq!(result, 6765);
    }
}
