use super::*;

// ── ILP tests (L0/L1) ─────────────────────────────────────────────────────

/// L0: `--partition-mode ilp` parses to PartitionMode::Ilp.
#[test]
fn ilp_structure_mode_parses() {
    use crate::args::{IlpFallback, IlpMethod, PartitionMode};
    use clap::ValueEnum;
    let parsed = PartitionMode::from_str("ilp", true).expect("'ilp' must be a valid PartitionMode");
    assert_eq!(
        parsed,
        PartitionMode::Ilp,
        "parsed PartitionMode must be Ilp"
    );
    assert_eq!(
        IlpMethod::from_str("formulation-only", true).unwrap(),
        IlpMethod::FormulationOnly
    );
    assert_eq!(
        IlpMethod::from_str("branch-and-cut", true).unwrap(),
        IlpMethod::BranchAndCut
    );
    assert_eq!(
        IlpMethod::from_str("iterative-separation", true).unwrap(),
        IlpMethod::IterativeSeparation
    );
    assert_eq!(
        IlpFallback::from_str("metis", true).unwrap(),
        IlpFallback::Metis
    );
    assert_eq!(
        IlpFallback::from_str("error", true).unwrap(),
        IlpFallback::Error
    );
}

/// L1: 4x4 grid (16 tracts) with max_tracts=5 — size guard fires, falls back to METIS.
/// Verifies: returns Ok, all 16 tracts assigned, two non-empty partitions.
#[test]
fn ilp_size_guard_fallback() {
    let (adj, pop) = small_grid(4, 4); // 16 tracts
    let tracts: HashSet<usize> = (0..16).collect();
    let result = split_subgraph_ilp(
        &adj,
        &pop,
        &tracts,
        1.05, // balance_tolerance
        crate::args::IlpMethod::FormulationOnly,
        crate::args::IlpFallback::Metis,
        60,   // time_limit_secs
        0.01, // optimality_gap
        5,    // max_tracts: 16 > 5 => guard fires
        None,
    );
    let (left, right) = result.expect("size-guard fallback must not fail");
    assert!(
        !left.is_empty(),
        "left partition must be non-empty after METIS fallback"
    );
    assert!(
        !right.is_empty(),
        "right partition must be non-empty after METIS fallback"
    );
    assert_eq!(
        left.len() + right.len(),
        16,
        "all 16 tracts must be assigned"
    );
    // Verify disjoint
    for t in &left {
        assert!(!right.contains(t), "partitions must be disjoint");
    }
}

/// L1: 4-node path graph — Phase 1 FormulationOnly falls back to METIS,
/// returns a valid 2-partition of all 4 nodes.
#[test]
fn ilp_phase1_produces_valid_plan() {
    // Path: 0-1-2-3  (4 nodes, well within max_tracts=500)
    let adj = vec![vec![1usize], vec![0, 2], vec![1, 3], vec![2usize]];
    let pop = vec![100i64; 4];
    let tracts: HashSet<usize> = (0..4).collect();
    let result = split_subgraph_ilp(
        &adj,
        &pop,
        &tracts,
        1.05, // balance_tolerance
        crate::args::IlpMethod::FormulationOnly,
        crate::args::IlpFallback::Metis,
        300,  // time_limit_secs
        0.01, // optimality_gap
        500,  // max_tracts (no guard)
        None,
    );
    let (left, right) = result.expect("ilp phase1 must not fail");
    assert!(!left.is_empty(), "left partition must be non-empty");
    assert!(!right.is_empty(), "right partition must be non-empty");
    assert_eq!(left.len() + right.len(), 4, "all 4 tracts must be covered");
    for t in &left {
        assert!(!right.contains(t), "partitions must be disjoint");
    }
}

#[test]
fn ilp_fallback_error_rejects_size_guard() {
    let (adj, pop) = small_grid(4, 4);
    let tracts: HashSet<usize> = (0..16).collect();
    let err = split_subgraph_ilp(
        &adj,
        &pop,
        &tracts,
        1.05,
        crate::args::IlpMethod::FormulationOnly,
        crate::args::IlpFallback::Error,
        60,
        0.01,
        5,
        None,
    )
    .expect_err("fallback=error must reject size-guard fallback");
    assert!(err.contains("--ilp-fallback=error"));
}

#[test]
fn ilp_fallback_error_rejects_missing_solver_plan() {
    let adj = vec![vec![1usize], vec![0, 2], vec![1usize]];
    let pop = vec![100i64; 3];
    let tracts: HashSet<usize> = (0..3).collect();
    let err = split_subgraph_ilp(
        &adj,
        &pop,
        &tracts,
        0.005,
        crate::args::IlpMethod::BranchAndCut,
        crate::args::IlpFallback::Error,
        300,
        0.01,
        500,
        None,
    )
    .expect_err("fallback=error must reject missing solver plan");
    assert!(err.contains("no solver plan returned"));
    assert!(err.contains("--ilp-fallback=error"));
}

