use super::*;

// ── CompactBisect (B.7) ───────────────────────────────────────────────────────

/// Configuration for the CompactBisect algorithm.
///
/// At each bisection level, runs `seeds_per_level` candidate splits via METIS,
/// filters to those within `epsilon` of the minimum edge-cut, then selects the
/// candidate maximising geometric-mean Polsby-Popper: sqrt(PP(L) * PP(R)).
///
/// When `graph` has no geometry data (vertex_areas/vertex_ext_perimeters empty),
/// CompactBisect degrades gracefully to standard minimum-edge-cut selection.
#[derive(Debug, Clone)]
pub struct CompactBisectOpts {
    /// Number of METIS seeds to try at each bisection node. Higher = better
    /// approximation of the true minimum. Typical: 20-100.
    pub seeds_per_level: usize,
    /// Fraction above minimum edge-cut that is still considered "near-minimum".
    /// Candidates with EC > (1+epsilon)*EC_min are excluded from PP selection.
    /// Typical: 0.05 (5%).
    pub epsilon: f64,
}

impl Default for CompactBisectOpts {
    fn default() -> Self {
        Self {
            seeds_per_level: 50,
            epsilon: 0.05,
        }
    }
}

/// Select the best bisection candidate by geometric-mean Polsby-Popper,
/// among candidates within epsilon of the minimum edge-cut.
///
/// Returns the (left, right) partition maximising sqrt(PP(left)*PP(right)).
/// Falls back to the minimum-edge-cut candidate if geometry is unavailable.
pub(crate) fn select_compact_split(
    candidates: &[(HashSet<usize>, HashSet<usize>, f64)], // (left, right, edge_cut)
    graph: &bisect_data::AdjacencyGraph,
    epsilon: f64,
) -> (HashSet<usize>, HashSet<usize>) {
    assert!(!candidates.is_empty());

    let ec_min = candidates
        .iter()
        .map(|(_, _, ec)| *ec)
        .fold(f64::INFINITY, f64::min);
    let threshold = ec_min * (1.0 + epsilon);

    let near_min: Vec<&(HashSet<usize>, HashSet<usize>, f64)> = candidates
        .iter()
        .filter(|(_, _, ec)| *ec <= threshold)
        .collect();

    // If no geometry: return the minimum-edge-cut candidate
    if !graph.has_geometry() {
        let best = near_min
            .iter()
            .min_by(|a, b| a.2.partial_cmp(&b.2).unwrap_or(std::cmp::Ordering::Equal))
            .copied()
            .unwrap_or(&candidates[0]);
        return (best.0.clone(), best.1.clone());
    }

    // Geometric-mean PP selection: argmax sqrt(PP(L) * PP(R))
    let best_idx = near_min
        .iter()
        .enumerate()
        .map(|(i, (l, r, _))| {
            let gm = graph.geometric_mean_pp(l, r).unwrap_or(0.0);
            (i, gm)
        })
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(i, _)| i)
        .unwrap_or(0);

    let best = near_min[best_idx];
    (best.0.clone(), best.1.clone())
}

/// Return the METIS version string.
/// libmetis is vendored at compile time via the metis-rs crate; no external binary needed.
pub fn detect_gpmetis_version() -> String {
    #[cfg(feature = "c-ffi-engine")]
    {
        "METIS 5.1.0 (vendored C FFI via metis-rs 0.2)".to_string()
    }
    #[cfg(not(feature = "c-ffi-engine"))]
    {
        "metis-core (pure Rust workspace engine)".to_string()
    }
}

/// METIS is now embedded via the metis-rs FFI crate — no external gpmetis binary needed.
/// Kept for API compatibility; always returns None.
#[allow(dead_code)]
pub fn find_gpmetis() -> Option<String> {
    None
}

/// BFS connectivity check: returns true if the subgraph (local indices 0..n) is connected.
pub(crate) fn is_connected(adj: &[Vec<usize>]) -> bool {
    let n = adj.len();
    if n <= 1 {
        return true;
    }
    let mut visited = vec![false; n];
    let mut queue = std::collections::VecDeque::new();
    visited[0] = true;
    queue.push_back(0usize);
    while let Some(v) = queue.pop_front() {
        for &nb in &adj[v] {
            if !visited[nb] {
                visited[nb] = true;
                queue.push_back(nb);
            }
        }
    }
    visited.iter().all(|&v| v)
}

