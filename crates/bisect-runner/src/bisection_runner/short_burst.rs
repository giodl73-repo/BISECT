use super::*;

// ── ShortBurst ────────────────────────────────────────────────────────────────

/// Run a Short-Burst ReCom search on the full k-way plan.
///
/// Algorithm:
///   1. Build initial k-way plan via `run_all_splits`.
///   2. For each burst i in 0..n_bursts:
///      a. Derive chain seed: SHA-256("SHORT_BURST_CHAIN_" || i.to_le_bytes() || "_" || base_seed.to_le_bytes()) → u64.
///      b. Construct a fresh RecomChain from `current_assignment` with `k` districts.
///      c. Step the chain `burst_length` times.
///      d. Record the ENDPOINT (not the minimum within the burst).
///      e. Set `current_assignment` = endpoint (chain restarts from here).
///   3. Sort endpoints by (EC ASC, burst_idx ASC).
///   4. Return plan at rank floor(p * n_bursts), clamped.
///
/// Returns `(assignment, burst_seeds, selected_burst_idx)`.
pub fn run_short_burst(
    adjacency: &[Vec<usize>],
    vertex_weights: &[i64],
    edge_weights: &HashMap<(usize, usize), f64>,
    num_districts: usize,
    balance_tolerance: f64,
    niter: u32,
    base_seed: u64,
    burst_length: usize,
    n_bursts: usize,
    p: f64,
) -> Result<(HashMap<usize, usize>, Vec<u64>, usize), String> {
    use bisect_ensemble::recom::RecomChain;
    use sha2::Digest;

    if num_districts <= 1 {
        let trivial: HashMap<usize, usize> = (0..adjacency.len()).map(|i| (i, 1)).collect();
        return Ok((trivial, vec![], 0));
    }

    // Build initial plan.
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

    if n_bursts == 0 {
        return Ok((initial, vec![], 0));
    }

    // Build Vec<Vec<u32>> adjacency for RecomChain.
    let adj_u32: Vec<Vec<u32>> = adjacency
        .iter()
        .map(|nbrs| nbrs.iter().map(|&n| n as u32).collect())
        .collect();
    let pop: Vec<i64> = vertex_weights.to_vec();
    let n = adjacency.len();

    // Convert initial HashMap<usize,usize> assignment to Vec<u32> (1-based).
    let assignment_to_vec = |asgn: &HashMap<usize, usize>| -> Vec<u32> {
        (0..n)
            .map(|i| asgn.get(&i).copied().unwrap_or(1) as u32)
            .collect()
    };

    // Count EC from a Vec<u32> assignment using the bisection_runner adjacency.
    let count_ec_vec = |asgn: &[u32]| -> usize {
        let mut cut = 0usize;
        for (v, nbrs) in adjacency.iter().enumerate() {
            for &nb in nbrs {
                if nb > v && asgn[v] != asgn[nb] {
                    cut += 1;
                }
            }
        }
        cut
    };

    let mut current_vec = assignment_to_vec(&initial);

    // Derive per-burst seed: SHA-256("SHORT_BURST_CHAIN_" || burst_idx || "_" || base_seed) → u64.
    let derive_seed = |burst_idx: usize| -> u64 {
        let mut h = sha2::Sha256::new();
        h.update(b"SHORT_BURST_CHAIN_");
        h.update((burst_idx as u64).to_le_bytes());
        h.update(b"_");
        h.update(base_seed.to_le_bytes());
        let d = h.finalize();
        u64::from_le_bytes(d[..8].try_into().unwrap())
    };

    let mut burst_seeds: Vec<u64> = Vec::with_capacity(n_bursts);
    // Collect (ec, burst_idx, assignment_vec).
    let mut endpoints: Vec<(usize, usize, Vec<u32>)> = Vec::with_capacity(n_bursts);

    for burst_idx in 0..n_bursts {
        let chain_seed = derive_seed(burst_idx);
        burst_seeds.push(chain_seed);

        let mut rng = SmallRng::seed_from_u64(chain_seed);
        let mut chain = RecomChain::new(
            adj_u32.clone(),
            pop.clone(),
            current_vec.clone(),
            num_districts as u32,
            balance_tolerance,
        );

        for _ in 0..burst_length {
            chain.step(&mut rng);
        }

        // Record ENDPOINT (not minimum within burst).
        let endpoint = chain.assignment.clone();
        let ec = count_ec_vec(&endpoint);
        endpoints.push((ec, burst_idx, endpoint.clone()));

        // Next burst starts from this endpoint.
        current_vec = endpoint;
    }

    // Sort by (EC ASC, burst_idx ASC) for determinism on EC ties.
    endpoints.sort_by(|(ec1, idx1, _), (ec2, idx2, _)| ec1.cmp(ec2).then(idx1.cmp(idx2)));

    // Pick plan at rank floor(p * n_bursts), clamped to [0, n_bursts-1].
    let rank = ((p * n_bursts as f64).floor() as usize).min(endpoints.len() - 1);
    let (_, selected_burst_idx, chosen_vec) = endpoints.into_iter().nth(rank).unwrap();

    // Convert Vec<u32> back to HashMap<usize,usize>.
    let assignment: HashMap<usize, usize> = chosen_vec
        .iter()
        .enumerate()
        .map(|(i, &d)| (i, d as usize))
        .collect();

    Ok((assignment, burst_seeds, selected_burst_idx))
}

