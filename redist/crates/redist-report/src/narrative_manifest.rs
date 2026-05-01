//! Narrative manifest producer (Plan Comparison plan Task 9, COVENANT C-3).
//!
//! Writes `narrative_manifest.json` (schema `narrative-manifest v1`) sidecar
//! alongside every `redist compare --format narrative` output. The manifest
//! pins:
//! - The narrative template (path + sha256)
//! - The threshold values used (leaning_threshold, close_call_band, MoE inputs)
//! - The plan manifest SHAs (NOT just labels — labels can be re-bound to new
//!   content, COVENANT C-3 receipt)
//! - The analysis JSON SHAs for both plans
//! - approved_by + approved_at (audit trail; signed name committed)
//! - redist build commit + version + rustc
//! - civic-counter-proposal tag passthrough (when applicable)
//!
//! Reproducibility contract: re-running the same `compare` command on the
//! same inputs MUST produce a byte-identical manifest. To make this stable:
//! - All paths are RELATIVE to repo root (M-04, PP-31).
//! - approved_at is pinned via `SOURCE_DATE_EPOCH` env when set; otherwise
//!   `1970-01-01T00:00:00Z` is the [DRAFT] mode value (so unsigned narratives
//!   are also byte-stable).
//! - Field map is a `BTreeMap` so JSON key order is canonical (sorted).
//! - serde_json::to_string_pretty with default settings; LF line endings.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// Top-level narrative manifest. Schema version: `narrative-manifest v1`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NarrativeManifest {
    pub schema_version: String,
    /// Template path RELATIVE to repo root (e.g.,
    /// `redist/crates/redist-report/templates/comparison-narrative.j2.md`).
    pub template_path: String,
    pub template_sha256: String,
    pub leaning_threshold: f64,
    pub close_call_band: f64,
    /// Optional MoE inputs — present when the narrative consumed CIs for
    /// directional-claim suppression. Empty when MoE was not exercised.
    pub moe_inputs: BTreeMap<String, MoeInputRecord>,

    pub plan_a_label: String,
    pub plan_a_manifest_sha256: String,
    pub plan_b_label: String,
    pub plan_b_manifest_sha256: String,
    /// Set when `--baseline ENACTED` was used; null otherwise.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub baseline_label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub baseline_manifest_sha256: Option<String>,

    /// Source analysis JSON SHAs per plan, keyed by analysis filename.
    /// e.g., {"partisan.json": "abc...", "vra_analysis.json": "def..."}.
    /// BTreeMap nested for canonical ordering.
    pub analysis_sha256: BTreeMap<String, BTreeMap<String, String>>,

    /// Sign-off name; null when the narrative is `[DRAFT]`.
    pub approved_by: Option<String>,
    /// ISO-8601 UTC timestamp; pinned via SOURCE_DATE_EPOCH env for
    /// reproducibility. `1970-01-01T00:00:00Z` for unsigned [DRAFT] mode.
    pub approved_at: String,

    pub redist_version: String,
    pub redist_build_commit: String,
    pub redist_build_commit_short: String,
    pub rustc_version: String,

    /// Civic-counter-proposal tag passthrough (BD-N3 / SSI Task 7).
    /// Populated when either plan's manifest carries
    /// `submission_type=civic_counter_proposal`. Drives the framing label
    /// in the narrative + the watermark on the (deferred) summary card.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub submission_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub civic_counter_proposal_attribution: Option<CivicCounterProposalAttribution>,

    /// Optional comments-overlay reference (Civic Bidirectional consumer side).
    /// Present when --comments-label was supplied; empty otherwise.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comments_label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comments_overlay_sha256: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoeInputRecord {
    pub plan_a_low: f64,
    pub plan_a_point: f64,
    pub plan_a_high: f64,
    pub plan_b_low: f64,
    pub plan_b_point: f64,
    pub plan_b_high: f64,
    /// "monotone" or "non-monotone" — informs which suppression rule fires.
    pub monotonicity: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CivicCounterProposalAttribution {
    pub plan_label: String,
    pub submitted_by: String,
    pub submitted_at: String,
}

