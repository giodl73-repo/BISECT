use anyhow::{Context, Result};
use bisect_ilp::{
    build_formulation, master_lp_string, solve, BranchAndCutMode, IlpModelArtifact, IlpSolveReport,
    IlpSolver,
};
use rplan_core::{
    canonical_sha256, CanonicalOrder, DistrictPlan, EdgeKind, EdgeSemantics, PlanUnitIndex,
    RplanContext, SourceHashes, UnitEdge, UnitGraph, UnitKind, DISTRICT_PLAN_SCHEMA_VERSION,
    RCTX_VERSION,
};
use rplan_io::{RplanDocument, RplanMetadataV02, RplanProvenance, RPLAN_V02};
use serde_json::json;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

const PACKAGE_ID: &str = "U.16+branch-and-cut-path8-benchmark";
const PAPER: &str = "U.16+branch-and-cut-redistricting";
const GENERATED_AT: &str = "2026-05-11T00:00:00Z";
const UNIT_COUNT: usize = 8;
const DISTRICTS: usize = 2;
const POP_TOLERANCE: f64 = 0.005;
const POP_TOLERANCE_PERCENT: f64 = POP_TOLERANCE * 100.0;

fn main() -> Result<()> {
    let out_dir = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(default_out_dir);
    std::fs::create_dir_all(&out_dir)
        .with_context(|| format!("create package directory {}", out_dir.display()))?;

    let adjacency = path_adjacency(UNIT_COUNT);
    let populations = vec![100; UNIT_COUNT];
    let formulation = build_formulation(&adjacency, &populations, DISTRICTS, POP_TOLERANCE);
    let mut result = solve(
        &formulation,
        &adjacency,
        &populations,
        DISTRICTS,
        POP_TOLERANCE,
        IlpSolver::BranchAndCut {
            mode: BranchAndCutMode::IterativeSeparation,
            incumbent_assignment: None,
            solver_name: Some("branch_and_cut_exact_branch_and_bound".to_string()),
        },
        0.0,
    );
    result.solve_time_secs = 0.0;
    let solution = result
        .plan
        .as_ref()
        .context("branch-and-cut benchmark solver did not return a plan")?;
    let assignment = (0..UNIT_COUNT)
        .map(|idx| {
            solution
                .get(&idx)
                .copied()
                .with_context(|| format!("solver plan missing unit {idx}"))
        })
        .collect::<Result<Vec<_>>>()?;

    let source_fixture = json!({
        "schema_version": "bisect-u16-synthetic-source-v1",
        "fixture_id": "path8-synthetic",
        "unit_count": UNIT_COUNT,
        "edge_count_undirected": undirected_edge_count(&adjacency),
        "populations": populations,
        "description": "Eight tract-shaped synthetic units on a path graph with equal populations.",
    });
    let source_fixture_hash = canonical_sha256(&source_fixture)?;

    let mut units = PlanUnitIndex {
        unit_kind: UnitKind::Tract,
        state: Some("WA".to_string()),
        year: Some(2020),
        canonical_order: CanonicalOrder::ExplicitUnitIds,
        unit_ids: (1..=UNIT_COUNT)
            .map(|idx| format!("53016{:06}", idx * 100))
            .collect(),
        unit_universe_hash: String::new(),
        source_id: Some("path8-synthetic".to_string()),
    };
    units.unit_universe_hash = units.compute_unit_universe_hash()?;

    let plan = DistrictPlan {
        schema_version: DISTRICT_PLAN_SCHEMA_VERSION.to_string(),
        units: units.clone(),
        assignment: assignment
            .iter()
            .copied()
            .map(|district| district as u32)
            .collect(),
        k: DISTRICTS,
        display_labels: vec!["1".to_string(), "2".to_string()],
        allow_empty_districts: false,
    };

    let mut context = RplanContext {
        rctx_version: RCTX_VERSION.to_string(),
        context_hash: String::new(),
        units: units.clone(),
        graph: Some(UnitGraph {
            edge_semantics: EdgeSemantics::Undirected,
            adjacency: adjacency
                .iter()
                .map(|edges| {
                    edges
                        .iter()
                        .copied()
                        .map(|to| UnitEdge {
                            to: to as u32,
                            kind: EdgeKind::Boundary,
                            weight: None,
                        })
                        .collect()
                })
                .collect(),
        }),
        populations: Some(populations.clone()),
        subdivisions: None,
        demographics: None,
        geometry: None,
        source_hashes: SourceHashes {
            entries: BTreeMap::from([("fixture".to_string(), source_fixture_hash.clone())]),
        },
    };
    context.context_hash = context.compute_context_hash()?;

    let document = RplanDocument {
        rplan_version: RPLAN_V02.to_string(),
        plan: plan.clone(),
        metadata: RplanMetadataV02 {
            label: PACKAGE_ID.to_string(),
            jurisdiction: "WA-synthetic".to_string(),
            chamber: "congressional".to_string(),
            created_at: GENERATED_AT.to_string(),
            description: Some(
                "Synthetic path graph solved by bisect-ilp branch-and-cut exact branch-and-bound."
                    .to_string(),
            ),
        },
        provenance: RplanProvenance {
            producer: BTreeMap::from([
                ("crate".to_string(), json!("bisect-cli")),
                ("method_crate".to_string(), json!("bisect-ilp")),
                ("method".to_string(), json!("branch-and-cut")),
            ]),
            source_hashes: BTreeMap::from([("fixture".to_string(), source_fixture_hash)]),
            conversion_lineage: vec![json!({
                "workflow": "cargo run -p bisect-cli --no-default-features --example u16_branch_cut_path8_benchmark_package",
                "package_tier": "benchmark-package",
            })],
        },
        geometry: None,
        extensions: BTreeMap::new(),
    };

    let model_path = out_dir.join("node_root.lp");
    let report_path = out_dir.join("ilp-solve-report.json");
    let transcript_path = out_dir.join("method-transcript.json");
    let timing_path = out_dir.join("benchmark-notes.json");
    let plan_path = out_dir.join("plan.rplan");
    let context_path = out_dir.join("context.rctx");
    let certificate_path = out_dir.join("audit-certificate.json");
    let command_path = out_dir.join("command-transcript.txt");
    let manifest_path = out_dir.join("manifest.json");

    std::fs::write(
        &model_path,
        master_lp_string(&adjacency, &populations, DISTRICTS, POP_TOLERANCE)?,
    )
    .with_context(|| format!("write {}", model_path.display()))?;
    let model_sha256 = bisect_report::sha256_file(&model_path)?;
    let solve_report = IlpSolveReport::with_model_artifact(
        formulation,
        result.clone(),
        IlpModelArtifact {
            format: "cplex-lp".to_string(),
            path: "node_root.lp".to_string(),
            sha256: model_sha256.clone(),
        },
    );
    std::fs::write(
        &report_path,
        format!("{}\n", solve_report.to_json_string()?),
    )
    .with_context(|| format!("write {}", report_path.display()))?;
    let report_sha256 = bisect_report::sha256_file(&report_path)?;

    std::fs::write(&plan_path, rplan_io::write_rplan_string(&document)?)
        .with_context(|| format!("write {}", plan_path.display()))?;
    std::fs::write(&context_path, rplan_io::write_rctx_string(&context)?)
        .with_context(|| format!("write {}", context_path.display()))?;

    let plan_hash = plan.plan_hash()?;
    let branch_certificate = result.branch_and_cut.clone();
    let transcript = json!({
        "schema_version": "method-transcript-v1",
        "package_id": PACKAGE_ID,
        "package_tier": "benchmark-package",
        "paper": PAPER,
        "producer_crate": "bisect-ilp",
        "producer_version": env!("CARGO_PKG_VERSION"),
        "method": "branch-and-cut",
        "solver_used": result.solver_used,
        "source_workflow": "cargo run -p bisect-cli --no-default-features --example u16_branch_cut_path8_benchmark_package",
        "status": result.status,
        "input_fixture": "path8-synthetic",
        "unit_count": UNIT_COUNT,
        "edge_count_undirected": undirected_edge_count(&adjacency),
        "districts": DISTRICTS,
        "population_tolerance": POP_TOLERANCE,
        "plan_hash": plan_hash,
        "context_hash": context.context_hash,
        "unit_universe_hash": units.unit_universe_hash,
        "optimal_edge_cut": result.optimal_ec,
        "solve_time_secs": result.solve_time_secs,
        "model_artifact_path": "node_root.lp",
        "model_artifact_sha256": model_sha256,
        "solve_report_path": "ilp-solve-report.json",
        "solve_report_sha256": report_sha256,
        "branch_and_cut_certificate": branch_certificate,
        "assignment": assignment,
        "scope_note": "Synthetic benchmark-tier package for exact branch-and-cut lineage, model-artifact hashing, and verifier acceptance. It is not a real-data runtime or scalability claim."
    });
    write_pretty_json(&transcript_path, &transcript)?;
    let transcript_sha256 = bisect_report::sha256_file(&transcript_path)?;

    let timing_notes = json!({
        "schema_version": "benchmark-timing-notes-v1",
        "package_id": PACKAGE_ID,
        "timing_kind": "deterministic-exact-solver-smoke",
        "unit_count": UNIT_COUNT,
        "edge_count_undirected": undirected_edge_count(&adjacency),
        "reference_hardware": {
            "cpu": "not pinned",
            "core_count": "not pinned",
            "ram": "not pinned",
            "os": "Windows development workstation"
        },
        "measurement_protocol": [
            "cargo run -p bisect-cli --no-default-features --example u16_branch_cut_path8_benchmark_package",
            "cargo run -p rplan-cli -- verify-certificate --certificate docs/examples/rplan-benchmark-packages/U.16+branch-and-cut-path8-benchmark/audit-certificate.json --plan docs/examples/rplan-benchmark-packages/U.16+branch-and-cut-path8-benchmark/plan.rplan --context docs/examples/rplan-benchmark-packages/U.16+branch-and-cut-path8-benchmark/context.rctx",
            "cargo run -p bisect-cli -- verify --manifest docs/examples/rplan-benchmark-packages/U.16+branch-and-cut-path8-benchmark/manifest.json"
        ],
        "wall_clock_claim": "none",
        "scope_note": "This benchmark package proves exact-solver artifact packaging and verifier integration for a bounded synthetic k=2 instance. It makes no broad performance claim."
    });
    write_pretty_json(&timing_path, &timing_notes)?;
    let timing_sha256 = bisect_report::sha256_file(&timing_path)?;

    let optimality_gap = result
        .branch_and_cut
        .as_ref()
        .and_then(|certificate| certificate.optimality_gap);
    let lineage = rplan_audit::AlgorithmLineage::new(
        "bisect-ilp",
        env!("CARGO_PKG_VERSION"),
        "branch-and-cut",
        Vec::new(),
        json!({
            "lineage_schema_version": "bisect-ilp-branch-and-cut-lineage-v1",
            "package_id": PACKAGE_ID,
            "input_fixture": "path8-synthetic",
            "solver_used": result.solver_used,
            "status": result.status,
            "optimal_edge_cut": result.optimal_ec,
            "optimality_gap": optimality_gap,
            "model_artifact_path": "node_root.lp",
            "model_artifact_sha256": model_sha256,
            "solve_report_path": "ilp-solve-report.json",
            "solve_report_sha256": report_sha256,
            "method_transcript_path": "method-transcript.json",
            "method_transcript_sha256": transcript_sha256,
            "benchmark_notes_path": "benchmark-notes.json",
            "benchmark_notes_sha256": timing_sha256,
        }),
    )?;
    let profile = synthetic_legal_profile();
    let certificate = rplan_audit::audit_plan_with_lineage(
        &plan,
        Some(&context),
        &profile,
        rplan_audit::RuntimeProvenance {
            binary_name: "bisect-cli-example".to_string(),
            binary_version: env!("CARGO_PKG_VERSION").to_string(),
            solver: Some(rplan_audit::SolverProvenance {
                name: "branch-and-cut".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
                mode: Some("exact-branch-and-bound-k2".to_string()),
                time_limit_secs: None,
                optimality_gap,
            }),
            ..rplan_audit::RuntimeProvenance::default()
        },
        &[
            rplan_audit::AuditConstraint::Population,
            rplan_audit::AuditConstraint::Contiguity,
        ],
        GENERATED_AT,
        Some(lineage),
    )?;
    write_pretty_json(&certificate_path, &certificate)?;

    std::fs::write(
        &command_path,
        "cargo run -p bisect-cli --no-default-features --example u16_branch_cut_path8_benchmark_package\n\
         cargo run -p rplan-cli -- verify-certificate --certificate docs/examples/rplan-benchmark-packages/U.16+branch-and-cut-path8-benchmark/audit-certificate.json --plan docs/examples/rplan-benchmark-packages/U.16+branch-and-cut-path8-benchmark/plan.rplan --context docs/examples/rplan-benchmark-packages/U.16+branch-and-cut-path8-benchmark/context.rctx\n\
         cargo run -p bisect-cli -- verify --manifest docs/examples/rplan-benchmark-packages/U.16+branch-and-cut-path8-benchmark/manifest.json\n",
    )
    .with_context(|| format!("write {}", command_path.display()))?;

    let files = vec![
        manifest_file(&out_dir, "plan.rplan", "plan")?,
        manifest_file(&out_dir, "context.rctx", "context")?,
        manifest_file(&out_dir, "audit-certificate.json", "certificate")?,
        manifest_file(&out_dir, "ilp-solve-report.json", "ilp-solve-report")?,
        manifest_file(&out_dir, "node_root.lp", "model-artifact")?,
        manifest_file(&out_dir, "method-transcript.json", "method-transcript")?,
        manifest_file(&out_dir, "benchmark-notes.json", "benchmark-notes")?,
        manifest_file(&out_dir, "command-transcript.txt", "command-transcript")?,
    ];
    let manifest = json!({
        "schema_version": "benchmark-rplan-package-manifest-v1",
        "example_id": PACKAGE_ID,
        "paper": PAPER,
        "package_tier": "benchmark-package",
        "source_workflow": "cargo run -p bisect-cli --no-default-features --example u16_branch_cut_path8_benchmark_package",
        "generated_at_utc": GENERATED_AT,
        "status": "proved-optimal",
        "benchmark_scope": "deterministic synthetic exact-solver smoke package",
        "data_requirements": "none; synthetic 8-unit path graph",
        "repository_footprint": "small in-repo package",
        "files": files,
        "verification_command": "cargo run -p rplan-cli -- verify-certificate --certificate docs/examples/rplan-benchmark-packages/U.16+branch-and-cut-path8-benchmark/audit-certificate.json --plan docs/examples/rplan-benchmark-packages/U.16+branch-and-cut-path8-benchmark/plan.rplan --context docs/examples/rplan-benchmark-packages/U.16+branch-and-cut-path8-benchmark/context.rctx",
        "bisect_verification_command": "cargo run -p bisect-cli -- verify --manifest docs/examples/rplan-benchmark-packages/U.16+branch-and-cut-path8-benchmark/manifest.json",
        "scope_note": "Generated by the bisect-ilp branch-and-cut exact branch-and-bound path for a deterministic 8-unit synthetic path. The package proves benchmark-tier exact artifact packaging, model-artifact hashing, and RPLAN certificate acceptance, not real-data scaling."
    });
    write_pretty_json(&manifest_path, &manifest)?;

    println!("wrote {}", out_dir.display());
    Ok(())
}

