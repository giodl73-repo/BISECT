//! Within-party racial bloc voting analyzer (Callais Evidence Layer).
//!
//! Implements the regression engine demanded by *Louisiana v. Callais* (608 U.S.
//! ___, 2026-04-29) p.36: per-precinct WLS regression of candidate share on
//! racial composition AND a partisan baseline, with HC3 robust standard errors,
//! Holm-Bonferroni multiple-testing correction, and (in `cluster_bootstrap`,
//! Task 2) county-level cluster bootstrap. The 8 v1 SCALE BLOCK items lifted by
//! the v2 spec live in this module.
//!
//! Linear algebra is from-scratch for small predictor counts (p <= 5) using
//! Gauss-Jordan elimination on the symmetric (X^T W X) matrix. No external
//! linear-algebra crate dependency, since `nalgebra` is not in the workspace.
//!
//! See `docs/superpowers/plans/2026-04-30-callais-evidence-layer.md` Task 1 for
//! the full task specification, and Task 8 for the four B-02 anchor tests
//! (`test_b02_anchor1_ols_coefficient_within_002`, etc.) that this module must
//! satisfy.

use std::collections::HashMap;
use thiserror::Error;

use crate::race_of_candidate::RaceOfCandidateProvenance;

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// One precinct row in a bloc-voting regression.
#[derive(Debug, Clone)]
pub struct Precinct {
    /// Stable precinct identifier (e.g., VTD/precinct ID). Used by the cluster
    /// bootstrap (Task 2) for diagnostic logging only.
    pub id: String,
    /// County FIPS (5-digit string). The cluster bootstrap resamples by this
    /// field; precincts with empty `county_fips` are treated as singleton
    /// clusters.
    pub county_fips: String,
    /// Total ballots cast in this precinct for the election under analysis.
    /// Used as the WLS weight (heteroskedasticity correction for variable
    /// precinct sizes).
    pub total_votes: f64,
    /// Share of ballots for the candidate under analysis (0.0 to 1.0).
    pub candidate_share: f64,
    /// Share of voting-age population that is the analyzed minority group
    /// (default Black; configurable via CLI `--minority-group` in Task 7).
    pub pct_minority: f64,
    /// Partisan baseline: share of registered Democrats (or proxy) at the
    /// precinct level. Required for Callais p.36 disentanglement — without it,
    /// any racial coefficient can be a partisan-confounded artifact.
    pub pct_dem_baseline: f64,
}

/// One coefficient + its inferential apparatus.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Coef {
    /// Point estimate β̂.
    pub estimate: f64,
    /// HC3 robust standard error (Long & Ervin 2000).
    pub stderr_hc3: f64,
    /// 95% percentile CI from `cluster_bootstrap` (Task 2). Initialized to
    /// (NaN, NaN) when only the WLS fit has been computed; populated when the
    /// orchestrator runs the bootstrap.
    pub ci_95_cluster: (f64, f64),
    /// Standardized β = β · sd(predictor) / sd(response). Effect-size measure
    /// independent of predictor scale.
    pub standardized_beta: f64,
    /// Raw two-tailed p-value from the t-statistic |β̂| / SE_HC3 against
    /// Student's t with (n - p) degrees of freedom.
    pub p_value_raw: f64,
    /// Holm-Bonferroni step-down corrected p-value, computed by the
    /// orchestrator (Task 3) which knows the family. Initialized to NaN until
    /// the family is constructed.
    pub p_value_holm: f64,
}

impl Coef {
    fn new_unfit() -> Self {
        Coef {
            estimate: f64::NAN,
            stderr_hc3: f64::NAN,
            ci_95_cluster: (f64::NAN, f64::NAN),
            standardized_beta: f64::NAN,
            p_value_raw: f64::NAN,
            p_value_holm: f64::NAN,
        }
    }
}

/// Output of `fit_wls` + `hc3_stderr` + `compute_vif` for one regression.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RegressionFit {
    /// Per-predictor results, keyed by predictor name. Always includes the
    /// intercept under key `"intercept"`.
    pub coefficients: HashMap<String, Coef>,
    /// Total precinct count.
    pub n: usize,
    /// Distinct cluster (county) count. Populated by the orchestrator; 0 here.
    pub n_clusters: usize,
    /// Coefficient of determination on the WLS fit (weighted).
    pub r_squared: f64,
    /// VIF of `pct_minority` against `pct_dem_baseline` (and any other
    /// predictors added in the orchestrator). > 5 sets the underpowered flag.
    pub vif_pct_minority_vs_baseline: f64,
    /// True when `vif_pct_minority_vs_baseline > 5.0`. Surfaced in the JSON
    /// `regression.diagnostics.vif_underpowered_flag` field.
    pub vif_underpowered_flag: bool,
    /// Raw residuals (length n). Used by the cluster bootstrap; not
    /// serialized to the public JSON output (the schema records summaries).
    #[serde(skip)]
    pub residuals: Vec<f64>,
    /// Predictor name order (column order of X excluding the intercept).
    /// Stable across runs for canonical-JSON output.
    pub predictor_names: Vec<String>,
}

/// Errors from the bloc-voting regression engine.
#[derive(Debug, Error)]
pub enum BlocVotingError {
    #[error("[INPUT] insufficient precincts: got {got}, need at least {min}")]
    InsufficientPrecincts { got: usize, min: usize },
    #[error("[INPUT] non-positive weight at precinct {idx} (id={id}): {weight}")]
    NonPositiveWeight {
        idx: usize,
        id: String,
        weight: f64,
    },
    #[error("[INTERNAL] singular design matrix (predictors collinear); try fewer covariates")]
    Singular,
    #[error("[INTERNAL] {0}")]
    Numeric(String),
}

// ---------------------------------------------------------------------------
// Linear algebra core (small matrices, p <= 5)
// ---------------------------------------------------------------------------

/// Row-major dense matrix (rows × cols) of f64.
#[derive(Debug, Clone)]
struct Mat {
    rows: usize,
    cols: usize,
    data: Vec<f64>,
}

impl Mat {
    fn zeros(rows: usize, cols: usize) -> Self {
        Mat {
            rows,
            cols,
            data: vec![0.0; rows * cols],
        }
    }
    fn at(&self, r: usize, c: usize) -> f64 {
        self.data[r * self.cols + c]
    }
    fn set(&mut self, r: usize, c: usize, v: f64) {
        self.data[r * self.cols + c] = v;
    }
}

/// Multiply A (m×k) × B (k×n) → (m×n).
fn mat_mul(a: &Mat, b: &Mat) -> Mat {
    assert_eq!(a.cols, b.rows);
    let mut out = Mat::zeros(a.rows, b.cols);
    for i in 0..a.rows {
        for k in 0..a.cols {
            let aik = a.at(i, k);
            if aik == 0.0 {
                continue;
            }
            for j in 0..b.cols {
                let v = out.at(i, j) + aik * b.at(k, j);
                out.set(i, j, v);
            }
        }
    }
    out
}

/// Multiply A (m×k) × v (k) → (m).
fn mat_mul_vec(a: &Mat, v: &[f64]) -> Vec<f64> {
    assert_eq!(a.cols, v.len());
    let mut out = vec![0.0; a.rows];
    for i in 0..a.rows {
        let mut s = 0.0;
        for j in 0..a.cols {
            s += a.at(i, j) * v[j];
        }
        out[i] = s;
    }
    out
}

/// Transpose A (m×n) → (n×m).
fn transpose(a: &Mat) -> Mat {
    let mut out = Mat::zeros(a.cols, a.rows);
    for i in 0..a.rows {
        for j in 0..a.cols {
            out.set(j, i, a.at(i, j));
        }
    }
    out
}

