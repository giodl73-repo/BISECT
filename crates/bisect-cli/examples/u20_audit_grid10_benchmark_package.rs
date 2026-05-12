use anyhow::{Context, Result};
use rplan_core::{
    canonical_sha256, CanonicalOrder, DistrictPlan, EdgeKind, EdgeSemantics, PlanUnitIndex,
    RplanContext, SourceHashes, UnitEdge, UnitGraph, UnitKind, DISTRICT_PLAN_SCHEMA_VERSION,
    RCTX_VERSION,
};
use rplan_io::{RplanDocument, RplanMetadataV02, RplanProvenance, RPLAN_V02};
use serde_json::json;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

const PACKAGE_ID: &str = "U.20+audit-grid10-benchmark";
const PAPER: &str = "U.20+plan-audit-certificates";
const GENERATED_AT: &str = "2026-05-11T00:00:00Z";
const GRID_SIDE: usize = 10;
const DISTRICTS: usize = 2;

fn main() -> Result<()> {
    let out_dir = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(default_out_dir);
    std::fs::create_dir_all(&out_dir)
        .with_context(|| format!("create package directory {}", out_dir.display()))?;

    let adjacency = grid_adjacency(GRID_SIDE);
    let populations = vec![100; GRID_SIDE * GRID_SIDE];
    let assignment = vertical_split_assignment(GRID_SIDE);

    let source_fixture = json!({
        "schema_version": "rplan-u20-audit-synthetic-source-v1",
        "fixture_id": "grid10-audit-synthetic",
        "grid_side": GRID_SIDE,
        "unit_count": populations.len(),
        "edge_count_undirected": undirected_edge_count(&adjacency),
        "populations": populations,
        "description": "One hundred tract-shaped synthetic units on a 10x10 rook-adjacency grid with equal populations for audit verification stress.",
    });
    let source_fixture_hash = canonical_sha256(&source_fixture)?;

    let mut units = PlanUnitIndex {
        unit_kind: UnitKind::Tract,
        state: Some("WA".to_string()),
        year: Some(2020),
        canonical_order: CanonicalOrder::ExplicitUnitIds,
        unit_ids: (1..=populations.len())
            .map(|idx| format!("53020{:06}", idx * 100))
            .collect(),
        unit_universe_hash: String::new(),
        source_id: Some("grid10-audit-synthetic".to_string()),
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
                "Synthetic 10x10 grid plan used as a U.20 audit certificate benchmark.".to_string(),
            ),
        },
        provenance: RplanProvenance {
            producer: BTreeMap::from([
                ("crate".to_string(), json!("bisect-cli")),
                ("method_crate".to_string(), json!("rplan-audit")),
                ("method".to_string(), json!("audit-fixed-point")),
            ]),
            source_hashes: BTreeMap::from([("fixture".to_string(), source_fixture_hash)]),
            conversion_lineage: vec![json!({
                "workflow": "cargo run -p bisect-cli --no-default-features --example u20_audit_grid10_benchmark_package",
                "package_tier": "benchmark-package",
            })],
        },
        geometry: None,
        extensions: BTreeMap::new(),
    };

    let plan_path = out_dir.join("plan.rplan");
    let context_path = out_dir.join("context.rctx");
    let summary_path = out_dir.join("audit-stress-summary.json");
    let transcript_path = out_dir.join("method-transcript.json");
    let timing_path = out_dir.join("benchmark-notes.json");
    let certificate_path = out_dir.join("audit-certificate.json");
    let command_path = out_dir.join("command-transcript.txt");
    let manifest_path = out_dir.join("manifest.json");

    std::fs::write(&plan_path, rplan_io::write_rplan_string(&document)?)
        .with_context(|| format!("write {}", plan_path.display()))?;
    std::fs::write(&context_path, rplan_io::write_rctx_string(&context)?)
        .with_context(|| format!("write {}", context_path.display()))?;

    let plan_hash = plan.plan_hash()?;
    let summary = json!({
        "schema_version": "u20-audit-stress-summary-v1",
        "package_id": PACKAGE_ID,
        "package_tier": "benchmark-package",
        "fixture": "grid10-audit-synthetic",
        "unit_count": populations.len(),
        "edge_count_undirected": undirected_edge_count(&adjacency),
        "districts": DISTRICTS,
        "district_populations": district_populations(&assignment, &populations, DISTRICTS),
        "constraints": ["plan-shape", "population", "contiguity"],
        "plan_hash": plan_hash,
        "context_hash": context.context_hash,
        "unit_universe_hash": units.unit_universe_hash,
        "scope_note": "Audit-only benchmark package for U.20 verifier scale and fixed-point certificate behavior. It makes no construction, optimization, or legal sufficiency claim."
    });
    write_pretty_json(&summary_path, &summary)?;
    let summary_sha256 = bisect_report::sha256_file(&summary_path)?;

    let transcript = json!({
        "schema_version": "method-transcript-v1",
        "package_id": PACKAGE_ID,
        "package_tier": "benchmark-package",
        "paper": PAPER,
        "producer_crate": "rplan-audit",
        "producer_version": env!("CARGO_PKG_VERSION"),
        "method": "audit-fixed-point",
        "source_workflow": "cargo run -p bisect-cli --no-default-features --example u20_audit_grid10_benchmark_package",
        "status": "audit-certificate-issued",
        "input_fixture": "grid10-audit-synthetic",
        "grid_side": GRID_SIDE,
        "unit_count": populations.len(),
        "edge_count_undirected": undirected_edge_count(&adjacency),
        "plan_hash": plan_hash,
        "context_hash": context.context_hash,
        "summary_path": "audit-stress-summary.json",
        "summary_sha256": summary_sha256,
        "scope_note": "This transcript records certificate generation for a deterministic synthetic RPLAN/RCTX pair, not an algorithmic search or construction run."
    });
    write_pretty_json(&transcript_path, &transcript)?;
    let transcript_sha256 = bisect_report::sha256_file(&transcript_path)?;

    let timing_notes = json!({
        "schema_version": "benchmark-timing-notes-v1",
        "package_id": PACKAGE_ID,
        "timing_kind": "deterministic-audit-verifier-smoke",
        "unit_count": populations.len(),
        "edge_count_undirected": undirected_edge_count(&adjacency),
        "reference_hardware": {
            "cpu": "not pinned",
            "core_count": "not pinned",
            "ram": "not pinned",
            "os": "Windows development workstation"
        },
        "measurement_protocol": [
            "cargo run -p bisect-cli --no-default-features --example u20_audit_grid10_benchmark_package",
            "cargo run -p rplan-cli -- verify-certificate --certificate docs/examples/rplan-benchmark-packages/U.20+audit-grid10-benchmark/audit-certificate.json --plan docs/examples/rplan-benchmark-packages/U.20+audit-grid10-benchmark/plan.rplan --context docs/examples/rplan-benchmark-packages/U.20+audit-grid10-benchmark/context.rctx",
            "cargo run -p bisect-cli -- verify --manifest docs/examples/rplan-benchmark-packages/U.20+audit-grid10-benchmark/manifest.json"
        ],
        "wall_clock_claim": "none",
        "scope_note": "This benchmark package records verifier protocol and scale metadata. It is not a runtime benchmark claim."
    });
    write_pretty_json(&timing_path, &timing_notes)?;
    let timing_sha256 = bisect_report::sha256_file(&timing_path)?;

    let lineage = rplan_audit::AlgorithmLineage::new(
        "rplan-audit",
        env!("CARGO_PKG_VERSION"),
        "audit-fixed-point",
        Vec::new(),
        json!({
            "lineage_schema_version": "u20-audit-fixed-point-lineage-v1",
            "package_id": PACKAGE_ID,
            "input_fixture": "grid10-audit-synthetic",
            "summary_path": "audit-stress-summary.json",
            "summary_sha256": summary_sha256,
            "method_transcript_path": "method-transcript.json",
            "method_transcript_sha256": transcript_sha256,
            "benchmark_notes_path": "benchmark-notes.json",
            "benchmark_notes_sha256": timing_sha256,
            "status": "audit-certificate-issued",
        }),
    )?;
    let certificate = rplan_audit::audit_plan_with_lineage(
        &plan,
        Some(&context),
        &rplan_audit::LegalProfile::us_congressional_project_v1(2020),
        rplan_audit::RuntimeProvenance {
            binary_name: "bisect-cli-example".to_string(),
            binary_version: env!("CARGO_PKG_VERSION").to_string(),
            ..rplan_audit::RuntimeProvenance::default()
        },
        &[
            rplan_audit::AuditConstraint::PlanShape,
            rplan_audit::AuditConstraint::Population,
            rplan_audit::AuditConstraint::Contiguity,
        ],
        GENERATED_AT,
        Some(lineage),
    )?;
    write_pretty_json(&certificate_path, &certificate)?;

    std::fs::write(
        &command_path,
        "cargo run -p bisect-cli --no-default-features --example u20_audit_grid10_benchmark_package\n\
         cargo run -p rplan-cli -- verify-certificate --certificate docs/examples/rplan-benchmark-packages/U.20+audit-grid10-benchmark/audit-certificate.json --plan docs/examples/rplan-benchmark-packages/U.20+audit-grid10-benchmark/plan.rplan --context docs/examples/rplan-benchmark-packages/U.20+audit-grid10-benchmark/context.rctx\n\
         cargo run -p bisect-cli -- verify --manifest docs/examples/rplan-benchmark-packages/U.20+audit-grid10-benchmark/manifest.json\n",
    )
    .with_context(|| format!("write {}", command_path.display()))?;

    let files = vec![
        manifest_file(&out_dir, "plan.rplan", "plan")?,
        manifest_file(&out_dir, "context.rctx", "context")?,
        manifest_file(&out_dir, "audit-certificate.json", "certificate")?,
        manifest_file(
            &out_dir,
            "audit-stress-summary.json",
            "audit-stress-summary",
        )?,
        manifest_file(&out_dir, "method-transcript.json", "method-transcript")?,
        manifest_file(&out_dir, "benchmark-notes.json", "benchmark-notes")?,
        manifest_file(&out_dir, "command-transcript.txt", "command-transcript")?,
    ];
    let manifest = json!({
        "schema_version": "benchmark-rplan-package-manifest-v1",
        "example_id": PACKAGE_ID,
        "paper": PAPER,
        "package_tier": "benchmark-package",
        "source_workflow": "cargo run -p bisect-cli --no-default-features --example u20_audit_grid10_benchmark_package",
        "generated_at_utc": GENERATED_AT,
        "status": "audit-certificate-issued",
        "benchmark_scope": "deterministic synthetic audit verifier smoke package",
        "data_requirements": "none; synthetic 10x10 rook-adjacency grid",
        "repository_footprint": "small in-repo package",
        "files": files,
        "verification_command": "cargo run -p rplan-cli -- verify-certificate --certificate docs/examples/rplan-benchmark-packages/U.20+audit-grid10-benchmark/audit-certificate.json --plan docs/examples/rplan-benchmark-packages/U.20+audit-grid10-benchmark/plan.rplan --context docs/examples/rplan-benchmark-packages/U.20+audit-grid10-benchmark/context.rctx",
        "bisect_verification_command": "cargo run -p bisect-cli -- verify --manifest docs/examples/rplan-benchmark-packages/U.20+audit-grid10-benchmark/manifest.json",
        "scope_note": "Generated as an audit-only RPLAN/RCTX fixed-point package for a deterministic 100-unit synthetic grid. The package proves benchmark-tier verifier packaging and certificate acceptance, not construction quality or legal sufficiency."
    });
    write_pretty_json(&manifest_path, &manifest)?;

    println!("wrote {}", out_dir.display());
    Ok(())
}

