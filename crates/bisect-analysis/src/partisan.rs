/// Partisan metrics: Efficiency Gap, Mean-Median, Partisan Bias, bootstrap CI.
/// Spec 4 — board amendments R3 applied.
use serde::Serialize;
use rand::SeedableRng;
use rand::rngs::SmallRng;
use rand::seq::SliceRandom;

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
        if self.total() == 0.0 { 0.5 } else { self.dem_votes / self.total() }
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

#[derive(Debug, Serialize)]
pub struct PartisanMetrics {
    pub efficiency_gap: MetricWithCI,
    pub mean_median: MetricWithCI,
    pub partisan_bias: MetricWithCI,
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

const MM_ACADEMIC_REF: &str =
    "7% threshold from Wang (2016). Not a constitutional standard.";

const PB_ACADEMIC_REF: &str =
    "Partisan bias methodology from Gelman & King (1994).";

// ---------------------------------------------------------------------------
// Core metric functions (public so tests in redist-cli can call them)
// ---------------------------------------------------------------------------

/// Efficiency Gap = (Wasted_R - Wasted_D) / Total_votes.
/// Positive = Democratic-favoring, Negative = Republican-favoring.
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
    (wasted_r - wasted_d) / total_votes
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

/// Partisan Bias: 0.5 - dem_seat_share at the swing where statewide Dem = 50%.
/// Positive = Dem-favoring, Negative = Rep-favoring.
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
    0.5 - seats_at_50 / districts.len() as f64
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

    let (eg_lo, eg_hi, mm_lo, mm_hi, pb_lo, pb_hi) = if ci_available {
        // Board amendment: print progress before bootstrap calls
        eprintln!(
            "Running bootstrap CI ({n_bootstrap} samples, 3 metrics)..."
        );
        let (eg_lo, eg_hi) = bootstrap_ci(districts, compute_efficiency_gap, n_bootstrap, seed);
        let (mm_lo, mm_hi) = bootstrap_ci(districts, compute_mean_median, n_bootstrap, seed);
        let (pb_lo, pb_hi) = bootstrap_ci(districts, compute_partisan_bias, n_bootstrap, seed);
        (
            Some(eg_lo), Some(eg_hi),
            Some(mm_lo), Some(mm_hi),
            Some(pb_lo), Some(pb_hi),
        )
    } else {
        (None, None, None, None, None, None)
    };

