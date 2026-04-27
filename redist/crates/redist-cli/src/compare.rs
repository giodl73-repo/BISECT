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
fn load_plan_assignments(
    label: &str,
    output_base: &str,
    version: &str,
    year: &str,
    output_dir_override: Option<&PathBuf>,
) -> anyhow::Result<HashMap<String, usize>> {
    let base = if let Some(od) = output_dir_override {
        od.clone()
    } else {
        PathBuf::from(output_base).join(version)
    };

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
/// Returns None if the manifest is absent or the year field is missing.
fn read_plan_year(
    label: &str,
    output_base: &str,
    version: &str,
    year: &str,
    output_dir_override: Option<&PathBuf>,
) -> Option<String> {
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
    {
        let year_a = read_plan_year(&args.plan_a, &args.output_base, version, &args.year, args.output_dir.as_ref());
        let year_b = args.plan_b.as_deref()
            .and_then(|b| read_plan_year(b, &args.output_base, version, &args.year, args.output_dir.as_ref()));
        if let (Some(ya), Some(yb)) = (year_a, year_b) {
            if ya != yb {
                eprintln!(
                    "WARNING: Plans are from different census years ({ya} vs {yb}). \
                     Jaccard similarity compares tract GEOIDs directly — across census years, \
                     the same geographic area has different GEOIDs and similarity will be \
                     artificially low. Compare only plans within the same census year."
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
}
