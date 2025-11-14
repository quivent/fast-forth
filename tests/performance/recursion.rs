/// Recursion benchmarks
///
/// Tests deep recursion and stack management

use fast_forth::ForthEngine;

/// Ackermann function (very recursive!)
pub fn ackermann_rust(m: u64, n: u64) -> u64 {
    match (m, n) {
        (0, n) => n + 1,
        (m, 0) => ackermann_rust(m - 1, 1),
        (m, n) => ackermann_rust(m - 1, ackermann_rust(m, n - 1)),
    }
}

/// Factorial (simple recursion)
pub fn factorial_rust(n: u64) -> u64 {
    match n {
        0 | 1 => 1,
        n => n * factorial_rust(n - 1),
    }
}

/// Tower of Hanoi (recursive algorithm)
pub fn hanoi_moves_rust(n: u64) -> u64 {
    match n {
        0 => 0,
        1 => 1,
        n => 2 * hanoi_moves_rust(n - 1) + 1,
    }
}

/// Forth implementations
pub const ACKERMANN_FORTH: &str = r#"
\ Ackermann function
: ACKERMANN ( m n -- result )
    OVER 0= IF
        NIP 1+
    ELSE
        DUP 0= IF
            DROP 1- 1 RECURSE
        ELSE
            OVER 1- -ROT 1- RECURSE RECURSE
        THEN
    THEN
;
"#;

pub const FACTORIAL_FORTH: &str = r#"
\ Factorial
: FACTORIAL ( n -- n! )
    DUP 2 < IF
        DROP 1
    ELSE
        DUP 1- RECURSE *
    THEN
;
"#;

pub const HANOI_FORTH: &str = r#"
\ Tower of Hanoi move count
: HANOI-MOVES ( n -- moves )
    DUP 1 <= IF
        EXIT
    ELSE
        1- RECURSE 2 * 1+
    THEN
;
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ackermann() {
        assert_eq!(ackermann_rust(0, 0), 1);
        assert_eq!(ackermann_rust(0, 5), 6);
        assert_eq!(ackermann_rust(1, 0), 2);
        assert_eq!(ackermann_rust(1, 5), 7);
        assert_eq!(ackermann_rust(2, 5), 13);
        assert_eq!(ackermann_rust(3, 3), 61);
        // Don't test larger values - exponential growth!
    }

    #[test]
    fn test_factorial() {
        assert_eq!(factorial_rust(0), 1);
        assert_eq!(factorial_rust(1), 1);
        assert_eq!(factorial_rust(5), 120);
        assert_eq!(factorial_rust(10), 3628800);
    }

    #[test]
    fn test_hanoi() {
        assert_eq!(hanoi_moves_rust(1), 1);
        assert_eq!(hanoi_moves_rust(2), 3);
        assert_eq!(hanoi_moves_rust(3), 7);
        assert_eq!(hanoi_moves_rust(4), 15);
        assert_eq!(hanoi_moves_rust(10), 1023);
    }

    #[test]
    fn bench_factorial() {
        let (result, duration) = super::super::measure(|| factorial_rust(20));
        println!("Rust factorial(20): {} in {:?}", result, duration);
        assert_eq!(result, 2432902008176640000);
    }

    #[test]
    fn bench_hanoi() {
        let (result, duration) = super::super::measure(|| hanoi_moves_rust(25));
        println!("Rust hanoi(25): {} in {:?}", result, duration);
        assert_eq!(result, 33554431);
    }

    #[test]
    fn bench_ackermann() {
        let (result, duration) = super::super::measure(|| ackermann_rust(3, 5));
        println!("Rust ackermann(3,5): {} in {:?}", result, duration);
        assert_eq!(result, 253);
    }
}
