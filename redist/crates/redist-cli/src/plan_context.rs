/// PlanContext — single source of truth for all metadata about an existing plan.
///
/// Architecture invariant (see docs/superpowers/specs/2026-04-28-redist-cli-architecture.md):
/// Any Class B command (analyze, report, compare, verify, map, export) MUST load its
/// plan metadata from this struct, never from LocationRegistry or load_all_states().
/// The manifest.json written by `redist state` is the authoritative record.
use std::path::{Path, PathBuf};
use redist_report::PlanManifest;

/// All metadata and derived paths for an existing redistricting plan.
#[derive(Debug)]
pub struct PlanContext {
    pub plan_dir: PathBuf,
    pub manifest: PlanManifest,
}

impl PlanContext {
    /// Load a plan by label from the standard output tree.
    ///
    /// Returns Err with an actionable message (listing available plans) if the
    /// plan directory or manifest is missing.
    pub fn from_label(
        output_base: &Path,
        version: &str,
        year: &str,
        label: &str,
    ) -> anyhow::Result<Self> {
        let plan_dir = output_base
            .join(version)
            .join(year)
            .join("plans")
            .join(label);

        if !plan_dir.exists() {
            let available = list_available_plans(output_base, version, year);
            let hint = if available.is_empty() {
                format!(
                    "No plans found in {}/{}/{}/plans/. \
                     Run 'redist state --label {}' to create one.",
                    output_base.display(), version, year, label
                )
            } else {
                format!("Available plans: {}", available.join(", "))
            };
            anyhow::bail!(
                "Plan '{}' not found at {}\n{}",
                label, plan_dir.display(), hint
            );
        }

        let manifest_path = plan_dir.join("manifest.json");
        if !manifest_path.exists() {
            anyhow::bail!(
                "Plan '{}' exists but has no manifest.json — the plan may be corrupt.\n\
                 Delete the directory and re-run: redist state --label {}",
                label, label
            );
        }

        let manifest: PlanManifest = serde_json::from_str(
            &std::fs::read_to_string(&manifest_path)
                .map_err(|e| anyhow::anyhow!("cannot read manifest.json: {e}"))?
        ).map_err(|e| anyhow::anyhow!("invalid manifest.json: {e}"))?;

        Ok(Self { plan_dir, manifest })
    }

    // ── Metadata accessors — ALL from manifest, never from global tables ──────

    /// Number of districts in this plan (from manifest — handles house/senate/international).
    pub fn num_districts(&self) -> usize {
        self.manifest.num_districts
    }

    /// Chamber type: "congressional", "house", "senate", "parliamentary", etc.
    pub fn chamber(&self) -> &str {
        &self.manifest.chamber
    }

    /// Two-letter state/location code: "WA", "IE", "MT-PARLIAMENT", etc.
    pub fn state_code(&self) -> &str {
        &self.manifest.state_code
    }

    /// Census or electoral year: "2020", "2022", etc.
    pub fn year(&self) -> &str {
        &self.manifest.year
    }

    /// Human label for this plan run.
    pub fn label(&self) -> &str {
        &self.manifest.label
    }

    /// Balance tolerance as a fraction (0.005 = 0.5%, 0.05 = 5%).
    pub fn balance_tolerance_frac(&self) -> f64 {
        self.manifest.balance_tolerance_pct / 100.0
    }

    /// Seats per constituency (1 for single-member systems).
    pub fn seats_per_district(&self) -> usize {
        self.manifest.seats_per_district.max(1)
    }

    /// Total seats across all constituencies.
    pub fn total_seats(&self) -> usize {
        self.manifest.total_seats.max(1)
    }

    // ── Derived paths — computed from plan_dir, never hardcoded ──────────────

    pub fn analysis_dir(&self) -> PathBuf {
        self.plan_dir.join("analysis")
    }

    pub fn data_dir(&self) -> PathBuf {
        self.plan_dir.join("data")
    }

    pub fn maps_dir(&self) -> PathBuf {
        self.plan_dir.join("maps")
    }

    pub fn assignments_path(&self) -> PathBuf {
        self.data_dir().join("final_assignments.json")
    }

    pub fn analysis_file(&self, name: &str) -> PathBuf {
        self.analysis_dir().join(name)
    }

    pub fn analysis_file_exists(&self, name: &str) -> bool {
        self.analysis_file(name).exists()
    }
}

