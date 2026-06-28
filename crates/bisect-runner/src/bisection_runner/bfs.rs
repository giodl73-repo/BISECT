use super::*;

// ── BFS Region-Growing (T.12) ─────────────────────────────────────────────────

/// Derive the BFS-algorithm seed from base_seed.
///
/// SHA-256("BFS_SEED_" || base_seed:u64le) → first 8 bytes as u64le.
/// The prefix "BFS_SEED_" embeds algorithm identity; any change requires a
/// prefix change so that existing audit trails remain valid.
///
/// Test assertion: `bfs_growth_seed(0)` must equal `0x6e340a3e9e4f3ca8`.
pub(crate) fn bfs_growth_seed(base_seed: u64) -> u64 {
    use sha2::{Digest, Sha256};
    let mut h = Sha256::new();
    h.update(b"BFS_SEED_");
    h.update(base_seed.to_le_bytes());
    let d = h.finalize();
    u64::from_le_bytes(d[..8].try_into().unwrap())
}

/// Derive a deterministic per-node BFS seed from the base seed and the node path.
///
/// SHA-256("BFS_NODE_" || path.as_bytes() || "_" || base_seed.to_le_bytes()) → first 8 bytes.
pub fn derive_bfs_seed(base_seed: u64, path: &str) -> u64 {
    use sha2::{Digest, Sha256};
    let mut h = Sha256::new();
    h.update(b"BFS_NODE_");
    h.update(path.as_bytes());
    h.update(b"_");
    h.update(base_seed.to_le_bytes());
    let d = h.finalize();
    u64::from_le_bytes(d[..8].try_into().unwrap())
}

