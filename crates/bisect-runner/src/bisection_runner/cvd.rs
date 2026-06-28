use super::*;

// ── Centroidal Voronoi Districts (CVD) — Phase 1 (graph-distance) ─────────────

/// Derive a per-node seed for CVD via SHA-256.
///
/// Prefix "CVD_INIT_" is distinct from SA_NODE_, FLIP_CHAIN_, etc.
/// An auditor can recompute: SHA-256("CVD_INIT_" || path || "_" || base_seed:le64) → u64le.
pub fn derive_cvd_seed(base_seed: u64, path: &str) -> u64 {
    use sha2::{Digest, Sha256};
    let mut h = Sha256::new();
    h.update(b"CVD_INIT_");
    h.update(path.as_bytes());
    h.update(b"_");
    h.update(base_seed.to_le_bytes());
    let d = h.finalize();
    u64::from_le_bytes(d[..8].try_into().unwrap())
}

/// Which distance metric CVD uses for assignment and seed update.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VoronoiMetric {
    /// Phase 1: BFS hop-count on the tract adjacency graph. No extra data needed.
    GraphDistance,
    /// Phase 2: Euclidean distance on Albers-projected (lon, lat) coordinates.
    /// Requires `tract_centroids` to be non-empty in `LoadedGraph`.
    Geographic,
}

impl std::str::FromStr for VoronoiMetric {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "graph-distance" | "graph_distance" | "GraphDistance" => {
                Ok(VoronoiMetric::GraphDistance)
            }
            "geographic" | "Geographic" => Ok(VoronoiMetric::Geographic),
            other => Err(format!(
                "unknown cvd-metric '{}': use 'graph-distance' or 'geographic'",
                other
            )),
        }
    }
}

/// Derive a per-node seed for CVD Phase 2 (geographic) via SHA-256.
///
/// Prefix "CVD_GEO_INIT_" is distinct from Phase 1 "CVD_INIT_".
/// An auditor can recompute: SHA-256("CVD_GEO_INIT_" || path || "_" || base_seed:u64le) => u64le.
pub fn derive_cvd_geo_seed(base_seed: u64, path: &str) -> u64 {
    use sha2::{Digest, Sha256};
    let mut h = Sha256::new();
    h.update(b"CVD_GEO_INIT_");
    h.update(path.as_bytes());
    h.update(b"_");
    h.update(base_seed.to_le_bytes());
    let d = h.finalize();
    u64::from_le_bytes(d[..8].try_into().unwrap())
}

/// Simplified Albers Equal Area Conic projection for EPSG:5070 (continental US).
/// Spherical approximation -- error <= 3m at census-tract resolution.
/// Returns (x_meters, y_meters) relative to the projection origin.
pub fn albers_project(lon: f64, lat: f64) -> (f64, f64) {
    let lon0 = (-96.0f64).to_radians();
    let phi1 = (29.5f64).to_radians();
    let phi2 = (45.5f64).to_radians();
    let lon_r = lon.to_radians();
    let lat_r = lat.to_radians();
    let n = 0.5 * (phi1.sin() + phi2.sin());
    let c = phi1.cos().powi(2) + 2.0 * n * phi1.sin();
    let rho0 = c.max(0.0).sqrt() / n;
    let rho = (c - 2.0 * n * lat_r.sin()).max(0.0).sqrt() / n;
    let theta = n * (lon_r - lon0);
    let x = rho * theta.sin();
    let y = rho0 - rho * theta.cos();
    (x * 6_371_000.0, y * 6_371_000.0)
}

pub(crate) fn euclidean_dist(a: (f64, f64), b: (f64, f64)) -> f64 {
    let dx = a.0 - b.0;
    let dy = a.1 - b.1;
    (dx * dx + dy * dy).sqrt()
}

/// BFS distances from `start` to all other nodes in a local subgraph adjacency list.
/// Returns `Vec<usize>` of length `local_adj.len()`.
/// Unreachable nodes get `usize::MAX`.
pub(crate) fn bfs_distances_from(start: usize, local_adj: &[Vec<usize>]) -> Vec<usize> {
    let n = local_adj.len();
    let mut dist = vec![usize::MAX; n];
    if n == 0 {
        return dist;
    }
    dist[start] = 0;
    let mut queue = std::collections::VecDeque::new();
    queue.push_back(start);
    while let Some(v) = queue.pop_front() {
        let d = dist[v];
        for &nb in &local_adj[v] {
            if dist[nb] == usize::MAX {
                dist[nb] = d + 1;
                queue.push_back(nb);
            }
        }
    }
    dist
}

