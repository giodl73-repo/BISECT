use super::*;
#[derive(Debug,Clone,serde::Serialize)]
pub struct IlpNodeReport {pub depth:usize,pub path:String,pub unit_indices:Vec<usize>,pub metis_multiplier:f64,pub ilp_population_tolerance:f64,pub stage:String,pub report_json:Option<String>,pub lp_text:Option<String>}
pub type IlpNodeObserver<'a> = dyn Fn(IlpNodeReport)->Result<(),String>+Sync+'a;

// ── ILP Exact Redistricting (U.6) ────────────────────────────────────────────

/// Split a subgraph into two balanced parts using ILP (exact redistricting).
///
/// Build the ILP formulation and call the shared solver. Bounded branch-and-cut
/// can return a two-district plan. When no plan is returned, apply the declared
/// METIS fallback (the standard `split_subgraph` path with ncon=1) or error.
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
    // Preserve this legacy helper's parameter interpretation. The tree
    // dispatcher below supplies distinct, explicitly typed numerical roles.
    split_subgraph_ilp_with_population_tolerance(adjacency,vertex_weights,tract_indices,balance_tolerance,balance_tolerance,method,fallback,time_limit_secs,optimality_gap,max_tracts,solve_report_path,None)
}

fn split_subgraph_ilp_with_population_tolerance(
    adjacency:&[Vec<usize>],vertex_weights:&[i64],tract_indices:&HashSet<usize>,
    balance_tolerance:f64,ilp_population_tolerance:f64,
    method:crate::args::IlpMethod,fallback:crate::args::IlpFallback,
    time_limit_secs:u64,optimality_gap:f64,max_tracts:usize,solve_report_path:Option<&Path>,
    observation:Option<(&IlpNodeObserver<'_>,usize,&str)>,
)->Result<(HashSet<usize>,HashSet<usize>),String>{
    // Size guard: fall back to METIS when the subgraph is too large.
    if tract_indices.len() > max_tracts {
        if let Some((observe,depth,path))=observation {let mut unit_indices:Vec<_>=tract_indices.iter().copied().collect();unit_indices.sort_unstable();observe(IlpNodeReport{depth,path:path.into(),unit_indices,metis_multiplier:balance_tolerance,ilp_population_tolerance,stage:"size-guard-skipped".into(),report_json:None,lp_text:None})?;}
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

    // Formulation-only and unsupported sizes can return no plan; bounded
    // branch-and-cut methods can return an actual local assignment.
    let formulation = bisect_ilp::build_formulation(&local_adj, &local_pop, 2, ilp_population_tolerance);
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
        ilp_population_tolerance,
        solver,
        optimality_gap,
    );
    if let Some((observe,depth,path))=observation {
        use sha2::{Digest,Sha256};
        // Bound observation before allocating potentially large model text.
        if formulation.n_variables()>200000{return Err("ILP observation model exceeds 200,000 variables".into());}
        let lp=bisect_ilp::master_lp_string(&local_adj,&local_pop,2,ilp_population_tolerance).map_err(|e|e.to_string())?;
        let name=if path.is_empty(){"root"}else{path};
        let report=bisect_ilp::solve_report_json_with_model_artifact(formulation.clone(),result.clone(),bisect_ilp::IlpModelArtifact{format:"cplex-lp".into(),path:format!("node_{name}.lp"),sha256:format!("{:x}",Sha256::digest(lp.as_bytes()))}).map_err(|e|e.to_string())?;
        observe(IlpNodeReport{depth,path:path.into(),unit_indices:sorted.clone(),metis_multiplier:balance_tolerance,ilp_population_tolerance,stage:"native-solver-attempt".into(),report_json:Some(report),lp_text:Some(lp)})?;
    }

    if let Some(path) = solve_report_path {
        let lp_path = path.with_extension("lp");
        write_ilp_master_lp(&lp_path, &local_adj, &local_pop, 2, ilp_population_tolerance)?;
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
        // Use the returned solver plan directly.
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
/// Each node uses the selected shared solver and declared fallback policy.
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
    run_all_splits_ilp_with_observer(adjacency,vertex_weights,_edge_weights,num_districts,balance_tolerance,method,fallback,time_limit_secs,optimality_gap,max_tracts,solve_report_dir,None)
}
pub fn run_all_splits_ilp_with_observer(
    adjacency:&[Vec<usize>],vertex_weights:&[i64],_edge_weights:&HashMap<(usize,usize),f64>,num_districts:usize,balance_tolerance:f64,
    method:crate::args::IlpMethod,fallback:crate::args::IlpFallback,time_limit_secs:u64,optimality_gap:f64,max_tracts:usize,
    solve_report_dir:Option<&Path>,observer:Option<&IlpNodeObserver<'_>>,
)->Result<HashMap<usize,usize>,String>{
    run_ilp_dispatch(adjacency,vertex_weights,_edge_weights,num_districts,balance_tolerance,method,fallback,time_limit_secs,optimality_gap,max_tracts,solve_report_dir,observer).map(|(assignments,_)|assignments)
}

/// Return the number of solve reports written by this invocation, rather than
/// counting potentially stale files left in a caller-owned report directory.
pub fn run_all_splits_ilp_with_report_count(
    adjacency:&[Vec<usize>],vertex_weights:&[i64],edge_weights:&HashMap<(usize,usize),f64>,num_districts:usize,balance_tolerance:f64,
    method:crate::args::IlpMethod,fallback:crate::args::IlpFallback,time_limit_secs:u64,optimality_gap:f64,max_tracts:usize,
    solve_report_dir:Option<&Path>,
)->Result<(HashMap<usize,usize>,usize),String>{
    run_ilp_dispatch(adjacency,vertex_weights,edge_weights,num_districts,balance_tolerance,method,fallback,time_limit_secs,optimality_gap,max_tracts,solve_report_dir,None)
}

fn run_ilp_dispatch(
    adjacency:&[Vec<usize>],vertex_weights:&[i64],_edge_weights:&HashMap<(usize,usize),f64>,num_districts:usize,balance_tolerance:f64,
    method:crate::args::IlpMethod,fallback:crate::args::IlpFallback,time_limit_secs:u64,optimality_gap:f64,max_tracts:usize,
    solve_report_dir:Option<&Path>,observer:Option<&IlpNodeObserver<'_>>,
)->Result<(HashMap<usize,usize>,usize),String>{
    let n = adjacency.len();
    let written_reports=std::sync::Mutex::new(Vec::new());

    if num_districts == 1 {
        return Ok(((0..n).map(|i| (i, 1)).collect(),0));
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
                let writes_report=solve_report_path.is_some()&&tracts.len()<=max_tracts;
                let (left, right) = split_subgraph_ilp_with_population_tolerance(
                    adjacency,
                    vertex_weights,
                    &tracts,
                    node_ufactor,
                    balance_tolerance / node.k as f64,
                    method,
                    fallback,
                    time_limit_secs,
                    optimality_gap,
                    max_tracts,
                    solve_report_path.as_deref(),
                    observer.map(|observe|(observe,depth,node.path.as_str())),
                )
                .map_err(|e| format!("depth {} node '{}' (ilp): {e}", depth, node.path))?;
                if writes_report{written_reports.lock().map_err(|_|"ILP report tracking lock poisoned")?.push(solve_report_path.unwrap());}
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
    let reports=written_reports.into_inner().map_err(|_|"ILP report tracking lock poisoned")?;let report_count=reports.len();
    if let Some(dir) = solve_report_dir.filter(|_|report_count>0) {
        crate::ilp_audit::write_ilp_audit_summary_for_reports(dir,reports, &dir.join("audit-summary.json"))?;
    }
    Ok((assignments,report_count))
}
