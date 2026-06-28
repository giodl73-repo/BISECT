use super::*;

// ── ILP Exact Redistricting (U.6) ────────────────────────────────────────────

/// Split a subgraph into two balanced parts using ILP (exact redistricting).
///
/// Phase 1: build the ILP formulation and call `solve`.
/// Since the current backends return `plan: None`, fall back to METIS (same as
/// the standard `split_subgraph` path with ncon=1).
///
/// When `tract_indices.len() > max_tracts`, skips the ILP entirely and falls
/// back to METIS directly (size guard).
pub fn split_subgraph_ilp(
    adjacency: &[Vec<usize>],
    vertex_weights: &[i64],
    tract_indices: &HashSet<usize>,
    balance_tolerance: f64,
    method: crate::args::IlpMethod,
    fallback: crate::args::IlpFallback,
    time_limit_secs: u64,
    optimality_gap: f64,
    max_tracts: usize,
    solve_report_path: Option<&Path>,
) -> Result<(HashSet<usize>, HashSet<usize>), String> {
    // Size guard: fall back to METIS when the subgraph is too large.
    if tract_indices.len() > max_tracts {
        if matches!(fallback, crate::args::IlpFallback::Error) {
            return Err(format!(
                "ILP skipped for node ({} tracts > {} limit) and --ilp-fallback=error",
                tract_indices.len(),
                max_tracts
            ));
        }
        eprintln!(
            "WARNING: ILP solver skipped for node ({} tracts > {} limit). Falling back to METIS.",
            tract_indices.len(),
            max_tracts,
        );
        let empty_ew: HashMap<(usize, usize), f64> = HashMap::new();
        return split_subgraph(
            adjacency,
            vertex_weights,
            1,
            &empty_ew,
            tract_indices,
            balance_tolerance,
            10,
            None,
            None,
            None,
        );
    }

    // Build local index mapping: local -> global (sorted for determinism).
    let mut sorted: Vec<usize> = tract_indices.iter().copied().collect();
    sorted.sort_unstable();

    // Build local adjacency (local indices only).
    let local_adj: Vec<Vec<usize>> = sorted
        .iter()
        .map(|&g| {
            adjacency[g]
                .iter()
                .filter(|&&nb| tract_indices.contains(&nb))
                .map(|&nb| sorted.partition_point(|&x| x < nb))
                .collect()
        })
        .collect();

    // Build local population array (one value per local node).
    let local_pop: Vec<i64> = sorted.iter().map(|&g| vertex_weights[g].max(1)).collect();

    // Phase 1: build formulation and solve (current methods return plan: None).
    let formulation = bisect_ilp::build_formulation(&local_adj, &local_pop, 2, balance_tolerance);
    let solver = match method {
        crate::args::IlpMethod::FormulationOnly => bisect_ilp::IlpSolver::FormulationOnly,
        crate::args::IlpMethod::BranchAndCut => bisect_ilp::IlpSolver::BranchAndCut {
            mode: bisect_ilp::BranchAndCutMode::LazyCallback,
            incumbent_assignment: None,
            solver_name: None,
        },
        crate::args::IlpMethod::IterativeSeparation => bisect_ilp::IlpSolver::BranchAndCut {
            mode: bisect_ilp::BranchAndCutMode::IterativeSeparation,
            incumbent_assignment: None,
            solver_name: None,
        },
    };
    let result = bisect_ilp::solve(
        &formulation,
        &local_adj,
        &local_pop,
        2,
        balance_tolerance,
        solver,
        optimality_gap,
    );

    if let Some(path) = solve_report_path {
        let lp_path = path.with_extension("lp");
        write_ilp_master_lp(&lp_path, &local_adj, &local_pop, 2, balance_tolerance)?;
        let model_artifact = ilp_model_artifact_for_report(path, &lp_path)?;
        write_ilp_solve_report(
            path,
            formulation.clone(),
            result.clone(),
            Some(model_artifact),
        )?;
        bisect_ilp::verify_model_artifact_for_report(path)
            .map_err(|e| format!("verify ilp model artifact: {e}"))?;
    }

    if result.plan.is_some() {
        // Phase 2 (future): use the ILP plan directly.
        // Convert local district assignments -> global left/right sets.
        let plan = result.plan.unwrap();
        let mut left = HashSet::new();
        let mut right = HashSet::new();
        for (local_idx, district) in plan {
            let global = sorted[local_idx];
            if district == 0 {
                left.insert(global);
            } else {
                right.insert(global);
            }
        }
        Ok((left, right))
    } else {
        if matches!(fallback, crate::args::IlpFallback::Error) {
            return Err(format!(
                "ILP {method}: no solver plan returned (status={:?}) and --ilp-fallback=error",
                result.status
            ));
        }
        eprintln!(
            "ILP {method}: no solver plan returned (status={:?}) — falling back to METIS \
             (vars={}, constraints={}, time_limit={}s gap={:.3})",
            result.status,
            formulation.n_variables(),
            formulation.n_constraints,
            time_limit_secs,
            optimality_gap,
        );
        let empty_ew: HashMap<(usize, usize), f64> = HashMap::new();
        split_subgraph(
            adjacency,
            vertex_weights,
            1,
            &empty_ew,
            tract_indices,
            balance_tolerance,
            10,
            None,
            None,
            None,
        )
    }
}

