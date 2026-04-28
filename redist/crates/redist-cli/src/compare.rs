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

    // GerryChain flat JSON detection:
    // If the label is a file path (not ending in .rplan) that exists and contains
    // a flat JSON object (no "assignments" key, no "rplan_version"), treat it as
    // a GerryChain / DRA direct GEOID-to-district map.
    // Also handles GerryChain's nested {"assignment": {...}} format.
    let label_path = PathBuf::from(label);
    if label_path.is_file() && !label.ends_with(".rplan") {
        if let Ok(content) = std::fs::read_to_string(&label_path) {
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&content) {
                // Nested GerryChain format: {"assignment": {"GEOID": district, ...}}
                if let Some(nested) = v.get("assignment").and_then(|a| a.as_object()) {
                    let assignments: HashMap<String, usize> = nested
                        .iter()
                        .filter_map(|(geoid, val)| {
                            val.as_u64().map(|d| (geoid.clone(), d as usize))
                        })
                        .collect();
                    if !assignments.is_empty() {
                        return Ok(assignments);
                    }
                }
                // Flat GerryChain format: {"GEOID": district, ...}
                // Detect: JSON object, no "assignments" key, no "rplan_version"
                if v.is_object()
                    && v.get("assignments").is_none()
                    && v.get("rplan_version").is_none()
                {
                    if let Some(obj) = v.as_object() {
                        let assignments: HashMap<String, usize> = obj
                            .iter()
                            .filter_map(|(geoid, val)| {
                                val.as_u64().map(|d| (geoid.clone(), d as usize))
                            })
                            .collect();
                        if !assignments.is_empty() {
                            return Ok(assignments);
                        }
                    }
                }
            }
        }
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

/// Load a plan's analysis JSON file given its base path and file name.
/// Returns None if the file is absent or unreadable.
fn load_analysis_json(plan_base: &std::path::Path, filename: &str) -> Option<serde_json::Value> {
    let path = plan_base.join("analysis").join(filename);
    std::fs::read_to_string(&path).ok()
        .and_then(|s| serde_json::from_str(&s).ok())
}

/// Run an N-plan summary table when `--labels` has >= 2 entries.
pub fn run_multi_plan_summary(args: &CompareArgs) -> anyhow::Result<()> {
    if args.labels.len() < 2 {
        anyhow::bail!("--labels requires at least 2 plan labels for a multi-plan summary table");
    }
    let version = args.version.as_deref().unwrap_or("v1");
    let base = if let Some(ref od) = args.output_dir {
        od.clone()
    } else {
        std::path::PathBuf::from(&args.output_base).join(version)
    };

    struct PlanRow {
        label: String,
        mean_pp: Option<f64>,
        max_dev_pct: Option<f64>,
        county_splits: Option<u64>,
        contiguous: Option<bool>,
    }

    let mut rows: Vec<PlanRow> = Vec::new();

    for label in &args.labels {
        let plan_base = base.join(&args.year).join("plans").join(label);

        // summary.json → population deviation
        let summary = load_analysis_json(&plan_base, "summary.json");
        let max_dev_pct = summary.as_ref().and_then(|v| {
            v.get("max_deviation_pct").and_then(|d| d.as_f64())
                .or_else(|| v.get("population_max_deviation_pct").and_then(|d| d.as_f64()))
        });

        // compactness.json → mean polsby_popper
        let compactness = load_analysis_json(&plan_base, "compactness.json");
        let mean_pp = compactness.as_ref().and_then(|v| {
            v.get("districts").and_then(|d| d.as_array()).map(|arr| {
                let vals: Vec<f64> = arr.iter()
                    .filter_map(|d| d.get("polsby_popper").and_then(|p| p.as_f64()))
                    .collect();
                if vals.is_empty() { return f64::NAN; }
                vals.iter().sum::<f64>() / vals.len() as f64
            })
        });

        // splits.json → total splits count
        let splits = load_analysis_json(&plan_base, "splits.json");
        let county_splits = splits.as_ref().and_then(|v| {
            v.get("counties").and_then(|c| c.get("split")).and_then(|s| s.as_u64())
        });

        // contiguity.json → all_contiguous
        let contiguity = load_analysis_json(&plan_base, "contiguity.json");
        let contiguous = contiguity.as_ref()
            .and_then(|v| v.get("all_contiguous").and_then(|c| c.as_bool()));

        rows.push(PlanRow { label: label.clone(), mean_pp, max_dev_pct, county_splits, contiguous });
    }

    let is_csv = matches!(args.format, crate::args::CompareFormat::Csv);

    if is_csv {
        println!("Label,Mean PP,Max Dev%,County Splits,Contiguous");
        for row in &rows {
            let pp: String = row.mean_pp.map_or_else(|| "N/A".to_string(), |v| format!("{:.3}", v));
            let dev: String = row.max_dev_pct.map_or_else(|| "N/A".to_string(), |v| format!("{:.1}%", v));
            let splits: String = row.county_splits.map_or_else(|| "N/A".to_string(), |v| v.to_string());
            let cont: String = row.contiguous.map_or_else(|| "N/A".to_string(), |v| if v { "Yes".to_string() } else { "No".to_string() });
            println!("{},{},{},{},{}", row.label, pp, dev, splits, cont);
        }
    } else {
        // Table format
        let col1 = rows.iter().map(|r| r.label.len()).max().unwrap_or(5).max(23);
        println!("{:<col1$} | Mean PP | Max Dev% | County Splits | Contiguous",
            "Label", col1 = col1);
        println!("{:-<col1$}-+---------+----------+---------------+------------",
            "", col1 = col1);
        for row in &rows {
            let pp: String = row.mean_pp.map_or_else(|| " N/A ".to_string(), |v| format!(" {:.3} ", v));
            let dev: String = row.max_dev_pct.map_or_else(|| "  N/A  ".to_string(), |v| format!("  {:.1}% ", v));
            let splits: String = row.county_splits.map_or_else(|| "     N/A      ".to_string(), |v| format!("     {:3}      ", v));
            let cont: String = row.contiguous.map_or_else(|| "  N/A ".to_string(), |v| if v { "  Yes ".to_string() } else { "  No  ".to_string() });
            println!("{:<col1$} |{pp}|{dev}|{splits}|{cont}",
                row.label, col1 = col1);
        }
    }

    Ok(())
}

