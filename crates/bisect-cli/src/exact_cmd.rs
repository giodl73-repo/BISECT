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
    if report.solution.is_some() {
        write_solution_package(args, &context, &report, lineage)?;
    }
    Ok(())
}

fn write_solution_package(
    args: &ExactArgs,
    context: &rplan_core::RplanContext,
    report: &bisect_column::BranchPriceReport,
    lineage: rplan_audit::AlgorithmLineage,
) -> anyhow::Result<()> {
    let solution = report
        .solution
        .as_ref()
        .ok_or_else(|| anyhow!("branch-price report has no solved plan"))?;
    let generated_at = bisect_report::now_iso8601();
    let jurisdiction = context
        .units
        .state
        .clone()
        .unwrap_or_else(|| "US".to_string());
    let document = rplan_io::RplanDocument {
        rplan_version: rplan_io::RPLAN_V02.to_string(),
        plan: rplan_core::DistrictPlan {
            schema_version: rplan_core::DISTRICT_PLAN_SCHEMA_VERSION.to_string(),
            units: context.units.clone(),
            assignment: solution
                .assignment
                .iter()
                .map(|&district| district as u32)
                .collect(),
            k: args.districts,
            display_labels: (0..args.districts)
                .map(|district| district.to_string())
                .collect(),
            allow_empty_districts: false,
        },
        metadata: rplan_io::RplanMetadataV02 {
            label: "branch-and-price-exact".to_string(),
            jurisdiction: jurisdiction.clone(),
            chamber: "congressional".to_string(),
            created_at: generated_at.clone(),
            description: Some("U.17 branch-and-price exact fixture solution".to_string()),
        },
        provenance: rplan_io::RplanProvenance {
            producer: std::collections::BTreeMap::from([
                ("name".to_string(), serde_json::json!("bisect exact")),
                ("crate".to_string(), serde_json::json!("bisect-column")),
                ("method".to_string(), serde_json::json!("branch-and-price")),
            ]),
            source_hashes: context.source_hashes.entries.clone(),
            conversion_lineage: Vec::new(),
        },
        geometry: None,
        extensions: std::collections::BTreeMap::new(),
    };
    let profile = exact_legal_profile(
        &jurisdiction,
        &document.metadata.chamber,
        document.plan.units.year.unwrap_or(2020),
        args.tolerance,
    );
    let certificate = rplan_audit::audit_plan_with_lineage(
        &document.plan,
        Some(context),
        &profile,
        rplan_audit::RuntimeProvenance {
            binary_name: "bisect".to_string(),
            binary_version: env!("CARGO_PKG_VERSION").to_string(),
            git_commit: option_env!("GIT_COMMIT").map(str::to_string),
            build_profile: None,
            solver: Some(rplan_audit::SolverProvenance {
                name: "branch-and-price".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
                mode: Some(format!("{:?}", report.status)),
                time_limit_secs: None,
                optimality_gap: report.gap,
            }),
        },
        &[
            rplan_audit::AuditConstraint::Population,
            rplan_audit::AuditConstraint::Contiguity,
        ],
        &generated_at,
        Some(lineage),
    )?;
    if matches!(certificate.result, rplan_audit::AuditResult::Fail) {
        return Err(anyhow!("exact branch-price solution failed RPLAN audit"));
    }

    let rplan_path = args.out_dir.join("exact.rplan");
    let rctx_path = args.out_dir.join("exact.rctx");
    let certificate_path = args.out_dir.join("audit-certificate.json");
    std::fs::write(&rplan_path, rplan_io::write_rplan_string(&document)?)
        .with_context(|| format!("write {}", rplan_path.display()))?;
    std::fs::write(&rctx_path, rplan_io::write_rctx_string(context)?)
        .with_context(|| format!("write {}", rctx_path.display()))?;
    std::fs::write(
        &certificate_path,
        serde_json::to_string_pretty(&certificate).context("serialize exact audit certificate")?,
    )
    .with_context(|| format!("write {}", certificate_path.display()))?;
    write_exact_manifest(
        args,
        &document,
        &certificate,
        &profile,
        report,
        &generated_at,
    )
}

fn write_exact_manifest(
    args: &ExactArgs,
    document: &rplan_io::RplanDocument,
    certificate: &rplan_audit::AuditCertificate,
    profile: &rplan_audit::LegalProfile,
    report: &bisect_column::BranchPriceReport,
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
        partition_mode: "branch-and-price".to_string(),
        binary_version: env!("CARGO_PKG_VERSION").to_string(),
        created_at: generated_at.to_string(),
        balance_tolerance_pct: args.tolerance,
        population_balance_valid: true,
        edge_cut: report
            .solution
            .as_ref()
            .map(|solution| solution.objective as f64),
        rplan_path: Some("exact.rplan".to_string()),
        rctx_path: Some("exact.rctx".to_string()),
        audit_certificate_path: Some("audit-certificate.json".to_string()),
        audit_certificate_sha256: Some(bisect_report::sha256_file(
            &args.out_dir.join("audit-certificate.json"),
        )?),
        audit_certificate_content_hash: Some(certificate.content_hash.clone()),
        audit_result: Some(rplan_audit_result_label(&certificate.result).to_string()),
        legal_profile_id: Some(profile.profile_id.clone()),
        context_hash: certificate.context_hash.clone(),
        n_units: document.plan.assignment.len(),
        unit_type: "rplan unit".to_string(),
        ..bisect_report::PlanManifest::default()
    };
    let manifest_path = args.out_dir.join("manifest.json");
    std::fs::write(
        &manifest_path,
        serde_json::to_string_pretty(&manifest).context("serialize exact manifest")?,
    )
    .with_context(|| format!("write {}", manifest_path.display()))?;
    Ok(())
}

fn exact_legal_profile(
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
        profile_id: "BISECT_BRANCH_PRICE_EXACT_V1".to_string(),
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
                    let to = usize::try_from(edge.to).map_err(|_| {
                        anyhow!("graph edge target at unit {from} does not fit usize")
                    })?;
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

        let exact_plan_text = std::fs::read_to_string(out_dir.join("exact.rplan")).unwrap();
        let exact_context_text = std::fs::read_to_string(out_dir.join("exact.rctx")).unwrap();
        let certificate_text =
            std::fs::read_to_string(out_dir.join("audit-certificate.json")).unwrap();
        let manifest_text = std::fs::read_to_string(out_dir.join("manifest.json")).unwrap();
        let exact_plan = rplan_io::read_rplan_str(&exact_plan_text).unwrap();
        let exact_context = rplan_io::read_rctx_str(&exact_context_text).unwrap();
        let certificate: rplan_audit::AuditCertificate =
            serde_json::from_str(&certificate_text).unwrap();
        rplan_audit::verify_audit_certificate(
            &certificate,
            Some(&exact_plan.plan),
            Some(&exact_context),
        )
        .unwrap();
        let manifest: bisect_report::PlanManifest = serde_json::from_str(&manifest_text).unwrap();
        assert_eq!(manifest.rplan_path.as_deref(), Some("exact.rplan"));
        assert_eq!(
            manifest.audit_certificate_path.as_deref(),
            Some("audit-certificate.json")
        );
        assert_eq!(manifest.edge_cut, Some(1.0));
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
