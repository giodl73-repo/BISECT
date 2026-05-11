use crate::args::{ImproveArgs, ImproveMethodArg};
use anyhow::{anyhow, Context};

pub fn run_improve(args: &ImproveArgs) -> anyhow::Result<()> {
    let plan_text = std::fs::read_to_string(&args.plan)
        .with_context(|| format!("read input RPLAN {}", args.plan.display()))?;
    let context_text = std::fs::read_to_string(&args.context)
        .with_context(|| format!("read input RCTX {}", args.context.display()))?;
    let mut document = rplan_io::read_rplan_str(&plan_text)
        .with_context(|| format!("parse input RPLAN {}", args.plan.display()))?;
    let context = rplan_io::read_rctx_str(&context_text)
        .with_context(|| format!("parse input RCTX {}", args.context.display()))?;

    let graph = context
        .graph
        .as_ref()
        .ok_or_else(|| anyhow!("RCTX graph is required for local-search improvement"))?;
    let populations = context
        .populations
        .as_ref()
        .ok_or_else(|| anyhow!("RCTX populations are required for local-search improvement"))?;
    let adjacency = graph_adjacency(graph)?;
    let assignment: Vec<usize> = document
        .plan
        .assignment
        .iter()
        .map(|&district| district as usize)
        .collect();
    let tolerance = args.tolerance / 100.0;
    let method = match args.method {
        ImproveMethodArg::OneMove => bisect_local_search::LocalSearchMethod::OneMove,
    };
    let parent_plan_hash = document.plan.plan_hash()?;
    let result = bisect_local_search::improve_one_move(
        &adjacency,
        populations,
        &assignment,
        bisect_local_search::LocalSearchConfig {
            k: document.plan.k,
            tolerance,
            method,
        },
    )?;

    document.plan.assignment = result
        .assignment
        .iter()
        .map(|&district| district as u32)
        .collect();
    document.metadata.label = args
        .label
        .clone()
        .unwrap_or_else(|| format!("{}-improved", document.metadata.label));
    let generated_at = args
        .generated_at
        .clone()
        .unwrap_or_else(bisect_report::now_iso8601);
    document.metadata.created_at = generated_at.clone();
    document
        .provenance
        .producer
        .insert("name".to_string(), serde_json::json!("bisect improve"));
    document.provenance.producer.insert(
        "crate".to_string(),
        serde_json::json!("bisect-local-search"),
    );
    document.provenance.producer.insert(
        "method".to_string(),
        serde_json::json!(result.summary.method.clone()),
    );

    let summary_path = args.out_dir.join("local-search-summary.json");
    let rplan_path = args.out_dir.join("improved.rplan");
    let rctx_path = args.out_dir.join("improved.rctx");
    let certificate_path = args.out_dir.join("audit-certificate.json");
    std::fs::create_dir_all(&args.out_dir)
        .with_context(|| format!("create output directory {}", args.out_dir.display()))?;
    std::fs::write(
        &summary_path,
        serde_json::to_string_pretty(&result.summary).context("serialize local-search summary")?,
    )
    .with_context(|| format!("write {}", summary_path.display()))?;

    let mut lineage_extra = result.summary.algorithm_lineage_extra();
    if let Some(obj) = lineage_extra.as_object_mut() {
        obj.insert(
            "summary_path".to_string(),
            serde_json::json!("local-search-summary.json"),
        );
        obj.insert(
            "summary_sha256".to_string(),
            serde_json::json!(bisect_report::sha256_file(&summary_path)?),
        );
    }
    let lineage = rplan_audit::AlgorithmLineage::new(
        "bisect-local-search",
        env!("CARGO_PKG_VERSION"),
        result.summary.method.clone(),
        vec![parent_plan_hash],
        lineage_extra,
    )?;
    let year = document.plan.units.year.unwrap_or(2020);
    let profile = local_search_legal_profile(
        &document.metadata.jurisdiction,
        &document.metadata.chamber,
        year,
        args.tolerance,
    );
    let certificate = rplan_audit::audit_plan_with_lineage(
        &document.plan,
        Some(&context),
        &profile,
        rplan_audit::RuntimeProvenance {
            binary_name: "bisect".to_string(),
            binary_version: env!("CARGO_PKG_VERSION").to_string(),
            git_commit: option_env!("GIT_COMMIT").map(str::to_string),
            build_profile: None,
            solver: None,
        },
        &[
            rplan_audit::AuditConstraint::Population,
            rplan_audit::AuditConstraint::Contiguity,
        ],
        &generated_at,
        Some(lineage),
    )?;
    if matches!(certificate.result, rplan_audit::AuditResult::Fail) {
        return Err(anyhow!("improved plan failed RPLAN audit"));
    }
    let certificate_json =
        serde_json::to_string_pretty(&certificate).context("serialize audit certificate")?;

    std::fs::write(&rplan_path, rplan_io::write_rplan_string(&document)?)
        .with_context(|| format!("write {}", rplan_path.display()))?;
    std::fs::write(&rctx_path, rplan_io::write_rctx_string(&context)?)
        .with_context(|| format!("write {}", rctx_path.display()))?;
    std::fs::write(&certificate_path, certificate_json)
        .with_context(|| format!("write {}", certificate_path.display()))?;
    write_manifest(
        &args.out_dir,
        &document,
        &certificate,
        &profile,
        args.tolerance,
        &generated_at,
    )?;

    println!(
        "improve: {} -> {} ({:?})",
        result.summary.initial_edge_cut, result.summary.final_edge_cut, result.status
    );
    println!("wrote {}", rplan_path.display());
    println!("wrote {}", rctx_path.display());
    println!("wrote {}", certificate_path.display());
    Ok(())
}

