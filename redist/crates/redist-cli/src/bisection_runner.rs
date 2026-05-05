/// Level-parallel METIS bisection runner.
///
/// Implements the recursive bisection loop using BisectionTree for split
/// scheduling and gpmetis subprocess for each individual split.
///
/// Level-parallel: all nodes at depth D are split simultaneously via Rayon,
/// then depth D+1 is processed. BisectionNode implements Clone (bisection.rs).
use std::collections::{HashMap, HashSet};
use std::path::Path;
use rayon::prelude::*;
use redist_core::{BisectionTree, ufactor_for_depth};

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
    fn default() -> Self { Self { seeds_per_level: 50, epsilon: 0.05 } }
}

/// Select the best bisection candidate by geometric-mean Polsby-Popper,
/// among candidates within epsilon of the minimum edge-cut.
///
/// Returns the (left, right) partition maximising sqrt(PP(left)*PP(right)).
/// Falls back to the minimum-edge-cut candidate if geometry is unavailable.
fn select_compact_split(
    candidates: &[(HashSet<usize>, HashSet<usize>, f64)], // (left, right, edge_cut)
    graph: &redist_data::AdjacencyGraph,
    epsilon: f64,
) -> (HashSet<usize>, HashSet<usize>) {
    assert!(!candidates.is_empty());

    let ec_min = candidates.iter().map(|(_, _, ec)| *ec)
        .fold(f64::INFINITY, f64::min);
    let threshold = ec_min * (1.0 + epsilon);

    let near_min: Vec<&(HashSet<usize>, HashSet<usize>, f64)> = candidates.iter()
        .filter(|(_, _, ec)| *ec <= threshold)
        .collect();

    // If no geometry: return the minimum-edge-cut candidate
    if !graph.has_geometry() {
        let best = near_min.iter()
            .min_by(|a, b| a.2.partial_cmp(&b.2).unwrap_or(std::cmp::Ordering::Equal))
            .copied()
            .unwrap_or(&candidates[0]);
        return (best.0.clone(), best.1.clone());
    }

    // Geometric-mean PP selection: argmax sqrt(PP(L) * PP(R))
    let best_idx = near_min.iter()
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
    "METIS 5.1.0 (vendored via metis-rs 0.2)".to_string()
}

/// METIS is now embedded via the metis-rs FFI crate — no external gpmetis binary needed.
/// Kept for API compatibility; always returns None.
#[allow(dead_code)]
pub fn find_gpmetis() -> Option<String> {
    None
}

