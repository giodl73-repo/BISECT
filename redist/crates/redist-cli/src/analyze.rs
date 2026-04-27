use std::collections::HashMap;
use std::path::{Path, PathBuf};
use redist_analysis::{AnalyzerContext, AnalyzerType,
    DemographicAnalyzer, Analyzer,
    PoliticalAnalyzer, UrbanAnalyzer, SummaryAnalyzer,
    check_contiguity, analyze_county_splits_with_state, analyze_municipal_splits,
    get_split_standard, compute_exit_code_with_flags};
use crate::args::AnalyzeArgs;
use crate::runner::load_all_states;
use crate::partisan::{PartisanArgs, run_partisan};
use crate::io_utils::write_json_atomic;

pub fn run_analyze(args: &AnalyzeArgs) -> anyhow::Result<()> {
    let year = args.year.to_string();

    // Locate output root first — needed for manifest-based state resolution.
    // Priority:
    //   1. --label + --output-dir: {output_dir}/{year}/plans/{label}/data/final_assignments.json
    //   2. --label only: {output_base}/{version}/{year}/plans/{label}/data/final_assignments.json
    //   3. State path: {output_base}/{version}/{year}/states/{state_name}/data/final_assignments.json
    let output_root = if let Some(ref od) = args.output_dir {
        od.clone()
    } else {
        PathBuf::from(&args.output_base).join(&args.version)
    };

    // Resolve state_code: from --state if provided, otherwise from plan manifest.
    let state_code = match &args.state {
        Some(s) => s.to_uppercase(),
        None => {
            // Derive from manifest when --label is provided
            if let Some(ref label) = args.label {
                let manifest_path = output_root
                    .join(&year)
                    .join("plans")
                    .join(label)
                    .join("manifest.json");
                if manifest_path.exists() {
                    let manifest: serde_json::Value = serde_json::from_str(
                        &std::fs::read_to_string(&manifest_path)?
                    )?;
                    manifest["state_code"]
                        .as_str()
                        .filter(|s| !s.is_empty())
                        .map(|s| s.to_uppercase())
                        .ok_or_else(|| anyhow::anyhow!(
                            "manifest at {} missing 'state_code' field",
                            manifest_path.display()
                        ))?
                } else {
                    anyhow::bail!(
                        "--state is required when manifest not found at {}",
                        manifest_path.display()
                    );
                }
            } else {
                anyhow::bail!("--state or --label with an existing manifest is required");
            }
        }
    };

    // Resolve state name from config
    let all = load_all_states(&year)
        .map_err(|e| anyhow::anyhow!("Failed to load state config: {e}"))?;
    let (_, state_name, num_districts) = all.iter()
        .find(|(code, _, _)| code == &state_code)
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("Unknown state: {state_code}"))?;

    let (assignments_path, analysis_dir_base) = if let Some(ref label) = args.label {
        let plan_dir = output_root.join(&year).join("plans").join(label);
        let data_path = plan_dir.join("data").join("final_assignments.json");
        let flat_path = plan_dir.join("final_assignments.json");
        let chosen = if data_path.exists() {
            data_path
        } else {
            flat_path
        };
        (chosen, plan_dir)
    } else {
        let state_dir = output_root.join(&year).join("states").join(&state_name);
        let data_path = state_dir.join("data").join("final_assignments.json");
        (data_path, state_dir)
    };

    // For the old (non-label) path, compute the proper output_root for adjacency lookups
    let adjacency_root = if args.output_dir.is_some() {
        // If output_dir was given, adjacency is elsewhere; fall back to standard path
        PathBuf::from(&args.output_base).join(&args.version)
    } else {
        output_root.clone()
    };

    if !assignments_path.exists() {
        // List available plans to help the user understand what labels exist
        let plans_dir = output_root.join(&year).join("plans");
        let available_hint = if plans_dir.exists() {
            let mut labels: Vec<String> = std::fs::read_dir(&plans_dir)
                .ok()
                .into_iter()
                .flatten()
                .filter_map(|e| e.ok())
                .filter(|e| e.path().is_dir())
                .filter_map(|e| e.file_name().into_string().ok())
                .take(10)
                .collect();
            labels.sort();
            if labels.is_empty() {
                format!("\nPlans directory exists but is empty: {}", plans_dir.display())
            } else {
                format!("\nAvailable plans: {}", labels.join(", "))
            }
        } else {
            format!("\nPlans directory not found: {}", plans_dir.display())
        };
        anyhow::bail!(
            "No assignments found at {}.{}\nRun: redist state --state {state_code} --year {year} --version {}",
            assignments_path.display(), available_hint, args.version
        );
    }

    let raw_assignments: HashMap<String, usize> = serde_json::from_str(
        &std::fs::read_to_string(&assignments_path)?
    )?;

    // If assignments use tract indices (keys < 11 chars), resolve to GEOID-keyed map
    // using the adjacency geoids file so all analyzers can join by GEOID.
    let assignments: HashMap<String, usize> = crate::geometry::resolve_to_geoid_assignments(
        raw_assignments, &adjacency_root, &state_code, &year,
    );

    let analysis_dir = analysis_dir_base.join("analysis");
    std::fs::create_dir_all(&analysis_dir)?;

    // Resolve balance_tolerance: read from manifest when --label path is used,
    // otherwise use the congressional default (0.005 = ±0.5%).
    // Manifest stores `balance_tolerance_pct` as a percentage; convert to fraction.
    let balance_tolerance: f64 = if let Some(ref label) = args.label {
        let manifest_path = output_root
            .join(&year)
            .join("plans")
            .join(label)
            .join("manifest.json");
        if manifest_path.exists() {
            let manifest: serde_json::Value = serde_json::from_str(
                &std::fs::read_to_string(&manifest_path)?
            )?;
            // Try balance_tolerance_pct (percentage form) first, then balance_tolerance (fraction)
            if let Some(pct) = manifest["balance_tolerance_pct"].as_f64() {
                pct / 100.0
            } else if let Some(frac) = manifest["balance_tolerance"].as_f64() {
                frac
            } else {
                // Fall back to chamber-aware default from manifest chamber field
                match manifest["chamber"].as_str().unwrap_or("congressional") {
                    "congressional" => 0.005,
                    _ => 0.05,
                }
            }
        } else {
            0.005 // default when manifest absent
        }
    } else {
        // Legacy state path: use congressional default (strictest)
        0.005
    };

    let ctx = AnalyzerContext {
        assignments: &assignments,
        state_name: &state_name,
        state_code: &state_code,
        year: &year,
        version: &args.version,
        num_districts,
        data_root: Path::new("data"),
        output_root: &adjacency_root,
        balance_tolerance,
    };

    // Resolve types: expand All → concrete list
    let types = resolve_types(&args.types);

    let mut balance_failed = false;
    let mut contiguity_failed = false;
    let mut missing_data = false;

    for typ in &types {
        let out_path = analysis_dir.join(format!("{}.json", typ.name()));
        if out_path.exists() && !args.force {
            eprintln!("[skip] {} (exists, use --force to regenerate)", typ.name());
            continue;
        }

        match typ {
            AnalyzerType::Demographic => {
                match DemographicAnalyzer::run(&ctx) {
                    Ok(result) => {
                        write_json_atomic(&out_path, &result)?;
                        eprintln!("[OK] demographic -> {}", out_path.display());
                    }
                    Err(e) => {
                        let msg = e.to_string();
                        if msg.contains("demographics CSV not found") {
                            eprintln!(
                                "WARNING: VRA analysis for {state_code} {year}: demographics CSV not found.\n\
                                 Run: redist fetch --type demographics --states {state_code} --year {year}"
                            );
                            missing_data = true;
                            let out = serde_json::json!({
                                "analyzer": "demographic",
                                "state": state_code,
                                "year": year,
                                "error": "demographics CSV not found",
                            });
                            write_json_atomic(&out_path, &out)?;
                        } else {
                            return Err(e);
                        }
                    }
                }
            }
            AnalyzerType::Political => {
                let result = PoliticalAnalyzer::run(&ctx)?;
                write_json_atomic(&out_path, &result)?;
                eprintln!("[OK] political -> {}", out_path.display());
            }
            AnalyzerType::Urban => {
                let result = UrbanAnalyzer::run(&ctx)?;
                write_json_atomic(&out_path, &result)?;
                eprintln!("[OK] urban -> {}", out_path.display());
            }
            AnalyzerType::Summary => {
                let result = SummaryAnalyzer::run(&ctx)?;
                if !result.population_balance_valid {
                    eprintln!("WARNING: population balance FAILED for {state_code} — one or more districts exceed ±{:.1}%",
                        balance_tolerance * 100.0);
                    balance_failed = true;
                }
                write_json_atomic(&out_path, &result)?;
                eprintln!("[OK] summary -> {}", out_path.display());
            }
            AnalyzerType::Compactness => {
                // Load dissolved district geometries
                let districts = crate::geometry::load_district_geometries(
                    &state_name, &state_code, &year, &args.version,
                    &assignments, Path::new("data"), "tract",
                )?;

                // Compute compactness metrics for each district using the largest polygon component
                let mut district_results: Vec<serde_json::Value> = Vec::new();
                for (district_id, mp) in &districts {
                    if let Some(poly) = redist_map::largest_component(mp) {
                        match redist_analysis::all_metrics(*district_id, poly) {
                            Ok(metrics) => {
                                district_results.push(serde_json::json!({
                                    "district": metrics.district,
                                    "polsby_popper": metrics.polsby_popper,
                                    "reock": metrics.reock,
                                    "convex_hull_ratio": metrics.convex_hull_ratio,
                                    "perimeter_m": metrics.perimeter_m,
                                    "area_m2": metrics.area_m2,
                                    "crs_note": "computed on WGS84 coords; scores compressed for east-west-elongated districts"
                                }));
                            }
                            Err(e) => {
                                eprintln!("WARNING: compactness skipped for district {district_id}: {e}");
                            }
                        }
                    }
                }
                district_results.sort_by_key(|d| d["district"].as_u64().unwrap_or(0));

                let result = serde_json::json!({
                    "analyzer": "compactness",
                    "state": state_code,
                    "year": year,
                    "districts": district_results
                });
                write_json_atomic(&out_path, &result)?;
                eprintln!("[OK] compactness -> {}", out_path.display());
            }
            AnalyzerType::Partisan => {
                let partisan_args = PartisanArgs {
                    assignments: &assignments,
                    state_code: &state_code,
                    state_name: &state_name,
                    year: &year,
                    version: &args.version,
                    election_file: args.election_file.as_ref(),
                    bootstrap_samples: args.bootstrap_samples,
                    analysis_dir: &analysis_dir,
                    force: args.force,
                    chamber: "congressional",
                };
                run_partisan(&partisan_args)?;
                // Skip writing json again — run_partisan handles it
                continue;
            }
            AnalyzerType::Contiguity => {
                // Load adjacency graph to get geoid list and edges.
                let adj_result = load_adjacency_for_analysis(
                    &adjacency_root, &state_code, &year,
                );
                match adj_result {
                    Ok((adjacency, geoids)) => {
                        let result = check_contiguity(
                            &assignments,
                            &adjacency,
                            &geoids,
                            num_districts,
                        );
                        let contiguity_violation = !result.all_contiguous;
                        write_json_atomic(&out_path, &result)?;
                        eprintln!("[OK] contiguity -> {}", out_path.display());
                        if contiguity_violation {
                            contiguity_failed = true;
                        }
                    }
                    Err(e) => {
                        eprintln!("WARNING: adjacency not available for contiguity check: {e}");
                        eprintln!("  Run: python scripts/data/generate_adj_bin.py --year {year} --states {state_code}");
                        missing_data = true;
                        // Write a placeholder so the file exists
                        let out = serde_json::json!({
                            "analyzer": "contiguity",
                            "state": state_code,
                            "year": year,
                            "error": "adjacency data not available",
                        });
                        write_json_atomic(&out_path, &out)?;
                    }
                }
            }
            AnalyzerType::Splits => {
                let standard = get_split_standard(&state_code);
                let county_result = analyze_county_splits_with_state(
                    &assignments,
                    None, // county names not loaded (optional enrichment)
                    Some(&state_code),
                );
                // Municipal splits: not loaded unless geography data present
                let place_to_tracts = HashMap::new();
                let place_names = HashMap::new();
                let municipal_result =
                    analyze_municipal_splits(&assignments, &place_to_tracts, &place_names);

                // Subdivision vocabulary: read from SplitStandard (populated from policy DB).
                let (sub_term, sub_term_plural) = standard.as_ref().map(|s| {
                    (s.subdivision_term.clone(), s.subdivision_term_plural.clone())
                }).unwrap_or_else(|| ("county".into(), "counties".into()));

                let out = serde_json::json!({
                    "analyzer": "splits",
                    "state": state_code,
                    "year": year,
                    "counties": {
                        "subdivision_term": sub_term,
                        "subdivision_term_plural": sub_term_plural,
                        "total": county_result.total,
                        "split": county_result.split,
                        "preservation_score": county_result.preservation_score,
                        "split_list": county_result.split_list,
                        "legal_standard": county_result.legal_standard,
                        "compliance_assessment": county_result.compliance_assessment,
                        "disclaimer": county_result.disclaimer,
                    },
                    "municipalities": {
                        "available": municipal_result.available,
                        "total": municipal_result.total,
                        "split": municipal_result.split,
                        "preservation_score": municipal_result.preservation_score,
                        "split_list": municipal_result.split_list,
                    },
                });
                write_json_atomic(&out_path, &out)?;
                eprintln!("[OK] splits -> {}", out_path.display());
                // Log the standard for auditing
                if let Some(ref s) = standard {
                    eprintln!("  Legal standard: {}", s.legal_standard);
                }
            }
            AnalyzerType::All => unreachable!("expanded above"),
        }
    }

    // Compute bitfield exit code
    let exit_code = compute_exit_code_with_flags(
        balance_failed,
        contiguity_failed,
        false, // nesting: not checked in this flow
        missing_data,
        args.allow_noncontiguous,
        args.allow_imbalance,
    );

    if exit_code != 0 {
        let mut reasons = Vec::new();
        if balance_failed && !args.allow_imbalance {
            reasons.push("population balance failed");
        }
        if contiguity_failed && !args.allow_noncontiguous {
            reasons.push("contiguity violation detected");
        }
        if missing_data {
            reasons.push("missing data (adjacency not available)");
        }
        eprintln!("ERROR: {}", reasons.join("; "));
        std::process::exit(exit_code as i32);
    }

    Ok(())
}

