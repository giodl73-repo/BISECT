use rmath_core::{invert, mat_mul, mat_mul_vec, transpose, DenseMatrix};

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