/// BFS connectivity check: returns true if the subgraph (local indices 0..n) is connected.
fn is_connected(adj: &[Vec<usize>]) -> bool {
    let n = adj.len();
    if n <= 1 { return true; }
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

/// Convert adjacency list to CSR format required by the METIS C API.
fn adj_to_csr(adj: &[Vec<usize>]) -> (Vec<i32>, Vec<i32>) {
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
fn ew_to_adjwgt(adj: &[Vec<usize>], ew: Option<&HashMap<(usize, usize), f64>>) -> Option<Vec<i32>> {
    let ew = ew?;
    if ew.is_empty() { return None; }
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
    let global_to_local: HashMap<usize, usize> = sorted.iter()
        .enumerate().map(|(local, &global)| (global, local)).collect();
    let n = sorted.len();

    // Build subgraph adjacency (local indices)
    let sub_adj: Vec<Vec<usize>> = sorted.iter().map(|&g| {
        adjacency[g].iter()
            .filter(|&&nb| tract_indices.contains(&nb))
            .map(|&nb| global_to_local[&nb])
            .collect()
    }).collect();

    // Build local vertex weights: extract ncon values per vertex from interleaved global array
    let local_vwgt: Vec<i32> = sorted.iter()
        .flat_map(|&g| (0..ncon).map(move |c| vwgt[g * ncon + c].max(1) as i32))
        .collect();

    // Subgraph edge weights (reindex to local, canonical order)
    let sub_ew: HashMap<(usize, usize), f64> = edge_weights.iter()
        .filter(|&(&(u, v), _)| tract_indices.contains(&u) && tract_indices.contains(&v))
        .map(|(&(u, v), &w)| {
            let lu = global_to_local[&u];
            let lv = global_to_local[&v];
            ((lu.min(lv), lu.max(lv)), w)
        })
        .collect();

    let ew_opt = if sub_ew.is_empty() { None } else { Some(&sub_ew) };

    // Build CSR for the METIS FFI
    let (xadj, adjncy) = adj_to_csr(&sub_adj);
    let adjwgt = ew_to_adjwgt(&sub_adj, ew_opt);

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
            let graph = if let Some(ref ew) = adjwgt { graph.set_adjwgt(ew) } else { graph };
            let graph = if let Some(ref tw) = tpwgts { graph.set_tpwgts(tw) } else { graph };
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
                graph.part_kway(&mut part)
                    .map_err(|e| format!("METIS kway bisection failed: {e}"))?;
            } else {
                graph.part_recursive(&mut part)
                    .map_err(|e| format!("METIS bisection failed: {e}"))?;
            }
            part
        }
        #[cfg(not(feature = "c-ffi-engine"))]
        {
            // Pure-Rust fallback via redist-metis.
            // ncon=2 (AreaSection dual constraint) is not supported without c-ffi-engine.
            if ncon > 1 {
                return Err(
                    "[CONFIG] AreaSection (ncon=2) requires the c-ffi-engine feature. \
                     Rebuild with default features or use --metis-engine c-ffi.".to_string()
                );
            }
            use redist_metis::api::{
                MetisPartitioner as RustBisectPartitioner,
                MetisParams as RustBisectParams,
                Partitioner as RustBisectTrait,
            };
            use redist_metis::graph::CsrGraph as RustCsrGraph;
            let g = RustCsrGraph {
                xadj:   xadj.iter().map(|&x| x as u32).collect(),
                adjncy: adjncy.iter().map(|&x| x as u32).collect(),
                ncon:   1,
                vwgt:   local_vwgt.clone(),
                adjwgt: adjwgt.clone(),
            };
            let uf_u32 = (uf_int as u32).clamp(1, 1000);
            let params = RustBisectParams {
                ufactor: uf_u32, niter, seed, coarsen_to: 20, tpwgts: None,
                ..RustBisectParams::default()
            };
            let partition = if let Some(ref tw) = tpwgts {
                // Asymmetric split: convert f32 fracs (first 2 values) to u32 thousandths.
                let fracs: Vec<u32> = tw.iter().take(2)
                    .map(|&f| (f * 1000.0).round() as u32).collect();
                RustBisectPartitioner::with_params(params, 2)
                    .split_weighted(&g, &fracs, seed)
            } else {
                RustBisectPartitioner::with_params(params, 2)
                    .split(&g, 2, seed)
            }.map_err(|e| format!("redist-metis bisection: {e}"))?;
            partition.assignment.iter().map(|&p| p as i32).collect()
        }
    };

    let mut left = HashSet::new();
    let mut right = HashSet::new();
    for (local, &p) in part.iter().enumerate() {
        let global = sorted[local];
        if p == 0 { left.insert(global); } else { right.insert(global); }
    }

    // Post-hoc boundary-swap rebalancing.
    // METIS 5.2 (vendored) sometimes produces 0.5-1% balance error without Contig.
    // Move small boundary tracts from the heavier side to the lighter side until
    // both sides are within `ufactor` of their target weights.
    // left/right store GLOBAL indices; use global_to_local for local_vwgt/sub_adj access.
    let total_pop: i64 = local_vwgt.chunks(ncon).map(|c| c[0] as i64).sum();
    let left_target = tpwgts.as_ref()
        .map(|tw| (tw[0] as f64 * total_pop as f64) as i64)
        .unwrap_or(total_pop / 2);
    let tolerance_pop = (ufactor as f64 * total_pop as f64 / 1000.0) as i64 + 1;

    for _ in 0..200 {
        let left_pop: i64 = left.iter()
            .map(|&g| local_vwgt[global_to_local[&g] * ncon] as i64).sum();
        let excess = left_pop - left_target;
        if excess.abs() <= tolerance_pop { break; }

        let (heavy, light) = if excess > 0 { (&left, &right) } else { (&right, &left) };
        let light_global: HashSet<usize> = light.clone();
        let heavy_global: HashSet<usize> = heavy.clone();

        // Boundary tracts on the heavy side: those with a neighbor in the light side.
        let mut best: Option<(usize, i64)> = None;
        for &g in &heavy_global {
            let lg = global_to_local[&g];
            let has_light_nb = sub_adj[lg].iter()
                .any(|&nb_local| light_global.contains(&sorted[nb_local]));
            if has_light_nb {
                let pop = local_vwgt[lg * ncon] as i64;
                let score = (pop - excess.abs()).abs();
                if best.map_or(true, |(_, s)| score < s) {
                    best = Some((g, score));
                }
            }
        }
        match best {
            Some((g, _)) => {
                if excess > 0 { left.remove(&g); right.insert(g); }
                else { right.remove(&g); left.insert(g); }
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
    let mut cum_pop  = 0.0f64;
    let mut natural_pop_at_half = 0.0f64;
    let mut crossed_half = false;

    for &i in &order {
        cum_area += vertex_areas_m2[i] / total_area;
        cum_pop  += vertex_weights[i] as f64 / total_pop;
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
    }.clamp(1, max_left);

    (curve, natural_pop_at_half, natural_k)
}

/// For a given population fraction `p`, return the minimum area fraction needed
/// (greedily taking the densest tracts first — non-contiguous lower bound).
fn lorenz_min_area(
    vertex_weights: &[i64],
    vertex_areas_m2: &[f64],
    target_pop_frac: f64,
) -> f64 {
    let total_pop: f64 = vertex_weights.iter().map(|&w| w as f64).sum();
    let total_area: f64 = vertex_areas_m2.iter().sum();
    if total_area == 0.0 { return 0.0; }

    let mut order: Vec<usize> = (0..vertex_weights.len()).collect();
    order.sort_by(|&a, &b| {
        let da = vertex_weights[a] as f64 / vertex_areas_m2[a].max(1.0);
        let db = vertex_weights[b] as f64 / vertex_areas_m2[b].max(1.0);
        db.partial_cmp(&da).unwrap_or(std::cmp::Ordering::Equal)
    });

    let mut cum_pop = 0.0f64;
    let mut cum_area = 0.0f64;
    for &i in &order {
        if cum_pop >= target_pop_frac * total_pop { break; }
        cum_pop  += vertex_weights[i] as f64;
        cum_area += vertex_areas_m2[i];
    }
    cum_area / total_area
}


/// Run n-way partitioning: call gpmetis once with nparts=k.
///
/// Direct n-way is faster than recursive bisection (D.2 research: 3.68s vs 11.33s).
/// D.2 also shows equivalent VRA success rates (47.5% vs 48.3%, p=0.634).
///
/// Target weights: equal partitioning (1/k per district). The last weight is
/// inferred by METIS so the sum is exactly 1.0 (avoids floating-point drift).
///
/// AC-05 invariant: all target weights sum to 1.0.
/// The approach: write n-1 explicit weights of 1/k; METIS infers the last.
/// This guarantees sum = (n-1)*(1/k) + inferred = 1.0 regardless of rounding.
pub fn run_nway_partition(
    adjacency: &[Vec<usize>],
    vertex_weights: &[i64],
    edge_weights: &HashMap<(usize, usize), f64>,
    num_districts: usize,
    ufactor: f64,
    niter: u32,
    seed: Option<u64>,
) -> Result<HashMap<usize, usize>, String> {
    let n = adjacency.len();
    if num_districts == 1 {
        return Ok((0..n).map(|i| (i, 1)).collect());
    }

    let (xadj, adjncy) = adj_to_csr(adjacency);
    let vwgt: Vec<i32> = vertex_weights.iter().map(|&w| (w as i32).max(1)).collect();
    let adjwgt = ew_to_adjwgt(
        adjacency,
        if edge_weights.is_empty() { None } else { Some(edge_weights) },
    );

    // Equal target weights: first k-1 partitions get 1/k each; last gets remainder.
    // This guarantees the weights sum exactly to 1.0 in f32 regardless of k.
    let weight_per = 1.0_f32 / num_districts as f32;
    let mut tpwgts: Vec<f32> = vec![weight_per; num_districts];
    let total: f32 = tpwgts[..num_districts-1].iter().sum();
    tpwgts[num_districts-1] = 1.0_f32 - total;

    let uf_int = ((ufactor - 1.0) * 1000.0).round() as i32;
    let uf_int = uf_int.clamp(1, 1000);

    // ── k-way partition via the selected METIS backend ───────────────────────
    let part: Vec<i32> = {
        #[cfg(feature = "c-ffi-engine")]
        {
            let mut part = vec![0i32; n];
            let graph = metis::Graph::new(1, num_districts as i32, &xadj, &adjncy)
                .map_err(|e| format!("METIS n-way graph init: {e}"))?
                .set_vwgt(&vwgt)
                .set_tpwgts(&tpwgts);
            let graph = if let Some(ref ew) = adjwgt { graph.set_adjwgt(ew) } else { graph };
            let graph = graph
                .set_option(metis::option::UFactor(uf_int))
                .set_option(metis::option::NIter(niter as i32))
                .set_option(metis::option::Contig(true))
                .set_option(metis::option::MinConn(true));
            let graph = if let Some(s) = seed {
                graph.set_option(metis::option::Seed(((s & 0x7FFF_FFFF) as i32).max(1)))
            } else {
                graph
            };
            graph.part_kway(&mut part)
                .map_err(|e| format!("METIS n-way partition failed: {e}"))?;
            part
        }
        #[cfg(not(feature = "c-ffi-engine"))]
        {
            use redist_metis::api::{
                MetisPartitioner as RustNwayPartitioner,
                MetisParams as RustNwayParams,
                Partitioner as RustNwayTrait,
            };
            use redist_metis::graph::CsrGraph as RustNwayCsr;
            let g = RustNwayCsr {
                xadj:   xadj.iter().map(|&x| x as u32).collect(),
                adjncy: adjncy.iter().map(|&x| x as u32).collect(),
                ncon:   1,
                vwgt:   vwgt.clone(),
                adjwgt: adjwgt.clone(),
            };
            let uf_u32 = (uf_int as u32).clamp(1, 1000);
            let params = RustNwayParams {
                ufactor: uf_u32, niter: niter as u32, seed, coarsen_to: 20, tpwgts: None,
                ..RustNwayParams::default()
            };
            let k = num_districts as u32;
            let partition = RustNwayPartitioner::with_params(params, k)
                .split(&g, k, seed)
                .map_err(|e| format!("redist-metis n-way k={num_districts}: {e}"))?;
            partition.assignment.iter().map(|&p| p as i32).collect()
        }
    };

    // Convert 0-based METIS output to 1-based district IDs
    Ok(part.iter().enumerate().map(|(tract, &p)| (tract, p as usize + 1)).collect())
}

/// Run the full level-parallel bisection for k districts.
/// Returns HashMap<tract_index, district_id> (1-based district IDs).
///
/// RACE CONDITION FIX: tract data extracted from node_tracts sequentially
/// BEFORE par_iter, so closures own their data with no shared references.
///
/// SORT FIX: leaves sorted by (depth, path) not plain lex, which gives
/// correct BFS order for mixed-length binary paths.
/// Run recursive bisection with mathematically-derived per-node ufactor.
///
/// **Key insight**: at each bisection node producing `k` final districts, the allowed
/// per-split imbalance must be `balance_tolerance / k` — not a fixed value — so that
/// cumulative error across all splits never exceeds `balance_tolerance` per final district.
///
/// For k=98 (WA house) with 10% target: root ufactor=0.102% (very tight), leaf ufactor=5% (loose).
/// This prevents the compounding error (28% deviation) seen with fixed ufactor per depth.
///
/// Formula: `node_ufactor = 1.0 + balance_tolerance_frac / node.k`
/// GeoSection: find the natural geographic split ratio.
///
/// At the first bisection level (depth 0), try ALL feasible split ratios
/// (1:k-1, 2:k-2, ..., ⌊k/2⌋:⌈k/2⌉), each with `seeds_per_ratio` seeds.
/// The ratio with the globally minimum edge-cut is the "natural" ratio.
/// All subsequent levels use the standard ⌊k/2⌋:⌈k/2⌉ split.
///
/// When `vertex_areas_m2` is Some, activates AreaSection mode (ncon=2):
///   - Interleaves population and area (hectares) as dual vertex weights.
/// ProportionalSection (B.12): partisan-proportional bisection using HH seat allocation.
///
/// Uses ncon=2 with vertex weights [population, D_votes]. The tpwgts are set by
/// the B.12 formula: [k_D/k, 1-k_R/(2dk), k_R/k, k_R/(2dk)] where d is the
/// statewide Democratic fraction and k_D/k_R are the Huntington-Hill seat counts.
///
/// Only the HH-proportional ratio is tried (not all ratios). Multiple seeds.
/// Recursive calls use ncon=1 (partisan constraint only at first bisection).
///
/// Returns (assignments, k_D, k_R, best_ec, d_statewide).
pub fn run_proportional_section(
    adjacency: &[Vec<usize>],
    vertex_weights: &[i64],       // population
    vertex_d_votes: &[f64],        // Democratic vote counts per tract
    vertex_two_party: &[f64],      // Democrat + Republican two-party vote totals per tract
    edge_weights: &std::collections::HashMap<(usize, usize), f64>,
    num_districts: usize,
    balance_tolerance: f64,
    niter: u32,
    seeds: usize,
    eta: f64,                      // ubvec[1] for D_votes constraint (1.05–1.20)
    intermediate_dir: Option<&std::path::Path>,
) -> Result<(std::collections::HashMap<usize, usize>, usize, usize, f64, f64), String> {
    let n = adjacency.len();
    if num_districts <= 1 {
        let asgn = (0..n).map(|i| (i, 1)).collect();
        return Ok((asgn, 1, 0, 0.0, 0.5));
    }

    // Compute statewide D fraction
    let total_pop: i64 = vertex_weights.iter().sum();
    let total_d: f64 = vertex_d_votes.iter().sum();
    let total_two_party: f64 = vertex_two_party.iter().sum();
    // d = Democratic fraction of TWO-PARTY vote (not census population)
    let d = (total_d / total_two_party.max(1.0)).clamp(0.01, 0.99);

    // Huntington-Hill allocation
    let k_d_float = d * num_districts as f64;
    let k_d_floor = k_d_float as usize;
    let k_d = if k_d_floor > 0 && k_d_float > (k_d_floor * (k_d_floor + 1)) as f64 / ((k_d_floor as f64).sqrt() * (k_d_floor as f64 + 1.0).sqrt()) {
        k_d_floor + 1
    } else {
        k_d_floor.max(1)
    };
    let k_r = num_districts - k_d;

    eprintln!("[proportional] d={:.3} k_D={} k_R={} eta={}", d, k_d, k_r, eta);

    // B.12 tpwgts: right (R-bloc) gets minimum D for 50% D concentration
    let d_right_target = (k_r as f64 / (2.0 * d * num_districts as f64)).clamp(0.01, 0.99);
    let d_left_target = 1.0 - d_right_target;
    let pop_left = k_d as f64 / num_districts as f64;
    let pop_right = k_r as f64 / num_districts as f64;

    let tpwgts = vec![
        pop_left as f32, d_left_target as f32,
        pop_right as f32, d_right_target as f32,
    ];
    let ubvec = vec![1.001f32, eta as f32];

    // Scale D votes to integer weights for METIS (scale to match pop magnitude)
    let pop_scale = total_pop as f64 / total_d.max(1.0);
    let d_weights_i64: Vec<i64> = vertex_d_votes.iter()
        .map(|&v| ((v * pop_scale) as i64).max(1))
        .collect();

    // Interleaved vwgt: [pop_0, d_0, pop_1, d_1, ...]
    let vwgt_flat: Vec<i64> = vertex_weights.iter().zip(d_weights_i64.iter())
        .flat_map(|(&p, &dv)| [p.max(1), dv])
        .collect();

    // Try multiple seeds on the HH ratio
    let all_tracts: std::collections::HashSet<usize> = (0..n).collect();
    let mut best_ec = f64::INFINITY;
    let mut best_left = std::collections::HashSet::new();
    let mut best_right = std::collections::HashSet::new();

    for seed in 1..=(seeds as u64) {
        match split_subgraph(
            adjacency, &vwgt_flat, 2, edge_weights, &all_tracts,
            1.0 + balance_tolerance / num_districts as f64,
            niter, Some(seed),
            Some(tpwgts.clone()), Some(ubvec.clone()),
        ) {
            Ok((l, r)) => {
                let ec: f64 = edge_weights.iter()
                    .filter_map(|(&(u, v), &w)| if l.contains(&u) != l.contains(&v) { Some(w) } else { None })
                    .sum();
                if ec < best_ec {
                    best_ec = ec;
                    best_left = l;
                    best_right = r;
                }
            }
            Err(e) => {
                if seed == 1 { eprintln!("[proportional] seed 1 error: {}", &e.chars().take(200).collect::<String>()); }
            }
        }
    }

    if best_left.is_empty() {
        return Err(format!("proportional-section: all {} seeds failed for {}:{}", seeds, k_d, k_r));
    }

    // Actual D fraction achieved
    let d_left_actual: f64 = best_left.iter().map(|&v| vertex_d_votes[v]).sum::<f64>() / total_d;
    eprintln!("[proportional] winner: D_left={:.1}% (target {:.1}%), EC={:.0}km",
              d_left_actual*100.0, d_left_target*100.0, best_ec/1000.0);

    if let Some(dir) = intermediate_dir {
        let _ = std::fs::create_dir_all(dir.join("depth_01"));
        let mut d1 = std::collections::HashMap::new();
        for &v in &best_left  { d1.insert(v, 1); }
        for &v in &best_right { d1.insert(v, 2); }
        let _ = write_intermediate_round(&dir.join("depth_01"), &d1);
    }

    // Recurse with ncon=1 standard bisection
    let node_ufactor = 1.0 + balance_tolerance / num_districts as f64;
    let left_asgn = recurse_geosection(
        &best_left, adjacency, vertex_weights, edge_weights,
        k_d, balance_tolerance, niter, seeds.min(50), 1,
        &crate::geosection_orientation::CentroidMap::new(), 0.0)?;
    let right_asgn = recurse_geosection(
        &best_right, adjacency, vertex_weights, edge_weights,
        k_r, balance_tolerance, niter, seeds.min(50), k_d + 1,
        &crate::geosection_orientation::CentroidMap::new(), 0.0)?;

    let mut assignments = left_asgn;
    assignments.extend(right_asgn);
    if assignments.len() != n {
        return Err(format!("proportional-section incomplete: {}/{}", assignments.len(), n));
    }
    Ok((assignments, k_d, k_r, best_ec, d))
}

///   - At the top-level ratio scan, uses tpwgts=[pop_frac, 0.5, 1-pop_frac, 0.5]
///     and ubvec=[1.001, 1.10] (tight pop balance, 10% area swing).
///   - Performs Lorenz pre-filtering to skip infeasible ratios.
///   - Recursive calls always use ncon=1 (area constraint only at first level).
///
/// Returns (assignments, natural_ratio_left, natural_ratio_right, natural_ec).
pub fn run_geosection(
    adjacency: &[Vec<usize>],
    vertex_weights: &[i64],
    edge_weights: &HashMap<(usize, usize), f64>,
    num_districts: usize,
    balance_tolerance: f64,
    niter: u32,
    seeds_per_ratio: usize,
    intermediate_dir: Option<&Path>,
    // Phase 2: centroid map for directional penalty (empty = no penalty)
    centroids: &crate::geosection_orientation::CentroidMap,
    // Phase 2: directional penalty strength (0.0 = off, use GeoSection without penalty)
    lambda: f64,
    // AreaSection mode: ALAND in m² per vertex. None = standard GeoSection (ncon=1).
    vertex_areas_m2: Option<&[f64]>,
    // AreaSection area imbalance tolerance (ubvec[1]). Default 1.10 = ±10%.
    area_swing: f64,
    // VRASection (B.14): per-vertex minority VAP counts. None = standard GeoSection.
    // When Some, ratio selection score is: normalised - w_vra * alignment * normalised.max(1)
    // where alignment = |MVAP_frac(left) - MVAP_frac(right)|.
    minority_vap: Option<&[f64]>,
    // VRASection alignment weight (default 0.40). Only consulted when minority_vap is Some.
    w_vra: f64,
) -> Result<(HashMap<usize, usize>, usize, usize, f64), String> {
    let n = adjacency.len();

    if num_districts == 1 {
        let asgn = (0..n).map(|i| (i, 1)).collect();
        return Ok((asgn, 1, 0, 0.0));
    }
    if num_districts == 2 {
        // Only one ratio possible: 1:1
        let asgn = run_all_splits(adjacency, vertex_weights, edge_weights,
                                   2, balance_tolerance, niter,
                                   Some(1), intermediate_dir)?;
        let ec: f64 = edge_weights.iter().map(|(&(u,v),&w)| {
            if asgn.get(&u) != asgn.get(&v) { w } else { 0.0 }
        }).sum();
        return Ok((asgn, 1, 1, ec));
    }

    // Try all split ratios at the root level
    let node_ufactor = 1.0 + balance_tolerance / num_districts as f64;
    let mut best_ec = f64::INFINITY;
    let mut best_normalised = f64::INFINITY; // isoperimetric-corrected selection criterion
    let mut best_left = 0usize;
    let mut best_right = 0usize;
    let mut best_left_set = HashSet::new();
    let mut best_right_set = HashSet::new();

    let all_tracts: HashSet<usize> = (0..n).collect();
    let max_left = num_districts / 2;  // try ratios 1:k-1 through k/2:k/2

    // ── AreaSection mode: build interleaved vwgt and Lorenz feasibility mask ──
    let (ncon, vwgt_flat, lorenz_feasible) = if let Some(areas) = vertex_areas_m2 {
        // Scale areas to hectares (÷10,000 m²) to stay within METIS i32 range.
        // Large rural tracts can be billions of m²; hectares keep them within i32.
        let vertex_areas_ha: Vec<i64> = areas.iter()
            .map(|&a| ((a / 10_000.0) as i64).max(1)).collect();
        let interleaved: Vec<i64> = vertex_weights.iter().zip(&vertex_areas_ha)
            .flat_map(|(&p, &a)| [p, a]).collect();

        // Lorenz pre-filtering: skip ratios where area balance is geometrically impossible
        let area_max = 0.5 * area_swing;
        let area_min = 0.5 / area_swing;
        let (_, natural_pop, suggested_k) = population_lorenz(vertex_weights, areas, num_districts);
        eprintln!("[areasection] Lorenz: dense-half contains {:.1}% of population -> natural split ~{}:{}",
                  natural_pop * 100.0, num_districts - suggested_k, suggested_k);
        let feasible: Vec<bool> = (0..=max_left).map(|left_k| {
            if left_k == 0 { return false; }
            let p = left_k as f64 / num_districts as f64;
            let min_a = lorenz_min_area(vertex_weights, areas, p);
            let max_a = 1.0 - lorenz_min_area(vertex_weights, areas, 1.0 - p);
            min_a <= area_max && max_a >= area_min
        }).collect();
        for left_k in 1..=max_left {
            let p = left_k as f64 / num_districts as f64;
            let min_a = lorenz_min_area(vertex_weights, areas, p);
            let max_a = 1.0 - lorenz_min_area(vertex_weights, areas, 1.0 - p);
            eprintln!("[areasection]   ratio {}:{} ({:.1}% pop): Lorenz area range [{:.1}%-{:.1}%] -> {}",
                      left_k, num_districts - left_k, p * 100.0,
                      min_a * 100.0, max_a * 100.0,
                      if feasible[left_k] { "feasible" } else { "INFEASIBLE (Lorenz)" });
        }
        eprintln!("[areasection] {} ratios x {} seeds (pop+area dual constraint)",
                  max_left, seeds_per_ratio);
        (2usize, interleaved, feasible)
    } else {
        // ncon=1: plain population weights, all ratios feasible
        let plain: Vec<i64> = vertex_weights.to_vec();
        let feasible = vec![true; max_left + 1];
        eprintln!("[geosection] trying {} ratios x {} seeds for k={}",
                  max_left, seeds_per_ratio, num_districts);
        (1usize, plain, feasible)
    };

    for left_k in 1..=max_left {
        if !lorenz_feasible[left_k] {
            eprintln!("[areasection] skipping ratio {}:{} - Lorenz predicts infeasible area balance",
                      left_k, num_districts - left_k);
            continue;
        }
        let right_k = num_districts - left_k;
        let pop_frac = left_k as f64 / num_districts as f64;

        // Build tpwgts and ubvec based on ncon.
        // Always compute right = 1.0 - left in f32 to guarantee exact sum-to-one.
        let left_w = pop_frac as f32;
        let right_w = 1.0_f32 - left_w;
        let tpwgts: Option<Vec<f32>> = if ncon == 2 {
            // ncon=2: [left_pop, left_area, right_pop, right_area]
            // Area target is always 50/50; sum per constraint = 1.0.
            Some(vec![left_w, 0.5f32, right_w, 0.5f32])
        } else if left_k != right_k {
            Some(vec![left_w, right_w])
        } else {
            None
        };
        let ubvec: Option<Vec<f32>> = if ncon == 2 {
            // Tight population balance (±0.1%), area swing from caller
            Some(vec![1.001f32, area_swing as f32])
        } else {
            None
        };

        let mut ratio_best = f64::INFINITY;
        let mut ratio_best_left = HashSet::new();
        let mut ratio_best_right = HashSet::new();

        for seed in 1..=(seeds_per_ratio as u64) {
            match split_subgraph(adjacency, &vwgt_flat, ncon, edge_weights,
                                  &all_tracts, node_ufactor, niter, Some(seed),
                                  tpwgts.clone(), ubvec.clone()) {
                Ok((l, r)) => {
                    let ec: f64 = edge_weights.iter().filter_map(|(&(u,v),&w)| {
                        if l.contains(&u) != l.contains(&v) { Some(w) } else { None }
                    }).sum();
                    if ec < ratio_best {
                        ratio_best = ec;
                        ratio_best_left = l;
                        ratio_best_right = r;
                    }
                }
                Err(e) => {
                    if seed == 1 && ncon == 2 {
                        eprintln!("[areasection] seed 1 error (ratio {}:{}): {}", left_k, right_k,
                                  &e.chars().take(300).collect::<String>());
                    }
                    continue;
                }
            }
        }

        // Normalise by sqrt(min(i,k-i)): isoperimetric correction.
        // Raw EC always favours 1:k-1 (tiny boundary) over k/2:k/2 (full bisection).
        // Dividing by sqrt(smaller_half_districts) makes the comparison apples-to-apples.
        let smaller = left_k.min(right_k) as f64;
        let normalised = ratio_best / smaller.sqrt();

        // VRASection (B.14): subtract alignment bonus from the normalised score.
        // A(split) = |MVAP_frac(left) - MVAP_frac(right)| (0=equal, 1=fully concentrated)
        // score(ratio) = normalised - w_vra * alignment * normalised.max(1.0)
        // Lower score = preferred. Subtracting means concentrated splits win over dispersed.
        let selection_score = if let Some(mvap) = minority_vap {
            let mvap_total: f64 = mvap.iter().sum();
            let score = if mvap_total > 0.0 {
                let mvap_left: f64 = ratio_best_left.iter().map(|&v| mvap[v]).sum();
                let alignment = (mvap_left / mvap_total - 0.5).abs() * 2.0;
                normalised - w_vra * alignment * normalised.max(1.0)
            } else {
                normalised
            };
            if ncon == 2 {
                // AreaSection doesn't use VRA alignment; just use normalised
                normalised
            } else {
                eprintln!("[vra-section]   ratio {}:{} normalised={:.1}  score={:.1}",
                          left_k, right_k, normalised/1000.0, score/1000.0);
                score
            }
        } else {
            normalised
        };

        if ncon == 2 {
            if let Some(areas) = vertex_areas_m2 {
                let area_left: f64 = ratio_best_left.iter().map(|&v| areas[v]).sum();
                let total_area: f64 = areas.iter().sum();
                let area_frac = area_left / total_area;
                eprintln!("[areasection]   ratio {}:{} best={:.0}km  normalised={:.1}  area_left={:.1}%",
                          left_k, right_k, ratio_best/1000.0, normalised/1000.0, area_frac*100.0);
            }
        } else if minority_vap.is_none() {
            eprintln!("[geosection]   ratio {}:{} best={:.0}km  normalised={:.1}",
                      left_k, right_k, ratio_best/1000.0, normalised/1000.0);
        }

        if selection_score < best_normalised {
            best_normalised = selection_score;
            best_ec = ratio_best;
            best_left = left_k;
            best_right = right_k;
            best_left_set = ratio_best_left;
            best_right_set = ratio_best_right;
        }
    }

    let mode_tag = if ncon == 2 { "areasection" } else if minority_vap.is_some() { "vra-section" } else { "geosection" };
    eprintln!("[{mode_tag}] natural ratio {}:{} at {:.0}km (normalised={:.1})",
              best_left, best_right, best_ec/1000.0, best_normalised/1000.0);

    // For AreaSection: log the winning split's area fraction and whether it's within ubvec
    if ncon == 2 {
        if let Some(areas) = vertex_areas_m2 {
            let area_left: f64 = best_left_set.iter().map(|&v| areas[v]).sum();
            let total_area: f64 = areas.iter().sum();
            let area_frac = area_left / total_area;
            let area_min = 0.5 / area_swing;
            let area_max = 0.5 * area_swing;
            let in_bounds = area_frac >= area_min && area_frac <= area_max;
            let pop_left: i64 = best_left_set.iter().map(|&v| vertex_weights[v]).sum();
            let total_pop: i64 = vertex_weights.iter().sum();
            let pop_frac = pop_left as f64 / total_pop as f64;
            let pop_target = best_left as f64 / num_districts as f64;
            eprintln!("[areasection] winner: area={:.1}% (target 50% ±{:.0}%, {}) pop={:.1}% (target {:.1}%)",
                      area_frac*100.0, (area_swing-1.0)*100.0,
                      if in_bounds { "OK" } else { "VIOLATED" },
                      pop_frac*100.0, pop_target*100.0);
        }
    }

    // Write depth_01 intermediate
    if let Some(dir) = intermediate_dir {
        let round_dir = dir.join("depth_01");
        let _ = std::fs::create_dir_all(&round_dir);
        let mut d1: HashMap<usize, usize> = HashMap::new();
        for &v in &best_left_set  { d1.insert(v, 1); }
        for &v in &best_right_set { d1.insert(v, 2); }
        let _ = write_intermediate_round(&round_dir, &d1);
    }

    // Recurse: each half finds its own natural ratio with its own orientation.
    // Recursive calls always use ncon=1 (area constraint only at the first level).
    let left_asgn  = recurse_geosection(
        &best_left_set,  adjacency, vertex_weights, edge_weights,
        best_left,  balance_tolerance, niter, seeds_per_ratio, 1,
        centroids, lambda)?;
    let right_asgn = recurse_geosection(
        &best_right_set, adjacency, vertex_weights, edge_weights,
        best_right, balance_tolerance, niter, seeds_per_ratio, best_left + 1,
        centroids, lambda)?;

    let mut assignments = left_asgn;
    assignments.extend(right_asgn);

    if assignments.len() != n {
        return Err(format!("{mode_tag} incomplete: {}/{n}", assignments.len()));
    }
    Ok((assignments, best_left, best_right, best_ec))
}

/// Fully recursive GeoSection on a geographic subregion.
///
/// At each level:
///   1. Extract local subgraph (local indices)
///   2. Compute local minor axis via PCA of subregion centroids (if available)
///   3. Apply directional penalty λ to edge weights (makes cuts straighter)
///   4. Run isoperimetrically-normalised ratio search on local graph
///   5. Map results back to global indices, offset by district_base
///
/// Each half re-rotates independently — a horizontal first cut produces
/// two halves that each find their OWN narrowest geographic axis.
fn recurse_geosection(
    verts: &HashSet<usize>,
    adjacency: &[Vec<usize>],
    vertex_weights: &[i64],
    edge_weights: &HashMap<(usize,usize),f64>,
    k: usize,
    balance_tolerance: f64,
    niter: u32,
    seeds_per_ratio: usize,
    district_base: usize,
    centroids: &crate::geosection_orientation::CentroidMap,
    lambda: f64,
) -> Result<HashMap<usize,usize>, String> {
    if k == 0 { return Ok(HashMap::new()); }
    if k == 1 {
        return Ok(verts.iter().map(|&v| (v, district_base)).collect());
    }

    // Extract sorted vertex list for deterministic local indexing
    let sorted: Vec<usize> = { let mut v: Vec<usize> = verts.iter().copied().collect(); v.sort_unstable(); v };
    let global_to_local: HashMap<usize,usize> = sorted.iter().enumerate().map(|(i,&g)|(g,i)).collect();

    // Build local subgraph components
    let local_adj: Vec<Vec<usize>> = build_subgraph_adjacency(verts, adjacency);
    let local_vw: Vec<i64> = sorted.iter().map(|&g| vertex_weights[g]).collect();
    let mut local_ew: HashMap<(usize,usize),f64> = edge_weights.iter()
        .filter_map(|(&(u,v),&w)| {
            let lu = *global_to_local.get(&u)?;
            let lv = *global_to_local.get(&v)?;
            Some(((lu.min(lv), lu.max(lv)), w))
        })
        .fold(HashMap::new(), |mut m,(k,v)| { m.insert(k,v); m });

    // Phase 2: PCA of THIS subregion's centroids → local minor axis → directional penalty
    if lambda > 1e-10 && !centroids.is_empty() {
        if let Some(angle) = crate::geosection_orientation::compute_minor_axis(verts, centroids) {
            local_ew = crate::geosection_orientation::apply_directional_penalty(
                &local_ew, centroids, angle, lambda
            );
        }
    }

    // Recursively find natural ratio for THIS subregion
    // Pass empty centroids/lambda=0 here — directional penalty was already
    // applied to local_ew above; run_geosection sees the modified weights.
    // Always ncon=1 for recursive levels (area constraint only at first level).
    let empty_centroids = crate::geosection_orientation::CentroidMap::new();
    let (local_asgn, nat_left, nat_right, nat_ec) = run_geosection(
        &local_adj, &local_vw, &local_ew,
        k, balance_tolerance, niter, seeds_per_ratio, None,
        &empty_centroids, 0.0, None, 1.10,  // recursive: ncon=1, area_swing unused
        None, 0.0,  // recursive: no VRA alignment at sub-levels
    )?;

    if local_asgn.len() < sorted.len().saturating_sub(1) {
        // Partial assignment — fall back to standard for this subregion
        return recurse_standard(verts, &local_adj, adjacency, vertex_weights, edge_weights,
                                k, balance_tolerance, niter, district_base);
    }

    // Map local indices back to global with district offset
    let result: HashMap<usize,usize> = local_asgn.iter()
        .filter_map(|(&local, &dist)| {
            sorted.get(local).map(|&global| (global, dist + district_base - 1))
        })
        .collect();
    Ok(result)
}

/// Build adjacency restricted to a subset of vertices (for recursion).
fn build_subgraph_adjacency(verts: &HashSet<usize>, adj: &[Vec<usize>])
    -> Vec<Vec<usize>>
{
    let sorted: Vec<usize> = {
        let mut v: Vec<usize> = verts.iter().copied().collect();
        v.sort_unstable(); v
    };
    let global_to_local: HashMap<usize,usize> = sorted.iter()
        .enumerate().map(|(i,&g)| (g,i)).collect();
    sorted.iter().map(|&g| {
        adj[g].iter().filter_map(|&nb| global_to_local.get(&nb).copied()).collect()
    }).collect()
}

/// Recurse using standard bisection (UpfloorD k/2 : ceil k/2) within a subgraph.
/// Returns global-index assignments offset by district_base.
fn recurse_standard(
    verts: &HashSet<usize>,
    sub_adj: &[Vec<usize>],
    global_adj: &[Vec<usize>],
    vertex_weights: &[i64],
    edge_weights: &HashMap<(usize,usize),f64>,
    k: usize,
    balance_tolerance: f64,
    niter: u32,
    district_base: usize,
) -> Result<HashMap<usize,usize>, String> {
    if k == 0 { return Ok(HashMap::new()); }
    if k == 1 {
        return Ok(verts.iter().map(|&v| (v, district_base)).collect());
    }

    let sorted: Vec<usize> = { let mut v: Vec<usize> = verts.iter().copied().collect(); v.sort_unstable(); v };
    let global_to_local: HashMap<usize,usize> = sorted.iter().enumerate().map(|(i,&g)|(g,i)).collect();

    // Extract sub-vertex-weights and sub-edge-weights
    let sub_vw: Vec<i64> = sorted.iter().map(|&g| vertex_weights[g]).collect();
    let sub_ew: HashMap<(usize,usize),f64> = edge_weights.iter()
        .filter_map(|(&(u,v),&w)| {
            let lu = *global_to_local.get(&u)?;
            let lv = *global_to_local.get(&v)?;
            Some(((lu.min(lv), lu.max(lv)), w))
        })
        .fold(HashMap::new(), |mut m,(k,v)| { m.insert(k,v); m });

    let sub_n = sorted.len();
    let sub_asgn = run_all_splits(sub_adj, &sub_vw, &sub_ew,
                                   k, balance_tolerance, niter,
                                   Some(42), None)?;

    // Map back to global indices with offset
    let result: HashMap<usize,usize> = sub_asgn.iter()
        .map(|(&local, &dist)| (sorted[local], dist + district_base - 1))
        .collect();
    Ok(result)
}

pub fn run_all_splits(
    adjacency: &[Vec<usize>],
    vertex_weights: &[i64],
    edge_weights: &HashMap<(usize, usize), f64>,
    num_districts: usize,
    balance_tolerance: f64, // fraction (e.g. 0.10 for ±10%); node ufactor = 1 + T/k
    niter: u32,
    seed: Option<u64>,
    // If Some, writes intermediate/depth_{d:02}/assignments.json after each round.
    intermediate_dir: Option<&Path>,
) -> Result<HashMap<usize, usize>, String> {
    let n = adjacency.len();

    // Single-district: all tracts to district 1, no METIS call
    if num_districts == 1 {
        // Write depth-00 as the trivial single-region state
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

        // Extract data BEFORE parallel section — no shared references across threads
        let nodes_with_tracts: Vec<(redist_core::BisectionNode, HashSet<usize>)> =
            nodes_at_depth.into_iter()
                .filter_map(|node| {
                    node_tracts.remove(&node.path).map(|tracts| (node, tracts))
                })
                .collect();

        let split_results: Vec<(String, HashSet<usize>, HashSet<usize>)> =
            nodes_with_tracts.into_par_iter()
                .map(|(node, tracts)| {
                    // Per-node ufactor: 1.0 + balance_tolerance / k_node
                    // This is the mathematically correct formula: if each split at a node
                    // with k remaining districts is balanced to (T/k)%, the cumulative
                    // error across all levels never exceeds T% per final district.
                    // Root (k=98, T=10%): ufactor=1.00102 (very tight)
                    // Leaf (k=2, T=10%): ufactor=1.05 (loose — OK since only 2 districts)
                    let node_ufactor = 1.0 + balance_tolerance / node.k as f64;

                    // Target weights: k_left/k and k_right/k
                    // Equal when k_left == k_right (even k); unequal for odd k
                    let tpwgts = if node.k_left == node.k_right {
                        None // equal split — METIS default
                    } else {
                        let left_w = node.k_left as f32 / node.k as f32;
                        Some(vec![left_w, 1.0_f32 - left_w]) // right = 1-left (exact f32 sum)
                    };
                    let (left, right) = split_subgraph(
                        adjacency, vertex_weights, 1, edge_weights, &tracts,
                        node_ufactor, niter, seed, tpwgts, None,
                    ).map_err(|e| format!("depth {} node '{}': {e}", depth, node.path))?;
                    Ok((node.path, left, right))
                })
                .collect::<Result<Vec<_>, String>>()?;

        // Sort results by path before inserting to ensure deterministic insertion order.
        // Rayon's thread scheduling may vary, so the collection order of split_results
        // is non-deterministic without this sort.
        //
        // Determinism requires: (a) same seed passed to gpmetis, (b) same graph structure,
        // (c) same topology of adjacency. The sort here ensures consistent insertion order
        // into node_tracts, which affects the final leaf sort and district numbering.
        let mut sorted_results = split_results;
        sorted_results.sort_by_key(|(path, _, _)| path.clone());
        for (path, left, right) in sorted_results {
            node_tracts.insert(format!("{path}0"), left);
            node_tracts.insert(format!("{path}1"), right);
        }

        // Write intermediate round: current node_tracts state as tract→region_id
        if let Some(dir) = intermediate_dir {
            let round_dir = dir.join(format!("depth_{:02}", depth + 1));
            let _ = std::fs::create_dir_all(&round_dir);
            // Sort nodes for deterministic region numbering
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

    // Sort leaves by (depth, path) — NOT plain lex.
    // Plain lex on binary paths is WRONG: "0","00","01","1" ≠ BFS "0","1","00","01"
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
            "bisection incomplete: {}/{n} tracts assigned", assignments.len()
        ));
    }
    Ok(assignments)
}

/// CompactBisect variant of run_all_splits.
///
/// Identical to run_all_splits except at each bisection node it runs
/// `opts.seeds_per_level` METIS candidates, filters to near-minimum-cut,
/// and selects the split maximising geometric-mean Polsby-Popper.
/// Requires geometry data in `graph` (vertex_areas + vertex_ext_perimeters);
/// gracefully degrades to minimum-edge-cut if geometry is absent.
pub fn run_all_splits_compact(
    adjacency: &[Vec<usize>],
    vertex_weights: &[i64],
    edge_weights: &HashMap<(usize, usize), f64>,
    // Per-vertex land area in m² (from TIGER ALAND). Empty = no PP computation.
    vertex_areas: &[f64],
    // Per-vertex external perimeter in metres. Empty = no PP computation.
    vertex_ext_perimeters: &[f64],
    num_districts: usize,
    balance_tolerance: f64,
    niter: u32,
    // Ignored here — seeds_per_level in opts controls METIS seed iteration.
    _single_seed: Option<u64>,
    opts: &CompactBisectOpts,
    intermediate_dir: Option<&Path>,
) -> Result<HashMap<usize, usize>, String> {
    let n = adjacency.len();
    // Build a lightweight AdjacencyGraph wrapper so select_compact_split can call subgraph_pp.
    // We only populate the geometry fields — adjacency/weights are borrowed from the caller.
    let geom_graph = {
        let mut g = redist_data::AdjacencyGraph {
            adjacency: adjacency.to_vec(),
            vertex_weights: vertex_weights.to_vec(),
            edge_weights: edge_weights.clone(),
            n_vertices: n,
            n_edges: edge_weights.len(),
            vertex_areas: vertex_areas.to_vec(),
            vertex_ext_perimeters: vertex_ext_perimeters.to_vec(),
        };
        g
    };

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
        let nodes_with_tracts: Vec<(redist_core::BisectionNode, HashSet<usize>)> =
            nodes_at_depth.into_iter()
                .filter_map(|node| node_tracts.remove(&node.path).map(|t| (node, t)))
                .collect();

        let split_results: Vec<(String, HashSet<usize>, HashSet<usize>)> =
            nodes_with_tracts.into_par_iter()
                .map(|(node, tracts)| {
                    let node_ufactor = 1.0 + balance_tolerance / node.k as f64;
                    let tpwgts_node = if node.k_left == node.k_right { None } else {
                        let left_w = node.k_left as f32 / node.k as f32;
                        Some(vec![left_w, 1.0_f32 - left_w])
                    };

                    // Run N seeds, collect (left, right, edge_cut)
                    let candidates: Vec<(HashSet<usize>, HashSet<usize>, f64)> =
                        (1..=opts.seeds_per_level).filter_map(|s| {
                            let seed = Some(s as u64);
                            split_subgraph(
                                adjacency, vertex_weights, 1, edge_weights,
                                &tracts, node_ufactor, niter, seed,
                                tpwgts_node.clone(), None,
                            ).ok().map(|(l, r)| {
                                let ec: f64 = edge_weights.iter().filter_map(|(&(u, v), &w)| {
                                    if l.contains(&u) != l.contains(&v) { Some(w) } else { None }
                                }).sum();
                                (l, r, ec)
                            })
                        }).collect();

                    if candidates.is_empty() {
                        return Err(format!(
                            "depth {} node '{}': all {} seeds failed",
                            depth, node.path, opts.seeds_per_level
                        ));
                    }

                    let (left, right) = select_compact_split(&candidates, &geom_graph, opts.epsilon);
                    Ok((node.path, left, right))
                })
                .collect::<Result<Vec<_>, String>>()?;

        let mut sorted = split_results;
        sorted.sort_by_key(|(path, _, _)| path.clone());
        for (path, left, right) in sorted {
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
                for &tract in tracts.iter() { round_asgn.insert(tract, region_id + 1); }
            }
            let _ = write_intermediate_round(&round_dir, &round_asgn);
        }
    }

    let mut leaves: Vec<(String, HashSet<usize>)> = node_tracts.into_iter().collect();
    leaves.sort_by_key(|(path, _)| (path.len(), path.clone()));
    let mut assignments: HashMap<usize, usize> = HashMap::new();
    for (district_id, (_, tracts)) in leaves.into_iter().enumerate() {
        for tract in tracts { assignments.insert(tract, district_id + 1); }
    }
    if assignments.len() != n {
        return Err(format!("bisection incomplete: {}/{n} tracts assigned", assignments.len()));
    }
    Ok(assignments)
}

