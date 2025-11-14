/// Performance test module
///
/// Contains reference implementations of benchmark algorithms

pub mod sieve;
pub mod fibonacci;
pub mod matrix;
pub mod recursion;

/// Helper to measure execution time
pub fn measure<F: FnOnce() -> R, R>(f: F) -> (R, std::time::Duration) {
    let start = std::time::Instant::now();
    let result = f();
    let duration = start.elapsed();
    (result, duration)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_measure() {
        let (result, duration) = measure(|| {
            std::thread::sleep(std::time::Duration::from_millis(10));
            42
        });
        assert_eq!(result, 42);
        assert!(duration.as_millis() >= 10);
    }
}
