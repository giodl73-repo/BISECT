//! Margin-of-error suppression for narrative claims (Plan Comparison plan Task 4, S-04).
//!
//! When a narrative would assert a directional claim (e.g., "Plan B has more
//! Democratic seats than Plan A") but the claim could flip within the metric's
//! confidence interval, the auto-text MUST be replaced with
//! "within margin of error; see numerical table." The narrative writer cannot
//! distinguish the two cases by itself; this module supplies the rule.
//!
//! Two metric monotonicities are handled:
//!
//! - **Monotone metrics** (Dem seats, mean PP, population deviation):
//!   suppression fires when the sign of `(plan_a.point - plan_b.point)` flips
//!   inside the CI overlap. Concretely: if either `(a.low - b.high)` or
//!   `(a.high - b.low)` has the opposite sign of `(a.point - b.point)`, the
//!   directional claim is unsafe.
//!
//! - **Non-monotone metrics** (MM count is a step function of an underlying
//!   continuous variable like BVAP share):  suppression fires when the input
//!   CI for ANY district crosses the 50% threshold (the BVAP cutoff that
//!   defines majority-minority status). Plan-level: if the MM-count CIs of
//!   the two plans overlap, the directional claim is unsafe.
//!
//! Spec: `docs/superpowers/specs/2026-04-30-plan-comparison-and-narrative.md`
//! Tracking: S-04 (formal "direction change" definition for non-monotone
//! metrics). v2.1.1 plan patch made this load-bearing.

/// One metric's confidence band: a point estimate plus low/high CI bounds.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CiBand {
    pub point: f64,
    pub low: f64,
    pub high: f64,
}

impl CiBand {
    /// Construct a CiBand. Asserts low <= point <= high in debug builds.
    pub fn new(point: f64, low: f64, high: f64) -> Self {
        debug_assert!(low <= point + 1e-12, "low {low} > point {point}");
        debug_assert!(point <= high + 1e-12, "point {point} > high {high}");
        CiBand { point, low, high }
    }
}

/// Whether a metric responds monotonically to its underlying inputs. MM count
/// is the load-bearing non-monotone case; everything else in our analyzer set
/// is monotone.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetricMonotonicity {
    Monotone,
    /// Step-function over a continuous BVAP/HVAP variable. Suppression fires
    /// when the input CI crosses any 50% threshold.
    NonMonotone,
}

/// Decide whether a narrative directional claim should be suppressed in favor
/// of the canonical "within margin of error" text.
///
/// Returns `Some("within margin of error; see numerical table.")` when the
/// directional claim is unsafe, or `None` when the auto-text may proceed.
///
/// The caller is responsible for substituting the returned string into the
/// narrative paragraph at the appropriate point.
pub fn suppress_or_emit(
    metric: &str,
    monotonicity: MetricMonotonicity,
    plan_a: CiBand,
    plan_b: CiBand,
) -> Option<String> {
    let _ = metric; // reserved for future per-metric logging
    match monotonicity {
        MetricMonotonicity::Monotone => suppress_monotone(plan_a, plan_b),
        MetricMonotonicity::NonMonotone => suppress_non_monotone(plan_a, plan_b),
    }
}

/// Canonical suppression text. Matches the spec's required wording exactly so
/// downstream consumers (Plan Comparison narrative, Court Submission Reports
/// Typst template) can grep for it.
pub const MOE_SUPPRESSED_TEXT: &str = "within margin of error; see numerical table.";

fn suppress_monotone(a: CiBand, b: CiBand) -> Option<String> {
    let point_diff = a.point - b.point;
    if point_diff == 0.0 {
        // Exactly tied; no directional claim was going to be made.
        return Some(MOE_SUPPRESSED_TEXT.to_string());
    }
    // The CI bracketing for "could the comparison flip?":
    // - If point_diff > 0 (A > B), it's safe iff a.low > b.high (no overlap).
    // - If point_diff < 0 (A < B), it's safe iff a.high < b.low (no overlap).
    let safe = if point_diff > 0.0 {
        a.low > b.high
    } else {
        a.high < b.low
    };
    if safe {
        None
    } else {
        Some(MOE_SUPPRESSED_TEXT.to_string())
    }
}