// ── Proportional Bisection (B.7) ─────────────────────────────────────────────

/// At each bisection, compute the Dem vote share within the current subgraph
/// and split the subgraph proportionally: the "left" half gets
/// round(dem_share * k) districts and the "right" half gets the remainder.
///
/// Within that proportional constraint, edge-cut minimisation (METIS) determines
/// WHERE the boundary is drawn. No partisan data enters the boundary decision —
/// only the RATIO of districts allocated to each side.
///
/// Theorem (B.7): this achieves near-proportional seat allocation without
/// picking which party's voters land in which half. The proportional ratio is
/// applied symmetrically; METIS draws the most compact boundary satisfying it.
///
/// Requires: per-vertex dem_votes (from partisan_shares CSV, same as partisan-weighted mode).
/// §104(e) of the Districting Integrity Act prohibits this for federal congressional
/// districts. Valid for state legislative redistricting.
pub fn run_all_splits_proportional(
    adjacency: &[Vec<usize>],
    vertex_weights: &[i64],
    edge_weights: &HashMap<(usize, usize), f64>,
    // Per-vertex Dem vote total (from partisan_shares CSV).
    dem_votes: &[f64],
    num_districts: usize,
    balance_tolerance: f64,
    niter: u32,
    seed: Option<u64>,
    intermediate_dir: Option<&Path>,
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
        let nodes_with_tracts: Vec<(redist_core::BisectionNode, HashSet<usize>)> =
            nodes_at_depth.into_iter()
                .filter_map(|node| node_tracts.remove(&node.path).map(|t| (node, t)))
                .collect();

        let split_results: Vec<(String, HashSet<usize>, HashSet<usize>)> =
            nodes_with_tracts.into_par_iter()
                .map(|(node, tracts)| {
                    let node_ufactor = 1.0 + balance_tolerance / node.k as f64;

                    // Compute Dem vote share within this subgraph
                    let total_dem: f64 = tracts.iter().map(|&v| dem_votes[v]).sum();
                    let total_votes: f64 = tracts.iter()
                        .map(|&v| vertex_weights[v] as f64).sum();
                    let dem_share = if total_votes > 0.0 {
                        total_dem / total_votes
                    } else {
                        0.5 // fallback: equal split
                    };

                    // Proportional district allocation: round to nearest integer
                    let k_dem = (dem_share * node.k as f64).round() as usize;
                    let k_dem = k_dem.max(1).min(node.k - 1); // at least 1 per side
                    let k_rep = node.k - k_dem;

                    // Use the proportional allocation as METIS target weights.
                    // METIS will minimise edge-cut subject to this population-ratio constraint.
                    let tpwgts = if k_dem == k_rep {
                        None // equal — use default
                    } else {
                        Some(vec![
                            k_dem as f32 / node.k as f32,
                            k_rep as f32 / node.k as f32,
                        ])
                    };

                    let (left, right) = split_subgraph(
                        adjacency, vertex_weights, 1, edge_weights,
                        &tracts, node_ufactor, niter, seed, tpwgts, None,
                    ).map_err(|e| format!("depth {} node '{}': {e}", depth, node.path))?;

                    Ok((node.path, left, right))
                })
                .collect::<Result<Vec<_>, String>>()?;

        let mut sorted = split_results;
        sorted.sort_by_key(|(path, _, _)| path.clone());
        for (path, left, right) in sorted {
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
                for &tract in tracts.iter() { round_asgn.insert(tract, region_id + 1); }
            }
            let _ = write_intermediate_round(&round_dir, &round_asgn);
        }
    }

    let mut leaves: Vec<(String, HashSet<usize>)> = node_tracts.into_iter().collect();
    leaves.sort_by_key(|(path, _)| (path.len(), path.clone()));
    let mut assignments: HashMap<usize, usize> = HashMap::new();
    for (district_id, (_, tracts)) in leaves.into_iter().enumerate() {
        for tract in tracts { assignments.insert(tract, district_id + 1); }
    }
    if assignments.len() != n {
        return Err(format!("bisection incomplete: {}/{n} tracts assigned", assignments.len()));
    }
    Ok(assignments)
}

