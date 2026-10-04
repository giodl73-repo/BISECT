use super::*;

// ── PercentileSweep ───────────────────────────────────────────────────────────

/// Run `n_seeds` independent bisections from the SHA-256 seed walk,
/// collect their edge cuts, and return the plan at rank `floor(p * n_seeds)`.
///
/// p=0.0 → minimum EC (equivalent to ConvergenceSweep without the non-improving stop).
/// p=0.5 → median EC (the "typical" plan within the bisection seed space).
/// p=1.0 → maximum EC (least compact valid plan).
///
/// NOTE: The bisection seed space and the ReCom ensemble space are different
/// distributions.  p=0.5 here targets the median of the *bisection family*, not
/// the median of all valid plans (which would require TargetedSweep).
pub fn run_all_splits_percentile(
    adjacency: &[Vec<usize>],
    vertex_weights: &[i64],
    edge_weights: &HashMap<(usize, usize), f64>,
    num_districts: usize,
    balance_tolerance: f64,
    niter: u32,
    base_seed: u64,
    n_seeds: usize,
    p: f64,
    _intermediate_dir: Option<&Path>,
) -> Result<HashMap<usize, usize>, String> {
    run_all_splits_percentile_tuned(adjacency,vertex_weights,edge_weights,num_districts,balance_tolerance,niter,base_seed,n_seeds,p,"cut",1).map(|(plan,_)|plan)
}

/// Preserve the seed walk and unweighted full-plan rank while tuning every split.
pub fn run_all_splits_percentile_tuned(adjacency:&[Vec<usize>],vertex_weights:&[i64],edge_weights:&HashMap<(usize,usize),f64>,num_districts:usize,balance_tolerance:f64,niter:u32,base_seed:u64,n_seeds:usize,p:f64,objective:&str,trials:u32) -> Result<(HashMap<usize,usize>,Option<serde_json::Value>),String> {
    use sha2::Digest;
    if !["cut","volume"].contains(&objective)||!(1..=100).contains(&trials){return Err("Invalid percentile METIS controls.".into());}

    if n_seeds == 0 || !p.is_finite() || !(0.0..=1.0).contains(&p) {
        return Err(
            "Percentile search requires a positive seed budget and a percentile in [0,1].".into(),
        );
    }

    if num_districts == 1 {
        return Ok(((0..adjacency.len()).map(|i| (i, 1)).collect(),None));
    }

    // Derive n_seeds seeds from SHA-256 walk.
    let seeds: Vec<u64> = (0..n_seeds)
        .map(|i| {
            let mut h = sha2::Sha256::new();
            h.update(b"PERCENTILE_SWEEP_V1_");
            // Fixed-width seed walk must match native64 and wasm32.
            h.update((i as u64).to_le_bytes());
            h.update(b"_");
            h.update(base_seed.to_le_bytes());
            let d = h.finalize();
            u64::from_le_bytes(d[..8].try_into().unwrap())
        })
        .collect();

    // Run all seeds sequentially. par_iter() would call the C METIS library from
    // multiple threads simultaneously; METIS shares global/TLS RNG state and is
    // not thread-safe, producing non-deterministic results under concurrent calls.
    // The pure-Rust metis-core engine is thread-safe, but we use sequential
    // iteration unconditionally so both engine paths produce identical output.
    let results: Vec<(usize, usize, HashMap<usize, usize>)> = seeds
        .iter()
        .enumerate()
        .map(|(idx, &seed)| {
            let asgn = run_all_splits_tuned(
                adjacency,
                vertex_weights,
                edge_weights,
                num_districts,
                balance_tolerance,
                niter,
                Some(seed),
                None, objective, trials,
            )
            .map_err(|error| format!("Percentile seed {idx} failed: {error}"))?;
            let ec = count_edge_cuts(&asgn, adjacency);
            Ok((idx, ec, asgn))
        })
        .collect::<Result<Vec<_>, String>>()?;

    let cuts_by_seed:Vec<usize>=results.iter().map(|(_,cut,_)|*cut).collect();

    // Sort by (edge_cut ASC, seed_index ASC) — secondary key breaks ties deterministically.
    let mut sorted = results;
    sorted.sort_by(|(i1, ec1, _), (i2, ec2, _)| ec1.cmp(ec2).then(i1.cmp(i2)));

    // Pick plan at rank floor(p * n_seeds), clamped to [0, n_seeds-1].
    let rank = ((p * n_seeds as f64).floor() as usize).min(sorted.len() - 1);
    let (selected_index,selected_cut,plan)=sorted.into_iter().nth(rank).unwrap();
    Ok((plan,Some(serde_json::json!({"method":"percentile-metis","refinement_objective":objective,"internal_trials_per_candidate":trials,"refinement_iterations":niter,"seed_count":n_seeds,"rank":rank,"selected_seed_index":selected_index,"selected_edge_cut":selected_cut,"cuts_by_seed":cuts_by_seed,"seed_walk":"PERCENTILE_SWEEP_V1-u64-le-sha256","selection":"unweighted-edge-cut-then-seed-index","scope":"full-plans-from-prescribed-floor-ceil-tree","internal_trial_selection":"population-excess-then-edge-cut"}))))
}

/// Count total edge cuts in an assignment.
pub(crate) fn count_edge_cuts(assignment: &HashMap<usize, usize>, adj: &[Vec<usize>]) -> usize {
    rgraph_core::undirected_edge_cut_by(adj, |node| assignment.get(&node).copied().unwrap_or(0))
        .expect("validated bisection-runner adjacency")
}

pub(crate) fn weighted_edge_cut(
    edge_weights: &HashMap<(usize, usize), f64>,
    left: &HashSet<usize>,
) -> f64 {
    // HashMap iteration order is randomized between processes. Floating-point
    // addition is order-dependent, and these totals select among close METIS
    // candidates, so always accumulate crossing weights in canonical edge order.
    let mut crossing: Vec<(&(usize, usize), &f64)> = edge_weights
        .iter()
        .filter(|((u, v), _)| left.contains(u) != left.contains(v))
        .collect();
    crossing.sort_unstable_by_key(|(edge, _)| **edge);
    crossing.into_iter().map(|(_, weight)| *weight).sum()
}