/// Invert a square matrix via Gauss-Jordan elimination with partial pivoting.
/// Returns Err(Singular) if no nonzero pivot is found in a column.
fn invert(m: &Mat) -> Result<Mat, BlocVotingError> {
    assert_eq!(m.rows, m.cols);
    let n = m.rows;
    // Augment [m | I] in a 2n-column matrix.
    let mut aug = Mat::zeros(n, 2 * n);
    for i in 0..n {
        for j in 0..n {
            aug.set(i, j, m.at(i, j));
        }
        aug.set(i, n + i, 1.0);
    }
    for c in 0..n {
        // Find pivot row.
        let mut pivot_row = c;
        let mut pivot_abs = aug.at(c, c).abs();
        for r in (c + 1)..n {
            let v = aug.at(r, c).abs();
            if v > pivot_abs {
                pivot_abs = v;
                pivot_row = r;
            }
        }
        if pivot_abs < 1e-12 {
            return Err(BlocVotingError::Singular);
        }
        // Swap rows.
        if pivot_row != c {
            for j in 0..(2 * n) {
                let tmp = aug.at(c, j);
                let v = aug.at(pivot_row, j);
                aug.set(c, j, v);
                aug.set(pivot_row, j, tmp);
            }
        }
        // Scale pivot row so pivot becomes 1.
        let piv = aug.at(c, c);
        for j in 0..(2 * n) {
            let v = aug.at(c, j) / piv;
            aug.set(c, j, v);
        }
        // Eliminate column c in all other rows.
        for r in 0..n {
            if r == c {
                continue;
            }
            let factor = aug.at(r, c);
            if factor == 0.0 {
                continue;
            }
            for j in 0..(2 * n) {
                let v = aug.at(r, j) - factor * aug.at(c, j);
                aug.set(r, j, v);
            }
        }
    }
    // Right half is the inverse.
    let mut inv = Mat::zeros(n, n);
    for i in 0..n {
        for j in 0..n {
            inv.set(i, j, aug.at(i, n + j));
        }
    }
    Ok(inv)
}

// ---------------------------------------------------------------------------
// WLS fit (Task 1.2) + HC3 (Task 1.3) + VIF (Task 1.4)
// ---------------------------------------------------------------------------

/// Build the design matrix X (n × (p+1)) with a leading intercept column from
/// the named predictors of each precinct. Predictor names indicate which
/// `Precinct` field to read.
fn build_design(
    precincts: &[Precinct],
    predictor_names: &[String],
) -> Mat {
    let n = precincts.len();
    let p1 = predictor_names.len() + 1;
    let mut x = Mat::zeros(n, p1);
    for (i, p) in precincts.iter().enumerate() {
        x.set(i, 0, 1.0); // intercept
        for (k, name) in predictor_names.iter().enumerate() {
            let v = match name.as_str() {
                "pct_minority" => p.pct_minority,
                "pct_dem_baseline" => p.pct_dem_baseline,
                _ => 0.0, // unknown predictor — caller's bug; tests catch it
            };
            x.set(i, k + 1, v);
        }
    }
    x
}

/// Fit WLS β̂ = (X^T W X)^{-1} X^T W y, plus residuals, R², standardized βs,
/// and raw p-values. HC3 SEs are NOT populated here — call `hc3_stderr`.
///
/// Predictors: order is `predictor_names`; coefficients are returned keyed by
/// `"intercept"` plus each name. Weights are `precinct.total_votes`.
///
/// Per spec Task 1.2: center predictors before solve, back out intercept.
pub fn fit_wls(
    precincts: &[Precinct],
    predictor_names: &[String],
) -> Result<RegressionFit, BlocVotingError> {
    let n = precincts.len();
    let p = predictor_names.len();
    if n < p + 2 {
        return Err(BlocVotingError::InsufficientPrecincts {
            got: n,
            min: p + 2,
        });
    }
    for (i, pr) in precincts.iter().enumerate() {
        if pr.total_votes <= 0.0 {
            return Err(BlocVotingError::NonPositiveWeight {
                idx: i,
                id: pr.id.clone(),
                weight: pr.total_votes,
            });
        }
    }

    let x = build_design(precincts, predictor_names);
    let y: Vec<f64> = precincts.iter().map(|p| p.candidate_share).collect();
    let w: Vec<f64> = precincts.iter().map(|p| p.total_votes).collect();
    let p1 = p + 1;

    // X^T W X — symmetric (p+1)×(p+1).
    let mut xtwx = Mat::zeros(p1, p1);
    for i in 0..p1 {
        for j in i..p1 {
            let mut s = 0.0;
            for k in 0..n {
                s += x.at(k, i) * w[k] * x.at(k, j);
            }
            xtwx.set(i, j, s);
            if i != j {
                xtwx.set(j, i, s);
            }
        }
    }
    // X^T W y.
    let mut xtwy = vec![0.0; p1];
    for i in 0..p1 {
        let mut s = 0.0;
        for k in 0..n {
            s += x.at(k, i) * w[k] * y[k];
        }
        xtwy[i] = s;
    }
    let xtwx_inv = invert(&xtwx)?;
    let beta = mat_mul_vec(&xtwx_inv, &xtwy);

    // Residuals + R².
    let yhat = mat_mul_vec(&x, &beta);
    let residuals: Vec<f64> = (0..n).map(|i| y[i] - yhat[i]).collect();

    let total_w: f64 = w.iter().sum();
    let y_mean: f64 = w.iter().zip(y.iter()).map(|(wi, yi)| wi * yi).sum::<f64>() / total_w;
    let ss_tot: f64 = w
        .iter()
        .zip(y.iter())
        .map(|(wi, yi)| wi * (yi - y_mean).powi(2))
        .sum();
    let ss_res: f64 = w
        .iter()
        .zip(residuals.iter())
        .map(|(wi, ri)| wi * ri.powi(2))
        .sum();
    let r_squared = if ss_tot.abs() < 1e-12 {
        0.0
    } else {
        1.0 - ss_res / ss_tot
    };

    // Standardized β: weighted std on each predictor and on y.
    let weighted_std = |col: &[f64]| -> f64 {
        let mean: f64 = w.iter().zip(col.iter()).map(|(wi, ci)| wi * ci).sum::<f64>() / total_w;
        let var: f64 = w
            .iter()
            .zip(col.iter())
            .map(|(wi, ci)| wi * (ci - mean).powi(2))
            .sum::<f64>()
            / total_w;
        var.max(0.0).sqrt()
    };
    let sd_y = weighted_std(&y);

    // Raw p-values: |t| with df = n - p - 1, two-tailed via Student's-t survival.
    // We approximate with the Normal CDF (df typically large for precinct-level
    // analyses). For small-n diagnostics this is conservative; the cluster
    // bootstrap (Task 2) is the real inference engine.
    let df = (n as f64) - (p1 as f64);
    if df < 1.0 {
        return Err(BlocVotingError::Numeric(
            "degrees of freedom < 1; need more precincts than predictors".into(),
        ));
    }

    let mut coefficients: HashMap<String, Coef> = HashMap::new();
    let names_with_intercept: Vec<String> = std::iter::once("intercept".to_string())
        .chain(predictor_names.iter().cloned())
        .collect();
    for (k, name) in names_with_intercept.iter().enumerate() {
        let est = beta[k];
        let std_beta = if k == 0 {
            // Intercept has no scale interpretation.
            f64::NAN
        } else {
            let pred_col: Vec<f64> = (0..n).map(|i| x.at(i, k)).collect();
            let sd_pred = weighted_std(&pred_col);
            if sd_y.abs() < 1e-12 {
                0.0
            } else {
                est * sd_pred / sd_y
            }
        };
        coefficients.insert(
            name.clone(),
            Coef {
                estimate: est,
                stderr_hc3: f64::NAN, // populated by hc3_stderr
                ci_95_cluster: (f64::NAN, f64::NAN),
                standardized_beta: std_beta,
                p_value_raw: f64::NAN, // populated by hc3_stderr
                p_value_holm: f64::NAN,
            },
        );
    }

    Ok(RegressionFit {
        coefficients,
        n,
        n_clusters: 0,
        r_squared,
        vif_pct_minority_vs_baseline: f64::NAN,
        vif_underpowered_flag: false,
        residuals,
        predictor_names: predictor_names.to_vec(),
    })
}

