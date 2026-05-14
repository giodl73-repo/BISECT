use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq)]
pub enum LinearAlgebraError {
    #[error("[INPUT] matrix dimensions do not conform: left {left:?}, right {right:?}")]
    DimensionMismatch {
        left: (usize, usize),
        right: (usize, usize),
    },
    #[error("[INPUT] matrix must be square, got {rows}x{cols}")]
    NotSquare { rows: usize, cols: usize },
    #[error("[INPUT] matrix contains non-finite value {value} at ({row}, {col})")]
    NonFiniteValue { row: usize, col: usize, value: f64 },
    #[error("[NUMERIC] singular matrix")]
    Singular,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DenseMatrix {
    rows: usize,
    cols: usize,
    data: Vec<f64>,
}

impl DenseMatrix {
    pub fn zeros(rows: usize, cols: usize) -> Self {
        Self {
            rows,
            cols,
            data: vec![0.0; rows * cols],
        }
    }

    pub fn from_row_major(
        rows: usize,
        cols: usize,
        data: Vec<f64>,
    ) -> Result<Self, LinearAlgebraError> {
        if data.len() != rows * cols {
            return Err(LinearAlgebraError::DimensionMismatch {
                left: (1, data.len()),
                right: (rows, cols),
            });
        }
        let matrix = Self { rows, cols, data };
        validate_finite(&matrix)?;
        Ok(matrix)
    }

    pub fn identity(n: usize) -> Self {
        let mut out = Self::zeros(n, n);
        for i in 0..n {
            out.set(i, i, 1.0);
        }
        out
    }

    pub fn rows(&self) -> usize {
        self.rows
    }

    pub fn cols(&self) -> usize {
        self.cols
    }

    pub fn at(&self, row: usize, col: usize) -> f64 {
        self.data[row * self.cols + col]
    }