fn write_manifest(
    out_dir: &std::path::Path,
    document: &rplan_io::RplanDocument,
    certificate: &rplan_audit::AuditCertificate,
    profile: &rplan_audit::LegalProfile,
    tolerance_percent: f64,
    generated_at: &str,
) -> anyhow::Result<()> {
    let manifest = bisect_report::PlanManifest {
        label: document.metadata.label.clone(),
        state_code: document
            .plan
            .units
            .state
            .clone()
            .unwrap_or_else(|| document.metadata.jurisdiction.clone()),
        year: document
            .plan
            .units
            .year
            .map(|year| year.to_string())
            .unwrap_or_default(),
        chamber: document.metadata.chamber.clone(),
        num_districts: document.plan.k,
        population_source: "rplan-context".to_string(),
        partition_mode: "local-search".to_string(),
        binary_version: env!("CARGO_PKG_VERSION").to_string(),
        created_at: generated_at.to_string(),
        balance_tolerance_pct: tolerance_percent,
        population_balance_valid: true,
        rplan_path: Some("improved.rplan".to_string()),
        rctx_path: Some("improved.rctx".to_string()),
        audit_certificate_path: Some("audit-certificate.json".to_string()),
        audit_certificate_sha256: Some(bisect_report::sha256_file(
            &out_dir.join("audit-certificate.json"),
        )?),
        audit_certificate_content_hash: Some(certificate.content_hash.clone()),
        audit_result: Some(rplan_audit_result_label(&certificate.result).to_string()),
        legal_profile_id: Some(profile.profile_id.clone()),
        context_hash: certificate.context_hash.clone(),
        n_units: document.plan.assignment.len(),
        unit_type: "rplan unit".to_string(),
        ..bisect_report::PlanManifest::default()
    };
    std::fs::write(
        out_dir.join("manifest.json"),
        serde_json::to_string_pretty(&manifest).context("serialize improve manifest")?,
    )
    .with_context(|| format!("write {}", out_dir.join("manifest.json").display()))?;
    Ok(())
}

fn rplan_audit_result_label(result: &rplan_audit::AuditResult) -> &'static str {
    match result {
        rplan_audit::AuditResult::Pass => "pass",
        rplan_audit::AuditResult::Fail => "fail",
        rplan_audit::AuditResult::PassWithWarnings => "pass-with-warnings",
    }
}

fn graph_adjacency(graph: &rplan_core::UnitGraph) -> anyhow::Result<Vec<Vec<usize>>> {
    graph
        .adjacency
        .iter()
        .enumerate()
        .map(|(from, edges)| {
            edges
                .iter()
                .map(|edge| {
                    let to = edge.to as usize;
                    if to >= graph.adjacency.len() {
                        Err(anyhow!("edge target {to} out of bounds for unit {from}"))
                    } else {
                        Ok(to)
                    }
                })
                .collect()
        })
        .collect()
}

