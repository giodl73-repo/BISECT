use super::*;

// ── Simulated Annealing ───────────────────────────────────────────────────────

/// Derive a deterministic per-node SA seed from the base seed and the node path.
/// SHA-256("SA_NODE_" || path.as_bytes() || "_" || base_seed.to_le_bytes()) -> first 8 bytes.
pub fn derive_sa_seed(base_seed: u64, path: &str) -> u64 {
    use sha2::{Digest, Sha256};
    let mut h = Sha256::new();
    h.update(b"SA_NODE_");
    h.update(path.as_bytes());
    h.update(b"_");
    h.update(base_seed.to_le_bytes());
    let d = h.finalize();
    u64::from_le_bytes(d[..8].try_into().unwrap())
}

/// Count edge cuts for a local (0=left, 1=right) binary partition over a subgraph.
/// `partition`: per-local-vertex assignment (0 or 1).
/// `sub_adj`: local subgraph adjacency list.
pub(crate) fn count_ec_local(partition: &[u8], sub_adj: &[Vec<usize>]) -> usize {
    let mut cut = 0usize;
    for (v, nbrs) in sub_adj.iter().enumerate() {
        for &nb in nbrs {
            if nb > v && partition[v] != partition[nb] {
                cut += 1;
            }
        }
    }
    cut
}

/// Check BFS connectivity of one side (side_val=0 or 1) in the local partition.
/// Returns true if all vertices with `partition[v] == side_val` are connected
/// via sub_adj restricted to those vertices.
pub(crate) fn is_side_connected(partition: &[u8], sub_adj: &[Vec<usize>], side_val: u8) -> bool {
    let members: Vec<usize> = (0..partition.len())
        .filter(|&v| partition[v] == side_val)
        .collect();
    if members.len() <= 1 {
        return true;
    }
    // BFS from first member
    let mut visited = vec![false; partition.len()];
    let start = members[0];
    visited[start] = true;
    let mut queue = std::collections::VecDeque::new();
    queue.push_back(start);
    while let Some(v) = queue.pop_front() {
        for &nb in &sub_adj[v] {
            if !visited[nb] && partition[nb] == side_val {
                visited[nb] = true;
                queue.push_back(nb);
            }
        }
    }
    members.iter().all(|&v| visited[v])
}

/// Split a subgraph using Simulated Annealing refinement of an initial METIS partition.
///
/// Algorithm:
///   1. Get initial bisection from METIS.
///   2. Run n_steps = steps_per_tract * |subgraph| SA steps with geometric cooling.
///   3. At each step: pick a random boundary tract, flip to the other district if
///      contiguous and balanced. Accept/reject via Boltzmann criterion.
///   4. Track best-ever EC plan and return it.
pub fn split_subgraph_sa(
    adjacency: &[Vec<usize>],
    vertex_weights: &[i64],
    edge_weights: &HashMap<(usize, usize), f64>,
    tract_indices: &HashSet<usize>,
    balance_tolerance: f64,
    steps_per_tract: usize,
    t0_factor: f64,
    t_final: f64,
    sa_seed: u64,
) -> Result<(HashSet<usize>, HashSet<usize>), String> {
    use rand::rngs::SmallRng;
    use rand::Rng;
    use rand::SeedableRng;

    if tract_indices.len() <= 1 {
        return Ok((tract_indices.clone(), HashSet::new()));
    }

    // Build local index mapping (sorted for determinism)
    let mut sorted: Vec<usize> = tract_indices.iter().copied().collect();
    sorted.sort_unstable();
    let global_to_local: HashMap<usize, usize> =
        sorted.iter().enumerate().map(|(i, &g)| (g, i)).collect();
    let n = sorted.len();

    // Build subgraph adjacency (local indices)
    let sub_adj: Vec<Vec<usize>> = sorted
        .iter()
        .map(|&g| {
            adjacency[g]
                .iter()
                .filter(|&&nb| tract_indices.contains(&nb))
                .map(|&nb| global_to_local[&nb])
                .collect()
        })
        .collect();

    // Build local vertex weights
    let local_pop: Vec<i64> = sorted.iter().map(|&g| vertex_weights[g].max(1)).collect();
    let total_pop: i64 = local_pop.iter().sum();
    let half_pop = total_pop / 2;
    let tolerance_pop = (balance_tolerance * total_pop as f64) as i64 + 1;

    // Get initial METIS partition — use a sub_vwgt of i64 for split_subgraph
    let sub_vwgt: Vec<i64> = local_pop.clone();
    let (metis_left, metis_right) = split_subgraph(
        adjacency,
        vertex_weights,
        1,
        edge_weights,
        tract_indices,
        balance_tolerance + 1.0, // let METIS use full state tolerance (we re-check ourselves)
        100,
        None,
        None,
        None,
    )?;

    // Build initial local partition (0=left, 1=right)
    let mut partition: Vec<u8> = sorted
        .iter()
        .map(|g| if metis_left.contains(g) { 0 } else { 1 })
        .collect();

    let n_steps = steps_per_tract * n;
    let initial_ec = count_ec_local(&partition, &sub_adj);

    // T_0 = max(1.0, t0_factor * initial_ec)
    let t0 = (t0_factor * initial_ec as f64).max(1.0);
    // Guard: t_final must be > 0 and <= t0; clamp to a small epsilon if zero
    let t_final_safe = t_final.max(1e-12).min(t0);

    let mut best_ec = initial_ec;
    let mut best_partition = partition.clone();
    // Track current plan's EC for Metropolis comparison (spec: delta_ec = count_ec(proposed) - count_ec(plan))
    let mut current_ec = initial_ec;

    let mut rng = SmallRng::seed_from_u64(sa_seed);

    // Helper: compute population of each side
    let side_pop = |p: &[u8], side: u8| -> i64 {
        p.iter()
            .enumerate()
            .filter(|(_, &s)| s == side)
            .map(|(i, _)| local_pop[i])
            .sum()
    };

    for step in 0..n_steps {
        // Geometric cooling: T at step s = T_0 * (t_final / T_0)^(s / n_steps)
        let t = if n_steps > 1 {
            t0 * (t_final_safe / t0).powf(step as f64 / (n_steps - 1) as f64)
        } else {
            t_final_safe
        };

        // Collect boundary tracts: tracts adjacent to the other district
        let boundary: Vec<usize> = (0..n)
            .filter(|&v| {
                let side = partition[v];
                sub_adj[v].iter().any(|&nb| partition[nb] != side)
            })
            .collect();

        if boundary.is_empty() {
            break;
        }

        // Pick random boundary tract
        let tract = boundary[rng.gen_range(0..boundary.len())];
        let current_side = partition[tract];
        let other_side = 1 - current_side;

        // Population check: would the flip remain balanced?
        let pop_current = side_pop(&partition, current_side);
        let pop_after_flip = pop_current - local_pop[tract];
        if (pop_after_flip - half_pop).abs() > tolerance_pop {
            continue; // skip: would violate balance
        }

        // Contiguity check: the side losing a tract must stay connected
        partition[tract] = other_side;
        let contiguous = is_side_connected(&partition, &sub_adj, current_side);
        if !contiguous {
            partition[tract] = current_side; // revert
            continue;
        }

        // Compute new EC and delta vs current plan (Metropolis criterion per spec)
        let new_ec = count_ec_local(&partition, &sub_adj);
        let delta_ec = new_ec as f64 - current_ec as f64;

        // Metropolis acceptance
        if delta_ec > 0.0 {
            let accept_prob = (-delta_ec / t).exp();
            if rng.gen::<f64>() >= accept_prob {
                partition[tract] = current_side; // reject
                continue;
            }
        }
        // Accepted: update current_ec and partition[tract] already = other_side
        current_ec = new_ec;

        // Track best-ever plan
        if new_ec < best_ec {
            best_ec = new_ec;
            best_partition = partition.clone();
        }
    }

    // Convert best_partition to global HashSets
    let left: HashSet<usize> = sorted
        .iter()
        .enumerate()
        .filter(|(i, _)| best_partition[*i] == 0)
        .map(|(_, &g)| g)
        .collect();
    let right: HashSet<usize> = sorted
        .iter()
        .enumerate()
        .filter(|(i, _)| best_partition[*i] == 1)
        .map(|(_, &g)| g)
        .collect();

    Ok((left, right))
}

