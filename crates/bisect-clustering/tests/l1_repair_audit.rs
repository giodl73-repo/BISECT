use std::collections::BTreeMap;

use bisect_clustering::assign::{capacity_cluster_repaired, ClusterConfig, ClusterStatus};
use bisect_clustering::fixtures;
use bisect_clustering::repair::{repair_to_valid_small, RepairStatus};
use rplan_core::{
    CanonicalOrder, DistrictPlan, EdgeKind, EdgeSemantics, PlanUnitIndex, RplanContext,
    SourceHashes, UnitEdge, UnitGraph, UnitKind, DISTRICT_PLAN_SCHEMA_VERSION, RCTX_VERSION,
};

#[test]
fn exhaustive_small_repair_fixes_disconnected_assignment() {
    let (adjacency, assignment) = fixtures::disconnected_assignment();
    let weights = vec![100, 100, 100, 100];
    let repaired = repair_to_valid_small(&adjacency, &weights, &assignment, 2, 0.01).unwrap();

    assert_eq!(repaired.assignment, vec![0, 0, 1, 1]);
    assert_eq!(repaired.changed_units, 2);
}

#[test]
fn repaired_grid_plan_passes_rplan_audit() {
    let fixture = fixtures::grid_3x3_k3();
    let result = capacity_cluster_repaired(
        &fixture.adjacency,
        &fixture.weights,
        ClusterConfig {
            k: fixture.k,
            tolerance: fixture.tolerance,
        },
    )
    .unwrap();

    assert_eq!(result.status, ClusterStatus::Valid);
    assert!(matches!(
        result.summary.repair_status,
        RepairStatus::NotNeeded | RepairStatus::Repaired
    ));

    let plan = district_plan(&fixture.weights, &result.assignment, fixture.k);
    let mut context = rplan_context(&fixture.adjacency, &fixture.weights, plan.units.clone());
    context.context_hash = context.compute_context_hash().unwrap();
    let profile = rplan_audit::LegalProfile {
        population_tolerance: rplan_audit::PopulationToleranceRule::Percent {
            max_deviation_percent: fixture.tolerance * 100.0,
        },
        ..rplan_audit::LegalProfile::us_congressional_project_v1(2020)
    };
    let certificate = rplan_audit::audit_plan_with_lineage(
        &plan,
        Some(&context),
        &profile,
        rplan_audit::RuntimeProvenance {
            binary_name: "bisect-clustering-test".to_string(),
            binary_version: "0.1.0".to_string(),
            ..Default::default()
        },
        &[
            rplan_audit::AuditConstraint::Population,
            rplan_audit::AuditConstraint::Contiguity,
        ],
        "2026-05-11T00:00:00Z",
        Some(
            result
                .summary
                .algorithm_lineage("0.1.0", Vec::new())
                .unwrap(),
        ),
    )
    .unwrap();

    assert_eq!(certificate.result, rplan_audit::AuditResult::Pass);
    rplan_audit::verify_audit_certificate(&certificate, Some(&plan), Some(&context)).unwrap();
}

fn district_plan(weights: &[i64], assignment: &[usize], k: usize) -> DistrictPlan {
    let mut units = PlanUnitIndex {
        unit_kind: UnitKind::Imported,
        state: Some("TT".to_string()),
        year: Some(2020),
        canonical_order: CanonicalOrder::ExplicitUnitIds,
        unit_ids: (0..weights.len()).map(|idx| format!("u{idx}")).collect(),
        unit_universe_hash: String::new(),
        source_id: Some("t15-fixture".to_string()),
    };
    units.unit_universe_hash = units.compute_unit_universe_hash().unwrap();
    DistrictPlan {
        schema_version: DISTRICT_PLAN_SCHEMA_VERSION.to_string(),
        units,
        assignment: assignment.iter().map(|&district| district as u32).collect(),
        k,
        display_labels: (0..k).map(|district| district.to_string()).collect(),
        allow_empty_districts: false,
    }
}

fn rplan_context(adjacency: &[Vec<usize>], weights: &[i64], units: PlanUnitIndex) -> RplanContext {
    RplanContext {
        rctx_version: RCTX_VERSION.to_string(),
        context_hash: String::new(),
        units,
        graph: Some(UnitGraph {
            edge_semantics: EdgeSemantics::Undirected,
            adjacency: adjacency
                .iter()
                .map(|neighbors| {
                    neighbors
                        .iter()
                        .map(|&to| UnitEdge {
                            to: to as u32,
                            kind: EdgeKind::Boundary,
                            weight: None,
                        })
                        .collect()
                })
                .collect(),
        }),
        populations: Some(weights.to_vec()),
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