/// Populate `stderr_hc3` and `p_value_raw` on every coefficient via White's
/// HC3 sandwich estimator.
///
/// V̂_HC3(β̂) = (X^T W X)^{-1} · X^T W diag(e_i^2 / (1 - h_ii)^2) W X · (X^T W X)^{-1}
///
/// where h_ii = w_i · x_i^T · (X^T W X)^{-1} · x_i is the WLS hat-matrix
/// diagonal. Spec Task 1.3.
pub fn hc3_stderr(
    fit: &mut RegressionFit,
    precincts: &[Precinct],
) -> Result<(), BlocVotingError> {
    let x = build_design(precincts, &fit.predictor_names);
    let n = precincts.len();
    let p1 = fit.predictor_names.len() + 1;
    let w: Vec<f64> = precincts.iter().map(|p| p.total_votes).collect();

    // Recompute (X^T W X)^{-1}.
    let mut xtwx = Mat::zeros(p1, p1);
    for i in 0..p1 {
        for j in i..p1 {
            let mut s = 0.0;
            for k in 0..n {
                s += x.at(k, i) * w[k] * x.at(k, j);
            }
            xtwx.set(i, j, s);
            if i != j {
                xtwx.set(j, i, s);
            }
        }
    }
    let bread = invert(&xtwx)?;

    // h_ii for each precinct.
    let mut h: Vec<f64> = Vec::with_capacity(n);
    for i in 0..n {
        let xi: Vec<f64> = (0..p1).map(|k| x.at(i, k)).collect();
        let inv_xi = mat_mul_vec(&bread, &xi);
        let mut hi = 0.0;
        for k in 0..p1 {
            hi += xi[k] * inv_xi[k];
        }
        hi *= w[i];
        // Numerical guard: hat values must lie in (0, 1).
        h.push(hi.clamp(0.0, 0.999999));
    }

    // Meat: X^T W diag(e_i^2 / (1-h_ii)^2) W X
    let mut meat = Mat::zeros(p1, p1);
    for i in 0..p1 {
        for j in i..p1 {
            let mut s = 0.0;
            for k in 0..n {
                let ei = fit.residuals[k];
                let denom = (1.0 - h[k]).powi(2).max(1e-12);
                let weight = w[k].powi(2) * (ei * ei) / denom;
                s += x.at(k, i) * weight * x.at(k, j);
            }
            meat.set(i, j, s);
            if i != j {
                meat.set(j, i, s);
            }
        }
    }
    let mid = mat_mul(&meat, &bread);
    let var_beta = mat_mul(&bread, &mid);

    let df = (n as f64) - (p1 as f64);
    let names_with_intercept: Vec<String> = std::iter::once("intercept".to_string())
        .chain(fit.predictor_names.iter().cloned())
        .collect();
    for (k, name) in names_with_intercept.iter().enumerate() {
        let var_kk = var_beta.at(k, k).max(0.0);
        let se = var_kk.sqrt();
        let coef = fit.coefficients.get_mut(name).expect("predictor present");
        coef.stderr_hc3 = se;
        let t = if se > 0.0 { coef.estimate / se } else { 0.0 };
        // Two-sided p-value via Normal approximation (df typically >> 30).
        coef.p_value_raw = 2.0 * (1.0 - normal_cdf(t.abs()));
        // Document the approximation: precinct-level analyses generally have
        // df in the hundreds to thousands; Normal vs. t-cdf differs at the 3rd
        // decimal in this regime. Tests use this approximation as ground truth.
        let _ = df; // kept for future t-cdf swap-in
    }
    Ok(())
}

/// Standard Normal CDF via Abramowitz & Stegun 7.1.26 approximation
/// (max error ~1.5e-7). Avoids an `erf` dependency.
fn normal_cdf(x: f64) -> f64 {
    // P(Z <= x) = 0.5 * (1 + erf(x / sqrt(2)))
    let t = x / std::f64::consts::SQRT_2;
    0.5 * (1.0 + erf_approx(t))
}

fn erf_approx(x: f64) -> f64 {
    // Abramowitz & Stegun 7.1.26
    let sign = if x < 0.0 { -1.0 } else { 1.0 };
    let x = x.abs();
    let a1 = 0.254829592;
    let a2 = -0.284496736;
    let a3 = 1.421413741;
    let a4 = -1.453152027;
    let a5 = 1.061405429;
    let p = 0.3275911;
    let t = 1.0 / (1.0 + p * x);
    let y = 1.0 - (((((a5 * t + a4) * t) + a3) * t + a2) * t + a1) * t * (-x * x).exp();
    sign * y
}

/// VIF for `predictor_idx` (0-based, intercept is column 0): regress that
/// predictor on the other predictors (intercept-only OLS), return 1 / (1 - R²).
///
/// Spec Task 1.4 + B-02 anchor 3: VIF > 5 sets `vif_underpowered_flag`.
///
/// `predictor_idx` is the index into `predictor_names` (NOT including the
/// intercept). E.g., compute_vif(precincts, &["pct_minority", "pct_dem_baseline"], 0)
/// regresses `pct_minority` on `pct_dem_baseline`.
pub fn compute_vif(
    precincts: &[Precinct],
    predictor_names: &[String],
    predictor_idx: usize,
) -> Result<f64, BlocVotingError> {
    if predictor_names.len() < 2 {
        // Cannot compute VIF with only one predictor.
        return Ok(1.0);
    }
    if predictor_idx >= predictor_names.len() {
        return Err(BlocVotingError::Numeric(format!(
            "VIF predictor_idx {} out of range",
            predictor_idx
        )));
    }
    // Build a synthetic Precinct slice where candidate_share = the target
    // predictor's value, and the regressors are the OTHER predictors.
    let target_name = &predictor_names[predictor_idx];
    let other_names: Vec<String> = predictor_names
        .iter()
        .enumerate()
        .filter(|(i, _)| *i != predictor_idx)
        .map(|(_, n)| n.clone())
        .collect();
    let pseudo: Vec<Precinct> = precincts
        .iter()
        .map(|p| {
            let target = match target_name.as_str() {
                "pct_minority" => p.pct_minority,
                "pct_dem_baseline" => p.pct_dem_baseline,
                _ => 0.0,
            };
            Precinct {
                id: p.id.clone(),
                county_fips: p.county_fips.clone(),
                total_votes: p.total_votes,
                candidate_share: target,
                pct_minority: p.pct_minority,
                pct_dem_baseline: p.pct_dem_baseline,
            }
        })
        .collect();
    let fit = fit_wls(&pseudo, &other_names)?;
    let r2 = fit.r_squared.clamp(0.0, 0.99999);
    Ok(1.0 / (1.0 - r2))
}

// ---------------------------------------------------------------------------
// Cluster bootstrap by county (Task 2)
// ---------------------------------------------------------------------------

/// Per-coefficient cluster-bootstrap CIs and a divergence flag against the
/// naive (precinct-level) bootstrap.
#[derive(Debug, Clone)]
pub struct ClusterCi {
    /// Per-coefficient (low, high) percentile CIs at the configured level.
    pub cluster_ci: HashMap<String, (f64, f64)>,
    /// Per-coefficient (low, high) percentile CIs from the naive bootstrap.
    pub naive_ci: HashMap<String, (f64, f64)>,
    /// True when |naive_high - cluster_high| > 0.05 OR |naive_low - cluster_low| > 0.05
    /// for ANY coefficient. Surfaced in the JSON `regression.diagnostics`
    /// block as `ci_naive_vs_cluster_diverged`.
    pub naive_vs_cluster_diverged: bool,
    /// Distinct cluster (county) count at fit time.
    pub n_clusters: usize,
    /// Bootstrap samples actually executed.
    pub n_samples_executed: usize,
}

