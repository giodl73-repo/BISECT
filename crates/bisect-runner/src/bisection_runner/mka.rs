use super::*;

// ── Moving-Knife Algorithm (T.13) ────────────────────────────────────────────

/// Compactness metric used by the Moving-Knife Algorithm.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MkaMetric {
    /// Reock score: Area(D) / Area(MEC(D)) — default. Requires centroid positions only.
    Reock,
    /// Polsby-Popper: 4π·Area(D) / Perimeter(D)² — requires precomputed edge lengths.
    /// Phase 1: falls back to Reock (perimeter data not yet in LoadedGraph).
    PolsbyPopper,
}

/// Derive a per-node seed for MKA via SHA-256.
///
/// Prefix "MKA_INIT_" is distinct from "CVD_GEO_INIT_" and "CVD_INIT_".
/// Length prefix on node_path eliminates boundary-collision risk.
pub fn mka_seed(base_seed: u64, node_path: &str) -> u64 {
    use sha2::{Digest, Sha256};
    let mut h = Sha256::new();
    h.update(b"MKA_INIT_");
    h.update((node_path.len() as u32).to_le_bytes());
    h.update(node_path.as_bytes());
    h.update(base_seed.to_le_bytes());
    let d = h.finalize();
    u64::from_le_bytes(d[..8].try_into().unwrap())
}

/// Welzl's minimum enclosing circle algorithm (randomised, expected O(n)).
///
/// Returns `(cx, cy, radius_sq)` — the squared radius avoids a sqrt for
/// intermediate comparisons.  The final Reock denominator uses `PI * radius_sq`.
/// For an empty point set returns (0.0, 0.0, 0.0).
/// For a single point returns that point with radius_sq = 0.0.
pub fn welzl_mec(points: &[(f64, f64)]) -> (f64, f64, f64) {
    if points.is_empty() {
        return (0.0, 0.0, 0.0);
    }
    if points.len() == 1 {
        return (points[0].0, points[0].1, 0.0);
    }

    // Helper: circumscribed circle of three points.
    // Returns None if the three points are collinear.
    let circumcircle = |a: (f64, f64), b: (f64, f64), c: (f64, f64)| -> Option<(f64, f64, f64)> {
        let ax = a.0;
        let ay = a.1;
        let bx = b.0 - ax;
        let by = b.1 - ay;
        let cx = c.0 - ax;
        let cy = c.1 - ay;
        let d = 2.0 * (bx * cy - by * cx);
        if d.abs() < 1e-10 {
            return None;
        }
        let ux = (cy * (bx * bx + by * by) - by * (cx * cx + cy * cy)) / d;
        let uy = (bx * (cx * cx + cy * cy) - cx * (bx * bx + by * by)) / d;
        let r2 = ux * ux + uy * uy;
        Some((ax + ux, ay + uy, r2))
    };

    // Smallest enclosing circle from 1 or 2 boundary points.
    let circle_from_boundary = |p: &[(f64, f64)]| -> (f64, f64, f64) {
        match p.len() {
            0 => (0.0, 0.0, 0.0),
            1 => (p[0].0, p[0].1, 0.0),
            _ => {
                let cx = (p[0].0 + p[1].0) * 0.5;
                let cy = (p[0].1 + p[1].1) * 0.5;
                let dx = p[0].0 - p[1].0;
                let dy = p[0].1 - p[1].1;
                (cx, cy, (dx * dx + dy * dy) * 0.25)
            }
        }
    };

    let point_in_circle = |pt: (f64, f64), cx: f64, cy: f64, r2: f64| -> bool {
        let dx = pt.0 - cx;
        let dy = pt.1 - cy;
        dx * dx + dy * dy <= r2 * (1.0 + 1e-10)
    };

    // Iterative Welzl (avoids stack overflow on large inputs).
    // We work on a shuffled copy to get expected O(n) performance.
    // The seed is fixed (not caller-provided) because MKA determinism comes
    // from the orientation sweep, not from the MEC itself.
    use rand::SeedableRng;
    let mut rng = rand::rngs::SmallRng::seed_from_u64(0xDEAD_BEEF_1234_5678);
    let mut pts: Vec<(f64, f64)> = points.to_vec();
    use rand::seq::SliceRandom;
    pts.shuffle(&mut rng);

    // Incremental Welzl with explicit boundary set (up to 3 points).
    let mut circle = circle_from_boundary(&pts[..1]);
    let (mut cx, mut cy, mut r2) = circle;

    for i in 1..pts.len() {
        let p = pts[i];
        if !point_in_circle(p, cx, cy, r2) {
            // p must be on the boundary — restart with {p}.
            let (ncx, ncy, nr2) = circle_from_boundary(&[p]);
            cx = ncx;
            cy = ncy;
            r2 = nr2;
            for j in 0..i {
                let q = pts[j];
                if !point_in_circle(q, cx, cy, r2) {
                    // q must also be on the boundary — restart with {p, q}.
                    let (ncx, ncy, nr2) = circle_from_boundary(&[p, q]);
                    cx = ncx;
                    cy = ncy;
                    r2 = nr2;
                    for k in 0..j {
                        let s = pts[k];
                        if !point_in_circle(s, cx, cy, r2) {
                            // Three boundary points: use circumcircle.
                            if let Some((ncx, ncy, nr2)) = circumcircle(p, q, s) {
                                cx = ncx;
                                cy = ncy;
                                r2 = nr2;
                            } else {
                                // Collinear: use the farthest pair.
                                let d_pq = {
                                    let dx = p.0 - q.0;
                                    let dy = p.1 - q.1;
                                    dx * dx + dy * dy
                                };
                                let d_ps = {
                                    let dx = p.0 - s.0;
                                    let dy = p.1 - s.1;
                                    dx * dx + dy * dy
                                };
                                let d_qs = {
                                    let dx = q.0 - s.0;
                                    let dy = q.1 - s.1;
                                    dx * dx + dy * dy
                                };
                                if d_pq >= d_ps && d_pq >= d_qs {
                                    let (ncx, ncy, nr2) = circle_from_boundary(&[p, q]);
                                    cx = ncx;
                                    cy = ncy;
                                    r2 = nr2;
                                } else if d_ps >= d_qs {
                                    let (ncx, ncy, nr2) = circle_from_boundary(&[p, s]);
                                    cx = ncx;
                                    cy = ncy;
                                    r2 = nr2;
                                } else {
                                    let (ncx, ncy, nr2) = circle_from_boundary(&[q, s]);
                                    cx = ncx;
                                    cy = ncy;
                                    r2 = nr2;
                                }
                            }
                        }
                    }
                }
            }
        }
        circle = (cx, cy, r2);
        let _ = circle;
    }
    (cx, cy, r2)
}

