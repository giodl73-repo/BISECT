use super::*;

// ── ForestRecomChain ──────────────────────────────────────────────────────────

/// Run a Forest ReCom MH chain on the full k-way plan.
///
/// Steps:
/// 1. Build initial plan from `run_all_splits` (single METIS call).
/// 2. Construct a ForestRecomChain from that assignment.
/// 3. Run `steps` chain steps, collecting accepted plans with their EC.
/// 4. Sort by (EC ASC, step_idx ASC) and return plan at rank floor(p × accepted_count).
///
/// If no steps are accepted, return the initial plan.
pub fn run_forest_recom(
    adjacency: &[Vec<usize>],
    vertex_weights: &[i64],
    edge_weights: &HashMap<(usize, usize), f64>,
    num_districts: usize,
    balance_tolerance: f64,
    niter: u32,
    base_seed: u64,
    steps: usize,
    p: f64,
) -> Result<HashMap<usize, usize>, String> {
    use bisect_ensemble::forest_recom::ForestRecomChain;
    use rand::rngs::SmallRng;
    use rand::SeedableRng;
    use sha2::Digest;

    let n = adjacency.len();

    if num_districts <= 1 {
        return Ok((0..n).map(|i| (i, 1)).collect());
    }

    // Build initial plan from METIS.
    let initial = run_all_splits(
        adjacency,
        vertex_weights,
        edge_weights,
        num_districts,
        balance_tolerance,
        niter,
        Some(base_seed),
        None,
    )?;

    if steps == 0 {
        return Ok(initial);
    }

    // Convert adjacency from Vec<Vec<usize>> to Vec<Vec<u32>> for ForestRecomChain.
    let local_adj: Vec<Vec<u32>> = adjacency
        .iter()
        .map(|nbrs| nbrs.iter().map(|&nb| nb as u32).collect())
        .collect();

    // Convert initial plan (1-based HashMap<usize,usize>) to Vec<u32>.
    let initial_assignment: Vec<u32> = (0..n)
        .map(|i| initial.get(&i).copied().unwrap_or(1) as u32)
        .collect();

    let pop: Vec<i64> = vertex_weights.to_vec();

    let mut chain = ForestRecomChain::new(
        local_adj,
        pop,
        initial_assignment,
        num_districts as u32,
        balance_tolerance,
    );

    // Include initial plan in the accepted list.
    let initial_ec = count_edge_cuts(&initial, adjacency);
    let mut accepted: Vec<(usize, usize, Vec<u32>)> = Vec::new();
    accepted.push((initial_ec, 0, chain.assignment.clone()));

    for step_idx in 0..steps {
        // Derive two independent seeds for forward/reverse trees.
        let forward_seed = {
            let mut h = sha2::Sha256::new();
            h.update(b"FR_FORWARD_");
            h.update((step_idx as u64).to_le_bytes());
            h.update(b"_");
            h.update(0u32.to_le_bytes());
            h.update(b"_");
            h.update(base_seed.to_le_bytes());
            let d = h.finalize();
            u64::from_le_bytes(d[..8].try_into().unwrap())
        };
        let reverse_seed = {
            let mut h = sha2::Sha256::new();
            h.update(b"FR_REVERSE_");
            h.update((step_idx as u64).to_le_bytes());
            h.update(b"_");
            h.update(0u32.to_le_bytes());
            h.update(b"_");
            h.update(base_seed.to_le_bytes());
            let d = h.finalize();
            u64::from_le_bytes(d[..8].try_into().unwrap())
        };

        let mut rng_forward = SmallRng::seed_from_u64(forward_seed);
        let mut rng_reverse = SmallRng::seed_from_u64(reverse_seed);

        let rec = chain.step(&mut rng_forward, &mut rng_reverse);
        if rec.accepted {
            let ec = rec.cut_edges;
            accepted.push((ec, step_idx + 1, chain.assignment.clone()));
        }
    }

    // Sort by (EC ASC, step_idx ASC) for determinism.
    accepted.sort_by(|(ec1, idx1, _), (ec2, idx2, _)| ec1.cmp(ec2).then(idx1.cmp(idx2)));

    let accepted_count = accepted.len();
    let rank = ((p * accepted_count as f64).floor() as usize).min(accepted_count - 1);
    let (_, _, chosen) = accepted.into_iter().nth(rank).unwrap();

    // Convert Vec<u32> back to HashMap<usize, usize>.
    let assignment: HashMap<usize, usize> = chosen
        .iter()
        .enumerate()
        .map(|(i, &d)| (i, d as usize))
        .collect();

    // Suppress unused-variable warning for edge_weights (used via run_all_splits only).
    let _ = edge_weights;

    Ok(assignment)
}
