//! Comparison report data structures (Plan Comparison plan Task 2).
//!
//! Defines the `ComparisonReport` struct that the narrative renderer consumes.
//! This module is the data layer; rendering lives in `narrative.rs`.
//!
//! Minimal scope for this commit: structures + a `from_pre_loaded` constructor.
//! The full assembler that reads analysis JSONs from disk + computes manifest
//! SHAs is the next session's pickup (it requires reaching into the
//! existing `redist-cli` plan-context module which is a cross-crate dance).

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// One side of the comparison: the data we need from a single plan.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanSide {
    pub label: String,
    pub manifest_sha256: String,
    /// Number of seats won by the named threshold-defining party (typically Dem).
    pub leaning_seats: usize,
    /// Total district count.
    pub n_districts: usize,
    /// Per-district Dem-share point estimates (length n_districts). Used by
    /// the close-call detector + the threshold-classification narrative.
    pub per_district_dem_share: Vec<f64>,
    /// Number of majority-minority districts (BVAP > 50%).
    pub mm_count: usize,
    /// Mean Polsby-Popper compactness across districts.
    pub mean_pp: f64,
    /// Total population.
    pub total_population: u64,
    /// Optional civic-counter-proposal tag passthrough from the plan's
    /// PlanManifest.submission_type. Drives the framing label in the narrative.
    #[serde(default)]
    pub submission_type: Option<String>,
    /// Submitter (when civic counter-proposal).
    #[serde(default)]
    pub submitted_by: Option<String>,
    /// Submission timestamp (when civic counter-proposal).
    #[serde(default)]
    pub submitted_at: Option<String>,
    /// Per-analysis-file SHA-256s (e.g., {"partisan.json": "..."}).
    /// Used by the narrative_manifest writer.
    pub analysis_sha256: BTreeMap<String, String>,
}

/// Diff between two plans: which tracts moved, how many people, which districts changed.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DiffSummary {
    pub tracts_changed: usize,
    pub population_changed: u64,
    /// Sorted list of district numbers (in plan A's labeling) where any tract
    /// was reassigned.
    pub districts_with_changes: Vec<usize>,
}

/// Top-level comparison report consumed by the narrative renderer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonReport {
    pub plan_a: PlanSide,
    pub plan_b: PlanSide,
    pub baseline: Option<PlanSide>,
    pub diff: DiffSummary,
}

impl ComparisonReport {
    /// Construct from already-loaded plan sides. Used by tests and by the
    /// CLI dispatcher once it's wired (Task 11).
    pub fn from_loaded(
        plan_a: PlanSide,
        plan_b: PlanSide,
        baseline: Option<PlanSide>,
        diff: DiffSummary,
    ) -> Self {
        ComparisonReport {
            plan_a,
            plan_b,
            baseline,
            diff,
        }
    }

    /// True iff either plan carries the civic-counter-proposal tag.
    pub fn has_civic_counter_proposal(&self) -> bool {
        is_civic(&self.plan_a) || is_civic(&self.plan_b)
    }
}

fn is_civic(p: &PlanSide) -> bool {
    p.submission_type.as_deref() == Some("civic_counter_proposal")
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture_plan_a() -> PlanSide {
        PlanSide {
            label: "vt_state_proposal".into(),
            manifest_sha256: "a".repeat(64),
            leaning_seats: 5,
            n_districts: 6,
            per_district_dem_share: vec![0.30, 0.45, 0.55, 0.60, 0.65, 0.70],
            mm_count: 1,
            mean_pp: 0.42,
            total_population: 600_000,
            submission_type: None,
            submitted_by: None,
            submitted_at: None,
            analysis_sha256: BTreeMap::new(),
        }
    }
    fn fixture_plan_b() -> PlanSide {
        PlanSide {
            label: "vt_civic_alt".into(),
            manifest_sha256: "b".repeat(64),
            leaning_seats: 4,
            n_districts: 6,
            per_district_dem_share: vec![0.25, 0.40, 0.547, 0.553, 0.62, 0.68],
            mm_count: 1,
            mean_pp: 0.40,
            total_population: 600_000,
            submission_type: Some("civic_counter_proposal".into()),
            submitted_by: Some("League of Women Voters Vermont".into()),
            submitted_at: Some("2026-04-15T12:00:00Z".into()),
            analysis_sha256: BTreeMap::new(),
        }
    }

    #[test]
    fn test_has_civic_counter_proposal_detects_b() {
        let report = ComparisonReport::from_loaded(
            fixture_plan_a(),
            fixture_plan_b(),
            None,
            DiffSummary::default(),
        );
        assert!(report.has_civic_counter_proposal());
    }

    #[test]
    fn test_has_civic_counter_proposal_false_when_neither() {
        let report = ComparisonReport::from_loaded(
            fixture_plan_a(),
            fixture_plan_a(),
            None,
            DiffSummary::default(),
        );
        assert!(!report.has_civic_counter_proposal());
    }

    #[test]
    fn test_diff_summary_default_is_empty() {
        let d = DiffSummary::default();
        assert_eq!(d.tracts_changed, 0);
        assert_eq!(d.population_changed, 0);
        assert!(d.districts_with_changes.is_empty());
    }
}
