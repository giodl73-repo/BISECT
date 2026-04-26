use std::collections::HashMap;
use std::path::{Path, PathBuf};
use redist_analysis::{AnalyzerContext, AnalyzerType,
    DemographicAnalyzer, Analyzer,
    PoliticalAnalyzer, UrbanAnalyzer, SummaryAnalyzer};
use crate::args::AnalyzeArgs;
use crate::runner::load_all_states;

pub fn run_analyze(args: &AnalyzeArgs) -> anyhow::Result<()> {
    let state_code = args.state.to_uppercase();
    let year = args.year.to_string();

    // Resolve state name from config
    let all = load_all_states(&year)
        .map_err(|e| anyhow::anyhow!("Failed to load state config: {e}"))?;
    let (_, state_name, num_districts) = all.iter()
        .find(|(code, _, _)| code == &state_code)
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("Unknown state: {state_code}"))?;

    // Locate assignment file — path mirrors runner.rs: {output}/{version}/states/{state}/data/
    let output_root = PathBuf::from(&args.output_base).join(&args.version);
    let state_data_dir = output_root.join("states").join(&state_name).join("data");
    let assignments_path = state_data_dir.join("final_assignments.json");

    if !assignments_path.exists() {
        anyhow::bail!(
            "No assignments found at {}.\nRun: redist state --state {state_code} --year {year} --version {}",
            assignments_path.display(), args.version
        );
    }

    let raw_assignments: HashMap<String, usize> = serde_json::from_str(
        &std::fs::read_to_string(&assignments_path)?
    )?;

    // If assignments use tract indices (keys < 11 chars), resolve to GEOID-keyed map
    // using the adjacency geoids file so all analyzers can join by GEOID.
    let assignments: HashMap<String, usize> = {
        let uses_index = raw_assignments.keys().next().map(|k| k.len() < 11).unwrap_or(false);
        if uses_index {
            let state_code_lower = state_code.to_lowercase();
            let geoid_file = format!("{state_code_lower}_adjacency_{year}_geoids.json");
            let geoid_path = output_root.join("data").join(&year).join("adjacency").join(&geoid_file);
            if geoid_path.exists() {
                let raw_geoids: HashMap<String, String> = serde_json::from_str(
                    &std::fs::read_to_string(&geoid_path)?
                )?;
                // raw_geoids: {"0": "50005957100", ...}
                // Build GEOID -> district from index -> district
                raw_geoids.into_iter()
                    .filter_map(|(idx, geoid)| raw_assignments.get(&idx).map(|&d| (geoid, d)))
                    .collect()
            } else {
                eprintln!("WARNING: geoid mapping not found at {}; analyzers may show unmatched tracts",
                    geoid_path.display());
                raw_assignments
            }
        } else {
            raw_assignments
        }
    };

    let analysis_dir = state_data_dir.parent()
        .unwrap_or(&state_data_dir)
        .join("analysis");
    std::fs::create_dir_all(&analysis_dir)?;

    let ctx = AnalyzerContext {
        assignments: &assignments,
        state_name: &state_name,
        state_code: &state_code,
        year: &year,
        version: &args.version,
        num_districts,
        data_root: Path::new("data"),
        output_root: &output_root,
    };

    // Resolve types: expand All → concrete list
    let types = resolve_types(&args.types);

    let mut balance_failed = false;

    for typ in &types {
        let out_path = analysis_dir.join(format!("{}.json", typ.name()));
        if out_path.exists() && !args.force {
            eprintln!("[skip] {} (exists, use --force to regenerate)", typ.name());
            continue;
        }

        match typ {
            AnalyzerType::Demographic => {
                let result = DemographicAnalyzer::run(&ctx)?;
                write_json_atomic(&out_path, &result)?;
                eprintln!("[OK] demographic -> {}", out_path.display());
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
                    eprintln!("WARNING: population balance FAILED for {state_code} — one or more districts exceed ±0.5%");
                    balance_failed = true;
                }
                write_json_atomic(&out_path, &result)?;
                eprintln!("[OK] summary -> {}", out_path.display());
            }
            AnalyzerType::Compactness => {
                eprintln!("[skip] compactness — requires dissolved district geometries (not yet wired)");
            }
            AnalyzerType::All => unreachable!("expanded above"),
        }
    }

    if balance_failed && !args.allow_imbalance {
        eprintln!("ERROR: population balance failed. Use --allow-imbalance to suppress this exit code.");
        std::process::exit(2);
    }

    Ok(())
}

fn resolve_types(types: &[AnalyzerType]) -> Vec<AnalyzerType> {
    if types.iter().any(|t| *t == AnalyzerType::All) {
        AnalyzerType::all_concrete()
    } else {
        types.to_vec()
    }
}

fn write_json_atomic<T: serde::Serialize>(path: &Path, value: &T) -> anyhow::Result<()> {
    let tmp = path.with_extension("tmp.json");
    std::fs::write(&tmp, serde_json::to_string_pretty(value)?)?;
    std::fs::rename(&tmp, path)?;
    Ok(())
}