/// Inputs needed to build a narrative manifest. The caller (the comparison
/// assembler in Task 2) is responsible for computing every SHA — this module
/// is a pure record builder.
#[derive(Debug, Clone)]
#[allow(clippy::struct_field_names)]
pub struct NarrativeManifestInputs {
    pub template_path_rel: String,
    pub template_sha256: String,
    pub leaning_threshold: f64,
    pub close_call_band: f64,
    pub moe_inputs: BTreeMap<String, MoeInputRecord>,

    pub plan_a_label: String,
    pub plan_a_manifest_sha256: String,
    pub plan_b_label: String,
    pub plan_b_manifest_sha256: String,
    pub baseline_label: Option<String>,
    pub baseline_manifest_sha256: Option<String>,

    pub analysis_sha256: BTreeMap<String, BTreeMap<String, String>>,

    pub approved_by: Option<String>,

    pub redist_version: String,
    pub redist_build_commit: String,
    pub rustc_version: String,

    pub submission_type: Option<String>,
    pub civic_counter_proposal_attribution: Option<CivicCounterProposalAttribution>,

    pub comments_label: Option<String>,
    pub comments_overlay_sha256: Option<String>,
}

/// Build a `NarrativeManifest` from inputs. Pure function — reads
/// `SOURCE_DATE_EPOCH` from the env. For tests that need to control the env
/// without races, use `build_narrative_manifest_with_clock`.
pub fn build_narrative_manifest(inputs: NarrativeManifestInputs) -> NarrativeManifest {
    let secs = std::env::var("SOURCE_DATE_EPOCH")
        .ok()
        .and_then(|s| s.parse::<i64>().ok());
    build_narrative_manifest_with_clock(inputs, secs)
}

/// Pure-function variant: caller supplies the SOURCE_DATE_EPOCH override
/// directly (`Some(secs)` for a pinned timestamp; `None` for the
/// `1970-01-01T00:00:00Z` sentinel). No env reads — tests can run in
/// parallel safely.
///
/// Reproducibility:
/// - `approved_at` is `Some(secs)` formatted as ISO-8601 UTC, or the sentinel.
/// - All map fields use `BTreeMap` (canonical key order).
/// - `redist_build_commit_short` is the first 8 chars of `redist_build_commit`
///   (or empty when the commit is empty).
pub fn build_narrative_manifest_with_clock(
    inputs: NarrativeManifestInputs,
    source_date_epoch: Option<i64>,
) -> NarrativeManifest {
    let approved_at = source_date_epoch
        .map(iso8601_from_unix_secs)
        .unwrap_or_else(|| "1970-01-01T00:00:00Z".to_string());
    let short = inputs
        .redist_build_commit
        .chars()
        .take(8)
        .collect::<String>();
    NarrativeManifest {
        schema_version: "narrative-manifest v1".to_string(),
        template_path: inputs.template_path_rel,
        template_sha256: inputs.template_sha256,
        leaning_threshold: inputs.leaning_threshold,
        close_call_band: inputs.close_call_band,
        moe_inputs: inputs.moe_inputs,
        plan_a_label: inputs.plan_a_label,
        plan_a_manifest_sha256: inputs.plan_a_manifest_sha256,
        plan_b_label: inputs.plan_b_label,
        plan_b_manifest_sha256: inputs.plan_b_manifest_sha256,
        baseline_label: inputs.baseline_label,
        baseline_manifest_sha256: inputs.baseline_manifest_sha256,
        analysis_sha256: inputs.analysis_sha256,
        approved_by: inputs.approved_by,
        approved_at,
        redist_version: inputs.redist_version,
        redist_build_commit: inputs.redist_build_commit,
        redist_build_commit_short: short,
        rustc_version: inputs.rustc_version,
        submission_type: inputs.submission_type,
        civic_counter_proposal_attribution: inputs.civic_counter_proposal_attribution,
        comments_label: inputs.comments_label,
        comments_overlay_sha256: inputs.comments_overlay_sha256,
    }
}