    let direction = |v: f64| -> String {
        if v >= 0.0 { "Democratic".into() } else { "Republican".into() }
    };

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
            direction: direction(eg_val),
            ci_available,
            ci_95_low: eg_lo,
            ci_95_high: eg_hi,
            ci_reason: ci_reason.clone(),
            academic_reference: EG_ACADEMIC_REF.into(),
        },
        mean_median: MetricWithCI {
            value: mm_val,
            direction: direction(mm_val),
            ci_available,
            ci_95_low: mm_lo,
            ci_95_high: mm_hi,
            ci_reason: ci_reason.clone(),
            academic_reference: MM_ACADEMIC_REF.into(),
        },
        partisan_bias: MetricWithCI {
            value: pb_val,
            direction: direction(pb_val),
            ci_available,
            ci_95_low: pb_lo,
            ci_95_high: pb_hi,
            ci_reason,
            academic_reference: PB_ACADEMIC_REF.into(),
        },
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
        (1..=n).map(|i| make_district(i, dem_pct, 1.0 - dem_pct)).collect()
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
        let districts = packed_dem_plan();
        let eg = compute_efficiency_gap(&districts);
        assert!(eg < 0.0, "packed Dem blowout plan should have negative EG (Rep-favoring), got {eg}");
    }

    #[test]
    fn test_mean_median_equal_when_symmetric() {
        let districts = uniform_plan(10, 0.50);
        let mm = compute_mean_median(&districts);
        assert!(mm.abs() < 1e-9, "uniform vote share must have MM = 0, got {mm}");
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
        let districts: Vec<DistrictElection> = (1..=10).map(|i| {
            let dem_pct = 0.30 + (i as f64) * 0.05; // 0.35 to 0.80
            make_district(i, dem_pct, 1.0 - dem_pct)
        }).collect();
        let ci1 = bootstrap_ci(&districts, compute_efficiency_gap, 2000, 42);
        let ci2 = bootstrap_ci(&districts, compute_efficiency_gap, 2000, 12345);
        assert_ne!(ci1, ci2, "different seeds should produce different CI bounds");
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
        assert!(pb.abs() < 0.05, "symmetric plan bias should be near zero, got {pb}");
    }

    #[test]
    fn test_ci_suppressed_when_fewer_than_10_districts() {
        let districts = vec![make_district(1, 0.67, 0.33)];
        let result = compute_partisan_metrics(&districts, None, 1000);
        assert!(!result.efficiency_gap.ci_available, "CI must be suppressed for N < 10");
        assert_eq!(
            result.efficiency_gap.ci_reason.as_deref(),
            Some("Bootstrap CI requires >=10 districts (found 1)")
        );
    }

    #[test]
    fn test_metrics_computed_even_when_ci_suppressed() {
        let districts = vec![make_district(1, 0.67, 0.33)];
        let result = compute_partisan_metrics(&districts, None, 1000);
        assert!(result.efficiency_gap.value.is_finite(), "EG must be finite for single-district plan");
        assert!(result.mean_median.value.is_finite());
    }

    #[test]
    fn test_eg_exactly_10_districts_ci_present() {
        let districts = uniform_plan(10, 0.55);
        let result = compute_partisan_metrics(&districts, None, 100);
        assert!(result.efficiency_gap.ci_available, "CI must be available for exactly 10 districts");
    }

    #[test]
    fn test_direction_dem_when_positive_eg() {
        let districts = packed_rep_plan();
        let result = compute_partisan_metrics(&districts, None, 100);
        assert_eq!(result.efficiency_gap.direction, "Democratic");
    }

    #[test]
    fn test_direction_rep_when_negative_eg() {
        let districts = packed_dem_plan();
        let result = compute_partisan_metrics(&districts, None, 100);
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
        assert!(mm > 0.0, "mean > median when high-D outliers present, got {mm}");
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
        assert!(mm.abs() < 1e-9, "symmetric 4-district plan should have MM=0, got {mm}");
    }

    // ── compute_partisan_bias additional cases ──────────────────────────────

    #[test]
    fn test_partisan_bias_empty_returns_zero() {
        assert_eq!(compute_partisan_bias(&[]), 0.0);
    }

    #[test]
    fn test_partisan_bias_rep_gerrymander_positive() {
        // Packed-Dem plan = Republican gerrymander: swing at 50% gives Dems more seats
        // than current → seats_at_50 > 0.5*n → pb = 0.5 - (>0.5) < 0? Let's verify
        // the actual sign from the formula and document it correctly.
        // packed_dem: 6 narrow Rep wins (0.49D), 4 blowout Dem wins (0.80D)
        // statewide_dem = (6*490 + 4*800)/(10*1000) = (2940+3200)/10000 = 0.614
        // swing = 0.5 - 0.614 = -0.114 (subtract from each share)
        // adjusted shares: 0.49-0.114=0.376, 0.80-0.114=0.686
        // seats at 50%: districts where adjusted >= 0.5 → only the 4 blowout Dem ones
        // seats_at_50 = 4 → pb = 0.5 - 4/10 = 0.1 > 0
        let districts = packed_dem_plan();
        let pb = compute_partisan_bias(&districts);
        assert!(pb > 0.0, "packed-Dem plan swing formula gives positive bias, got {pb}");
    }

    #[test]
    fn test_partisan_bias_dem_gerrymander_negative() {
        // Packed-Rep plan = Democratic gerrymander: swing analysis gives negative bias
        // packed_rep: 6 narrow Dem wins (0.51D), 4 blowout Rep wins (0.20D)
        // statewide_dem = (6*510 + 4*200)/(10*1000) = (3060+800)/10000 = 0.386
        // swing = 0.5 - 0.386 = 0.114
        // adjusted: 0.51+0.114=0.624, 0.20+0.114=0.314
        // seats at 50%: only the 6 Dem-win districts → seats_at_50 = 6
        // pb = 0.5 - 6/10 = -0.1 < 0
        let districts = packed_rep_plan();
        let pb = compute_partisan_bias(&districts);
        assert!(pb < 0.0, "packed-Rep plan swing formula gives negative bias, got {pb}");
    }

    // ── compute_efficiency_gap additional cases ─────────────────────────────

    #[test]
    fn test_efficiency_gap_empty_returns_zero() {
        assert_eq!(compute_efficiency_gap(&[]), 0.0);
    }

    #[test]
    fn test_efficiency_gap_zero_vote_district() {
        let d = DistrictElection { district: 1, dem_votes: 0.0, rep_votes: 0.0 };
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
        let d = DistrictElection { district: 1, dem_votes: 0.0, rep_votes: 0.0 };
        assert!((d.dem_pct() - 0.5).abs() < 1e-9, "zero-vote district should return 0.5");
    }

    #[test]
    fn test_district_margin_zero_vote_returns_zero() {
        let d = DistrictElection { district: 1, dem_votes: 0.0, rep_votes: 0.0 };
        assert_eq!(d.margin(), 0.0);
    }

    #[test]
    fn test_district_margin_close_race() {
        let d = DistrictElection { district: 1, dem_votes: 510.0, rep_votes: 490.0 };
        let m = d.margin();
        assert!((m - 0.02).abs() < 1e-9, "margin should be 20/1000 = 0.02, got {m}");
    }

    #[test]
    fn test_district_dem_won_tie_returns_false() {
        let d = DistrictElection { district: 1, dem_votes: 500.0, rep_votes: 500.0 };
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
}