pub(crate) fn is_connected_subset(adjacency: &[Vec<usize>], vertices: &HashSet<usize>) -> bool {
    if vertices.len() <= 1 {
        return true;
    }
    let nodes: Vec<usize> = vertices.iter().copied().collect();
    rgraph_core::node_subset_connected(adjacency, &nodes)
        .expect("validated bisection-runner adjacency and subset")
}

/// Convert adjacency list to CSR format required by the METIS C API.
pub(crate) fn adj_to_csr(adj: &[Vec<usize>]) -> (Vec<i32>, Vec<i32>) {
    let mut xadj = Vec::with_capacity(adj.len() + 1);
    let mut adjncy = Vec::new();
    xadj.push(0i32);
    for neighbors in adj {
        for &nb in neighbors {
            adjncy.push(nb as i32);
        }
        xadj.push(adjncy.len() as i32);
    }
    (xadj, adjncy)
}

/// Build the edge-weight array parallel to `adjncy` (CSR order).
/// Returns None when there are no edge weights (METIS uses unit weights by default).
/// Weights are scaled metres → centimetres (×100), truncated (matching Python int(x*100)),
/// and clamped to minimum 1.
pub(crate) fn ew_to_adjwgt(
    adj: &[Vec<usize>],
    ew: Option<&HashMap<(usize, usize), f64>>,
) -> Option<Vec<i32>> {
    let ew = ew?;
    if ew.is_empty() {
        return None;
    }
    let mut adjwgt = Vec::new();
    for (i, neighbors) in adj.iter().enumerate() {
        for &j in neighbors {
            let key = (i.min(j), i.max(j));
            let w_m = ew.get(&key).copied().unwrap_or(1.0);
            let w_cm = ((w_m * 100.0) as i32).max(1);
            adjwgt.push(w_cm);
        }
    }
    Some(adjwgt)
}

