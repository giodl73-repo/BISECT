use super::*;

// ── Merge-Split MCMC ─────────────────────────────────────────────────────────

/// Run `steps` Merge-Split MH steps on a k-way partition, collect all accepted
/// plans (plus the initial plan), sort by (EC ASC, step_idx ASC), and return
/// the plan at rank floor(p × accepted_count).
///
/// Algorithm per step:
///   1. Select an adjacent district pair (d_i, d_j) with pair reselection.
///   2. Merge region = d_i ∪ d_j, sample forward UST (rng_step).
///   3. Count balanced cuts (forward). Sample one uniformly → proposed split.
///   4. Sample reverse UST (rng_reverse). Count balanced cuts (reverse).
///   5. Accept with MH ratio = forward_cuts / reverse_cuts.
///
/// Seed derivation for two independent RNGs per step:
///   step_seed    = SHA-256("MS_STEP_"    || step:u64le || "_" || 0u32le || "_" || base_seed:u64le) → u64
///   reverse_seed = SHA-256("MS_REVERSE_" || step:u64le || "_" || 0u32le || "_" || base_seed:u64le) → u64
///
/// Returns `HashMap<usize, usize>` (tract → district, 1-based).
/// If steps == 0 or no steps are accepted, returns the initial METIS plan.
pub fn run_merge_split(
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
    use bisect_ensemble::merge_split::MergeSplitChain;
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

    // Convert adjacency from Vec<Vec<usize>> to Vec<Vec<u32>> for MergeSplitChain.
    let local_adj: Vec<Vec<u32>> = adjacency
        .iter()
        .map(|nbrs| nbrs.iter().map(|&nb| nb as u32).collect())
        .collect();

    // Convert initial plan (1-based HashMap<usize,usize>) to Vec<u32>.
    let initial_assignment: Vec<u32> = (0..n)
        .map(|i| initial.get(&i).copied().unwrap_or(1) as u32)
        .collect();

    let pop: Vec<i64> = vertex_weights.to_vec();

    let mut chain = MergeSplitChain::new(
        local_adj,
        pop,
        initial_assignment,
        num_districts as u32,
        balance_tolerance,
    );

    // Include initial plan in the accepted list (step_idx 0).
    let initial_ec = count_edge_cuts(&initial, adjacency);
    let mut accepted: Vec<(usize, usize, Vec<u32>)> = Vec::new();
    accepted.push((initial_ec, 0, chain.assignment.clone()));

    for step_idx in 0..steps {
        // Derive two independent seeds per step.
        let step_seed = {
            let mut h = sha2::Sha256::new();
            h.update(b"MS_STEP_");
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
            h.update(b"MS_REVERSE_");
            h.update((step_idx as u64).to_le_bytes());
            h.update(b"_");
            h.update(0u32.to_le_bytes());
            h.update(b"_");
            h.update(base_seed.to_le_bytes());
            let d = h.finalize();
            u64::from_le_bytes(d[..8].try_into().unwrap())
        };

        let mut rng_step = SmallRng::seed_from_u64(step_seed);
        let mut rng_rev = SmallRng::seed_from_u64(reverse_seed);

        let rec = chain.step(&mut rng_step, &mut rng_rev);
        if rec.accepted {
            // Compute edge cut from the current chain assignment.
            let ec = {
                let mut cut = 0usize;
                for (v, nbrs) in adjacency.iter().enumerate() {
                    for &nb in nbrs {
                        if nb > v && chain.assignment[v] != chain.assignment[nb] {
                            cut += 1;
                        }
                    }
                }
                cut
            };
            accepted.push((ec, step_idx + 1, chain.assignment.clone()));
        }
    }

    // Sort by (EC ASC, step_idx ASC) for determinism on EC ties.
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

/// Parallel Tempering: `n_replicas` ForestRecomChains on a geometric tolerance ladder.
///
/// Cold chain (replica 0) accumulates plans; `select_plan(p)` returns the plan at
/// percentile `p` of the cold chain edge-cut distribution.  Hot chains mix faster and
/// exchange plans with the cold chain every `swap_interval` steps.
///
/// Returns `HashMap<usize, usize>` (tract → 1-based district).
pub fn run_parallel_tempering(
    adjacency: &[Vec<usize>],
    vertex_weights: &[i64],
    edge_weights: &HashMap<(usize, usize), f64>,
    num_districts: usize,
    niter: u32,
    base_seed: u64,
    n_replicas: usize,
    swap_interval: usize,
    cold_tolerance: f64,
    hot_tolerance: f64,
    steps: usize,
    p: f64,
) -> Result<HashMap<usize, usize>, String> {
    use bisect_ensemble::parallel_tempering::{
        replica_rngs, replica_seed, swap_seed, ParallelTemperingChain,
    };
    use rand::rngs::SmallRng;
    use rand::SeedableRng;

    let n = adjacency.len();

    if num_districts <= 1 {
        return Ok((0..n).map(|i| (i, 1)).collect());
    }

    // 1. Build initial plan from METIS using the cold tolerance.
    let initial = run_all_splits(
        adjacency,
        vertex_weights,
        edge_weights,
        num_districts,
        cold_tolerance,
        niter,
        Some(base_seed),
        None,
    )?;

    if steps == 0 {
        return Ok(initial);
    }

    // 2. Convert adjacency Vec<Vec<usize>> → Vec<Vec<u32>> for ParallelTemperingChain.
    let local_adj: Vec<Vec<u32>> = adjacency
        .iter()
        .map(|nbrs| nbrs.iter().map(|&nb| nb as u32).collect())
        .collect();

    // 3. Convert initial plan (1-based HashMap<usize,usize>) → Vec<u32>.
    let initial_assignment: Vec<u32> = (0..n)
        .map(|i| initial.get(&i).copied().unwrap_or(1) as u32)
        .collect();

    let pop: Vec<i64> = vertex_weights.to_vec();

    // 4. Construct the chain.
    let mut chain = ParallelTemperingChain::new(
        local_adj,
        pop,
        initial_assignment,
        num_districts as u32,
        cold_tolerance,
        hot_tolerance,
        n_replicas,
        swap_interval,
    );

    // 5. Run `steps` steps.
    for step in 1..=steps {
        // Build per-replica (rng_fwd, rng_rev) pairs.
        let mut rng_replicas: Vec<(SmallRng, SmallRng)> = (0..n_replicas)
            .map(|i| {
                let rseed = replica_seed(base_seed, i as u32, step as u64);
                replica_rngs(rseed)
            })
            .collect();

        // Swap RNG: pair index 0 covers all adjacent swaps for this step.
        let sseed = swap_seed(base_seed, step as u64, 0u32);
        let mut rng_swap = SmallRng::seed_from_u64(sseed);

        chain.step(&mut rng_replicas, &mut rng_swap);
    }

    // 6. Select plan from cold chain at percentile p.
    let chosen = chain.select_plan(p);

    // 7. Convert Vec<u32> → HashMap<usize, usize>.
    let assignment: HashMap<usize, usize> = chosen
        .iter()
        .enumerate()
        .map(|(i, &d)| (i, d as usize))
        .collect();

    // Suppress unused-variable warning for edge_weights (used via run_all_splits only).
    let _ = edge_weights;

    Ok(assignment)
}

/// VRA-aware Forest ReCom: runs `steps` steps of `VraRecomChain`, which wraps
/// `ForestRecomChain` with a hard VRA rejection rule preserving majority-minority
/// districts (those with `minority_vap[t]` fraction >= `vap_threshold`).
///
/// `minority_vap`: per-tract minority VAP fraction (0.0–1.0), aligned to `adjacency`.
/// Returns the plan at percentile `p` of accepted cold-chain EC distribution.
pub fn run_vra_recom(
    adjacency: &[Vec<usize>],
    vertex_weights: &[i64],
    edge_weights: &HashMap<(usize, usize), f64>,
    num_districts: usize,
    niter: u32,
    base_seed: u64,
    steps: usize,
    p: f64,
    vap_threshold: f64,
    minority_vap: &[f64],
) -> Result<HashMap<usize, usize>, String> {
    use bisect_ensemble::vra_recom::VraRecomChain;
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
        0.005,
        niter,
        Some(base_seed),
        None,
    )?;

    if steps == 0 {
        return Ok(initial);
    }

    // Convert adjacency Vec<Vec<usize>> → Vec<Vec<u32>>.
    let local_adj: Vec<Vec<u32>> = adjacency
        .iter()
        .map(|nbrs| nbrs.iter().map(|&nb| nb as u32).collect())
        .collect();

    // Convert initial plan → Vec<u32>.
    let initial_assignment: Vec<u32> = (0..n)
        .map(|i| initial.get(&i).copied().unwrap_or(1) as u32)
        .collect();

    let pop: Vec<i64> = vertex_weights.to_vec();
    let mvap: Vec<f64> = minority_vap.to_vec();

    let mut chain = VraRecomChain::new(
        local_adj,
        pop,
        initial_assignment,
        num_districts as u32,
        0.005,
        mvap,
        vap_threshold,
    );

    // Include initial plan in the accepted list.
    let initial_ec = count_edge_cuts(&initial, adjacency);
    let mut accepted: Vec<(usize, usize, Vec<u32>)> = Vec::new();
    accepted.push((initial_ec, 0, chain.inner.assignment.clone()));

    for step_idx in 0..steps {
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
            let ec = rec.inner.cut_edges;
            accepted.push((ec, step_idx + 1, chain.inner.assignment.clone()));
        }
    }

    // Sort by (EC ASC, step_idx ASC) for determinism.
    accepted.sort_by(|(ec1, idx1, _), (ec2, idx2, _)| ec1.cmp(ec2).then(idx1.cmp(idx2)));

    let accepted_count = accepted.len();
    let rank = ((p * accepted_count as f64).floor() as usize).min(accepted_count - 1);
    let (_, _, chosen) = accepted.into_iter().nth(rank).unwrap();

    // Convert Vec<u32> → HashMap<usize, usize>.
    let assignment: HashMap<usize, usize> = chosen
        .iter()
        .enumerate()
        .map(|(i, &d)| (i, d as usize))
        .collect();

    let _ = edge_weights;

    Ok(assignment)
}
