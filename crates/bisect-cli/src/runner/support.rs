//! Helpers peeled from the historical `runner.rs` tail.
//! Keep `run_single_state` orchestration in `mod.rs`; put rplan/spectral/weight
//! side paths here.

use super::*;

pub(crate) fn count_ilp_solve_reports(intermediate_dir: &std::path::Path) -> usize {
    let report_root = intermediate_dir.join("ilp_solve_reports");
    if !report_root.exists() {
        return 0;
    }

    let mut stack = vec![report_root];
    let mut count = 0usize;
    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path
                .extension()
                .and_then(|ext| ext.to_str())
                .is_some_and(|ext| ext.eq_ignore_ascii_case("json"))
                && path.file_name().and_then(|name| name.to_str()) != Some("audit-summary.json")
            {
                count += 1;
            }
        }
    }
    count
}

#[derive(Debug, Clone)]
pub(crate) struct RunnerAuditSidecars {
    pub(crate) rplan_path: String,
    pub(crate) rctx_path: String,
    pub(crate) audit_certificate_path: String,
    pub(crate) audit_certificate_sha256: String,
    pub(crate) audit_certificate_content_hash: String,
    pub(crate) audit_result: String,
    pub(crate) legal_profile_id: String,
    pub(crate) context_hash: String,
}

pub(crate) fn runner_tiger_sha256(cfg: &StateConfig) -> Result<Option<String>, String> {
    let Some(path) = rplan_tiger_tract_path(&cfg.state_code, &cfg.year) else {
        return Ok(None);
    };
    let hash = bisect_report::sha256_file(&path)
        .map_err(|e| format!("hash TIGER source {}: {e}", path.display()))?;
    Ok(Some(hash))
}

pub(crate) fn write_rplan_audit_sidecars(
    plan_root: &std::path::Path,
    cfg: &StateConfig,
    label: &str,
    graph: &crate::adjacency_loader::LoadedGraph,
    assignments: &HashMap<usize, usize>,
    adjacency_file: &str,
    adjacency_path: &std::path::Path,
    tiger_source_url: &str,
    balance_tolerance: f64,
    generated_at_utc: &str,
) -> Result<RunnerAuditSidecars, String> {
    let units = build_rplan_units(cfg, graph)?;
    let assignment =
        build_rplan_assignment(assignments, graph.n_vertices, cfg.effective_num_districts())?;
    let plan = rplan_core::DistrictPlan {
        schema_version: rplan_core::DISTRICT_PLAN_SCHEMA_VERSION.to_string(),
        units: units.clone(),
        assignment,
        k: cfg.effective_num_districts(),
        display_labels: (1..=cfg.effective_num_districts())
            .map(|district_id| district_id.to_string())
            .collect(),
        allow_empty_districts: false,
    };

    let geometry = build_rplan_geometry(cfg, graph);
    let tiger_geometry_path = if geometry.is_some() {
        rplan_tiger_tract_path(&cfg.state_code, &cfg.year)
    } else {
        None
    };
    let source_hashes = build_rplan_source_hashes(adjacency_path, tiger_geometry_path.as_deref())?;
    let mut context = rplan_core::RplanContext {
        rctx_version: rplan_core::RCTX_VERSION.to_string(),
        context_hash: String::new(),
        units: units.clone(),
        graph: Some(build_rplan_graph(graph)),
        populations: Some(graph.vertex_weights.clone()),
        subdivisions: build_rplan_subdivisions(graph),
        demographics: build_rplan_demographics(cfg, graph)?,
        geometry,
        source_hashes: source_hashes.clone(),
    };
    context.context_hash = context
        .compute_context_hash()
        .map_err(|e| format!("context hash failed: {e}"))?;

    let document = rplan_io::RplanDocument {
        rplan_version: rplan_io::RPLAN_V02.to_string(),
        plan: plan.clone(),
        metadata: rplan_io::RplanMetadataV02 {
            label: label.to_string(),
            jurisdiction: cfg.state_code.clone(),
            chamber: cfg.chamber.clone(),
            created_at: generated_at_utc.to_string(),
            description: Some(format!(
                "Final plan emitted by bisect runner for {} {} {}",
                cfg.state_code, cfg.chamber, cfg.year
            )),
        },
        provenance: rplan_io::RplanProvenance {
            producer: BTreeMap::from([
                ("name".to_string(), serde_json::json!("bisect")),
                (
                    "version".to_string(),
                    serde_json::json!(env!("CARGO_PKG_VERSION")),
                ),
                (
                    "adjacency_file".to_string(),
                    serde_json::json!(adjacency_file),
                ),
                (
                    "tiger_source_url".to_string(),
                    serde_json::json!(tiger_source_url),
                ),
            ]),
            source_hashes: source_hashes.entries.clone(),
            conversion_lineage: Vec::new(),
        },
        geometry: None,
        extensions: BTreeMap::new(),
    };

    let report_vra = context.demographics.is_some();
    let report_geometry = context.geometry.is_some();
    let profile = runner_legal_profile(cfg, balance_tolerance, report_vra);
    let mut constraints = vec![
        rplan_audit::AuditConstraint::PlanShape,
        rplan_audit::AuditConstraint::Population,
        rplan_audit::AuditConstraint::Contiguity,
        rplan_audit::AuditConstraint::Splits,
    ];
    if report_vra {
        constraints.push(rplan_audit::AuditConstraint::Vra);
    }
    if report_geometry {
        constraints.push(rplan_audit::AuditConstraint::Geometry);
    }
    let algorithm_lineage = runner_algorithm_lineage(cfg, plan_root)?;
    let certificate = rplan_audit::audit_plan_with_lineage(
        &plan,
        Some(&context),
        &profile,
        rplan_audit::RuntimeProvenance {
            binary_name: "bisect".to_string(),
            binary_version: env!("CARGO_PKG_VERSION").to_string(),
            git_commit: option_env!("GIT_COMMIT").map(str::to_string),
            build_profile: None,
            solver: None,
        },
        &constraints,
        generated_at_utc,
        algorithm_lineage,
    )
    .map_err(|e| format!("audit certificate generation failed: {e}"))?;

    let rplan_rel = "plan.rplan".to_string();
    let rctx_rel = "context.rctx".to_string();
    let audit_rel = "audit-certificate.json".to_string();
    let rplan_path = plan_root.join(&rplan_rel);
    let rctx_path = plan_root.join(&rctx_rel);
    let audit_path = plan_root.join(&audit_rel);

    let rplan_json =
        rplan_io::write_rplan_string(&document).map_err(|e| format!("RPLAN write failed: {e}"))?;
    std::fs::write(&rplan_path, rplan_json)
        .map_err(|e| format!("cannot write {}: {e}", rplan_path.display()))?;

    let rctx_json =
        rplan_io::write_rctx_string(&context).map_err(|e| format!("RCTX write failed: {e}"))?;
    std::fs::write(&rctx_path, rctx_json)
        .map_err(|e| format!("cannot write {}: {e}", rctx_path.display()))?;

    let audit_json = serde_json::to_string_pretty(&certificate)
        .map_err(|e| format!("audit certificate serialization failed: {e}"))?;
    std::fs::write(&audit_path, audit_json)
        .map_err(|e| format!("cannot write {}: {e}", audit_path.display()))?;
    let audit_sha256 = bisect_report::sha256_file(&audit_path)
        .map_err(|e| format!("cannot hash {}: {e}", audit_path.display()))?;

    Ok(RunnerAuditSidecars {
        rplan_path: rplan_rel,
        rctx_path: rctx_rel,
        audit_certificate_path: audit_rel,
        audit_certificate_sha256: audit_sha256,
        audit_certificate_content_hash: certificate.content_hash,
        audit_result: audit_result_label(&certificate.result).to_string(),
        legal_profile_id: profile.profile_id,
        context_hash: certificate.context_hash.unwrap_or_default(),
    })
}