/// List up to 10 plan labels available in the standard output tree.
fn list_available_plans(output_base: &Path, version: &str, year: &str) -> Vec<String> {
    let plans_dir = output_base.join(version).join(year).join("plans");
    std::fs::read_dir(&plans_dir)
        .ok()
        .map(|entries| {
            let mut labels: Vec<String> = entries
                .filter_map(|e| e.ok())
                .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
                .map(|e| e.file_name().to_string_lossy().into_owned())
                .collect();
            labels.sort();
            labels.truncate(10);
            labels
        })
        .unwrap_or_default()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn write_test_manifest(plan_dir: &Path, num_districts: usize, chamber: &str, state_code: &str, year: &str) {
        std::fs::create_dir_all(plan_dir).unwrap();
        let manifest = serde_json::json!({
            "label": plan_dir.file_name().unwrap().to_str().unwrap(),
            "state_code": state_code,
            "year": year,
            "chamber": chamber,
            "num_districts": num_districts,
            "population_source": "total",
            "partition_mode": "edge-weighted",
            "seed": null,
            "binary_version": "0.1.0",
            "binary_sha256": "",
            "binary_download_url": "",
            "adjacency_file": "",
            "adjacency_sha256": "",
            "adjacency_build_command": "",
            "adjacency_build_version": "0.1.0",
            "tiger_source_url": "",
            "tiger_sha256": null,
            "created_at": "2026-04-28T00:00:00Z",
            "balance_tolerance_pct": 5.0,
            "population_balance_valid": true,
            "seats_per_district": 1,
            "total_seats": num_districts,
            "electoral_system": "single_member",
            "gpmetis_version": "unknown"
        });
        std::fs::write(
            plan_dir.join("manifest.json"),
            serde_json::to_string_pretty(&manifest).unwrap(),
        ).unwrap();
    }

    fn make_plan_dir(tmp: &TempDir, label: &str) -> PathBuf {
        tmp.path().join("v1").join("2020").join("plans").join(label)
    }

    #[test]
    fn test_plan_context_loads_num_districts_from_manifest_not_registry() {
        // WA house = 98 districts, NOT 10 (which is WA congressional).
        // PlanContext must read from manifest, never from load_all_states().
        let tmp = TempDir::new().unwrap();
        let plan_dir = make_plan_dir(&tmp, "wa_house_test");
        write_test_manifest(&plan_dir, 98, "house", "WA", "2020");

        let ctx = PlanContext::from_label(tmp.path(), "v1", "2020", "wa_house_test").unwrap();
        assert_eq!(ctx.num_districts(), 98, "must read 98 from manifest, not 10 (congressional)");
        assert_eq!(ctx.chamber(), "house");
        assert_eq!(ctx.state_code(), "WA");
    }

    #[test]
    fn test_plan_context_international_reads_correctly() {
        // Ireland: 43 constituencies, STV
        let tmp = TempDir::new().unwrap();
        let plan_dir = make_plan_dir(&tmp, "ireland_dail_2022");
        write_test_manifest(&plan_dir, 43, "parliamentary", "IE", "2022");

        let ctx = PlanContext::from_label(tmp.path(), "v1", "2020", "ireland_dail_2022").unwrap();
        assert_eq!(ctx.num_districts(), 43);
        assert_eq!(ctx.chamber(), "parliamentary");
        assert_eq!(ctx.state_code(), "IE");
    }

    #[test]
    fn test_plan_context_missing_plan_gives_available_list() {
        let tmp = TempDir::new().unwrap();
        // Create two existing plans
        write_test_manifest(&make_plan_dir(&tmp, "wa_house_v1"), 98, "house", "WA", "2020");
        write_test_manifest(&make_plan_dir(&tmp, "wa_senate_v1"), 49, "senate", "WA", "2020");

        let result = PlanContext::from_label(tmp.path(), "v1", "2020", "nonexistent_plan");
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("nonexistent_plan"), "error must name the missing plan");
        assert!(
            msg.contains("wa_house_v1") || msg.contains("wa_senate_v1"),
            "error must list available plans: {msg}"
        );
    }

    #[test]
    fn test_plan_context_corrupt_plan_no_manifest() {
        let tmp = TempDir::new().unwrap();
        let plan_dir = make_plan_dir(&tmp, "corrupt_plan");
        std::fs::create_dir_all(&plan_dir).unwrap();
        // No manifest.json written

        let result = PlanContext::from_label(tmp.path(), "v1", "2020", "corrupt_plan");
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("manifest.json"), "error must mention manifest");
    }

    #[test]
    fn test_plan_context_accessors_match_manifest() {
        let tmp = TempDir::new().unwrap();
        let plan_dir = make_plan_dir(&tmp, "vt_congressional_2020");
        write_test_manifest(&plan_dir, 1, "congressional", "VT", "2020");

        let ctx = PlanContext::from_label(tmp.path(), "v1", "2020", "vt_congressional_2020").unwrap();
        assert_eq!(ctx.num_districts(), 1);
        assert_eq!(ctx.chamber(), "congressional");
        assert_eq!(ctx.state_code(), "VT");
        assert_eq!(ctx.year(), "2020");
        assert_eq!(ctx.label(), "vt_congressional_2020");
        assert!((ctx.balance_tolerance_frac() - 0.05).abs() < 1e-9);
        assert_eq!(ctx.seats_per_district(), 1);
    }

    #[test]
    fn test_plan_context_derived_paths() {
        let tmp = TempDir::new().unwrap();
        let plan_dir = make_plan_dir(&tmp, "vt_test");
        write_test_manifest(&plan_dir, 1, "congressional", "VT", "2020");

        let ctx = PlanContext::from_label(tmp.path(), "v1", "2020", "vt_test").unwrap();
        assert_eq!(ctx.analysis_dir(), plan_dir.join("analysis"));
        assert_eq!(ctx.data_dir(), plan_dir.join("data"));
        assert_eq!(ctx.assignments_path(), plan_dir.join("data").join("final_assignments.json"));
        assert_eq!(ctx.analysis_file("summary.json"), plan_dir.join("analysis").join("summary.json"));
    }

    #[test]
    fn test_list_available_plans_sorted() {
        let tmp = TempDir::new().unwrap();
        write_test_manifest(&make_plan_dir(&tmp, "z_plan"), 1, "congressional", "VT", "2020");
        write_test_manifest(&make_plan_dir(&tmp, "a_plan"), 1, "congressional", "VT", "2020");
        write_test_manifest(&make_plan_dir(&tmp, "m_plan"), 1, "congressional", "VT", "2020");

        let plans = list_available_plans(tmp.path(), "v1", "2020");
        assert_eq!(plans, vec!["a_plan", "m_plan", "z_plan"]);
    }
}
