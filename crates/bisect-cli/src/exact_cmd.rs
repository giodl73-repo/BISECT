use crate::args::{DiscoveryRefinementArg, ExactArgs, ExactMethodArg};
use anyhow::{anyhow, Context};
use serde::Serialize;
use std::collections::{BTreeMap, BinaryHeap, HashMap, HashSet, VecDeque};

pub fn run_exact(args: &ExactArgs) -> anyhow::Result<()> {
    match args.method {
        ExactMethodArg::CanonicalExhaustive => run_canonical_exhaustive(args),
        ExactMethodArg::CertifiedRecursive => run_certified_recursive(args),
        ExactMethodArg::CertifiedDiscovery => run_certified_discovery(args),
        ExactMethodArg::BranchAndPrice => run_branch_and_price(args),
    }
}

fn run_certified_discovery(args: &ExactArgs) -> anyhow::Result<()> {
    if args.districts < 2 {
        return Err(anyhow!(
            "certified-discovery currently requires --districts >= 2"
        ));
    }
    let context_text = std::fs::read_to_string(&args.context)
        .with_context(|| format!("read input RCTX {}", args.context.display()))?;
    let context = rplan_io::read_rctx_str(&context_text)
        .with_context(|| format!("parse input RCTX {}", args.context.display()))?;
    let populations = context
        .populations
        .as_ref()
        .ok_or_else(|| anyhow!("RCTX populations are required for certified discovery"))?;
    let root = certified_root_instance_from_context(&context, args.districts)?;
    let adjacency = graph_adjacency(
        context
            .graph
            .as_ref()
            .ok_or_else(|| anyhow!("RCTX graph is required for certified discovery"))?,
    )?;
    let land_adjacency = context
        .graph
        .as_ref()
        .expect("certified discovery graph checked above")
        .adjacency
        .iter()
        .map(|edges| {
            edges
                .iter()
                .filter_map(|edge| {
                    (!matches!(edge.kind, rplan_core::EdgeKind::Bridge)).then_some(edge.to as usize)
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    let discovery=bisect_runner::certified_discovery::discover(&root,&adjacency,&land_adjacency,args.discovery_seed,args.discovery_refinement)?;
    std::fs::create_dir_all(&args.out_dir)
        .with_context(|| format!("create output directory {}", args.out_dir.display()))?;
    std::fs::write(
        args.out_dir.join("certified-split-instance.json"),
        serde_json::to_string_pretty(&root)?,
    )?;
    std::fs::write(
        args.out_dir.join("certified-discovery.json"),
        serde_json::to_string_pretty(&discovery)?,
    )?;
    if args.districts == 2 {
        write_discovery_solution_package(
            args,
            &context,
            &discovery.objective.canonical_assignment,
        )?;
    }
    write_discovery_manifest(args, &root, &discovery)
}

// Exercise the exact shared helper source in the existing detailed CLI regression tests.
#[cfg(test)]
include!("../../bisect-runner/src/certified_discovery_helpers.rs");

fn run_certified_recursive(args: &ExactArgs) -> anyhow::Result<()> {
    if args.districts < 2 {
        return Err(anyhow!(
            "certified-recursive currently requires --districts >= 2"
        ));
    }
    let context_text = std::fs::read_to_string(&args.context)
        .with_context(|| format!("read input RCTX {}", args.context.display()))?;
    let context = rplan_io::read_rctx_str(&context_text)
        .with_context(|| format!("parse input RCTX {}", args.context.display()))?;
    if context.units.unit_ids.len() > args.exact_fixture_limit {
        return Err(anyhow!(
            "certified recursive instance has {} units, above --exact-fixture-limit {}",
            context.units.unit_ids.len(),
            args.exact_fixture_limit
        ));
    }
    let root = certified_root_instance_from_context(&context, args.districts)?;
    let tree = bisect_ilp::solve_certified_bisection_tree_bounded(root)?;
    bisect_ilp::verify_certified_bisection_tree_bounded(&tree)?;
    let assignment = recursive_tree_assignment(&context, &tree)?;

    std::fs::create_dir_all(&args.out_dir)
        .with_context(|| format!("create output directory {}", args.out_dir.display()))?;
    let tree_path = args.out_dir.join("certified-bisection-tree.json");
    std::fs::write(&tree_path, serde_json::to_string_pretty(&tree)?)
        .with_context(|| format!("write {}", tree_path.display()))?;
    write_recursive_solution_package(args, &context, &assignment, &tree)?;
    write_recursive_package_manifest(args, &tree)
}

fn certified_root_instance_from_context(
    context: &rplan_core::RplanContext,
    districts: usize,
) -> anyhow::Result<bisect_ilp::CertifiedSplitInstance> {
    let populations = context
        .populations
        .as_ref()
        .ok_or_else(|| anyhow!("RCTX populations are required for certified splitting"))?;
    let (k_left, k_right) = bisect_ilp::canonical_seat_split(districts)?;
    let unit_ids = context.units.unit_ids.clone();
    Ok(bisect_ilp::CertifiedSplitInstance {
        schema_version: bisect_ilp::CERTIFIED_SPLIT_INSTANCE_SCHEMA_VERSION.to_string(),
        model_id: bisect_ilp::CERTIFIED_SPLIT_MODEL_ID.to_string(),
        node_path: String::new(),
        parent_certificate_id: None,
        unit_universe_hash: bisect_ilp::certified_split_unit_universe_hash(&unit_ids)?,
        unit_ids,
        populations: populations.clone(),
        edges: exact_edges_from_context(context, "certified split")?,
        k_parent: districts,
        k_left,
        k_right,
        orientation_rule: bisect_ilp::canonical_orientation_rule(k_left, k_right),
    })
}

#[derive(Serialize)]
struct CertifiedDiscoveryManifest {
    schema_version: String,
    method: String,
    status: String,
    instance_hash: String,
    discovery_id: String,
    seed: u64,
    files: BTreeMap<String, String>,
    claim_boundary: String,
}

fn write_discovery_manifest(
    args: &ExactArgs,
    instance: &bisect_ilp::CertifiedSplitInstance,
    discovery: &bisect_ilp::CertifiedSplitDiscovery,
) -> anyhow::Result<()> {
    let mut files = BTreeMap::new();
    for name in [
        "audit-certificate.json",
        "certified-discovery.json",
        "certified-split-instance.json",
        "discovery.rctx",
        "discovery.rplan",
    ] {
        let path = args.out_dir.join(name);
        if path.is_file() {
            files.insert(name.to_string(), bisect_report::sha256_file(&path)?);
        }
    }
    let manifest = CertifiedDiscoveryManifest {
        schema_version: "certified-split-discovery-package-v1".to_string(),
        method: "certified-discovery".to_string(),
        status: "unproved-incumbent".to_string(),
        instance_hash: instance.hash()?,
        discovery_id: discovery.discovery_id.clone(),
        seed: args.discovery_seed,
        files,
        claim_boundary: "Deterministic connected METIS incumbent and exact objective record; not an optimality proof or certified split.".to_string(),
    };
    std::fs::write(
        args.out_dir.join("certified-discovery-manifest.json"),
        serde_json::to_string_pretty(&manifest)?,
    )?;
    Ok(())
}

fn write_discovery_solution_package(
    args: &ExactArgs,
    context: &rplan_core::RplanContext,
    assignment: &[u8],
) -> anyhow::Result<()> {
    let generated_at = args
        .generated_at
        .clone()
        .unwrap_or_else(bisect_report::now_iso8601);
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
            assignment: assignment.iter().map(|&label| u32::from(label)).collect(),
            k: args.districts,
            display_labels: (1..=args.districts)
                .map(|district| district.to_string())
                .collect(),
            allow_empty_districts: false,
        },
        metadata: rplan_io::RplanMetadataV02 {
            label: "certified-discovery-incumbent".to_string(),
            jurisdiction: jurisdiction.clone(),
            chamber: "congressional".to_string(),
            created_at: generated_at.clone(),
            description: Some(
                "Connected deterministic discovery incumbent; exact proof status is separate."
                    .to_string(),
            ),
        },
        provenance: rplan_io::RplanProvenance {
            producer: BTreeMap::from([
                ("name".to_string(), serde_json::json!("bisect exact")),
                (
                    "method".to_string(),
                    serde_json::json!("certified-discovery"),
                ),
            ]),
            source_hashes: context.source_hashes.entries.clone(),
            conversion_lineage: Vec::new(),
        },
        geometry: None,
        extensions: BTreeMap::new(),
    };
    let profile = exact_legal_profile(
        &jurisdiction,
        &document.metadata.chamber,
        document.plan.units.year.unwrap_or(2020),
        args.tolerance,
    );
    let certificate = rplan_audit::audit_plan(
        &document.plan,
        Some(context),
        &profile,
        rplan_audit::RuntimeProvenance {
            binary_name: "bisect".to_string(),
            binary_version: env!("CARGO_PKG_VERSION").to_string(),
            git_commit: option_env!("GIT_COMMIT").map(str::to_string),
            build_profile: None,
            solver: Some(rplan_audit::SolverProvenance {
                name: "METIS discovery".to_string(),
                version: None,
                mode: Some("unproved-incumbent".to_string()),
                time_limit_secs: None,
                optimality_gap: None,
            }),
        },
        &[rplan_audit::AuditConstraint::Contiguity],
        &generated_at,
    )?;
    if matches!(certificate.result, rplan_audit::AuditResult::Fail) {
        return Err(anyhow!(
            "certified discovery failed contiguity audit: {}",
            serde_json::to_string(&certificate.checks)?
        ));
    }
    std::fs::write(
        args.out_dir.join("discovery.rplan"),
        rplan_io::write_rplan_string(&document)?,
    )?;
    std::fs::write(
        args.out_dir.join("discovery.rctx"),
        rplan_io::write_rctx_string(context)?,
    )?;
    std::fs::write(
        args.out_dir.join("audit-certificate.json"),
        serde_json::to_string_pretty(&certificate)?,
    )?;
    Ok(())
}

fn run_canonical_exhaustive(args: &ExactArgs) -> anyhow::Result<()> {
    if args.districts != 2 {
        return Err(anyhow!(
            "canonical-exhaustive currently requires --districts 2"
        ));
    }
    let context_text = std::fs::read_to_string(&args.context)
        .with_context(|| format!("read input RCTX {}", args.context.display()))?;
    let context = rplan_io::read_rctx_str(&context_text)
        .with_context(|| format!("parse input RCTX {}", args.context.display()))?;
    let instance = exact_instance_from_context(&context, args)?;
    let artifacts = bisect_ilp::solve_exact_canonical_artifacts(&instance)?;
    bisect_ilp::verify_exact_canonical_artifacts(
        &instance,
        &artifacts.certificate,
        &artifacts.proof,
    )?;
    let certificate = artifacts.certificate;

    std::fs::create_dir_all(&args.out_dir)
        .with_context(|| format!("create output directory {}", args.out_dir.display()))?;
    let instance_path = args.out_dir.join("exact-canonical-instance.json");
    let certificate_path = args.out_dir.join("exact-canonical-certificate.json");
    let proof_path = args.out_dir.join("exact-canonical-proof.json");
    std::fs::write(
        &instance_path,
        serde_json::to_string_pretty(&instance).context("serialize exact instance")?,
    )
    .with_context(|| format!("write {}", instance_path.display()))?;
    std::fs::write(
        &certificate_path,
        serde_json::to_string_pretty(&certificate).context("serialize exact certificate")?,
    )
    .with_context(|| format!("write {}", certificate_path.display()))?;
    std::fs::write(
        &proof_path,
        serde_json::to_string_pretty(&artifacts.proof).context("serialize exact proof")?,
    )
    .with_context(|| format!("write {}", proof_path.display()))?;

    if let bisect_ilp::ExactCertificateResult::Optimal { assignment, .. } = &certificate.result {
        write_canonical_solution_package(args, &context, assignment, &certificate)?;
    }
    write_canonical_package_manifest(args, &certificate)
}

fn exact_instance_from_context(
    context: &rplan_core::RplanContext,
    args: &ExactArgs,
) -> anyhow::Result<bisect_ilp::ExactCanonicalInstance> {
    let populations = context
        .populations
        .as_ref()
        .ok_or_else(|| anyhow!("RCTX populations are required for canonical exact solving"))?;
    if context.units.unit_ids.len() > args.exact_fixture_limit {
        return Err(anyhow!(
            "canonical exact instance has {} units, above --exact-fixture-limit {}",
            context.units.unit_ids.len(),
            args.exact_fixture_limit
        ));
    }
    Ok(bisect_ilp::ExactCanonicalInstance {
        schema_version: bisect_ilp::EXACT_INSTANCE_SCHEMA_VERSION.to_string(),
        model_id: bisect_ilp::EXACT_MODEL_ID.to_string(),
        unit_ids: context.units.unit_ids.clone(),
        populations: populations.clone(),
        edges: exact_edges_from_context(context, "canonical exact")?,
        k: args.districts,
    })
}

fn exact_edges_from_context(
    context: &rplan_core::RplanContext,
    purpose: &str,
) -> anyhow::Result<Vec<bisect_ilp::ExactEdge>> {
    let graph = context
        .graph
        .as_ref()
        .ok_or_else(|| anyhow!("RCTX graph is required for {purpose} solving"))?;
    let mut edges = BTreeMap::new();
    for (left, neighbors) in graph.adjacency.iter().enumerate() {
        for edge in neighbors {
            let right =
                usize::try_from(edge.to).map_err(|_| anyhow!("edge target does not fit usize"))?;
            if left >= right {
                continue;
            }
            let weight = match edge.weight {
                None => 1_u64,
                Some(value)
                    if value.is_finite()
                        && value > 0.0
                        && (value - value.round()).abs() < 1e-9
                        && value <= u64::MAX as f64 =>
                {
                    value as u64
                }
                Some(value) => {
                    return Err(anyhow!(
                        "{purpose} edge ({left}, {right}) has non-positive or non-integer weight {value}"
                    ));
                }
            };
            edges.insert(
                (left, right),
                bisect_ilp::ExactEdge {
                    left,
                    right,
                    weight,
                },
            );
        }
    }
    Ok(edges.into_values().collect())
}

fn write_canonical_solution_package(
    args: &ExactArgs,
    context: &rplan_core::RplanContext,
    assignment: &[u8],
    exact_certificate: &bisect_ilp::ExactCanonicalCertificate,
) -> anyhow::Result<()> {
    let generated_at = args
        .generated_at
        .clone()
        .unwrap_or_else(bisect_report::now_iso8601);
    let jurisdiction = context
        .units
        .state
        .clone()
        .unwrap_or_else(|| "XX".to_string());
    let document = rplan_io::RplanDocument {
        rplan_version: rplan_io::RPLAN_V02.to_string(),
        plan: rplan_core::DistrictPlan {
            schema_version: rplan_core::DISTRICT_PLAN_SCHEMA_VERSION.to_string(),
            units: context.units.clone(),
            assignment: assignment.iter().map(|&district| district as u32).collect(),
            k: args.districts,
            display_labels: (1..=args.districts)
                .map(|district| district.to_string())
                .collect(),
            allow_empty_districts: false,
        },
        metadata: rplan_io::RplanMetadataV02 {
            label: "exact-canonical".to_string(),
            jurisdiction: jurisdiction.clone(),
            chamber: "congressional".to_string(),
            created_at: generated_at.clone(),
            description: Some("E0 bounded exact canonical benchmark solution".to_string()),
        },
        provenance: rplan_io::RplanProvenance {
            producer: BTreeMap::from([
                ("name".to_string(), serde_json::json!("bisect exact")),
                ("crate".to_string(), serde_json::json!("bisect-ilp")),
                (
                    "method".to_string(),
                    serde_json::json!("canonical-exhaustive"),
                ),
                (
                    "exact_certificate_id".to_string(),
                    serde_json::json!(exact_certificate.certificate_id),
                ),
            ]),
            source_hashes: context.source_hashes.entries.clone(),
            conversion_lineage: Vec::new(),
        },
        geometry: None,
        extensions: BTreeMap::new(),
    };
    let profile = rplan_audit::LegalProfile {
        schema_version: rplan_audit::LEGAL_PROFILE_SCHEMA_VERSION.to_string(),
        profile_id: "EXACT_CANONICAL_E0_MODEL_V1".to_string(),
        jurisdiction,
        chamber: rplan_audit::Chamber::Congressional,
        year: document.plan.units.year.unwrap_or(2020),
        population_tolerance: rplan_audit::PopulationToleranceRule::Percent {
            max_deviation_percent: args.tolerance,
        },
        contiguity_required: true,
        county_split_rule: rplan_audit::SplitRule::NotEvaluated,
        municipal_split_rule: rplan_audit::SplitRule::NotEvaluated,
        nesting_rule: rplan_audit::NestingRule::NotEvaluated,
        vra_policy: rplan_audit::VraPolicy::NotEvaluated,
    };
    let lineage = rplan_audit::AlgorithmLineage::new(
        "bisect-ilp",
        env!("CARGO_PKG_VERSION"),
        "canonical-exhaustive",
        Vec::new(),
        serde_json::json!({
            "exact_certificate_id": exact_certificate.certificate_id,
            "instance_hash": exact_certificate.instance_hash,
            "model_id": exact_certificate.model_id,
        }),
    )?;
    let audit = rplan_audit::audit_plan_with_lineage(
        &document.plan,
        Some(context),
        &profile,
        rplan_audit::RuntimeProvenance {
            binary_name: "bisect".to_string(),
            binary_version: env!("CARGO_PKG_VERSION").to_string(),
            git_commit: option_env!("GIT_COMMIT").map(str::to_string),
            build_profile: None,
            solver: Some(rplan_audit::SolverProvenance {
                name: "bounded-exhaustive-enumeration".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
                mode: Some("exact-canonical-k2".to_string()),
                time_limit_secs: None,
                optimality_gap: Some(0.0),
            }),
        },
        &[
            rplan_audit::AuditConstraint::PlanShape,
            rplan_audit::AuditConstraint::Population,
            rplan_audit::AuditConstraint::Contiguity,
        ],
        &generated_at,
        Some(lineage),
    )?;
    if matches!(audit.result, rplan_audit::AuditResult::Fail) {
        return Err(anyhow!("exact canonical solution failed RPLAN audit"));
    }
    std::fs::write(
        args.out_dir.join("exact.rplan"),
        rplan_io::write_rplan_string(&document)?,
    )?;
    std::fs::write(
        args.out_dir.join("exact.rctx"),
        rplan_io::write_rctx_string(context)?,
    )?;
    std::fs::write(
        args.out_dir.join("audit-certificate.json"),
        serde_json::to_string_pretty(&audit)?,
    )?;
    Ok(())
}

#[derive(Serialize)]
struct ExactPackageManifest {
    schema_version: String,
    method: String,
    result: String,
    instance_hash: String,
    exact_certificate_id: String,
    exact_proof_id: String,
    declared_population_tolerance_percent: f64,
    files: BTreeMap<String, String>,
    claim_boundary: String,
}

fn write_canonical_package_manifest(
    args: &ExactArgs,
    certificate: &bisect_ilp::ExactCanonicalCertificate,
) -> anyhow::Result<()> {
    let result = match &certificate.result {
        bisect_ilp::ExactCertificateResult::Optimal { .. } => "optimal",
        bisect_ilp::ExactCertificateResult::Infeasible => "infeasible",
    };
    let mut files = BTreeMap::new();
    for name in [
        "exact-canonical-instance.json",
        "exact-canonical-certificate.json",
        "exact-canonical-proof.json",
        "exact.rplan",
        "exact.rctx",
        "audit-certificate.json",
    ] {
        let path = args.out_dir.join(name);
        if path.is_file() {
            files.insert(name.to_string(), bisect_report::sha256_file(&path)?);
        }
    }
    let manifest = ExactPackageManifest {
        schema_version: "exact-canonical-package-manifest-v1".to_string(),
        method: "canonical-exhaustive".to_string(),
        result: result.to_string(),
        instance_hash: certificate.instance_hash.clone(),
        exact_certificate_id: certificate.certificate_id.clone(),
        exact_proof_id: certificate.proof.transcript_id.clone(),
        declared_population_tolerance_percent: args.tolerance,
        files,
        claim_boundary: format!(
            "Bounded E0 k=2 exhaustive certificate with at most {} units; not national exact readiness or legal certification.",
            args.exact_fixture_limit
        ),
    };
    std::fs::write(
        args.out_dir.join("exact-package-manifest.json"),
        serde_json::to_string_pretty(&manifest)?,
    )?;
    Ok(())
}

fn recursive_tree_assignment(
    context: &rplan_core::RplanContext,
    tree: &bisect_ilp::CertifiedBisectionTree,
) -> anyhow::Result<Vec<u32>> {
    let index = context
        .units
        .unit_ids
        .iter()
        .enumerate()
        .map(|(index, unit_id)| (unit_id.as_str(), index))
        .collect::<BTreeMap<_, _>>();
    let mut assignment = vec![u32::MAX; context.units.unit_ids.len()];
    for leaf in &tree.leaves {
        let district = u32::try_from(leaf.district_index)
            .map_err(|_| anyhow!("recursive district index does not fit u32"))?;
        for unit_id in &leaf.unit_ids {
            let unit_index = *index
                .get(unit_id.as_str())
                .ok_or_else(|| anyhow!("tree leaf references unknown unit {unit_id}"))?;
            if assignment[unit_index] != u32::MAX {
                return Err(anyhow!("tree leaf assigns unit {unit_id} more than once"));
            }
            assignment[unit_index] = district;
        }
    }
    if assignment.contains(&u32::MAX) {
        return Err(anyhow!("recursive tree does not assign every context unit"));
    }
    Ok(assignment)
}

fn write_recursive_solution_package(
    args: &ExactArgs,
    context: &rplan_core::RplanContext,
    assignment: &[u32],
    tree: &bisect_ilp::CertifiedBisectionTree,
) -> anyhow::Result<()> {
    let generated_at = args
        .generated_at
        .clone()
        .unwrap_or_else(bisect_report::now_iso8601);
    let jurisdiction = context
        .units
        .state
        .clone()
        .unwrap_or_else(|| "XX".to_string());
    let document = rplan_io::RplanDocument {
        rplan_version: rplan_io::RPLAN_V02.to_string(),
        plan: rplan_core::DistrictPlan {
            schema_version: rplan_core::DISTRICT_PLAN_SCHEMA_VERSION.to_string(),
            units: context.units.clone(),
            assignment: assignment.to_vec(),
            k: args.districts,
            display_labels: (1..=args.districts)
                .map(|district| district.to_string())
                .collect(),
            allow_empty_districts: false,
        },
        metadata: rplan_io::RplanMetadataV02 {
            label: "certified-recursive-bisection".to_string(),
            jurisdiction: jurisdiction.clone(),
            chamber: "congressional".to_string(),
            created_at: generated_at.clone(),
            description: Some(
                "Bounded exact certification of the standard-bisect recursive tree".to_string(),
            ),
        },
        provenance: rplan_io::RplanProvenance {
            producer: BTreeMap::from([
                ("name".to_string(), serde_json::json!("bisect exact")),
                ("crate".to_string(), serde_json::json!("bisect-ilp")),
                (
                    "method".to_string(),
                    serde_json::json!("certified-recursive"),
                ),
                (
                    "certified_tree_id".to_string(),
                    serde_json::json!(tree.tree_id),
                ),
            ]),
            source_hashes: context.source_hashes.entries.clone(),
            conversion_lineage: Vec::new(),
        },
        geometry: None,
        extensions: BTreeMap::new(),
    };
    let profile = rplan_audit::LegalProfile {
        schema_version: rplan_audit::LEGAL_PROFILE_SCHEMA_VERSION.to_string(),
        profile_id: "CERTIFIED_RECURSIVE_BISECTION_BOUNDED_V1".to_string(),
        jurisdiction,
        chamber: rplan_audit::Chamber::Congressional,
        year: document.plan.units.year.unwrap_or(2020),
        population_tolerance: rplan_audit::PopulationToleranceRule::Percent {
            max_deviation_percent: args.tolerance,
        },
        contiguity_required: true,
        county_split_rule: rplan_audit::SplitRule::NotEvaluated,
        municipal_split_rule: rplan_audit::SplitRule::NotEvaluated,
        nesting_rule: rplan_audit::NestingRule::NotEvaluated,
        vra_policy: rplan_audit::VraPolicy::NotEvaluated,
    };
    let lineage = rplan_audit::AlgorithmLineage::new(
        "bisect-ilp",
        env!("CARGO_PKG_VERSION"),
        "certified-recursive",
        Vec::new(),
        serde_json::json!({
            "certified_tree_id": tree.tree_id,
            "root_unit_universe_hash": tree.root_unit_universe_hash,
            "split_count": tree.nodes.len(),
            "leaf_count": tree.leaves.len(),
        }),
    )?;
    let audit = rplan_audit::audit_plan_with_lineage(
        &document.plan,
        Some(context),
        &profile,
        rplan_audit::RuntimeProvenance {
            binary_name: "bisect".to_string(),
            binary_version: env!("CARGO_PKG_VERSION").to_string(),
            git_commit: option_env!("GIT_COMMIT").map(str::to_string),
            build_profile: None,
            solver: Some(rplan_audit::SolverProvenance {
                name: "bounded-certified-recursive-enumeration".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
                mode: Some("standard-bisect-tree".to_string()),
                time_limit_secs: None,
                optimality_gap: Some(0.0),
            }),
        },
        &[
            rplan_audit::AuditConstraint::PlanShape,
            rplan_audit::AuditConstraint::Population,
            rplan_audit::AuditConstraint::Contiguity,
        ],
        &generated_at,
        Some(lineage),
    )?;
    if matches!(audit.result, rplan_audit::AuditResult::Fail) {
        return Err(anyhow!("certified recursive solution failed RPLAN audit"));
    }
    std::fs::write(
        args.out_dir.join("exact.rplan"),
        rplan_io::write_rplan_string(&document)?,
    )?;
    std::fs::write(
        args.out_dir.join("exact.rctx"),
        rplan_io::write_rctx_string(context)?,
    )?;
    std::fs::write(
        args.out_dir.join("audit-certificate.json"),
        serde_json::to_string_pretty(&audit)?,
    )?;
    Ok(())
}

#[derive(Serialize)]
struct CertifiedRecursivePackageManifest {
    schema_version: String,
    method: String,
    tree_id: String,
    root_unit_universe_hash: String,
    districts: usize,
    split_count: usize,
    leaf_count: usize,
    declared_population_tolerance_percent: f64,
    files: BTreeMap<String, String>,
    claim_boundary: String,
}

fn write_recursive_package_manifest(
    args: &ExactArgs,
    tree: &bisect_ilp::CertifiedBisectionTree,
) -> anyhow::Result<()> {
    let mut files = BTreeMap::new();
    for name in [
        "certified-bisection-tree.json",
        "exact.rplan",
        "exact.rctx",
        "audit-certificate.json",
    ] {
        let path = args.out_dir.join(name);
        if path.is_file() {
            files.insert(name.to_string(), bisect_report::sha256_file(&path)?);
        }
    }
    let manifest = CertifiedRecursivePackageManifest {
        schema_version: "certified-recursive-bisection-package-v1".to_string(),
        method: "certified-recursive".to_string(),
        tree_id: tree.tree_id.clone(),
        root_unit_universe_hash: tree.root_unit_universe_hash.clone(),
        districts: tree.k,
        split_count: tree.nodes.len(),
        leaf_count: tree.leaves.len(),
        declared_population_tolerance_percent: args.tolerance,
        files,
        claim_boundary: format!(
            "Bounded exact certification of the sequential standard-bisect tree with at most {} units per split; not global unrestricted-map optimality or block-scale readiness.",
            args.exact_fixture_limit
        ),
    };
    std::fs::write(
        args.out_dir.join("certified-tree-package-manifest.json"),
        serde_json::to_string_pretty(&manifest)?,
    )?;
    Ok(())
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
    let generated_at = args
        .generated_at
        .clone()
        .unwrap_or_else(bisect_report::now_iso8601);
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
            display_labels: (1..=args.districts)
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
            generated_at: Some("2026-07-10T12:00:00Z".to_string()),
            discovery_seed: 1,
            discovery_refinement: DiscoveryRefinementArg::Full,
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

    #[test]
    fn run_exact_canonical_emits_verified_package() {
        let tmp = TempDir::new().unwrap();
        let context_path = tmp.path().join("fixture.rctx");
        let out_dir = tmp.path().join("canonical");
        std::fs::write(
            &context_path,
            rplan_io::write_rctx_string(&path4_context()).unwrap(),
        )
        .unwrap();

        run_exact(&ExactArgs {
            context: context_path,
            out_dir: out_dir.clone(),
            method: ExactMethodArg::CanonicalExhaustive,
            districts: 2,
            tolerance: 1.0,
            formulation_only: false,
            exact_fixture_limit: 8,
            generated_at: Some("2026-07-10T12:00:00Z".to_string()),
            discovery_seed: 1,
            discovery_refinement: DiscoveryRefinementArg::Full,
        })
        .unwrap();

        let instance: bisect_ilp::ExactCanonicalInstance = serde_json::from_str(
            &std::fs::read_to_string(out_dir.join("exact-canonical-instance.json")).unwrap(),
        )
        .unwrap();
        let exact_certificate: bisect_ilp::ExactCanonicalCertificate = serde_json::from_str(
            &std::fs::read_to_string(out_dir.join("exact-canonical-certificate.json")).unwrap(),
        )
        .unwrap();
        let proof: bisect_ilp::ExactProofTranscript = serde_json::from_str(
            &std::fs::read_to_string(out_dir.join("exact-canonical-proof.json")).unwrap(),
        )
        .unwrap();
        bisect_ilp::verify_exact_canonical_artifacts(&instance, &exact_certificate, &proof)
            .unwrap();
        let plan = rplan_io::read_rplan_str(
            &std::fs::read_to_string(out_dir.join("exact.rplan")).unwrap(),
        )
        .unwrap();
        assert_eq!(plan.plan.assignment, vec![0, 0, 1, 1]);
        let context =
            rplan_io::read_rctx_str(&std::fs::read_to_string(out_dir.join("exact.rctx")).unwrap())
                .unwrap();
        let audit: rplan_audit::AuditCertificate = serde_json::from_str(
            &std::fs::read_to_string(out_dir.join("audit-certificate.json")).unwrap(),
        )
        .unwrap();
        rplan_audit::verify_audit_certificate(&audit, Some(&plan.plan), Some(&context)).unwrap();
        let package: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(out_dir.join("exact-package-manifest.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(package["result"], "optimal");
        assert_eq!(package["method"], "canonical-exhaustive");
        assert_eq!(package["declared_population_tolerance_percent"], 1.0);
        assert_eq!(package["files"].as_object().unwrap().len(), 6);
        assert_eq!(package["exact_proof_id"], proof.transcript_id);
        for (name, expected_hash) in package["files"].as_object().unwrap() {
            assert_eq!(
                bisect_report::sha256_file(&out_dir.join(name)).unwrap(),
                expected_hash.as_str().unwrap()
            );
        }
    }

    #[test]
    fn run_exact_canonical_emits_infeasibility_without_plan() {
        let tmp = TempDir::new().unwrap();
        let context_path = tmp.path().join("infeasible.rctx");
        let out_dir = tmp.path().join("infeasible");
        std::fs::write(
            &context_path,
            rplan_io::write_rctx_string(&three_islands_context()).unwrap(),
        )
        .unwrap();

        run_exact(&ExactArgs {
            context: context_path,
            out_dir: out_dir.clone(),
            method: ExactMethodArg::CanonicalExhaustive,
            districts: 2,
            tolerance: 1.0,
            formulation_only: false,
            exact_fixture_limit: 8,
            generated_at: Some("2026-07-10T12:00:00Z".to_string()),
            discovery_seed: 1,
            discovery_refinement: DiscoveryRefinementArg::Full,
        })
        .unwrap();

        let certificate: bisect_ilp::ExactCanonicalCertificate = serde_json::from_str(
            &std::fs::read_to_string(out_dir.join("exact-canonical-certificate.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(
            certificate.result,
            bisect_ilp::ExactCertificateResult::Infeasible
        );
        assert!(!out_dir.join("exact.rplan").exists());
        let package: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(out_dir.join("exact-package-manifest.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(package["result"], "infeasible");
        assert_eq!(package["files"].as_object().unwrap().len(), 3);
    }

    #[test]
    fn run_exact_canonical_rejects_non_integer_edge_weight() {
        let tmp = TempDir::new().unwrap();
        let context_path = tmp.path().join("bad-weight.rctx");
        let mut context = path4_context();
        context.graph.as_mut().unwrap().adjacency[0][0].weight = Some(1.5);
        context.graph.as_mut().unwrap().adjacency[1][0].weight = Some(1.5);
        std::fs::write(
            &context_path,
            rplan_io::write_rctx_string(&context).unwrap(),
        )
        .unwrap();
        let error = run_exact(&ExactArgs {
            context: context_path,
            out_dir: tmp.path().join("out"),
            method: ExactMethodArg::CanonicalExhaustive,
            districts: 2,
            tolerance: 1.0,
            formulation_only: false,
            exact_fixture_limit: 8,
            generated_at: Some("2026-07-10T12:00:00Z".to_string()),
            discovery_seed: 1,
            discovery_refinement: DiscoveryRefinementArg::Full,
        })
        .unwrap_err();
        assert!(error.to_string().contains("non-positive or non-integer"));
    }

    #[test]
    fn run_exact_certified_recursive_emits_verified_tree_package() {
        let tmp = TempDir::new().unwrap();
        let context_path = tmp.path().join("path8.rctx");
        let out_dir = tmp.path().join("recursive");
        std::fs::write(
            &context_path,
            rplan_io::write_rctx_string(&path_context(8)).unwrap(),
        )
        .unwrap();

        run_exact(&ExactArgs {
            context: context_path,
            out_dir: out_dir.clone(),
            method: ExactMethodArg::CertifiedRecursive,
            districts: 4,
            tolerance: 1.0,
            formulation_only: false,
            exact_fixture_limit: 8,
            generated_at: Some("2026-07-10T12:00:00Z".to_string()),
            discovery_seed: 1,
            discovery_refinement: DiscoveryRefinementArg::Full,
        })
        .unwrap();

        let tree: bisect_ilp::CertifiedBisectionTree = serde_json::from_str(
            &std::fs::read_to_string(out_dir.join("certified-bisection-tree.json")).unwrap(),
        )
        .unwrap();
        bisect_ilp::verify_certified_bisection_tree_bounded(&tree).unwrap();
        assert_eq!(tree.nodes.len(), 3);
        assert_eq!(tree.leaves.len(), 4);
        let plan = rplan_io::read_rplan_str(
            &std::fs::read_to_string(out_dir.join("exact.rplan")).unwrap(),
        )
        .unwrap();
        assert_eq!(plan.plan.assignment, vec![0, 0, 1, 1, 2, 2, 3, 3]);
        let package: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(out_dir.join("certified-tree-package-manifest.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(package["method"], "certified-recursive");
        assert_eq!(package["tree_id"], tree.tree_id);
        assert_eq!(package["files"].as_object().unwrap().len(), 4);
        for (name, expected_hash) in package["files"].as_object().unwrap() {
            assert_eq!(
                bisect_report::sha256_file(&out_dir.join(name)).unwrap(),
                expected_hash.as_str().unwrap()
            );
        }
    }

    #[test]
    fn run_exact_certified_recursive_enforces_fixture_limit() {
        let tmp = TempDir::new().unwrap();
        let context_path = tmp.path().join("path8.rctx");
        std::fs::write(
            &context_path,
            rplan_io::write_rctx_string(&path_context(8)).unwrap(),
        )
        .unwrap();
        let error = run_exact(&ExactArgs {
            context: context_path,
            out_dir: tmp.path().join("out"),
            method: ExactMethodArg::CertifiedRecursive,
            districts: 4,
            tolerance: 1.0,
            formulation_only: false,
            exact_fixture_limit: 7,
            generated_at: Some("2026-07-10T12:00:00Z".to_string()),
            discovery_seed: 1,
            discovery_refinement: DiscoveryRefinementArg::Full,
        })
        .unwrap_err();
        assert!(error.to_string().contains("above --exact-fixture-limit"));
    }

    #[test]
    fn run_exact_certified_discovery_is_deterministic_and_unproved() {
        let tmp = TempDir::new().unwrap();
        let context_path = tmp.path().join("path8.rctx");
        std::fs::write(
            &context_path,
            rplan_io::write_rctx_string(&path_context(8)).unwrap(),
        )
        .unwrap();
        let run = |out_dir: std::path::PathBuf| {
            run_exact(&ExactArgs {
                context: context_path.clone(),
                out_dir,
                method: ExactMethodArg::CertifiedDiscovery,
                districts: 4,
                tolerance: 1.0,
                formulation_only: false,
                exact_fixture_limit: 8,
                generated_at: Some("2026-07-10T12:00:00Z".to_string()),
                discovery_seed: 42,
                discovery_refinement: DiscoveryRefinementArg::Full,
            })
            .unwrap();
        };
        let first = tmp.path().join("first");
        let second = tmp.path().join("second");
        run(first.clone());
        run(second.clone());
        assert_eq!(
            std::fs::read(first.join("certified-discovery.json")).unwrap(),
            std::fs::read(second.join("certified-discovery.json")).unwrap()
        );
        let instance: bisect_ilp::CertifiedSplitInstance = serde_json::from_str(
            &std::fs::read_to_string(first.join("certified-split-instance.json")).unwrap(),
        )
        .unwrap();
        let discovery: bisect_ilp::CertifiedSplitDiscovery = serde_json::from_str(
            &std::fs::read_to_string(first.join("certified-discovery.json")).unwrap(),
        )
        .unwrap();
        let requests =
            bisect_ilp::compile_certified_split_compact_proof_requests(&instance, &discovery)
                .unwrap();
        assert_eq!(requests.len(), 3);
        let manifest: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(first.join("certified-discovery-manifest.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(manifest["status"], "unproved-incumbent");
        assert_eq!(manifest["seed"], 42);
    }

    #[test]
    fn run_exact_certified_discovery_supports_unequal_seat_ratio() {
        let tmp = TempDir::new().unwrap();
        let context_path = tmp.path().join("path10.rctx");
        std::fs::write(
            &context_path,
            rplan_io::write_rctx_string(&path_context(10)).unwrap(),
        )
        .unwrap();
        let out_dir = tmp.path().join("unequal");
        run_exact(&ExactArgs {
            context: context_path,
            out_dir: out_dir.clone(),
            method: ExactMethodArg::CertifiedDiscovery,
            districts: 5,
            tolerance: 1.0,
            formulation_only: false,
            exact_fixture_limit: 10,
            generated_at: Some("2026-07-10T12:00:00Z".to_string()),
            discovery_seed: 42,
            discovery_refinement: DiscoveryRefinementArg::NrsV01,
        })
        .unwrap();
        let instance: bisect_ilp::CertifiedSplitInstance = serde_json::from_str(
            &std::fs::read_to_string(out_dir.join("certified-split-instance.json")).unwrap(),
        )
        .unwrap();
        let discovery: bisect_ilp::CertifiedSplitDiscovery = serde_json::from_str(
            &std::fs::read_to_string(out_dir.join("certified-discovery.json")).unwrap(),
        )
        .unwrap();
        assert_eq!((instance.k_left, instance.k_right), (2, 3));
        assert_eq!(discovery.objective.canonical_assignment.len(), 10);
        assert!(discovery.method.contains("partition-type=recursive"));
        assert!(discovery
            .method
            .contains("initial-dfs-minimum-deviation-candidates=2"));
        assert!(discovery
            .method
            .contains("initial-dfs-minimum-deviation-cut-candidates=2"));
        assert!(discovery
            .method
            .contains("initial-dfs-minimum-deviation-cut-partitions=2"));
        assert!(discovery
            .method
            .contains("nrs-v0-2-fallback-activated=false"));
        assert!(discovery
            .method
            .contains("nrs-v0-2-fallback-evaluated-candidates=0"));
        assert!(discovery
            .method
            .contains("nrs-v0-3-fallback-activated=false"));
        assert!(discovery
            .method
            .contains("nrs-v0-3-fallback-evaluated-candidates=0"));
        assert!(discovery.method.contains("refinement=nrsv01"));
        assert!(bisect_ilp::certified_split_children_connected(
            &instance,
            &discovery.objective.canonical_assignment
        )
        .unwrap());
    }

    #[test]
    fn nrs_dfs_tree_cut_normalizes_fragmented_candidate_deterministically() {
        let context = path_context(10);
        let instance = certified_root_instance_from_context(&context, 5).unwrap();
        let adjacency = graph_adjacency(context.graph.as_ref().unwrap()).unwrap();
        let raw = (0..10).map(|unit| (unit % 2) as u8).collect::<Vec<_>>();
        let first = nrs_dfs_tree_cut_candidate(&instance, &adjacency, &raw, 0).unwrap();
        let second = nrs_dfs_tree_cut_candidate(&instance, &adjacency, &raw, 0).unwrap();
        assert_eq!(first, second);
        assert_eq!(first.minimum_deviation_candidate_count, 2);
        assert_eq!(first.minimum_deviation_cut_candidate_count, 2);
        assert_eq!(first.minimum_deviation_cut_partition_count, 2);
        assert!(rgraph_core::assignment_labels_connected(
            &adjacency,
            &first.assignment,
            [0_u8, 1_u8]
        )
        .unwrap());
        let left_population = instance
            .populations
            .iter()
            .zip(&first.assignment)
            .filter_map(|(&population, &label)| (label == 0).then_some(population))
            .sum::<i64>();
        assert_eq!(left_population, 400);
    }

    #[test]
    fn nrs_dfs_tree_cut_collapses_complementary_orientations() {
        let context = path_context(10);
        let instance = certified_root_instance_from_context(&context, 4).unwrap();
        let adjacency = graph_adjacency(context.graph.as_ref().unwrap()).unwrap();
        let raw = (0..10).map(|unit| (unit % 2) as u8).collect::<Vec<_>>();
        let candidate = nrs_dfs_tree_cut_candidate(&instance, &adjacency, &raw, 0).unwrap();
        assert_eq!((instance.k_left, instance.k_right), (2, 2));
        assert_eq!(candidate.minimum_deviation_cut_candidate_count, 2);
        assert_eq!(candidate.minimum_deviation_cut_partition_count, 1);
    }

    #[test]
    fn nrs_fallback_diagnostics_collapse_complementary_assignments() {
        let mut diagnostics = NrsFallbackDiagnosticAccumulator::default();
        diagnostics.observe(&(10, 5, 4, vec![0, 0, 1]), &[0, 0, 1]);
        diagnostics.observe(&(10, 5, 4, vec![1, 1, 0]), &[1, 1, 0]);
        diagnostics.observe(&(10, 5, 4, vec![0, 1, 0]), &[0, 1, 0]);
        let result = diagnostics.finish();

        assert_eq!(result.evaluated_candidate_count, 3);
        assert_eq!(result.minimum_deviation_candidate_count, 3);
        assert_eq!(result.minimum_deviation_cut_candidate_count, 3);
        assert_eq!(result.minimum_deviation_cut_partition_count, 2);
    }

    #[test]
    fn nrs_dfs_tree_cut_supports_nonzero_root() {
        let context = path_context(10);
        let instance = certified_root_instance_from_context(&context, 5).unwrap();
        let adjacency = graph_adjacency(context.graph.as_ref().unwrap()).unwrap();
        let raw = (0..10).map(|unit| (unit % 2) as u8).collect::<Vec<_>>();
        let candidate = nrs_dfs_tree_cut_candidate(&instance, &adjacency, &raw, 9).unwrap();
        assert!(rgraph_core::assignment_labels_connected(
            &adjacency,
            &candidate.assignment,
            [0_u8, 1_u8]
        )
        .unwrap());
    }

    #[test]
    fn nrs_dfs_tree_cut_replaces_connected_unbalanced_candidate() {
        let context = path_context(10);
        let instance = certified_root_instance_from_context(&context, 5).unwrap();
        let adjacency = graph_adjacency(context.graph.as_ref().unwrap()).unwrap();
        let raw = vec![0, 0, 0, 0, 0, 0, 1, 1, 1, 1];
        assert!(rgraph_core::assignment_labels_connected(&adjacency, &raw, [0_u8, 1_u8]).unwrap());
        let candidate = nrs_dfs_tree_cut_candidate(&instance, &adjacency, &raw, 0).unwrap();
        assert_ne!(candidate.assignment, raw);
        assert!(rgraph_core::assignment_labels_connected(
            &adjacency,
            &candidate.assignment,
            [0_u8, 1_u8]
        )
        .unwrap());
        let left_population = instance
            .populations
            .iter()
            .zip(&candidate.assignment)
            .filter_map(|(&population, &label)| (label == 0).then_some(population))
            .sum::<i64>();
        assert_eq!(left_population, 400);
    }

    #[test]
    fn nrs_v0_3_bridge_aware_fallback_balances_disconnected_land_components() {
        let unit_ids = (0..6).map(|unit| format!("u{unit}")).collect::<Vec<_>>();
        let instance = bisect_ilp::CertifiedSplitInstance {
            schema_version: bisect_ilp::CERTIFIED_SPLIT_INSTANCE_SCHEMA_VERSION.to_string(),
            model_id: bisect_ilp::CERTIFIED_SPLIT_MODEL_ID.to_string(),
            node_path: String::new(),
            parent_certificate_id: None,
            unit_universe_hash: bisect_ilp::certified_split_unit_universe_hash(&unit_ids).unwrap(),
            unit_ids,
            populations: vec![30, 20, 20, 30, 40, 60],
            edges: (0..5)
                .map(|left| bisect_ilp::ExactEdge {
                    left,
                    right: left + 1,
                    weight: 1,
                })
                .collect(),
            k_parent: 2,
            k_left: 1,
            k_right: 1,
            orientation_rule: bisect_ilp::SplitOrientationRule::EqualSeatsUnitZeroLeft,
        };
        let adjacency = vec![
            vec![1],
            vec![0, 2],
            vec![1, 3],
            vec![2, 4],
            vec![3, 5],
            vec![4],
        ];
        let land_adjacency = vec![vec![1], vec![0, 2], vec![1, 3], vec![2], vec![5], vec![4]];
        let raw = vec![0, 0, 0, 0, 0, 1];
        assert!(
            nrs_population_deviation_scaled(&instance, &raw)
                > nrs_population_tolerance_scaled_bound(&instance)
        );

        let candidate = nrs_v0_3_bridge_aware_component_candidate(
            &instance,
            &adjacency,
            &land_adjacency,
            &raw,
            &raw,
            0,
            0,
        )
        .unwrap();

        assert_eq!(
            nrs_population_deviation_scaled(&instance, &candidate.assignment),
            0
        );
        assert!(nrs_assignment_side_connected(
            &adjacency,
            &candidate.assignment,
            0
        ));
        assert!(nrs_assignment_side_connected(
            &adjacency,
            &candidate.assignment,
            1
        ));
    }

    #[test]
    fn zero_population_boundary_cleanup_reduces_cut_without_changing_population() {
        let unit_ids = (0..4).map(|unit| format!("u{unit}")).collect::<Vec<_>>();
        let instance = bisect_ilp::CertifiedSplitInstance {
            schema_version: bisect_ilp::CERTIFIED_SPLIT_INSTANCE_SCHEMA_VERSION.to_string(),
            model_id: bisect_ilp::CERTIFIED_SPLIT_MODEL_ID.to_string(),
            node_path: String::new(),
            parent_certificate_id: None,
            unit_universe_hash: bisect_ilp::certified_split_unit_universe_hash(&unit_ids).unwrap(),
            unit_ids,
            populations: vec![1, 0, 1, 1],
            edges: vec![
                bisect_ilp::ExactEdge {
                    left: 0,
                    right: 1,
                    weight: 1,
                },
                bisect_ilp::ExactEdge {
                    left: 1,
                    right: 2,
                    weight: 1,
                },
                bisect_ilp::ExactEdge {
                    left: 1,
                    right: 3,
                    weight: 1,
                },
                bisect_ilp::ExactEdge {
                    left: 2,
                    right: 3,
                    weight: 1,
                },
            ],
            k_parent: 2,
            k_left: 1,
            k_right: 1,
            orientation_rule: bisect_ilp::SplitOrientationRule::EqualSeatsUnitZeroLeft,
        };
        let adjacency = vec![vec![1], vec![0, 2, 3], vec![1, 3], vec![1, 2]];
        let mut assignment = vec![0, 0, 1, 1];
        let before =
            bisect_ilp::evaluate_certified_split_objective(&instance, &assignment).unwrap();
        assert_eq!(
            improve_zero_population_boundary(&instance, &adjacency, &mut assignment).unwrap(),
            1
        );
        let after = bisect_ilp::evaluate_certified_split_objective(&instance, &assignment).unwrap();
        assert_eq!(assignment, vec![0, 1, 1, 1]);
        assert_eq!(
            after.max_population_deviation_scaled,
            before.max_population_deviation_scaled
        );
        assert!(after.weighted_boundary_cut < before.weighted_boundary_cut);
    }

    #[test]
    fn population_repair_uses_cut_delta_before_geoid_order() {
        let unit_ids = (0..6).map(|unit| format!("u{unit}")).collect::<Vec<_>>();
        let instance = bisect_ilp::CertifiedSplitInstance {
            schema_version: bisect_ilp::CERTIFIED_SPLIT_INSTANCE_SCHEMA_VERSION.to_string(),
            model_id: bisect_ilp::CERTIFIED_SPLIT_MODEL_ID.to_string(),
            node_path: String::new(),
            parent_certificate_id: None,
            unit_universe_hash: bisect_ilp::certified_split_unit_universe_hash(&unit_ids).unwrap(),
            unit_ids,
            populations: vec![2, 1, 1, 1, 1, 1],
            edges: vec![
                bisect_ilp::ExactEdge {
                    left: 0,
                    right: 1,
                    weight: 10,
                },
                bisect_ilp::ExactEdge {
                    left: 1,
                    right: 3,
                    weight: 10,
                },
                bisect_ilp::ExactEdge {
                    left: 2,
                    right: 3,
                    weight: 1,
                },
                bisect_ilp::ExactEdge {
                    left: 0,
                    right: 2,
                    weight: 1,
                },
                bisect_ilp::ExactEdge {
                    left: 1,
                    right: 4,
                    weight: 1,
                },
                bisect_ilp::ExactEdge {
                    left: 2,
                    right: 4,
                    weight: 10,
                },
                bisect_ilp::ExactEdge {
                    left: 4,
                    right: 5,
                    weight: 1,
                },
            ],
            k_parent: 2,
            k_left: 1,
            k_right: 1,
            orientation_rule: bisect_ilp::SplitOrientationRule::EqualSeatsUnitZeroLeft,
        };
        let adjacency = vec![
            vec![1, 2],
            vec![0, 3, 4],
            vec![0, 3, 4],
            vec![1, 2],
            vec![1, 2, 5],
            vec![4],
        ];
        let mut assignment = vec![0, 0, 0, 0, 1, 1];
        assert_eq!(
            improve_discovery_population(&instance, &adjacency, &mut assignment, None).unwrap(),
            (1, 1)
        );
        assert_eq!(assignment, vec![0, 0, 1, 0, 1, 1]);
    }

    #[test]
    fn population_repair_moves_connected_subtree_past_articulation_local_minimum() {
        let unit_ids = (0..4).map(|unit| format!("u{unit}")).collect::<Vec<_>>();
        let instance = bisect_ilp::CertifiedSplitInstance {
            schema_version: bisect_ilp::CERTIFIED_SPLIT_INSTANCE_SCHEMA_VERSION.to_string(),
            model_id: bisect_ilp::CERTIFIED_SPLIT_MODEL_ID.to_string(),
            node_path: String::new(),
            parent_certificate_id: None,
            unit_universe_hash: bisect_ilp::certified_split_unit_universe_hash(&unit_ids).unwrap(),
            unit_ids,
            populations: vec![20, 10, 20, 50],
            edges: vec![
                bisect_ilp::ExactEdge {
                    left: 0,
                    right: 1,
                    weight: 1,
                },
                bisect_ilp::ExactEdge {
                    left: 1,
                    right: 2,
                    weight: 1,
                },
                bisect_ilp::ExactEdge {
                    left: 1,
                    right: 3,
                    weight: 1,
                },
            ],
            k_parent: 2,
            k_left: 1,
            k_right: 1,
            orientation_rule: bisect_ilp::SplitOrientationRule::EqualSeatsUnitZeroLeft,
        };
        let adjacency = vec![vec![1], vec![0, 2, 3], vec![1], vec![1]];
        assert_eq!(nrs_population_tolerance_scaled_bound(&instance), 1);
        let mut within_requested_tolerance = vec![0, 1, 1, 1];
        assert_eq!(
            improve_discovery_population(
                &instance,
                &adjacency,
                &mut within_requested_tolerance,
                Some(60),
            )
            .unwrap(),
            (0, 0)
        );
        assert_eq!(within_requested_tolerance, vec![0, 1, 1, 1]);
        let mut assignment = vec![0, 1, 1, 1];
        assert_eq!(
            improve_discovery_population(&instance, &adjacency, &mut assignment, None).unwrap(),
            (1, 2)
        );
        assert_eq!(assignment, vec![0, 0, 0, 1]);
        assert!(bisect_ilp::certified_split_children_connected(&instance, &assignment).unwrap());
    }

    #[test]
    fn same_population_swap_reduces_cut_and_preserves_balance() {
        let unit_ids = (0..6).map(|unit| format!("u{unit}")).collect::<Vec<_>>();
        let instance = bisect_ilp::CertifiedSplitInstance {
            schema_version: bisect_ilp::CERTIFIED_SPLIT_INSTANCE_SCHEMA_VERSION.to_string(),
            model_id: bisect_ilp::CERTIFIED_SPLIT_MODEL_ID.to_string(),
            node_path: String::new(),
            parent_certificate_id: None,
            unit_universe_hash: bisect_ilp::certified_split_unit_universe_hash(&unit_ids).unwrap(),
            unit_ids,
            populations: vec![1, 2, 3, 1, 2, 3],
            edges: vec![
                bisect_ilp::ExactEdge {
                    left: 0,
                    right: 1,
                    weight: 1,
                },
                bisect_ilp::ExactEdge {
                    left: 1,
                    right: 2,
                    weight: 1,
                },
                bisect_ilp::ExactEdge {
                    left: 3,
                    right: 4,
                    weight: 1,
                },
                bisect_ilp::ExactEdge {
                    left: 4,
                    right: 5,
                    weight: 1,
                },
                bisect_ilp::ExactEdge {
                    left: 2,
                    right: 3,
                    weight: 10,
                },
                bisect_ilp::ExactEdge {
                    left: 0,
                    right: 5,
                    weight: 10,
                },
            ],
            k_parent: 2,
            k_left: 1,
            k_right: 1,
            orientation_rule: bisect_ilp::SplitOrientationRule::EqualSeatsUnitZeroLeft,
        };
        let adjacency = vec![
            vec![1, 5],
            vec![0, 2],
            vec![1, 3],
            vec![2, 4],
            vec![3, 5],
            vec![0, 4],
        ];
        let mut assignment = vec![0, 0, 0, 1, 1, 1];
        let before =
            bisect_ilp::evaluate_certified_split_objective(&instance, &assignment).unwrap();
        assert_eq!(
            improve_same_population_swaps(&instance, &adjacency, &mut assignment).unwrap(),
            1
        );
        let after = bisect_ilp::evaluate_certified_split_objective(&instance, &assignment).unwrap();
        assert_eq!(assignment, vec![0, 0, 1, 1, 1, 0]);
        assert_eq!(
            after.max_population_deviation_scaled,
            before.max_population_deviation_scaled
        );
        assert!(after.weighted_boundary_cut < before.weighted_boundary_cut);
    }

    #[test]
    fn one_to_two_swap_reduces_cut_and_preserves_balance() {
        let unit_ids = (0..7).map(|unit| format!("u{unit}")).collect::<Vec<_>>();
        let mut edges = Vec::new();
        for group in [&[0, 1, 2][..], &[3, 4, 5, 6][..]] {
            for (index, &left) in group.iter().enumerate() {
                for &right in &group[index + 1..] {
                    edges.push(bisect_ilp::ExactEdge {
                        left,
                        right,
                        weight: 1,
                    });
                }
            }
        }
        for (left, right) in [(1, 4), (1, 6), (3, 0), (3, 2), (5, 0), (5, 2)] {
            edges.push(bisect_ilp::ExactEdge {
                left: left.min(right),
                right: left.max(right),
                weight: 10,
            });
        }
        edges.sort_by_key(|edge| (edge.left, edge.right));
        edges.dedup_by_key(|edge| (edge.left, edge.right));
        let instance = bisect_ilp::CertifiedSplitInstance {
            schema_version: bisect_ilp::CERTIFIED_SPLIT_INSTANCE_SCHEMA_VERSION.to_string(),
            model_id: bisect_ilp::CERTIFIED_SPLIT_MODEL_ID.to_string(),
            node_path: String::new(),
            parent_certificate_id: None,
            unit_universe_hash: bisect_ilp::certified_split_unit_universe_hash(&unit_ids).unwrap(),
            unit_ids,
            populations: vec![1, 3, 2, 1, 1, 2, 2],
            edges: edges.clone(),
            k_parent: 2,
            k_left: 1,
            k_right: 1,
            orientation_rule: bisect_ilp::SplitOrientationRule::EqualSeatsUnitZeroLeft,
        };
        let mut adjacency = vec![Vec::new(); 7];
        for edge in edges {
            adjacency[edge.left].push(edge.right);
            adjacency[edge.right].push(edge.left);
        }
        let mut assignment = vec![0, 0, 0, 1, 1, 1, 1];
        let before =
            bisect_ilp::evaluate_certified_split_objective(&instance, &assignment).unwrap();
        assert!(
            improve_one_to_two_population_swaps(&instance, &adjacency, &mut assignment).unwrap()
                >= 1
        );
        let after = bisect_ilp::evaluate_certified_split_objective(&instance, &assignment).unwrap();
        assert_eq!(
            after.max_population_deviation_scaled,
            before.max_population_deviation_scaled
        );
        assert!(after.weighted_boundary_cut < before.weighted_boundary_cut);
        assert!(bisect_ilp::certified_split_children_connected(&instance, &assignment).unwrap());
    }

    #[test]
    fn two_to_two_swap_reduces_cut_and_preserves_balance() {
        let unit_ids = (0..8).map(|unit| format!("u{unit}")).collect::<Vec<_>>();
        let mut edge_weights = BTreeMap::new();
        for group in [&[0, 1, 2, 3][..], &[4, 5, 6, 7][..]] {
            for (index, &left) in group.iter().enumerate() {
                for &right in &group[index + 1..] {
                    edge_weights.insert((left, right), 1);
                }
            }
        }
        for left in [1, 2] {
            for right in [5, 6] {
                edge_weights.insert((left, right), 10);
            }
        }
        for right in [4, 7] {
            for left in [0, 3] {
                edge_weights.insert((left, right), 10);
            }
        }
        let edges = edge_weights
            .iter()
            .map(|(&(left, right), &weight)| bisect_ilp::ExactEdge {
                left,
                right,
                weight,
            })
            .collect::<Vec<_>>();
        let instance = bisect_ilp::CertifiedSplitInstance {
            schema_version: bisect_ilp::CERTIFIED_SPLIT_INSTANCE_SCHEMA_VERSION.to_string(),
            model_id: bisect_ilp::CERTIFIED_SPLIT_MODEL_ID.to_string(),
            node_path: String::new(),
            parent_certificate_id: None,
            unit_universe_hash: bisect_ilp::certified_split_unit_universe_hash(&unit_ids).unwrap(),
            unit_ids,
            populations: vec![1, 2, 3, 4, 1, 2, 3, 4],
            edges: edges.clone(),
            k_parent: 2,
            k_left: 1,
            k_right: 1,
            orientation_rule: bisect_ilp::SplitOrientationRule::EqualSeatsUnitZeroLeft,
        };
        let mut adjacency = vec![Vec::new(); 8];
        for edge in edges {
            adjacency[edge.left].push(edge.right);
            adjacency[edge.right].push(edge.left);
        }
        let mut assignment = vec![0, 0, 0, 0, 1, 1, 1, 1];
        let before =
            bisect_ilp::evaluate_certified_split_objective(&instance, &assignment).unwrap();
        assert!(
            improve_two_to_two_population_swaps(&instance, &adjacency, &mut assignment).unwrap()
                >= 1
        );
        let after = bisect_ilp::evaluate_certified_split_objective(&instance, &assignment).unwrap();
        assert_eq!(
            after.max_population_deviation_scaled,
            before.max_population_deviation_scaled
        );
        assert!(after.weighted_boundary_cut < before.weighted_boundary_cut);
        assert!(bisect_ilp::certified_split_children_connected(&instance, &assignment).unwrap());
    }

    fn path4_context() -> RplanContext {
        path_context(4)
    }

    fn path_context(unit_count: usize) -> RplanContext {
        let mut units = PlanUnitIndex {
            unit_kind: UnitKind::Imported,
            state: Some("TT".to_string()),
            year: Some(2020),
            canonical_order: CanonicalOrder::ExplicitUnitIds,
            unit_ids: (0..unit_count).map(|idx| format!("u{idx:02}")).collect(),
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
                adjacency: (0..unit_count)
                    .map(|unit| {
                        let mut neighbors = Vec::new();
                        if unit > 0 {
                            neighbors.push(edge((unit - 1) as u32));
                        }
                        if unit + 1 < unit_count {
                            neighbors.push(edge((unit + 1) as u32));
                        }
                        neighbors
                    })
                    .collect(),
            }),
            populations: Some(vec![100; unit_count]),
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

    fn three_islands_context() -> RplanContext {
        let mut units = PlanUnitIndex {
            unit_kind: UnitKind::Imported,
            state: Some("TT".to_string()),
            year: Some(2020),
            canonical_order: CanonicalOrder::ExplicitUnitIds,
            unit_ids: ["a", "b", "c"].into_iter().map(str::to_string).collect(),
            unit_universe_hash: String::new(),
            source_id: Some("exact-infeasible-fixture".to_string()),
        };
        units.unit_universe_hash = units.compute_unit_universe_hash().unwrap();
        RplanContext {
            rctx_version: RCTX_VERSION.to_string(),
            context_hash: String::new(),
            units,
            graph: Some(UnitGraph {
                edge_semantics: EdgeSemantics::Undirected,
                adjacency: vec![vec![], vec![], vec![]],
            }),
            populations: Some(vec![1, 1, 1]),
            subdivisions: None,
            demographics: None,
            geometry: None,
            source_hashes: SourceHashes {
                entries: BTreeMap::from([(
                    "fixture".to_string(),
                    format!("sha256:{}", "2".repeat(64)),
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
