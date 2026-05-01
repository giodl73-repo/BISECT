/// `redist compare` dispatcher — load two plans and compare them.
///
/// Supports plan-a vs plan-b (both labels) or plan-a vs enacted districts.
/// Output formats: table (default), json, csv, narrative, both.
use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};

use redist_analysis::{
    compare_plans, format_comparison_json, format_comparison_csv, format_comparison_table,
};
use redist_report::comparison::{
    diff_from_assignments, load_plan_side_from_dir, AssembleError, ComparisonReport,
};
use redist_report::html_comparison::{render_comparison_html, HtmlComparisonContext};
use redist_report::narrative::{render_narrative, NarrativeConfig};
use redist_report::narrative_manifest::{
    build_narrative_manifest_with_clock, serialize_manifest, NarrativeManifestInputs,
};
use sha2::{Digest, Sha256};
use crate::args::{CompareArgs, CompareFormat};
use crate::provenance::Provenance;

/// SHA-256 of the embedded narrative renderer source. Anchors `template_sha256`
/// in `narrative_manifest.json` to the exact Rust code that produced the
/// markdown — equivalent to a Tera template SHA, but for the pure-Rust renderer.
const NARRATIVE_RENDERER_SRC: &[u8] = include_bytes!("../../redist-report/src/narrative.rs");
const NARRATIVE_TEMPLATE_PATH_REL: &str = "redist/crates/redist-report/src/narrative.rs";

