use super::*;

// ── FlipChain ─────────────────────────────────────────────────────────────────

/// Run a Flip-chain search on the full k-way plan.
///
/// Algorithm:
///   1. Build initial k-way plan via `run_all_splits` (deterministic starting point).
///   2. Derive chain seed: SHA-256("FLIP_CHAIN_" || 0u64 || "_" || base_seed) -> first 8 bytes as u64.
///   3. For each step in 0..flip_steps:
///      a. Collect boundary tracts (tracts with a neighbour in a different district).
///      b. If none, stop early.
///      c. Pick random boundary tract `t` and random adjacent district `d_target != d_src`.
///      d. Flip: plan[t] = d_target.
///      e. Validity: (a) d_src stays contiguous (BFS), (b) both districts within population tolerance.
///      f. If valid, accept and record (EC(plan), plan). If invalid, revert.
///   4. Sort visited by (EC ASC, insertion_idx ASC) for determinism.
///   5. Return plan at rank floor(p * visited_count), plus visited_count and rank.
///
/// The visited list always starts with the initial plan (at least 1 entry).
/// Returns `(assignment, visited_count, selected_rank)`.
pub fn run_flip_chain(
    adjacency: &[Vec<usize>],
    vertex_weights: &[i64],
    edge_weights: &HashMap<(usize, usize), f64>,
    num_districts: usize,
    balance_tolerance: f64,
    flip_steps: usize,
    base_seed: u64,
    p: f64,
) -> Result<(HashMap<usize, usize>, usize, usize), String> {
    use rand::Rng;
    use sha2::Digest;

    let n = adjacency.len();

    if num_districts <= 1 {
        let trivial: HashMap<usize, usize> = (0..n).map(|i| (i, 1)).collect();
        return Ok((trivial, 1, 0));
    }

    // Build initial plan (deterministic starting point).
    let initial = run_all_splits(
        adjacency,
        vertex_weights,
        edge_weights,
        num_districts,
        balance_tolerance,
        100,
        Some(base_seed),
        None,
    )?;

    // Derive chain RNG seed.
    let chain_seed = {
        let mut h = sha2::Sha256::new();
        h.update(b"FLIP_CHAIN_");
        h.update(0u64.to_le_bytes());
        h.update(b"_");
        h.update(base_seed.to_le_bytes());
        let d = h.finalize();
        u64::from_le_bytes(d[..8].try_into().unwrap())
    };

    let mut rng = SmallRng::seed_from_u64(chain_seed);

    // Work with a flat Vec<usize> for O(1) access.
    let mut plan: Vec<usize> = (0..n)
        .map(|i| initial.get(&i).copied().unwrap_or(1))
        .collect();

    // Per-district population (districts are 1-based up to num_districts).
    let total_pop: i64 = vertex_weights.iter().sum();
    let ideal_pop = total_pop as f64 / num_districts as f64;
    let max_dev = ((balance_tolerance * total_pop as f64) as i64 + 1).max(1);

    let mut dist_pop: Vec<i64> = vec![0i64; num_districts + 1];
    for (i, &d) in plan.iter().enumerate() {
        if d < dist_pop.len() {
            dist_pop[d] += vertex_weights[i];
        }
    }

    let initial_ec = count_edge_cuts(&initial, adjacency);

    // visited: (ec, insertion_idx, plan_snapshot)
    let mut visited: Vec<(usize, usize, Vec<usize>)> = Vec::new();
    visited.push((initial_ec, 0, plan.clone()));

    for _step in 0..flip_steps {
        // Collect boundary tracts.
        let boundary: Vec<usize> = (0..n)
            .filter(|&v| {
                let dv = plan[v];
                adjacency[v].iter().any(|&nb| plan[nb] != dv)
            })
            .collect();

        if boundary.is_empty() {
            break;
        }

        let t = boundary[rng.gen_range(0..boundary.len())];
        let d_src = plan[t];

        // Collect adjacent districts (unique, not d_src).
        let mut adj_set = std::collections::HashSet::new();
        for &nb in &adjacency[t] {
            let d = plan[nb];
            if d != d_src {
                adj_set.insert(d);
            }
        }
        if adj_set.is_empty() {
            continue;
        }
        let adj_districts: Vec<usize> = adj_set.into_iter().collect();
        let d_target = adj_districts[rng.gen_range(0..adj_districts.len())];

        // Tentatively flip.
        plan[t] = d_target;
        dist_pop[d_src] -= vertex_weights[t];
        dist_pop[d_target] += vertex_weights[t];

        // Population balance check.
        let dev_src = (dist_pop[d_src] as f64 - ideal_pop).abs() as i64;
        let dev_tgt = (dist_pop[d_target] as f64 - ideal_pop).abs() as i64;
        if dev_src > max_dev || dev_tgt > max_dev {
            plan[t] = d_src;
            dist_pop[d_src] += vertex_weights[t];
            dist_pop[d_target] -= vertex_weights[t];
            continue;
        }

        // Contiguity check: d_src must stay connected after removing t.
        // BFS over tracts still in d_src (plan[v] == d_src, which excludes t now).
        let d_src_first = (0..n).find(|&v| plan[v] == d_src);
        let contiguous = match d_src_first {
            None => true, // d_src is empty — vacuously connected
            Some(start) => {
                let mut vis_bfs = vec![false; n];
                vis_bfs[start] = true;
                let mut queue = std::collections::VecDeque::new();
                queue.push_back(start);
                while let Some(v) = queue.pop_front() {
                    for &nb in &adjacency[v] {
                        if !vis_bfs[nb] && plan[nb] == d_src {
                            vis_bfs[nb] = true;
                            queue.push_back(nb);
                        }
                    }
                }
                (0..n).filter(|&v| plan[v] == d_src).all(|v| vis_bfs[v])
            }
        };

        if !contiguous {
            plan[t] = d_src;
            dist_pop[d_src] += vertex_weights[t];
            dist_pop[d_target] -= vertex_weights[t];
            continue;
        }

        // Accepted: record snapshot.
        let ec = {
            let mut cut = 0usize;
            for (v, nbrs) in adjacency.iter().enumerate() {
                for &nb in nbrs {
                    if nb > v && plan[v] != plan[nb] {
                        cut += 1;
                    }
                }
            }
            cut
        };
        let insertion_idx = visited.len();
        visited.push((ec, insertion_idx, plan.clone()));
    }

    let visited_count = visited.len();

    // Sort by (EC ASC, insertion_idx ASC).
    visited.sort_by(|(ec1, idx1, _), (ec2, idx2, _)| ec1.cmp(ec2).then(idx1.cmp(idx2)));

    let rank = ((p * visited_count as f64).floor() as usize).min(visited_count - 1);
    let (_, _, chosen_vec) = visited.into_iter().nth(rank).unwrap();

    let assignment: HashMap<usize, usize> = chosen_vec
        .iter()
        .enumerate()
        .map(|(i, &d)| (i, d))
        .collect();

    // Suppress unused-variable warning for edge_weights (only used via run_all_splits).
    let _ = edge_weights;

    Ok((assignment, visited_count, rank))
}
