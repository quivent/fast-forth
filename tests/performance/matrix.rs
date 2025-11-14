/// Matrix multiplication benchmark
///
/// Tests array operations and nested loops

/// Matrix multiplication in Rust
pub fn matrix_multiply_rust(
    a: &[Vec<i64>],
    b: &[Vec<i64>],
) -> Vec<Vec<i64>> {
    let rows_a = a.len();
    let cols_a = a[0].len();
    let cols_b = b[0].len();

    let mut result = vec![vec![0; cols_b]; rows_a];

    for i in 0..rows_a {
        for j in 0..cols_b {
            for k in 0..cols_a {
                result[i][j] += a[i][k] * b[k][j];
            }
        }
    }

    result
}

/// Forth implementation
pub const MATRIX_MULTIPLY_FORTH: &str = r#"
\ Matrix multiplication
\ Matrices stored as flat arrays with dimensions
\ Usage: addr1 rows1 cols1 addr2 rows2 cols2 addr-result MATRIX-MULT

: MATRIX-MULT ( addr1 r1 c1 addr2 r2 c2 addr-res -- )
    \ TODO: Implement matrix multiplication
;
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matrix_multiply_2x2() {
        let a = vec![
            vec![1, 2],
            vec![3, 4],
        ];
        let b = vec![
            vec![5, 6],
            vec![7, 8],
        ];
        let expected = vec![
            vec![19, 22],
            vec![43, 50],
        ];
        assert_eq!(matrix_multiply_rust(&a, &b), expected);
    }

    #[test]
    fn test_matrix_multiply_3x3() {
        let a = vec![
            vec![1, 2, 3],
            vec![4, 5, 6],
            vec![7, 8, 9],
        ];
        let b = vec![
            vec![9, 8, 7],
            vec![6, 5, 4],
            vec![3, 2, 1],
        ];
        let result = matrix_multiply_rust(&a, &b);

        // Verify dimensions
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].len(), 3);

        // Check first row
        assert_eq!(result[0][0], 30);
        assert_eq!(result[0][1], 24);
        assert_eq!(result[0][2], 18);
    }

    #[test]
    fn test_matrix_multiply_identity() {
        let a = vec![
            vec![1, 2],
            vec![3, 4],
        ];
        let identity = vec![
            vec![1, 0],
            vec![0, 1],
        ];
        assert_eq!(matrix_multiply_rust(&a, &identity), a);
    }

    #[test]
    fn bench_matrix_multiply_small() {
        let size = 10;
        let a: Vec<Vec<i64>> = (0..size)
            .map(|i| (0..size).map(|j| (i * size + j) as i64).collect())
            .collect();
        let b = a.clone();

        let (result, duration) = super::super::measure(|| {
            matrix_multiply_rust(&a, &b)
        });

        println!("Rust matrix multiply {}x{}: {:?}", size, size, duration);
        assert_eq!(result.len(), size);
    }

    #[test]
    fn bench_matrix_multiply_medium() {
        let size = 50;
        let a: Vec<Vec<i64>> = (0..size)
            .map(|i| (0..size).map(|j| (i * size + j) as i64).collect())
            .collect();
        let b = a.clone();

        let (result, duration) = super::super::measure(|| {
            matrix_multiply_rust(&a, &b)
        });

        println!("Rust matrix multiply {}x{}: {:?}", size, size, duration);
        assert_eq!(result.len(), size);
    }
}
