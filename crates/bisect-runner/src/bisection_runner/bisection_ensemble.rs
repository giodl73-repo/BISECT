use super::*;

// ── BisectionEnsemble ─────────────────────────────────────────────────────────

/// Split a subgraph using a local ReCom 2-way ensemble.
///
/// Runs `ensemble_steps` ReCom steps starting from the initial METIS bisection,
/// collects all accepted plans, sorts by edge cut, and returns the plan at
/// rank `floor(p * accepted_count)`.
///
/// This replaces the single METIS call at each bisection tree node with a
/// local feasibility sample. Because it's always k=2, there are no prime-k
/// bipartition failures regardless of the full-state k.
pub fn split_subgraph_bisection_ensemble(
    adjacency: &[Vec<usize>],
    vwgt: &[i64],
    edge_weights: &HashMap<(usize, usize), f64>,
    tract_indices: &HashSet<usize>,
    ufactor: f64,
    niter: u32,
    base_seed: Option<u64>,
    tpwgts: Option<Vec<f32>>,
    ensemble_steps: usize,
    p: f64,
) -> Result<(HashSet<usize>, HashSet<usize>), String> {
    use rand::rngs::SmallRng;
    use rand::SeedableRng;

    // Fall back to standard METIS bisection for very small regions.
    if tract_indices.len() <= 4 {
        return split_subgraph(
            adjacency,
            vwgt,
            1,
            edge_weights,
            tract_indices,
            ufactor,
            niter,
            base_seed,
            tpwgts,
            None,
        );
    }

    // Build local subgraph.
    let mut sorted: Vec<usize> = tract_indices.iter().copied().collect();
    sorted.sort_unstable();
    let global_to_local: HashMap<usize, u32> = sorted
        .iter()
        .enumerate()
        .map(|(i, &g)| (g, i as u32))
        .collect();
    #[allow(unused_imports)]
    use bisect_ensemble::recom::RecomChain;
    let n = sorted.len();

    let local_adj: Vec<Vec<u32>> = sorted
        .iter()
        .map(|&g| {
            adjacency[g]
                .iter()
                .filter_map(|&nb| global_to_local.get(&nb).copied())
                .collect()
        })
        .collect();
    let local_pop: Vec<i64> = sorted.iter().map(|&g| vwgt[g]).collect();

    // Seed initial partition via METIS bisection.
    let (init_left, init_right) = split_subgraph(
        adjacency,
        vwgt,
        1,
        edge_weights,
        tract_indices,
        ufactor,
        niter,
        base_seed,
        tpwgts.clone(),
        None,
    )?;
    let initial_assignment: Vec<u32> = sorted
        .iter()
        .map(|g| if init_left.contains(g) { 1 } else { 2 })
        .collect();

    // Run local ReCom ensemble.
    let seed = base_seed.unwrap_or(0xDEAD_BEEF_CAFE_1234);
    let mut rng = SmallRng::seed_from_u64(seed);
    let total_pop: f64 = local_pop.iter().map(|&p| p as f64).sum();
    let chain_tolerance = ufactor - 1.0;
    let mut chain = if let Some(ref tw) = tpwgts {
        RecomChain::new_with_target_pops(
            local_adj,
            local_pop,
            initial_assignment,
            vec![tw[0] as f64 * total_pop, tw[1] as f64 * total_pop],
            chain_tolerance,
        )
    } else {
        RecomChain::new(local_adj, local_pop, initial_assignment, 2, chain_tolerance)
    };

    let mut accepted_assignments: Vec<(usize, Vec<u32>)> = Vec::new();
    // Include the initial METIS plan.
    {
        let ec: usize = chain
            .assignment
            .iter()
            .enumerate()
            .flat_map(|(v, &d)| chain.adj[v].iter().map(move |&nb| (v, nb as usize, d)))
            .filter(|(v, nb, d)| chain.assignment[*nb] != *d && *nb > *v)
            .count();
        accepted_assignments.push((ec, chain.assignment.clone()));
    }

    for _ in 0..ensemble_steps {
        let rec = chain.step(&mut rng);
        if rec.accepted {
            let ec = rec.cut_edges;
            accepted_assignments.push((ec, chain.assignment.clone()));
        }
    }

    if accepted_assignments.is_empty() {
        // No accepted proposals — return the METIS result.
        return Ok((init_left, init_right));
    }

    // Sort by edge cut (ascending) and pick the p-th percentile.
    accepted_assignments.sort_by_key(|(ec, _)| *ec);
    let rank = ((p * accepted_assignments.len() as f64).floor() as usize)
        .min(accepted_assignments.len() - 1);
    let (_, chosen) = accepted_assignments.into_iter().nth(rank).unwrap();

    let left: HashSet<usize> = sorted
        .iter()
        .enumerate()
        .filter(|(i, _)| chosen[*i] == 1)
        .map(|(_, &g)| g)
        .collect();
    let right: HashSet<usize> = sorted
        .iter()
        .enumerate()
        .filter(|(i, _)| chosen[*i] == 2)
        .map(|(_, &g)| g)
        .collect();
    Ok((left, right))
}
