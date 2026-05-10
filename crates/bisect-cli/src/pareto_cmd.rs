//! `bisect pareto` — Pareto-optimal redistricting ensemble via NSGA-II (U.7).
//!
//! Generates the Pareto frontier over three objectives:
//!   - Compactness (edge-cut minimisation)
//!   - Partisan fairness (seat deviation from proportionality)
//!   - VRA compliance (minority-district deficit)
//!
//! Outputs NDJSON with one JSON object per Pareto-optimal plan.
//! Spec: docs/specs/2026-05-07-pareto-redistricting.md (Accepted, R2 avg 3.75/4)

use std::fs;
use std::io::BufWriter;
use std::path::PathBuf;

use anyhow::Context;

use crate::adjacency_loader::load_adjacency_pkl;
use crate::args::ParetoArgs;
use crate::fetch::load_manifest;
use crate::runner::load_all_states;

/// Run the `bisect pareto` command.
pub fn run_pareto(args: &ParetoArgs) -> anyhow::Result<()> {
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
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "no district count for '{}'. Use bisect fetch --year {}",
                    state_upper,
                    year
                )
            })?
    };

    // ── Load adjacency graph ─────────────────────────────────────────────────
    let adj_path = resolve_adj_path(&state_lower, &year)?;
    eprintln!(
        "[bisect pareto] {} {} — loading {} adjacency tracts",
        state_upper,
        year,
        adj_path.file_name().unwrap_or_default().to_string_lossy()
    );

    let graph = load_adjacency_pkl(&adj_path)
        .map_err(|e| anyhow::anyhow!("failed to load adjacency {}: {e}", adj_path.display()))?;

    let n = graph.adjacency.len();
    eprintln!(
        "[bisect pareto] n={n} tracts, k={k} districts, population={}, generations={}",
        args.population, args.generations
    );

    // ── Derive base seed ─────────────────────────────────────────────────────
    let base_seed = args.base_seed.unwrap_or_else(|| {
        use sha2::{Digest, Sha256};
        let mut h = Sha256::new();
        h.update(state_lower.as_bytes());
        h.update(b"_pareto_");
        h.update(year.as_bytes());
        let digest = h.finalize();
        u64::from_le_bytes(digest[..8].try_into().unwrap_or([0u8; 8]))
    });

    // ── Run NSGA-II ──────────────────────────────────────────────────────────
    let config = bisect_pareto::ParetoConfig {
        n_population: args.population,
        n_generations: args.generations,
        base_seed,
        balance_tolerance: 0.005,
    };

    eprintln!("[bisect pareto] running NSGA-II (seed={base_seed})");
    let result = bisect_pareto::run_nsga2(
        &graph.adjacency,
        &graph.vertex_weights,
        k,
        None, // d_votes: not loaded in Phase 1
        None, // minority_vap: not loaded in Phase 1
        &[],
        config,
    )
    .map_err(|e| anyhow::anyhow!("NSGA-II failed: {e}"))?;

    eprintln!(
        "[bisect pareto] frontier size: {} Pareto-optimal plans in {:.1}s",
        result.frontier.len(),
        result.runtime_secs
    );

    // ── Write NDJSON output ──────────────────────────────────────────────────
    let output_path = args
        .output
        .clone()
        .unwrap_or_else(|| format!("{}_pareto_{}.ndjson", state_lower, year));
    let path = PathBuf::from(&output_path);
    let file = fs::File::create(&path)
        .with_context(|| format!("cannot create output file: {}", path.display()))?;
    let mut writer = BufWriter::new(file);

    result
        .write_ndjson(&mut writer)
        .with_context(|| format!("failed to write NDJSON to {}", path.display()))?;

    eprintln!(
        "[bisect pareto] wrote {} plans to {}",
        result.frontier.len(),
        path.display()
    );
    Ok(())
}

/// Resolve the adjacency .pkl file path (same logic as ensemble.rs).
fn resolve_adj_path(state_lower: &str, year: &str) -> anyhow::Result<PathBuf> {
    let manifest = load_manifest().map_err(|e| anyhow::anyhow!("cannot load manifest: {e}"))?;
    let outputs_dir = PathBuf::from(&manifest.local_outputs_dir);
    let filename = format!("{state_lower}_adjacency_{year}.pkl");

    for version in ["V3", "V4"] {
        let path = outputs_dir
            .join(version)
            .join("data")
            .join(year)
            .join("adjacency")
            .join(&filename);
        if path.exists() {
            return Ok(path);
        }
    }
    anyhow::bail!(
        "adjacency file not found for {state_lower} {year}. Run: bisect fetch --year {year}"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::args::{Commands, ParetoArgs};

    /// L0: verify Commands::Pareto variant compiles and wraps ParetoArgs.
    #[test]
    fn pareto_command_exists() {
        // Construct a ParetoArgs and wrap it in Commands::Pareto.
        // If Commands::Pareto doesn't exist or ParetoArgs fields change,
        // this test fails at compile time.
        let pa = ParetoArgs {
            state: "NC".to_string(),
            year: crate::args::Year::Y2020,
            population: 100,
            generations: 200,
            base_seed: Some(42),
            output: None,
        };
        let cmd = Commands::Pareto(pa);
        // Verify it matches as expected.
        assert!(
            matches!(cmd, Commands::Pareto(_)),
            "Commands::Pareto must match the Pareto variant"
        );
    }
}
