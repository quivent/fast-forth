/// Sieve of Eratosthenes benchmark
///
/// Classic algorithm for finding prime numbers
/// Good test of:
/// - Loops
/// - Array access
/// - Conditionals

use fast_forth::ForthEngine;

/// Reference implementation in Rust
pub fn sieve_rust(limit: usize) -> Vec<usize> {
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

/// Forth implementation (to be executed)
pub const SIEVE_FORTH: &str = r#"
\ Sieve of Eratosthenes
\ Usage: limit SIEVE
\ Returns count of primes found

: SIEVE ( limit -- count )
    DUP 2 + CELLS ALLOCATE DROP  \ Allocate array
    DUP 0 FILL                    \ Initialize to 0

    \ Mark composites
    2 OVER SQRT 1+ 0 DO
        I OVER + C@ 0= IF
            I DUP * OVER SWAP DO
                I OVER + 1 SWAP C!
            I +LOOP
        THEN
    LOOP

    \ Count primes
    0 SWAP
    2 OVER 0 DO
        I OVER + C@ 0= IF
            SWAP 1+ SWAP
        THEN
    LOOP
    NIP
;
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sieve_rust() {
        let primes = sieve_rust(30);
        assert_eq!(primes, vec![2, 3, 5, 7, 11, 13, 17, 19, 23, 29]);
    }

    #[test]
    fn test_sieve_rust_small() {
        let primes = sieve_rust(10);
        assert_eq!(primes, vec![2, 3, 5, 7]);
    }

    #[test]
    fn test_sieve_rust_100() {
        let primes = sieve_rust(100);
        assert_eq!(primes.len(), 25); // There are 25 primes under 100
    }

    #[test]
    fn test_sieve_rust_1000() {
        let primes = sieve_rust(1000);
        assert_eq!(primes.len(), 168); // There are 168 primes under 1000
    }

    #[test]
    fn test_sieve_forth_parsing() {
        // TODO: Test that Forth implementation parses correctly
        let _engine = ForthEngine::new();
        // engine.eval(SIEVE_FORTH).unwrap();
    }

    #[test]
    fn test_sieve_forth_execution() {
        // TODO: Test Forth implementation produces correct results
        // let mut engine = ForthEngine::new();
        // engine.eval(SIEVE_FORTH).unwrap();
        // engine.eval("30 SIEVE").unwrap();
        // assert_eq!(engine.stack(), &[10]); // 10 primes under 30
    }

    #[test]
    fn bench_sieve_rust() {
        let (primes, duration) = super::super::measure(|| sieve_rust(10000));
        println!("Rust sieve(10000): {} primes in {:?}", primes.len(), duration);
        assert_eq!(primes.len(), 1229); // There are 1229 primes under 10000
    }
}