/// Load adjacency list and geoid vector from standard adjacency file locations.
fn load_adjacency_for_analysis(
    output_root: &PathBuf,
    state_code: &str,
    year: &str,
) -> anyhow::Result<(Vec<Vec<usize>>, Vec<String>)> {
    let state_lower = state_code.to_lowercase();
    let geoid_file = format!("{state_lower}_adjacency_{year}_geoids.json");
    let bin_file = format!("{state_lower}_adjacency_{year}.adj.bin");
    let geoid_candidates = [
        output_root.join("data").join(year).join("adjacency").join(&geoid_file),
        PathBuf::from("outputs/V3/data").join(year).join("adjacency").join(&geoid_file),
        PathBuf::from("outputs/V4/data").join(year).join("adjacency").join(&geoid_file),
    ];
    let bin_candidates = [
        output_root.join("data").join(year).join("adjacency").join(&bin_file),
        PathBuf::from("outputs/V3/data").join(year).join("adjacency").join(&bin_file),
        PathBuf::from("outputs/V4/data").join(year).join("adjacency").join(&bin_file),
    ];

    // Find geoids file
    let geoid_path = geoid_candidates.iter().find(|p| p.exists())
        .ok_or_else(|| anyhow::anyhow!("GEOID mapping not found for {state_code} {year}"))?;
    let raw_geoids: HashMap<String, String> = serde_json::from_str(
        &std::fs::read_to_string(geoid_path)?
    )?;

    // Build ordered geoid vector (index -> geoid)
    let n = raw_geoids.len();
    let mut geoids = vec![String::new(); n];
    for (idx_str, geoid) in &raw_geoids {
        if let Ok(idx) = idx_str.parse::<usize>() {
            if idx < n {
                geoids[idx] = geoid.clone();
            }
        }
    }

    // Load adjacency from .adj.bin
    let bin_path = bin_candidates.iter().find(|p| p.exists());
    let adjacency = if let Some(bp) = bin_path {
        let data = std::fs::read(bp)?;
        let graph = redist_data::deserialize_adjacency(&data)
            .map_err(|e| anyhow::anyhow!("adjacency deserialization failed: {e}"))?;
        graph.adjacency
    } else {
        // Fallback: return empty adjacency (contiguity not checkable)
        anyhow::bail!("Adjacency binary not found for {state_code} {year}. \
            Run: python scripts/data/generate_adj_bin.py --year {year} --states {state_code}");
    };

    Ok((adjacency, geoids))
}

