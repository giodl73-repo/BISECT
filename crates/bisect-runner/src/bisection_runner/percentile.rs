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
    intermediate_dir: Option<&Path>,
) -> Result<HashMap<usize, usize>, String> {
    use sha2::{Digest, Sha256};

    if num_districts == 1 {
        return Ok((0..adjacency.len()).map(|i| (i, 1)).collect());
    }

    // Derive n_seeds seeds from SHA-256 walk.
    let seeds: Vec<u64> = (0..n_seeds)
        .map(|i| {
            let mut h = sha2::Sha256::new();
            h.update(b"PERCENTILE_SWEEP_V1_");
            h.update(i.to_le_bytes());
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
    let n = adjacency.len();
    let results: Vec<(usize, usize, HashMap<usize, usize>)> = seeds
        .iter()
        .enumerate()
        .map(|(idx, &seed)| {
            let asgn = run_all_splits(
                adjacency,
                vertex_weights,
                edge_weights,
                num_districts,
                balance_tolerance,
                niter,
                Some(seed),
                None,
            )
            .unwrap_or_else(|_| (0..n).map(|i| (i, 1)).collect());
            let ec = count_edge_cuts(&asgn, adjacency);
            (idx, ec, asgn)
        })
        .collect();

    // Sort by (edge_cut ASC, seed_index ASC) — secondary key breaks ties deterministically.
    let mut sorted = results;
    sorted.sort_by(|(i1, ec1, _), (i2, ec2, _)| ec1.cmp(ec2).then(i1.cmp(i2)));

    // Pick plan at rank floor(p * n_seeds), clamped to [0, n_seeds-1].
    let rank = ((p * n_seeds as f64).floor() as usize).min(sorted.len() - 1);
    Ok(sorted.into_iter().nth(rank).map(|(_, _, a)| a).unwrap())
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
    edge_weights
        .iter()
        .filter_map(|(&(u, v), &weight)| (left.contains(&u) != left.contains(&v)).then_some(weight))
        .sum()
}