/// Split a subgraph into two balanced parts using BFS Region-Growing.
///
/// Algorithm (T.12 spec §1):
///   1. Build local index (sorted, deterministic)
///   2. Select 2 seeds by k-farthest BFS spread:
///      - seed[0] = population-weighted random sample using bfs_growth_seed(base_seed)
///      - seed[1] = tract with maximum BFS distance from seed[0]
///   3. BFS growth via min-heap:
///      - Priority = |ideal_pop - current_pop[district]| (lower = district needs more tracts)
///      - Assign each unassigned tract to the adjacent district with the greatest deficit
///   4. Post-hoc rebalance (200-iter boundary-swap, same logic as split_subgraph)
///
/// Returns (global left set, global right set) where left = district 0, right = district 1.
pub fn split_subgraph_bfs(
    adjacency: &[Vec<usize>],
    vertex_weights: &[i64],
    tract_indices: &HashSet<usize>,
    balance_tolerance: f64,
    base_seed: u64,
) -> Result<(HashSet<usize>, HashSet<usize>), String> {
    use rand::distributions::WeightedIndex;
    use rand::prelude::*;
    use rand::rngs::SmallRng;
    use rand::SeedableRng;
    use std::cmp::Reverse;
    use std::collections::BinaryHeap;

    // Degenerate: 0 or 1 tracts
    if tract_indices.len() <= 1 {
        return Ok((tract_indices.clone(), HashSet::new()));
    }

    // Build local index mapping (sorted for determinism)
    let mut sorted: Vec<usize> = tract_indices.iter().copied().collect();
    sorted.sort_unstable();
    let global_to_local: HashMap<usize, usize> =
        sorted.iter().enumerate().map(|(i, &g)| (g, i)).collect();
    let m = sorted.len();

    // Build subgraph adjacency (local indices)
    let local_adj: Vec<Vec<usize>> = sorted
        .iter()
        .map(|&g| {
            adjacency[g]
                .iter()
                .filter(|&&nb| tract_indices.contains(&nb))
                .map(|&nb| global_to_local[&nb])
                .collect()
        })
        .collect();

    // Local vertex weights (minimum 1 to avoid zero-weight tracts)
    let local_pop: Vec<i64> = sorted.iter().map(|&g| vertex_weights[g].max(1)).collect();
    let total_pop: i64 = local_pop.iter().sum();
    let ideal_pop = total_pop / 2;

    // ── Step 1: Seed selection ────────────────────────────────────────────────
    // seed[0]: population-weighted random sample
    let seed_rng_val = bfs_growth_seed(base_seed);
    let mut rng = SmallRng::seed_from_u64(seed_rng_val);

    let weights: Vec<u64> = local_pop.iter().map(|&p| p.max(1) as u64).collect();
    let dist =
        WeightedIndex::new(&weights).map_err(|e| format!("bfs-growth WeightedIndex: {e}"))?;
    let seed0 = dist.sample(&mut rng);

    // seed[1]: tract with maximum BFS distance from seed[0]
    let dist_from_s0 = bfs_distances_from(seed0, &local_adj);
    let seed1 = (0..m)
        .filter(|&v| dist_from_s0[v] != usize::MAX)
        .max_by_key(|&v| dist_from_s0[v])
        .unwrap_or_else(|| if seed0 == 0 { 1 } else { 0 });
    // Guard: seeds must be distinct
    let seed1 = if seed1 == seed0 {
        (seed0 + 1) % m
    } else {
        seed1
    };

    // ── Step 2: BFS growth ────────────────────────────────────────────────────
    // assignment: None = unassigned, Some(0) = left, Some(1) = right
    let mut assignment: Vec<Option<usize>> = vec![None; m];
    assignment[seed0] = Some(0);
    assignment[seed1] = Some(1);

    // Track current population per district
    let mut dist_pop: [i64; 2] = [local_pop[seed0], local_pop[seed1]];

    // Priority queue: (Reverse(current_pop), tract_local, district)
    // The district with the LOWEST current population needs the next tract most.
    // Reverse wraps i64 to flip max-heap to min-heap:
    //   smaller current_pop → smaller Reverse(pop) in natural order →
    //   larger Reverse(pop) in max-heap order → popped first.
    // This ensures balanced growth: each next tract goes to the emptier district.
    let mut heap: BinaryHeap<(Reverse<i64>, usize, usize)> = BinaryHeap::new();

    // Seed the heap with neighbors of seed0 and seed1
    for (seed_local, side) in [(seed0, 0usize), (seed1, 1usize)] {
        for &nb in &local_adj[seed_local] {
            if assignment[nb].is_none() {
                heap.push((Reverse(dist_pop[side]), nb, side));
            }
        }
    }

    while let Some((_, tract, side)) = heap.pop() {
        if assignment[tract].is_some() {
            continue; // already assigned by another path
        }
        assignment[tract] = Some(side);
        dist_pop[side] += local_pop[tract];

        // Add unassigned neighbors to the heap with the updated current_pop
        for &nb in &local_adj[tract] {
            if assignment[nb].is_none() {
                heap.push((Reverse(dist_pop[side]), nb, side));
            }
        }
    }

    // Ensure all tracts are assigned (disconnected graph safety net)
    for v in 0..m {
        if assignment[v].is_none() {
            // Assign any unassigned tract to the side with the smaller population
            let side = if dist_pop[0] <= dist_pop[1] { 0 } else { 1 };
            assignment[v] = Some(side);
            dist_pop[side] += local_pop[v];
        }
    }

    // ── Step 3: Post-hoc rebalance ────────────────────────────────────────────
    // Same boundary-swap logic as split_subgraph and split_subgraph_cvd.
    let tolerance_pop = (balance_tolerance * total_pop as f64) as i64 + 1;

    for _ in 0..200 {
        let left_pop: i64 = (0..m)
            .filter(|&v| assignment[v] == Some(0))
            .map(|v| local_pop[v])
            .sum();
        let excess = left_pop - ideal_pop;
        if excess.abs() <= tolerance_pop {
            break;
        }

        let (heavy_side, light_side) = if excess > 0 { (0usize, 1usize) } else { (1, 0) };

        let mut best: Option<(usize, i64)> = None;
        for v in 0..m {
            if assignment[v] != Some(heavy_side) {
                continue;
            }
            let has_light_nb = local_adj[v]
                .iter()
                .any(|&nb| assignment[nb] == Some(light_side));
            if !has_light_nb {
                continue;
            }
            let pop = local_pop[v];
            let score = (pop - excess.abs()).abs();
            if best.map_or(true, |(_, s)| score < s) {
                best = Some((v, score));
            }
        }
        match best {
            Some((v, _)) => {
                assignment[v] = Some(light_side);
            }
            None => break,
        }
    }

    // ── Convert local assignment to global HashSets ───────────────────────────
    let mut left = HashSet::new();
    let mut right = HashSet::new();
    for (local, side_opt) in assignment.iter().enumerate() {
        let global = sorted[local];
        match side_opt {
            Some(0) => {
                left.insert(global);
            }
            _ => {
                right.insert(global);
            }
        }
    }

    Ok((left, right))
}

/// Run the full bisection tree using BFS Region-Growing at each node (T.12).
///
/// Structurally identical to run_all_splits_sa/run_all_splits_cvd but calls
/// split_subgraph_bfs at each bisection node instead of SA/CVD/METIS.
pub fn run_all_splits_bfs(
    adjacency: &[Vec<usize>],
    vertex_weights: &[i64],
    num_districts: usize,
    balance_tolerance: f64,
    intermediate_dir: Option<&Path>,
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
                let bfs_seed = derive_bfs_seed(base_seed, &node.path);
                let (left, right) =
                    split_subgraph_bfs(adjacency, vertex_weights, &tracts, node_ufactor, bfs_seed)
                        .map_err(|e| {
                            format!("depth {} node '{}' (bfs-growth): {e}", depth, node.path)
                        })?;
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
            "bfs-growth bisection incomplete: {}/{n} tracts assigned",
            assignments.len()
        ));
    }
    Ok(assignments)
}