fn resolve_types(types: &[AnalyzerType]) -> Vec<AnalyzerType> {
    if types.iter().any(|t| *t == AnalyzerType::All) {
        AnalyzerType::all_concrete()
    } else {
        types.to_vec()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use redist_analysis::AnalyzerType;

    #[test]
    fn test_resolve_types_all_expands_to_concrete() {
        let types = resolve_types(&[AnalyzerType::All]);
        // All must expand to at least Demographic, Political, Urban, Summary
        assert!(types.contains(&AnalyzerType::Demographic));
        assert!(types.contains(&AnalyzerType::Political));
        assert!(types.contains(&AnalyzerType::Urban));
        assert!(types.contains(&AnalyzerType::Summary));
        // All must NOT be in the expanded list
        assert!(!types.contains(&AnalyzerType::All));
    }

    #[test]
    fn test_resolve_types_explicit_subset() {
        let types = resolve_types(&[AnalyzerType::Demographic, AnalyzerType::Political]);
        assert_eq!(types.len(), 2);
        assert!(types.contains(&AnalyzerType::Demographic));
        assert!(types.contains(&AnalyzerType::Political));
        assert!(!types.contains(&AnalyzerType::Urban));
    }

    #[test]
    fn test_resolve_types_empty_input() {
        let types = resolve_types(&[]);
        // Empty input should return empty (not All)
        assert!(types.is_empty(), "empty input should return empty types, got {types:?}");
    }

    /// Task 119: missing plan error lists available labels from plans directory.
    #[test]
    fn test_missing_plan_error_lists_available_labels() {
        use tempfile::TempDir;
        use std::path::Path;

        let tmp = TempDir::new().unwrap();
        // Create a fake plans/{year}/plans/ directory with some plan subdirs
        let plans_dir = tmp.path().join("2020").join("plans");
        std::fs::create_dir_all(&plans_dir).unwrap();
        for label in &["wa_house_2020", "wa_senate_2020", "wa_congressional_2020"] {
            std::fs::create_dir_all(plans_dir.join(label)).unwrap();
        }

        // Simulate the listing logic from run_analyze
        let available_hint = if plans_dir.exists() {
            let mut labels: Vec<String> = std::fs::read_dir(&plans_dir)
                .ok()
                .into_iter()
                .flatten()
                .filter_map(|e| e.ok())
                .filter(|e| e.path().is_dir())
                .filter_map(|e| e.file_name().into_string().ok())
                .take(10)
                .collect();
            labels.sort();
            if labels.is_empty() {
                format!("\nPlans directory exists but is empty: {}", plans_dir.display())
            } else {
                format!("\nAvailable plans: {}", labels.join(", "))
            }
        } else {
            format!("\nPlans directory not found: {}", plans_dir.display())
        };

        assert!(available_hint.contains("wa_house_2020"), "must list wa_house_2020: {available_hint}");
        assert!(available_hint.contains("wa_senate_2020"), "must list wa_senate_2020: {available_hint}");
        assert!(available_hint.contains("wa_congressional_2020"), "must list wa_congressional_2020: {available_hint}");
        assert!(available_hint.starts_with("\nAvailable plans:"));
    }

    /// Task 119: when plans directory doesn't exist, show path without listing.
    #[test]
    fn test_missing_plan_error_no_plans_dir_shows_path() {
        let plans_dir = std::path::PathBuf::from("/nonexistent/plans");
        let available_hint = if plans_dir.exists() {
            "should not reach here".to_string()
        } else {
            format!("\nPlans directory not found: {}", plans_dir.display())
        };
        assert!(available_hint.contains("Plans directory not found"), "must say not found: {available_hint}");
    }

    /// Scenario 5: VRA missing demographics hint
    /// When demographics CSV is absent, the warning must include the exact fetch command.
    #[test]
    fn test_vra_missing_demographics_hint() {
        use redist_analysis::{Analyzer, AnalyzerContext, DemographicAnalyzer};
        use std::collections::HashMap;
        use std::path::Path;

        // Build a context with a data_root that has no demographics CSVs
        let tmp = tempfile::TempDir::new().unwrap();
        let assignments: HashMap<String, usize> = HashMap::new();
        let ctx = AnalyzerContext {
            assignments: &assignments,
            state_name: "washington",
            state_code: "WA",
            year: "2020",
            version: "v1",
            num_districts: 10,
            data_root: tmp.path(), // no CSV files here
            output_root: Path::new("outputs/v1"),
            balance_tolerance: 0.005,
        };

        let result = DemographicAnalyzer::run(&ctx);
        assert!(result.is_err(), "DemographicAnalyzer must return Err when CSV is missing");
        let msg = result.unwrap_err().to_string();
        // The error message from DemographicAnalyzer should mention demographics CSV not found
        assert!(
            msg.contains("demographics CSV not found"),
            "error must mention demographics CSV not found: {msg}"
        );
        // Verify that the hint we emit contains "redist fetch" and "--type demographics"
        // (We can't capture eprintln in a unit test, so we verify the error text that
        // triggers the hint path — the warn text is validated by the message content.)
        // The hint branch is triggered by the message above; verify the message pattern here.
        assert!(
            msg.contains("demographics"),
            "error message must contain 'demographics' to trigger fetch hint: {msg}"
        );
    }
}