pub(crate) fn build_rplan_units(
    cfg: &StateConfig,
    graph: &crate::adjacency_loader::LoadedGraph,
) -> Result<rplan_core::PlanUnitIndex, String> {
    let unit_kind = match cfg.plan_resolution.as_str() {
        "bg" => rplan_core::UnitKind::BlockGroup,
        "county" => rplan_core::UnitKind::County,
        _ => rplan_core::UnitKind::Tract,
    };
    let mut unit_ids = Vec::with_capacity(graph.n_vertices);
    for idx in 0..graph.n_vertices {
        unit_ids.push(
            graph
                .index_to_geoid
                .get(&idx)
                .cloned()
                .unwrap_or_else(|| idx.to_string()),
        );
    }
    let mut units = rplan_core::PlanUnitIndex {
        unit_kind,
        state: Some(cfg.state_code.clone()),
        year: cfg.year.parse().ok(),
        canonical_order: rplan_core::CanonicalOrder::ExplicitUnitIds,
        unit_ids,
        unit_universe_hash: String::new(),
        source_id: Some(format!("bisect-adjacency-{}-{}", cfg.state_code, cfg.year)),
    };
    if units.validate().is_err() {
        units.unit_kind = rplan_core::UnitKind::Imported;
    }
    units.unit_universe_hash = units
        .compute_unit_universe_hash()
        .map_err(|e| format!("unit universe hash failed: {e}"))?;
    units
        .validate()
        .map_err(|e| format!("RPLAN unit validation failed: {e}"))?;
    Ok(units)
}

pub(crate) fn build_rplan_assignment(
    assignments: &HashMap<usize, usize>,
    n_vertices: usize,
    k: usize,
) -> Result<Vec<u32>, String> {
    let mut out = Vec::with_capacity(n_vertices);
    for idx in 0..n_vertices {
        let one_based = assignments
            .get(&idx)
            .copied()
            .ok_or_else(|| format!("missing final assignment for unit index {idx}"))?;
        if one_based == 0 || one_based > k {
            return Err(format!(
                "final assignment for unit index {idx} has district {one_based}, expected 1..={k}"
            ));
        }
        out.push((one_based - 1) as u32);
    }
    Ok(out)
}

pub(crate) fn build_rplan_graph(graph: &crate::adjacency_loader::LoadedGraph) -> rplan_core::UnitGraph {
    let adjacency = graph
        .adjacency
        .iter()
        .enumerate()
        .map(|(from, neighbors)| {
            neighbors
                .iter()
                .copied()
                .map(|to| {
                    let key = if from < to { (from, to) } else { (to, from) };
                    rplan_core::UnitEdge {
                        to: to as u32,
                        kind: rplan_core::EdgeKind::Boundary,
                        weight: graph.edge_weights.get(&key).copied(),
                    }
                })
                .collect()
        })
        .collect();
    rplan_core::UnitGraph {
        edge_semantics: rplan_core::EdgeSemantics::Undirected,
        adjacency,
    }
}