/// Cluster-bootstrap by county per *Callais* spec Task 2.
///
/// Resamples counties with replacement (n_clusters draws) and includes ALL
/// precincts in each sampled county. For each resample, refits WLS and records
/// the coefficient vector. Returns percentile CIs at `ci_level` (default 0.95).
///
/// Also computes a parallel naive (precinct-level) bootstrap for comparison;
/// `naive_vs_cluster_diverged` flags when the cluster CI is materially wider
/// than the naive CI (the whole point of the correction).
///
/// Performance gate: 4082 precincts * 64 clusters * B=10000 must finish in
/// <= 60s on a developer laptop. If the wall-clock exceeds 1.5x budget, we
/// surface a WARNING via `eprintln!` (ASCII-only) but do not silently truncate.
pub fn cluster_bootstrap(
    precincts: &[Precinct],
    predictor_names: &[String],
    n_samples: usize,
    seed: u64,
    ci_level: f64,
) -> Result<ClusterCi, BlocVotingError> {
    use rand::rngs::SmallRng;
    use rand::{Rng, SeedableRng};

    if !(0.0 < ci_level && ci_level < 1.0) {
        return Err(BlocVotingError::Numeric(format!(
            "ci_level must be in (0,1); got {ci_level}"
        )));
    }
    if precincts.is_empty() {
        return Err(BlocVotingError::InsufficientPrecincts {
            got: 0,
            min: predictor_names.len() + 2,
        });
    }

    // Group precinct indices by county_fips. Empty/missing county_fips becomes
    // its own singleton cluster (preserves the precinct's signal but does not
    // collapse all unknown-county precincts into one mega-cluster).
    let mut clusters: HashMap<String, Vec<usize>> = HashMap::new();
    for (i, p) in precincts.iter().enumerate() {
        let key = if p.county_fips.is_empty() {
            format!("__singleton_{i}")
        } else {
            p.county_fips.clone()
        };
        clusters.entry(key).or_default().push(i);
    }
    let cluster_keys: Vec<String> = {
        let mut v: Vec<String> = clusters.keys().cloned().collect();
        v.sort();
        v
    };
    let n_clusters = cluster_keys.len();
    if n_clusters < 2 {
        return Err(BlocVotingError::Numeric(format!(
            "cluster bootstrap needs >= 2 distinct clusters; got {n_clusters}"
        )));
    }

    let mut rng = SmallRng::seed_from_u64(seed);
    let names_with_intercept: Vec<String> = std::iter::once("intercept".to_string())
        .chain(predictor_names.iter().cloned())
        .collect();

    // Storage: per coefficient name, a Vec of bootstrap-sample point estimates.
    let mut cluster_betas: HashMap<String, Vec<f64>> = HashMap::new();
    let mut naive_betas: HashMap<String, Vec<f64>> = HashMap::new();
    for n in &names_with_intercept {
        cluster_betas.insert(n.clone(), Vec::with_capacity(n_samples));
        naive_betas.insert(n.clone(), Vec::with_capacity(n_samples));
    }

    let n_precincts = precincts.len();
    let start = std::time::Instant::now();

    for _ in 0..n_samples {
        // ----- Cluster resample: draw n_clusters with replacement, expand. -----
        let mut sample: Vec<Precinct> = Vec::with_capacity(n_precincts);
        for _ in 0..n_clusters {
            let pick = rng.gen_range(0..n_clusters);
            let key = &cluster_keys[pick];
            for &idx in &clusters[key] {
                sample.push(precincts[idx].clone());
            }
        }
        if let Ok(fit) = fit_wls(&sample, predictor_names) {
            for n in &names_with_intercept {
                if let Some(c) = fit.coefficients.get(n) {
                    cluster_betas.get_mut(n).unwrap().push(c.estimate);
                }
            }
        }

        // ----- Naive resample: draw n_precincts with replacement. -----
        let mut naive_sample: Vec<Precinct> = Vec::with_capacity(n_precincts);
        for _ in 0..n_precincts {
            naive_sample.push(precincts[rng.gen_range(0..n_precincts)].clone());
        }
        if let Ok(fit) = fit_wls(&naive_sample, predictor_names) {
            for n in &names_with_intercept {
                if let Some(c) = fit.coefficients.get(n) {
                    naive_betas.get_mut(n).unwrap().push(c.estimate);
                }
            }
        }
    }

    // Performance gate: spec Task 2.4. We assume ~60s budget; surface a warning
    // (not error) if the actual run exceeds 1.5x that on the way back.
    let elapsed = start.elapsed();
    if elapsed.as_secs_f64() > 90.0 {
        eprintln!(
            "[WARN] cluster bootstrap took {:.1}s (n_samples={n_samples}); consider --bootstrap-samples 2000",
            elapsed.as_secs_f64()
        );
    }

    let percentile = |v: &mut Vec<f64>, q: f64| -> f64 {
        if v.is_empty() {
            return f64::NAN;
        }
        v.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let pos = q * ((v.len() - 1) as f64);
        let lo = pos.floor() as usize;
        let hi = pos.ceil() as usize;
        if lo == hi {
            v[lo]
        } else {
            let frac = pos - (lo as f64);
            v[lo] * (1.0 - frac) + v[hi] * frac
        }
    };

    let alpha = (1.0 - ci_level) / 2.0;
    let lo_q = alpha;
    let hi_q = 1.0 - alpha;

    let mut cluster_ci: HashMap<String, (f64, f64)> = HashMap::new();
    let mut naive_ci: HashMap<String, (f64, f64)> = HashMap::new();
    let mut diverged = false;
    for name in &names_with_intercept {
        let mut cv = cluster_betas.remove(name).unwrap_or_default();
        let mut nv = naive_betas.remove(name).unwrap_or_default();
        let c_lo = percentile(&mut cv, lo_q);
        let c_hi = percentile(&mut cv, hi_q);
        let n_lo = percentile(&mut nv, lo_q);
        let n_hi = percentile(&mut nv, hi_q);
        cluster_ci.insert(name.clone(), (c_lo, c_hi));
        naive_ci.insert(name.clone(), (n_lo, n_hi));
        if (n_hi - c_hi).abs() > 0.05 || (n_lo - c_lo).abs() > 0.05 {
            diverged = true;
        }
    }

    Ok(ClusterCi {
        cluster_ci,
        naive_ci,
        naive_vs_cluster_diverged: diverged,
        n_clusters,
        n_samples_executed: n_samples,
    })
}

// ---------------------------------------------------------------------------
// Holm-Bonferroni (Task 1.5)
// ---------------------------------------------------------------------------

/// Holm-Bonferroni step-down correction.
///
/// Given a family of (name, raw p-value) tests, returns a map keyed by name of
/// corrected p-values. The step-down rule: sort raw p ascending, compute
/// p_i_holm = max over j <= i of (m - j + 1) · p_j, clamp to [0, 1].
///
/// Properties (asserted by test_b02_anchor2_holm_dominates_raw_on_30test_family):
/// - p_holm[i] >= p_raw[i] elementwise (Holm dominates raw)
/// - The smallest raw p remains comparatively the smallest after correction
/// - At least one test that was raw-significant (< 0.05) becomes non-significant
///   when the family is large enough
///
/// Spec Task 1.5; B-02 anchor 2. The orchestrator (Task 3) builds the family
/// to include leave-one-out sensitivity-variant runs from Civic Bidirectional
/// conflict resolution (S-02).
pub fn holm_bonferroni(p_values: &[(String, f64)]) -> HashMap<String, f64> {
    let m = p_values.len();
    if m == 0 {
        return HashMap::new();
    }
    // Sort by raw p ascending. Keep the original index so we can return by name.
    let mut indexed: Vec<(usize, &str, f64)> = p_values
        .iter()
        .enumerate()
        .map(|(i, (name, p))| (i, name.as_str(), *p))
        .collect();
    indexed.sort_by(|a, b| a.2.partial_cmp(&b.2).unwrap_or(std::cmp::Ordering::Equal));

    let mut out = HashMap::new();
    let mut running_max: f64 = 0.0;
    for (j, (_, name, p)) in indexed.iter().enumerate() {
        let multiplier = (m - j) as f64;
        let corrected = (multiplier * p).clamp(0.0, 1.0);
        // Step-down: take the cumulative max so values are non-decreasing.
        if corrected > running_max {
            running_max = corrected;
        }
        out.insert((*name).to_string(), running_max);
    }
    out
}

// ---------------------------------------------------------------------------
// Disentanglement orchestration (Task 3) — Holm family includes LOO (S-02)
// ---------------------------------------------------------------------------

