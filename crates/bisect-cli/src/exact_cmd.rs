use crate::args::{ExactArgs, ExactMethodArg};
use anyhow::{anyhow, Context};

pub fn run_exact(args: &ExactArgs) -> anyhow::Result<()> {
    match args.method {
        ExactMethodArg::BranchAndPrice => run_branch_and_price(args),
    }
}

fn run_branch_and_price(args: &ExactArgs) -> anyhow::Result<()> {
    let context_text = std::fs::read_to_string(&args.context)
        .with_context(|| format!("read input RCTX {}", args.context.display()))?;
    let context = rplan_io::read_rctx_str(&context_text)
        .with_context(|| format!("parse input RCTX {}", args.context.display()))?;
    let graph = context
        .graph
        .as_ref()
        .ok_or_else(|| anyhow!("RCTX graph is required for exact branch-and-price"))?;
    let populations = context
        .populations
        .as_ref()
        .ok_or_else(|| anyhow!("RCTX populations are required for exact branch-and-price"))?;
    let adjacency = graph_adjacency(graph)?;
    let report = bisect_column::solve_branch_price(
        &adjacency,
        populations,
        bisect_column::BranchPriceConfig {
            k: args.districts,
            tolerance: args.tolerance / 100.0,
            formulation_only: args.formulation_only,
            exact_fixture_limit: args.exact_fixture_limit,
        },
    )?;

    std::fs::create_dir_all(&args.out_dir)
        .with_context(|| format!("create output directory {}", args.out_dir.display()))?;
    let report_path = args.out_dir.join("branch-price-report.json");
    std::fs::write(
        &report_path,
        serde_json::to_string_pretty(&report).context("serialize branch-price report")?,
    )
    .with_context(|| format!("write {}", report_path.display()))?;

    let lineage = report
        .algorithm_lineage(env!("CARGO_PKG_VERSION"), Vec::new())
        .context("build branch-price algorithm lineage")?;
    let lineage_path = args.out_dir.join("algorithm-lineage.json");
    std::fs::write(
        &lineage_path,
        serde_json::to_string_pretty(&lineage).context("serialize branch-price lineage")?,
    )
    .with_context(|| format!("write {}", lineage_path.display()))?;
    Ok(())
}

fn graph_adjacency(graph: &rplan_core::UnitGraph) -> anyhow::Result<Vec<Vec<usize>>> {
    graph
        .adjacency
        .iter()
        .enumerate()
        .map(|(idx, edges)| {
            edges
                .iter()
                .map(|edge| {
                    usize::try_from(edge.to)
                        .map_err(|_| anyhow!("graph edge target at unit {idx} does not fit usize"))
                })
                .collect()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    use tempfile::TempDir;

    use rplan_core::{
        CanonicalOrder, EdgeKind, EdgeSemantics, PlanUnitIndex, RplanContext, SourceHashes,
        UnitEdge, UnitGraph, UnitKind, RCTX_VERSION,
    };

    #[test]
    fn run_exact_branch_and_price_emits_report_and_lineage() {
        let tmp = TempDir::new().unwrap();
        let context_path = tmp.path().join("fixture.rctx");
        let out_dir = tmp.path().join("exact");
        std::fs::write(
            &context_path,
            rplan_io::write_rctx_string(&path4_context()).unwrap(),
        )
        .unwrap();

        run_exact(&ExactArgs {
            context: context_path,
            out_dir: out_dir.clone(),
            method: ExactMethodArg::BranchAndPrice,
            districts: 2,
            tolerance: 1.0,
            formulation_only: false,
            exact_fixture_limit: 8,
        })
        .unwrap();

        let report_text =
            std::fs::read_to_string(out_dir.join("branch-price-report.json")).unwrap();
        let report: bisect_column::BranchPriceReport = serde_json::from_str(&report_text).unwrap();
        assert_eq!(
            report.status,
            bisect_column::BranchPriceStatus::ExactFixtureOptimal
        );
        assert_eq!(report.solution.unwrap().assignment, vec![0, 0, 1, 1]);

        let lineage_text = std::fs::read_to_string(out_dir.join("algorithm-lineage.json")).unwrap();
        let lineage: rplan_audit::AlgorithmLineage = serde_json::from_str(&lineage_text).unwrap();
        assert_eq!(lineage.producer_crate, "bisect-column");
        assert_eq!(lineage.method, "branch-and-price");
    }

    fn path4_context() -> RplanContext {
        let mut units = PlanUnitIndex {
            unit_kind: UnitKind::Imported,
            state: Some("TT".to_string()),
            year: Some(2020),
            canonical_order: CanonicalOrder::ExplicitUnitIds,
            unit_ids: (0..4).map(|idx| format!("u{idx}")).collect(),
            unit_universe_hash: String::new(),
            source_id: Some("u17-fixture".to_string()),
        };
        units.unit_universe_hash = units.compute_unit_universe_hash().unwrap();
        RplanContext {
            rctx_version: RCTX_VERSION.to_string(),
            context_hash: String::new(),
            units,
            graph: Some(UnitGraph {
                edge_semantics: EdgeSemantics::Undirected,
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
            source_hashes: SourceHashes {
                entries: BTreeMap::from([(
                    "fixture".to_string(),
                    format!("sha256:{}", "1".repeat(64)),
                )]),
            },
        }
    }

    fn edge(to: u32) -> UnitEdge {
        UnitEdge {
            to,
            kind: EdgeKind::Boundary,
            weight: None,
        }
    }
}