/// Split a subgraph (identified by `tract_indices`) into two balanced parts.
///
/// Unified path for ncon=1 (population only) and ncon=2 (population + area).
/// Returns (left_indices, right_indices) where left = partition 0, right = partition 1.
///
/// Parameters:
/// - `vwgt`: interleaved vertex weights, length = n*ncon.
///   For ncon=1: plain population array (same as before).
///   For ncon=2: [pop_0, area_0, pop_1, area_1, ...].
/// - `ncon`: number of balance constraints (1 or 2).
/// - `tpwgts`: full tpwgts array for METIS (length = nparts*ncon = 2*ncon).
///   None = METIS default (equal split).
/// - `ubvec`: per-constraint imbalance tolerances (length = ncon).
///   None = use ufactor default for all constraints.
pub fn split_subgraph(
    adjacency: &[Vec<usize>],
    vwgt: &[i64],
    ncon: usize,
    edge_weights: &HashMap<(usize, usize), f64>,
    tract_indices: &HashSet<usize>,
    // ufactor: METIS decimal multiplier (e.g. 1.001 = 0.1%). Use ufactor_for_depth().
    ufactor: f64,
    niter: u32,
    seed: Option<u64>,
    // tpwgts: full target-weights array (length = nparts*ncon = 2*ncon).
    // None = equal 50/50 split (METIS default).
    tpwgts: Option<Vec<f32>>,
    // ubvec: per-constraint imbalance tolerances.  None = use ufactor for all constraints.
    ubvec: Option<Vec<f32>>,
) -> Result<(HashSet<usize>, HashSet<usize>), String> {
    if tract_indices.len() <= 1 {
        return Ok((tract_indices.clone(), HashSet::new()));
    }

    // Build local index mapping: local → global (sorted for determinism)
    let mut sorted: Vec<usize> = tract_indices.iter().copied().collect();
    sorted.sort_unstable();
    let global_to_local: HashMap<usize, usize> = sorted
        .iter()
        .enumerate()
        .map(|(local, &global)| (global, local))
        .collect();
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

    // Build local vertex weights: extract ncon values per vertex from interleaved global array
    let local_vwgt: Vec<i32> = sorted
        .iter()
        .flat_map(|&g| (0..ncon).map(move |c| vwgt[g * ncon + c].max(1) as i32))
        .collect();

    // Subgraph edge weights (reindex to local, canonical order)
    let sub_ew: HashMap<(usize, usize), f64> = edge_weights
        .iter()
        .filter(|&(&(u, v), _)| tract_indices.contains(&u) && tract_indices.contains(&v))
        .map(|(&(u, v), &w)| {
            let lu = global_to_local[&u];
            let lv = global_to_local[&v];
            ((lu.min(lv), lu.max(lv)), w)
        })
        .collect();

    let ew_opt = if sub_ew.is_empty() {
        None
    } else {
        Some(&sub_ew)
    };

    // Build CSR for the METIS FFI
    let (xadj, adjncy) = adj_to_csr(&sub_adj);
    let adjwgt = ew_to_adjwgt(&sub_adj, ew_opt);

    // Empty-graph fast path: no edges → METIS has no signal; split by sorted order.
    // This handles isolated subgraphs (e.g. two disconnected components) without
    // calling METIS, which may stall on empty adjacency in the pure-Rust engine.
    if adjncy.is_empty() {
        let total_pop: i64 = sorted.iter().map(|&g| vwgt[g * ncon]).sum();
        let target_frac = tpwgts.as_ref().map(|tw| tw[0] as f64).unwrap_or(0.5);
        let left_target = (target_frac * total_pop as f64) as i64;
        let mut running = 0i64;
        let mut split_at = 1usize;
        for (idx, &g) in sorted.iter().enumerate() {
            if idx > 0 && running >= left_target {
                split_at = idx;
                break;
            }
            running += vwgt[g * ncon].max(1);
            split_at = (idx + 1).min(sorted.len() - 1);
        }
        let left: HashSet<usize> = sorted[..split_at].iter().copied().collect();
        let right: HashSet<usize> = sorted[split_at..].iter().copied().collect();
        return Ok((left, right));
    }

    // METIS imbalance = (1 + ufactor/1000).
    //   ufactor=1  → 0.1% tolerance
    //   ufactor=50 → 5.0% tolerance
    let uf_int = ((ufactor - 1.0) * 1000.0).round() as i32;
    // Floor at 5 (0.5%): Contig+MinConn constraints limit METIS's partition choices,
    // so per-level tolerance must stay above the practical minimum for small subgraphs.
    let uf_int = uf_int.clamp(5, 1000);

    // ── Bisect via the selected METIS backend ────────────────────────────────
    let part: Vec<i32> = {
        #[cfg(feature = "c-ffi-engine")]
        {
            let mut part = vec![0i32; n];
            let graph = metis::Graph::new(ncon as i32, 2, &xadj, &adjncy)
                .map_err(|e| format!("METIS graph init: {e}"))?
                .set_vwgt(&local_vwgt);
            let graph = if let Some(ref ew) = adjwgt {
                graph.set_adjwgt(ew)
            } else {
                graph
            };
            let graph = if let Some(ref tw) = tpwgts {
                graph.set_tpwgts(tw)
            } else {
                graph
            };
            let graph = if let Some(ref ub) = ubvec {
                graph.set_ubvec(ub)
            } else {
                graph
            };
            let graph = graph
                .set_option(metis::option::UFactor(uf_int.max(1)))
                .set_option(metis::option::NIter(niter as i32));
            let graph = if let Some(s) = seed {
                graph.set_option(metis::option::Seed(((s & 0x7FFF_FFFF) as i32).max(1)))
            } else {
                graph
            };
            if tpwgts.is_some() {
                graph
                    .part_kway(&mut part)
                    .map_err(|e| format!("METIS kway bisection failed: {e}"))?;
            } else {
                graph
                    .part_recursive(&mut part)
                    .map_err(|e| format!("METIS bisection failed: {e}"))?;
            }
            part
        }
        #[cfg(not(feature = "c-ffi-engine"))]
        {
            // Pure-Rust fallback via metis-core.
            // ncon=2 (AreaSection dual constraint) is not supported without c-ffi-engine.
            if ncon > 1 {
                return Err(
                    "[CONFIG] AreaSection (ncon=2) requires the c-ffi-engine feature. \
                     Rebuild with default features or use --metis-engine c-ffi."
                        .to_string(),
                );
            }
            use metis_core::{
                CsrGraph as RustCsrGraph, MetisParams as RustBisectParams,
                MetisPartitioner as RustBisectPartitioner, Partitioner as RustBisectTrait,
            };
            let g = RustCsrGraph::new(
                xadj.iter().map(|&x| x as u32).collect(),
                adjncy.iter().map(|&x| x as u32).collect(),
                1,
                local_vwgt.clone(),
                adjwgt.clone(),
            )
            .map_err(|e| format!("metis-core bisection graph: {e}"))?;
            let uf_u32 = (uf_int as u32).clamp(1, 1000);
            let mut params = RustBisectParams::kway()
                .with_ufactor(uf_u32)
                .with_niter(niter)
                .with_coarsen_to(20);
            if let Some(seed) = seed {
                params = params.with_seed(seed);
            }
            let partition = if let Some(ref tw) = tpwgts {
                // Asymmetric split: convert f32 fracs (first 2 values) to u32 thousandths.
                let fracs: Vec<u32> = tw
                    .iter()
                    .take(2)
                    .map(|&f| (f * 1000.0).round() as u32)
                    .collect();
                RustBisectPartitioner::with_params(params, 2).split_weighted(&g, &fracs, seed)
            } else {
                RustBisectPartitioner::with_params(params, 2).split(&g, 2, seed)
            }
            .map_err(|e| format!("metis-core bisection: {e}"))?;
            partition.assignment().iter().map(|&p| p as i32).collect()
        }
    };

    let mut left = HashSet::new();
    let mut right = HashSet::new();
    for (local, &p) in part.iter().enumerate() {
        let global = sorted[local];
        if p == 0 {
            left.insert(global);
        } else {
            right.insert(global);
        }
    }

    // Post-hoc boundary-swap rebalancing.
    // METIS 5.2 (vendored) sometimes produces 0.5-1% balance error without Contig.
    // Move small boundary tracts from the heavier side to the lighter side until
    // both sides are within `ufactor` of their target weights.
    // left/right store GLOBAL indices; use global_to_local for local_vwgt/sub_adj access.
    let total_pop: i64 = local_vwgt.chunks(ncon).map(|c| c[0] as i64).sum();
    let left_target = tpwgts
        .as_ref()
        .map(|tw| (tw[0] as f64 * total_pop as f64) as i64)
        .unwrap_or(total_pop / 2);
    let tolerance_pop = ((ufactor - 1.0).max(0.0) * left_target.max(1) as f64) as i64 + 1;

    for _ in 0..200 {
        let left_pop: i64 = left
            .iter()
            .map(|&g| local_vwgt[global_to_local[&g] * ncon] as i64)
            .sum();
        let excess = left_pop - left_target;
        if excess.abs() <= tolerance_pop {
            break;
        }

        let (heavy, light) = if excess > 0 {
            (&left, &right)
        } else {
            (&right, &left)
        };
        let light_global: HashSet<usize> = light.clone();
        let heavy_global: HashSet<usize> = heavy.clone();

        // Boundary tracts on the heavy side: those with a neighbor in the light side.
        let mut best: Option<(usize, i64)> = None;
        for &g in &heavy_global {
            let lg = global_to_local[&g];
            let has_light_nb = sub_adj[lg]
                .iter()
                .any(|&nb_local| light_global.contains(&sorted[nb_local]));
            if has_light_nb {
                let mut heavy_after = heavy_global.clone();
                heavy_after.remove(&g);
                if !is_connected_subset(adjacency, &heavy_after) {
                    continue;
                }
                let mut light_after = light_global.clone();
                light_after.insert(g);
                if !is_connected_subset(adjacency, &light_after) {
                    continue;
                }
                let pop = local_vwgt[lg * ncon] as i64;
                let score = (pop - excess.abs()).abs();
                if best.map_or(true, |(_, s)| score < s) {
                    best = Some((g, score));
                }
            }
        }
        match best {
            Some((g, _)) => {
                if excess > 0 {
                    left.remove(&g);
                    right.insert(g);
                } else {
                    right.remove(&g);
                    left.insert(g);
                }
            }
            None => break,
        }
    }

    Ok((left, right))
}