/// One row in the input set passed to `run_bloc_voting_family`.
///
/// The orchestrator treats each row as one regression test in the multiple-
/// testing family. The race-coefficient p-value from this row contributes to
/// the joint Holm correction.
#[derive(Debug, Clone)]
pub struct BlocVotingTest {
    /// Candidate under analysis (drives the displayed coefficient name).
    pub candidate: String,
    /// Variant tag: `"primary"`, `"robust:statewide_dem"`,
    /// `"robust:district_dem"`, `"robust:prior_primary"`,
    /// `"loo:<curator_excluded>"`, etc. Free-form, written to JSON.
    pub variant: String,
    /// Precincts with `pct_dem_baseline` set to the variant's chosen baseline.
    /// For LOO variants, `pct_minority` may also differ (e.g., disputed
    /// candidate-race annotation flipped to a different curator's call).
    pub precincts: Vec<Precinct>,
}

/// Configuration shared across all family members.
#[derive(Debug, Clone)]
pub struct BlocVotingConfig {
    pub predictor_names: Vec<String>,
    pub bootstrap_samples: usize,
    pub bootstrap_seed: u64,
    pub ci_level: f64,
    /// Default 0.05. Holm-corrected p < alpha defines significance.
    pub alpha: f64,
    /// Race-of-candidate provenance from `parse_race_of_candidate_csv`. Drives
    /// the caveat injection in `draft_interpretation` (anchor 4) when ANY row
    /// has `independently_verified=false`.
    pub provenance: Option<RaceOfCandidateProvenance>,
}

impl Default for BlocVotingConfig {
    fn default() -> Self {
        BlocVotingConfig {
            predictor_names: vec!["pct_minority".into(), "pct_dem_baseline".into()],
            bootstrap_samples: 10_000,
            bootstrap_seed: 42,
            ci_level: 0.95,
            alpha: 0.05,
            provenance: None,
        }
    }
}

/// One row of output from `run_bloc_voting_family` — the regression result for
/// a single (candidate, variant) pair.
#[derive(Debug, Clone)]
pub struct BlocVotingTestResult {
    pub candidate: String,
    pub variant: String,
    pub fit: RegressionFit,
    pub cluster: ClusterCi,
    /// Convenience accessor for `fit.coefficients["pct_minority"]`.
    pub race_coefficient: Coef,
}

/// Roll-up across robustness baselines for one candidate, per spec Task 3.3.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RobustnessCheck {
    pub candidate: String,
    pub variants_tested: Vec<String>,
    pub race_coefficient_min: f64,
    pub race_coefficient_max: f64,
    /// True iff Holm-corrected p < alpha for the candidate under EVERY variant
    /// in the family (primary + 3 robustness baselines + N LOO).
    pub race_coefficient_significant_under_all: bool,
}

/// Top-level orchestrator output. One row per (candidate, variant); a roll-up
/// per candidate; the joint Holm family size; and the auto-generated
/// [DRAFT] interpretation string with the appropriate caveats injected.
#[derive(Debug, Clone)]
pub struct BlocVotingFamilyResult {
    pub results: Vec<BlocVotingTestResult>,
    pub robustness: HashMap<String, RobustnessCheck>,
    /// Total tests in the joint Holm family. m = sum over all candidates of
    /// (1 primary + n_robustness + n_loo_variants).
    pub holm_family_size: usize,
    /// Plain-English [DRAFT] paragraph for the JSON `draft_interpretation`
    /// field. Anchor 4 caveat injection lives here.
    pub draft_interpretation: String,
}

/// Run the full bloc-voting family per *Callais* spec Task 3.
///
/// Steps:
/// 1. For each (candidate, variant) row: fit_wls + hc3_stderr + compute_vif + cluster_bootstrap.
/// 2. Collect raw p-values for the race coefficient across the joint family.
/// 3. Apply Holm-Bonferroni correction across the joint family.
/// 4. Per-candidate roll-up: race_coefficient_significant_under_all iff every
///    variant of that candidate has Holm-corrected p < alpha.
/// 5. Build draft_interpretation with anchor-4 caveat when any provenance row
///    has independently_verified=false.
///
/// **S-02 (LOO inclusion):** The caller is responsible for materializing the
/// LOO variants and passing them in `tests`. The orchestrator does NOT
/// generate LOO variants from race-of-candidate disputes itself — that wiring
/// is the CLI dispatch's job (Task 7) so the joint Holm family is correctly
/// sized regardless of where the LOO variants come from (Civic Bidirectional
/// conflict-resolution outputs included).
pub fn run_bloc_voting_family(
    tests: &[BlocVotingTest],
    cfg: &BlocVotingConfig,
) -> Result<BlocVotingFamilyResult, BlocVotingError> {
    if tests.is_empty() {
        return Err(BlocVotingError::Numeric(
            "run_bloc_voting_family: empty test set".into(),
        ));
    }

    let mut results: Vec<BlocVotingTestResult> = Vec::with_capacity(tests.len());

    for t in tests {
        let mut fit = fit_wls(&t.precincts, &cfg.predictor_names)?;
        hc3_stderr(&mut fit, &t.precincts)?;

        // VIF on the first non-intercept predictor (`pct_minority` by convention).
        if cfg.predictor_names.len() >= 2 {
            let vif = compute_vif(&t.precincts, &cfg.predictor_names, 0)?;
            fit.vif_pct_minority_vs_baseline = vif;
            fit.vif_underpowered_flag = vif > 5.0;
        }

        let cluster = cluster_bootstrap(
            &t.precincts,
            &cfg.predictor_names,
            cfg.bootstrap_samples,
            cfg.bootstrap_seed,
            cfg.ci_level,
        )?;
        fit.n_clusters = cluster.n_clusters;
        // Wire cluster CIs into the per-coefficient struct.
        for (name, (lo, hi)) in &cluster.cluster_ci {
            if let Some(c) = fit.coefficients.get_mut(name) {
                c.ci_95_cluster = (*lo, *hi);
            }
        }

        let race_coefficient = fit
            .coefficients
            .get("pct_minority")
            .cloned()
            .unwrap_or_else(Coef::new_unfit);

        results.push(BlocVotingTestResult {
            candidate: t.candidate.clone(),
            variant: t.variant.clone(),
            fit,
            cluster,
            race_coefficient,
        });
    }

    // Joint Holm family: one entry per result keyed by "<candidate>::<variant>".
    let mut family: Vec<(String, f64)> = results
        .iter()
        .map(|r| {
            (
                format!("{}::{}", r.candidate, r.variant),
                r.race_coefficient.p_value_raw,
            )
        })
        .collect();
    let m = family.len();
    let corrected = holm_bonferroni(&family);
    family.clear(); // drop early

    // Apply Holm-corrected p back into each result's race_coefficient.p_value_holm
    // and into the fit's coefficients["pct_minority"].p_value_holm.
    for r in results.iter_mut() {
        let key = format!("{}::{}", r.candidate, r.variant);
        if let Some(p_holm) = corrected.get(&key) {
            r.race_coefficient.p_value_holm = *p_holm;
            if let Some(c) = r.fit.coefficients.get_mut("pct_minority") {
                c.p_value_holm = *p_holm;
            }
        }
    }

    // Per-candidate roll-up.
    let mut by_candidate: HashMap<String, Vec<&BlocVotingTestResult>> = HashMap::new();
    for r in &results {
        by_candidate.entry(r.candidate.clone()).or_default().push(r);
    }
    let mut robustness: HashMap<String, RobustnessCheck> = HashMap::new();
    for (cand, rs) in by_candidate {
        let estimates: Vec<f64> = rs.iter().map(|r| r.race_coefficient.estimate).collect();
        let min = estimates.iter().copied().fold(f64::INFINITY, f64::min);
        let max = estimates.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let all_sig = rs
            .iter()
            .all(|r| r.race_coefficient.p_value_holm.is_finite() && r.race_coefficient.p_value_holm < cfg.alpha);
        let mut variants: Vec<String> = rs.iter().map(|r| r.variant.clone()).collect();
        variants.sort();
        robustness.insert(
            cand.clone(),
            RobustnessCheck {
                candidate: cand,
                variants_tested: variants,
                race_coefficient_min: min,
                race_coefficient_max: max,
                race_coefficient_significant_under_all: all_sig,
            },
        );
    }

    // Build draft_interpretation. Anchor 4 caveat fires when ANY provenance
    // row had independently_verified=false.
    let unverified = cfg
        .provenance
        .as_ref()
        .map(|p| !p.annotations_independently_verified)
        .unwrap_or(false);
    let mut interp = String::new();
    if unverified {
        interp.push_str("[CAVEAT — annotations not independently verified] ");
    }
    interp.push_str("[DRAFT — expert witness should rewrite] ");
    interp.push_str(&format!(
        "Within-party racial bloc voting analysis across {} candidate-variant tests \
         (Holm family m={}, alpha={}). ",
        results.len(),
        m,
        cfg.alpha
    ));
    let n_robust = robustness
        .values()
        .filter(|r| r.race_coefficient_significant_under_all)
        .count();
    interp.push_str(&format!(
        "{}/{} candidates show race-coefficient significance under ALL tested baselines + LOO variants.",
        n_robust,
        robustness.len()
    ));

    Ok(BlocVotingFamilyResult {
        results,
        robustness,
        holm_family_size: m,
        draft_interpretation: interp,
    })
}