/// Compute the Reock compactness score for a set of projected centroid positions.
///
/// Reock_approx = (n_tracts * mean_tract_area) / (PI * MEC_radius²)
///
/// When `tract_areas` is empty (unit tests), uses `n` as the proxy numerator.
/// The result is clamped to [0.0, 1.0] because the centroid-only MEC
/// approximation can overestimate the Reock score for large boundary tracts.
pub fn reock_score(projected_pts: &[(f64, f64)], tract_areas: &[f64]) -> f64 {
    if projected_pts.is_empty() {
        return 0.0;
    }
    let (_, _, r2) = welzl_mec(projected_pts);
    let mec_area = std::f64::consts::PI * r2;
    if mec_area <= 0.0 {
        return 1.0_f64.clamp(0.0, 1.0);
    }
    let numerator = if tract_areas.is_empty() {
        projected_pts.len() as f64
    } else {
        tract_areas.iter().sum::<f64>()
    };
    (numerator / mec_area).clamp(0.0, 1.0)
}

/// Moving-Knife Algorithm — find the orientation θ ∈ [0, π) that maximises
/// the minimum Reock score of the two halves produced by a linear sweep.
///
/// Returns just θ* in radians for use by AreaSection hybrid warm-start.
pub fn split_subgraph_mka_direction(
    tract_indices: &HashSet<usize>,
    centroids: &[(f64, f64)],
    n_orientations: usize,
) -> f64 {
    use std::f64::consts::PI;
    if tract_indices.is_empty() || n_orientations == 0 {
        return 0.0;
    }

    let mut sorted: Vec<usize> = tract_indices.iter().copied().collect();
    sorted.sort_unstable();
    let m = sorted.len();

    let projected: Vec<(f64, f64)> = sorted
        .iter()
        .map(|&g| {
            if g < centroids.len() {
                let (lon, lat) = centroids[g];
                albers_project(lon, lat)
            } else {
                (0.0, 0.0)
            }
        })
        .collect();

    let mut best_score = f64::NEG_INFINITY;
    let mut best_theta = 0.0f64;

    for k in 0..n_orientations {
        let theta = (k as f64) * PI / (n_orientations as f64);
        let cos_t = theta.cos();
        let sin_t = theta.sin();

        // Project each centroid onto the sweep axis.
        let mut proj_vals: Vec<(f64, usize)> = (0..m)
            .map(|i| (projected[i].0 * cos_t + projected[i].1 * sin_t, i))
            .collect();
        proj_vals.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

        // Find balance split: cumulative count >= m/2 (population-unaware direction-only version).
        let split_idx = (m / 2).saturating_sub(1).min(m - 1);

        let left_pts: Vec<(f64, f64)> = proj_vals[..=split_idx]
            .iter()
            .map(|&(_, i)| projected[i])
            .collect();
        let right_pts: Vec<(f64, f64)> = proj_vals[split_idx + 1..]
            .iter()
            .map(|&(_, i)| projected[i])
            .collect();

        if left_pts.is_empty() || right_pts.is_empty() {
            continue;
        }

        let rl = reock_score(&left_pts, &[]);
        let rr = reock_score(&right_pts, &[]);
        let score = rl.min(rr);

        if score > best_score {
            best_score = score;
            best_theta = theta;
        }
    }
    best_theta
}

