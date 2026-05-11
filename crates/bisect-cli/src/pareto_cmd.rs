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

    if let Some(selected_index) = args.selected_frontier_index {
        write_selected_frontier_package(args, &result, selected_index, k, &state_lower, &year)?;
    }
    Ok(())
}

fn write_selected_frontier_package(
    args: &ParetoArgs,
    result: &bisect_pareto::ParetoResult,
    selected_index: usize,
    k: usize,
    state_lower: &str,
    year: &str,
) -> anyhow::Result<()> {
    let context_path = args.selected_frontier_context.as_ref().ok_or_else(|| {
        anyhow::anyhow!("--selected-frontier-context is required with --selected-frontier-index")
    })?;
    let context_text = std::fs::read_to_string(context_path)
        .with_context(|| format!("read selected frontier RCTX {}", context_path.display()))?;
    let context = rplan_io::read_rctx_str(&context_text)
        .with_context(|| format!("parse selected frontier RCTX {}", context_path.display()))?;
    let out_dir = args.selected_frontier_out.clone().unwrap_or_else(|| {
        PathBuf::from(format!(
            "{}_pareto_{}_selected_{selected_index}",
            state_lower, year
        ))
    });
    let label = args
        .selected_frontier_label
        .clone()
        .unwrap_or_else(|| format!("{}_pareto_{}_selected_{selected_index}", state_lower, year));
    let generated_at = bisect_report::now_iso8601();
    let package = bisect_pareto::write_selected_frontier_package(
        result,
        selected_index,
        &context,
        k,
        &out_dir,
        &label,
        args.selected_frontier_tolerance,
        &generated_at,
    )
    .with_context(|| format!("write selected frontier package {}", out_dir.display()))?;
    eprintln!(
        "[bisect pareto] wrote selected frontier package {} ({})",
        out_dir.display(),
        package.audit_certificate_path
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
    use clap::Parser;
    use std::collections::BTreeMap;

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
            selected_frontier_index: None,
            selected_frontier_context: None,
            selected_frontier_out: None,
            selected_frontier_label: None,
            selected_frontier_tolerance: 0.5,
        };
        let cmd = Commands::Pareto(pa);
        // Verify it matches as expected.
        assert!(
            matches!(cmd, Commands::Pareto(_)),
            "Commands::Pareto must match the Pareto variant"
        );
    }

    #[test]
    fn pareto_selected_frontier_args_parse() {
        let cli = crate::args::Cli::parse_from([
            "bisect",
            "pareto",
            "--state",
            "TT",
            "--selected-frontier-index",
            "0",
            "--selected-frontier-context",
            "fixture.rctx",
            "--selected-frontier-out",
            "selected",
            "--selected-frontier-label",
            "chosen",
            "--selected-frontier-tolerance",
            "1.25",
        ]);
        let Commands::Pareto(args) = cli.command else {
            panic!("expected pareto command");
        };
        assert_eq!(args.selected_frontier_index, Some(0));
        assert_eq!(
            args.selected_frontier_context.as_deref(),
            Some(std::path::Path::new("fixture.rctx"))
        );
        assert_eq!(
            args.selected_frontier_out.as_deref(),
            Some(std::path::Path::new("selected"))
        );
        assert_eq!(args.selected_frontier_label.as_deref(), Some("chosen"));
        assert_eq!(args.selected_frontier_tolerance, 1.25);
    }

    #[test]
    fn pareto_selected_frontier_package_helper_writes_sidecars() {
        let tmp = tempfile::TempDir::new().unwrap();
        let context_path = tmp.path().join("fixture.rctx");
        let out_dir = tmp.path().join("selected");
        let context = path4_context();
        std::fs::write(
            &context_path,
            rplan_io::write_rctx_string(&context).unwrap(),
        )
        .unwrap();
        let args = ParetoArgs {
            state: "TT".to_string(),
            year: crate::args::Year::Y2020,
            population: 4,
            generations: 1,
            base_seed: Some(7),
            output: None,
            selected_frontier_index: Some(0),
            selected_frontier_context: Some(context_path),
            selected_frontier_out: Some(out_dir.clone()),
            selected_frontier_label: Some("selected-tt".to_string()),
            selected_frontier_tolerance: 50.0,
        };
        let result = pareto_result_fixture();

        write_selected_frontier_package(&args, &result, 0, 2, "tt", "2020").unwrap();

        assert!(out_dir.join("selected-frontier.rplan").exists());
        assert!(out_dir.join("selected-frontier.rctx").exists());
        assert!(out_dir.join("audit-certificate.json").exists());
        assert!(out_dir.join("manifest.json").exists());
        let document = rplan_io::read_rplan_str(
            &std::fs::read_to_string(out_dir.join("selected-frontier.rplan")).unwrap(),
        )
        .unwrap();
        let context = rplan_io::read_rctx_str(
            &std::fs::read_to_string(out_dir.join("selected-frontier.rctx")).unwrap(),
        )
        .unwrap();
        let certificate: rplan_audit::AuditCertificate = serde_json::from_str(
            &std::fs::read_to_string(out_dir.join("audit-certificate.json")).unwrap(),
        )
        .unwrap();
        rplan_audit::verify_audit_certificate(&certificate, Some(&document.plan), Some(&context))
            .unwrap();
        let manifest: bisect_report::PlanManifest =
            serde_json::from_str(&std::fs::read_to_string(out_dir.join("manifest.json")).unwrap())
                .unwrap();
        crate::verify::verify_manifest_rplan_audit_certificate(&manifest, &out_dir).unwrap();
    }

    fn pareto_result_fixture() -> bisect_pareto::ParetoResult {
        bisect_pareto::ParetoResult {
            frontier: vec![bisect_pareto::ParetoEntry {
                plan: vec![1, 1, 2, 2],
                objectives: bisect_pareto::Objectives {
                    ec: 1.0,
                    d_seats: 0.0,
                    vra_deficit: 0.0,
                },
                dominated: false,
                generation_found: 0,
                validity_status: Some("valid".to_string()),
                audit_certificate_path: None,
                audit_certificate_sha256: None,
                audit_certificate_content_hash: None,
            }],
            config: bisect_pareto::ParetoConfig {
                n_population: 4,
                n_generations: 1,
                base_seed: 7,
                balance_tolerance: 0.5,
            },
            runtime_secs: 0.0,
            pareto_version: "1.0".to_string(),
        }
    }

    fn path4_context() -> rplan_core::RplanContext {
        let mut units = rplan_core::PlanUnitIndex {
            unit_kind: rplan_core::UnitKind::Imported,
            state: Some("TT".to_string()),
            year: Some(2020),
            canonical_order: rplan_core::CanonicalOrder::ExplicitUnitIds,
            unit_ids: (0..4).map(|idx| format!("u{idx}")).collect(),
            unit_universe_hash: String::new(),
            source_id: Some("pareto-cli-fixture".to_string()),
        };
        units.unit_universe_hash = units.compute_unit_universe_hash().unwrap();
        let mut context = rplan_core::RplanContext {
            rctx_version: rplan_core::RCTX_VERSION.to_string(),
            context_hash: String::new(),
            units,
            graph: Some(rplan_core::UnitGraph {
                edge_semantics: rplan_core::EdgeSemantics::Undirected,
                adjacency: vec![
                    vec![edge(1)],
                    vec![edge(0), edge(2)],
                    vec![edge(1), edge(3)],
                    vec![edge(2)],
                ],
            }),
            populations: Some(vec![100, 100, 100, 100]),
            subdivisions: None,
            demographics: None,
            geometry: None,
            source_hashes: rplan_core::SourceHashes {
                entries: BTreeMap::from([(
                    "fixture".to_string(),
                    format!("sha256:{}", "3".repeat(64)),
                )]),
            },
        };
        context.context_hash = context.compute_context_hash().unwrap();
        context
    }

    fn edge(to: u32) -> rplan_core::UnitEdge {
        rplan_core::UnitEdge {
            to,
            kind: rplan_core::EdgeKind::Boundary,
            weight: None,
        }
    }
}