pub(crate) fn build_rplan_subdivisions(
    graph: &crate::adjacency_loader::LoadedGraph,
) -> Option<rplan_core::SubdivisionContext> {
    let mut county_ids = Vec::with_capacity(graph.n_vertices);
    for idx in 0..graph.n_vertices {
        let county_id = graph.index_to_geoid.get(&idx).and_then(|geoid| {
            (geoid.len() >= 5 && geoid[..5].bytes().all(|byte| byte.is_ascii_digit()))
                .then(|| geoid[..5].to_string())
        });
        county_ids.push(county_id);
    }

    county_ids
        .iter()
        .any(Option::is_some)
        .then_some(rplan_core::SubdivisionContext {
            county_ids: Some(county_ids),
            municipal_ids: None,
        })
}

pub(crate) fn build_rplan_demographics(
    cfg: &StateConfig,
    graph: &crate::adjacency_loader::LoadedGraph,
) -> Result<Option<rplan_core::DemographicContext>, String> {
    let state_name = cfg.state_name.to_lowercase().replace(' ', "_");
    let state_code = cfg.state_code.to_lowercase();
    let candidates = [
        format!("{state_name}_vap_{}.csv", cfg.year),
        format!("{state_code}_vap_{}.csv", cfg.year),
        format!("{state_name}_cvap_{}.csv", cfg.year),
        format!("{state_code}_cvap_{}.csv", cfg.year),
    ];
    let demo_dir = std::path::Path::new("data")
        .join(&cfg.year)
        .join("demographics");
    for candidate in candidates {
        let path = demo_dir.join(candidate);
        if !path.exists() {
            continue;
        }
        let demo = load_vap_demographics(&path)
            .map_err(|e| format!("VAP demographics load failed at {}: {e}", path.display()))?;
        let context =
            align_vap_demographics_to_adjacency(&demo, &graph.index_to_geoid, graph.n_vertices);
        return Ok(Some(context));
    }
    Ok(None)
}

pub(crate) fn build_rplan_source_hashes(
    adjacency_path: &std::path::Path,
    tiger_geometry_path: Option<&std::path::Path>,
) -> Result<rplan_core::SourceHashes, String> {
    let mut entries = BTreeMap::new();
    if adjacency_path.exists() {
        let hash = bisect_report::sha256_file(adjacency_path)
            .map_err(|e| format!("hash adjacency source {}: {e}", adjacency_path.display()))?;
        entries.insert("adjacency".to_string(), format!("sha256:{hash}"));
    }
    if let Some(path) = tiger_geometry_path {
        if path.exists() {
            let hash = bisect_report::sha256_file(path)
                .map_err(|e| format!("hash TIGER geometry source {}: {e}", path.display()))?;
            entries.insert("geometry".to_string(), format!("sha256:{hash}"));
        }
    }
    Ok(rplan_core::SourceHashes { entries })
}

pub(crate) fn build_rplan_geometry(
    cfg: &StateConfig,
    graph: &crate::adjacency_loader::LoadedGraph,
) -> Option<rplan_core::GeometryContext> {
    let tiger_path = rplan_tiger_tract_path(&cfg.state_code, &cfg.year)?;
    let records = bisect_data::read_tiger_tracts(&tiger_path).ok()?;
    let geoid_to_hash: HashMap<String, String> = records
        .iter()
        .map(|record| (record.geoid.clone(), sha256_bytes(&record.geometry_wkb)))
        .collect();

    let mut unit_geometry_hashes = Vec::with_capacity(graph.n_vertices);
    for idx in 0..graph.n_vertices {
        let geoid = graph.index_to_geoid.get(&idx)?;
        unit_geometry_hashes.push(geoid_to_hash.get(geoid)?.clone());
    }

    Some(rplan_core::GeometryContext {
        source_id: Some(format!(
            "tiger-tract-geometry-{}-{}",
            cfg.state_code.to_uppercase(),
            cfg.year
        )),
        crs: rplan_tiger_crs_label(&tiger_path),
        unit_geometry_hashes: Some(unit_geometry_hashes),
    })
}

pub(crate) fn rplan_tiger_tract_path(state_code: &str, year: &str) -> Option<std::path::PathBuf> {
    let fips = state_code_to_fips(&state_code.to_uppercase())?;
    let yy = year.get(2..4).unwrap_or(year);
    let candidates = [
        format!("tl_{year}_{fips}_tract"),
        format!("tl_{year}_{fips}_tract{yy}"),
        format!("tl_2010_{fips}_tract{yy}"),
    ];
    for stem in candidates {
        let path = std::path::PathBuf::from("data")
            .join(year)
            .join("tiger")
            .join("tracts")
            .join(&stem)
            .join(format!("{stem}.shp"));
        if path.exists() {
            return Some(path);
        }
    }
    None
}