/// Format a Unix seconds-since-epoch as ISO-8601 UTC without an external
/// chrono dependency. Mirrors `redist-cli::provenance::format_unix_iso8601`
/// (kept duplicated to avoid a redist-cli ↔ redist-report cycle).
fn iso8601_from_unix_secs(secs: i64) -> String {
    if secs < 0 {
        return "1970-01-01T00:00:00Z".to_string();
    }
    let total_secs = secs as u64;
    let days = total_secs / 86_400;
    let secs_today = total_secs % 86_400;
    let hour = secs_today / 3600;
    let min = (secs_today % 3600) / 60;
    let sec = secs_today % 60;

    // Days-from-epoch -> Y/M/D via cumulative-month-length walk.
    let (year, month, day) = days_to_ymd(days);
    format!(
        "{year:04}-{month:02}-{day:02}T{hour:02}:{min:02}:{sec:02}Z",
        year = year, month = month, day = day, hour = hour, min = min, sec = sec
    )
}

fn days_to_ymd(days_since_epoch: u64) -> (u32, u32, u32) {
    let mut year: u32 = 1970;
    let mut days = days_since_epoch;
    loop {
        let yr_days = if is_leap(year) { 366 } else { 365 };
        if days < yr_days {
            break;
        }
        days -= yr_days;
        year += 1;
    }
    let month_lens: [u64; 12] = if is_leap(year) {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };
    let mut month = 1u32;
    for &m_len in &month_lens {
        if days < m_len {
            break;
        }
        days -= m_len;
        month += 1;
    }
    let day = (days + 1) as u32;
    (year, month, day)
}