/// L0: ILP formulation variable counts for 4-node path graph, k=2.
/// n_binary_x = n*k = 4*2 = 8
/// n_binary_z = |E| = 3  (path 0-1-2-3 has 3 edges)
/// n_flow_vars = 2*|E|*k = 2*3*2 = 12
#[test]
fn ilp_formulation_counts_correct() {
    let adj = vec![vec![1usize], vec![0, 2], vec![1, 3], vec![2usize]];
    let pop = vec![100i64; 4];
    let formulation = bisect_ilp::build_formulation(&adj, &pop, 2, 0.005);
    assert_eq!(
        formulation.n_binary_x, 8,
        "n_binary_x should be n*k = 4*2 = 8"
    );
    assert_eq!(
        formulation.n_binary_z, 3,
        "n_binary_z should be |E| = 3 for path 0-1-2-3"
    );
    assert_eq!(
        formulation.n_flow_vars, 12,
        "n_flow_vars should be 2*|E|*k = 2*3*2 = 12"
    );
}

/// L1: run_all_splits_ilp on a 4x4 grid — all 16 tracts assigned into 2 districts.
#[test]
fn run_all_splits_ilp_produces_complete_assignment() {
    let (adj, pop) = small_grid(4, 4); // 16 tracts
    let ew: HashMap<(usize, usize), f64> = HashMap::new();
    let result = run_all_splits_ilp(
        &adj,
        &pop,
        &ew,
        2,    // num_districts
        0.05, // balance_tolerance
        crate::args::IlpMethod::FormulationOnly,
        crate::args::IlpFallback::Metis,
        60,   // time_limit_secs
        0.01, // optimality_gap
        500,  // max_tracts (no size guard)
        None,
    );
    let assignments = result.expect("run_all_splits_ilp must succeed on 4x4 grid");
    assert_eq!(assignments.len(), 16, "all 16 tracts must be assigned");
    let districts: std::collections::HashSet<usize> = assignments.values().copied().collect();
    assert_eq!(districts.len(), 2, "must produce exactly 2 districts");
}

#[test]
fn run_all_splits_ilp_writes_solve_report() {
    let (adj, pop) = small_grid(2, 2);
    let ew: HashMap<(usize, usize), f64> = HashMap::new();
    let tmp = tempfile::TempDir::new().unwrap();
    let result = run_all_splits_ilp(
        &adj,
        &pop,
        &ew,
        2,
        0.05,
        crate::args::IlpMethod::BranchAndCut,
        crate::args::IlpFallback::Metis,
        60,
        0.01,
        500,
        Some(tmp.path()),
    );
    result.expect("ilp run must succeed");
    let report_path = tmp.path().join("depth_00").join("node_root.json");
    let lp_path = tmp.path().join("depth_00").join("node_root.lp");
    let summary_path = tmp.path().join("audit-summary.json");
    let json = std::fs::read_to_string(&report_path).expect("read ilp solve report");
    let value: serde_json::Value = serde_json::from_str(&json).expect("parse report json");
    assert_eq!(value["schema_version"], "ilp-solve-report-v1");
    assert_eq!(value["model_artifact"]["format"], "cplex-lp");
    assert_eq!(value["model_artifact"]["path"], "node_root.lp");
    assert_eq!(
        value["model_artifact"]["sha256"].as_str().unwrap().len(),
        64
    );
    assert_eq!(value["audit_summary"]["outcome"], "exact-plan");
    assert_eq!(value["audit_summary"]["proof_status"], "proved-optimal");
    assert_eq!(value["audit_summary"]["has_model_artifact"], true);
    assert_eq!(value["audit_summary"]["fallback_required"], false);
    assert_eq!(value["result"]["status"]["status"], "optimal");
    assert_eq!(value["result"]["optimal_ec"], 2);
    assert_eq!(value["result"]["branch_and_cut"]["cut_count"], 0);
    assert_eq!(
        value["result"]["branch_and_cut"]["exact_search"]["search_strategy"],
        "k2-branch-and-bound"
    );
    assert!(
        value["result"]["branch_and_cut"]["exact_search"]["nodes_visited"]
            .as_u64()
            .unwrap()
            > 0
    );

    let lp = std::fs::read_to_string(&lp_path).expect("read ilp master LP");
    assert!(lp.contains("BISECT U.16 branch-and-cut master LP"));
    assert!(lp.contains("Minimize"));
    assert!(lp.contains("Binary"));

    let verified = bisect_ilp::verify_model_artifact_for_report(&report_path)
        .expect("solve report model artifact should verify");
    assert_eq!(verified.format, "cplex-lp");
    assert_eq!(verified.path, lp_path);

    let summary_json = std::fs::read_to_string(&summary_path).expect("read ilp audit summary");
    let summary: serde_json::Value =
        serde_json::from_str(&summary_json).expect("parse ilp audit summary");
    assert_eq!(summary["checked"], 1);
    assert_eq!(summary["passed"], 1);
    assert_eq!(summary["failed"], 0);
    assert_eq!(summary["outcomes"]["exact-plan"], 1);
    assert_eq!(summary["proof_statuses"]["proved-optimal"], 1);
}