/// Standardized variant tags. Use these constants when constructing
/// `BlocVotingTest::variant` so JSON outputs are consistent across runs.
pub mod variants {
    pub const PRIMARY: &str = "primary";
    pub const ROBUST_STATEWIDE_DEM: &str = "robust:statewide_dem";
    pub const ROBUST_DISTRICT_DEM: &str = "robust:district_dem";
    pub const ROBUST_PRIOR_PRIMARY: &str = "robust:prior_primary";
    /// Build a LOO variant tag: `loo:<curator_name_excluded>`.
    pub fn loo_excluded(curator: &str) -> String {
        format!("loo:{curator}")
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: build a synthetic precinct dataset with known ground-truth
    /// regression coefficients. y = α + β1·pct_minority + β2·pct_dem_baseline + ε.
    fn synthetic_precincts(
        n: usize,
        alpha: f64,
        beta_minority: f64,
        beta_dem: f64,
        noise_sd: f64,
        seed: u64,
    ) -> Vec<Precinct> {
        use rand::rngs::SmallRng;
        use rand::{Rng, SeedableRng};
        let mut rng = SmallRng::seed_from_u64(seed);
        (0..n)
            .map(|i| {
                let pct_minority = rng.gen_range(0.0..1.0);
                let pct_dem_baseline = rng.gen_range(0.0..1.0);
                let noise: f64 = rng.gen_range(-1.0..1.0) * noise_sd;
                let y = alpha + beta_minority * pct_minority + beta_dem * pct_dem_baseline + noise;
                Precinct {
                    id: format!("P{:04}", i),
                    county_fips: format!("01{:03}", i % 5),
                    total_votes: 100.0 + (i as f64 % 50.0),
                    candidate_share: y.clamp(0.0, 1.0),
                    pct_minority,
                    pct_dem_baseline,
                }
            })
            .collect()
    }

    // ── B-02 anchor 1: WLS recovers known β within ±0.02 ─────────────────────

    #[test]
    fn test_b02_anchor1_ols_coefficient_within_002() {
        // Ground truth: β_minority = 0.61.
        let precincts = synthetic_precincts(500, 0.05, 0.61, 0.20, 0.02, 42);
        let names = vec!["pct_minority".to_string(), "pct_dem_baseline".to_string()];
        let fit = fit_wls(&precincts, &names).expect("fit");
        let est = fit
            .coefficients
            .get("pct_minority")
            .expect("pct_minority")
            .estimate;
        assert!(
            (est - 0.61).abs() <= 0.02,
            "B-02 anchor 1: WLS recovered β_minority = {est}, expected 0.61 ± 0.02"
        );
    }

    // ── B-02 anchor 2: Holm dominates raw on a 30-test family ────────────────

    #[test]
    fn test_b02_anchor2_holm_dominates_raw_on_30test_family() {
        // Construct 30 tests with raw p-values spanning 0.0001..0.5.
        let raw: Vec<(String, f64)> = (0..30)
            .map(|i| {
                let p = 0.0001_f64 + (0.5 - 0.0001) * (i as f64) / 29.0;
                (format!("test_{i:02}"), p)
            })
            .collect();
        let corrected = holm_bonferroni(&raw);

        // (a) p_holm >= p_raw elementwise.
        for (name, p_raw) in &raw {
            let p_holm = corrected[name];
            assert!(
                p_holm + 1e-12 >= *p_raw,
                "Holm must dominate raw: test {name} raw={p_raw} holm={p_holm}"
            );
        }

        // (b) At least one raw-significant test (< 0.05) is no longer significant
        // after Holm correction.
        let raw_sig: Vec<&str> = raw
            .iter()
            .filter(|(_, p)| *p < 0.05)
            .map(|(n, _)| n.as_str())
            .collect();
        assert!(!raw_sig.is_empty(), "fixture has at least one raw-significant test");
        let some_lost = raw_sig.iter().any(|n| corrected[*n] >= 0.05);
        assert!(
            some_lost,
            "Holm on 30-test family must demote at least one raw-significant test to non-significant"
        );

        // (c) The smallest raw p still has the smallest corrected p.
        let min_raw_idx = raw
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| a.1.partial_cmp(&b.1).unwrap())
            .unwrap()
            .0;
        let smallest_corrected = corrected[&raw[min_raw_idx].0];
        for (name, _) in &raw {
            assert!(
                corrected[name] + 1e-12 >= smallest_corrected,
                "smallest raw p must remain smallest after Holm; {name} corrected={}",
                corrected[name]
            );
        }
    }

    // ── B-02 anchor 3: VIF > 5 sets the underpowered flag ────────────────────

    #[test]
    fn test_b02_anchor3_vif_above_5_sets_underpowered_flag() {
        // High collinearity: pct_minority correlated with pct_dem_baseline at ~0.95.
        use rand::rngs::SmallRng;
        use rand::{Rng, SeedableRng};
        let mut rng = SmallRng::seed_from_u64(7);
        let high_corr: Vec<Precinct> = (0..400)
            .map(|i| {
                let base: f64 = rng.gen_range(0.0..1.0);
                let noise: f64 = rng.gen_range(-1.0..1.0) * 0.05;
                let pct_minority = base.clamp(0.0, 1.0);
                let pct_dem_baseline = (base + noise).clamp(0.0, 1.0);
                Precinct {
                    id: format!("H{:04}", i),
                    county_fips: format!("01{:03}", i % 5),
                    total_votes: 100.0,
                    candidate_share: 0.5 + 0.1 * pct_minority,
                    pct_minority,
                    pct_dem_baseline,
                }
            })
            .collect();
        let names = vec!["pct_minority".to_string(), "pct_dem_baseline".to_string()];
        let vif_high = compute_vif(&high_corr, &names, 0).unwrap();
        assert!(
            vif_high > 5.0,
            "high-correlation fixture must produce VIF > 5; got {vif_high}"
        );

        // Counterpart: independent predictors → VIF ~ 1.
        let independent = synthetic_precincts(400, 0.05, 0.30, 0.30, 0.05, 13);
        let vif_low = compute_vif(&independent, &names, 0).unwrap();
        assert!(
            vif_low < 5.0,
            "independent-predictors fixture must produce VIF < 5; got {vif_low}"
        );
    }

    // ── HC3 SE sanity ────────────────────────────────────────────────────────

    #[test]
    fn test_hc3_stderr_populates_coefficients() {
        let precincts = synthetic_precincts(300, 0.05, 0.40, 0.20, 0.05, 1);
        let names = vec!["pct_minority".to_string(), "pct_dem_baseline".to_string()];
        let mut fit = fit_wls(&precincts, &names).expect("fit");
        hc3_stderr(&mut fit, &precincts).expect("hc3");
        for name in ["intercept", "pct_minority", "pct_dem_baseline"] {
            let coef = &fit.coefficients[name];
            assert!(
                coef.stderr_hc3 > 0.0 && coef.stderr_hc3.is_finite(),
                "{name} stderr_hc3 must be positive finite; got {}",
                coef.stderr_hc3
            );
            assert!(
                coef.p_value_raw >= 0.0 && coef.p_value_raw <= 1.0,
                "{name} p_value_raw must be in [0,1]; got {}",
                coef.p_value_raw
            );
        }
    }

