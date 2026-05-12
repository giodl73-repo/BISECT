use anyhow::{Context, Result};
use bisect_clustering::{regionalize, ClusterConfig, ClusterStatus};
use rplan_core::{
    canonical_sha256, CanonicalOrder, DistrictPlan, EdgeKind, EdgeSemantics, PlanUnitIndex,
    RplanContext, SourceHashes, UnitEdge, UnitGraph, UnitKind, DISTRICT_PLAN_SCHEMA_VERSION,
    RCTX_VERSION,
};
use rplan_io::{RplanDocument, RplanMetadataV02, RplanProvenance, RPLAN_V02};
use serde_json::json;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

const PACKAGE_ID: &str = "T.16+regionalization-path100-benchmark";
const PAPER: &str = "T.16+hierarchical-regionalization";
const GENERATED_AT: &str = "2026-05-11T00:00:00Z";
const UNIT_COUNT: usize = 100;

fn main() -> Result<()> {
    let out_dir = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(default_out_dir);
    std::fs::create_dir_all(&out_dir)
        .with_context(|| format!("create package directory {}", out_dir.display()))?;

    let adjacency = path_adjacency(UNIT_COUNT);
    let populations = vec![100; UNIT_COUNT];
    let config = ClusterConfig {
        k: 2,
        tolerance: 0.01,
    };
    let result = regionalize(&adjacency, &populations, config)
        .map_err(|err| anyhow::anyhow!("construct regionalization benchmark plan: {err}"))?;
    if result.status != ClusterStatus::Valid {
        anyhow::bail!(
            "regionalization benchmark generator produced non-valid status: {:?}",
            result.status
        );
    }

    let source_fixture = json!({
        "schema_version": "bisect-clustering-synthetic-source-v1",
        "fixture_id": "path100-regionalization-synthetic",
        "unit_count": populations.len(),
        "edge_count_undirected": undirected_edge_count(&adjacency),
        "populations": populations,
        "description": "One hundred tract-shaped synthetic units on a path graph with equal populations.",
    });
    let source_fixture_hash = canonical_sha256(&source_fixture)?;

    let mut units = PlanUnitIndex {
        unit_kind: UnitKind::Tract,
        state: Some("WA".to_string()),
        year: Some(2020),
        canonical_order: CanonicalOrder::ExplicitUnitIds,
        unit_ids: (1..=populations.len())
            .map(|idx| format!("53016{:06}", idx * 100))
            .collect(),
        unit_universe_hash: String::new(),
        source_id: Some("path100-regionalization-synthetic".to_string()),
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
                "Synthetic 100-unit path constructed by bisect-clustering regionalization."
                    .to_string(),
            ),
        },
        provenance: RplanProvenance {
            producer: BTreeMap::from([
                ("crate".to_string(), json!("bisect-cli")),
                ("method_crate".to_string(), json!("bisect-clustering")),
                ("method".to_string(), json!("hierarchical-regionalization")),
            ]),
            source_hashes: BTreeMap::from([("fixture".to_string(), source_fixture_hash)]),
            conversion_lineage: vec![json!({
                "workflow": "cargo run -p bisect-cli --no-default-features --example regionalization_path100_benchmark_package",
                "package_tier": "benchmark-package",
            })],
        },
        geometry: None,
        extensions: BTreeMap::new(),
    };

    let summary_path = out_dir.join("regionalization-summary.json");
    let merge_log_path = out_dir.join("merge-log.json");
    let transcript_path = out_dir.join("method-transcript.json");
    let timing_path = out_dir.join("benchmark-notes.json");
    let plan_path = out_dir.join("plan.rplan");
    let context_path = out_dir.join("context.rctx");
    let certificate_path = out_dir.join("audit-certificate.json");
    let command_path = out_dir.join("command-transcript.txt");
    let manifest_path = out_dir.join("manifest.json");

    write_pretty_json(&summary_path, &result.summary)?;
    let summary_sha256 = bisect_report::sha256_file(&summary_path)?;
    write_pretty_json(&merge_log_path, &result.merge_log)?;
    let merge_log_sha256 = bisect_report::sha256_file(&merge_log_path)?;

    std::fs::write(&plan_path, rplan_io::write_rplan_string(&document)?)
        .with_context(|| format!("write {}", plan_path.display()))?;
    std::fs::write(&context_path, rplan_io::write_rctx_string(&context)?)
        .with_context(|| format!("write {}", context_path.display()))?;

    let plan_hash = plan.plan_hash()?;
    let transcript = json!({
        "schema_version": "method-transcript-v1",
        "package_id": PACKAGE_ID,
        "package_tier": "benchmark-package",
        "paper": PAPER,
        "producer_crate": "bisect-clustering",
        "producer_version": env!("CARGO_PKG_VERSION"),
        "method": "hierarchical-regionalization",
        "source_workflow": "cargo run -p bisect-cli --no-default-features --example regionalization_path100_benchmark_package",
        "status": result.summary.capacity_status,
        "input_fixture": "path100-regionalization-synthetic",
        "unit_count": populations.len(),
        "edge_count_undirected": undirected_edge_count(&adjacency),
        "plan_hash": plan_hash,
        "context_hash": context.context_hash,
        "unit_universe_hash": units.unit_universe_hash,
        "summary_path": "regionalization-summary.json",
        "summary_sha256": summary_sha256,
        "merge_log_path": "merge-log.json",
        "merge_log_sha256": merge_log_sha256,
        "assignment": result.assignment,
        "summary": result.summary,
        "scope_note": "Synthetic benchmark-tier package for hierarchical-regionalization lineage and verifier acceptance. It is not real-data quality, legal sufficiency, or wall-clock performance evidence."
    });
    write_pretty_json(&transcript_path, &transcript)?;
    let transcript_sha256 = bisect_report::sha256_file(&transcript_path)?;

    let timing_notes = json!({
        "schema_version": "benchmark-timing-notes-v1",
        "package_id": PACKAGE_ID,
        "timing_kind": "deterministic-construction-scale-smoke",
        "unit_count": populations.len(),
        "edge_count_undirected": undirected_edge_count(&adjacency),
        "reference_hardware": {
            "cpu": "not pinned",
            "core_count": "not pinned",
            "ram": "not pinned",
            "os": "Windows development workstation"
        },
        "measurement_protocol": [
            "cargo run -p bisect-cli --no-default-features --example regionalization_path100_benchmark_package",
            "cargo run -p rplan-cli -- verify-certificate --certificate docs/examples/rplan-benchmark-packages/T.16+regionalization-path100-benchmark/audit-certificate.json --plan docs/examples/rplan-benchmark-packages/T.16+regionalization-path100-benchmark/plan.rplan --context docs/examples/rplan-benchmark-packages/T.16+regionalization-path100-benchmark/context.rctx",
            "cargo run -p bisect-cli -- verify --manifest docs/examples/rplan-benchmark-packages/T.16+regionalization-path100-benchmark/manifest.json"
        ],
        "wall_clock_claim": "none",
        "scope_note": "This benchmark package records benchmark protocol and scale metadata. It is a verifier-scale smoke benchmark, not a formal runtime claim."
    });
    write_pretty_json(&timing_path, &timing_notes)?;
    let timing_sha256 = bisect_report::sha256_file(&timing_path)?;

    let mut lineage = result
        .summary
        .algorithm_lineage(env!("CARGO_PKG_VERSION"), Vec::new())?;
    lineage.method = "hierarchical-regionalization".to_string();
    let mut extra = lineage.extra;
    extra["package_id"] = json!(PACKAGE_ID);
    extra["input_fixture"] = json!("path100-regionalization-synthetic");
    extra["summary_path"] = json!("regionalization-summary.json");
    extra["summary_sha256"] = json!(summary_sha256);
    extra["merge_log_path"] = json!("merge-log.json");
    extra["merge_log_sha256"] = json!(merge_log_sha256);
    extra["method_transcript_path"] = json!("method-transcript.json");
    extra["method_transcript_sha256"] = json!(transcript_sha256);
    extra["benchmark_notes_path"] = json!("benchmark-notes.json");
    extra["benchmark_notes_sha256"] = json!(timing_sha256);
    lineage.extra = extra;

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
    if certificate.result == rplan_audit::AuditResult::Fail {
        anyhow::bail!(
            "regionalization benchmark package audit failed; refusing to write public benchmark package"
        );
    }
    write_pretty_json(&certificate_path, &certificate)?;

    std::fs::write(
        &command_path,
        "cargo run -p bisect-cli --no-default-features --example regionalization_path100_benchmark_package\n\
         cargo run -p rplan-cli -- verify-certificate --certificate docs/examples/rplan-benchmark-packages/T.16+regionalization-path100-benchmark/audit-certificate.json --plan docs/examples/rplan-benchmark-packages/T.16+regionalization-path100-benchmark/plan.rplan --context docs/examples/rplan-benchmark-packages/T.16+regionalization-path100-benchmark/context.rctx\n\
         cargo run -p bisect-cli -- verify --manifest docs/examples/rplan-benchmark-packages/T.16+regionalization-path100-benchmark/manifest.json\n",
    )
    .with_context(|| format!("write {}", command_path.display()))?;

    let files = vec![
        manifest_file(&out_dir, "plan.rplan", "plan")?,
        manifest_file(&out_dir, "context.rctx", "context")?,
        manifest_file(&out_dir, "audit-certificate.json", "certificate")?,
        manifest_file(
            &out_dir,
            "regionalization-summary.json",
            "construction-report",
        )?,
        manifest_file(&out_dir, "merge-log.json", "merge-log")?,
        manifest_file(&out_dir, "method-transcript.json", "method-transcript")?,
        manifest_file(&out_dir, "benchmark-notes.json", "benchmark-notes")?,
        manifest_file(&out_dir, "command-transcript.txt", "command-transcript")?,
    ];
    let manifest = json!({
        "schema_version": "benchmark-rplan-package-manifest-v1",
        "example_id": PACKAGE_ID,
        "paper": PAPER,
        "package_tier": "benchmark-package",
        "source_workflow": "cargo run -p bisect-cli --no-default-features --example regionalization_path100_benchmark_package",
        "generated_at_utc": GENERATED_AT,
        "status": "constructed",
        "benchmark_scope": "deterministic synthetic hierarchical-regionalization scale-smoke package",
        "data_requirements": "none; synthetic 100-unit path graph",
        "repository_footprint": "small in-repo package",
        "files": files,
        "verification_command": "cargo run -p rplan-cli -- verify-certificate --certificate docs/examples/rplan-benchmark-packages/T.16+regionalization-path100-benchmark/audit-certificate.json --plan docs/examples/rplan-benchmark-packages/T.16+regionalization-path100-benchmark/plan.rplan --context docs/examples/rplan-benchmark-packages/T.16+regionalization-path100-benchmark/context.rctx",
        "bisect_verification_command": "cargo run -p bisect-cli -- verify --manifest docs/examples/rplan-benchmark-packages/T.16+regionalization-path100-benchmark/manifest.json",
        "scope_note": "Generated by the bisect-clustering regionalization routine over a deterministic 100-unit synthetic path. The package proves benchmark-tier packaging, construction lineage, merge-log artifact capture, and verifier acceptance, not real-data construction quality or wall-clock performance."
    });
    write_pretty_json(&manifest_path, &manifest)?;

    println!("wrote {}", out_dir.display());
    Ok(())
}

fn path_adjacency(count: usize) -> Vec<Vec<usize>> {
    let mut adjacency = vec![Vec::new(); count];
    for idx in 0..count {
        if idx > 0 {
            adjacency[idx].push(idx - 1);
        }
        if idx + 1 < count {
            adjacency[idx].push(idx + 1);
        }
    }
    adjacency
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