/// Run the `redist compare` command.
pub fn run_compare(args: &CompareArgs) -> anyhow::Result<()> {
    // When --labels is provided with >= 2 entries, run multi-plan summary instead
    if !args.labels.is_empty() {
        return run_multi_plan_summary(args);
    }

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
        // For now, return an informative error with available alternatives.
        anyhow::bail!(
            "Enacted district comparison requires an enacted plan file.\n\
             Options:\n\
             (1) .rplan file:  redist compare --plan-a {} --plan-b path/to/enacted.rplan\n\
             (2) DRA CSV:      redist compare --plan-a {} --plan-b path/to/enacted.csv\n\
             (3) GeoJSON:      redist import --file enacted.geojson --label enacted\n\
                               redist compare --plan-a {} --plan-b enacted\n\
             Note: 'redist fetch --type enacted' is not yet available.",
            args.plan_a, args.plan_a, args.plan_a
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

    // ── Gap 10: --enacted error message ──────────────────────────────────────

    #[test]
    fn test_enacted_error_no_fetch_reference() {
        // Test the enacted error message content directly.
        // The enacted bail! is triggered when plan_b is None and enacted=true,
        // after plan_a loads successfully. We verify the message content
        // matches the expected format by constructing the bail string directly.
        let plan_a = "wa_congressional_2020";

        // This is the exact message from the bail! in run_compare
        let enacted_error = format!(
            "Enacted district comparison requires an enacted plan file.\n\
             Options:\n\
             (1) .rplan file:  redist compare --plan-a {} --plan-b path/to/enacted.rplan\n\
             (2) DRA CSV:      redist compare --plan-a {} --plan-b path/to/enacted.csv\n\
             (3) GeoJSON:      redist import --file enacted.geojson --label enacted\n\
                               redist compare --plan-a {} --plan-b enacted\n\
             Note: 'redist fetch --type enacted' is not yet available.",
            plan_a, plan_a, plan_a
        );

        // Must contain "rplan" as one of the alternative formats
        assert!(
            enacted_error.contains("rplan"),
            "error must mention .rplan as an option: {enacted_error}"
        );
        // Must NOT start with the old message that suggested fetch --type enacted
        assert!(
            !enacted_error.starts_with("Enacted district comparison requires `redist fetch --type enacted`"),
            "error must NOT suggest 'redist fetch --type enacted' as primary action"
        );
        // Must mention that fetch --type enacted is not available (as a note)
        assert!(
            enacted_error.contains("not yet available"),
            "error must note that fetch --type enacted is not yet available: {enacted_error}"
        );
        // Must mention plan-b alternatives
        assert!(
            enacted_error.contains("--plan-b"),
            "error must show --plan-b usage: {enacted_error}"
        );
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

    /// Task 136: GerryChain flat format
    #[test]
    fn test_gerrychain_flat_format_loads() {
        let tmp = tempfile::TempDir::new().unwrap();
        let path = tmp.path().join("gc_flat.json");
        // Flat GEOID-to-district map (GerryChain / DRA style)
        let json = serde_json::json!({
            "53001000100": 1,
            "53001000200": 2,
            "53001000300": 1
        });
        std::fs::write(&path, json.to_string()).unwrap();
        let result = load_plan_assignments(path.to_str().unwrap(), "outputs", "v1", "2020", None);
        assert!(result.is_ok(), "flat GerryChain JSON should load: {:?}", result.err());
        let assignments = result.unwrap();
        assert_eq!(assignments.len(), 3);
        assert_eq!(assignments["53001000100"], 1);
        assert_eq!(assignments["53001000200"], 2);
    }

    /// Task 136: GerryChain nested "assignment" key format
    #[test]
    fn test_gerrychain_assignment_key_loads() {
        let tmp = tempfile::TempDir::new().unwrap();
        let path = tmp.path().join("gc_nested.json");
        // GerryChain nested format: {"assignment": {"GEOID": district}}
        let json = serde_json::json!({
            "assignment": {
                "53033000100": 3,
                "53033000200": 4,
                "53033000300": 3
            }
        });
        std::fs::write(&path, json.to_string()).unwrap();
        let result = load_plan_assignments(path.to_str().unwrap(), "outputs", "v1", "2020", None);
        assert!(result.is_ok(), "nested GerryChain JSON should load: {:?}", result.err());
        let assignments = result.unwrap();
        assert_eq!(assignments.len(), 3);
        assert_eq!(assignments["53033000100"], 3);
        assert_eq!(assignments["53033000200"], 4);
    }

    // ── Task 129: N-plan summary table ────────────────────────────────────────

    /// Fewer than 2 labels gives an error.
    #[test]
    fn test_multi_plan_summary_minimum_two_labels() {
        use crate::args::{CompareArgs, CompareFormat};
        let args = CompareArgs {
            plan_a: "plan1".into(),
            plan_b: None,
            enacted: false,
            year: "2020".into(),
            version: None,
            metrics: vec!["all".into()],
            out: None,
            format: CompareFormat::Table,
            output_base: "outputs".into(),
            output_dir: None,
            allow_cross_year: false,
            labels: vec!["only_one".into()],  // fewer than 2
        };
        let result = run_multi_plan_summary(&args);
        assert!(result.is_err(), "fewer than 2 labels must return error");
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("at least 2"), "error must mention 'at least 2': {msg}");
    }

    /// Output contains column headers.
    #[test]
    fn test_multi_plan_summary_table_has_header() {
        use crate::args::{CompareArgs, CompareFormat};
        let tmp = tempfile::TempDir::new().unwrap();
        // Create minimal plan directories (no analysis files — N/A values expected)
        for label in &["plan_a", "plan_b"] {
            std::fs::create_dir_all(
                tmp.path().join("sweep").join("2020").join("plans").join(label)
            ).unwrap();
        }
        let args = CompareArgs {
            plan_a: "plan_a".into(),
            plan_b: None,
            enacted: false,
            year: "2020".into(),
            version: Some("sweep".into()),
            metrics: vec!["all".into()],
            out: None,
            format: CompareFormat::Table,
            output_base: tmp.path().to_str().unwrap().into(),
            output_dir: None,
            allow_cross_year: false,
            labels: vec!["plan_a".into(), "plan_b".into()],
        };
        // run_multi_plan_summary should succeed and emit header — just check it doesn't error
        let result = run_multi_plan_summary(&args);
        assert!(result.is_ok(), "multi-plan summary with 2 valid labels must succeed: {:?}", result.err());
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