fn path_adjacency(n: usize) -> Vec<Vec<usize>> {
    let mut adjacency = vec![Vec::new(); n];
    for idx in 0..n {
        if idx > 0 {
            adjacency[idx].push(idx - 1);
        }
        if idx + 1 < n {
            adjacency[idx].push(idx + 1);
        }
    }
    adjacency
}

fn undirected_edge_count(adjacency: &[Vec<usize>]) -> usize {
    adjacency.iter().map(Vec::len).sum::<usize>() / 2
}

fn synthetic_legal_profile() -> rplan_audit::LegalProfile {
    rplan_audit::LegalProfile {
        schema_version: rplan_audit::LEGAL_PROFILE_SCHEMA_VERSION.to_string(),
        profile_id: "U16_BRANCH_AND_CUT_SYNTHETIC_V1".to_string(),
        jurisdiction: "WA-synthetic".to_string(),
        chamber: rplan_audit::Chamber::Congressional,
        year: 2020,
        population_tolerance: rplan_audit::PopulationToleranceRule::Percent {
            max_deviation_percent: POP_TOLERANCE_PERCENT,
        },
        contiguity_required: true,
        county_split_rule: rplan_audit::SplitRule::CountOnly,
        municipal_split_rule: rplan_audit::SplitRule::NotEvaluated,
        nesting_rule: rplan_audit::NestingRule::NotEvaluated,
        vra_policy: rplan_audit::VraPolicy::NotEvaluated,
    }
}

fn default_out_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("repo root")
        .join("docs")
        .join("examples")
        .join("rplan-benchmark-packages")
        .join(PACKAGE_ID)
}

fn write_pretty_json<T: serde::Serialize>(path: &Path, value: &T) -> Result<()> {
    let text = serde_json::to_string_pretty(value)?;
    std::fs::write(path, format!("{text}\n")).with_context(|| format!("write {}", path.display()))
}

fn manifest_file(package_root: &Path, path: &str, role: &str) -> Result<serde_json::Value> {
    let sha256 = bisect_report::sha256_file(&package_root.join(path))?;
    Ok(json!({
        "path": path,
        "sha256": sha256,
        "role": role,
    }))
}
