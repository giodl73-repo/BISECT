//! `redist ensemble` — weighted plan ensemble generation.
//!
//! Currently supports: --method smc (Sequential Monte Carlo, G.7 spec accepted 3.1/4).
//! Output: NDJSON with importance-weighted plans + audit metadata.

use std::fs;
use std::io::BufWriter;
use std::path::PathBuf;

use anyhow::Context;

use crate::args::{EnsembleArgs, EnsembleMethod};
use crate::adjacency_loader::load_adjacency_pkl;
use crate::fetch::load_manifest;
use crate::runner::load_all_states;

/// Run the `redist ensemble` command.
pub fn run_ensemble(args: &EnsembleArgs) -> anyhow::Result<()> {
    match args.method {
        EnsembleMethod::Smc => run_smc_ensemble(args),
    }
}

/// Resolve the adjacency file path using the same logic as runner.rs.
fn resolve_adj_path(state_lower: &str, year: &str) -> anyhow::Result<PathBuf> {
    let manifest = load_manifest()
        .map_err(|e| anyhow::anyhow!("cannot load manifest: {e}"))?;
    let outputs_dir = PathBuf::from(&manifest.local_outputs_dir);
    let filename = format!("{state_lower}_adjacency_{year}.pkl");

    for version in ["V3", "V4"] {
        let path = outputs_dir.join(version).join("data").join(year).join("adjacency").join(&filename);
        if path.exists() {
            return Ok(path);
        }
    }
    anyhow::bail!("adjacency file not found for {state_lower} {year}. Run: redist fetch --year {year}")
}

fn run_smc_ensemble(args: &EnsembleArgs) -> anyhow::Result<()> {
    let state_lower = args.state.to_lowercase();
    let state_upper = args.state.to_uppercase();
    let year = args.year.to_string();

    // ── Resolve district count k ─────────────────────────────────────────────
    let k = {
        let all = load_all_states(&year)
            .map_err(|e| anyhow::anyhow!("cannot load state registry for {year}: {e}"))?;
        all.iter()
            .find(|(code, _, _)| code == &state_upper)
            .map(|(_, _, n)| *n)
            .ok_or_else(|| anyhow::anyhow!(
                "no district count for '{}'. Use redist fetch --year {}",
                state_upper, year
            ))?
    };

    // ── Load adjacency graph ─────────────────────────────────────────────────
    let adj_path = resolve_adj_path(&state_lower, &year)?;
    eprintln!("[redist ensemble smc] {} {} — loading {} adjacency tracts",
        state_upper, year, adj_path.file_name().unwrap_or_default().to_string_lossy());

    let graph = load_adjacency_pkl(&adj_path)
        .map_err(|e| anyhow::anyhow!("failed to load adjacency {}: {e}", adj_path.display()))?;

    let adjacency = &graph.adjacency;
    let pop: &Vec<i64> = &graph.vertex_weights;
    let n = adjacency.len();

    eprintln!("[redist ensemble smc] n={n} tracts, k={k} districts");
    eprintln!("[redist ensemble smc] particles={}, pop_tolerance={:.3}, resample_threshold={:.2}",
        args.particles, args.pop_tolerance, args.resample_threshold);

    // ── Content-derived base seed (mirrors B.16 §3.1 formula) ───────────────
    let base_seed = args.base_seed.unwrap_or_else(|| {
        use sha2::{Digest, Sha256};
        let mut h = Sha256::new();
        h.update(state_lower.as_bytes());
        h.update(b"_");
        h.update(year.as_bytes());
        h.update(b"SMC_SEED_V1");
        let d = h.finalize();
        u64::from_le_bytes(d[..8].try_into().unwrap())
    });
    eprintln!("[redist ensemble smc] base_seed={base_seed}");

    // ── Run SMC ──────────────────────────────────────────────────────────────
    let cfg = redist_smc::SmcConfig {
        n_particles: args.particles,
        resample_threshold: args.resample_threshold,
        pop_tolerance: args.pop_tolerance,
        base_seed,
    };

    eprintln!("[redist ensemble smc] Running SMC... (may take several minutes for N={}).",
        args.particles);
    let result = redist_smc::run_smc(adjacency, pop, k, cfg)
        .map_err(|e| anyhow::anyhow!("SMC run failed: {e}"))?;

    eprintln!("[redist ensemble smc] Complete: {} plans, {} resamplings",
        result.n_plans(), result.resample_count);
    if !result.ess_trace.is_empty() {
        let min_ess = result.ess_trace.iter().cloned().fold(f64::INFINITY, f64::min);
        let min_stage = result.ess_trace.iter()
            .position(|&e| e == min_ess).map(|i| i + 1).unwrap_or(0);
        eprintln!("[redist ensemble smc] Min ESS={min_ess:.0} at stage {min_stage}/{}",
            result.ess_trace.len());
    }

    // ── Write NDJSON output ──────────────────────────────────────────────────
    let output_path = args.output.clone()
        .unwrap_or_else(|| format!("{state_lower}_smc_{year}.ndjson"));

    let file = fs::File::create(&output_path)
        .with_context(|| format!("cannot create output file: {output_path}"))?;
    let mut writer = BufWriter::new(file);

    let write_cfg = redist_smc::WriteConfig {
        resample_threshold: args.resample_threshold,
        pop_tolerance: args.pop_tolerance,
    };
    result.write_ndjson(&mut writer, &write_cfg)
        .with_context(|| "failed to write NDJSON output")?;

    eprintln!("[redist ensemble smc] Written: {output_path}");
    eprintln!("[redist ensemble smc] Verify: check 'file_sha256' in the final metadata line.");
    Ok(())
}