// ── ShortBurstForest ──────────────────────────────────────────────────────────

/// Run a Short-Burst Forest ReCom search on the full k-way plan.
///
/// Algorithm:
///   1. Build initial k-way plan via `run_all_splits`.
///   2. For each burst i in 0..n_bursts:
///      a. Derive burst seed: SHA-256("SBF_CHAIN_" || i.to_le_bytes() || "_" || base_seed.to_le_bytes()) → u64.
///      b. Construct a fresh ForestRecomChain from `current_assignment`.
///      c. Step the chain `burst_length` times, deriving two RNG streams per step:
///         - forward_seed: SHA-256("SBF_FWD_" || step:u32le || "_" || burst_seed:u64le) → u64
///         - reverse_seed: SHA-256("SBF_REV_" || step:u32le || "_" || burst_seed:u64le) → u64
///      d. Record the ENDPOINT (not the minimum within the burst).
///      e. Set `current_assignment` = endpoint (chain restarts from here).
///   3. Sort endpoints by (EC ASC, burst_idx ASC).
///   4. Return plan at rank floor(p * n_bursts), clamped.
///
/// Returns `HashMap<usize, usize>` (tract → district, 1-based).
pub fn run_short_burst_forest(
    adjacency: &[Vec<usize>],
    vertex_weights: &[i64],
    edge_weights: &HashMap<(usize, usize), f64>,
    num_districts: usize,
    balance_tolerance: f64,
    niter: u32,
    base_seed: u64,
    burst_length: usize,
    n_bursts: usize,
    p: f64,
) -> Result<HashMap<usize, usize>, String> {
    use bisect_ensemble::forest_recom::ForestRecomChain;
    use rand::rngs::SmallRng;
    use rand::SeedableRng;
    use sha2::Digest;

    if num_districts <= 1 {
        let trivial: HashMap<usize, usize> = (0..adjacency.len()).map(|i| (i, 1)).collect();
        return Ok(trivial);
    }

    // Build initial plan.
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

    if n_bursts == 0 {
        return Ok(initial);
    }

    // Build Vec<Vec<u32>> adjacency for ForestRecomChain.
    let adj_u32: Vec<Vec<u32>> = adjacency
        .iter()
        .map(|nbrs| nbrs.iter().map(|&n| n as u32).collect())
        .collect();
    let pop: Vec<i64> = vertex_weights.to_vec();
    let n = adjacency.len();

    // Convert initial HashMap<usize,usize> assignment to Vec<u32> (1-based).
    let assignment_to_vec = |asgn: &HashMap<usize, usize>| -> Vec<u32> {
        (0..n)
            .map(|i| asgn.get(&i).copied().unwrap_or(1) as u32)
            .collect()
    };

    // Count EC from a Vec<u32> assignment using the bisection_runner adjacency.
    let count_ec_vec = |asgn: &[u32]| -> usize {
        let mut cut = 0usize;
        for (v, nbrs) in adjacency.iter().enumerate() {
            for &nb in nbrs {
                if nb > v && asgn[v] != asgn[nb] {
                    cut += 1;
                }
            }
        }
        cut
    };

    // Derive burst-level seed: SHA-256("SBF_CHAIN_" || burst_idx:u64le || "_" || base_seed:u64le).
    let derive_burst_seed = |burst_idx: usize| -> u64 {
        let mut h = sha2::Sha256::new();
        h.update(b"SBF_CHAIN_");
        h.update((burst_idx as u64).to_le_bytes());
        h.update(b"_");
        h.update(base_seed.to_le_bytes());
        let d = h.finalize();
        u64::from_le_bytes(d[..8].try_into().unwrap())
    };

    // Derive per-step forward seed: SHA-256("SBF_FWD_" || step:u32le || "_" || burst_seed:u64le).
    let derive_forward_seed = |step: u32, burst_seed: u64| -> u64 {
        let mut h = sha2::Sha256::new();
        h.update(b"SBF_FWD_");
        h.update(step.to_le_bytes());
        h.update(b"_");
        h.update(burst_seed.to_le_bytes());
        let d = h.finalize();
        u64::from_le_bytes(d[..8].try_into().unwrap())
    };

    // Derive per-step reverse seed: SHA-256("SBF_REV_" || step:u32le || "_" || burst_seed:u64le).
    let derive_reverse_seed = |step: u32, burst_seed: u64| -> u64 {
        let mut h = sha2::Sha256::new();
        h.update(b"SBF_REV_");
        h.update(step.to_le_bytes());
        h.update(b"_");
        h.update(burst_seed.to_le_bytes());
        let d = h.finalize();
        u64::from_le_bytes(d[..8].try_into().unwrap())
    };

    let mut current_vec = assignment_to_vec(&initial);

    // Collect (ec, burst_idx, assignment_vec).
    let mut endpoints: Vec<(usize, usize, Vec<u32>)> = Vec::with_capacity(n_bursts);

    for burst_idx in 0..n_bursts {
        let burst_seed = derive_burst_seed(burst_idx);

        let mut chain = ForestRecomChain::new(
            adj_u32.clone(),
            pop.clone(),
            current_vec.clone(),
            num_districts as u32,
            balance_tolerance,
        );

        for step in 0..burst_length {
            let fwd_seed = derive_forward_seed(step as u32, burst_seed);
            let rev_seed = derive_reverse_seed(step as u32, burst_seed);
            let mut rng_fwd = SmallRng::seed_from_u64(fwd_seed);
            let mut rng_rev = SmallRng::seed_from_u64(rev_seed);
            chain.step(&mut rng_fwd, &mut rng_rev);
        }

        // Record ENDPOINT (not minimum within burst).
        let endpoint = chain.assignment.clone();
        let ec = count_ec_vec(&endpoint);
        endpoints.push((ec, burst_idx, endpoint.clone()));

        // Next burst starts from this endpoint.
        current_vec = endpoint;
    }

    // Sort by (EC ASC, burst_idx ASC) for determinism on EC ties.
    endpoints.sort_by(|(ec1, idx1, _), (ec2, idx2, _)| ec1.cmp(ec2).then(idx1.cmp(idx2)));

    // Pick plan at rank floor(p * n_bursts), clamped to [0, n_bursts-1].
    let rank = ((p * n_bursts as f64).floor() as usize).min(endpoints.len() - 1);
    let (_, _, chosen_vec) = endpoints.into_iter().nth(rank).unwrap();

    // Convert Vec<u32> back to HashMap<usize,usize>.
    let assignment: HashMap<usize, usize> = chosen_vec
        .iter()
        .enumerate()
        .map(|(i, &d)| (i, d as usize))
        .collect();

    Ok(assignment)
}