fn is_leap(year: u32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

/// Serialize the manifest to a byte-stable JSON string. Uses pretty-print
/// with 2-space indent (matches serde_json::to_string_pretty default).
pub fn serialize_manifest(m: &NarrativeManifest) -> serde_json::Result<String> {
    serde_json::to_string_pretty(m)
}

/// Compute the relative path from `repo_root` to `target` as a string with
/// forward slashes (per M-04 / PP-31). Returns `target.display().to_string()`
/// if the relative computation fails (caller should treat that as a [INTERNAL]
/// error in production).
pub fn path_relative_to_repo_root(repo_root: &Path, target: &Path) -> String {
    target
        .strip_prefix(repo_root)
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .unwrap_or_else(|_| target.display().to_string().replace('\\', "/"))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture_inputs() -> NarrativeManifestInputs {
        let mut analysis_a: BTreeMap<String, String> = BTreeMap::new();
        analysis_a.insert("partisan.json".into(), "a".repeat(64));
        analysis_a.insert("vra_analysis.json".into(), "b".repeat(64));
        analysis_a.insert("compactness.json".into(), "c".repeat(64));
        let mut analysis_b: BTreeMap<String, String> = BTreeMap::new();
        analysis_b.insert("partisan.json".into(), "d".repeat(64));
        analysis_b.insert("vra_analysis.json".into(), "e".repeat(64));
        analysis_b.insert("compactness.json".into(), "f".repeat(64));
        let mut analysis_sha256: BTreeMap<String, BTreeMap<String, String>> = BTreeMap::new();
        analysis_sha256.insert("plan_a".into(), analysis_a);
        analysis_sha256.insert("plan_b".into(), analysis_b);
        NarrativeManifestInputs {
            template_path_rel: "redist/crates/redist-report/templates/comparison-narrative.j2.md".into(),
            template_sha256: "0".repeat(64),
            leaning_threshold: 0.55,
            close_call_band: 0.02,
            moe_inputs: BTreeMap::new(),
            plan_a_label: "vt_2020_a".into(),
            plan_a_manifest_sha256: "1".repeat(64),
            plan_b_label: "vt_2020_b".into(),
            plan_b_manifest_sha256: "2".repeat(64),
            baseline_label: None,
            baseline_manifest_sha256: None,
            analysis_sha256,
            approved_by: None,
            redist_version: "0.1.0-test".into(),
            redist_build_commit: "deadbeef0123456789abcdef".into(),
            rustc_version: "rustc 1.80.0".into(),
            submission_type: None,
            civic_counter_proposal_attribution: None,
            comments_label: None,
            comments_overlay_sha256: None,
        }
    }

    // No env-touching helper needed — tests use build_narrative_manifest_with_clock
    // to avoid the parallel-test env race.

    #[test]
    fn test_build_narrative_manifest_basic_shape() {
        let inputs = fixture_inputs();
        let m = build_narrative_manifest(inputs);
        assert_eq!(m.schema_version, "narrative-manifest v1");
        assert_eq!(m.leaning_threshold, 0.55);
        assert_eq!(m.close_call_band, 0.02);
        assert_eq!(m.plan_a_label, "vt_2020_a");
        assert_eq!(m.plan_b_label, "vt_2020_b");
        // build_commit_short is first 8 chars.
        assert_eq!(m.redist_build_commit_short, "deadbeef");
    }

    #[test]
    fn test_draft_mode_approved_at_is_epoch_sentinel() {
        // None override -> 1970-01-01 sentinel so [DRAFT] manifests are byte-stable.
        let m = build_narrative_manifest_with_clock(fixture_inputs(), None);
        assert_eq!(m.approved_at, "1970-01-01T00:00:00Z");
    }

    #[test]
    fn test_source_date_epoch_pins_approved_at() {
        // 1700000000 = 2023-11-14T22:13:20Z
        let m = build_narrative_manifest_with_clock(fixture_inputs(), Some(1_700_000_000));
        assert_eq!(m.approved_at, "2023-11-14T22:13:20Z");
    }

    #[test]
    fn test_byte_identical_re_render_with_pinned_clock() {
        let m1 = build_narrative_manifest_with_clock(fixture_inputs(), Some(1_700_000_000));
        let m2 = build_narrative_manifest_with_clock(fixture_inputs(), Some(1_700_000_000));
        let s1 = serialize_manifest(&m1).unwrap();
        let s2 = serialize_manifest(&m2).unwrap();
        assert_eq!(s1, s2, "byte-identical re-render with pinned clock");
    }

    #[test]
    fn test_analysis_sha256_keys_are_canonically_ordered() {
        let inputs = fixture_inputs();
        let m = build_narrative_manifest(inputs);
        let s = serialize_manifest(&m).unwrap();
        // BTreeMap iteration is sorted; in JSON output the inner map should
        // have keys in alphabetical order.
        let pos_compactness = s.find("\"compactness.json\"").unwrap();
        let pos_partisan = s.find("\"partisan.json\"").unwrap();
        let pos_vra = s.find("\"vra_analysis.json\"").unwrap();
        assert!(pos_compactness < pos_partisan, "compactness must precede partisan");
        assert!(pos_partisan < pos_vra, "partisan must precede vra_analysis");
    }

    #[test]
    fn test_approved_by_set_does_not_affect_approved_at_pinning() {
        // Whether the narrative is signed or not, approved_at is pinned via
        // the supplied clock (NOT current time). This is the byte-identical
        // re-render contract.
        let mut inputs = fixture_inputs();
        inputs.approved_by = Some("Jane Q. Citizen".into());
        let m = build_narrative_manifest_with_clock(inputs, Some(1_700_000_000));
        assert_eq!(m.approved_at, "2023-11-14T22:13:20Z");
        assert_eq!(m.approved_by.as_deref(), Some("Jane Q. Citizen"));
    }

    #[test]
    fn test_baseline_fields_serialized_only_when_set() {
        let inputs = fixture_inputs();
        let m_no_baseline = build_narrative_manifest(inputs.clone());
        let s_no = serialize_manifest(&m_no_baseline).unwrap();
        assert!(!s_no.contains("baseline_label"), "absent baseline must be omitted");

        let mut inputs_with = inputs;
        inputs_with.baseline_label = Some("vt_enacted".into());
        inputs_with.baseline_manifest_sha256 = Some("9".repeat(64));
        let m_with = build_narrative_manifest(inputs_with);
        let s_with = serialize_manifest(&m_with).unwrap();
        assert!(s_with.contains("baseline_label"));
        assert!(s_with.contains("vt_enacted"));
    }

    #[test]
    fn test_civic_counter_proposal_attribution_round_trip() {
        let mut inputs = fixture_inputs();
        inputs.submission_type = Some("civic_counter_proposal".into());
        inputs.civic_counter_proposal_attribution = Some(CivicCounterProposalAttribution {
            plan_label: "lwvla_alternative_2026".into(),
            submitted_by: "League of Women Voters Louisiana".into(),
            submitted_at: "2026-04-15T12:00:00Z".into(),
        });
        let m = build_narrative_manifest(inputs);
        let s = serialize_manifest(&m).unwrap();
        let parsed: NarrativeManifest = serde_json::from_str(&s).unwrap();
        assert_eq!(parsed.submission_type.as_deref(), Some("civic_counter_proposal"));
        let attr = parsed.civic_counter_proposal_attribution.unwrap();
        assert_eq!(attr.submitted_by, "League of Women Voters Louisiana");
    }

    #[test]
    fn test_path_relative_to_repo_root_uses_forward_slashes() {
        let root = Path::new("/repo");
        let target = Path::new("/repo/templates/comparison-narrative.j2.md");
        let rel = path_relative_to_repo_root(root, target);
        assert!(!rel.contains('\\'), "must use forward slashes: {rel}");
        assert_eq!(rel, "templates/comparison-narrative.j2.md");
    }

    #[test]
    fn test_path_relative_to_repo_root_handles_outside_root() {
        let root = Path::new("/repo");
        let target = Path::new("/elsewhere/foo.md");
        // Falls back to the absolute display string with forward slashes.
        let rel = path_relative_to_repo_root(root, target);
        assert!(!rel.contains('\\'));
    }

    #[test]
    fn test_iso8601_from_unix_secs_known_values() {
        // Sanity-check the chrono-free formatter against a few well-known dates.
        assert_eq!(iso8601_from_unix_secs(0), "1970-01-01T00:00:00Z");
        assert_eq!(iso8601_from_unix_secs(86_400), "1970-01-02T00:00:00Z");
        // 2000-01-01T00:00:00Z = 946,684,800
        assert_eq!(iso8601_from_unix_secs(946_684_800), "2000-01-01T00:00:00Z");
        // 1700000000 = 2023-11-14T22:13:20Z (the SOURCE_DATE_EPOCH fixture).
        assert_eq!(iso8601_from_unix_secs(1_700_000_000), "2023-11-14T22:13:20Z");
        // 2024-02-29T12:34:56Z (leap day) = 1709210096
        assert_eq!(iso8601_from_unix_secs(1_709_210_096), "2024-02-29T12:34:56Z");
    }

    #[test]
    fn test_negative_epoch_clamps_to_sentinel() {
        assert_eq!(iso8601_from_unix_secs(-1), "1970-01-01T00:00:00Z");
    }

    #[test]
    fn test_build_narrative_manifest_basic_shape_with_explicit_clock() {
        // Explicit-clock variant: independent of env state, parallel-safe.
        let m = build_narrative_manifest_with_clock(fixture_inputs(), Some(1_700_000_000));
        assert_eq!(m.schema_version, "narrative-manifest v1");
        assert_eq!(m.approved_at, "2023-11-14T22:13:20Z");
    }
}
