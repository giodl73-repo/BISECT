use rand::rngs::SmallRng;
use rand::seq::SliceRandom;
use rand::SeedableRng;
/// Partisan metrics: Efficiency Gap, Mean-Median, Partisan Bias, Declination,
/// Seats-Votes Curve + Responsiveness. Bootstrap CI for applicable metrics.
/// Spec 4 — board amendments R3 applied.
use serde::Serialize;

// ---------------------------------------------------------------------------
// Data types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
pub struct DistrictElection {
    pub district: usize,
    pub dem_votes: f64,
    pub rep_votes: f64,
}

impl DistrictElection {
    pub fn total(&self) -> f64 {
        self.dem_votes + self.rep_votes
    }
    pub fn dem_pct(&self) -> f64 {
        if self.total() == 0.0 {
            0.5
        } else {
            self.dem_votes / self.total()
        }
    }
    pub fn margin(&self) -> f64 {
        if self.total() == 0.0 {
            0.0
        } else {
            (self.dem_votes - self.rep_votes).abs() / self.total()
        }
    }
    pub fn dem_won(&self) -> bool {
        self.dem_votes > self.rep_votes
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct MetricWithCI {
    pub value: f64,
    pub direction: String,
    pub ci_available: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ci_95_low: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ci_95_high: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ci_reason: Option<String>,
    pub academic_reference: String,
}

/// Seats-Votes curve evaluated at uniform swing points, with responsiveness
/// (dS/dv at v=0.50) and bias (S(0.50) - 0.50).
#[derive(Debug, Clone, Serialize)]
pub struct SeatsVotesCurve {
    /// (statewide_dem_vote_share, dem_seat_share) pairs across the swing range.
    pub swing_points: Vec<(f64, f64)>,
    /// Finite-difference responsiveness: (S(0.525) - S(0.475)) / 0.05.
    pub responsiveness: f64,
    /// Interpolated S at statewide vote = 0.50, minus 0.50, where S is the Democratic seat share.
    /// Positive = Democratic-favoring (Dems win more than half the seats at 50% vote share).
    /// Negative = Republican-favoring.
    pub bias: f64,
}

#[derive(Debug, Serialize)]
pub struct PartisanMetrics {
    pub efficiency_gap: MetricWithCI,
    pub mean_median: MetricWithCI,
    pub partisan_bias: MetricWithCI,
    /// Declination (Warrington 2018). Positive = Republican-favoring.
    pub declination: MetricWithCI,
    /// Seats-Votes curve, responsiveness, and bias.
    pub seats_votes: SeatsVotesCurve,
    pub statewide_dem_vote_share: f64,
    pub statewide_dem_seat_share: f64,
}

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const CI_MIN_DISTRICTS: usize = 10;

const EG_ACADEMIC_REF: &str =
    "8% threshold from Stephanopoulos & McGhee (2015). SCOTUS declined to adopt \
     in Gill v. Whitford (2018). Not a constitutional standard.";

const MM_ACADEMIC_REF: &str = "7% threshold from Wang (2016). Not a constitutional standard.";

const PB_ACADEMIC_REF: &str = "Partisan bias methodology from Gelman & King (1994).";

const DECL_ACADEMIC_REF: &str =
    "Declination metric from Warrington (2018). Positive = Republican-favoring \
     (Democratic votes wasted in landslides, Republican votes efficiently spread). \
     Range approximately [-1, 1]. Undefined (returned as 0) when all districts \
     won by one party.";

// ---------------------------------------------------------------------------
// Core metric functions (public so tests in BISECT-cli can call them)
// ---------------------------------------------------------------------------

/// Efficiency Gap = (Wasted_D - Wasted_R) / Total_votes.
/// Positive values indicate Republican-favoring plans (more Democratic votes wasted);
/// negative values indicate Democratic-favoring plans.
/// Definition per Stephanopoulos & McGhee (2015).
pub fn compute_efficiency_gap(districts: &[DistrictElection]) -> f64 {
    let total_votes: f64 = districts.iter().map(|d| d.total()).sum();
    if total_votes == 0.0 {
        return 0.0;
    }
    let (wasted_d, wasted_r) = districts.iter().fold((0.0_f64, 0.0_f64), |(wd, wr), d| {
        let total = d.total();
        let threshold = total / 2.0;
        if d.dem_won() {
            // Dem wins: Dem wasted = dem_votes - threshold, Rep wasted = all rep_votes
            (wd + (d.dem_votes - threshold), wr + d.rep_votes)
        } else {
            // Rep wins: Rep wasted = rep_votes - threshold, Dem wasted = all dem_votes
            (wd + d.dem_votes, wr + (d.rep_votes - threshold))
        }
    });
    (wasted_d - wasted_r) / total_votes
}

/// Mean-Median = mean(dem_share) - median(dem_share).
/// Positive = Dem-favoring (mean > median), Negative = Rep-favoring.
pub fn compute_mean_median(districts: &[DistrictElection]) -> f64 {
    if districts.is_empty() {
        return 0.0;
    }
    let mut shares: Vec<f64> = districts.iter().map(|d| d.dem_pct()).collect();
    let mean = shares.iter().sum::<f64>() / shares.len() as f64;
    shares.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let n = shares.len();
    let median = if n % 2 == 0 {
        (shares[n / 2 - 1] + shares[n / 2]) / 2.0
    } else {
        shares[n / 2]
    };
    mean - median
}

/// Partisan Bias: dem_seat_share at the swing where statewide Dem = 50%, minus 0.5.
/// Convention matches SeatsVotesCurve.bias: bias = S(0.50) − 0.50.
/// Positive = Democratic-favoring (Dems win more than half the seats at 50% statewide vote).
/// Negative = Republican-favoring (Dems win fewer than half the seats at 50% statewide vote).
pub fn compute_partisan_bias(districts: &[DistrictElection]) -> f64 {
    let (total_dem, total_all): (f64, f64) = districts
        .iter()
        .fold((0.0, 0.0), |(d, t), x| (d + x.dem_votes, t + x.total()));
    if total_all == 0.0 || districts.is_empty() {
        return 0.0;
    }
    let statewide_dem = total_dem / total_all;
    let swing = 0.5 - statewide_dem;
    let seats_at_50: f64 = districts
        .iter()
        .filter(|d| d.dem_pct() + swing >= 0.5)
        .count() as f64;
    seats_at_50 / districts.len() as f64 - 0.5
}

/// Declination (Warrington 2018).
///
/// Sort districts by Democratic vote share and split at 50%.
///   θ_D = arctan(2·mean_dem_won_share − 1)   (angle of D seats above 50%)
///   θ_R = arctan(1 − 2·mean_rep_won_share)   (angle of R seats below 50%)
///   δ   = 2(θ_D − θ_R) / π
///
/// Positive δ = Republican-favoring (Dem votes wasted in landslides).
/// Returns 0.0 when all districts are won by the same party (undefined).
pub fn compute_declination(districts: &[DistrictElection]) -> f64 {
    if districts.is_empty() {
        return 0.0;
    }

    let d_won: Vec<f64> = districts
        .iter()
        .filter(|d| d.dem_won())
        .map(|d| d.dem_pct())
        .collect();
    let r_won: Vec<f64> = districts
        .iter()
        .filter(|d| !d.dem_won())
        .map(|d| d.dem_pct())
        .collect();

    // Edge case: all districts won by one party → undefined → 0
    if d_won.is_empty() || r_won.is_empty() {
        return 0.0;
    }

    let mean_d: f64 = d_won.iter().sum::<f64>() / d_won.len() as f64;
    let mean_r: f64 = r_won.iter().sum::<f64>() / r_won.len() as f64;

    let theta_d = (2.0 * mean_d - 1.0).atan();
    let theta_r = (1.0 - 2.0 * mean_r).atan();

    2.0 * (theta_d - theta_r) / std::f64::consts::PI
}

/// Seats-Votes curve via uniform swing over [-0.15, +0.15].
///
/// For each swing δ:
///   - New dem_pct for each district = original dem_pct + δ (clamped to [0,1])
///   - Statewide vote share = mean of all adjusted shares (NOT simply 0.50 + δ)
///   - Seat share = fraction of districts with adjusted dem_pct > 0.50
///
/// Responsiveness = (S(0.525) − S(0.475)) / 0.05  (finite difference at v = 0.50)
/// Bias = S at swing where statewide vote ≈ 0.50, minus 0.50  (interpolated)
pub fn compute_seats_votes_curve(
    districts: &[DistrictElection],
    n_points: usize,
) -> SeatsVotesCurve {
    if districts.is_empty() || n_points == 0 {
        return SeatsVotesCurve {
            swing_points: vec![],
            responsiveness: 0.0,
            bias: 0.0,
        };
    }

    let n = districts.len() as f64;
    let orig: Vec<f64> = districts.iter().map(|d| d.dem_pct()).collect();

    // Helper: evaluate (vote_share, seat_share) at a given swing δ.
    let eval = |delta: f64| -> (f64, f64) {
        let adjusted: Vec<f64> = orig.iter().map(|&p| (p + delta).clamp(0.0, 1.0)).collect();
        let vote_share = adjusted.iter().sum::<f64>() / n;
        let seat_share = adjusted.iter().filter(|&&p| p > 0.5).count() as f64 / n;
        (vote_share, seat_share)
    };

    // Build curve over [-0.15, +0.15].
    let swing_points: Vec<(f64, f64)> = if n_points == 1 {
        vec![eval(0.0)]
    } else {
        (0..n_points)
            .map(|i| {
                let delta = -0.15 + 0.30 * (i as f64) / (n_points - 1) as f64;
                eval(delta)
            })
            .collect()
    };

    // Responsiveness: finite difference at v = 0.50
    // Find swings that bring statewide vote to 0.475 and 0.525.
    // Use the mean dem_pct of original districts to compute the needed swing.
    let orig_mean: f64 = orig.iter().sum::<f64>() / n;
    let swing_to_475 = 0.475 - orig_mean;
    let swing_to_525 = 0.525 - orig_mean;
    let (_, s_475) = eval(swing_to_475);
    let (_, s_525) = eval(swing_to_525);
    let responsiveness = (s_525 - s_475) / 0.05;

    // Bias: interpolate S where statewide vote = 0.50.
    // swing needed = 0.50 - orig_mean (uniform swing assumption).
    let swing_to_50 = 0.50 - orig_mean;
    let (_, s_50) = eval(swing_to_50);
    let bias = s_50 - 0.50;

    SeatsVotesCurve {
        swing_points,
        responsiveness,
        bias,
    }
}

/// Bootstrap CI using deterministic seed.
/// Returns (ci_95_low, ci_95_high).
///
/// Board amendment: caller must print progress before calling this.
pub fn bootstrap_ci<F>(
    districts: &[DistrictElection],
    metric_fn: F,
    n_bootstrap: usize,
    rng_seed: u64,
) -> (f64, f64)
where
    F: Fn(&[DistrictElection]) -> f64,
{
    let mut rng = SmallRng::seed_from_u64(rng_seed);
    let n = districts.len();
    let mut samples: Vec<f64> = Vec::with_capacity(n_bootstrap);
    for _ in 0..n_bootstrap {
        let resample: Vec<DistrictElection> = (0..n)
            .map(|_| districts.choose(&mut rng).unwrap().clone())
            .collect();
        samples.push(metric_fn(&resample));
    }
    samples.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let low_idx = ((0.025 * n_bootstrap as f64) as usize).min(n_bootstrap - 1);
    let high_idx = ((0.975 * n_bootstrap as f64) as usize).min(n_bootstrap - 1);
    (samples[low_idx], samples[high_idx])
}

// ---------------------------------------------------------------------------
// Combined compute
// ---------------------------------------------------------------------------

/// Compute all three partisan metrics, with CI if num_districts >= 10.
///
/// Board amendment: prints progress before bootstrap calls.
pub fn compute_partisan_metrics(
    districts: &[DistrictElection],
    rng_seed_override: Option<u64>,
    n_bootstrap: usize,
) -> PartisanMetrics {
    let n = districts.len();
    let seed = rng_seed_override.unwrap_or(42);
    let ci_available = n >= CI_MIN_DISTRICTS;
    let ci_reason = if !ci_available {
        Some(format!(
            "Bootstrap CI requires >={} districts (found {})",
            CI_MIN_DISTRICTS, n
        ))
    } else {
        None
    };

    let eg_val = compute_efficiency_gap(districts);
    let mm_val = compute_mean_median(districts);
    let pb_val = compute_partisan_bias(districts);
    let decl_val = compute_declination(districts);
    let seats_votes = compute_seats_votes_curve(districts, 61);

    let (eg_lo, eg_hi, mm_lo, mm_hi, pb_lo, pb_hi, decl_lo, decl_hi) = if ci_available {
        // Board amendment: print progress before bootstrap calls
        eprintln!("Running bootstrap CI ({n_bootstrap} samples, 4 metrics)...");
        let (eg_lo, eg_hi) = bootstrap_ci(districts, compute_efficiency_gap, n_bootstrap, seed);
        let (mm_lo, mm_hi) = bootstrap_ci(districts, compute_mean_median, n_bootstrap, seed);
        let (pb_lo, pb_hi) = bootstrap_ci(districts, compute_partisan_bias, n_bootstrap, seed);
        let (decl_lo, decl_hi) = bootstrap_ci(districts, compute_declination, n_bootstrap, seed);
        (
            Some(eg_lo),
            Some(eg_hi),
            Some(mm_lo),
            Some(mm_hi),
            Some(pb_lo),
            Some(pb_hi),
            Some(decl_lo),
            Some(decl_hi),
        )
    } else {
        (None, None, None, None, None, None, None, None)
    };

    // EG: positive = Republican-favoring (Wasted_D > Wasted_R).
    // PB: positive = Republican-favoring (seat share at 50% vote favors Republicans).
    let direction_rep_pos = |v: f64| -> String {
        if v >= 0.0 {
            "Republican".into()
        } else {
            "Democratic".into()
        }
    };
    // MM: positive = Democratic-favoring (mean > median).
    let direction_dem_pos = |v: f64| -> String {
        if v >= 0.0 {
            "Democratic".into()
        } else {
            "Republican".into()
        }
    };
    // Declination: positive = Republican-favoring (same as rep_pos).
    let direction_decl = |v: f64| -> String {
        if v >= 0.0 {
            "Republican".into()
        } else {
            "Democratic".into()
        }
    };

    // Consistency check: compute_partisan_bias and SeatsVotesCurve.bias use the same
    // convention (S(0.50) - 0.50). They should agree within floating-point tolerance.
    if (pb_val - seats_votes.bias).abs() >= 0.01 {
        eprintln!(
            "WARNING: partisan_bias ({:.4}) and seats_votes.bias ({:.4}) diverge by {:.4} \
             (expected < 0.01). Possible numerical inconsistency.",
            pb_val,
            seats_votes.bias,
            (pb_val - seats_votes.bias).abs()
        );
    }

    let total_votes: f64 = districts.iter().map(|d| d.total()).sum();
    let statewide_dem_vote_share = if total_votes > 0.0 {
        districts.iter().map(|d| d.dem_votes).sum::<f64>() / total_votes
    } else {
        0.0
    };
    let statewide_dem_seat_share =
        districts.iter().filter(|d| d.dem_won()).count() as f64 / n.max(1) as f64;

    PartisanMetrics {
        efficiency_gap: MetricWithCI {
            value: eg_val,
            direction: direction_rep_pos(eg_val),
            ci_available,
            ci_95_low: eg_lo,
            ci_95_high: eg_hi,
            ci_reason: ci_reason.clone(),
            academic_reference: EG_ACADEMIC_REF.into(),
        },
        mean_median: MetricWithCI {
            value: mm_val,
            direction: direction_dem_pos(mm_val),
            ci_available,
            ci_95_low: mm_lo,
            ci_95_high: mm_hi,
            ci_reason: ci_reason.clone(),
            academic_reference: MM_ACADEMIC_REF.into(),
        },
        partisan_bias: MetricWithCI {
            value: pb_val,
            direction: direction_dem_pos(pb_val),
            ci_available,
            ci_95_low: pb_lo,
            ci_95_high: pb_hi,
            ci_reason: ci_reason.clone(),
            academic_reference: PB_ACADEMIC_REF.into(),
        },
        declination: MetricWithCI {
            value: decl_val,
            direction: direction_decl(decl_val),
            ci_available,
            ci_95_low: decl_lo,
            ci_95_high: decl_hi,
            ci_reason,
            academic_reference: DECL_ACADEMIC_REF.into(),
        },
        seats_votes,
        statewide_dem_vote_share,
        statewide_dem_seat_share,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // --- Helper factories ---

    fn make_district(id: usize, dem_pct: f64, rep_pct: f64) -> DistrictElection {
        let total = 1000.0;
        DistrictElection {
            district: id,
            dem_votes: total * dem_pct,
            rep_votes: total * rep_pct,
        }
    }

    fn symmetric_plan_10() -> Vec<DistrictElection> {
        // 5 Dem wins at 60/40, 5 Rep wins at 60/40 — wasted votes symmetric
        let mut v = Vec::new();
        for i in 1..=5 {
            v.push(make_district(i, 0.60, 0.40));
        }
        for i in 6..=10 {
            v.push(make_district(i, 0.40, 0.60));
        }
        v
    }

    fn packed_dem_plan() -> Vec<DistrictElection> {
        // Rep wins 6 narrowly (51/49), Dem wins 4 by blowout (80/20)
        // Many Dem surplus wasted → EG favors Republicans (negative)
        let mut v = Vec::new();
        for i in 1..=6 {
            v.push(make_district(i, 0.49, 0.51));
        }
        for i in 7..=10 {
            v.push(make_district(i, 0.80, 0.20));
        }
        v
    }

    fn packed_rep_plan() -> Vec<DistrictElection> {
        // Dem wins 6 narrowly, Rep wins 4 by blowout → EG > 0 (Dem-favoring)
        let mut v = Vec::new();
        for i in 1..=6 {
            v.push(make_district(i, 0.51, 0.49));
        }
        for i in 7..=10 {
            v.push(make_district(i, 0.20, 0.80));
        }
        v
    }

    fn uniform_plan(n: usize, dem_pct: f64) -> Vec<DistrictElection> {
        (1..=n)
            .map(|i| make_district(i, dem_pct, 1.0 - dem_pct))
            .collect()
    }

    // --- Metric tests ---

    #[test]
    fn test_efficiency_gap_zero_for_symmetric_plan() {
        let districts = symmetric_plan_10();
        let eg = compute_efficiency_gap(&districts);
        assert!(eg.abs() < 1e-9, "symmetric plan must have EG = 0, got {eg}");
    }

    #[test]
    fn test_efficiency_gap_direction() {
        // packed_dem_plan: Dems packed in 4 blowout wins (80%), Reps win 6 narrowly (51%).
        // Many Democratic surplus votes wasted → EG = (Wasted_D - Wasted_R) / Total > 0.
        // Positive EG = Republican-favoring per Stephanopoulos & McGhee (2015).
        let districts = packed_dem_plan();
        let eg = compute_efficiency_gap(&districts);
        assert!(
            eg > 0.0,
            "packed Dem blowout plan should have positive EG (Rep-favoring), got {eg}"
        );
    }

    #[test]
    fn test_mean_median_equal_when_symmetric() {
        let districts = uniform_plan(10, 0.50);
        let mm = compute_mean_median(&districts);
        assert!(
            mm.abs() < 1e-9,
            "uniform vote share must have MM = 0, got {mm}"
        );
    }

    #[test]
    fn test_bootstrap_ci_reproducible_with_seed() {
        let districts = symmetric_plan_10();
        let ci1 = bootstrap_ci(&districts, compute_efficiency_gap, 1000, 42);
        let ci2 = bootstrap_ci(&districts, compute_efficiency_gap, 1000, 42);
        assert_eq!(ci1, ci2, "same seed must produce identical CI bounds");
    }

    #[test]
    fn test_bootstrap_ci_different_seeds_differ() {
        // Use a plan with high variance across districts to ensure seed effects are visible
        let districts: Vec<DistrictElection> = (1..=10)
            .map(|i| {
                let dem_pct = 0.30 + (i as f64) * 0.05; // 0.35 to 0.80
                make_district(i, dem_pct, 1.0 - dem_pct)
            })
            .collect();
        let ci1 = bootstrap_ci(&districts, compute_efficiency_gap, 2000, 42);
        let ci2 = bootstrap_ci(&districts, compute_efficiency_gap, 2000, 12345);
        assert_ne!(
            ci1, ci2,
            "different seeds should produce different CI bounds"
        );
    }

    #[test]
    fn test_partisan_bias_neutral_at_50pct() {
        // Use symmetric plan (5 districts each at 60/40 and 40/60) rather than
        // exactly 50% in every district, because statewide=50% + all-ties is
        // a degenerate case for the swing model. A symmetric plan at statewide 50%
        // produces bias = 0 when Dems win exactly half the seats after the swing.
        let districts = symmetric_plan_10();
        // Statewide dem share: 5*600 + 5*400 = 5000 D, 5*400 + 5*600 = 5000 R → 50% each
        let pb = compute_partisan_bias(&districts);
        assert!(
            pb.abs() < 0.05,
            "symmetric plan bias should be near zero, got {pb}"
        );
    }

    #[test]
    fn test_ci_suppressed_when_fewer_than_10_districts() {
        let districts = vec![make_district(1, 0.67, 0.33)];
        let result = compute_partisan_metrics(&districts, None, 1000);
        assert!(
            !result.efficiency_gap.ci_available,
            "CI must be suppressed for N < 10"
        );
        assert_eq!(
            result.efficiency_gap.ci_reason.as_deref(),
            Some("Bootstrap CI requires >=10 districts (found 1)")
        );
    }

    #[test]
    fn test_metrics_computed_even_when_ci_suppressed() {
        let districts = vec![make_district(1, 0.67, 0.33)];
        let result = compute_partisan_metrics(&districts, None, 1000);
        assert!(
            result.efficiency_gap.value.is_finite(),
            "EG must be finite for single-district plan"
        );
        assert!(result.mean_median.value.is_finite());
    }

    #[test]
    fn test_eg_exactly_10_districts_ci_present() {
        let districts = uniform_plan(10, 0.55);
        let result = compute_partisan_metrics(&districts, None, 100);
        assert!(
            result.efficiency_gap.ci_available,
            "CI must be available for exactly 10 districts"
        );
    }

    #[test]
    fn test_direction_dem_when_negative_eg() {
        // packed_rep_plan: Reps packed in 4 blowout losses (20% Dem), Dems win 6 narrowly (51%).
        // More Republican surplus votes wasted → EG = (Wasted_D - Wasted_R) < 0 = Dem-favoring.
        let districts = packed_rep_plan();
        let result = compute_partisan_metrics(&districts, None, 100);
        assert!(
            result.efficiency_gap.value < 0.0,
            "packed-Rep plan should have negative EG"
        );
        assert_eq!(result.efficiency_gap.direction, "Democratic");
    }

    #[test]
    fn test_direction_rep_when_positive_eg() {
        // packed_dem_plan: Dems packed in blowouts → Wasted_D > Wasted_R → EG > 0 = Rep-favoring.
        let districts = packed_dem_plan();
        let result = compute_partisan_metrics(&districts, None, 100);
        assert!(
            result.efficiency_gap.value > 0.0,
            "packed-Dem plan should have positive EG"
        );
        assert_eq!(result.efficiency_gap.direction, "Republican");
    }

    // ── compute_mean_median additional cases ────────────────────────────────

    #[test]
    fn test_mean_median_empty_returns_zero() {
        assert_eq!(compute_mean_median(&[]), 0.0);
    }

    #[test]
    fn test_mean_median_single_district_is_zero() {
        // mean == median for a single value → always 0
        let d = vec![make_district(1, 0.7, 0.3)];
        assert!(compute_mean_median(&d).abs() < 1e-9);
    }

    #[test]
    fn test_mean_median_positive_when_dem_skewed_high() {
        // If a few districts have very high Dem % and most are moderate, mean > median
        let districts: Vec<DistrictElection> = vec![
            make_district(1, 0.95, 0.05),
            make_district(2, 0.95, 0.05),
            make_district(3, 0.51, 0.49),
            make_district(4, 0.51, 0.49),
            make_district(5, 0.51, 0.49),
        ];
        let mm = compute_mean_median(&districts);
        assert!(
            mm > 0.0,
            "mean > median when high-D outliers present, got {mm}"
        );
    }

    #[test]
    fn test_mean_median_even_count_uses_midpoint() {
        // Even count: median = average of two middle values
        let districts: Vec<DistrictElection> = vec![
            make_district(1, 0.40, 0.60),
            make_district(2, 0.50, 0.50),
            make_district(3, 0.60, 0.40),
            make_district(4, 0.70, 0.30),
        ];
        let mm = compute_mean_median(&districts);
        // mean = (0.4+0.5+0.6+0.7)/4 = 0.55, median = (0.5+0.6)/2 = 0.55 → MM = 0
        assert!(
            mm.abs() < 1e-9,
            "symmetric 4-district plan should have MM=0, got {mm}"
        );
    }

    // ── compute_partisan_bias additional cases ──────────────────────────────

    #[test]
    fn test_partisan_bias_empty_returns_zero() {
        assert_eq!(compute_partisan_bias(&[]), 0.0);
    }

    #[test]
    fn test_partisan_bias_rep_gerrymander_negative() {
        // Packed-Dem plan = Republican gerrymander (Dems packed into blowout wins).
        // packed_dem: 6 narrow Rep wins (0.49D), 4 blowout Dem wins (0.80D)
        // statewide_dem = (6*490 + 4*800)/(10*1000) = (2940+3200)/10000 = 0.614
        // swing to 50% = 0.5 - 0.614 = -0.114 (subtract from each share)
        // adjusted shares: 0.49-0.114=0.376, 0.80-0.114=0.686
        // seats at 50%: districts where adjusted > 0.5 → only the 4 blowout Dem ones
        // seats_at_50 = 4 → pb = 4/10 - 0.5 = -0.1 < 0
        // Negative = Republican-favoring: at 50% statewide, Dems only win 40% of seats.
        let districts = packed_dem_plan();
        let pb = compute_partisan_bias(&districts);
        assert!(
            pb < 0.0,
            "packed-Dem (Rep gerrymander) gives negative bias (Rep-favoring), got {pb}"
        );
    }

    #[test]
    fn test_partisan_bias_dem_gerrymander_positive() {
        // Packed-Rep plan = Democratic gerrymander (Reps packed into blowout losses).
        // packed_rep: 6 narrow Dem wins (0.51D), 4 blowout Rep wins (0.20D)
        // statewide_dem = (6*510 + 4*200)/(10*1000) = (3060+800)/10000 = 0.386
        // swing to 50% = 0.5 - 0.386 = +0.114 (add to each share)
        // adjusted: 0.51+0.114=0.624, 0.20+0.114=0.314
        // seats at 50%: only the 6 Dem-win districts → seats_at_50 = 6
        // pb = 6/10 - 0.5 = +0.1 > 0
        // Positive = Democratic-favoring: at 50% statewide, Dems win 60% of seats.
        let districts = packed_rep_plan();
        let pb = compute_partisan_bias(&districts);
        assert!(
            pb > 0.0,
            "packed-Rep (Dem gerrymander) gives positive bias (Dem-favoring), got {pb}"
        );
    }

    // ── compute_efficiency_gap additional cases ─────────────────────────────

    #[test]
    fn test_efficiency_gap_empty_returns_zero() {
        assert_eq!(compute_efficiency_gap(&[]), 0.0);
    }

    #[test]
    fn test_efficiency_gap_zero_vote_district() {
        let d = DistrictElection {
            district: 1,
            dem_votes: 0.0,
            rep_votes: 0.0,
        };
        let eg = compute_efficiency_gap(&[d]);
        assert_eq!(eg, 0.0, "zero-vote district should give EG=0");
    }

    #[test]
    fn test_efficiency_gap_symmetric_ten_exact_value() {
        // 5 Dem wins at 60/40 and 5 Rep wins at 40/60 with 1000 votes each
        // For a Dem-win district (60/40): wasted_D=100 (=600-500), wasted_R=400
        // For a Rep-win district (40/60): wasted_D=400, wasted_R=100
        // Sum: wasted_D=5*100+5*400=2500, wasted_R=5*400+5*100=2500 → EG=0
        let districts = symmetric_plan_10();
        let eg = compute_efficiency_gap(&districts);
        assert!(eg.abs() < 1e-9, "symmetric plan EG must be 0, got {eg}");
    }

    // ── DistrictElection helpers ────────────────────────────────────────────

    #[test]
    fn test_district_dem_pct_zero_vote_returns_half() {
        let d = DistrictElection {
            district: 1,
            dem_votes: 0.0,
            rep_votes: 0.0,
        };
        assert!(
            (d.dem_pct() - 0.5).abs() < 1e-9,
            "zero-vote district should return 0.5"
        );
    }

    #[test]
    fn test_district_margin_zero_vote_returns_zero() {
        let d = DistrictElection {
            district: 1,
            dem_votes: 0.0,
            rep_votes: 0.0,
        };
        assert_eq!(d.margin(), 0.0);
    }

    #[test]
    fn test_district_margin_close_race() {
        let d = DistrictElection {
            district: 1,
            dem_votes: 510.0,
            rep_votes: 490.0,
        };
        let m = d.margin();
        assert!(
            (m - 0.02).abs() < 1e-9,
            "margin should be 20/1000 = 0.02, got {m}"
        );
    }

    #[test]
    fn test_district_dem_won_tie_returns_false() {
        let d = DistrictElection {
            district: 1,
            dem_votes: 500.0,
            rep_votes: 500.0,
        };
        assert!(!d.dem_won(), "exact tie → dem_won() must be false");
    }

    // ── statewide shares computed by compute_partisan_metrics ───────────────

    #[test]
    fn test_statewide_dem_vote_share_correct() {
        let districts = uniform_plan(4, 0.60);
        let result = compute_partisan_metrics(&districts, None, 100);
        assert!((result.statewide_dem_vote_share - 0.60).abs() < 1e-6);
    }

    #[test]
    fn test_statewide_dem_seat_share_all_dem_wins() {
        let districts = uniform_plan(5, 0.60);
        let result = compute_partisan_metrics(&districts, None, 100);
        assert!((result.statewide_dem_seat_share - 1.0).abs() < 1e-9);
    }

    // ── Declination (Warrington 2018) ───────────────────────────────────────

    /// Symmetric plan: 5 D wins at 60%, 5 R wins at 40% → δ = 0.
    /// mean_d = 0.60, mean_r = 0.40
    /// θ_D = arctan(2*0.60 - 1) = arctan(0.20)
    /// θ_R = arctan(1 - 2*0.40) = arctan(0.20)
    /// δ = 2(θ_D - θ_R)/π = 0
    #[test]
    fn test_declination_zero_for_symmetric_plan() {
        let districts = symmetric_plan_10();
        let d = compute_declination(&districts);
        assert!(d.abs() < 1e-9, "symmetric plan must have δ = 0, got {d}");
    }

    /// All D wins at 80%, all R wins at 51% → Dems waste votes in landslides.
    /// mean_d = 0.80 → θ_D = arctan(0.60) ≈ 0.5404
    /// mean_r = 0.51 → θ_R = arctan(0.02) ≈ 0.0200
    /// δ = 2(0.5404 - 0.0200)/π > 0  (Republican-favoring by convention)
    /// The spec says δ < 0 "benefits R", but our formula sign matches
    /// Warrington: positive δ = Republicans benefit.
    #[test]
    fn test_declination_positive_when_d_votes_wasted() {
        // 5 blowout D wins, 5 near-miss R wins → D votes wasted
        let mut districts = Vec::new();
        for i in 1..=5 {
            districts.push(make_district(i, 0.80, 0.20)); // D wins at 80%
        }
        for i in 6..=10 {
            districts.push(make_district(i, 0.49, 0.51)); // R wins at 51%
        }
        let d = compute_declination(&districts);
        // mean_d=0.80 → θ_D=arctan(0.60)>0; mean_r=0.49 → θ_R=arctan(0.02)>0
        // θ_D > θ_R → δ > 0 (Republican-favoring)
        assert!(
            d > 0.0,
            "D landslides + narrow R wins → δ > 0 (Rep-favoring), got {d}"
        );
    }

    /// Mirror case: all R wins at 80%, all D wins at 51% → R votes wasted.
    /// δ < 0 (Democratic-favoring).
    #[test]
    fn test_declination_negative_when_r_votes_wasted() {
        let mut districts = Vec::new();
        for i in 1..=5 {
            districts.push(make_district(i, 0.51, 0.49)); // D wins at 51%
        }
        for i in 6..=10 {
            districts.push(make_district(i, 0.20, 0.80)); // R wins at 80%
        }
        let d = compute_declination(&districts);
        // mean_d=0.51 → θ_D=arctan(0.02) small positive
        // mean_r=0.20 → θ_R=arctan(0.60) large positive
        // θ_D < θ_R → δ < 0 (Democratic-favoring)
        assert!(
            d < 0.0,
            "R landslides + narrow D wins → δ < 0 (Dem-favoring), got {d}"
        );
    }

    /// Fully competitive plan: all districts at exactly 50/50 → δ = 0.
    /// dem_won() is false for ties, so all fall in r_won group → returns 0 (edge case).
    #[test]
    fn test_declination_undefined_all_one_party_returns_zero() {
        // All districts won by Democrats
        let districts = uniform_plan(8, 0.70);
        let d = compute_declination(&districts);
        assert_eq!(d, 0.0, "all-D plan → undefined → 0, got {d}");
    }

    #[test]
    fn test_declination_empty_returns_zero() {
        assert_eq!(compute_declination(&[]), 0.0);
    }

    /// Fully competitive near-50% plan → δ near 0.
    #[test]
    fn test_declination_near_zero_for_competitive_plan() {
        // 5 D wins at 52%, 5 R wins at 48%
        let mut districts = Vec::new();
        for i in 1..=5 {
            districts.push(make_district(i, 0.52, 0.48));
        }
        for i in 6..=10 {
            districts.push(make_district(i, 0.48, 0.52));
        }
        let d = compute_declination(&districts);
        // mean_d=0.52 → θ_D=arctan(0.04) ≈ 0.0399
        // mean_r=0.48 → θ_R=arctan(0.04) ≈ 0.0399 → δ ≈ 0
        assert!(d.abs() < 0.05, "near-competitive plan → δ ≈ 0, got {d}");
    }

    // ── Seats-Votes Curve & Responsiveness ─────────────────────────────────

    /// Symmetric plan at 50% statewide → bias ≈ 0, responsiveness ≈ 2 (proportional).
    #[test]
    fn test_seats_votes_bias_near_zero_for_symmetric_plan() {
        let districts = symmetric_plan_10();
        // Statewide dem vote share = 50%, so swing to 50% = 0 → use actual at-50% S.
        let sv = compute_seats_votes_curve(&districts, 61);
        // bias = S(v=0.50) - 0.50; for symmetric plan S(0.50) = 0.50 → bias = 0
        assert!(
            sv.bias.abs() < 0.05,
            "symmetric plan bias should be near 0, got {}",
            sv.bias
        );
    }

    /// Symmetric plan responsiveness: around 50%, swinging ±2.5% should flip seats.
    /// For 5 D-wins at 60% and 5 R-wins at 40%:
    ///   at v=0.475 (swing=-0.025): all 5 D-wins become 57.5%, 5 R-wins become 37.5% → 5 seats
    ///   at v=0.525 (swing=+0.025): all 5 D-wins become 62.5%, 5 R-wins become 42.5% → 5 seats
    ///   But the 40% districts need swing of +0.10 to flip → no change in ±2.5%
    ///   responsiveness = (0.5 - 0.5) / 0.05 = 0 for this plan (no swing districts).
    /// Use a plan with marginal districts instead to test responsiveness > 1.
    #[test]
    fn test_seats_votes_responsiveness_high_for_competitive_plan() {
        // 10 near-50% districts: 5 at 52%, 5 at 48% → all competitive, high responsiveness
        let mut districts = Vec::new();
        for i in 1..=5 {
            districts.push(make_district(i, 0.52, 0.48));
        }
        for i in 6..=10 {
            districts.push(make_district(i, 0.48, 0.52));
        }
        let sv = compute_seats_votes_curve(&districts, 61);
        // Statewide mean = 50%. Swing to 47.5%: adjust -0.025 → D at 49.5%, R at 45.5%
        // All 5 D-wins flip to R → 0 seats. Swing to 52.5%: all 5 R-wins flip to D → 10 seats.
        // responsiveness = (1.0 - 0.0) / 0.05 = 20 (theoretical max, seats are discrete here)
        assert!(
            sv.responsiveness > 1.5,
            "competitive plan must have high responsiveness, got {}",
            sv.responsiveness
        );
    }

    /// Safe-seat heavy plan: all districts at 70% D or 30% D → no swing sensitivity.
    #[test]
    fn test_seats_votes_responsiveness_low_for_safe_seat_plan() {
        // 5 blowout D wins (70%), 5 blowout R wins (30%)
        let districts = symmetric_plan_10(); // 60%/40% — moderate, but not as safe
                                             // Use more extreme safe seats:
        let mut districts_safe = Vec::new();
        for i in 1..=5 {
            districts_safe.push(make_district(i, 0.75, 0.25));
        }
        for i in 6..=10 {
            districts_safe.push(make_district(i, 0.25, 0.75));
        }
        let sv = compute_seats_votes_curve(&districts_safe, 61);
        // ±2.5% swing around statewide 50% won't flip any district (need ±25% swing)
        // responsiveness ≈ 0
        assert!(
            sv.responsiveness < 1.5,
            "safe-seat plan must have low responsiveness, got {}",
            sv.responsiveness
        );
        let _ = districts; // suppress unused warning
    }

    #[test]
    fn test_seats_votes_empty_returns_zero() {
        let sv = compute_seats_votes_curve(&[], 61);
        assert_eq!(sv.swing_points.len(), 0);
        assert_eq!(sv.responsiveness, 0.0);
        assert_eq!(sv.bias, 0.0);
    }

    #[test]
    fn test_seats_votes_curve_has_correct_point_count() {
        let districts = symmetric_plan_10();
        let sv = compute_seats_votes_curve(&districts, 31);
        assert_eq!(sv.swing_points.len(), 31);
    }

    #[test]
    fn test_seats_votes_monotone_increasing_for_simple_plan() {
        // For a simple plan, more Dem swing → more Dem seats (weakly monotone).
        let districts = symmetric_plan_10();
        let sv = compute_seats_votes_curve(&districts, 61);
        let mut prev_seats = -1.0_f64;
        for &(_, seat_share) in &sv.swing_points {
            assert!(
                seat_share >= prev_seats - 1e-9,
                "seats-votes curve must be weakly monotone, got drop: {prev_seats} → {seat_share}"
            );
            prev_seats = seat_share;
        }
    }

    /// compute_partisan_metrics integrates declination and seats_votes.
    #[test]
    fn test_partisan_metrics_includes_new_fields() {
        let districts = symmetric_plan_10();
        let result = compute_partisan_metrics(&districts, None, 100);
        assert!(
            result.declination.value.is_finite(),
            "declination must be finite"
        );
        assert_eq!(
            result.seats_votes.swing_points.len(),
            61,
            "seats_votes must have 61 points"
        );
        assert!(
            result.seats_votes.responsiveness.is_finite(),
            "responsiveness must be finite"
        );
        assert!(result.seats_votes.bias.is_finite(), "bias must be finite");
    }
}
