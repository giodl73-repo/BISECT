//! The native complete-column benchmark, with lossless population input.
use serde::Deserialize;
use serde_json::{json, Value};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Instance {
    adjacency: Vec<Vec<usize>>,
    populations: Vec<i64>,
    k: usize,
    tolerance: f64,
    formulation_only: bool,
    exact_fixture_limit: usize,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ContextSettings {
    pub k: usize,
    pub tolerance: f64,
    pub formulation_only: bool,
    pub exact_fixture_limit: usize,
    pub label: String,
    pub chamber: String,
    pub created_at: String,
}

pub(crate) fn solve_context(text: &str, settings: ContextSettings) -> Result<Value, String> {
    if text.len() > 8 * 1024 * 1024 { return Err("Context exceeds 8 MiB.".into()); }
    for field in [&settings.label, &settings.chamber, &settings.created_at] {
        if field.trim().is_empty() || field.len() > 200 {
            return Err("Plan metadata must be nonempty and at most 200 bytes.".into());
        }
    }
    let context = rplan_io::read_rctx_str(text).map_err(|e| e.to_string())?;
    if context.units.unit_ids.len() > bisect_column::pricing::MAX_PRICING_UNITS {
        return Err("Complete column pricing supports at most 20 units.".into());
    }
    let graph = context.graph.as_ref().ok_or("RCTX graph is required.")?;
    if graph.edge_semantics != rplan_core::EdgeSemantics::Undirected {
        return Err("Column benchmark requires an undirected graph.".into());
    }
    let populations = context.populations.as_ref().ok_or("RCTX populations are required.")?;
    let adjacency = graph.adjacency.iter().map(|edges| edges.iter().map(|edge| {
        usize::try_from(edge.to).map_err(|_| "Graph target does not fit usize.")
    }).collect::<Result<Vec<_>, _>>()).collect::<Result<Vec<_>, _>>()?;
    let instance = Instance { adjacency, populations: populations.clone(), k: settings.k,
        tolerance: settings.tolerance, formulation_only: settings.formulation_only,
        exact_fixture_limit: settings.exact_fixture_limit };
    let report = solve_instance(&instance)?;
    let lineage = report.algorithm_lineage(env!("CARGO_PKG_VERSION"), Vec::new()).map_err(|e| e.to_string())?;
    let document = report.solution.as_ref().map(|solution| {
        let document = rplan_io::RplanDocument {
            rplan_version: rplan_io::RPLAN_V02.into(),
            plan: rplan_core::DistrictPlan {
                schema_version: rplan_core::DISTRICT_PLAN_SCHEMA_VERSION.into(),
                units: context.units.clone(),
                assignment: solution.assignment.iter().map(|&d| d as u32).collect(),
                k: settings.k, display_labels: (1..=settings.k).map(|d| d.to_string()).collect(),
                allow_empty_districts: false,
            },
            metadata: rplan_io::RplanMetadataV02 { label: settings.label.clone(),
                jurisdiction: context.units.state.clone().unwrap_or_else(|| "US".into()),
                chamber: settings.chamber.clone(), created_at: settings.created_at.clone(),
                description: Some("Native complete-column benchmark solution; no external proof certificate.".into()) },
            provenance: rplan_io::RplanProvenance {
                producer: std::collections::BTreeMap::from([
                    ("crate".into(), json!("bisect-column")),
                    ("method".into(), json!("branch-and-price")),
                    ("parameters".into(), json!({"k":settings.k,"tolerance":settings.tolerance,"formulation_only":settings.formulation_only,"exact_fixture_limit":settings.exact_fixture_limit})),
                    ("context_hash".into(), json!(context.context_hash)),
                ]),
                source_hashes: context.source_hashes.entries.clone(), conversion_lineage: Vec::new(),
            }, geometry: None, extensions: std::collections::BTreeMap::new(),
        };
        let serialized = rplan_io::write_rplan_string(&document).map_err(|e| e.to_string())?;
        rplan_io::read_rplan_str(&serialized).map_err(|e| e.to_string())?;
        Ok::<_, String>((serialized, document.plan.plan_hash().map_err(|e| e.to_string())?))
    }).transpose()?;
    use sha2::{Digest, Sha256};
    Ok(json!({"schema_version":"bisect-branch-price-context-output-v1","report":report,
        "context_hash":context.context_hash,"context_bytes_sha256":format!("sha256:{:x}",Sha256::digest(text.as_bytes())),
        "context_json":text,"document_json":document.as_ref().map(|d|&d.0),"plan_hash":document.as_ref().map(|d|&d.1),
        "algorithm_lineage_json":serde_json::to_string_pretty(&lineage).map_err(|e|e.to_string())?,"algorithm_lineage":lineage,"total_population":populations.iter().sum::<i64>().to_string(),
        "scope":"Native complete-column benchmark on hash-checked supplied RCTX; at most 20 units for pricing and 12 for browser exact master search. Objective counts unweighted graph edges, including bridges; supplied edge weights and kinds are retained in the exported context but do not change the objective. Population tolerance uses native f64 arithmetic. Solved assignments export as RPLAN; formulation-only and infeasible outputs export no plan. Input bytes and context identity are retained; source truth, legal compliance and external proof verification are not established. Plan has no embedded map geometry."}))
}

pub(crate) fn solve(text: &str) -> Result<Value, String> {
    let input: Instance = crate::certified_exact::parse(text)?;
    let report = solve_instance(&input)?;
    use sha2::{Digest, Sha256};
    Ok(json!({
        "schema_version": "bisect-branch-price-browser-output-v1",
        "report": report,
        "input_bytes_sha256": format!("sha256:{:x}", Sha256::digest(text.as_bytes())),
        "total_population": input.populations.iter().sum::<i64>().to_string(),
        "scope": "Native complete-column benchmark: at most 20 units for pricing, at most 12 for exact master search. Population bounds use the native f64 tolerance arithmetic. Formulation-only carries no solution or optimality bound. Exact-fixture-optimal means minimum unweighted graph boundary among feasible connected k-way partitions of the supplied graph; no external proof certificate or source authentication."
    }))
}

fn solve_instance(input: &Instance) -> Result<bisect_column::BranchPriceReport, String> {
    // Complete master enumeration can be expensive even with bounded pricing.
    // Larger inputs can still generate the native formulation without claiming a solve.
    if input.exact_fixture_limit > 12 {
        return Err("Browser exact master limit must be at most 12 units.".into());
    }
    bisect_column::solve_branch_price(
        &input.adjacency,
        &input.populations,
        bisect_column::BranchPriceConfig {
            k: input.k,
            tolerance: input.tolerance,
            formulation_only: input.formulation_only,
            exact_fixture_limit: input.exact_fixture_limit,
        },
    ).map_err(|e| e.to_string())
}