pub(crate) fn rplan_tiger_crs_label(shp_path: &std::path::Path) -> Option<String> {
    let prj_path = shp_path.with_extension("prj");
    let prj = std::fs::read_to_string(prj_path).ok()?;
    if prj.contains("NAD_1983")
        || prj.contains("North_American_1983")
        || prj.contains("North_American_Datum_1983")
    {
        Some("EPSG:4269".to_string())
    } else if prj.contains("WGS_1984") || prj.contains("WGS 84") {
        Some("EPSG:4326".to_string())
    } else {
        Some("unknown".to_string())
    }
}

pub(crate) fn sha256_bytes(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let digest = hasher.finalize();
    let mut out = String::with_capacity("sha256:".len() + digest.len() * 2);
    out.push_str("sha256:");
    for byte in digest {
        out.push_str(&format!("{byte:02x}"));
    }
    out
}

pub(crate) fn runner_legal_profile(
    cfg: &StateConfig,
    balance_tolerance: f64,
    report_vra: bool,
) -> rplan_audit::LegalProfile {
    rplan_audit::LegalProfile {
        schema_version: rplan_audit::LEGAL_PROFILE_SCHEMA_VERSION.to_string(),
        profile_id: "BISECT_RUNNER_PROFILE_V1".to_string(),
        jurisdiction: cfg.state_code.clone(),
        chamber: match cfg.chamber.as_str() {
            "congressional" => rplan_audit::Chamber::Congressional,
            "state-house" => rplan_audit::Chamber::StateHouse,
            "state-senate" => rplan_audit::Chamber::StateSenate,
            "local" => rplan_audit::Chamber::Local,
            other => rplan_audit::Chamber::Custom(other.to_string()),
        },
        year: cfg.year.parse().unwrap_or(0),
        population_tolerance: rplan_audit::PopulationToleranceRule::Percent {
            max_deviation_percent: balance_tolerance * 100.0,
        },
        contiguity_required: true,
        county_split_rule: rplan_audit::SplitRule::CountOnly,
        municipal_split_rule: rplan_audit::SplitRule::NotEvaluated,
        nesting_rule: rplan_audit::NestingRule::NotEvaluated,
        vra_policy: if report_vra {
            rplan_audit::VraPolicy::ReportOpportunityDistricts {
                minority_group: "minority".to_string(),
                vap_threshold: 0.5,
            }
        } else {
            rplan_audit::VraPolicy::NotEvaluated
        },
    }
}