// ── ShortBurstMergeSplit ──────────────────────────────────────────────────────

/// Run a Short-Burst Merge-Split search on the full k-way plan.
///
/// Algorithm:
///   1. Build initial k-way plan via `run_all_splits`.
///   2. For each burst i in 0..n_bursts:
///      a. Derive burst seed: SHA-256("SBMS_CHAIN_" || i.to_le_bytes() || "_" || base_seed.to_le_bytes()) → u64.
///      b. Construct a fresh MergeSplitChain from `current_assignment`.
///      c. Step the chain `burst_length` times, deriving two RNG streams per step:
///         - forward_seed: SHA-256("SBF_FWD_" || step:u32le || "_" || burst_seed:u64le) → u64
///         - reverse_seed: SHA-256("SBF_REV_" || step:u32le || "_" || burst_seed:u64le) → u64
///      d. Record the ENDPOINT (not the minimum within the burst).
///      e. Set `current_assignment` = endpoint (chain restarts from here).
///   3. Sort endpoints by (EC ASC, burst_idx ASC).
///   4. Return plan at rank floor(p * n_bursts), clamped.
///
/// Returns `HashMap<usize, usize>` (tract → district, 1-based).
pub fn run_short_burst_merge_split(
    adjacency: &[Vec<usize>],
    vertex_weights: &[i64],
    edge_weights: &HashMap<(usize, usize), f64>,
    num_districts: usize,
    balance_tolerance: f64,
    niter: u32,
    base_seed: u64,
    burst_length: usize,
    n_bursts: usize,
    p: f64,
) -> Result<HashMap<usize, usize>, String> {
    use bisect_ensemble::merge_split::MergeSplitChain;
    use rand::rngs::SmallRng;
    use rand::SeedableRng;
    use sha2::Digest;

    if num_districts <= 1 {
        let trivial: HashMap<usize, usize> = (0..adjacency.len()).map(|i| (i, 1)).collect();
        return Ok(trivial);
    }

    // Build initial plan.
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

    if n_bursts == 0 {
        return Ok(initial);
    }

    // Build Vec<Vec<u32>> adjacency for MergeSplitChain.
    let adj_u32: Vec<Vec<u32>> = adjacency
        .iter()
        .map(|nbrs| nbrs.iter().map(|&n| n as u32).collect())
        .collect();
    let pop: Vec<i64> = vertex_weights.to_vec();
    let n = adjacency.len();

    // Convert initial HashMap<usize,usize> assignment to Vec<u32> (1-based).
    let assignment_to_vec = |asgn: &HashMap<usize, usize>| -> Vec<u32> {
        (0..n)
            .map(|i| asgn.get(&i).copied().unwrap_or(1) as u32)
            .collect()
    };

    // Count EC from a Vec<u32> assignment using the bisection_runner adjacency.
    let count_ec_vec = |asgn: &[u32]| -> usize {
        let mut cut = 0usize;
        for (v, nbrs) in adjacency.iter().enumerate() {
            for &nb in nbrs {
                if nb > v && asgn[v] != asgn[nb] {
                    cut += 1;
                }
            }
        }
        cut
    };

    // Derive burst-level seed: SHA-256("SBMS_CHAIN_" || burst_idx:u64le || "_" || base_seed:u64le).
    let derive_burst_seed = |burst_idx: usize| -> u64 {
        let mut h = sha2::Sha256::new();
        h.update(b"SBMS_CHAIN_");
        h.update((burst_idx as u64).to_le_bytes());
        h.update(b"_");
        h.update(base_seed.to_le_bytes());
        let d = h.finalize();
        u64::from_le_bytes(d[..8].try_into().unwrap())
    };

    // Derive per-step forward seed: SHA-256("SBF_FWD_" || step:u32le || "_" || burst_seed:u64le).
    let derive_forward_seed = |step: u32, burst_seed: u64| -> u64 {
        let mut h = sha2::Sha256::new();
        h.update(b"SBF_FWD_");
        h.update(step.to_le_bytes());
        h.update(b"_");
        h.update(burst_seed.to_le_bytes());
        let d = h.finalize();
        u64::from_le_bytes(d[..8].try_into().unwrap())
    };

    // Derive per-step reverse seed: SHA-256("SBF_REV_" || step:u32le || "_" || burst_seed:u64le).
    let derive_reverse_seed = |step: u32, burst_seed: u64| -> u64 {
        let mut h = sha2::Sha256::new();
        h.update(b"SBF_REV_");
        h.update(step.to_le_bytes());
        h.update(b"_");
        h.update(burst_seed.to_le_bytes());
        let d = h.finalize();
        u64::from_le_bytes(d[..8].try_into().unwrap())
    };

    let mut current_vec = assignment_to_vec(&initial);

    // Collect (ec, burst_idx, assignment_vec).
    let mut endpoints: Vec<(usize, usize, Vec<u32>)> = Vec::with_capacity(n_bursts);

    for burst_idx in 0..n_bursts {
        let burst_seed = derive_burst_seed(burst_idx);

        let mut chain = MergeSplitChain::new(
            adj_u32.clone(),
            pop.clone(),
            current_vec.clone(),
            num_districts as u32,
            balance_tolerance,
        );

        for step in 0..burst_length {
            let fwd_seed = derive_forward_seed(step as u32, burst_seed);
            let rev_seed = derive_reverse_seed(step as u32, burst_seed);
            let mut rng_fwd = SmallRng::seed_from_u64(fwd_seed);
            let mut rng_rev = SmallRng::seed_from_u64(rev_seed);
            chain.step(&mut rng_fwd, &mut rng_rev);
        }

        // Record ENDPOINT (not minimum within burst).
        let endpoint = chain.assignment.clone();
        let ec = count_ec_vec(&endpoint);
        endpoints.push((ec, burst_idx, endpoint.clone()));

        // Next burst starts from this endpoint.
        current_vec = endpoint;
    }

    // Sort by (EC ASC, burst_idx ASC) for determinism on EC ties.
    endpoints.sort_by(|(ec1, idx1, _), (ec2, idx2, _)| ec1.cmp(ec2).then(idx1.cmp(idx2)));

    // Pick plan at rank floor(p * n_bursts), clamped to [0, n_bursts-1].
    let rank = ((p * n_bursts as f64).floor() as usize).min(endpoints.len() - 1);
    let (_, _, chosen_vec) = endpoints.into_iter().nth(rank).unwrap();

    // Convert Vec<u32> back to HashMap<usize,usize>.
    let assignment: HashMap<usize, usize> = chosen_vec
        .iter()
        .enumerate()
        .map(|(i, &d)| (i, d as usize))
        .collect();

    Ok(assignment)
}