/// Medoid approximation: find the tract in `district_tracts` that minimises
/// the sum of BFS distances to a random sample of up to 50 other tracts.
/// Returns the local index of the medoid.
pub(crate) fn find_medoid(
    district_tracts: &[usize],
    local_adj: &[Vec<usize>],
    rng: &mut rand::rngs::SmallRng,
) -> usize {
    use rand::seq::SliceRandom;
    if district_tracts.is_empty() {
        return 0;
    }
    if district_tracts.len() == 1 {
        return district_tracts[0];
    }
    // Sample up to 50 tracts as probe set (exact for small districts)
    let sample_size = district_tracts.len().min(50);
    let mut probe: Vec<usize> = district_tracts.to_vec();
    probe.shuffle(rng);
    probe.truncate(sample_size);

    let mut best_node = district_tracts[0];
    let mut best_sum = usize::MAX;

    for &candidate in district_tracts {
        let dist_from_cand = bfs_distances_from(candidate, local_adj);
        let sum: usize = probe
            .iter()
            .map(|&p| dist_from_cand[p].min(usize::MAX / probe.len()))
            .sum();
        if sum < best_sum {
            best_sum = sum;
            best_node = candidate;
        }
    }
    best_node
}

/// Centroidal Voronoi Districts — graph-distance variant (Phase 1, T.10 spec).
///
/// Seeds k=2 district centers by k-farthest spread, assigns tracts to nearest
/// center by BFS hop count, iterates until seeds stabilise (medoid update),
/// then applies the same post-hoc boundary-swap rebalance as split_subgraph().
///
/// Phase 1: no geographic coordinate data required — pure graph topology.
/// Phase 2 (geographic Euclidean) is deferred until tract_centroids land in LoadedGraph.
pub fn split_subgraph_cvd(
    adjacency: &[Vec<usize>],
    vertex_weights: &[i64],
    tract_indices: &HashSet<usize>,
    k: usize, // always 2 for bisection use
    balance_tolerance: f64,
    n_iter: usize, // max CVD iterations (default: 20)
    base_seed: u64,
) -> Result<(HashSet<usize>, HashSet<usize>), String> {
    use rand::rngs::SmallRng;
    use rand::SeedableRng;

    // Degenerate: 0 or 1 tracts
    if tract_indices.len() <= 1 {
        return Ok((tract_indices.clone(), HashSet::new()));
    }
    // Degenerate: fewer tracts than districts
    if tract_indices.len() < k {
        let mut sorted: Vec<usize> = tract_indices.iter().copied().collect();
        sorted.sort_unstable();
        let left: HashSet<usize> = sorted[..1].iter().copied().collect();
        let right: HashSet<usize> = sorted[1..].iter().copied().collect();
        return Ok((left, right));
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

    // Local vertex weights
    let local_pop: Vec<i64> = sorted.iter().map(|&g| vertex_weights[g].max(1)).collect();
    let total_pop: i64 = local_pop.iter().sum();

    let mut rng = SmallRng::seed_from_u64(base_seed);

    // ── Step 1: k-farthest seed initialisation ──────────────────────────────
    // seed[0] = base_seed % m (deterministic, CVD_INIT prefix ensures separation from SA)
    let seed0 = (base_seed as usize) % m;

    // seed[1] = tract with max BFS distance from seed[0]
    let dist_from_s0 = bfs_distances_from(seed0, &local_adj);
    let seed1 = (0..m)
        .filter(|&v| dist_from_s0[v] != usize::MAX)
        .max_by_key(|&v| dist_from_s0[v])
        .unwrap_or((seed0 + 1) % m);

    let mut seeds = [seed0, seed1];
    // Guard: seeds must be distinct
    if seeds[0] == seeds[1] {
        seeds[1] = (seeds[0] + 1) % m;
    }

    // ── CVD iteration ────────────────────────────────────────────────────────
    let mut assignment: Vec<usize> = vec![0; m]; // 0 = left, 1 = right
    for _iter in 0..n_iter.max(1) {
        // ── Voronoi assignment: assign each tract to nearest seed by BFS ──
        // Compute BFS distances from each seed
        let dist_s: Vec<Vec<usize>> = seeds
            .iter()
            .map(|&s| bfs_distances_from(s, &local_adj))
            .collect();

        let prev_assignment = assignment.clone();
        for v in 0..m {
            let d0 = dist_s[0][v];
            let d1 = dist_s[1][v];
            // Assign to nearest seed; tie-break to 0
            assignment[v] = if d1 < d0 { 1 } else { 0 };
        }

        // ── Update seeds to medoid of each district ──────────────────────
        let prev_seeds = seeds;
        for j in 0..k {
            let district_tracts: Vec<usize> = (0..m).filter(|&v| assignment[v] == j).collect();
            if !district_tracts.is_empty() {
                seeds[j] = find_medoid(&district_tracts, &local_adj, &mut rng);
            }
            // else: seed unchanged (shouldn't happen unless one district is empty)
        }

        // ── Check convergence ────────────────────────────────────────────
        // Seeds stable AND assignment stable → converged
        if seeds == prev_seeds || assignment == prev_assignment {
            break;
        }
    }

    // ── Post-hoc rebalance: same boundary-swap as split_subgraph ────────────
    let half_pop = total_pop / 2;
    let tolerance_pop = (balance_tolerance * total_pop as f64) as i64 + 1;

    for _ in 0..200 {
        let left_pop: i64 = (0..m)
            .filter(|&v| assignment[v] == 0)
            .map(|v| local_pop[v])
            .sum();
        let excess = left_pop - half_pop;
        if excess.abs() <= tolerance_pop {
            break;
        }

        let (heavy_side, light_side) = if excess > 0 { (0usize, 1usize) } else { (1, 0) };

        // Find boundary tract on heavy side minimising |pop - |excess||
        let mut best: Option<(usize, i64)> = None;
        for v in 0..m {
            if assignment[v] != heavy_side {
                continue;
            }
            let has_light_nb = local_adj[v].iter().any(|&nb| assignment[nb] == light_side);
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
                assignment[v] = light_side;
            }
            None => break,
        }
    }

    // ── Convert local assignment to global HashSets ──────────────────────────
    let mut left = HashSet::new();
    let mut right = HashSet::new();
    for (local, &side) in assignment.iter().enumerate() {
        let global = sorted[local];
        if side == 0 {
            left.insert(global);
        } else {
            right.insert(global);
        }
    }

    Ok((left, right))
}