/// Run the full bisection tree using Simulated Annealing at each node.
///
/// Identical to `run_all_splits_with_search` but calls `split_subgraph_sa`
/// at each bisection node instead of METIS directly.
pub fn run_all_splits_sa(
    adjacency: &[Vec<usize>],
    vertex_weights: &[i64],
    edge_weights: &HashMap<(usize, usize), f64>,
    num_districts: usize,
    balance_tolerance: f64,
    niter: u32,
    seed: Option<u64>,
    intermediate_dir: Option<&Path>,
    steps_per_tract: usize,
    t0_factor: f64,
    t_final: f64,
    base_seed: u64,
) -> Result<HashMap<usize, usize>, String> {
    let n = adjacency.len();

    if num_districts == 1 {
        if let Some(dir) = intermediate_dir {
            let round_dir = dir.join("depth_00");
            let _ = std::fs::create_dir_all(&round_dir);
            let asgn: HashMap<usize, usize> = (0..n).map(|i| (i, 1)).collect();
            let _ = write_intermediate_round(&round_dir, &asgn);
        }
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
                let sa_seed = derive_sa_seed(base_seed, &node.path);
                let (left, right) = split_subgraph_sa(
                    adjacency,
                    vertex_weights,
                    edge_weights,
                    &tracts,
                    node_ufactor,
                    steps_per_tract,
                    t0_factor,
                    t_final,
                    sa_seed,
                )
                .map_err(|e| format!("depth {} node '{}' (SA): {e}", depth, node.path))?;
                Ok((node.path, left, right))
            })
            .collect::<Result<Vec<_>, String>>()?;

        let mut sorted_results = split_results;
        sorted_results.sort_by_key(|(path, _, _)| path.clone());
        for (path, left, right) in sorted_results {
            node_tracts.insert(format!("{path}0"), left);
            node_tracts.insert(format!("{path}1"), right);
        }

        if let Some(dir) = intermediate_dir {
            let round_dir = dir.join(format!("depth_{:02}", depth + 1));
            let _ = std::fs::create_dir_all(&round_dir);
            let mut nodes: Vec<(&String, &HashSet<usize>)> = node_tracts.iter().collect();
            nodes.sort_by_key(|(path, _)| (path.len(), *path));
            let mut round_asgn: HashMap<usize, usize> = HashMap::with_capacity(n);
            for (region_id, (_, tracts)) in nodes.iter().enumerate() {
                for &tract in tracts.iter() {
                    round_asgn.insert(tract, region_id + 1);
                }
            }
            let _ = write_intermediate_round(&round_dir, &round_asgn);
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
            "SA bisection incomplete: {}/{n} tracts assigned",
            assignments.len()
        ));
    }
    Ok(assignments)
}
