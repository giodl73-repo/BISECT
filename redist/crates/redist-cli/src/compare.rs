/// `redist compare` dispatcher — load two plans and compare them.
///
/// Supports plan-a vs plan-b (both labels) or plan-a vs enacted districts.
/// Output formats: table (default), json, csv.
use std::collections::HashMap;
use std::path::PathBuf;

use redist_analysis::{
    compare_plans, format_comparison_json, format_comparison_csv, format_comparison_table,
};
use crate::args::{CompareArgs, CompareFormat};

/// Load a plan's final_assignments.json given a label, version, year, and base dir.
///
/// If `label` ends with `.rplan`, parses it as an RPLAN file and extracts assignments
/// from `rplan["assignments"]` (map of GEOID → district_id as integer).
/// If `label` is an absolute path or starts with `.` or `/`, also tries it as a
/// direct file path to final_assignments.json.
fn load_plan_assignments(
    label: &str,
    output_base: &str,
    version: &str,
    year: &str,
    output_dir_override: Option<&PathBuf>,
) -> anyhow::Result<HashMap<String, usize>> {
    // If label ends with .rplan, parse it as RPLAN format.
    if label.ends_with(".rplan") {
        let content = std::fs::read_to_string(label)
            .map_err(|e| anyhow::anyhow!("cannot read .rplan file '{}': {}", label, e))?;
        // Parse as generic JSON to extract assignments (map of GEOID → district_id as integer)
        let v: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| anyhow::anyhow!("failed to parse .rplan '{}': {}", label, e))?;
        let assignments_obj = v["assignments"]
            .as_object()
            .ok_or_else(|| anyhow::anyhow!(".rplan file '{}' missing 'assignments' field", label))?;
        let assignments: HashMap<String, usize> = assignments_obj
            .iter()
            .map(|(geoid, dist_val)| {
                let dist = dist_val.as_u64()
                    .ok_or_else(|| anyhow::anyhow!(
                        "assignment value for GEOID {} must be integer, got {}", geoid, dist_val
                    ))?;
                Ok((geoid.clone(), dist as usize))
            })
            .collect::<anyhow::Result<_>>()?;
        return Ok(assignments);
    }

    let base = if let Some(od) = output_dir_override {
        od.clone()
    } else {
        PathBuf::from(output_base).join(version)
    };

    // If label looks like an absolute or relative file path, try it directly as
    // final_assignments.json (or a directory containing data/final_assignments.json).
    let is_path_like = label.starts_with('/') || label.starts_with('\\')
        || label.starts_with('.')
        || (label.len() >= 3 && label.chars().nth(1) == Some(':'));
    if is_path_like {
        let p = PathBuf::from(label);
        let direct = p.join("data").join("final_assignments.json");
        if direct.exists() {
            let raw: HashMap<String, usize> = serde_json::from_str(
                &std::fs::read_to_string(&direct)?
            )?;
            return Ok(raw);
        }
        if p.is_file() {
            let raw: HashMap<String, usize> = serde_json::from_str(
                &std::fs::read_to_string(&p)?
            )?;
            return Ok(raw);
        }
    }

    // Try labeled plan path: {base}/{year}/plans/{label}/data/final_assignments.json
    // (this mirrors the path written by runner.rs: {output_dir}/{year}/plans/{label}/data/)
    let plan_path = base
        .join(year)
        .join("plans")
        .join(label)
        .join("data")
        .join("final_assignments.json");
    if plan_path.exists() {
        let raw: HashMap<String, usize> = serde_json::from_str(
            &std::fs::read_to_string(&plan_path)?
        )?;
        return Ok(raw);
    }

    // Fallback: {base}/{year}/plans/{label}/final_assignments.json (no data/ subdir)
    let flat_path = base
        .join(year)
        .join("plans")
        .join(label)
        .join("final_assignments.json");
    if flat_path.exists() {
        let raw: HashMap<String, usize> = serde_json::from_str(
            &std::fs::read_to_string(&flat_path)?
        )?;
        return Ok(raw);
    }

    // Also accept a bare directory path as label
    let direct_data = PathBuf::from(label).join("data").join("final_assignments.json");
    if direct_data.exists() {
        let raw: HashMap<String, usize> = serde_json::from_str(
            &std::fs::read_to_string(&direct_data)?
        )?;
        return Ok(raw);
    }

    anyhow::bail!(
        "Plan '{}' not found at {} or {}",
        label,
        plan_path.display(),
        flat_path.display()
    )
}