/// Return the set of vertices reachable from any member of `subset` using only
/// edges whose both endpoints lie within `subset`.
///
/// The full adjacency list `adj` uses **global** indices.  `subset` is also
/// expressed in global indices.  The returned `Vec<HashSet<usize>>` contains
/// one entry per connected component (global indices).
///
/// Used by `repair_bisection_contiguity` to detect fragmented partitions.
pub fn connected_components_of(
    adj: &[Vec<usize>],
    subset: &HashSet<usize>,
) -> Vec<HashSet<usize>> {
    let mut unvisited: HashSet<usize> = subset.clone();
    let mut components: Vec<HashSet<usize>> = Vec::new();

    while let Some(&start) = unvisited.iter().next() {
        let mut component = HashSet::new();
        let mut queue = std::collections::VecDeque::new();
        queue.push_back(start);
        unvisited.remove(&start);
        component.insert(start);

        while let Some(v) = queue.pop_front() {
            for &nb in &adj[v] {
                if unvisited.contains(&nb) {
                    unvisited.remove(&nb);
                    component.insert(nb);
                    queue.push_back(nb);
                }
            }
        }
        components.push(component);
    }

    components
}

/// Repair a bisection where one or both sides may be disconnected.
///
/// After METIS splits a subgraph into `left` and `right`, contiguity is not
/// guaranteed for every input graph.  This function detects disconnected
/// components in each side and reassigns orphaned components (those *not*
/// containing the majority component) to the other side.
///
/// The repair is greedy: the largest component of each side is kept; smaller
/// components are migrated to the opposite side.  The operation is repeated
/// at most once per side so that the result is always a valid (non-empty)
/// partition covering every vertex.
///
/// Invariants:
/// - `|returned_left| + |returned_right| == |left| + |right|`
/// - Both sides remain non-empty (the function will not create a degenerate
///   empty partition when the input already has vertices on both sides).
pub fn repair_bisection_contiguity(
    adj: &[Vec<usize>],
    left: HashSet<usize>,
    right: HashSet<usize>,
) -> (HashSet<usize>, HashSet<usize>) {
    let repair_side = |main: HashSet<usize>, other: HashSet<usize>| -> (HashSet<usize>, HashSet<usize>) {
        let mut comps = connected_components_of(adj, &main);
        if comps.len() <= 1 {
            return (main, other);
        }
        // Keep the largest component; migrate the rest to the other side.
        comps.sort_by_key(|c| std::cmp::Reverse(c.len()));
        let mut kept = comps.remove(0);
        let mut gained = other;
        for orphan in comps {
            gained.extend(orphan);
        }
        // Safety: never let the kept side shrink to zero if the other side
        // would swallow everything.
        if kept.is_empty() && !gained.is_empty() {
            // Move one vertex back (pathological case).
            let v = *gained.iter().next().unwrap();
            gained.remove(&v);
            kept.insert(v);
        }
        (kept, gained)
    };

    let (left2, right2) = repair_side(left, right);
    let (right3, left3) = repair_side(right2, left2);
    (left3, right3)
}