fn local_search_legal_profile(
    jurisdiction: &str,
    chamber: &str,
    year: u16,
    tolerance_percent: f64,
) -> rplan_audit::LegalProfile {
    let chamber = match chamber {
        "congressional" => rplan_audit::Chamber::Congressional,
        other => rplan_audit::Chamber::Custom(other.to_string()),
    };
    rplan_audit::LegalProfile {
        schema_version: rplan_audit::LEGAL_PROFILE_SCHEMA_VERSION.to_string(),
        profile_id: "BISECT_LOCAL_SEARCH_IMPROVE_V1".to_string(),
        jurisdiction: jurisdiction.to_string(),
        chamber,
        year,
        population_tolerance: rplan_audit::PopulationToleranceRule::Percent {
            max_deviation_percent: tolerance_percent,
        },
        contiguity_required: true,
        county_split_rule: rplan_audit::SplitRule::CountOnly,
        municipal_split_rule: rplan_audit::SplitRule::NotEvaluated,
        nesting_rule: rplan_audit::NestingRule::NotEvaluated,
        vra_policy: rplan_audit::VraPolicy::NotEvaluated,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;
    use rplan_core::{
        CanonicalOrder, DistrictPlan, EdgeKind, EdgeSemantics, PlanUnitIndex, RplanContext,
        SourceHashes, UnitEdge, UnitGraph, UnitKind, DISTRICT_PLAN_SCHEMA_VERSION, RCTX_VERSION,
    };
    use std::collections::BTreeMap;
    use tempfile::TempDir;

    #[test]
    fn improve_args_parse_one_move() {
        let cli = crate::args::Cli::parse_from([
            "bisect",
            "improve",
            "--plan",
            "input.rplan",
            "--context",
            "input.rctx",
            "--out-dir",
            "out",
            "--method",
            "one-move",
            "--tolerance",
            "34",
        ]);

        let crate::args::Commands::Improve(args) = cli.command else {
            panic!("expected improve command");
        };
        assert_eq!(args.method, ImproveMethodArg::OneMove);
        assert_eq!(args.tolerance, 34.0);
    }

    #[test]
    fn run_improve_emits_rplan_context_summary_and_certificate() {
        let tmp = TempDir::new().unwrap();
        let input_plan = tmp.path().join("input.rplan");
        let input_context = tmp.path().join("input.rctx");
        let out_dir = tmp.path().join("out");
        let (document, context) = fixture_document_and_context();
        std::fs::write(
            &input_plan,
            rplan_io::write_rplan_string(&document).unwrap(),
        )
        .unwrap();
        std::fs::write(
            &input_context,
            rplan_io::write_rctx_string(&context).unwrap(),
        )
        .unwrap();

        run_improve(&ImproveArgs {
            plan: input_plan,
            context: input_context,
            out_dir: out_dir.clone(),
            method: ImproveMethodArg::OneMove,
            tolerance: 34.0,
            label: Some("improved-fixture".to_string()),
            generated_at: Some("2026-05-11T00:00:00Z".to_string()),
        })
        .unwrap();

        let improved_text = std::fs::read_to_string(out_dir.join("improved.rplan")).unwrap();
        let improved = rplan_io::read_rplan_str(&improved_text).unwrap();
        assert_eq!(improved.metadata.label, "improved-fixture");
        assert_eq!(improved.plan.assignment, vec![0, 0, 1, 1, 1, 1]);

        let cert_text = std::fs::read_to_string(out_dir.join("audit-certificate.json")).unwrap();
        let cert: rplan_audit::AuditCertificate = serde_json::from_str(&cert_text).unwrap();
        assert_ne!(cert.result, rplan_audit::AuditResult::Fail);
        let lineage = cert.algorithm_lineage.unwrap();
        assert_eq!(lineage.producer_crate, "bisect-local-search");
        assert_eq!(lineage.method, "one-move");
        assert_eq!(lineage.extra["summary_path"], "local-search-summary.json");

        crate::verify::run_verify(&crate::args::VerifyArgs {
            manifest: out_dir.join("manifest.json"),
            min_similarity: 0.99,
            verify_label: None,
            output_base: "outputs".to_string(),
            dry_run: true,
            skip_binary_check: true,
            label: None,
            verify_assignments_only: false,
            plan_ref: None,
        })
        .unwrap();
    }

    fn fixture_document_and_context() -> (rplan_io::RplanDocument, RplanContext) {
        let mut units = PlanUnitIndex {
            unit_kind: UnitKind::Imported,
            state: Some("ZZ".to_string()),
            year: Some(2020),
            canonical_order: CanonicalOrder::ExplicitUnitIds,
            unit_ids: (0..6).map(|idx| format!("u{idx}")).collect(),
            unit_universe_hash: String::new(),
            source_id: Some("fixture".to_string()),
        };
        units.unit_universe_hash = units.compute_unit_universe_hash().unwrap();
        let plan = DistrictPlan {
            schema_version: DISTRICT_PLAN_SCHEMA_VERSION.to_string(),
            units: units.clone(),
            assignment: vec![0, 0, 0, 1, 1, 1],
            k: 2,
            display_labels: vec!["1".to_string(), "2".to_string()],
            allow_empty_districts: false,
        };
        let graph = UnitGraph {
            edge_semantics: EdgeSemantics::Undirected,
            adjacency: vec![
                edges(&[1, 2]),
                edges(&[0]),
                edges(&[0, 3, 4, 5]),
                edges(&[2, 4]),
                edges(&[2, 3, 5]),
                edges(&[2, 4]),
            ],
        };
        let context = RplanContext {
            rctx_version: RCTX_VERSION.to_string(),
            context_hash: String::new(),
            units: units.clone(),
            graph: Some(graph),
            populations: Some(vec![100; 6]),
            subdivisions: None,
            demographics: None,
            geometry: None,
            source_hashes: SourceHashes {
                entries: BTreeMap::new(),
            },
        };
        (
            rplan_io::RplanDocument {
                rplan_version: rplan_io::RPLAN_V02.to_string(),
                plan,
                metadata: rplan_io::RplanMetadataV02 {
                    label: "fixture".to_string(),
                    jurisdiction: "ZZ".to_string(),
                    chamber: "congressional".to_string(),
                    created_at: "2026-05-11T00:00:00Z".to_string(),
                    description: None,
                },
                provenance: rplan_io::RplanProvenance::default(),
                geometry: None,
                extensions: BTreeMap::new(),
            },
            context,
        )
    }

    fn edges(targets: &[u32]) -> Vec<UnitEdge> {
        targets
            .iter()
            .map(|&to| UnitEdge {
                to,
                kind: EdgeKind::Boundary,
                weight: None,
            })
            .collect()
    }
}