/// Population-area Lorenz analysis for AreaSection feasibility.
///
/// Sorts tracts densest-first, accumulates cumulative (area_frac, pop_frac).
/// Returns:
///   - `curve`: Vec<(area_frac, pop_frac)> sampled at each tract boundary
///   - `natural_pop_at_half_area`: population fraction contained in the densest 50% of area
///   - `suggested_left_k`: nearest valid district count to the natural split
///
/// Used to pre-filter infeasible ratios before running dual-constraint METIS.
pub fn population_lorenz(
    vertex_weights: &[i64],
    vertex_areas_m2: &[f64],
    num_districts: usize,
) -> (Vec<(f64, f64)>, f64, usize) {
    let total_pop: f64 = vertex_weights.iter().map(|&w| w as f64).sum();
    let total_area: f64 = vertex_areas_m2.iter().sum();
    if total_pop == 0.0 || total_area == 0.0 {
        return (vec![], 0.0, num_districts / 2);
    }

    // Sort tract indices by density (pop/area), densest first
    let mut order: Vec<usize> = (0..vertex_weights.len()).collect();
    order.sort_by(|&a, &b| {
        let da = vertex_weights[a] as f64 / vertex_areas_m2[a].max(1.0);
        let db = vertex_weights[b] as f64 / vertex_areas_m2[b].max(1.0);
        db.partial_cmp(&da).unwrap_or(std::cmp::Ordering::Equal)
    });

    let mut curve: Vec<(f64, f64)> = Vec::with_capacity(order.len() + 1);
    curve.push((0.0, 0.0));
    let mut cum_area = 0.0f64;
    let mut cum_pop = 0.0f64;
    let mut natural_pop_at_half = 0.0f64;
    let mut crossed_half = false;

    for &i in &order {
        cum_area += vertex_areas_m2[i] / total_area;
        cum_pop += vertex_weights[i] as f64 / total_pop;
        curve.push((cum_area, cum_pop));
        if !crossed_half && cum_area >= 0.5 {
            // Interpolate to find exact pop fraction at area = 0.5
            let prev = curve[curve.len() - 2];
            let t = (0.5 - prev.0) / (cum_area - prev.0).max(1e-12);
            natural_pop_at_half = prev.1 + t * (cum_pop - prev.1);
            crossed_half = true;
        }
    }

    // Nearest valid district count: round natural_pop_at_half × num_districts,
    // clamped to 1..=num_districts/2 (we always label the smaller side left).
    let natural_k_raw = (natural_pop_at_half * num_districts as f64).round() as usize;
    let max_left = num_districts / 2;
    // The natural split could be on either side; take the smaller label
    let natural_k = if natural_k_raw > max_left {
        num_districts - natural_k_raw
    } else {
        natural_k_raw
    }
    .clamp(1, max_left);

    (curve, natural_pop_at_half, natural_k)
}

/// For a given population fraction `p`, return the minimum area fraction needed
/// (greedily taking the densest tracts first — non-contiguous lower bound).
pub(crate) fn lorenz_min_area(
    vertex_weights: &[i64],
    vertex_areas_m2: &[f64],
    target_pop_frac: f64,
) -> f64 {
    let total_pop: f64 = vertex_weights.iter().map(|&w| w as f64).sum();
    let total_area: f64 = vertex_areas_m2.iter().sum();
    if total_area == 0.0 {
        return 0.0;
    }

    let mut order: Vec<usize> = (0..vertex_weights.len()).collect();
    order.sort_by(|&a, &b| {
        let da = vertex_weights[a] as f64 / vertex_areas_m2[a].max(1.0);
        let db = vertex_weights[b] as f64 / vertex_areas_m2[b].max(1.0);
        db.partial_cmp(&da).unwrap_or(std::cmp::Ordering::Equal)
    });

    let mut cum_pop = 0.0f64;
    let mut cum_area = 0.0f64;
    for &i in &order {
        if cum_pop >= target_pop_frac * total_pop {
            break;
        }
        cum_pop += vertex_weights[i] as f64;
        cum_area += vertex_areas_m2[i];
    }
    cum_area / total_area
}