fn grid_adjacency(side: usize) -> Vec<Vec<usize>> {
    let mut adjacency = vec![Vec::new(); side * side];
    for row in 0..side {
        for col in 0..side {
            let idx = row * side + col;
            if row > 0 {
                adjacency[idx].push((row - 1) * side + col);
            }
            if col > 0 {
                adjacency[idx].push(row * side + col - 1);
            }
            if col + 1 < side {
                adjacency[idx].push(row * side + col + 1);
            }
            if row + 1 < side {
                adjacency[idx].push((row + 1) * side + col);
            }
        }
    }
    adjacency
}

fn vertical_split_assignment(side: usize) -> Vec<usize> {
    (0..side)
        .flat_map(|row| {
            (0..side).map(move |col| {
                let _idx = row * side + col;
                if col < side / 2 {
                    0
                } else {
                    1
                }
            })
        })
        .collect()
}

fn district_populations(assignment: &[usize], populations: &[i64], districts: usize) -> Vec<i64> {
    let mut totals = vec![0; districts];
    for (&district, &population) in assignment.iter().zip(populations) {
        totals[district] += population;
    }
    totals
}

fn undirected_edge_count(adjacency: &[Vec<usize>]) -> usize {
    adjacency.iter().map(Vec::len).sum::<usize>() / 2
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