fn suppress_non_monotone(a: CiBand, b: CiBand) -> Option<String> {
    // MM-count CIs are integer-valued; if the two plans' CIs overlap, the
    // directional claim is unsafe. Plan-level definition per spec Task 4.3.
    // We treat the CI as [low, high] inclusive. Overlap iff a.low <= b.high
    // AND b.low <= a.high.
    let overlap = a.low <= b.high && b.low <= a.high;
    if overlap {
        Some(MOE_SUPPRESSED_TEXT.to_string())
    } else {
        None
    }
}

/// Per-district MM-indeterminacy check (spec Task 4.3 formal non-monotone
/// definition): if the BVAP CI of any district straddles the 0.50 majority
/// threshold, that district's MM status is "indeterminate" — the analyst
/// SHOULD widen the plan-level MM-count CI before invoking `suppress_or_emit`.
///
/// `bvap_ci_per_district` is a slice of (low, high) CI bounds for the
/// district's BVAP share (or HVAP, etc.).
/// `threshold` is the majority cutoff (default 0.50; configurable for state-
/// specific rules).
///
/// Returns the count of districts with indeterminate MM status. The narrative
/// assembler should fold this into the plan-level CI bounds.
pub fn count_indeterminate_districts(bvap_ci_per_district: &[(f64, f64)], threshold: f64) -> usize {
    bvap_ci_per_district
        .iter()
        .filter(|(low, high)| *low < threshold && threshold < *high)
        .count()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // ── Monotone metric (Dem seats) ─────────────────────────────────────────

    #[test]
    fn test_monotone_no_overlap_emits_no_suppression() {
        // Plan A: 7 ± 0.3 seats. Plan B: 5 ± 0.3 seats. Clear gap; no overlap.
        let a = CiBand::new(7.0, 6.7, 7.3);
        let b = CiBand::new(5.0, 4.7, 5.3);
        assert_eq!(suppress_or_emit("dem_seats", MetricMonotonicity::Monotone, a, b), None);
    }

    #[test]
    fn test_monotone_ci_overlap_emits_suppression() {
        // Spec's worked example (Task 4.6): Plan A 5.0 ± 0.6 vs Plan B 5.4 ± 0.5.
        // Point diff is -0.4 (A < B). Safety requires a.high < b.low: 5.6 < 4.9 — false.
        let a = CiBand::new(5.0, 4.4, 5.6);
        let b = CiBand::new(5.4, 4.9, 5.9);
        let result = suppress_or_emit("dem_seats", MetricMonotonicity::Monotone, a, b);
        assert_eq!(result, Some(MOE_SUPPRESSED_TEXT.to_string()));
    }

    #[test]
    fn test_monotone_exact_tie_emits_suppression() {
        let a = CiBand::new(5.0, 4.8, 5.2);
        let b = CiBand::new(5.0, 4.7, 5.3);
        let result = suppress_or_emit("dem_seats", MetricMonotonicity::Monotone, a, b);
        assert_eq!(result, Some(MOE_SUPPRESSED_TEXT.to_string()));
    }

    #[test]
    fn test_monotone_a_less_than_b_no_overlap_emits_no_suppression() {
        let a = CiBand::new(3.0, 2.9, 3.1);
        let b = CiBand::new(5.0, 4.8, 5.2);
        assert_eq!(suppress_or_emit("dem_seats", MetricMonotonicity::Monotone, a, b), None);
    }

    #[test]
    fn test_monotone_a_greater_with_touching_intervals_emits_suppression() {
        // Edge case: a.low == b.high. Strict inequality required for safety;
        // touching is not safe.
        let a = CiBand::new(5.0, 4.5, 5.5);
        let b = CiBand::new(4.0, 3.5, 4.5);
        // a.low (4.5) > b.high (4.5) is FALSE (touching is not strict), so suppress.
        let result = suppress_or_emit("dem_seats", MetricMonotonicity::Monotone, a, b);
        assert_eq!(result, Some(MOE_SUPPRESSED_TEXT.to_string()));
    }

    // ── Non-monotone metric (MM count) ──────────────────────────────────────

    #[test]
    fn test_non_monotone_distinct_mm_counts_no_suppression() {
        // Plan A MM=2 [1.5, 2.5]; Plan B MM=0 [0.0, 0.5]. Gap; safe to claim
        // "Plan A has more MM districts."
        let a = CiBand::new(2.0, 1.5, 2.5);
        let b = CiBand::new(0.0, 0.0, 0.5);
        assert_eq!(
            suppress_or_emit("mm_count", MetricMonotonicity::NonMonotone, a, b),
            None
        );
    }

    #[test]
    fn test_non_monotone_overlapping_mm_counts_emits_suppression() {
        // Spec example (Task 4.6): Plan A MM=1 [BVAP CI 0.48-0.53] -> MM CI [0,1];
        // Plan B MM=0 [0.46-0.49] -> MM CI [0,0]. Overlap at 0 -> suppress.
        let a = CiBand::new(1.0, 0.0, 1.0);
        let b = CiBand::new(0.0, 0.0, 0.0);
        let result = suppress_or_emit("mm_count", MetricMonotonicity::NonMonotone, a, b);
        assert_eq!(result, Some(MOE_SUPPRESSED_TEXT.to_string()));
    }

    #[test]
    fn test_non_monotone_touching_intervals_overlap_suppression() {
        // a.low == b.high (touch at one point). Spec definition: overlap iff
        // a.low <= b.high; touching counts. Suppress.
        let a = CiBand::new(2.0, 1.0, 2.0);
        let b = CiBand::new(3.0, 2.0, 4.0);
        let result = suppress_or_emit("mm_count", MetricMonotonicity::NonMonotone, a, b);
        assert_eq!(result, Some(MOE_SUPPRESSED_TEXT.to_string()));
    }

    // ── Indeterminate-district counter ──────────────────────────────────────

    #[test]
    fn test_count_indeterminate_districts_all_clear() {
        // Every district's BVAP CI is well above or well below 0.50.
        let cis = vec![(0.30, 0.35), (0.40, 0.45), (0.55, 0.60)];
        assert_eq!(count_indeterminate_districts(&cis, 0.50), 0);
    }

    #[test]
    fn test_count_indeterminate_districts_one_straddles() {
        let cis = vec![(0.30, 0.35), (0.48, 0.53), (0.60, 0.65)];
        assert_eq!(count_indeterminate_districts(&cis, 0.50), 1);
    }

    #[test]
    fn test_count_indeterminate_districts_threshold_at_boundary_exclusive() {
        // CI [0.48, 0.50] does NOT straddle: high equals threshold (not strictly above).
        let cis = vec![(0.48, 0.50)];
        assert_eq!(count_indeterminate_districts(&cis, 0.50), 0);
    }

    #[test]
    fn test_count_indeterminate_districts_configurable_threshold() {
        // Hispanic-majority threshold might be different; allow caller to set.
        let cis = vec![(0.62, 0.68)];  // straddles 0.65 but not 0.50
        assert_eq!(count_indeterminate_districts(&cis, 0.50), 0);
        assert_eq!(count_indeterminate_districts(&cis, 0.65), 1);
    }

    // ── Constant text ───────────────────────────────────────────────────────

    #[test]
    fn test_moe_suppressed_text_is_canonical() {
        // Downstream consumers grep for this string verbatim; never paraphrase.
        assert_eq!(
            MOE_SUPPRESSED_TEXT,
            "within margin of error; see numerical table."
        );
    }
}