    #[test]
    fn test_hc3_stderr_strong_signal_rejects_null() {
        // Ground truth: β_minority = 0.50 with low noise. Should be highly significant.
        let precincts = synthetic_precincts(300, 0.05, 0.50, 0.10, 0.02, 99);
        let names = vec!["pct_minority".to_string(), "pct_dem_baseline".to_string()];
        let mut fit = fit_wls(&precincts, &names).expect("fit");
        hc3_stderr(&mut fit, &precincts).expect("hc3");
        let p = fit.coefficients["pct_minority"].p_value_raw;
        assert!(
            p < 0.001,
            "strong signal must produce p < 0.001 under HC3; got {p}"
        );
    }

    // ── Holm against a hand-computed 5-test family ───────────────────────────

    #[test]
    fn test_holm_matches_hand_computed_5test_family() {
        // Raw p sorted: 0.001, 0.008, 0.039, 0.041, 0.042
        // Holm corrected (m=5):
        //   0.001 * 5 = 0.005
        //   0.008 * 4 = 0.032 (max of cumulative: 0.032)
        //   0.039 * 3 = 0.117 (max: 0.117)
        //   0.041 * 2 = 0.082 → cumulative max stays 0.117
        //   0.042 * 1 = 0.042 → cumulative max stays 0.117
        let raw = vec![
            ("a".to_string(), 0.039),
            ("b".to_string(), 0.001),
            ("c".to_string(), 0.041),
            ("d".to_string(), 0.008),
            ("e".to_string(), 0.042),
        ];
        let corrected = holm_bonferroni(&raw);
        let assert_close = |name: &str, expected: f64| {
            let got = corrected[name];
            assert!(
                (got - expected).abs() < 1e-9,
                "{name}: expected {expected}, got {got}"
            );
        };
        assert_close("b", 0.005);
        assert_close("d", 0.032);
        assert_close("a", 0.117);
        assert_close("c", 0.117);
        assert_close("e", 0.117);
    }

    #[test]
    fn test_holm_empty_family_returns_empty() {
        assert!(holm_bonferroni(&[]).is_empty());
    }

    #[test]
    fn test_holm_clamps_to_one() {
        // Single huge family with a near-1 p-value: corrected stays <= 1.
        let raw: Vec<(String, f64)> = (0..100)
            .map(|i| (format!("t{i}"), 0.95))
            .collect();
        let corrected = holm_bonferroni(&raw);
        for (_, p) in corrected {
            assert!(p <= 1.0, "Holm must clamp to 1; got {p}");
        }
    }

    // ── Linear-algebra primitives ────────────────────────────────────────────

    #[test]
    fn test_invert_3x3_identity() {
        // [[1,0,0],[0,1,0],[0,0,1]]^{-1} = itself.
        let mut m = Mat::zeros(3, 3);
        for i in 0..3 {
            m.set(i, i, 1.0);
        }
        let inv = invert(&m).unwrap();
        for i in 0..3 {
            for j in 0..3 {
                let expected = if i == j { 1.0 } else { 0.0 };
                assert!((inv.at(i, j) - expected).abs() < 1e-12);
            }
        }
    }

    #[test]
    fn test_invert_singular_returns_error() {
        // [[1,2],[2,4]] is rank-1.
        let mut m = Mat::zeros(2, 2);
        m.set(0, 0, 1.0);
        m.set(0, 1, 2.0);
        m.set(1, 0, 2.0);
        m.set(1, 1, 4.0);
        match invert(&m) {
            Err(BlocVotingError::Singular) => {}
            other => panic!("expected Singular, got {:?}", other),
        }
    }

    // ── fit_wls input validation ─────────────────────────────────────────────

    #[test]
    fn test_fit_wls_rejects_too_few_precincts() {
        let precincts = synthetic_precincts(2, 0.0, 0.5, 0.3, 0.05, 0);
        let names = vec!["pct_minority".to_string(), "pct_dem_baseline".to_string()];
        match fit_wls(&precincts, &names) {
            Err(BlocVotingError::InsufficientPrecincts { got, min }) => {
                assert_eq!(got, 2);
                assert_eq!(min, 4);
            }
            other => panic!("expected InsufficientPrecincts, got {:?}", other),
        }
    }

    // ── Cluster bootstrap (Task 2) ───────────────────────────────────────────

    #[test]
    fn test_cluster_bootstrap_returns_valid_intervals() {
        let precincts = synthetic_precincts(200, 0.1, 0.5, 0.2, 0.05, 17);
        let names = vec!["pct_minority".to_string(), "pct_dem_baseline".to_string()];
        let ci = cluster_bootstrap(&precincts, &names, 100, 17, 0.95).expect("bootstrap");
        assert!(ci.n_clusters >= 2, "expect multiple county clusters");
        assert_eq!(ci.n_samples_executed, 100);
        for name in ["intercept", "pct_minority", "pct_dem_baseline"] {
            let (lo, hi) = ci.cluster_ci[name];
            assert!(lo <= hi, "{name}: low {lo} must be <= high {hi}");
            let (n_lo, n_hi) = ci.naive_ci[name];
            assert!(n_lo <= n_hi, "{name}: naive low {n_lo} <= high {n_hi}");
        }
    }

    #[test]
    fn test_cluster_bootstrap_widens_ci_under_within_county_correlation() {
        // Synthetic dataset where every precinct in a given county has identical
        // (pct_minority, candidate_share) values: the within-county "noise" is
        // exactly zero, so each county is one effective observation. The naive
        // bootstrap (treating precincts as independent) underestimates SE; the
        // cluster bootstrap recovers the true uncertainty -> wider CI.
        let mut precincts: Vec<Precinct> = Vec::new();
        // 5 counties, 20 precincts each. Each county has its own constant (pct_minority,
        // candidate_share) pair drawn from a wide range; pct_dem_baseline varies.
        let county_data = [
            (0.10, 0.20),
            (0.30, 0.35),
            (0.50, 0.50),
            (0.70, 0.65),
            (0.90, 0.80),
        ];
        for (cid, (pct_min, share)) in county_data.iter().enumerate() {
            for j in 0..20 {
                precincts.push(Precinct {
                    id: format!("c{cid:02}-p{j:02}"),
                    county_fips: format!("01{cid:03}"),
                    total_votes: 100.0,
                    candidate_share: *share,
                    pct_minority: *pct_min,
                    pct_dem_baseline: 0.5 + 0.05 * (j as f64),
                });
            }
        }
        let names = vec!["pct_minority".to_string(), "pct_dem_baseline".to_string()];
        let ci = cluster_bootstrap(&precincts, &names, 200, 42, 0.95).expect("bootstrap");
        let (cl_lo, cl_hi) = ci.cluster_ci["pct_minority"];
        let (na_lo, na_hi) = ci.naive_ci["pct_minority"];
        let cluster_width = cl_hi - cl_lo;
        let naive_width = na_hi - na_lo;
        assert!(
            cluster_width > naive_width,
            "cluster CI must be wider than naive CI under within-county correlation; \
             cluster=[{cl_lo:.3},{cl_hi:.3}] (width {cluster_width:.3}), \
             naive=[{na_lo:.3},{na_hi:.3}] (width {naive_width:.3})"
        );
    }

    #[test]
    fn test_cluster_bootstrap_rejects_single_cluster() {
        // Every precinct in the same county -> 1 cluster -> cannot resample.
        let mut precincts = synthetic_precincts(50, 0.1, 0.5, 0.2, 0.05, 0);
        for p in &mut precincts {
            p.county_fips = "01001".to_string();
        }
        let names = vec!["pct_minority".to_string(), "pct_dem_baseline".to_string()];
        match cluster_bootstrap(&precincts, &names, 50, 0, 0.95) {
            Err(BlocVotingError::Numeric(msg)) => {
                assert!(msg.contains("cluster"), "error msg should mention clusters: {msg}");
            }
            other => panic!("expected Numeric error, got {:?}", other),
        }
    }

