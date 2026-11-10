/// Plan discovery — scans output directories for manifests and analysis JSONs.
use crate::app::PlanSummary;
use std::path::{Path, PathBuf};

/// Scan `{output_base}/{version}/{year}/plans/` for plan subdirectories.
/// Reads manifest.json and analysis/*.json for each plan.
/// Returns plans sorted by label.
pub fn discover_plans(output_base: &str, version: &str, year: &str) -> Vec<PlanSummary> {
    let plans_dir = PathBuf::from(output_base)
        .join(version)
        .join(year)
        .join("plans");

    let Ok(entries) = std::fs::read_dir(&plans_dir) else {
        return Vec::new();
    };

    let registry = bisect_cli::policy::LocationRegistry::load();

    let mut plans: Vec<PlanSummary> = entries
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
        .filter_map(|entry| {
            let plan_dir = entry.path();
            read_plan(&plan_dir, &registry)
        })
        .collect();

    plans.sort_by(|a, b| a.label.cmp(&b.label));
    plans
}

/// Read a single plan directory.
fn read_plan(
    plan_dir: &Path,
    registry: &bisect_cli::policy::LocationRegistry,
) -> Option<PlanSummary> {
    let manifest_content=std::fs::read_to_string(plan_dir.join("manifest.json")).ok()?;
    let analysis=["compactness.json","summary.json","splits.json","contiguity.json"].into_iter().filter_map(|file|std::fs::read_to_string(plan_dir.join("analysis").join(file)).ok().map(|text|(file.to_string(),text))).collect();
    let row=bisect_report::tui_library::read_plan(&plan_dir.to_string_lossy(),plan_dir.file_name().and_then(|s|s.to_str()).unwrap_or(""),&manifest_content,&analysis,|state|registry.state_name(state).unwrap_or_else(||state.to_string()))?;
    Some(PlanSummary{label:row.label,state_code:row.state_code,state_name:row.state_name,chamber:row.chamber,year:row.year,num_districts:row.num_districts as usize,mean_pp:row.mean_pp,max_deviation_pct:row.max_deviation_pct,county_splits:row.county_splits.map(|n|n as usize),all_contiguous:row.all_contiguous,plan_dir:plan_dir.to_path_buf()})
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn write_json(path: &Path, content: &str) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, content).unwrap();
    }

    fn make_plan(
        plans_dir: &Path,
        label: &str,
        state_code: &str,
        chamber: &str,
        year: &str,
        districts: u64,
    ) -> PathBuf {
        let plan_dir = plans_dir.join(label);
        fs::create_dir_all(&plan_dir).unwrap();

        let manifest = serde_json::json!({
            "label": label,
            "state_code": state_code,
            "chamber": chamber,
            "year": year,
            "num_districts": districts
        });
        write_json(&plan_dir.join("manifest.json"), &manifest.to_string());
        plan_dir
    }

    #[test]
    fn test_discover_plans_empty_dir_returns_empty() {
        let tmp = TempDir::new().unwrap();
        // Create the plans directory but leave it empty
        let plans_dir = tmp.path().join("v1").join("2020").join("plans");
        fs::create_dir_all(&plans_dir).unwrap();
        let result = discover_plans(tmp.path().to_str().unwrap(), "v1", "2020");
        assert!(result.is_empty(), "empty plans dir should return empty vec");
    }

    #[test]
    fn test_discover_plans_missing_dir_returns_empty() {
        let tmp = TempDir::new().unwrap();
        // Don't create the plans directory at all
        let result = discover_plans(tmp.path().to_str().unwrap(), "v1", "2020");
        assert!(
            result.is_empty(),
            "missing plans dir should return empty vec"
        );
    }

    #[test]
    fn test_discover_plans_reads_manifest_fields() {
        let tmp = TempDir::new().unwrap();
        let plans_dir = tmp.path().join("v1").join("2020").join("plans");
        make_plan(&plans_dir, "wa_house_2020", "WA", "house", "2020", 98);

        let result = discover_plans(tmp.path().to_str().unwrap(), "v1", "2020");
        assert_eq!(result.len(), 1);
        let plan = &result[0];
        assert_eq!(plan.label, "wa_house_2020");
        assert_eq!(plan.state_code, "WA");
        assert_eq!(plan.chamber, "house");
        assert_eq!(plan.year, "2020");
        assert_eq!(plan.num_districts, 98);
    }

    #[test]
    fn test_discover_plans_reads_analysis_files() {
        let tmp = TempDir::new().unwrap();
        let plans_dir = tmp.path().join("v1").join("2020").join("plans");
        let plan_dir = make_plan(&plans_dir, "vt_cong_2020", "VT", "congressional", "2020", 1);

        // Write analysis files
        write_json(
            &plan_dir.join("analysis").join("compactness.json"),
            r#"{"mean_polsby_popper": 0.42}"#,
        );
        write_json(
            &plan_dir.join("analysis").join("summary.json"),
            r#"{"max_deviation_pct": 0.15}"#,
        );
        write_json(
            &plan_dir.join("analysis").join("splits.json"),
            r#"{"split_count": 3}"#,
        );
        write_json(
            &plan_dir.join("analysis").join("contiguity.json"),
            r#"{"all_contiguous": true}"#,
        );

        let result = discover_plans(tmp.path().to_str().unwrap(), "v1", "2020");
        assert_eq!(result.len(), 1);
        let plan = &result[0];
        assert!((plan.mean_pp.unwrap() - 0.42).abs() < 1e-9);
        assert!((plan.max_deviation_pct.unwrap() - 0.15).abs() < 1e-9);
        assert_eq!(plan.county_splits, Some(3));
        assert_eq!(plan.all_contiguous, Some(true));
    }

    #[test]
    fn test_discover_plans_sorted_by_label() {
        let tmp = TempDir::new().unwrap();
        let plans_dir = tmp.path().join("v1").join("2020").join("plans");
        make_plan(&plans_dir, "zz_cong_2020", "ZZ", "congressional", "2020", 1);
        make_plan(&plans_dir, "aa_cong_2020", "AA", "congressional", "2020", 1);
        make_plan(&plans_dir, "mm_cong_2020", "MM", "congressional", "2020", 1);

        let result = discover_plans(tmp.path().to_str().unwrap(), "v1", "2020");
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].label, "aa_cong_2020");
        assert_eq!(result[1].label, "mm_cong_2020");
        assert_eq!(result[2].label, "zz_cong_2020");
    }
}