/// Moving-Knife Algorithm bisection (T.13 spec 3.75/4).
///
/// For each orientation θ ∈ [0°, 180°) in steps of 180/n_orientations degrees:
///   1. Project all tract centroids onto the sweep axis (cos θ, sin θ).
///   2. Sort by projection value.
///   3. Binary-search for the population-balanced split point.
///   4. Compute the Reock score for each half.
///   5. Track the orientation that maximises min(Reock_left, Reock_right).
/// Post-hoc: 200-iteration boundary-swap rebalance (same as CVD/BFS).
pub fn split_subgraph_mka(
    adjacency: &[Vec<usize>],
    vertex_weights: &[i64],
    tract_indices: &HashSet<usize>,
    centroids: &[(f64, f64)],
    balance_tolerance: f64,
    n_orientations: usize,
    metric: MkaMetric,
    base_seed: u64,
    node_path: &str,
) -> Result<(HashSet<usize>, HashSet<usize>), String> {
    use std::f64::consts::PI;

    if centroids.is_empty() {
        return Err("[CONFIG] --structure moving-knife requires centroid data. \
             Run: bisect fetch --type centroids"
            .to_string());
    }

    if tract_indices.len() <= 1 {
        return Ok((tract_indices.clone(), HashSet::new()));
    }

    if tract_indices.len() == 2 {
        let mut sorted: Vec<usize> = tract_indices.iter().copied().collect();
        sorted.sort_unstable();
        let left: HashSet<usize> = std::iter::once(sorted[0]).collect();
        let right: HashSet<usize> = std::iter::once(sorted[1]).collect();
        return Ok((left, right));
    }

    // Build local index (sorted for determinism).
    let mut sorted: Vec<usize> = tract_indices.iter().copied().collect();
    sorted.sort_unstable();
    let global_to_local: HashMap<usize, usize> =
        sorted.iter().enumerate().map(|(i, &g)| (g, i)).collect();
    let m = sorted.len();

    // Build local adjacency for post-hoc rebalance.
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
    let target_pop = total_pop / 2;

    // Project centroids via Albers.
    let projected: Vec<(f64, f64)> = sorted
        .iter()
        .map(|&g| {
            if g < centroids.len() {
                let (lon, lat) = centroids[g];
                albers_project(lon, lat)
            } else {
                (0.0, 0.0)
            }
        })
        .collect();

    let n_orient = n_orientations.max(1);
    let mut best_score = f64::NEG_INFINITY;
    let mut best_assignment: Vec<usize> = vec![0; m];

    // Tie-breaking RNG (seed from MKA seed formula).
    let _tie_seed = mka_seed(base_seed, node_path);

    for k in 0..n_orient {
        let theta = (k as f64) * PI / (n_orient as f64);
        let cos_t = theta.cos();
        let sin_t = theta.sin();

        // Project each centroid onto the sweep axis.
        let mut proj_vals: Vec<(f64, usize)> = (0..m)
            .map(|i| (projected[i].0 * cos_t + projected[i].1 * sin_t, i))
            .collect();
        proj_vals.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

        // Binary search for population balance: find first split_idx where
        // cumulative population >= target_pop.
        let mut cum_pop: i64 = 0;
        let mut split_idx = m - 1;
        for (idx, &(_, local_i)) in proj_vals.iter().enumerate() {
            cum_pop += local_pop[local_i];
            if cum_pop >= target_pop {
                split_idx = idx;
                break;
            }
        }

        // Build left/right assignment for this orientation.
        let mut assignment = vec![1usize; m]; // default: right
        for &(_, local_i) in &proj_vals[..=split_idx] {
            assignment[local_i] = 0; // left
        }

        let left_pts: Vec<(f64, f64)> = (0..m)
            .filter(|&i| assignment[i] == 0)
            .map(|i| projected[i])
            .collect();
        let right_pts: Vec<(f64, f64)> = (0..m)
            .filter(|&i| assignment[i] == 1)
            .map(|i| projected[i])
            .collect();

        if left_pts.is_empty() || right_pts.is_empty() {
            continue;
        }

        let score = match metric {
            MkaMetric::Reock => {
                let rl = reock_score(&left_pts, &[]);
                let rr = reock_score(&right_pts, &[]);
                rl.min(rr)
            }
            MkaMetric::PolsbyPopper => {
                // Phase 1: fall back to Reock (perimeter data not available).
                let rl = reock_score(&left_pts, &[]);
                let rr = reock_score(&right_pts, &[]);
                rl.min(rr)
            }
        };

        if score > best_score {
            best_score = score;
            best_assignment = assignment;
        }
    }

    // Post-hoc rebalance: 200-iteration boundary-swap (same as CVD/BFS).
    let half_pop = total_pop / 2;
    let tolerance_pop = (balance_tolerance * total_pop as f64) as i64 + 1;
    for _ in 0..200 {
        let left_pop: i64 = (0..m)
            .filter(|&v| best_assignment[v] == 0)
            .map(|v| local_pop[v])
            .sum();
        let excess = left_pop - half_pop;
        if excess.abs() <= tolerance_pop {
            break;
        }
        let (heavy_side, light_side) = if excess > 0 { (0usize, 1usize) } else { (1, 0) };
        let mut best_swap: Option<(usize, i64)> = None;
        for v in 0..m {
            if best_assignment[v] != heavy_side {
                continue;
            }
            if !local_adj[v]
                .iter()
                .any(|&nb| best_assignment[nb] == light_side)
            {
                continue;
            }
            let pop = local_pop[v];
            let score = (pop - excess.abs()).abs();
            if best_swap.map_or(true, |(_, s)| score < s) {
                best_swap = Some((v, score));
            }
        }
        match best_swap {
            Some((v, _)) => {
                best_assignment[v] = light_side;
            }
            None => break,
        }
    }

    // Convert local assignment to global sets.
    let mut left = HashSet::new();
    let mut right = HashSet::new();
    for (local, &side) in best_assignment.iter().enumerate() {
        let global = sorted[local];
        if side == 0 {
            left.insert(global);
        } else {
            right.insert(global);
        }
    }

    // Guarantee both halves are non-empty (degenerate rebalance guard).
    if left.is_empty() {
        let g = sorted[0];
        right.remove(&g);
        left.insert(g);
    }
    if right.is_empty() {
        let g = sorted[sorted.len() - 1];
        left.remove(&g);
        right.insert(g);
    }

    Ok((left, right))
}

/// Run the full bisection tree using Moving-Knife Algorithm at each node.
///
/// Mirrors run_all_splits_bfs and run_all_splits_cvd exactly but calls
/// split_subgraph_mka() at each node. Uses level-parallel Rayon (same as CVD).
pub fn run_all_splits_mka(
    adjacency: &[Vec<usize>],
    vertex_weights: &[i64],
    num_districts: usize,
    balance_tolerance: f64,
    intermediate_dir: Option<&Path>,
    n_orientations: usize,
    metric: MkaMetric,
    base_seed: u64,
    tract_centroids: &[(f64, f64)],
) -> Result<HashMap<usize, usize>, String> {
    if tract_centroids.is_empty() {
        return Err("[CONFIG] --structure moving-knife requires centroid data. \
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
                let (left, right) = split_subgraph_mka(
                    adjacency,
                    vertex_weights,
                    &tracts,
                    tract_centroids,
                    node_ufactor,
                    n_orientations,
                    metric,
                    base_seed,
                    &node.path,
                )
                .map_err(|e| format!("depth {} node '{}' (mka): {e}", depth, node.path))?;
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
            "moving-knife bisection incomplete: {}/{n} tracts assigned",
            assignments.len()
        ));
    }
    Ok(assignments)
}