pub(crate) fn runner_algorithm_lineage(
    cfg: &StateConfig,
    plan_root: &std::path::Path,
) -> Result<Option<rplan_audit::AlgorithmLineage>, String> {
    match &cfg.algo.split {
        SplitStrategy::Ilp {
            method,
            fallback,
            time_limit_secs,
            optimality_gap,
            max_tracts,
        } => {
            let summary_path = plan_root
                .join("intermediate")
                .join("ilp_solve_reports")
                .join("audit-summary.json");
            let audit_summary_sha256 = if summary_path.exists() {
                Some(
                    bisect_report::sha256_file(&summary_path)
                        .map_err(|e| format!("hash ILP audit summary for lineage failed: {e}"))?,
                )
            } else {
                None
            };

            let mut extra = serde_json::json!({
                "lineage_schema": "bisect-ilp-lineage-v1",
                "method": method.to_string(),
                "fallback": fallback.to_string(),
                "time_limit_secs": time_limit_secs,
                "optimality_gap": optimality_gap,
                "max_tracts": max_tracts,
                "solve_report_dir": "intermediate/ilp_solve_reports",
                "audit_summary_path": "intermediate/ilp_solve_reports/audit-summary.json",
            });
            if let Some(sha256) = audit_summary_sha256 {
                extra["audit_summary_sha256"] = serde_json::json!(sha256);
            }
            if let Some(summary) = read_ilp_lineage_summary(&summary_path)? {
                extra["audit_summary"] = summary;
            }
            rplan_audit::AlgorithmLineage::new(
                "bisect-ilp",
                env!("CARGO_PKG_VERSION"),
                method.to_string(),
                Vec::new(),
                extra,
            )
            .map(Some)
            .map_err(|e| format!("ILP lineage construction failed: {e}"))
        }
        SplitStrategy::CapacityClustering => {
            let summary_path = plan_root
                .join("intermediate")
                .join("capacity_clustering_summary.json");
            let text = std::fs::read_to_string(&summary_path)
                .map_err(|e| format!("read capacity-clustering summary for lineage failed: {e}"))?;
            let summary: bisect_clustering::ClusterSummary =
                serde_json::from_str(&text).map_err(|e| {
                    format!("parse capacity-clustering summary for lineage failed: {e}")
                })?;
            let summary_sha256 = bisect_report::sha256_file(&summary_path)
                .map_err(|e| format!("hash capacity-clustering summary for lineage failed: {e}"))?;
            let mut extra = summary.algorithm_lineage_extra();
            if let Some(obj) = extra.as_object_mut() {
                obj.insert(
                    "summary_path".to_string(),
                    serde_json::json!("intermediate/capacity_clustering_summary.json"),
                );
                obj.insert(
                    "summary_sha256".to_string(),
                    serde_json::json!(summary_sha256),
                );
            }
            rplan_audit::AlgorithmLineage::new(
                "bisect-clustering",
                env!("CARGO_PKG_VERSION"),
                summary.method,
                Vec::new(),
                extra,
            )
            .map(Some)
            .map_err(|e| format!("capacity-clustering lineage construction failed: {e}"))
        }
        SplitStrategy::Spectral { .. } => {
            let summary_path = plan_root.join("intermediate").join("spectral_summary.json");
            let text = std::fs::read_to_string(&summary_path)
                .map_err(|e| format!("read spectral summary for lineage failed: {e}"))?;
            let summary: serde_json::Value = serde_json::from_str(&text)
                .map_err(|e| format!("parse spectral summary for lineage failed: {e}"))?;
            let summary_sha256 = bisect_report::sha256_file(&summary_path)
                .map_err(|e| format!("hash spectral summary for lineage failed: {e}"))?;
            let extra = serde_json::json!({
                "lineage_schema_version": "bisect-spectral-lineage-v1",
                "method": "spectral",
                "summary_path": "intermediate/spectral_summary.json",
                "summary_sha256": summary_sha256,
                "summary": summary,
            });
            rplan_audit::AlgorithmLineage::new(
                "bisect-apportion",
                env!("CARGO_PKG_VERSION"),
                "spectral",
                Vec::new(),
                extra,
            )
            .map(Some)
            .map_err(|e| format!("spectral lineage construction failed: {e}"))
        }
        SplitStrategy::Regionalization => {
            let summary_path = plan_root
                .join("intermediate")
                .join("regionalization_summary.json");
            let text = std::fs::read_to_string(&summary_path)
                .map_err(|e| format!("read regionalization summary for lineage failed: {e}"))?;
            let summary: bisect_clustering::output::RegionalizationSummary =
                serde_json::from_str(&text).map_err(|e| {
                    format!("parse regionalization summary for lineage failed: {e}")
                })?;
            let summary_sha256 = bisect_report::sha256_file(&summary_path)
                .map_err(|e| format!("hash regionalization summary for lineage failed: {e}"))?;
            let merge_path = plan_root
                .join("intermediate")
                .join("regionalization_merges.json");
            let merge_sha256 = bisect_report::sha256_file(&merge_path)
                .map_err(|e| format!("hash regionalization merges for lineage failed: {e}"))?;
            let mut extra = summary.algorithm_lineage_extra();
            if let Some(obj) = extra.as_object_mut() {
                obj.insert(
                    "summary_path".to_string(),
                    serde_json::json!("intermediate/regionalization_summary.json"),
                );
                obj.insert(
                    "summary_sha256".to_string(),
                    serde_json::json!(summary_sha256),
                );
                obj.insert(
                    "merge_log_path".to_string(),
                    serde_json::json!("intermediate/regionalization_merges.json"),
                );
                obj.insert(
                    "merge_log_sha256".to_string(),
                    serde_json::json!(merge_sha256),
                );
            }
            rplan_audit::AlgorithmLineage::new(
                "bisect-clustering",
                env!("CARGO_PKG_VERSION"),
                summary.method,
                Vec::new(),
                extra,
            )
            .map(Some)
            .map_err(|e| format!("regionalization lineage construction failed: {e}"))
        }
        SplitStrategy::FlowConstruction => {
            let summary_path = plan_root
                .join("intermediate")
                .join("flow_construction_summary.json");
            let text = std::fs::read_to_string(&summary_path)
                .map_err(|e| format!("read flow summary for lineage failed: {e}"))?;
            let summary: bisect_flow::FlowSummary = serde_json::from_str(&text)
                .map_err(|e| format!("parse flow summary for lineage failed: {e}"))?;
            let summary_sha256 = bisect_report::sha256_file(&summary_path)
                .map_err(|e| format!("hash flow summary for lineage failed: {e}"))?;
            let mut extra = summary.algorithm_lineage_extra();
            if let Some(obj) = extra.as_object_mut() {
                obj.insert(
                    "summary_path".to_string(),
                    serde_json::json!("intermediate/flow_construction_summary.json"),
                );
                obj.insert(
                    "summary_sha256".to_string(),
                    serde_json::json!(summary_sha256),
                );
            }
            rplan_audit::AlgorithmLineage::new(
                "bisect-flow",
                env!("CARGO_PKG_VERSION"),
                summary.method,
                Vec::new(),
                extra,
            )
            .map(Some)
            .map_err(|e| format!("flow lineage construction failed: {e}"))
        }
        _ => Ok(None),
    }
}

pub(crate) fn read_ilp_lineage_summary(path: &std::path::Path) -> Result<Option<serde_json::Value>, String> {
    if !path.exists() {
        return Ok(None);
    }
    let text = std::fs::read_to_string(path)
        .map_err(|e| format!("read ILP audit summary for lineage failed: {e}"))?;
    let summary: serde_json::Value = serde_json::from_str(&text)
        .map_err(|e| format!("parse ILP audit summary for lineage failed: {e}"))?;
    Ok(Some(serde_json::json!({
        "checked": summary.get("checked").cloned().unwrap_or(serde_json::Value::Null),
        "passed": summary.get("passed").cloned().unwrap_or(serde_json::Value::Null),
        "failed": summary.get("failed").cloned().unwrap_or(serde_json::Value::Null),
        "fallback_required": summary.get("fallback_required").cloned().unwrap_or(serde_json::Value::Null),
        "outcomes": summary.get("outcomes").cloned().unwrap_or_else(|| serde_json::json!({})),
        "proof_statuses": summary.get("proof_statuses").cloned().unwrap_or_else(|| serde_json::json!({})),
        "exact_search_strategies": summary.get("exact_search_strategies").cloned().unwrap_or_else(|| serde_json::json!({})),
    })))
}

