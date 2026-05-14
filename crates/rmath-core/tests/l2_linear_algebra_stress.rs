use rmath_core::{invert, mat_mul, DenseMatrix};

#[test]
#[ignore = "L2 numeric stress: Hilbert-like inverse smoke test"]
fn l2_hilbert_like_inverse_multiplies_to_identity() {
    let n = 8;
    let data: Vec<f64> = (0..n)
        .flat_map(|i| (0..n).map(move |j| 1.0 / ((i + j + 1) as f64)))
        .collect();
    let matrix = DenseMatrix::from_row_major(n, n, data).unwrap();

    let inv = invert(&matrix).unwrap();
    let identity = mat_mul(&matrix, &inv).unwrap();

    for i in 0..n {
        for j in 0..n {
            let expected = if i == j { 1.0 } else { 0.0 };
            assert!((identity.at(i, j) - expected).abs() < 1e-6);
        }
    }
}