/// Centroidal Voronoi Districts -- geographic Euclidean variant (Phase 2, T.10 spec).
///
/// Seeds k=2 district centers by k-farthest Euclidean spread on Albers-projected
/// coordinates, assigns tracts to nearest center by Euclidean distance, iterates
/// updating seeds to the population-weighted mean coordinate (true Lloyd's algorithm),
/// then applies the same post-hoc boundary-swap rebalance as split_subgraph_cvd().
///
/// Requires `tract_centroids` to be non-empty (one entry per global tract index).
pub fn split_subgraph_cvd_geographic(
    adjacency: &[Vec<usize>],
    vertex_weights: &[i64],
    tract_centroids: &[(f64, f64)],
    tract_indices: &HashSet<usize>,
    balance_tolerance: f64,
    n_iter: usize,
    base_seed: u64,
    node_path: &str,
) -> Result<(HashSet<usize>, HashSet<usize>), String> {
    if tract_centroids.is_empty() {
        return Err("[CONFIG] --cvd-metric geographic requires centroid data. \
             Run: bisect fetch --type centroids"
            .to_string());
    }

    if tract_indices.len() <= 1 {
        return Ok((tract_indices.clone(), HashSet::new()));
    }

    let mut sorted: Vec<usize> = tract_indices.iter().copied().collect();
    sorted.sort_unstable();
    let global_to_local: HashMap<usize, usize> =
        sorted.iter().enumerate().map(|(i, &g)| (g, i)).collect();
    let m = sorted.len();

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

    let local_pop: Vec<i64> = sorted.iter().map(|&g| vertex_weights[g].max(1)).collect();
    let total_pop: i64 = local_pop.iter().sum();

    let projected: Vec<(f64, f64)> = sorted
        .iter()
        .map(|&g| {
            let (lon, lat) = tract_centroids[g];
            albers_project(lon, lat)
        })
        .collect();

    // k-farthest seed initialisation (Euclidean, deterministic)
    let cvd_geo_seed = derive_cvd_geo_seed(base_seed, node_path);
    let seed0_local = (cvd_geo_seed as usize) % m;
    let seed1_local = (0..m)
        .max_by(|&a, &b| {
            let da = euclidean_dist(projected[seed0_local], projected[a]);
            let db = euclidean_dist(projected[seed0_local], projected[b]);
            da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
        })
        .unwrap_or((seed0_local + 1) % m);
    let seed1_local = if seed1_local == seed0_local {
        (seed0_local + 1) % m
    } else {
        seed1_local
    };

    let mut seed_coords: [(f64, f64); 2] = [projected[seed0_local], projected[seed1_local]];
    let mut assignment: Vec<usize> = vec![0; m];

    for _iter in 0..n_iter.max(1) {
        let prev_assignment = assignment.clone();
        for v in 0..m {
            let d0 = euclidean_dist(projected[v], seed_coords[0]);
            let d1 = euclidean_dist(projected[v], seed_coords[1]);
            assignment[v] = if d1 < d0 { 1 } else { 0 };
        }

        let prev_seed_coords = seed_coords;
        for j in 0usize..2 {
            let district_total_pop: i64 = (0..m)
                .filter(|&v| assignment[v] == j)
                .map(|v| local_pop[v])
                .sum();
            if district_total_pop == 0 {
                continue;
            }
            let wx: f64 = (0..m)
                .filter(|&v| assignment[v] == j)
                .map(|v| projected[v].0 * local_pop[v] as f64)
                .sum::<f64>()
                / district_total_pop as f64;
            let wy: f64 = (0..m)
                .filter(|&v| assignment[v] == j)
                .map(|v| projected[v].1 * local_pop[v] as f64)
                .sum::<f64>()
                / district_total_pop as f64;
            let mean_coord = (wx, wy);
            let nearest = (0..m)
                .min_by(|&a, &b| {
                    let da = euclidean_dist(projected[a], mean_coord);
                    let db = euclidean_dist(projected[b], mean_coord);
                    da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
                })
                .unwrap_or(0);
            seed_coords[j] = projected[nearest];
        }

        let eps = 1.0f64;
        let stable = (0..2).all(|j| euclidean_dist(seed_coords[j], prev_seed_coords[j]) < eps);
        if stable || assignment == prev_assignment {
            break;
        }
    }

    // Post-hoc rebalance: same 200-iter boundary-swap as split_subgraph_cvd
    let half_pop = total_pop / 2;
    let tolerance_pop = (balance_tolerance * total_pop as f64) as i64 + 1;
    for _ in 0..200 {
        let left_pop: i64 = (0..m)
            .filter(|&v| assignment[v] == 0)
            .map(|v| local_pop[v])
            .sum();
        let excess = left_pop - half_pop;
        if excess.abs() <= tolerance_pop {
            break;
        }
        let (heavy_side, light_side) = if excess > 0 { (0usize, 1usize) } else { (1, 0) };
        let mut best: Option<(usize, i64)> = None;
        for v in 0..m {
            if assignment[v] != heavy_side {
                continue;
            }
            if !local_adj[v].iter().any(|&nb| assignment[nb] == light_side) {
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
                assignment[v] = light_side;
            }
            None => break,
        }
    }

    let mut left = HashSet::new();
    let mut right = HashSet::new();
    for (local, &side) in assignment.iter().enumerate() {
        let global = sorted[local];
        if side == 0 {
            left.insert(global);
        } else {
            right.insert(global);
        }
    }
    Ok((left, right))
}