pub(crate) fn audit_result_label(result: &rplan_audit::AuditResult) -> &'static str {
    match result {
        rplan_audit::AuditResult::Pass => "pass",
        rplan_audit::AuditResult::Fail => "fail",
        rplan_audit::AuditResult::PassWithWarnings => "pass-with-warnings",
    }
}

pub(crate) fn run_spectral_recursive(
    adjacency: &[Vec<usize>],
    weights: &[i64],
    k: usize,
    tolerance: f64,
    max_iters: usize,
) -> Result<(Vec<usize>, serde_json::Value), String> {
    if k == 0 {
        return Err("spectral: k must be greater than zero".to_string());
    }
    if adjacency.len() != weights.len() {
        return Err("spectral: adjacency and weight lengths must match".to_string());
    }
    let mut assignment = vec![0usize; adjacency.len()];
    let mut node_summaries = Vec::new();
    split_spectral_node(
        adjacency,
        weights,
        &(0..adjacency.len()).collect::<Vec<_>>(),
        k,
        0,
        tolerance,
        max_iters,
        &mut assignment,
        &mut node_summaries,
    )?;
    let edge_cut = count_edge_cuts_zero_based(&assignment, adjacency);
    Ok((
        assignment,
        serde_json::json!({
            "schema_version": "bisect-spectral-run-summary-v1",
            "method": "spectral",
            "max_iters": max_iters,
            "tolerance": tolerance,
            "k": k,
            "edge_cut": edge_cut,
            "nodes": node_summaries,
        }),
    ))
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn split_spectral_node(
    adjacency: &[Vec<usize>],
    weights: &[i64],
    vertices: &[usize],
    k: usize,
    district_offset: usize,
    tolerance: f64,
    max_iters: usize,
    assignment: &mut [usize],
    node_summaries: &mut Vec<serde_json::Value>,
) -> Result<(), String> {
    if k == 1 {
        for &vertex in vertices {
            assignment[vertex] = district_offset;
        }
        return Ok(());
    }
    let local_index: HashMap<usize, usize> = vertices
        .iter()
        .enumerate()
        .map(|(local, &global)| (global, local))
        .collect();
    let local_adjacency: Vec<Vec<usize>> = vertices
        .iter()
        .map(|&global| {
            adjacency[global]
                .iter()
                .filter_map(|neighbor| local_index.get(neighbor).copied())
                .collect()
        })
        .collect();
    let local_weights: Vec<i64> = vertices.iter().map(|&global| weights[global]).collect();
    let left_k = k / 2;
    let right_k = k - left_k;
    let result = bisect_apportion::spectral_bisect(
        &local_adjacency,
        &local_weights,
        bisect_apportion::SpectralConfig {
            max_iters,
            tolerance,
            target_fraction: left_k as f64 / k as f64,
        },
    )
    .map_err(|e| format!("spectral split failed: {e}"))?;
    node_summaries.push(serde_json::to_value(&result.summary).map_err(|e| e.to_string())?);

    let mut left = Vec::new();
    let mut right = Vec::new();
    for (local, &global) in vertices.iter().enumerate() {
        if result.assignment[local] == 0 {
            left.push(global);
        } else {
            right.push(global);
        }
    }
    split_spectral_node(
        adjacency,
        weights,
        &left,
        left_k,
        district_offset,
        tolerance,
        max_iters,
        assignment,
        node_summaries,
    )?;
    split_spectral_node(
        adjacency,
        weights,
        &right,
        right_k,
        district_offset + left_k,
        tolerance,
        max_iters,
        assignment,
        node_summaries,
    )
}

pub(crate) fn count_edge_cuts_zero_based(assignment: &[usize], adjacency: &[Vec<usize>]) -> usize {
    rgraph_core::undirected_edge_cut(adjacency, assignment)
        .expect("validated zero-based spectral adjacency and assignment")
}

/// Check if a state's outputs already exist and are complete.
pub fn state_already_complete(
    output_dir: &PathBuf,
    state_code: &str,
    year: &str,
    reprocess: bool,
) -> bool {
    if reprocess {
        return false;
    }
    let data_dir = output_dir
        .join(year)
        .join("states")
        .join(state_code.to_lowercase())
        .join("data");
    data_dir.join("final_assignments.json").exists()
        || data_dir.join("final_assignments.pkl").exists()
}

/// Filter configs to only those needing processing.
pub fn filter_incomplete(configs: Vec<StateConfig>) -> Vec<StateConfig> {
    configs
        .into_iter()
        .filter(|cfg| {
            !state_already_complete(&cfg.output_dir, &cfg.state_code, &cfg.year, cfg.reprocess)
        })
        .collect()
}

/// Build edge weights from a `WeightSpec` using the composable `EdgeWeighter` pipeline.
///
/// Steps applied in order:
///   1. Geographic boundary lengths (if `spec.geographic`)
///   2. Minority / VRA signal (if `spec.minority_weighting`)
///   3. Partisan signal (if `spec.partisan_shares.is_some()`)
///   4. County subdivision stickiness (if `spec.alpha_county > 1e-10`)
///
/// Returns `Ok(EdgeMap)` on success. Errors from data loading propagate as `Err(String)`.
pub(crate) fn build_edge_weights(
    spec: &WeightSpec,
    graph: &crate::adjacency_loader::LoadedGraph,
    state_code: &str,
    state_name: &str,
    year: &str,
    _output_dir: &std::path::PathBuf,
    position: i32,
) -> Result<HashMap<(usize, usize), f64>, String> {
    use crate::edge_weights::{
        ComposedWeighter, GeographicWeighter, MinorityOverrideWeighter, PartisanOverrideWeighter,
        SubdivisionWeighter,
    };

    let edges: Vec<(usize, usize)> = graph
        .adjacency
        .iter()
        .enumerate()
        .flat_map(|(i, nbrs)| nbrs.iter().filter(move |&&j| j > i).map(move |&j| (i, j)))
        .collect();

    let mut composer = ComposedWeighter::new();

    // Step 1: Geographic base weights (TIGER boundary lengths).
    if spec.geographic {
        composer = composer.push(GeographicWeighter::from_map(graph.edge_weights.clone()));
    }

    // Step 2: Minority / VRA — override variant (from scratch, historic behaviour).
    if spec.minority_weighting {
        status(
            position,
            &format!("{state_code}: VRA mode — loading demographics"),
        );
        let demo_path = std::path::Path::new("data")
            .join(year)
            .join("demographics")
            .join(format!("{state_name}_demographics_{year}.csv"));
        let demo =
            load_demographics(&demo_path).map_err(|e| format!("demographics load failed: {e}"))?;
        let minority_fracs =
            align_demographics_to_adjacency(&demo, &graph.index_to_geoid, graph.n_vertices);
        composer = composer.push(MinorityOverrideWeighter::new(
            edges.clone(),
            minority_fracs,
            0.40,
        ));
    }

    // Step 3: Partisan signal — override variant (from scratch, historic behaviour).
    if let Some(ref partisan_path) = spec.partisan_shares {
        if !partisan_path.as_os_str().is_empty() {
            status(
                position,
                &format!(
                    "{state_code}: partisan-weighted — loading {}",
                    partisan_path.display()
                ),
            );
            let dem_shares =
                load_partisan_shares(partisan_path, &graph.index_to_geoid, graph.n_vertices)
                    .map_err(|e| format!("partisan shares load failed: {e}"))?;
            composer = composer.push(PartisanOverrideWeighter::new(
                edges.clone(),
                dem_shares,
                spec.dem_threshold,
                spec.rep_threshold,
            ));
        }
    }

    // Step 4: Subdivision stickiness (T.3) — augment on whatever base is set.
    if spec.alpha_county > 1e-10 {
        composer = composer.push(SubdivisionWeighter::county_only(
            &graph.index_to_geoid,
            graph.n_vertices,
            spec.alpha_county,
        ));
    }

    // Step 5: Economic character similarity (M.9/M.1).
    if spec.economic_character {
        use crate::edge_weights::EconomicCharacterWeighter;
        use crate::lodes::{align_lodes_to_adjacency, load_lodes_wac_tract};
        status(
            position,
            &format!("{state_code}: economic-character -- loading LODES WAC"),
        );
        match load_lodes_wac_tract(state_name, year) {
            Ok(lodes_chars) if !lodes_chars.is_empty() => {
                let node_chars =
                    align_lodes_to_adjacency(&lodes_chars, &graph.index_to_geoid, graph.n_vertices);
                composer =
                    composer.push(EconomicCharacterWeighter::new(node_chars, spec.econ_alpha));
            }
            Ok(_) => {
                eprintln!(
                    "WARNING: LODES WAC not found for {state_name} {year}. \
                           Run: bisect fetch --type lodes --year {year} --states {state_code}"
                );
            }
            Err(e) => {
                eprintln!(
                    "WARNING: LODES WAC load error: {e}. Falling back to geographic weights."
                );
            }
        }
    }

    // Step 5b: Housing character similarity (M.3).
    if spec.housing_character {
        use crate::edge_weights::HousingCharacterWeighter;
        use crate::housing::{align_housing_to_adjacency, load_acs_housing_tract};
        status(
            position,
            &format!("{state_code}: housing-character -- loading ACS housing"),
        );
        match load_acs_housing_tract(state_name, year) {
            Ok(housing_chars) if !housing_chars.is_empty() => {
                let node_chars = align_housing_to_adjacency(
                    &housing_chars,
                    &graph.index_to_geoid,
                    graph.n_vertices,
                );
                composer =
                    composer.push(HousingCharacterWeighter::new(node_chars, spec.econ_alpha));
            }
            Ok(_) => {
                eprintln!(
                    "WARNING: ACS housing not found for {state_name} {year}. \
                           Run: bisect fetch --type acs-housing --year {year} --states {state_code}"
                );
            }
            Err(e) => {
                eprintln!(
                    "WARNING: ACS housing load error: {e}. Falling back to geographic weights."
                );
            }
        }
    }

    // Step 6: Zone co-membership (M.6).
    if spec.zone_membership {
        // Zone data loading deferred to Phase 2 — TIGER/EIA spatial join not yet implemented.
        // For now: warn and skip gracefully. The weighter struct and CLI flag are wired.
        eprintln!(
            "WARNING: --weights-override zone-membership requires TIGER school district \
                   and EIA Form 861 spatial join (Phase 2). Falling back to geographic weights."
        );
    }

    // If nothing was added to the composer, fall back to geographic weights
    if composer.is_empty() {
        return Ok(graph.edge_weights.clone());
    }

    Ok(composer.apply())
}

/// Apply COI (Communities of Interest) weights to edge weights.
///
/// Loads a JSON file mapping GEOID -> weight (0.0-1.0). For each edge (u, v),
/// multiplies the edge weight by sqrt(w_u * w_v) (geometric mean of endpoint weights).
/// Tracts not in the COI file get weight 1.0 (no modification).
///
/// The geometric mean ensures that if both endpoints of an edge are in the same
/// community (high weight), the edge is strengthened and METIS will avoid cutting it.
pub fn apply_coi_weights(
    mut edge_weights: HashMap<(usize, usize), f64>,
    coi_path: &std::path::Path,
    index_to_geoid: &HashMap<usize, String>,
) -> Result<HashMap<(usize, usize), f64>, String> {
    let content = std::fs::read_to_string(coi_path)
        .map_err(|e| format!("cannot read COI weights file {}: {e}", coi_path.display()))?;
    let coi_map: HashMap<String, f64> = serde_json::from_str(&content)
        .map_err(|e| format!("cannot parse COI weights JSON: {e}"))?;

    // Build a geoid -> weight lookup by index
    let get_weight = |idx: usize| -> f64 {
        index_to_geoid
            .get(&idx)
            .and_then(|geoid| coi_map.get(geoid))
            .copied()
            .unwrap_or(1.0)
    };

    for (&(u, v), weight) in edge_weights.iter_mut() {
        let w_u = get_weight(u);
        let w_v = get_weight(v);
        let factor = (w_u * w_v).sqrt();
        *weight *= factor;
    }

    Ok(edge_weights)
}

/// Load per-tract area (m²) and external perimeter (m) from TIGER shapefiles.
/// Used by CompactBisect to compute Polsby-Popper at each bisection level.
///
/// Area: ALAND field from TIGER (accurate, in m²).
/// External perimeter: approximated as 2√(π·ALAND) − Σ(shared edge weights).
/// The circular approximation is slightly off for elongated tracts but preserves
/// relative compactness rankings within the same subgraph.
///
/// Returns (vertex_areas, vertex_ext_perimeters) aligned to adjacency indices.
/// Returns empty vecs if TIGER file is not found (CompactBisect gracefully degrades).
pub fn load_tiger_geometry(
    state_code: &str,
    year: &str,
    index_to_geoid: &std::collections::HashMap<usize, String>,
    adjacency: &[Vec<usize>],
    edge_weights: &std::collections::HashMap<(usize, usize), f64>,
) -> (Vec<f64>, Vec<f64>) {
    let state_fips = state_code_to_fips(&state_code.to_uppercase()).map(|s| s.to_string());

    // Try TIGER path: data/{year}/tiger/tracts/tl_{year}_{fips}_tract/
    let tiger_path = state_fips.as_deref().and_then(|fips| {
        let p = std::path::PathBuf::from("data")
            .join(year)
            .join("tiger")
            .join("tracts")
            .join(format!("tl_{year}_{fips}_tract"))
            .join(format!("tl_{year}_{fips}_tract.shp"));
        if p.exists() {
            Some(p)
        } else {
            None
        }
    });

    let tiger_path = match tiger_path {
        Some(p) => p,
        None => {
            eprintln!("[compact-bisect] TIGER not found for {state_code} {year} — no geometry");
            return (Vec::new(), Vec::new());
        }
    };

    // Read tract records: geoid → aland
    let records = match bisect_data::read_tiger_tracts(&tiger_path) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("[compact-bisect] TIGER read failed: {e}");
            return (Vec::new(), Vec::new());
        }
    };

    let geoid_to_aland: std::collections::HashMap<String, f64> = records
        .iter()
        .map(|r| (r.geoid.clone(), r.aland as f64))
        .collect();

    let n = adjacency.len();
    let mut vertex_areas = vec![0.0f64; n];
    let mut vertex_ext_perimeters = vec![0.0f64; n];

    for (&idx, geoid) in index_to_geoid {
        if idx >= n {
            continue;
        }
        let aland = *geoid_to_aland.get(geoid).unwrap_or(&0.0);
        vertex_areas[idx] = aland;

        // Circular approximation of total perimeter: 2√(π·A)
        let total_perim_approx = if aland > 0.0 {
            2.0 * (std::f64::consts::PI * aland).sqrt()
        } else {
            0.0
        };

        // Subtract shared boundaries (in metres from adjacency edge weights)
        let shared: f64 = adjacency[idx]
            .iter()
            .map(|&j| {
                let key = (idx.min(j), idx.max(j));
                edge_weights.get(&key).copied().unwrap_or(0.0)
            })
            .sum();

        vertex_ext_perimeters[idx] = (total_perim_approx - shared).max(0.0);
    }

    (vertex_areas, vertex_ext_perimeters)
}