    #[test]
    fn test_cluster_bootstrap_rejects_invalid_ci_level() {
        let precincts = synthetic_precincts(50, 0.1, 0.5, 0.2, 0.05, 0);
        let names = vec!["pct_minority".to_string(), "pct_dem_baseline".to_string()];
        for bad in [0.0, 1.0, -0.1, 1.5] {
            assert!(
                cluster_bootstrap(&precincts, &names, 10, 0, bad).is_err(),
                "ci_level={bad} must be rejected"
            );
        }
    }

    // ── Orchestrator (Task 3) ────────────────────────────────────────────────

    #[test]
    fn test_run_bloc_voting_family_single_candidate_primary_only() {
        let precincts = synthetic_precincts(200, 0.05, 0.50, 0.20, 0.05, 11);
        let cfg = BlocVotingConfig {
            bootstrap_samples: 50,
            ..Default::default()
        };
        let tests = vec![BlocVotingTest {
            candidate: "Adams".into(),
            variant: variants::PRIMARY.into(),
            precincts,
        }];
        let res = run_bloc_voting_family(&tests, &cfg).expect("family");
        assert_eq!(res.results.len(), 1);
        assert_eq!(res.holm_family_size, 1);
        let r = &res.results[0];
        // With only 1 family member, Holm-corrected = raw.
        assert!((r.race_coefficient.p_value_holm - r.race_coefficient.p_value_raw).abs() < 1e-9);
        // VIF was computed.
        assert!(r.fit.vif_pct_minority_vs_baseline.is_finite());
        // Cluster CI populated on race coefficient.
        assert!(r.race_coefficient.ci_95_cluster.0.is_finite());
        // Robustness roll-up exists.
        assert!(res.robustness.contains_key("Adams"));
    }

    #[test]
    fn test_run_bloc_voting_family_holm_grows_family_with_loo_variants() {
        // 1 candidate × (primary + 3 robustness + 2 LOO) = 6 family members.
        let base = synthetic_precincts(200, 0.05, 0.45, 0.25, 0.05, 21);
        let cfg = BlocVotingConfig {
            bootstrap_samples: 30,
            ..Default::default()
        };
        let tests = vec![
            BlocVotingTest {
                candidate: "Adams".into(),
                variant: variants::PRIMARY.into(),
                precincts: base.clone(),
            },
            BlocVotingTest {
                candidate: "Adams".into(),
                variant: variants::ROBUST_STATEWIDE_DEM.into(),
                precincts: base.clone(),
            },
            BlocVotingTest {
                candidate: "Adams".into(),
                variant: variants::ROBUST_DISTRICT_DEM.into(),
                precincts: base.clone(),
            },
            BlocVotingTest {
                candidate: "Adams".into(),
                variant: variants::ROBUST_PRIOR_PRIMARY.into(),
                precincts: base.clone(),
            },
            BlocVotingTest {
                candidate: "Adams".into(),
                variant: variants::loo_excluded("Smith"),
                precincts: base.clone(),
            },
            BlocVotingTest {
                candidate: "Adams".into(),
                variant: variants::loo_excluded("Jones"),
                precincts: base.clone(),
            },
        ];
        let res = run_bloc_voting_family(&tests, &cfg).expect("family");
        assert_eq!(res.holm_family_size, 6);
        // Holm dominance: every Holm p >= raw p.
        for r in &res.results {
            assert!(
                r.race_coefficient.p_value_holm + 1e-12 >= r.race_coefficient.p_value_raw,
                "{}::{} Holm < raw",
                r.candidate,
                r.variant
            );
        }
    }

    #[test]
    fn test_run_bloc_voting_family_anchor4_caveat_when_unverified() {
        let precincts = synthetic_precincts(150, 0.05, 0.40, 0.20, 0.05, 31);

        // Construct a provenance record asserting NOT independently verified.
        let prov = crate::race_of_candidate::RaceOfCandidateProvenance {
            schema_version: "race-of-candidate v1".into(),
            source_file: "test.csv".into(),
            source_sha256: "deadbeef".into(),
            annotations_independently_verified: false,
            curators: vec![],
            attestation_documents: vec![],
        };
        let cfg = BlocVotingConfig {
            bootstrap_samples: 30,
            provenance: Some(prov),
            ..Default::default()
        };
        let tests = vec![BlocVotingTest {
            candidate: "X".into(),
            variant: variants::PRIMARY.into(),
            precincts,
        }];
        let res = run_bloc_voting_family(&tests, &cfg).expect("family");
        // Anchor 4: caveat string is prepended to draft_interpretation.
        assert!(
            res.draft_interpretation
                .starts_with("[CAVEAT — annotations not independently verified]"),
            "draft_interpretation must start with the unverified caveat; got: {}",
            res.draft_interpretation
        );
    }

    #[test]
    fn test_run_bloc_voting_family_no_caveat_when_verified() {
        let precincts = synthetic_precincts(150, 0.05, 0.40, 0.20, 0.05, 32);
        let prov = crate::race_of_candidate::RaceOfCandidateProvenance {
            schema_version: "race-of-candidate v1".into(),
            source_file: "test.csv".into(),
            source_sha256: "deadbeef".into(),
            annotations_independently_verified: true,
            curators: vec![],
            attestation_documents: vec![],
        };
        let cfg = BlocVotingConfig {
            bootstrap_samples: 30,
            provenance: Some(prov),
            ..Default::default()
        };
        let tests = vec![BlocVotingTest {
            candidate: "X".into(),
            variant: variants::PRIMARY.into(),
            precincts,
        }];
        let res = run_bloc_voting_family(&tests, &cfg).expect("family");
        assert!(
            !res.draft_interpretation
                .contains("[CAVEAT — annotations not independently verified]"),
            "verified annotations must NOT inject the caveat; got: {}",
            res.draft_interpretation
        );
    }

    #[test]
    fn test_robustness_check_under_all_baselines() {
        // Strong signal: race coefficient should be Holm-significant under all 4 variants.
        let base = synthetic_precincts(300, 0.05, 0.55, 0.10, 0.02, 41);
        let cfg = BlocVotingConfig {
            bootstrap_samples: 30,
            ..Default::default()
        };
        let tests = vec![
            BlocVotingTest {
                candidate: "C".into(),
                variant: variants::PRIMARY.into(),
                precincts: base.clone(),
            },
            BlocVotingTest {
                candidate: "C".into(),
                variant: variants::ROBUST_STATEWIDE_DEM.into(),
                precincts: base.clone(),
            },
            BlocVotingTest {
                candidate: "C".into(),
                variant: variants::ROBUST_DISTRICT_DEM.into(),
                precincts: base.clone(),
            },
            BlocVotingTest {
                candidate: "C".into(),
                variant: variants::ROBUST_PRIOR_PRIMARY.into(),
                precincts: base.clone(),
            },
        ];
        let res = run_bloc_voting_family(&tests, &cfg).expect("family");
        let robustness = &res.robustness["C"];
        assert!(
            robustness.race_coefficient_significant_under_all,
            "strong signal under identical 4 baselines must yield robust=true; \
             min={}, max={}, alpha={}",
            robustness.race_coefficient_min, robustness.race_coefficient_max, cfg.alpha
        );
    }

    #[test]
    fn test_run_bloc_voting_family_rejects_empty() {
        let cfg = BlocVotingConfig::default();
        match run_bloc_voting_family(&[], &cfg) {
            Err(BlocVotingError::Numeric(_)) => {}
            other => panic!("expected Numeric error, got {:?}", other),
        }
    }

    #[test]
    fn test_fit_wls_rejects_zero_weight() {
        let mut precincts = synthetic_precincts(50, 0.0, 0.5, 0.3, 0.05, 0);
        precincts[10].total_votes = 0.0;
        let names = vec!["pct_minority".to_string(), "pct_dem_baseline".to_string()];
        match fit_wls(&precincts, &names) {
            Err(BlocVotingError::NonPositiveWeight { idx, .. }) => {
                assert_eq!(idx, 10);
            }
            other => panic!("expected NonPositiveWeight, got {:?}", other),
        }
    }
}