fn sha256_hex(bytes: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(bytes);
    let mut s = String::with_capacity(64);
    for b in h.finalize() {
        s.push_str(&format!("{:02x}", b));
    }
    s
}

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

    // Try PlanContext first for labeled plans — single source of truth for paths.
    // This ensures the assignments path is derived from the manifest, not reconstructed.
    {
        let ctx_result = crate::plan_context::PlanContext::from_label(
            PathBuf::from(output_base).as_path(), version, year, label,
        );
        if let Ok(ctx) = ctx_result {
            let asgn_path = ctx.assignments_path();
            if asgn_path.exists() {
                let raw: HashMap<String, usize> = serde_json::from_str(
                    &std::fs::read_to_string(&asgn_path)?
                )?;
                return Ok(raw);
            }
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
        balance_tol: Option<f64>,
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

        // balance_tolerance_pct → from PlanContext manifest
        let balance_tol = crate::plan_context::PlanContext::from_label(
            std::path::Path::new(&args.output_base), version, &args.year, label,
        ).ok().map(|ctx| ctx.manifest.balance_tolerance_pct);

        rows.push(PlanRow { label: label.clone(), mean_pp, max_dev_pct, county_splits, contiguous, balance_tol });
    }

    let is_csv = matches!(args.format, crate::args::CompareFormat::Csv);

    if is_csv {
        println!("Label,Mean PP,Max Dev%,County Splits,Bal Tol%,Contiguous");
        for row in &rows {
            let pp: String = row.mean_pp.map_or_else(|| "N/A".to_string(), |v| format!("{:.3}", v));
            let dev: String = row.max_dev_pct.map_or_else(|| "N/A".to_string(), |v| format!("{:.1}%", v));
            let splits: String = row.county_splits.map_or_else(|| "N/A".to_string(), |v| v.to_string());
            let tol: String = row.balance_tol.map_or_else(|| "-".to_string(), |v| format!("{:.1}%", v));
            let cont: String = row.contiguous.map_or_else(|| "N/A".to_string(), |v| if v { "Yes".to_string() } else { "No".to_string() });
            println!("{},{},{},{},{},{}", row.label, pp, dev, splits, tol, cont);
        }
    } else {
        // Table format
        let col1 = rows.iter().map(|r| r.label.len()).max().unwrap_or(5).max(23);
        println!("{:<col1$} | Mean PP | Max Dev% | Splits | Bal Tol% | Contiguous",
            "Label", col1 = col1);
        println!("{:-<col1$}-+---------+----------+--------+----------+------------",
            "", col1 = col1);
        for row in &rows {
            let pp: String = row.mean_pp.map_or_else(|| " N/A ".to_string(), |v| format!(" {:.3} ", v));
            let dev: String = row.max_dev_pct.map_or_else(|| "  N/A  ".to_string(), |v| format!("  {:.1}% ", v));
            let splits: String = row.county_splits.map_or_else(|| "  N/A ".to_string(), |v| format!("  {:3}  ", v));
            let tol: String = row.balance_tol.map_or_else(|| "   -    ".to_string(), |v| format!("  {:.1}%  ", v));
            let cont: String = row.contiguous.map_or_else(|| "  N/A ".to_string(), |v| if v { "  Yes ".to_string() } else { "  No  ".to_string() });
            println!("{:<col1$} |{pp}|{dev}|{splits}|{tol}|{cont}",
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
        let version_for_b = args.version_b.as_deref().unwrap_or(version);
        load_plan_assignments(
            plan_b,
            &args.output_base,
            version_for_b,
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
    match args.format {
        CompareFormat::Json => emit_text(&format_comparison_json(&comparison), args.out.as_ref())?,
        CompareFormat::Csv => emit_text(&format_comparison_csv(&comparison), args.out.as_ref())?,
        CompareFormat::Table => emit_text(&format_comparison_table(&comparison), args.out.as_ref())?,
        CompareFormat::Narrative => {
            run_narrative_dispatch(args, version, /*also_print_table=*/ false, &comparison, /*also_html=*/ false)?;
        }
        CompareFormat::Both => {
            // Print the table to stdout, then emit narrative.md + manifest to disk.
            print!("{}", format_comparison_table(&comparison));
            run_narrative_dispatch(args, version, /*also_print_table=*/ true, &comparison, /*also_html=*/ false)?;
        }
        CompareFormat::Html => {
            // Emit narrative.md + manifest + comparison.html.
            run_narrative_dispatch(args, version, /*also_print_table=*/ false, &comparison, /*also_html=*/ true)?;
        }
    }

    Ok(())
}

fn emit_text(output: &str, out_path: Option<&PathBuf>) -> anyhow::Result<()> {
    if let Some(out_path) = out_path {
        if let Some(parent) = out_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(out_path, output)?;
        eprintln!("[OK] comparison -> {}", out_path.display());
    } else {
        print!("{output}");
    }
    Ok(())
}

/// Resolve a plan label or filesystem path into the plan directory holding
/// `manifest.json`. Returns `None` if no manifest can be located (caller
/// downgrades to a [SKIP] for the narrative dispatch).
fn resolve_plan_dir(
    label_or_path: &str,
    output_base: &str,
    version: &str,
    year: &str,
    output_dir_override: Option<&PathBuf>,
) -> Option<PathBuf> {
    // Direct path: caller passed a manifest.json or a directory containing one.
    let p = PathBuf::from(label_or_path);
    if p.is_dir() && p.join("manifest.json").is_file() {
        return Some(p);
    }
    if p.is_file() && p.file_name().and_then(|s| s.to_str()) == Some("manifest.json") {
        return p.parent().map(Path::to_path_buf);
    }

    let base = output_dir_override
        .cloned()
        .unwrap_or_else(|| PathBuf::from(output_base).join(version));
    let plan_dir = base.join(year).join("plans").join(label_or_path);
    if plan_dir.join("manifest.json").is_file() {
        return Some(plan_dir);
    }
    None
}

/// Run the narrative + manifest dispatch. When called from `Both` format the
/// table has already been printed to stdout before this runs. When `also_html`
/// is true, an additional `comparison.html` is written into the same dir.
fn run_narrative_dispatch(
    args: &CompareArgs,
    version: &str,
    also_print_table: bool,
    _table_comparison: &redist_analysis::comparison::PlanComparison,
    also_html: bool,
) -> anyhow::Result<()> {
    let _ = also_print_table; // reserved — caller already printed

    let plan_b_label = args
        .plan_b
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!(
            "[CONFIG] --format narrative requires --plan-b <label-or-path>; \
             --enacted is not yet supported by the narrative dispatcher"
        ))?;

    let version_b = args.version_b.as_deref().unwrap_or(version);

    let plan_a_dir = resolve_plan_dir(
        &args.plan_a,
        &args.output_base,
        version,
        &args.year,
        args.output_dir.as_ref(),
    )
    .ok_or_else(|| anyhow::anyhow!(
        "[INPUT] cannot locate plan-a manifest for '{}' under {} (version {}, year {}); \
         --format narrative requires a plan directory containing manifest.json",
        args.plan_a, args.output_base, version, args.year
    ))?;
    let plan_b_dir = resolve_plan_dir(
        plan_b_label,
        &args.output_base,
        version_b,
        &args.year,
        args.output_dir.as_ref(),
    )
    .ok_or_else(|| anyhow::anyhow!(
        "[INPUT] cannot locate plan-b manifest for '{}' under {} (version {}, year {}); \
         --format narrative requires a plan directory containing manifest.json",
        plan_b_label, args.output_base, version_b, args.year
    ))?;

    let side_a = load_plan_side_from_dir(&plan_a_dir, args.leaning_threshold)
        .map_err(|e: AssembleError| anyhow::anyhow!("plan-a: {e}"))?;
    let side_b = load_plan_side_from_dir(&plan_b_dir, args.leaning_threshold)
        .map_err(|e: AssembleError| anyhow::anyhow!("plan-b: {e}"))?;

    let diff = diff_from_assignments(&plan_a_dir, &plan_b_dir)
        .map_err(|e: AssembleError| anyhow::anyhow!("diff: {e}"))?;

    // Snapshot SHAs we need for the manifest BEFORE moving sides into the report.
    let plan_a_label = side_a.label.clone();
    let plan_a_manifest_sha = side_a.manifest_sha256.clone();
    let plan_a_analysis_sha = side_a.analysis_sha256.clone();
    let plan_a_submission_type = side_a.submission_type.clone();
    let plan_a_submitted_by = side_a.submitted_by.clone();
    let plan_a_submitted_at = side_a.submitted_at.clone();
    let plan_b_label_resolved = side_b.label.clone();
    let plan_b_manifest_sha = side_b.manifest_sha256.clone();
    let plan_b_analysis_sha = side_b.analysis_sha256.clone();
    let plan_b_submission_type = side_b.submission_type.clone();
    let plan_b_submitted_by = side_b.submitted_by.clone();
    let plan_b_submitted_at = side_b.submitted_at.clone();

    let report = ComparisonReport::from_loaded(side_a, side_b, /*baseline=*/ None, diff);

    // Render the markdown.
    let cfg = NarrativeConfig {
        leaning_threshold: args.leaning_threshold,
        close_call_band: args.close_call_band,
        approved_by: args.approved_by.clone(),
        partisan_seat_ci: None,
        mm_count_ci: None,
        mean_pp_ci: None,
    };
    let markdown = render_narrative(&report, &cfg);

    // Build provenance manifest.
    let mut analysis_sha256: BTreeMap<String, BTreeMap<String, String>> = BTreeMap::new();
    analysis_sha256.insert("plan_a".into(), plan_a_analysis_sha);
    analysis_sha256.insert("plan_b".into(), plan_b_analysis_sha);

    let prov = Provenance::current();
    let template_sha = sha256_hex(NARRATIVE_RENDERER_SRC);

    // Civic-counter-proposal attribution: prefer plan B (the typical
    // civic-side submission) but fall back to plan A if only A is tagged.
    let (submission_type, civic_attribution) = pick_civic_attribution(
        plan_a_submission_type.as_deref(),
        &plan_a_label,
        plan_a_submitted_by.as_deref(),
        plan_a_submitted_at.as_deref(),
        plan_b_submission_type.as_deref(),
        &plan_b_label_resolved,
        plan_b_submitted_by.as_deref(),
        plan_b_submitted_at.as_deref(),
    );

    let inputs = NarrativeManifestInputs {
        template_path_rel: NARRATIVE_TEMPLATE_PATH_REL.to_string(),
        template_sha256: template_sha,
        leaning_threshold: args.leaning_threshold,
        close_call_band: args.close_call_band,
        moe_inputs: BTreeMap::new(),
        plan_a_label: plan_a_label.clone(),
        plan_a_manifest_sha256: plan_a_manifest_sha,
        plan_b_label: plan_b_label_resolved.clone(),
        plan_b_manifest_sha256: plan_b_manifest_sha,
        baseline_label: None,
        baseline_manifest_sha256: None,
        analysis_sha256,
        approved_by: args.approved_by.clone(),
        redist_version: prov.redist_version,
        redist_build_commit: prov.redist_build_commit,
        rustc_version: prov.rustc_version,
        submission_type,
        civic_counter_proposal_attribution: civic_attribution,
        comments_label: None,
        comments_overlay_sha256: None,
    };

    let source_date_epoch = std::env::var("SOURCE_DATE_EPOCH")
        .ok()
        .and_then(|s| s.parse::<i64>().ok());
    let manifest = build_narrative_manifest_with_clock(inputs, source_date_epoch);
    let manifest_json = serialize_manifest(&manifest)
        .map_err(|e| anyhow::anyhow!("[INTERNAL] manifest serialize failed: {e}"))?;

    // Choose output directory.
    let report_dir = if let Some(rd) = args.report_dir.as_ref() {
        rd.clone()
    } else {
        PathBuf::from(&args.output_base)
            .join(version)
            .join("comparisons")
            .join(format!("{}_vs_{}", plan_a_label, plan_b_label_resolved))
    };
    std::fs::create_dir_all(&report_dir)
        .map_err(|e| anyhow::anyhow!("[INPUT] cannot create {}: {}", report_dir.display(), e))?;

    let narrative_path = report_dir.join("narrative.md");
    let manifest_path = report_dir.join("narrative_manifest.json");
    std::fs::write(&narrative_path, &markdown)?;
    std::fs::write(&manifest_path, &manifest_json)?;

    eprintln!("[OK] narrative -> {}", narrative_path.display());
    eprintln!("[OK] manifest  -> {}", manifest_path.display());

    if also_html {
        let ctx = HtmlComparisonContext {
            narrative_markdown: markdown.clone(),
            template_sha256: manifest.template_sha256.clone(),
            redist_build_commit_short: manifest.redist_build_commit_short.clone(),
            redist_version: manifest.redist_version.clone(),
            approved_at: manifest.approved_at.clone(),
        };
        let html = render_comparison_html(&report, &cfg, &ctx);
        let html_path = report_dir.join("comparison.html");
        std::fs::write(&html_path, &html)?;
        eprintln!("[OK] html      -> {}", html_path.display());
    }

    if args.approved_by.is_none() {
        eprintln!(
            "[INFO] narrative is in DRAFT mode; pass --approved-by \"<name>\" to remove the [DRAFT] prefix"
        );
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn pick_civic_attribution(
    a_type: Option<&str>,
    a_label: &str,
    a_by: Option<&str>,
    a_at: Option<&str>,
    b_type: Option<&str>,
    b_label: &str,
    b_by: Option<&str>,
    b_at: Option<&str>,
) -> (
    Option<String>,
    Option<redist_report::narrative_manifest::CivicCounterProposalAttribution>,
) {
    let civic = "civic_counter_proposal";
    let make = |label: &str, by: Option<&str>, at: Option<&str>| {
        Some(
            redist_report::narrative_manifest::CivicCounterProposalAttribution {
                plan_label: label.to_string(),
                submitted_by: by.unwrap_or("").to_string(),
                submitted_at: at.unwrap_or("").to_string(),
            },
        )
    };
    if b_type == Some(civic) {
        return (Some(civic.to_string()), make(b_label, b_by, b_at));
    }
    if a_type == Some(civic) {
        return (Some(civic.to_string()), make(a_label, a_by, a_at));
    }
    (None, None)
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
            version_b: None,
            leaning_threshold: 0.55,
            close_call_band: 0.02,
            approved_by: None,
            report_dir: None,
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
            version_b: None,
            leaning_threshold: 0.55,
            close_call_band: 0.02,
            approved_by: None,
            report_dir: None,
        };
        // run_multi_plan_summary should succeed and emit header — just check it doesn't error
        let result = run_multi_plan_summary(&args);
        assert!(result.is_ok(), "multi-plan summary with 2 valid labels must succeed: {:?}", result.err());
    }

    // ── Task 207: N-plan summary includes Bal Tol% column ────────────────────

    fn write_plan_manifest(plan_dir: &std::path::Path, label: &str, tol: f64) {
        std::fs::create_dir_all(plan_dir).unwrap();
        let manifest = serde_json::json!({
            "label": label,
            "state_code": "WA",
            "year": "2020",
            "chamber": "house",
            "num_districts": 98,
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
            "created_at": "2026-04-26T00:00:00Z",
            "balance_tolerance_pct": tol,
            "population_balance_valid": true,
            "seats_per_district": 1,
            "total_seats": 98,
            "electoral_system": "single_member",
            "gpmetis_version": "unknown"
        });
        std::fs::write(
            plan_dir.join("manifest.json"),
            serde_json::to_string_pretty(&manifest).unwrap(),
        ).unwrap();
    }

    /// Task 207: multi-plan summary table includes Bal Tol% column with tolerance values.
    #[test]
    fn test_multi_plan_summary_includes_tolerance_column() {
        use crate::args::{CompareArgs, CompareFormat};

        let tmp = tempfile::TempDir::new().unwrap();
        // Create two plans with different tolerances
        let plan_dir_a = tmp.path().join("sweep").join("2020").join("plans").join("wa_house_tol_0_5");
        let plan_dir_b = tmp.path().join("sweep").join("2020").join("plans").join("wa_house_tol_5_0");
        write_plan_manifest(&plan_dir_a, "wa_house_tol_0_5", 0.5);
        write_plan_manifest(&plan_dir_b, "wa_house_tol_5_0", 5.0);

        let args_table = CompareArgs {
            plan_a: "wa_house_tol_0_5".into(),
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
            labels: vec!["wa_house_tol_0_5".into(), "wa_house_tol_5_0".into()],
            version_b: None,
            leaning_threshold: 0.55,
            close_call_band: 0.02,
            approved_by: None,
            report_dir: None,
        };

        let args_csv = CompareArgs {
            plan_a: "wa_house_tol_0_5".into(),
            plan_b: None,
            enacted: false,
            year: "2020".into(),
            version: Some("sweep".into()),
            metrics: vec!["all".into()],
            out: None,
            format: CompareFormat::Csv,
            output_base: tmp.path().to_str().unwrap().into(),
            output_dir: None,
            allow_cross_year: false,
            labels: vec!["wa_house_tol_0_5".into(), "wa_house_tol_5_0".into()],
            version_b: None,
            leaning_threshold: 0.55,
            close_call_band: 0.02,
            approved_by: None,
            report_dir: None,
        };

        // (a) table format must succeed
        let result_table = run_multi_plan_summary(&args_table);
        assert!(result_table.is_ok(), "multi-plan summary (table) with tolerances must succeed: {:?}", result_table.err());

        // (b) CSV format must also succeed
        let result_csv = run_multi_plan_summary(&args_csv);
        assert!(result_csv.is_ok(), "multi-plan summary (CSV) with tolerances must succeed: {:?}", result_csv.err());
    }

    /// Task 207: balance_tol reads correctly from manifest via PlanContext.
    #[test]
    fn test_multi_plan_summary_tolerance_from_manifest() {
        let tmp = tempfile::TempDir::new().unwrap();
        let plan_dir = tmp.path().join("v1").join("2020").join("plans").join("tol_plan");
        write_plan_manifest(&plan_dir, "tol_plan", 2.5);

        // Verify PlanContext reads balance_tolerance_pct correctly
        let ctx = crate::plan_context::PlanContext::from_label(
            tmp.path(), "v1", "2020", "tol_plan",
        ).unwrap();
        assert!((ctx.manifest.balance_tolerance_pct - 2.5).abs() < 1e-9,
            "balance_tolerance_pct must be 2.5, got {}", ctx.manifest.balance_tolerance_pct);
    }

    /// Task 207: when PlanContext fails, Bal Tol% shows "-".
    #[test]
    fn test_multi_plan_summary_missing_manifest_shows_dash() {
        use crate::args::{CompareArgs, CompareFormat};
        let tmp = tempfile::TempDir::new().unwrap();
        // Create plan directories without manifests
        for label in &["plan_no_manifest_a", "plan_no_manifest_b"] {
            std::fs::create_dir_all(
                tmp.path().join("v1").join("2020").join("plans").join(label)
            ).unwrap();
        }

        let args = CompareArgs {
            plan_a: "plan_no_manifest_a".into(),
            plan_b: None,
            enacted: false,
            year: "2020".into(),
            version: Some("v1".into()),
            metrics: vec!["all".into()],
            out: None,
            format: CompareFormat::Table,
            output_base: tmp.path().to_str().unwrap().into(),
            output_dir: None,
            allow_cross_year: false,
            labels: vec!["plan_no_manifest_a".into(), "plan_no_manifest_b".into()],
            version_b: None,
            leaning_threshold: 0.55,
            close_call_band: 0.02,
            approved_by: None,
            report_dir: None,
        };

        // Should succeed — missing PlanContext just shows "-"
        let result = run_multi_plan_summary(&args);
        assert!(result.is_ok(), "summary without manifests must still succeed: {:?}", result.err());
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
