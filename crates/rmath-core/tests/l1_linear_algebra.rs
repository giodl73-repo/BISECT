use rmath_core::{
    center_in_place, dot, invert, l2_norm, mat_mul, mat_mul_vec, normalize_centered, transpose,
    DenseMatrix, LinearAlgebraError,
};

#[test]
fn l1_solve_wls_normal_equation_fixture() {
    let x =
        DenseMatrix::from_row_major(4, 2, vec![1.0, 0.0, 1.0, 1.0, 1.0, 2.0, 1.0, 3.0]).unwrap();
    let y = [1.0, 3.0, 5.0, 7.0];
    let xt = transpose(&x).unwrap();
    let xtx = mat_mul(&xt, &x).unwrap();
    let xty = mat_mul_vec(&xt, &y).unwrap();
    let beta = mat_mul_vec(&invert(&xtx).unwrap(), &xty).unwrap();

    assert!((beta[0] - 1.0).abs() < 1e-12);
    assert!((beta[1] - 2.0).abs() < 1e-12);
}

#[test]
fn l1_partial_pivoting_handles_zero_initial_pivot() {
    let matrix = DenseMatrix::from_row_major(2, 2, vec![0.0, 2.0, 1.0, 3.0]).unwrap();

    let inv = invert(&matrix).unwrap();
    let identity = mat_mul(&matrix, &inv).unwrap();

    assert!((identity.at(0, 0) - 1.0).abs() < 1e-12);
    assert!(identity.at(0, 1).abs() < 1e-12);
    assert!(identity.at(1, 0).abs() < 1e-12);
    assert!((identity.at(1, 1) - 1.0).abs() < 1e-12);
}

#[test]
fn l1_centered_normalization_matches_spectral_fixture() {
    let vector = normalize_centered(vec![-2.5, -1.5, -0.5, 0.5, 1.5, 2.5], 0.0).unwrap();

    assert!(vector.iter().sum::<f64>().abs() < 1e-12);
    assert!((l2_norm(&vector).unwrap() - 1.0).abs() < 1e-12);
    assert_eq!(vector[0].total_cmp(&vector[1]), std::cmp::Ordering::Less);
    assert_eq!(vector[4].total_cmp(&vector[5]), std::cmp::Ordering::Less);
}

#[test]
fn l1_projection_out_of_ones_matches_fiedler_fixture() {
    let mut vector = vec![
        (1.0_f64).sin(),
        (2.0_f64).sin(),
        (3.0_f64).sin(),
        (4.0_f64).sin(),
    ];

    center_in_place(&mut vector).unwrap();
    let ones = vec![1.0; vector.len()];

    assert!(dot(&vector, &ones).unwrap().abs() < 1e-12);
}

#[test]
fn l1_non_finite_vector_input_is_rejected() {
    match l2_norm(&[1.0, f64::NAN]) {
        Err(LinearAlgebraError::NonFiniteValue { row, col, value }) => {
            assert_eq!((row, col), (1, 0));
            assert!(value.is_nan());
        }
        other => panic!("expected non-finite vector error, got {other:?}"),
    }
}