/// Read the `year` field from a plan's manifest.json (for cross-year comparison guard).
///
/// When `label` ends with `.rplan`, reads `rplan["metadata"]["year"]` directly
/// from the RPLAN file instead of looking for manifest.json — because .rplan files
/// have no accompanying manifest.
///
/// Returns None if the manifest/rplan is absent or the year field is missing.
fn read_plan_year(
    label: &str,
    output_base: &str,
    version: &str,
    year: &str,
    output_dir_override: Option<&PathBuf>,
) -> Option<String> {
    // .rplan path: year lives in rplan["metadata"]["year"]
    if label.ends_with(".rplan") {
        let content = std::fs::read_to_string(label).ok()?;
        let v: serde_json::Value = serde_json::from_str(&content).ok()?;
        return v["metadata"]["year"].as_str().map(String::from);
    }

    let base = if let Some(od) = output_dir_override {
        od.clone()
    } else {
        PathBuf::from(output_base).join(version)
    };
    let manifest_path = base.join(year).join("plans").join(label).join("manifest.json");
    let content = std::fs::read_to_string(&manifest_path).ok()?;
    let v: serde_json::Value = serde_json::from_str(&content).ok()?;
    v.get("year")?.as_str().map(String::from)
}

/// Run the `redist compare` command.
pub fn run_compare(args: &CompareArgs) -> anyhow::Result<()> {
    let version = args.version.as_deref().unwrap_or("v1");

    // Load plan A
    let assignments_a = load_plan_assignments(
        &args.plan_a,
        &args.output_base,
        version,
        &args.year,
        args.output_dir.as_ref(),
    )?;

    // Load plan B (or enacted)
    let assignments_b = if let Some(ref plan_b) = args.plan_b {
        load_plan_assignments(
            plan_b,
            &args.output_base,
            version,
            &args.year,
            args.output_dir.as_ref(),
        )?
    } else if args.enacted {
        // Enacted comparison: not yet implemented (requires shapefile download)
        // For now, return an informative error.
        anyhow::bail!(
            "Enacted district comparison requires `redist fetch --type enacted` first. \
             Download enacted districts and rerun with --plan-b."
        );
    } else {
        anyhow::bail!(
            "Must provide either --plan-b <label> or --enacted"
        );
    };

    // Cross-year comparison guard: warn if plans are from different census years
    // (Jaccard similarity is meaningless across years — different tract GEOIDs)
    if !args.allow_cross_year {
        let year_a = read_plan_year(&args.plan_a, &args.output_base, version, &args.year, args.output_dir.as_ref());
        let year_b = args.plan_b.as_deref()
            .and_then(|b| read_plan_year(b, &args.output_base, version, &args.year, args.output_dir.as_ref()));
        if let (Some(ya), Some(yb)) = (year_a, year_b) {
            if ya != yb {
                eprintln!(
                    "WARNING: Plans are from different census years ({ya} vs {yb}). \
                     Jaccard similarity compares tract GEOIDs directly -- across census years, \
                     the same geographic area has different GEOIDs and similarity will be \
                     artificially low. Compare only plans within the same census year. \
                     Pass --allow-cross-year to suppress this warning for intentional \
                     cross-cycle comparisons."
                );
            }
        }
    }

    // Run comparison
    let mut comparison = compare_plans(&assignments_a, &assignments_b);
    // Set labels from args
    comparison.plan_a.label = args.plan_a.clone();
    comparison.plan_b.label = args
        .plan_b
        .clone()
        .unwrap_or_else(|| "enacted".to_string());

    // Format output
    let output = match args.format {
        CompareFormat::Json => format_comparison_json(&comparison),
        CompareFormat::Csv => format_comparison_csv(&comparison),
        CompareFormat::Table => format_comparison_table(&comparison),
    };

    // Write to file or stdout
    if let Some(ref out_path) = args.out {
        if let Some(parent) = out_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(out_path, &output)?;
        eprintln!("[OK] comparison -> {}", out_path.display());
    } else {
        print!("{output}");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::args::CompareArgs;

    fn parse_compare_args(extra: &[&str]) -> Result<CompareArgs, clap::Error> {
        use clap::Parser;
        let mut base = vec!["compare"];
        base.extend_from_slice(extra);
        CompareArgs::try_parse_from(base)
    }

    #[test]
    fn test_compare_args_plan_a_required() {
        let result = parse_compare_args(&["--plan-b", "plan2"]);
        assert!(result.is_err(), "--plan-a is required");
    }

    #[test]
    fn test_compare_args_plan_b_optional() {
        // Plan-a required; plan-b optional (enacted can be specified instead)
        let result = parse_compare_args(&["--plan-a", "plan1"]);
        // Parse succeeds; validation at runtime catches the neither-b-nor-enacted case
        if let Ok(args) = result {
            assert!(args.plan_b.is_none() && !args.enacted);
        }
    }

    #[test]
    fn test_compare_format_json_parsed() {
        let args = parse_compare_args(&[
            "--plan-a", "plan1", "--plan-b", "plan2",
            "--format", "json",
        ]).unwrap();
        assert!(matches!(args.format, CompareFormat::Json));
    }

    #[test]
    fn test_compare_format_csv_parsed() {
        let args = parse_compare_args(&[
            "--plan-a", "plan1", "--plan-b", "plan2",
            "--format", "csv",
        ]).unwrap();
        assert!(matches!(args.format, CompareFormat::Csv));
    }

    #[test]
    fn test_compare_format_table_is_default() {
        let args = parse_compare_args(&["--plan-a", "plan1", "--plan-b", "plan2"]).unwrap();
        assert!(matches!(args.format, CompareFormat::Table));
    }

    #[test]
    fn test_compare_enacted_flag_parsed() {
        let args = parse_compare_args(&["--plan-a", "plan1", "--enacted"]).unwrap();
        assert!(args.enacted);
        assert!(args.plan_b.is_none());
    }

    /// Task 113: read_plan_year from .rplan file
    /// When the label is a .rplan path, year is read from metadata.year (not manifest.json).
    #[test]
    fn test_read_plan_year_from_rplan_file() {
        let tmp = tempfile::TempDir::new().unwrap();
        let rplan_path = tmp.path().join("test_plan.rplan");

        let rplan_json = serde_json::json!({
            "rplan_version": "0.1",
            "metadata": {
                "label": "wa_test",
                "state_fips": "53",
                "state_code": "WA",
                "year": "2010",
                "chamber": "congressional",
                "num_districts": 9,
                "population_source": "total",
                "balance_tolerance_pct": 0.5,
                "created_at": "2026-04-26T00:00:00Z",
                "created_by": "test"
            },
            "assignments": {"53001000100": 1},
            "geometry": null
        });
        std::fs::write(&rplan_path, rplan_json.to_string()).unwrap();

        let path_str = rplan_path.to_str().unwrap();
        let year = read_plan_year(path_str, "outputs", "v1", "2020", None);
        assert_eq!(year, Some("2010".to_string()),
            "should read year 2010 from .rplan metadata, not manifest.json");
    }

    /// Task 113: cross-year warning fires when comparing .rplan files with different years.
    /// Tests the year-reading logic directly (no actual plan comparison needed).
    #[test]
    fn test_cross_year_warning_fires_for_rplan() {
        let tmp = tempfile::TempDir::new().unwrap();

        let make_rplan = |year: &str, path: &std::path::Path| {
            let json = serde_json::json!({
                "rplan_version": "0.1",
                "metadata": {
                    "label": "test",
                    "state_fips": "00",
                    "state_code": "XX",
                    "year": year,
                    "chamber": "congressional",
                    "num_districts": 1,
                    "population_source": "total",
                    "balance_tolerance_pct": 0.5,
                    "created_at": "2026-04-26T00:00:00Z",
                    "created_by": "test"
                },
                "assignments": {},
                "geometry": null
            });
            std::fs::write(path, json.to_string()).unwrap();
        };

        let plan_a_path = tmp.path().join("plan_2020.rplan");
        let plan_b_path = tmp.path().join("plan_2010.rplan");
        make_rplan("2020", &plan_a_path);
        make_rplan("2010", &plan_b_path);

        let year_a = read_plan_year(plan_a_path.to_str().unwrap(), "outputs", "v1", "2020", None);
        let year_b = read_plan_year(plan_b_path.to_str().unwrap(), "outputs", "v1", "2020", None);

        assert_eq!(year_a, Some("2020".to_string()), "plan A year should be 2020");
        assert_eq!(year_b, Some("2010".to_string()), "plan B year should be 2010");

        // The warning condition: years differ
        let should_warn = year_a.as_deref() != year_b.as_deref();
        assert!(should_warn, "cross-year warning should fire when rplan years differ");
    }

    /// Task 116: --allow-cross-year flag parses correctly.
    #[test]
    fn test_allow_cross_year_flag_parsed() {
        let args = parse_compare_args(&[
            "--plan-a", "plan1",
            "--plan-b", "plan2",
            "--allow-cross-year",
        ]).unwrap();
        assert!(args.allow_cross_year, "--allow-cross-year flag should be true when provided");
    }

    /// Task 116: --allow-cross-year defaults to false.
    #[test]
    fn test_allow_cross_year_flag_defaults_false() {
        let args = parse_compare_args(&["--plan-a", "plan1", "--plan-b", "plan2"]).unwrap();
        assert!(!args.allow_cross_year, "--allow-cross-year should default to false");
    }

    /// Scenario 14: External .rplan compare
    /// load_plan_assignments should parse a .rplan file directly when path ends with .rplan
    #[test]
    fn test_compare_accepts_rplan_path() {
        let tmp = tempfile::TempDir::new().unwrap();
        let rplan_path = tmp.path().join("enacted.rplan");

        // Write a minimal RPLAN file with assignments
        let rplan_json = serde_json::json!({
            "rplan_version": "0.1",
            "metadata": {
                "label": "wa_enacted",
                "state_fips": "53",
                "state_code": "WA",
                "year": "2020",
                "chamber": "congressional",
                "num_districts": 10,
                "population_source": "total",
                "balance_tolerance_pct": 0.5,
                "created_at": "2026-04-26T00:00:00Z",
                "created_by": "test"
            },
            "assignments": {
                "53033000100": 1,
                "53033000200": 2,
                "53033000300": 1
            },
            "geometry": null
        });
        std::fs::write(&rplan_path, rplan_json.to_string()).unwrap();

        let path_str = rplan_path.to_str().unwrap();
        let result = load_plan_assignments(path_str, "outputs", "v1", "2020", None);
        assert!(result.is_ok(), "should load .rplan file: {:?}", result.err());
        let assignments = result.unwrap();
        assert_eq!(assignments.len(), 3, "should have 3 assignments");
        assert_eq!(assignments["53033000100"], 1);
        assert_eq!(assignments["53033000200"], 2);
        assert_eq!(assignments["53033000300"], 1);
    }
}