/// Run the full bisection tree using Centroidal Voronoi at each node.
///
/// Structurally identical to run_all_splits_sa but calls split_subgraph_cvd
/// at each bisection node instead of SA or METIS.
pub fn run_all_splits_cvd(
    adjacency: &[Vec<usize>],
    vertex_weights: &[i64],
    num_districts: usize,
    balance_tolerance: f64,
    intermediate_dir: Option<&Path>,
    n_iter: usize,
    base_seed: u64,
    metric: VoronoiMetric,
    tract_centroids: &[(f64, f64)],
) -> Result<HashMap<usize, usize>, String> {
    // Early validation for geographic metric
    if metric == VoronoiMetric::Geographic && tract_centroids.is_empty() {
        return Err("[CONFIG] --cvd-metric geographic requires centroid data. \
             Run: bisect fetch --type centroids"
            .to_string());
    }
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
                let (left, right) = match metric {
                    VoronoiMetric::GraphDistance => {
                        let cvd_seed = derive_cvd_seed(base_seed, &node.path);
                        split_subgraph_cvd(
                            adjacency,
                            vertex_weights,
                            &tracts,
                            2,
                            node_ufactor,
                            n_iter,
                            cvd_seed,
                        )
                        .map_err(|e| format!("depth {} node '{}' (CVD): {e}", depth, node.path))?
                    }
                    VoronoiMetric::Geographic => split_subgraph_cvd_geographic(
                        adjacency,
                        vertex_weights,
                        tract_centroids,
                        &tracts,
                        node_ufactor,
                        n_iter,
                        base_seed,
                        &node.path,
                    )
                    .map_err(|e| format!("depth {} node '{}' (CVD-geo): {e}", depth, node.path))?,
                };
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
            "CVD bisection incomplete: {}/{n} tracts assigned",
            assignments.len()
        ));
    }
    Ok(assignments)
}