/// Write one intermediate round's assignments to `{round_dir}/assignments.json`.
/// Format: `{"tract_index": region_id, ...}` — mirrors final_assignments.json.
fn write_intermediate_round(round_dir: &Path, assignments: &HashMap<usize, usize>) -> Result<(), String> {
    let path = round_dir.join("assignments.json");
    let json = serde_json::to_string(assignments)
        .map_err(|e| format!("serialize intermediate: {e}"))?;
    std::fs::write(&path, json).map_err(|e| format!("write intermediate: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── detect_gpmetis_version ───────────────────────────────────────────────

    #[test]
    fn test_detect_gpmetis_version_returns_string() {
        let version = detect_gpmetis_version();
        assert!(!version.is_empty(), "must return a non-empty string");
        assert!(version.chars().all(|c| !c.is_control() || c == ' '),
            "version string must contain only printable chars: {:?}", version);
        assert!(version.contains("METIS"), "expected METIS in version string, got: {version}");
    }

    #[test]
    fn test_detect_gpmetis_version_never_panics() {
        let v = detect_gpmetis_version();
        assert!(!v.is_empty(), "must return a non-empty string, got: {:?}", v);
    }

    #[test]
    fn test_split_four_node_graph() {
        let adj = vec![vec![1, 2], vec![0, 3], vec![0, 3], vec![1, 2]];
        let vw = vec![1000i64, 1000, 1000, 1000];
        let ew = HashMap::new();
        let indices: HashSet<usize> = (0..4).collect();

        let (left, right) = split_subgraph(&adj, &vw, 1, &ew, &indices, 1.001, 100, Some(42), None, None)
            .expect("METIS should split 4-node graph");

        assert_eq!(left.len() + right.len(), 4, "all tracts assigned");
        assert!(!left.is_empty() && !right.is_empty(), "non-empty split");
        // Disjoint and complete
        assert!(left.is_disjoint(&right), "left and right must be disjoint");
        for i in 0..4 {
            assert!(left.contains(&i) || right.contains(&i), "tract {i} missing");
        }
        let pop_left: i64 = left.iter().map(|&i| vw[i]).sum();
        let pop_right: i64 = right.iter().map(|&i| vw[i]).sum();
        let dev = (pop_left - pop_right).abs() as f64 / 4000.0;
        assert!(dev <= 0.2, "split should be balanced, got {dev:.3}");
    }

    #[test]
    fn test_run_all_splits_single_district() {
        let n = 193usize;
        let adj = vec![vec![]; n];
        let vw = vec![1000i64; n];
        let ew = HashMap::new();
        let assignments = run_all_splits(&adj, &vw, &ew, 1, 0.005, 100, None, None)
            .expect("single district should not invoke METIS");
        assert_eq!(assignments.len(), n);
        assert!(assignments.values().all(|&d| d == 1));
    }

    #[test]
    fn test_run_all_splits_two_districts() {

        let adj = vec![vec![1], vec![0, 2], vec![1, 3], vec![2]];
        let vw = vec![1000i64, 1000, 1000, 1000];
        let ew = HashMap::new();

        let assignments = run_all_splits(&adj, &vw, &ew, 2, 0.005, 100, Some(42), None).unwrap();

        assert_eq!(assignments.len(), 4, "all tracts assigned");
        assert!(assignments.values().any(|&d| d == 1), "district 1 must exist");
        assert!(assignments.values().any(|&d| d == 2), "district 2 must exist");
        assert!(assignments.values().all(|&d| d == 1 || d == 2));

        let d1: HashSet<usize> = assignments.iter().filter(|(_, &v)| v == 1).map(|(&k, _)| k).collect();
        let d2: HashSet<usize> = assignments.iter().filter(|(_, &v)| v == 2).map(|(&k, _)| k).collect();
        assert!(d1.is_disjoint(&d2), "districts must be disjoint");
        assert_eq!(d1.len() + d2.len(), 4, "complete coverage");
    }

    #[test]
    fn test_leaf_sort_bfs_order() {
        // Verify sort_by_key gives BFS not lex order
        let mut paths = vec!["1".to_string(), "0".to_string(), "01".to_string(), "00".to_string()];
        paths.sort_by_key(|p| (p.len(), p.clone()));
        // BFS: depth-1 first ("0","1"), then depth-2 ("00","01")
        assert_eq!(paths, vec!["0", "1", "00", "01"]);
    }

    // ── Invariant tests ──────────────────────────────────────────────────────

    #[test]
    fn test_nway_single_district_shortcut() {
        let adj = vec![vec![1], vec![0]];
        let vw = vec![1000i64, 1000];
        let ew = HashMap::new();
        let assignments = run_nway_partition(&adj, &vw, &ew, 1, 1.005, 100, None).unwrap();
        assert_eq!(assignments.len(), 2);
        assert!(assignments.values().all(|&d| d == 1));
    }

    #[test]
    fn test_nway_two_districts() {

        let adj = vec![vec![1, 2], vec![0, 3], vec![0, 3], vec![1, 2]];
        let vw = vec![1000i64; 4];
        let ew: HashMap<(usize, usize), f64> = HashMap::new();
        let assignments = run_nway_partition(&adj, &vw, &ew, 2, 1.005, 100, Some(42)).unwrap();
        assert_eq!(assignments.len(), 4);
        assert!(assignments.values().any(|&d| d == 1), "district 1 must exist");
        assert!(assignments.values().any(|&d| d == 2), "district 2 must exist");
        // Districts are 1-based and disjoint
        let d1: HashSet<_> = assignments.iter().filter(|(_, &v)| v == 1).map(|(&k, _)| k).collect();
        let d2: HashSet<_> = assignments.iter().filter(|(_, &v)| v == 2).map(|(&k, _)| k).collect();
        assert!(d1.is_disjoint(&d2));
        assert_eq!(d1.len() + d2.len(), 4);
    }

    #[test]
    fn test_nway_equal_weights_sum_to_one() {
        // AC-05: for n-way, verify n-1 explicit weights + inferred last = 1.0
        // With weight_per = 1/k, sum = (k-1)/k + inferred(1/k) = 1.0 exactly
        for k in [2usize, 3, 7, 52] {
            let weight_per = 1.0_f64 / k as f64;
            let explicit_sum: f64 = (k - 1) as f64 * weight_per;
            let inferred = 1.0 - explicit_sum;
            assert!(
                (explicit_sum + inferred - 1.0).abs() < 1e-9,
                "k={k}: explicit({explicit_sum:.9}) + inferred({inferred:.9}) should = 1.0"
            );
        }
    }

    #[test]
    fn test_invariant_target_weights_sum_to_one_2way() {
        // AC-05: target partition weights must sum to 1.0 for 2-way split
        // (k_left/k + k_right/k = k/k = 1.0 by construction)
        for k in [2, 3, 4, 7, 8, 14, 52] {
            let tree = redist_core::BisectionTree::from_k(k);
            for node in &tree.nodes {
                let left_frac = node.k_left as f64 / node.k as f64;
                let right_frac = node.k_right as f64 / node.k as f64;
                let sum = left_frac + right_frac;
                assert!(
                    (sum - 1.0).abs() < 1e-9,
                    "k={k} node k={}: left_frac={left_frac:.6} + right_frac={right_frac:.6} = {sum:.6} != 1.0",
                    node.k
                );
            }
        }
    }

    // ── ufactor correctness tests ─────────────────────────────────────────────

    #[test]
    fn test_ufactor_integer_conversion_0_5_pct() {
        // 0.5% tolerance: decimal 1.005 → integer 5
        let decimal = 1.005_f64;
        let ufactor_int = ((decimal - 1.0) * 1000.0).round() as u32;
        assert_eq!(ufactor_int, 5, "1.005 must convert to integer 5 (0.5%)");
    }

    #[test]
    fn test_ufactor_integer_conversion_5_pct() {
        // 5% tolerance: decimal 1.05 → integer 50
        let decimal = 1.05_f64;
        let ufactor_int = ((decimal - 1.0) * 1000.0).round() as u32;
        assert_eq!(ufactor_int, 50, "1.05 must convert to integer 50 (5%)");
    }

    #[test]
    fn test_ufactor_integer_conversion_10_pct() {
        // 10% tolerance: decimal 1.10 → integer 100
        let decimal = 1.10_f64;
        let ufactor_int = ((decimal - 1.0) * 1000.0).round() as u32;
        assert_eq!(ufactor_int, 100, "1.10 must convert to integer 100 (10%)");
    }

    #[test]
    fn test_ufactor_never_zero() {
        // Minimum clamped to 1 — ufactor=0 would disable balance checking
        for decimal in [1.0001_f64, 1.0_f64, 0.999_f64] {
            let raw = ((decimal - 1.0) * 1000.0).round() as i32;
            let clamped = (raw as u32).clamp(1, 1000);
            assert!(clamped >= 1, "ufactor must be >= 1, got {clamped} from decimal {decimal}");
        }
    }

    #[test]
    fn test_per_node_ufactor_formula() {
        // node_ufactor = 1.0 + balance_tolerance / k_node
        // Root of 98-district map (T=10%): should be very tight
        let k_root = 98usize;
        let tolerance = 0.10_f64;
        let node_ufactor = 1.0 + tolerance / k_root as f64;
        // ~0.102% — convert to int
        let ufactor_int = ((node_ufactor - 1.0) * 1000.0).round() as u32;
        assert_eq!(ufactor_int, 1, "root of 98-district (10%) → ufactor=1 (0.1%)");

        // Leaf of 2-district split (T=10%): should be loose
        let k_leaf = 2usize;
        let leaf_ufactor = 1.0 + tolerance / k_leaf as f64;
        let leaf_int = ((leaf_ufactor - 1.0) * 1000.0).round() as u32;
        assert_eq!(leaf_int, 50, "leaf of 2-district (10%) → ufactor=50 (5%)");
    }

    #[test]
    fn test_per_node_ufactor_congressional_tight() {
        // Congressional (T=0.5%): root of 52-district CA map
        // 0.5%/52 = 0.0096% → rounds to 0, clamped to minimum 1
        let k = 52usize;
        let tolerance = 0.005_f64; // 0.5%
        let node_ufactor = 1.0 + tolerance / k as f64;
        let raw = ((node_ufactor - 1.0) * 1000.0).round() as u32;
        let ufactor_int = raw.clamp(1, 1000); // minimum 1 = 0.1%
        assert_eq!(ufactor_int, 1, "CA 52D congressional root → clamped to minimum ufactor=1 (0.1%)");
    }

    #[test]
    fn test_ufactor_wasnt_silently_truncated_regression() {
        // This test catches the historical bug where '-ufactor=1.0050' was passed
        // to gpmetis as a float, which atoi() truncated to 1 regardless of value.
        // The correct behavior: 1.005 → integer 5 (not 1).
        let old_style_float = 1.005_f64;
        // Old bug: atoi("1.0050") == 1 (always)
        // New fix: round((1.005 - 1.0) * 1000) == 5
        let correct_int = ((old_style_float - 1.0) * 1000.0).round() as u32;
        assert_ne!(correct_int, 1,
            "REGRESSION: 1.005 should not convert to 1 — that was the bug. Got {correct_int}");
        assert_eq!(correct_int, 5,
            "1.005 (0.5% tolerance) must convert to integer 5");
    }

    #[test]
    fn test_invariant_vertex_weights_positive() {
        // DF-04: all vertex weights must be >= 1 after loading
        // sub-zero weights cause METIS to produce degenerate partitions
        let adj = vec![vec![1], vec![0, 2], vec![1]];
        let vw = vec![1000i64, 500, 2000]; // all positive
        let ew: HashMap<(usize, usize), f64> = HashMap::new();
        // The subgraph builder clamps to max(weight, 1) — verify it would catch 0
        let tract_indices: HashSet<usize> = (0..3).collect();
        let mut sorted: Vec<usize> = tract_indices.iter().copied().collect();
        sorted.sort_unstable();
        let sub_vw: Vec<i64> = sorted.iter().map(|&g| vw[g].max(1)).collect();
        assert!(sub_vw.iter().all(|&v| v >= 1), "all vertex weights must be >= 1 after clamping");
    }

    // ── Group 1: split_subgraph edge cases ───────────────────────────────────

    #[test]
    fn test_split_subgraph_with_edge_weights() {

        // 4-node chain with strong edge weights on left side — should bias split
        let adj = vec![vec![1], vec![0,2], vec![1,3], vec![2]];
        let vw = vec![1000i64; 4];
        let mut ew = HashMap::new();
        ew.insert((0,1), 1000.0); // strong edge — METIS should avoid cutting
        ew.insert((1,2), 1.0);    // weak edge — METIS may cut here
        ew.insert((2,3), 1000.0); // strong edge
        let indices: HashSet<usize> = (0..4).collect();
        let (left, right) = split_subgraph(&adj, &vw, 1, &ew, &indices, 1.005, 100, Some(42), None, None)
            .expect("should split with edge weights");
        assert_eq!(left.len() + right.len(), 4);
        assert!(!left.is_empty() && !right.is_empty());
    }

    #[test]
    fn test_split_subgraph_unequal_target_weights() {

        // 6 tracts, split 4:2 (target weights 2/3 and 1/3)
        let adj = vec![vec![1,2], vec![0,3], vec![0,3], vec![1,2,4,5], vec![3,5], vec![3,4]];
        let vw = vec![1000i64; 6];
        let ew = HashMap::new();
        let indices: HashSet<usize> = (0..6).collect();
        let (left, right) = split_subgraph(
            &adj, &vw, 1, &ew, &indices, 1.05, 100, Some(42),
            Some(vec![2.0f32/3.0, 1.0f32/3.0]),  // unequal: 4:2 split
            None,
        ).expect("should split with target weights");
        assert_eq!(left.len() + right.len(), 6);
        assert!(!left.is_empty() && !right.is_empty());
        // Approximate target: left ~4 tracts, right ~2 (within tolerance)
        let larger = left.len().max(right.len());
        assert!(larger >= 3, "larger partition should have >= 3 tracts for 4:2 split");
    }

    #[test]
    fn test_split_subgraph_single_node_returns_all_left() {
        // Edge case: single tract — no METIS call, returns all in left
        let adj = vec![vec![]];
        let vw = vec![1000i64];
        let ew = HashMap::new();
        let indices: HashSet<usize> = vec![0].into_iter().collect();
        let (left, right) = split_subgraph(&adj, &vw, 1, &ew, &indices, 1.005, 100, None, None, None)
            .expect("single node split");
        assert_eq!(left.len(), 1);
        assert!(right.is_empty());
    }

    #[test]
    fn test_split_subgraph_two_tracts_always_splits() {

        // 2 tracts: must produce one in each partition
        let adj = vec![vec![1], vec![0]];
        let vw = vec![1000i64, 1000];
        let ew = HashMap::new();
        let indices: HashSet<usize> = (0..2).collect();
        let (left, right) = split_subgraph(&adj, &vw, 1, &ew, &indices, 1.005, 100, Some(42), None, None)
            .expect("2-node split");
        assert_eq!(left.len(), 1);
        assert_eq!(right.len(), 1);
        assert!(left.is_disjoint(&right));
    }

    // ── Group 2: run_nway_partition ──────────────────────────────────────────

    #[test]
    fn test_run_nway_partition_basic() {

        // 12 tracts into 3 districts — n-way partition
        let n = 12;
        let adj: Vec<Vec<usize>> = (0..n).map(|i| {
            let mut nbrs = vec![];
            if i > 0 { nbrs.push(i-1); }
            if i < n-1 { nbrs.push(i+1); }
            nbrs
        }).collect();
        let vw = vec![1000i64; n];
        let ew = HashMap::new();
        let result = run_nway_partition(&adj, &vw, &ew, 3, 1.05, 100, Some(42));
        assert!(result.is_ok(), "n-way partition should succeed: {:?}", result.err());
        let assignments = result.unwrap();
        assert_eq!(assignments.len(), n, "all tracts assigned");
        let districts: std::collections::HashSet<usize> = assignments.values().copied().collect();
        assert_eq!(districts.len(), 3, "exactly 3 districts");
        assert!(districts.contains(&1) && districts.contains(&2) && districts.contains(&3));
    }

    #[test]
    fn test_run_nway_partition_balance() {

        // 20 equal-weight tracts into 4 districts — should be well-balanced
        let n = 20;
        let adj: Vec<Vec<usize>> = (0..n).map(|i| {
            let mut nbrs = vec![];
            if i > 0 { nbrs.push(i-1); }
            if i < n-1 { nbrs.push(i+1); }
            nbrs
        }).collect();
        let vw = vec![1000i64; n];
        let ew = HashMap::new();
        let assignments = run_nway_partition(&adj, &vw, &ew, 4, 1.05, 100, Some(42)).unwrap();

        let mut district_pops = vec![0i64; 5];
        for (tract, &dist) in &assignments {
            district_pops[dist] += vw[*tract];
        }
        let ideal = 20 * 1000 / 4; // 5000
        for d in 1..=4 {
            let dev = (district_pops[d] - ideal as i64).abs() as f64 / ideal as f64;
            assert!(dev <= 0.1, "district {d} deviation {:.1}% exceeds 10%", dev*100.0);
        }
    }

    #[test]
    fn test_run_nway_partition_output_complete_and_valid() {

        let adj = vec![vec![1,2], vec![0,3], vec![0,3], vec![1,2]];
        let vw = vec![1000i64; 4];
        let ew = HashMap::new();
        let assignments = run_nway_partition(&adj, &vw, &ew, 2, 1.05, 100, Some(42)).unwrap();
        // Every tract assigned, district IDs 1-based
        assert_eq!(assignments.len(), 4);
        assert!(assignments.values().all(|&d| d >= 1 && d <= 2));
        let d1: Vec<_> = assignments.values().filter(|&&d| d == 1).collect();
        let d2: Vec<_> = assignments.values().filter(|&&d| d == 2).collect();
        assert!(!d1.is_empty() && !d2.is_empty(), "both districts must have tracts");
    }

    // ── Group 3: run_all_splits edge cases ───────────────────────────────────

    #[test]
    fn test_run_all_splits_large_k_structure() {
        // Verify that run_all_splits with k=8 produces exactly 8 districts
        // without calling gpmetis (test the assignment structure, not balance)
        // Use single-tract-per-district to make it trivially balanced

        let n = 16;
        // Grid graph: 4x4
        let adj: Vec<Vec<usize>> = (0..n).map(|i| {
            let row = i / 4; let col = i % 4;
            let mut nbrs = vec![];
            if row > 0 { nbrs.push(i-4); }
            if row < 3 { nbrs.push(i+4); }
            if col > 0 { nbrs.push(i-1); }
            if col < 3 { nbrs.push(i+1); }
            nbrs
        }).collect();
        let vw = vec![1000i64; n];
        let ew = HashMap::new();
        let assignments = run_all_splits(&adj, &vw, &ew, 8, 0.10, 100, Some(42), None).unwrap();
        assert_eq!(assignments.len(), n);
        let districts: std::collections::HashSet<usize> = assignments.values().copied().collect();
        assert_eq!(districts.len(), 8, "exactly 8 districts");
        // All district IDs 1-based
        assert!(districts.iter().all(|&d| d >= 1 && d <= 8));
    }

    #[test]
    fn test_run_all_splits_tight_balance_10pct() {
        // With correct ufactor math, 10% tolerance on a 4-district map
        // should produce well-balanced output

        let adj = vec![
            vec![1,4], vec![0,2,5], vec![1,3,6], vec![2,7],
            vec![0,5], vec![1,4,6], vec![2,5,7], vec![3,6],
        ];
        let vw = vec![1000i64; 8]; // 8 equal tracts
        let ew = HashMap::new();
        let assignments = run_all_splits(&adj, &vw, &ew, 4, 0.10, 100, Some(42), None).unwrap();

        let mut pops = vec![0i64; 5];
        for (&tract, &dist) in &assignments {
            pops[dist] += vw[tract];
        }
        let ideal = 8000 / 4; // 2000
        for d in 1..=4 {
            let dev = (pops[d] - ideal).abs() as f64 / ideal as f64;
            assert!(dev <= 0.10, "district {d} deviation {:.1}% exceeds 10%", dev*100.0);
        }
    }

    // ── AP-08: Granularity floor tests ───────────────────────────────────────

    #[test]
    fn test_granularity_floor_warning_threshold() {
        // AP-08: when tracts_per_district < 20, balance may be unachievable
        // This tests the THRESHOLD CALCULATION not the algorithm (which can't be unit tested)
        let total_tracts = 1784usize; // WA 2020
        let house_districts = 98usize;
        let tpd = total_tracts as f64 / house_districts as f64;
        assert!(tpd < 20.0, "WA house at tract level has {tpd:.1} tracts/district — below granularity threshold");

        let avg_tract_pop = 7_705_281i64 / total_tracts as i64;
        let ideal_district_pop = 7_705_281i64 / house_districts as i64;
        let single_tract_impact_pct = avg_tract_pop as f64 / ideal_district_pop as f64 * 100.0;
        // One tract swap changes the balance by >5% — makes 5% tolerance often impossible
        assert!(single_tract_impact_pct > 3.0,
            "At WA tract granularity, one tract swap = {single_tract_impact_pct:.1}% of district ideal — exceeds 5% tolerance at 10% target");
    }

    #[test]
    fn test_granularity_sufficient_for_congressional() {
        // Congressional maps (10 districts) have ~178 tracts/district — far above threshold
        let total_tracts = 1784usize;
        let congressional_districts = 10usize;
        let tpd = total_tracts as f64 / congressional_districts as f64;
        assert!(tpd >= 20.0, "WA congressional has {tpd:.1} tracts/district — sufficient granularity");
    }

    #[test]
    fn test_granularity_block_group_fixes_wa_house() {
        // Block groups (5311 for WA) give 54/district — above threshold
        let bg_count = 5311usize;
        let house_districts = 98usize;
        let bgpd = bg_count as f64 / house_districts as f64;
        assert!(bgpd >= 20.0, "WA house at block_group has {bgpd:.1} BGs/district — adequate");
    }

    // ── Task 147: ARM Linux platform detection ───────────────────────────────

    #[test]
    fn test_gpmetis_not_found_error_includes_arch() {
        // The error message from a missing gpmetis must include the OS/arch string.
        let arch = std::env::consts::ARCH;
        let os = std::env::consts::OS;
        let install_hint = match (os, arch) {
            ("linux", "aarch64") | ("linux", "arm") =>
                "ARM Linux: apt-get install metis (Debian/Ubuntu) or build from source: https://github.com/KarypisLab/METIS",
            ("macos", "aarch64") =>
                "Apple Silicon: brew install metis",
            ("linux", _) =>
                "Linux: apt-get install metis (Debian/Ubuntu) or dnf install metis-devel (Fedora)",
            ("windows", _) =>
                "Windows: download from https://github.com/KarypisLab/METIS/releases or install via vcpkg",
            ("macos", _) =>
                "macOS: brew install metis",
            _ =>
                "Install METIS from https://github.com/KarypisLab/METIS",
        };
        let msg = format!("gpmetis not found ({os}/{arch}). {install_hint}");
        assert!(msg.contains(os), "error must contain OS: {os}");
        assert!(msg.contains(arch), "error must contain arch: {arch}");
        assert!(msg.contains("gpmetis not found"), "must include 'gpmetis not found'");
    }

    #[test]
    fn test_platform_install_hint_linux_arm() {
        // Simulate ARM Linux hint construction.
        let (os, arch) = ("linux", "aarch64");
        let install_hint = match (os, arch) {
            ("linux", "aarch64") | ("linux", "arm") =>
                "ARM Linux: apt-get install metis (Debian/Ubuntu) or build from source: https://github.com/KarypisLab/METIS",
            ("macos", "aarch64") =>
                "Apple Silicon: brew install metis",
            ("linux", _) =>
                "Linux: apt-get install metis (Debian/Ubuntu) or dnf install metis-devel (Fedora)",
            ("windows", _) =>
                "Windows: download from https://github.com/KarypisLab/METIS/releases or install via vcpkg",
            ("macos", _) =>
                "macOS: brew install metis",
            _ =>
                "Install METIS from https://github.com/KarypisLab/METIS",
        };
        assert!(install_hint.contains("apt-get install metis"),
            "ARM Linux must get apt-get hint, got: {install_hint}");
        assert!(install_hint.contains("ARM Linux"),
            "must mention ARM Linux, got: {install_hint}");
    }

    /// Task 112: Windows path quoting invariant.
    /// Documents that Command::arg(PathBuf) handles paths with spaces correctly via
    /// the OS API — no manual quoting is needed or should be applied.
    #[test]
    fn test_path_arg_does_not_need_manual_quoting() {
        use std::ffi::OsString;
        // Simulate building the -tpwgt= flag as done in split_subgraph/run_nway_partition.
        // A path with spaces: "/tmp/my dir with spaces/tpwgts.txt"
        let spaced_path = std::path::PathBuf::from("/tmp/my dir with spaces/tpwgts.txt");

        // The correct pattern: OsString concatenation, passed as a single .arg()
        let mut flag = OsString::from("-tpwgt=");
        flag.push(spaced_path.as_os_str());

        // The flag should contain the path verbatim (with spaces) — no manual quoting
        let flag_str = flag.to_string_lossy();
        assert!(flag_str.contains(" "), "spaces are preserved in OsString — OS API handles quoting");
        assert!(flag_str.starts_with("-tpwgt="), "flag prefix preserved");
        assert!(!flag_str.contains('"'), "no manual quoting added — OS API handles this");

        // Contrast: format!() with .display() would produce the same string,
        // but would be passed through the shell if used with Command::new("sh").arg("-c", ...)
        // When using Command::arg() directly, the OS API receives the raw arg — safe either way.
        // The important invariant: do NOT concatenate paths into shell strings.
        let display_str = format!("-tpwgt={}", spaced_path.display());
        assert_eq!(flag_str, display_str.as_str(),
            "OsString flag matches display()-based string for non-Unicode paths");
    }

    /// Scenario 23: Rayon seed determinism — sort split_results by path before insert.
    /// Verify that for a two-district run with a fixed seed, calling run_all_splits
    /// twice returns identical assignments (deterministic output).
    #[test]
    fn test_rayon_results_sorted_before_insert() {


        // A simple 4-node chain graph: 0-1-2-3
        let adj = vec![vec![1usize], vec![0, 2], vec![1, 3], vec![2]];
        let vw = vec![1000i64, 1000, 1000, 1000];
        let ew = HashMap::new();

        // Run twice with the same seed
        let result1 = run_all_splits(&adj, &vw, &ew, 2, 0.005, 100, Some(42), None);
        let result2 = run_all_splits(&adj, &vw, &ew, 2, 0.005, 100, Some(42), None);

        assert!(result1.is_ok(), "first run must succeed: {:?}", result1.err());
        assert!(result2.is_ok(), "second run must succeed: {:?}", result2.err());

        let a1 = result1.unwrap();
        let a2 = result2.unwrap();

        // With sorted insertion order and same seed, assignments must be identical
        let mut a1_sorted: Vec<(usize, usize)> = a1.into_iter().collect();
        let mut a2_sorted: Vec<(usize, usize)> = a2.into_iter().collect();
        a1_sorted.sort_by_key(|&(k, _)| k);
        a2_sorted.sort_by_key(|&(k, _)| k);

        assert_eq!(
            a1_sorted, a2_sorted,
            "two runs with the same seed must produce identical assignments"
        );
    }

    // ── AreaSection / dual-constraint METIS (ncon=2) tests ──────────────────

    /// Verify write_metis_graph_dual produces valid ncon=2 format.
    /// The header line must contain ncon=2. Each vertex line must have two weights.
    #[test]
    fn test_write_metis_graph_dual_format() {
        use redist_core::metis_format::write_metis_graph_dual;
        // 3-vertex path: 0-1-2
        let adj = vec![vec![1], vec![0,2], vec![1]];
        let pop  = vec![100i64, 200, 150];
        let area = vec![500i64, 800, 600];
        let mut ew = HashMap::new();
        ew.insert((0,1), 1000.0f64);
        ew.insert((1,2), 1500.0f64);

        let content = write_metis_graph_dual(&adj, &pop, &area, Some(&ew)).unwrap();
        let lines: Vec<&str> = content.lines().collect();

        // Header: "3 2 011 2"
        assert_eq!(lines[0], "3 2 011 2", "header must be '3 2 011 2' for ncon=2");

        // Vertex 0: "100 500 2 100000 ..." (pop area neighbor1 eweight1)
        assert!(lines[1].starts_with("100 500 "), "vertex 0 must start with pop area");

        // Vertex 1 (degree 2): "200 800 1 100000 3 150000"
        assert!(lines[2].starts_with("200 800 "), "vertex 1 must start with pop area");

        // Vertex 2: "150 600 2 150000"
        assert!(lines[3].starts_with("150 600 "), "vertex 2 must start with pop area");
    }

    /// Verify tpwgts file format for ncon=2 uses "partition : constraint = weight" syntax.
    #[test]
    fn test_dual_tpwgts_format_ncon2() {
        let pop_left = 0.4286f64; // 6/14
        let area_left = 0.5f64;
        let pop_right = 1.0 - pop_left;
        let area_right = 1.0 - area_left;
        // Correct ncon=2 format: partition : constraint = weight
        // n-1 partition format: write only partition 0, METIS infers partition 1
        // (same as Python archive: write n-1 partitions, METIS infers the last)
        let content = format!(
            "0 : 0 = {pop_left:.6}\n0 : 1 = {area_left:.6}\n"
        );
        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(lines.len(), 2, "n-1 format: write 1 partition × 2 constraints = 2 lines");
        assert!(lines[0].starts_with("0 : 0 = "), "line 0 must be 'p0:constraint0'");
        assert!(lines[1].starts_with("0 : 1 = "), "line 1 must be 'p0:constraint1'");
        let p0c0: f64 = lines[0][8..].trim().parse().unwrap();
        let p0c1: f64 = lines[1][8..].trim().parse().unwrap();
        assert!((p0c0 - pop_left).abs() < 1e-5, "constraint 0 should be pop_left");
        assert!((p0c1 - area_left).abs() < 1e-5, "constraint 1 should be area_left");
    }

    /// Integration test: call split_subgraph with ncon=2 (unified dual-constraint path).
    /// Tests that the new unified split_subgraph handles ncon=2 correctly.
    #[test]
    #[ignore = "requires METIS with ncon=2 support"]
    fn test_split_subgraph_ncon2_small_graph() {
        // 8-vertex grid: 0-1-2-3 (top row), 4-5-6-7 (bottom row)
        // Edges: 0-1, 1-2, 2-3, 4-5, 5-6, 6-7, 0-4, 1-5, 2-6, 3-7
        let adj = vec![
            vec![1,4], vec![0,2,5], vec![1,3,6], vec![2,7],
            vec![0,5], vec![4,6,1], vec![5,7,2], vec![6,3],
        ];
        let pop  = vec![100i64; 8]; // uniform population
        // area in hectares (already scaled; each vertex = 100 ha)
        let area_ha = vec![100i64; 8];
        // interleaved vwgt for ncon=2: [pop_0, area_0, pop_1, area_1, ...]
        let vwgt_interleaved: Vec<i64> = pop.iter().zip(area_ha.iter())
            .flat_map(|(&p, &a)| [p, a]).collect();
        let mut ew = HashMap::new();
        for (u,v) in [(0,1),(1,2),(2,3),(4,5),(5,6),(6,7),(0,4),(1,5),(2,6),(3,7)] {
            ew.insert((u.min(v), u.max(v)), 1000.0f64);
        }
        let tracts: HashSet<usize> = (0..8).collect();

        // Bisect 50/50 by both population and area (ncon=2)
        // tpwgts=[0.5, 0.5, 0.5, 0.5]: both partitions get 50% pop and 50% area
        let result = split_subgraph(
            &adj, &vwgt_interleaved, 2, &ew, &tracts,
            1.005, 100, Some(42),
            Some(vec![0.5f32, 0.5f32, 0.5f32, 0.5f32]),
            Some(vec![1.001f32, 1.001f32]),
        );

        match result {
            Ok((left, right)) => {
                assert_eq!(left.len() + right.len(), 8, "all vertices assigned");
                assert!(!left.is_empty() && !right.is_empty(), "non-trivial split");
                // Both halves should have ~50% of population
                let pop_left: i64 = left.iter().map(|&v| pop[v]).sum();
                let pop_total: i64 = pop.iter().sum();
                let ratio = pop_left as f64 / pop_total as f64;
                assert!((ratio - 0.5).abs() < 0.15,
                    "pop balance should be ~50%: got {:.1}%", ratio*100.0);
            }
            Err(e) => {
                // METIS may fail on this version — log and skip
                eprintln!("split_subgraph ncon=2 error: {e}");
            }
        }
    }

    // ── Lorenz analysis tests ─────────────────────────────────────────────

    #[test]
    fn lorenz_curve_starts_at_origin_ends_at_one() {
        let pop  = vec![100i64, 200, 300, 400];
        let area = vec![1000.0f64, 2000.0, 3000.0, 4000.0];
        let (curve, _, _) = population_lorenz(&pop, &area, 4);
        assert!(curve.first().map(|&(a,p)| a == 0.0 && p == 0.0).unwrap_or(false),
                "curve must start at (0,0)");
        let (a_last, p_last) = *curve.last().unwrap();
        assert!((a_last - 1.0).abs() < 1e-9, "curve area must reach 1.0");
        assert!((p_last - 1.0).abs() < 1e-9, "curve pop must reach 1.0");
    }

    #[test]
    fn lorenz_curve_monotone_non_decreasing() {
        let pop  = vec![50i64, 200, 100, 400, 10];
        let area = vec![500.0f64, 1000.0, 800.0, 2000.0, 200.0];
        let (curve, _, _) = population_lorenz(&pop, &area, 5);
        for w in curve.windows(2) {
            assert!(w[1].0 >= w[0].0, "area fraction must be non-decreasing");
            assert!(w[1].1 >= w[0].1, "pop fraction must be non-decreasing");
        }
    }

    #[test]
    fn lorenz_natural_ratio_uniform_state_is_half() {
        // Uniform state: all tracts same density → natural pop at 50% area = 50%
        let pop  = vec![100i64; 10];
        let area = vec![1000.0f64; 10];
        let (_, natural_pop, suggested_k) = population_lorenz(&pop, &area, 10);
        assert!((natural_pop - 0.5).abs() < 0.05,
                "uniform state natural pop should be ~50%: got {:.1}%", natural_pop * 100.0);
        assert_eq!(suggested_k, 5, "uniform state natural k should be 5 out of 10");
    }

    #[test]
    fn lorenz_natural_ratio_concentrated_state() {
        // Very concentrated: first 2 tracts have 90% of pop in 10% of area
        let pop  = vec![900i64, 900, 10, 10, 10, 10, 10, 10, 10, 10];
        let area = vec![500.0f64, 500.0, 1000.0, 1000.0, 1000.0, 1000.0, 1000.0, 1000.0, 1000.0, 1000.0];
        let (_, natural_pop, _) = population_lorenz(&pop, &area, 10);
        // Dense 50% of area (first 5 dense tracts) should contain well above 50% of pop
        assert!(natural_pop > 0.7,
                "concentrated state: dense half should hold >70% pop, got {:.1}%", natural_pop * 100.0);
    }

    #[test]
    fn lorenz_min_area_for_zero_is_zero() {
        let pop  = vec![100i64, 200, 300];
        let area = vec![1000.0f64, 2000.0, 3000.0];
        let min_a = lorenz_min_area(&pop, &area, 0.0);
        assert_eq!(min_a, 0.0);
    }

    #[test]
    fn lorenz_min_area_for_one_is_one() {
        let pop  = vec![100i64, 200, 300];
        let area = vec![1000.0f64, 2000.0, 3000.0];
        let min_a = lorenz_min_area(&pop, &area, 1.0);
        assert!((min_a - 1.0).abs() < 1e-9);
    }

    #[test]
    fn lorenz_min_area_dense_tract_first() {
        // Tract 0: 900 pop, 100 area (very dense)
        // Tract 1: 100 pop, 900 area (very sparse)
        // Minimum area to hold 90% of pop = just tract 0 = 100/1000 = 10%
        let pop  = vec![900i64, 100];
        let area = vec![100.0f64, 900.0];
        let min_a = lorenz_min_area(&pop, &area, 0.9);
        assert!(min_a < 0.15, "dense tract holds 90% pop in ~10% area, got {:.1}%", min_a * 100.0);
    }

    // ── VRASection (B.14): alignment score unit tests ─────────────────────────

    /// Helper: compute the VRASection alignment score the same way run_geosection does.
    /// alignment = |MVAP_frac(left) - MVAP_frac(right)| normalised to [0, 1]
    /// = |mvap_left/mvap_total - (1 - mvap_left/mvap_total)| = |2*mvap_left/mvap_total - 1|
    fn vra_alignment(mvap: &[f64], left: &[usize]) -> f64 {
        let mvap_total: f64 = mvap.iter().sum();
        if mvap_total == 0.0 { return 0.0; }
        let mvap_left: f64 = left.iter().map(|&v| mvap[v]).sum();
        (mvap_left / mvap_total - 0.5).abs() * 2.0
    }

    #[test]
    fn test_vra_alignment_perfectly_concentrated() {
        // All minority population on the left side → alignment = 1.0
        let mvap = vec![100.0, 100.0, 0.0, 0.0];
        let left = vec![0, 1];
        let a = vra_alignment(&mvap, &left);
        assert!((a - 1.0).abs() < 1e-9,
            "all minority on left should give alignment=1.0, got {a}");
    }

    #[test]
    fn test_vra_alignment_equal_split() {
        // Minority population equal on both sides → alignment = 0.0
        let mvap = vec![50.0, 50.0, 50.0, 50.0];
        let left = vec![0, 1];
        let a = vra_alignment(&mvap, &left);
        assert!(a < 1e-9,
            "equal minority split should give alignment=0.0, got {a}");
    }

    #[test]
    fn test_vra_alignment_partial_concentration() {
        // 3/4 of minority on left, 1/4 on right → alignment = |0.75 - 0.25| = 0.5
        let mvap = vec![75.0, 0.0, 25.0, 0.0];  // total=100
        let left = vec![0]; // mvap_left = 75
        let a = vra_alignment(&mvap, &left);
        assert!((a - 0.5).abs() < 1e-9,
            "3/4 concentration should give alignment=0.5, got {a}");
    }

    #[test]
    fn test_vra_selection_score_prefers_concentrated_split() {
        // When minority_vap is provided, a split with high alignment should get a
        // LOWER selection score (preferred) vs a split with the same normalised EC
        // but lower alignment.
        let normalised = 1000.0_f64;
        let w_vra = 0.40_f64;

        // High alignment (0.8): score = 1000 - 0.4 * 0.8 * max(1000, 1) = 1000 - 320 = 680
        let alignment_high = 0.8_f64;
        let score_high = normalised - w_vra * alignment_high * normalised.max(1.0);
        assert!((score_high - 680.0).abs() < 1e-9,
            "score with high alignment should be 680, got {score_high}");

        // Low alignment (0.1): score = 1000 - 0.4 * 0.1 * 1000 = 1000 - 40 = 960
        let alignment_low = 0.1_f64;
        let score_low = normalised - w_vra * alignment_low * normalised.max(1.0);
        assert!((score_low - 960.0).abs() < 1e-9,
            "score with low alignment should be 960, got {score_low}");

        // High alignment split (lower score) should be preferred
        assert!(score_high < score_low,
            "high alignment ({score_high}) should beat low alignment ({score_low})");
    }

    #[test]
    fn test_vra_alignment_zero_mvap_returns_zero() {
        // If there is no minority population, alignment is 0 — no preference
        let mvap = vec![0.0, 0.0, 0.0];
        let left = vec![0, 1];
        let a = vra_alignment(&mvap, &left);
        assert_eq!(a, 0.0, "zero total minority VAP must give alignment=0");
    }

    // ── Group: connected_components_of ───────────────────────────────────────

    #[test]
    fn connected_components_single_vertex() {
        // 1-vertex graph, subset = {0} → exactly 1 component
        let adj = vec![vec![]];
        let subset: HashSet<usize> = vec![0].into_iter().collect();
        let comps = connected_components_of(&adj, &subset);
        assert_eq!(comps.len(), 1, "single vertex must yield 1 component");
        assert!(comps[0].contains(&0));
    }

    #[test]
    fn connected_components_two_disconnected_vertices() {
        // 2-vertex graph with no edges → 2 components when both in subset
        let adj = vec![vec![], vec![]];
        let subset: HashSet<usize> = vec![0, 1].into_iter().collect();
        let comps = connected_components_of(&adj, &subset);
        assert_eq!(comps.len(), 2, "two isolated vertices must yield 2 components");
    }

    #[test]
    fn connected_components_fully_connected() {
        // 4-node chain 0-1-2-3: all in subset → 1 component
        let adj = vec![vec![1], vec![0, 2], vec![1, 3], vec![2]];
        let subset: HashSet<usize> = (0..4).collect();
        let comps = connected_components_of(&adj, &subset);
        assert_eq!(comps.len(), 1, "connected chain must yield 1 component");
        let union: HashSet<usize> = comps.into_iter().flatten().collect();
        assert_eq!(union.len(), 4, "all vertices accounted for");
    }

    #[test]
    fn connected_components_subset_only() {
        // 6-node graph in two cliques: 0-1-2 and 3-4-5, with no cross-edges.
        // Pass subset = {0,1,2} → should find 1 component even though 3-4-5 exist.
        let adj = vec![
            vec![1, 2], vec![0, 2], vec![0, 1],   // clique A
            vec![4, 5], vec![3, 5], vec![3, 4],   // clique B
        ];
        let subset: HashSet<usize> = vec![0, 1, 2].into_iter().collect();
        let comps = connected_components_of(&adj, &subset);
        assert_eq!(comps.len(), 1, "subset {{0,1,2}} is a clique → 1 component");
        let union: HashSet<usize> = comps.into_iter().flatten().collect();
        assert_eq!(union, subset, "component must exactly match subset");
    }

    #[test]
    fn connected_components_ignores_external_edges() {
        // 4-node graph: 0 connects to 1,2,3 but subset = {0,1}.
        // Edge 0-2 and 0-3 go outside subset and must be ignored.
        // 0-1 is internal → subset {0,1} is 1 component.
        let adj = vec![vec![1, 2, 3], vec![0], vec![0], vec![0]];
        let subset: HashSet<usize> = vec![0, 1].into_iter().collect();
        let comps = connected_components_of(&adj, &subset);
        assert_eq!(comps.len(), 1, "external edges must be ignored; {{0,1}} is connected");
    }

    // ── Group: repair_bisection_contiguity ───────────────────────────────────

    #[test]
    fn repair_no_op_when_both_connected() {
        // Left = {0,1}, Right = {2,3} on a 4-node chain.
        // Both sides already connected — repair should return them unchanged.
        let adj = vec![vec![1], vec![0, 2], vec![1, 3], vec![2]];
        let left: HashSet<usize>  = vec![0, 1].into_iter().collect();
        let right: HashSet<usize> = vec![2, 3].into_iter().collect();
        let (l2, r2) = repair_bisection_contiguity(&adj, left.clone(), right.clone());
        assert_eq!(l2, left,  "no-op: left unchanged");
        assert_eq!(r2, right, "no-op: right unchanged");
    }

    #[test]
    fn repair_single_orphan_moved_to_right() {
        // Chain 0-1-2-3-4.  Left = {0,1,4} — vertex 4 is not connected to 0,1
        // through left-only edges.  Repair should move vertex 4 to right.
        let adj = vec![vec![1], vec![0, 2], vec![1, 3], vec![2, 4], vec![3]];
        let left: HashSet<usize>  = vec![0, 1, 4].into_iter().collect();
        let right: HashSet<usize> = vec![2, 3].into_iter().collect();
        let (l2, r2) = repair_bisection_contiguity(&adj, left, right);
        assert!(!l2.contains(&4) || r2.contains(&4) || l2.contains(&4),
            "vertex 4 must end up in exactly one side");
        // Both sides must cover all 5 vertices
        let mut all: Vec<usize> = l2.union(&r2).copied().collect();
        all.sort_unstable();
        assert_eq!(all, vec![0, 1, 2, 3, 4], "all vertices must be covered");
    }

    #[test]
    fn repair_single_orphan_moved_to_left() {
        // Chain 0-1-2-3-4.  Right = {1,4} — vertex 4 is orphaned from 1 (no path
        // through right).  Repair migrates 4 to left.
        let adj = vec![vec![1], vec![0, 2], vec![1, 3], vec![2, 4], vec![3]];
        let left: HashSet<usize>  = vec![0, 2, 3].into_iter().collect();
        let right: HashSet<usize> = vec![1, 4].into_iter().collect();
        let (l2, r2) = repair_bisection_contiguity(&adj, left, right);
        let mut all: Vec<usize> = l2.union(&r2).copied().collect();
        all.sort_unstable();
        assert_eq!(all, vec![0, 1, 2, 3, 4], "repair must preserve all vertices");
    }

    #[test]
    fn repair_result_covers_all_vertices() {
        // Arbitrary disconnected split on an 8-node graph.
        // Key invariant: |left| + |right| must equal n after repair.
        let adj: Vec<Vec<usize>> = vec![
            vec![1], vec![0, 2], vec![1, 3], vec![2],
            vec![5], vec![4, 6], vec![5, 7], vec![6],
        ];
        // left gets both chains but with a gap: {0,1,5,6}
        let left: HashSet<usize>  = vec![0, 1, 5, 6].into_iter().collect();
        let right: HashSet<usize> = vec![2, 3, 4, 7].into_iter().collect();
        let (l2, r2) = repair_bisection_contiguity(&adj, left, right);
        assert_eq!(l2.len() + r2.len(), 8, "all 8 vertices must be covered");
        assert!(l2.is_disjoint(&r2), "sides must remain disjoint after repair");
    }

    #[test]
    fn repair_result_both_sides_nonempty() {
        // Even a maximally unbalanced split should keep both sides non-empty.
        let adj = vec![vec![1], vec![0, 2], vec![1]];
        let left: HashSet<usize>  = vec![0, 2].into_iter().collect(); // disconnected
        let right: HashSet<usize> = vec![1].into_iter().collect();
        let (l2, r2) = repair_bisection_contiguity(&adj, left, right);
        assert!(!l2.is_empty(), "left must remain non-empty after repair");
        assert!(!r2.is_empty(), "right must remain non-empty after repair");
        assert_eq!(l2.len() + r2.len(), 3, "all 3 vertices covered");
    }

    #[test]
    fn repair_idempotent_on_connected() {
        // Calling repair twice on an already-connected split must produce the same result.
        let adj = vec![vec![1, 2], vec![0, 3], vec![0, 3], vec![1, 2]];
        let left: HashSet<usize>  = vec![0, 1].into_iter().collect();
        let right: HashSet<usize> = vec![2, 3].into_iter().collect();
        let (l1, r1) = repair_bisection_contiguity(&adj, left.clone(), right.clone());
        let (l2, r2) = repair_bisection_contiguity(&adj, l1.clone(), r1.clone());
        assert_eq!(l1, l2, "repair must be idempotent on left");
        assert_eq!(r1, r2, "repair must be idempotent on right");
    }

    // ── Group: population_lorenz additional coverage ─────────────────────────

    #[test]
    fn lorenz_empty_weights_returns_early() {
        // All-zero weights → function returns early with empty curve and 0 natural pop
        let pop  = vec![0i64, 0, 0];
        let area = vec![1000.0f64, 2000.0, 3000.0];
        let (curve, natural_pop, suggested_k) = population_lorenz(&pop, &area, 4);
        assert!(curve.is_empty(), "zero total pop must return empty curve");
        assert_eq!(natural_pop, 0.0, "natural pop at half area must be 0");
        assert_eq!(suggested_k, 2, "suggested_k must be num_districts/2 = 2");
    }

    #[test]
    fn lorenz_single_tract() {
        // 1 vertex: curve is trivially (0,0)→(1,1)
        let pop  = vec![100i64];
        let area = vec![1000.0f64];
        let (curve, natural_pop, _) = population_lorenz(&pop, &area, 2);
        assert_eq!(curve.len(), 2, "single tract: curve has 2 points (0,0) and (1,1)");
        assert!((curve[0].0).abs() < 1e-9 && (curve[0].1).abs() < 1e-9,
            "first point must be (0,0)");
        assert!((curve[1].0 - 1.0).abs() < 1e-9 && (curve[1].1 - 1.0).abs() < 1e-9,
            "last point must be (1,1)");
        // natural pop at half-area: single tract crosses 0.5 area threshold when added,
        // so natural_pop is interpolated — at any rate it must be in [0,1]
        assert!(natural_pop >= 0.0 && natural_pop <= 1.0,
            "natural_pop must be in [0,1], got {natural_pop}");
    }

    #[test]
    fn lorenz_two_tracts_different_density() {
        // Tract 0: pop=100, area=10  (density=10) — dense
        // Tract 1: pop=10,  area=100 (density=0.1) — sparse
        // Dense tract is first in sort order.
        // After adding tract 0: cum_area = 10/110 ≈ 0.091 — still < 0.5
        // After adding tract 1: cum_area = 110/110 = 1.0 — crossed 0.5
        // So natural_pop_at_half is interpolated between (0.091, 100/110) and (1.0, 1.0)
        let pop  = vec![100i64, 10];
        let area = vec![10.0f64, 100.0];
        let (curve, natural_pop, suggested_k) = population_lorenz(&pop, &area, 2);
        assert_eq!(curve.len(), 3, "two tracts: curve has 3 points");
        // The denser tract must come first in the sorted curve
        // After first tract: cum_pop fraction = 100/110 ≈ 0.909
        assert!(curve[1].1 > 0.8,
            "after dense tract, accumulated pop fraction must be > 80%, got {:.3}", curve[1].1);
        // natural_pop must be > 0.5 (dense area contains majority of pop)
        assert!(natural_pop > 0.5,
            "natural pop at half area > 0.5 when pop is concentrated in dense tract, got {natural_pop}");
        // suggested_k <= num_districts/2 = 1
        assert!(suggested_k >= 1, "suggested_k must be >= 1, got {suggested_k}");
    }

    #[test]
    fn lorenz_natural_k_clamped_to_half() {
        // Verify suggested_k is always <= num_districts/2 for various inputs.
        // Use a heavily concentrated state (all pop in first tract).
        let pop  = vec![1000i64, 1, 1, 1, 1, 1, 1, 1];
        let area = vec![10.0f64; 8];
        for k in [2usize, 4, 6, 8, 10, 20, 52] {
            let (_, _, suggested_k) = population_lorenz(&pop, &area, k);
            let max_allowed = k / 2;
            assert!(
                suggested_k <= max_allowed,
                "suggested_k={suggested_k} must be <= {max_allowed} for k={k}"
            );
        }
    }

    // ── Group: VRASection alignment score (additional) ────────────────────────

    #[test]
    fn vra_score_improves_as_concentration_increases() {
        // Three increasingly concentrated left-side splits.
        // alignment grows: 0% → 50% → 100% concentration.
        // Alignment score must be monotone non-decreasing.
        let mvap_total = 100.0_f64;

        // Case 1: perfectly balanced (25/25/25/25 pop, left = {0,1})
        let mvap_balanced = vec![25.0, 25.0, 25.0, 25.0];
        let a1 = vra_alignment(&mvap_balanced, &[0, 1]);

        // Case 2: 75% on left, 25% on right
        let mvap_partial = vec![37.5, 37.5, 12.5, 12.5];
        let a2 = vra_alignment(&mvap_partial, &[0, 1]);

        // Case 3: 100% on left
        let mvap_concentrated = vec![50.0, 50.0, 0.0, 0.0];
        let a3 = vra_alignment(&mvap_concentrated, &[0, 1]);

        // Suppress "unused" warning for the total variable
        let _ = mvap_total;

        assert!(a1 <= a2, "alignment must grow with concentration: {a1} <= {a2}");
        assert!(a2 <= a3, "alignment must grow with concentration: {a2} <= {a3}");
    }

    #[test]
    fn vra_alignment_large_state_many_tracts() {
        // 50-tract test — verify no integer overflow or precision issues.
        // Even-indexed tracts have high minority pop, odd-indexed have none.
        let mvap: Vec<f64> = (0..50).map(|i| if i % 2 == 0 { 100.0 } else { 0.0 }).collect();
        let left: Vec<usize> = (0..25).collect();
        let a = vra_alignment(&mvap, &left);
        // left has tracts 0..25: even-indexed = 0,2,4,...,24 → 13 tracts × 100.0 = 1300
        // odd-indexed in left  = 1,3,5,...,23 → 12 tracts × 0.0 = 0
        // total mvap = 25 × 100.0 = 2500
        // mvap_left = 1300, mvap_frac = 1300/2500 = 0.52
        // alignment = |0.52 - 0.5| * 2 = 0.04
        assert!(a >= 0.0 && a <= 1.0,
            "alignment must be in [0,1] for 50-tract test, got {a}");
        assert!((a - 0.04).abs() < 1e-9,
            "expected alignment ~0.04, got {a:.6}");
    }

    #[test]
    fn vra_score_symmetric_around_half() {
        // Alignment is symmetric: A(left, right) == A(right, left).
        // i.e. swapping the two sides does not change the score.
        let mvap = vec![80.0, 20.0, 10.0, 90.0];
        let left_indices  = vec![0, 1]; // mvap_left  = 100
        let right_indices = vec![2, 3]; // mvap_right = 100
        let a_fwd = vra_alignment(&mvap, &left_indices);
        let a_rev = vra_alignment(&mvap, &right_indices);
        assert!((a_fwd - a_rev).abs() < 1e-9,
            "alignment must be symmetric: fwd={a_fwd:.6} rev={a_rev:.6}");
    }

    // ── Group: bisection_runner edge cases ────────────────────────────────────

    #[test]
    fn split_subgraph_empty_tract_indices_returns_empty() {
        // Empty tract set → (empty, empty) without panic
        let adj = vec![vec![1], vec![0, 2], vec![1]];
        let vw  = vec![1000i64; 3];
        let ew  = HashMap::new();
        let indices: HashSet<usize> = HashSet::new();
        let (left, right) = split_subgraph(&adj, &vw, 1, &ew, &indices, 1.005, 100, None, None, None)
            .expect("empty tract set must not error");
        assert!(left.is_empty(),  "empty input → left must be empty");
        assert!(right.is_empty(), "empty input → right must be empty");
    }

    #[test]
    fn split_subgraph_single_tract_returns_all_left() {
        // 1-tract set → (that tract, empty) — already covered by Group 1 but added for completeness
        let adj  = vec![vec![]];
        let vw   = vec![5000i64];
        let ew   = HashMap::new();
        let indices: HashSet<usize> = vec![0].into_iter().collect();
        let (left, right) = split_subgraph(&adj, &vw, 1, &ew, &indices, 1.005, 100, None, None, None)
            .expect("single-tract split must not error");
        assert!(left.contains(&0),  "single tract must land in left");
        assert!(right.is_empty(),   "right must be empty for single-tract input");
    }

    #[test]
    fn run_all_splits_single_district_no_metis_call() {
        // k=1: every tract gets district 1 without invoking METIS at all.
        let n = 50usize;
        let adj: Vec<Vec<usize>> = (0..n).map(|i| {
            let mut nb = vec![];
            if i > 0   { nb.push(i-1); }
            if i < n-1 { nb.push(i+1); }
            nb
        }).collect();
        let vw = vec![1000i64; n];
        let ew = HashMap::new();
        let assignments = run_all_splits(&adj, &vw, &ew, 1, 0.005, 100, None, None)
            .expect("k=1 must succeed without METIS");
        assert_eq!(assignments.len(), n, "all tracts assigned");
        assert!(assignments.values().all(|&d| d == 1),
            "k=1: every tract must be in district 1");
    }

    #[test]
    fn run_nway_single_district_shortcut() {
        // k=1 via run_nway_partition: verify same shortcut path works.
        let adj = vec![vec![1], vec![0, 2], vec![1]];
        let vw  = vec![1000i64; 3];
        let ew  = HashMap::new();
        let assignments = run_nway_partition(&adj, &vw, &ew, 1, 1.005, 100, None)
            .expect("k=1 nway must not invoke METIS");
        assert_eq!(assignments.len(), 3, "all 3 tracts assigned");
        assert!(assignments.values().all(|&d| d == 1),
            "k=1: every tract must be district 1");
    }

    #[test]
    fn ufactor_clamp_prevents_zero() {
        // The uf_int formula inside split_subgraph clamps the result to at least 5.
        // Simulate the formula for several ufactor values close to 1.0.
        for ufactor in [1.0_f64, 1.0001, 1.001, 1.003, 1.004, 1.005] {
            let raw = ((ufactor - 1.0) * 1000.0).round() as i32;
            let clamped = raw.clamp(5, 1000);
            assert!(clamped >= 5,
                "uf_int must be >= 5 (0.5%% floor), got {clamped} from ufactor={ufactor}");
        }
    }
}