    pub fn set(&mut self, row: usize, col: usize, value: f64) {
        self.data[row * self.cols + col] = value;
    }
}

pub fn mat_mul(a: &DenseMatrix, b: &DenseMatrix) -> Result<DenseMatrix, LinearAlgebraError> {
    validate_finite(a)?;
    validate_finite(b)?;
    if a.cols != b.rows {
        return Err(LinearAlgebraError::DimensionMismatch {
            left: (a.rows, a.cols),
            right: (b.rows, b.cols),
        });
    }
    let mut out = DenseMatrix::zeros(a.rows, b.cols);
    for i in 0..a.rows {
        for k in 0..a.cols {
            let aik = a.at(i, k);
            if aik == 0.0 {
                continue;
            }
            for j in 0..b.cols {
                out.set(i, j, out.at(i, j) + aik * b.at(k, j));
            }
        }
    }
    Ok(out)
}

pub fn mat_mul_vec(a: &DenseMatrix, v: &[f64]) -> Result<Vec<f64>, LinearAlgebraError> {
    validate_finite(a)?;
    validate_vector(v)?;
    if a.cols != v.len() {
        return Err(LinearAlgebraError::DimensionMismatch {
            left: (a.rows, a.cols),
            right: (v.len(), 1),
        });
    }
    let mut out = vec![0.0; a.rows];
    for i in 0..a.rows {
        let mut sum = 0.0;
        for (j, value) in v.iter().enumerate().take(a.cols) {
            sum += a.at(i, j) * value;
        }
        out[i] = sum;
    }
    Ok(out)
}

pub fn transpose(a: &DenseMatrix) -> Result<DenseMatrix, LinearAlgebraError> {
    validate_finite(a)?;
    let mut out = DenseMatrix::zeros(a.cols, a.rows);
    for i in 0..a.rows {
        for j in 0..a.cols {
            out.set(j, i, a.at(i, j));
        }
    }
    Ok(out)
}

pub fn invert(m: &DenseMatrix) -> Result<DenseMatrix, LinearAlgebraError> {
    validate_finite(m)?;
    if m.rows != m.cols {
        return Err(LinearAlgebraError::NotSquare {
            rows: m.rows,
            cols: m.cols,
        });
    }
    let n = m.rows;
    let mut aug = DenseMatrix::zeros(n, 2 * n);
    for i in 0..n {
        for j in 0..n {
            aug.set(i, j, m.at(i, j));
        }
        aug.set(i, n + i, 1.0);
    }
    for c in 0..n {
        let mut pivot_row = c;
        let mut pivot_abs = aug.at(c, c).abs();
        for r in (c + 1)..n {
            let value = aug.at(r, c).abs();
            if value > pivot_abs {
                pivot_abs = value;
                pivot_row = r;
            }
        }
        if pivot_abs < 1e-12 {
            return Err(LinearAlgebraError::Singular);
        }
        if pivot_row != c {
            for j in 0..(2 * n) {
                let tmp = aug.at(c, j);
                aug.set(c, j, aug.at(pivot_row, j));
                aug.set(pivot_row, j, tmp);
            }
        }
        let pivot = aug.at(c, c);
        for j in 0..(2 * n) {
            aug.set(c, j, aug.at(c, j) / pivot);
        }
        for r in 0..n {
            if r == c {
                continue;
            }
            let factor = aug.at(r, c);
            if factor == 0.0 {
                continue;
            }
            for j in 0..(2 * n) {
                aug.set(r, j, aug.at(r, j) - factor * aug.at(c, j));
            }
        }
    }

    let mut inv = DenseMatrix::zeros(n, n);
    for i in 0..n {
        for j in 0..n {
            inv.set(i, j, aug.at(i, n + j));
        }
    }
    Ok(inv)
}

fn validate_finite(matrix: &DenseMatrix) -> Result<(), LinearAlgebraError> {
    for row in 0..matrix.rows {
        for col in 0..matrix.cols {
            let value = matrix.at(row, col);
            if !value.is_finite() {
                return Err(LinearAlgebraError::NonFiniteValue { row, col, value });
            }
        }
    }
    Ok(())
}

fn validate_vector(values: &[f64]) -> Result<(), LinearAlgebraError> {
    for (row, &value) in values.iter().enumerate() {
        if !value.is_finite() {
            return Err(LinearAlgebraError::NonFiniteValue { row, col: 0, value });
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn l0_matrix_multiply_matches_hand_computed_values() {
        let a = DenseMatrix::from_row_major(2, 3, vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0]).unwrap();
        let b = DenseMatrix::from_row_major(3, 2, vec![7.0, 8.0, 9.0, 10.0, 11.0, 12.0]).unwrap();

        let product = mat_mul(&a, &b).unwrap();

        assert_eq!(
            product,
            DenseMatrix::from_row_major(2, 2, vec![58.0, 64.0, 139.0, 154.0]).unwrap()
        );
    }

    #[test]
    fn l0_matrix_vector_multiply_matches_hand_computed_values() {
        let a = DenseMatrix::from_row_major(2, 3, vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0]).unwrap();

        assert_eq!(
            mat_mul_vec(&a, &[10.0, 20.0, 30.0]).unwrap(),
            vec![140.0, 320.0]
        );
    }

    #[test]
    fn l0_transpose_swaps_axes() {
        let a = DenseMatrix::from_row_major(2, 3, vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0]).unwrap();
        let t = transpose(&a).unwrap();

        assert_eq!(
            t,
            DenseMatrix::from_row_major(3, 2, vec![1.0, 4.0, 2.0, 5.0, 3.0, 6.0]).unwrap()
        );
    }

    #[test]
    fn l0_inverse_multiplies_back_to_identity() {
        let a = DenseMatrix::from_row_major(2, 2, vec![4.0, 7.0, 2.0, 6.0]).unwrap();
        let inv = invert(&a).unwrap();
        let identity = mat_mul(&a, &inv).unwrap();

        for i in 0..2 {
            for j in 0..2 {
                let expected = if i == j { 1.0 } else { 0.0 };
                assert!((identity.at(i, j) - expected).abs() < 1e-12);
            }
        }
    }

    #[test]
    fn l0_singular_matrix_is_rejected() {
        let a = DenseMatrix::from_row_major(2, 2, vec![1.0, 2.0, 2.0, 4.0]).unwrap();

        assert_eq!(invert(&a), Err(LinearAlgebraError::Singular));
    }

    #[test]
    fn l0_dimension_mismatch_is_rejected() {
        let a = DenseMatrix::zeros(2, 3);
        let b = DenseMatrix::zeros(2, 3);

        assert_eq!(
            mat_mul(&a, &b),
            Err(LinearAlgebraError::DimensionMismatch {
                left: (2, 3),
                right: (2, 3)
            })
        );
    }
}
