use anyhow::{Context, Result};
use bisect_apportion::{spectral_bisect, SpectralConfig};
use rplan_core::{
    canonical_sha256, CanonicalOrder, DistrictPlan, EdgeKind, EdgeSemantics, PlanUnitIndex,
    RplanContext, SourceHashes, UnitEdge, UnitGraph, UnitKind, DISTRICT_PLAN_SCHEMA_VERSION,
    RCTX_VERSION,
};
use rplan_io::{RplanDocument, RplanMetadataV02, RplanProvenance, RPLAN_V02};
use serde_json::json;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

const PACKAGE_ID: &str = "T.14+spectral-generated-synthetic";
const PAPER: &str = "T.14+spectral-partitioning";
const GENERATED_AT: &str = "2026-05-11T00:00:00Z";

fn main() -> Result<()> {
    let out_dir = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(default_out_dir);
    std::fs::create_dir_all(&out_dir)
        .with_context(|| format!("create package directory {}", out_dir.display()))?;

    let adjacency = vec![
        vec![1],
        vec![0, 2],
        vec![1, 3],
        vec![2, 4],
        vec![3, 5],
        vec![4],
    ];
    let populations = vec![100; 6];
    let config = SpectralConfig {
        max_iters: 200,
        tolerance: 0.05,
        target_fraction: 0.5,
    };
    let result = spectral_bisect(&adjacency, &populations, config)?;

    let source_fixture = json!({
        "schema_version": "bisect-spectral-synthetic-source-v1",
        "fixture_id": "path6-synthetic",
        "adjacency": adjacency,
        "populations": populations,
        "description": "Six tract-shaped synthetic units on a path graph with equal populations.",
    });
    let source_fixture_hash = canonical_sha256(&source_fixture)?;

    let mut units = PlanUnitIndex {
        unit_kind: UnitKind::Tract,
        state: Some("WA".to_string()),
        year: Some(2020),
        canonical_order: CanonicalOrder::ExplicitUnitIds,
        unit_ids: vec![
            "53001000100".to_string(),
            "53001000200".to_string(),
            "53001000300".to_string(),
            "53001000400".to_string(),
            "53001000500".to_string(),
            "53001000600".to_string(),
        ],
        unit_universe_hash: String::new(),
        source_id: Some("path6-synthetic".to_string()),
    };
    units.unit_universe_hash = units.compute_unit_universe_hash()?;

    let plan = DistrictPlan {
        schema_version: DISTRICT_PLAN_SCHEMA_VERSION.to_string(),
        units: units.clone(),
        assignment: result
            .assignment
            .iter()
            .copied()
            .map(|district| district as u32)
            .collect(),
        k: 2,
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
                "Synthetic six-unit path graph partitioned by bisect-apportion spectral bisection."
                    .to_string(),
            ),
        },
        provenance: RplanProvenance {
            producer: BTreeMap::from([
                ("crate".to_string(), json!("bisect-cli")),
                ("method_crate".to_string(), json!("bisect-apportion")),
                ("method".to_string(), json!("spectral")),
            ]),
            source_hashes: BTreeMap::from([("fixture".to_string(), source_fixture_hash)]),
            conversion_lineage: vec![json!({
                "workflow": "cargo run -p bisect-cli --no-default-features --example spectral_path6_package",
                "package_tier": "method-produced-fixture",
            })],
        },
        geometry: None,
        extensions: BTreeMap::new(),
    };

    let summary_path = out_dir.join("spectral-summary.json");
    let transcript_path = out_dir.join("method-transcript.json");
    let plan_path = out_dir.join("plan.rplan");
    let context_path = out_dir.join("context.rctx");
    let certificate_path = out_dir.join("audit-certificate.json");
    let command_path = out_dir.join("command-transcript.txt");
    let manifest_path = out_dir.join("manifest.json");

    write_pretty_json(&summary_path, &result.summary)?;
    let summary_sha256 = bisect_report::sha256_file(&summary_path)?;

    std::fs::write(&plan_path, rplan_io::write_rplan_string(&document)?)
        .with_context(|| format!("write {}", plan_path.display()))?;
    std::fs::write(&context_path, rplan_io::write_rctx_string(&context)?)
        .with_context(|| format!("write {}", context_path.display()))?;

    let plan_hash = plan.plan_hash()?;
    let transcript = json!({
        "schema_version": "method-transcript-v1",
        "package_id": PACKAGE_ID,
        "package_tier": "method-produced-fixture",
        "paper": PAPER,
        "producer_crate": "bisect-apportion",
        "producer_version": env!("CARGO_PKG_VERSION"),
        "method": "spectral",
        "source_workflow": "cargo run -p bisect-cli --no-default-features --example spectral_path6_package",
        "status": "partitioned",
        "seed": null,
        "input_fixture": "path6-synthetic",
        "plan_hash": plan_hash,
        "context_hash": context.context_hash,
        "unit_universe_hash": units.unit_universe_hash,
        "summary_path": "spectral-summary.json",
        "summary_sha256": summary_sha256,
        "adjacency": adjacency,
        "populations": populations,
        "assignment": result.assignment,
        "vector": result.vector,
        "summary": result.summary,
        "scope_note": "Real crate-generated spectral bisection over a deterministic six-unit path fixture; validates spectral package generation and lineage, not real-data construction quality."
    });
    write_pretty_json(&transcript_path, &transcript)?;
    let transcript_sha256 = bisect_report::sha256_file(&transcript_path)?;

    let lineage = rplan_audit::AlgorithmLineage::new(
        "bisect-apportion",
        env!("CARGO_PKG_VERSION"),
        "spectral",
        Vec::new(),
        json!({
            "lineage_schema_version": "bisect-spectral-lineage-v1",
            "package_id": PACKAGE_ID,
            "input_fixture": "path6-synthetic",
            "summary_path": "spectral-summary.json",
            "summary_sha256": summary_sha256,
            "method_transcript_path": "method-transcript.json",
            "method_transcript_sha256": transcript_sha256,
            "status": "partitioned",
            "edge_cut": result.summary.edge_cut,
            "population_deviation": result.summary.population_deviation,
            "parameter_hash": result.summary.parameter_hash,
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
            rplan_audit::AuditConstraint::Population,
            rplan_audit::AuditConstraint::Contiguity,
        ],
        GENERATED_AT,
        Some(lineage),
    )?;
    write_pretty_json(&certificate_path, &certificate)?;

    std::fs::write(
        &command_path,
        "cargo run -p bisect-cli --no-default-features --example spectral_path6_package\n\
         cargo run -p rplan-cli -- verify-certificate --certificate docs/examples/rplan-method-packages/T.14+spectral-generated-synthetic/audit-certificate.json --plan docs/examples/rplan-method-packages/T.14+spectral-generated-synthetic/plan.rplan --context docs/examples/rplan-method-packages/T.14+spectral-generated-synthetic/context.rctx\n\
         cargo run -p bisect-cli -- verify --manifest docs/examples/rplan-method-packages/T.14+spectral-generated-synthetic/manifest.json\n",
    )
    .with_context(|| format!("write {}", command_path.display()))?;

    let files = vec![
        manifest_file(&out_dir, "plan.rplan", "plan")?,
        manifest_file(&out_dir, "context.rctx", "context")?,
        manifest_file(&out_dir, "audit-certificate.json", "certificate")?,
        manifest_file(&out_dir, "spectral-summary.json", "construction-report")?,
        manifest_file(&out_dir, "method-transcript.json", "method-transcript")?,
        manifest_file(&out_dir, "command-transcript.txt", "command-transcript")?,
    ];
    let manifest = json!({
        "schema_version": "u20-public-example-manifest-v1",
        "example_id": PACKAGE_ID,
        "paper": PAPER,
        "package_tier": "method-produced-fixture",
        "source_workflow": "cargo run -p bisect-cli --no-default-features --example spectral_path6_package",
        "generated_at_utc": GENERATED_AT,
        "status": "partitioned",
        "files": files,
        "verification_command": "cargo run -p rplan-cli -- verify-certificate --certificate docs/examples/rplan-method-packages/T.14+spectral-generated-synthetic/audit-certificate.json --plan docs/examples/rplan-method-packages/T.14+spectral-generated-synthetic/plan.rplan --context docs/examples/rplan-method-packages/T.14+spectral-generated-synthetic/context.rctx",
        "bisect_verification_command": "cargo run -p bisect-cli -- verify --manifest docs/examples/rplan-method-packages/T.14+spectral-generated-synthetic/manifest.json",
        "scope_note": "Generated by the bisect-apportion spectral bisection routine over a deterministic synthetic path graph. The package proves construction workflow packaging, lineage, and verifier acceptance rather than real-data construction quality or benchmark performance."
    });
    write_pretty_json(&manifest_path, &manifest)?;

    println!("wrote {}", out_dir.display());
    Ok(())
}

fn default_out_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("repo root")
        .join("docs")
        .join("examples")
        .join("rplan-method-packages")
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