/// Run the full bisection tree using ILP at each node.
///
/// Structurally identical to `run_all_splits_sa` / `run_all_splits_bfs` but
/// calls `split_subgraph_ilp` at each bisection node.
/// Phase 1: ILP is FormulationOnly so each node falls back to METIS.
pub fn run_all_splits_ilp(
    adjacency: &[Vec<usize>],
    vertex_weights: &[i64],
    _edge_weights: &HashMap<(usize, usize), f64>,
    num_districts: usize,
    balance_tolerance: f64,
    method: crate::args::IlpMethod,
    fallback: crate::args::IlpFallback,
    time_limit_secs: u64,
    optimality_gap: f64,
    max_tracts: usize,
    solve_report_dir: Option<&Path>,
) -> Result<HashMap<usize, usize>, String> {
    let n = adjacency.len();

    if num_districts == 1 {
        return Ok((0..n).map(|i| (i, 1)).collect());
    }

    let tree = BisectionTree::from_k(num_districts);
    let mut node_tracts: HashMap<String, HashSet<usize>> = HashMap::new();
    node_tracts.insert(String::new(), (0..n).collect());

    for depth in 0..tree.max_depth {
        let nodes_at_depth: Vec<_> = tree.nodes_at_depth(depth).into_iter().cloned().collect();

        let nodes_with_tracts: Vec<(bisect_core::BisectionNode, HashSet<usize>)> = nodes_at_depth
            .into_iter()
            .filter_map(|node| node_tracts.remove(&node.path).map(|tracts| (node, tracts)))
            .collect();

        let split_results: Vec<(String, HashSet<usize>, HashSet<usize>)> = nodes_with_tracts
            .into_par_iter()
            .map(|(node, tracts)| {
                let node_ufactor = 1.0 + balance_tolerance / node.k as f64;
                let solve_report_path = solve_report_dir.map(|dir| {
                    let node_name = if node.path.is_empty() {
                        "root".to_string()
                    } else {
                        node.path.clone()
                    };
                    dir.join(format!("depth_{depth:02}"))
                        .join(format!("node_{node_name}.json"))
                });
                let (left, right) = split_subgraph_ilp(
                    adjacency,
                    vertex_weights,
                    &tracts,
                    node_ufactor,
                    method,
                    fallback,
                    time_limit_secs,
                    optimality_gap,
                    max_tracts,
                    solve_report_path.as_deref(),
                )
                .map_err(|e| format!("depth {} node '{}' (ilp): {e}", depth, node.path))?;
                Ok((node.path, left, right))
            })
            .collect::<Result<Vec<_>, String>>()?;

        let mut sorted_results = split_results;
        sorted_results.sort_by_key(|(path, _, _)| path.clone());
        for (path, left, right) in sorted_results {
            node_tracts.insert(format!("{path}0"), left);
            node_tracts.insert(format!("{path}1"), right);
        }
    }

    let mut leaves: Vec<(String, HashSet<usize>)> = node_tracts.into_iter().collect();
    leaves.sort_by_key(|(path, _)| (path.len(), path.clone()));

    let mut assignments: HashMap<usize, usize> = HashMap::new();
    for (district_id, (_, tracts)) in leaves.into_iter().enumerate() {
        for tract in tracts {
            assignments.insert(tract, district_id + 1);
        }
    }

    if assignments.len() != n {
        return Err(format!(
            "ilp bisection incomplete: {}/{n} tracts assigned",
            assignments.len()
        ));
    }
    if let Some(dir) = solve_report_dir {
        crate::ilp_audit::write_ilp_audit_summary_for_dir(dir, &dir.join("audit-summary.json"))?;
    }
    Ok(assignments)
}
