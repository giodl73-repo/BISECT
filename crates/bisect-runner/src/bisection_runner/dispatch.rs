use super::*;

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
    run_all_splits_with_search(
        adjacency,
        vertex_weights,
        edge_weights,
        num_districts,
        balance_tolerance,
        niter,
        seed,
        intermediate_dir,
        None,
    )
}

/// Variant of `run_all_splits` that supports BisectionEnsemble search at each node.
/// `bisection_ensemble` = `Some((p, ensemble_steps))` to use local ReCom instead of METIS.
pub fn run_all_splits_with_search(
    adjacency: &[Vec<usize>],
    vertex_weights: &[i64],
    edge_weights: &HashMap<(usize, usize), f64>,
    num_districts: usize,
    balance_tolerance: f64,
    niter: u32,
    seed: Option<u64>,
    intermediate_dir: Option<&Path>,
    bisection_ensemble: Option<(f64, usize)>, // (p, ensemble_steps)
) -> Result<HashMap<usize, usize>, String> {
    run_all_splits_search(
        adjacency,
        vertex_weights,
        edge_weights,
        num_districts,
        balance_tolerance,
        niter,
        seed,
        intermediate_dir,
        bisection_ensemble,
        None, "cut", 1,
    )
}

/// Try the requested seed budget at each cut, selecting the least weighted
/// boundary cost among contiguous candidates that meet both child targets.
/// This is local cut search; no child plans are evaluated to choose a parent.
pub fn run_all_splits_multi(
    adjacency: &[Vec<usize>],
    vertex_weights: &[i64],
    edge_weights: &HashMap<(usize, usize), f64>,
    num_districts: usize,
    balance_tolerance: f64,
    niter: u32,
    base_seed: u64,
    seeds: usize,
    intermediate_dir: Option<&Path>,
) -> Result<HashMap<usize, usize>, String> {
    if seeds == 0 {
        return Err("Multi search requires at least one seed.".into());
    }
    run_all_splits_search(
        adjacency,
        vertex_weights,
        edge_weights,
        num_districts,
        balance_tolerance,
        niter,
        Some(base_seed),
        intermediate_dir,
        None,
        Some(seeds), "cut", 1,
    )
}

/// Standard floor/ceil recursive tree with explicit per-node METIS tuning.
pub fn run_all_splits_tuned(adjacency:&[Vec<usize>],vertex_weights:&[i64],edge_weights:&HashMap<(usize,usize),f64>,num_districts:usize,balance_tolerance:f64,niter:u32,seed:Option<u64>,multi_seeds:Option<usize>,objective:&str,trials:u32) -> Result<HashMap<usize,usize>,String> {
    if multi_seeds==Some(0)||!["cut","volume"].contains(&objective)||!(1..=100).contains(&trials){return Err("Invalid recursive tuning or seed budget.".into());}
    run_all_splits_search(adjacency,vertex_weights,edge_weights,num_districts,balance_tolerance,niter,seed,None,None,multi_seeds,objective,trials)
}

/// Local ensemble with tuned initialization and small-region fallback.
#[allow(clippy::too_many_arguments)]
pub fn run_all_splits_ensemble_tuned(
    adjacency:&[Vec<usize>],population:&[i64],edges:&HashMap<(usize,usize),f64>,
    k:usize,tolerance:f64,niter:u32,seed:u64,steps:usize,percentile:f64,objective:&str,trials:u32,
) -> Result<HashMap<usize,usize>,String> {
    if !["cut","volume"].contains(&objective) || !(1..=100).contains(&trials) || !percentile.is_finite() || !(0.0..=1.0).contains(&percentile) {
        return Err("Invalid local ensemble refinement or percentile.".into());
    }
    run_all_splits_search(adjacency,population,edges,k,tolerance,niter,Some(seed),None,Some((percentile,steps)),None,objective,trials)
}

fn run_all_splits_search(
    adjacency: &[Vec<usize>],
    vertex_weights: &[i64],
    edge_weights: &HashMap<(usize, usize), f64>,
    num_districts: usize,
    balance_tolerance: f64,
    niter: u32,
    seed: Option<u64>,
    intermediate_dir: Option<&Path>,
    bisection_ensemble: Option<(f64, usize)>,
    multi_seeds: Option<usize>,
    objective:&str, trials:u32,
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

        // Extract each node's data before invoking METIS.
        let nodes_with_tracts: Vec<(bisect_core::BisectionNode, HashSet<usize>)> = nodes_at_depth
            .into_iter()
            .filter_map(|node| node_tracts.remove(&node.path).map(|tracts| (node, tracts)))
            .collect();

        let split_results: Vec<(String, HashSet<usize>, HashSet<usize>)> = nodes_with_tracts
            // The vendored C METIS backend shares RNG state and is not thread-safe.
            // Keep calls in stable tree order so a fixed seed reproduces exactly.
            .into_iter()
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
                let (left, right) = if let Some(seeds) = multi_seeds {
                    let total:i64=tracts.iter().map(|&v|vertex_weights[v]).sum();
                    let left_target=total as f64*node.k_left as f64/node.k as f64;
                    let right_target=total as f64-left_target;
                    let mut best:Option<(f64,usize,HashSet<usize>,HashSet<usize>)>=None;
                    let mut failures=0;
                    for index in 0..seeds {
                        let candidate=split_subgraph_tuned(adjacency,vertex_weights,1,edge_weights,&tracts,node_ufactor,niter,
                            Some(seed.unwrap_or(0).wrapping_add(index as u64)),tpwgts.clone(),None,objective,trials);
                        let Ok((left,right))=candidate else {failures+=1;continue;};
                        let left_pop:i64=left.iter().map(|&v|vertex_weights[v]).sum();
                        let right_pop=total-left_pop;
                        if left.is_empty() || right.is_empty()
                            || !is_connected_subset(adjacency,&left) || !is_connected_subset(adjacency,&right)
                            || (left_pop as f64-left_target).abs()>left_target*(node_ufactor-1.0)+1.0
                            || (right_pop as f64-right_target).abs()>right_target*(node_ufactor-1.0)+1.0 {
                            failures+=1;continue;
                        }
                        let cut=if edge_weights.is_empty() {
                            adjacency.iter().enumerate().map(|(u,neighbors)|neighbors.iter()
                                .filter(|&&v|u<v && left.contains(&u)!=left.contains(&v)).count()).sum::<usize>() as f64
                        } else {weighted_edge_cut(edge_weights,&left)};
                        if best.as_ref().map_or(true,|(best_cut,best_index,_,_)|
                            cut.total_cmp(best_cut).then(index.cmp(best_index)).is_lt()) {
                            best=Some((cut,index,left,right));
                        }
                    }
                    let Some((_,_,left,right))=best else {
                        return Err(format!("depth {} node '{}': multi search found no balanced contiguous cut in {} seeds ({} rejected).",depth,node.path,seeds,failures));
                    };
                    (left,right)
                } else if let Some((p, ens_steps)) = bisection_ensemble {
                    split_subgraph_bisection_ensemble_tuned(
                        adjacency,
                        vertex_weights,
                        edge_weights,
                        &tracts,
                        node_ufactor,
                        niter,
                        seed,
                        tpwgts.clone(),
                        ens_steps,
                        p, objective, trials,
                    )
                    .map_err(|e| format!("depth {} node '{}' (ensemble): {e}", depth, node.path))?
                } else {
                    split_subgraph_tuned(
                        adjacency,
                        vertex_weights,
                        1,
                        edge_weights,
                        &tracts,
                        node_ufactor,
                        niter,
                        seed,
                        tpwgts,
                        None, objective, trials,
                    )
                    .map_err(|e| format!("depth {} node '{}': {e}", depth, node.path))?
                };
                Ok((node.path, left, right))
            })
            .collect::<Result<Vec<_>, String>>()?;

        // Preserve deterministic insertion order even if execution changes in the future.
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
            "bisection incomplete: {}/{n} tracts assigned",
            assignments.len()
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
    let geom_graph = bisect_data::AdjacencyGraph {
        adjacency: adjacency.to_vec(),
        vertex_weights: vertex_weights.to_vec(),
        edge_weights: edge_weights.clone(),
        n_vertices: n,
        n_edges: edge_weights.len(),
        vertex_areas: vertex_areas.to_vec(),
        vertex_ext_perimeters: vertex_ext_perimeters.to_vec(),
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
        let nodes_with_tracts: Vec<(bisect_core::BisectionNode, HashSet<usize>)> = nodes_at_depth
            .into_iter()
            .filter_map(|node| node_tracts.remove(&node.path).map(|t| (node, t)))
            .collect();

        let split_results: Vec<(String, HashSet<usize>, HashSet<usize>)> = nodes_with_tracts
            // Each candidate sequence invokes the non-thread-safe C METIS backend.
            .into_iter()
            .map(|(node, tracts)| {
                let node_ufactor = 1.0 + balance_tolerance / node.k as f64;
                let tpwgts_node = if node.k_left == node.k_right {
                    None
                } else {
                    let left_w = node.k_left as f32 / node.k as f32;
                    Some(vec![left_w, 1.0_f32 - left_w])
                };

                // Run N seeds, collect (left, right, edge_cut)
                let candidates: Vec<(HashSet<usize>, HashSet<usize>, f64)> = (1..=opts
                    .seeds_per_level)
                    .filter_map(|s| {
                        let seed = Some(s as u64);
                        split_subgraph(
                            adjacency,
                            vertex_weights,
                            1,
                            edge_weights,
                            &tracts,
                            node_ufactor,
                            niter,
                            seed,
                            tpwgts_node.clone(),
                            None,
                        )
                        .ok()
                        .map(|(l, r)| {
                            let ec = weighted_edge_cut(edge_weights, &l);
                            (l, r, ec)
                        })
                    })
                    .collect();

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
            "bisection incomplete: {}/{n} tracts assigned",
            assignments.len()
        ));
    }
    Ok(assignments)
}

// ── Proportional Bisection (B.7) ─────────────────────────────────────────────

/// At each node, use Democratic votes divided by census population to choose
/// round(share * seats), clamped to one through seats minus one, for the left
/// region. This is the legacy vote/population heuristic, not two-party share.
/// METIS chooses the boundary using population and the supplied edge weights.
/// Child recursion follows the chosen seat counts. This procedure does not
/// guarantee partisan election outcomes or certify proportional representation.
pub fn run_all_splits_proportional(
    adjacency: &[Vec<usize>],
    vertex_weights: &[i64],
    edge_weights: &HashMap<(usize, usize), f64>,
    dem_votes: &[f64],
    num_districts: usize,
    balance_tolerance: f64,
    niter: u32,
    seed: Option<u64>,
    intermediate_dir: Option<&Path>,
) -> Result<HashMap<usize, usize>, String> {
    run_all_splits_proportional_recorded(adjacency,vertex_weights,edge_weights,dem_votes,num_districts,balance_tolerance,niter,seed,intermediate_dir).map(|(assignments,_)|assignments)
}

pub fn run_all_splits_proportional_recorded(
    adjacency: &[Vec<usize>],
    vertex_weights: &[i64],
    edge_weights: &HashMap<(usize, usize), f64>,
    dem_votes: &[f64],
    num_districts: usize,
    balance_tolerance: f64,
    niter: u32,
    seed: Option<u64>,
    intermediate_dir: Option<&Path>,
) -> Result<(HashMap<usize, usize>,Vec<serde_json::Value>), String> {
    let n = adjacency.len();
    if n == 0 || vertex_weights.len() != n || dem_votes.len() != n
        || num_districts == 0 || num_districts > n
        || !balance_tolerance.is_finite() || balance_tolerance < 0.0
        || balance_tolerance > 1.0 || niter == 0
        || vertex_weights.iter().any(|&p| p < 0)
        || dem_votes.iter().zip(vertex_weights).any(|(&v,&p)| !v.is_finite() || v.is_sign_negative() || v > p as f64)
        || adjacency.iter().enumerate().any(|(u,ns)| ns.iter().any(|&v| v >= n || v == u || !adjacency[v].contains(&u)))
    {
        return Err("Invalid proportional graph, vote/population dimensions or options.".into());
    }
    let total_population = vertex_weights.iter().try_fold(0i64, |sum,&p| sum.checked_add(p))
        .ok_or("Proportional population total overflows i64.")?;
    if total_population == 0 {return Err("Proportional population total must be positive.".into());}

    // Keep leaves in the frontier so intermediate snapshots cover every tract.
    let mut frontier: Vec<(String,usize,HashSet<usize>)> = vec![(String::new(),num_districts,(0..n).collect())];
    let mut depth = 0;
    let mut splits = Vec::new();
    loop {
        if let Some(dir) = intermediate_dir {
            let mut regions: Vec<_> = frontier.iter().collect();
            regions.sort_by_key(|(path,_,_)| (path.len(),path.clone()));
            let round_asgn: HashMap<usize,usize> = regions.iter().enumerate()
                .flat_map(|(id,(_,_,tracts))| tracts.iter().map(move |&v| (v,id+1))).collect();
            let round_dir = dir.join(format!("depth_{depth:02}"));
            let _ = std::fs::create_dir_all(&round_dir);
            let _ = write_intermediate_round(&round_dir,&round_asgn);
        }
        if frontier.iter().all(|(_,k,_)| *k == 1) {break;}
        frontier.sort_by_key(|(path,_,_)| path.clone());
        let mut next = Vec::new();
        for (path,k,tracts) in frontier {
            if k == 1 {next.push((path,k,tracts));continue;}
            // Float accumulation order must match across native and WASM.
            let mut indices: Vec<_> = tracts.iter().copied().collect();
            indices.sort_unstable();
            let population: i64 = indices.iter().map(|&v| vertex_weights[v]).sum();
            let votes: f64 = indices.iter().map(|&v| dem_votes[v]).sum();
            let share = if population > 0 {votes/population as f64} else {0.5};
            let left_k = ((share*k as f64).round() as usize).clamp(1,k-1);
            let right_k = k-left_k;
            splits.push((path.clone(),left_k,right_k,population,votes,share));
            let targets = if left_k == right_k {None} else {Some(vec![left_k as f32/k as f32,right_k as f32/k as f32])};
            let (left,right) = split_subgraph(adjacency,vertex_weights,1,edge_weights,&tracts,
                1.0+balance_tolerance/k as f64,niter,seed,targets,None)
                .map_err(|e| format!("depth {depth} node '{path}': {e}"))?;
            if left.len() < left_k || right.len() < right_k {
                return Err(format!("depth {depth} node '{path}': insufficient tracts for chosen {left_k}:{right_k} seats"));
            }
            next.push((format!("{path}0"),left_k,left));
            next.push((format!("{path}1"),right_k,right));
        }
        frontier = next;
        depth += 1;
    }
    frontier.sort_by_key(|(path,_,_)| (path.len(),path.clone()));
    let leaf_paths: Vec<_> = frontier.iter().enumerate().map(|(id,(path,_,_))|(path.clone(),id+1)).collect();
    let assignments: HashMap<usize,usize> = frontier.into_iter().enumerate()
        .flat_map(|(id,(_,_,tracts))| tracts.into_iter().map(move |v| (v,id+1))).collect();
    if assignments.len() != n {return Err(format!("proportional bisection incomplete: {}/{n}",assignments.len()));}
    let evidence=splits.into_iter().map(|(path,left_k,right_k,population,votes,share)| {
        let left_path=format!("{path}0");let right_path=format!("{path}1");
        let left:Vec<_>=leaf_paths.iter().filter_map(|(p,id)|p.starts_with(&left_path).then_some(*id)).collect();
        let right:Vec<_>=leaf_paths.iter().filter_map(|(p,id)|p.starts_with(&right_path).then_some(*id)).collect();
        serde_json::json!({"path":path,"left_seats":left_k,"right_seats":right_k,"population":population,"dem_votes":votes,"share":share,"left_district_ids":left,"right_district_ids":right})
    }).collect();
    Ok((assignments,evidence))
}

/// Return the set of vertices reachable from any member of `subset` using only
/// edges whose both endpoints lie within `subset`.
///
/// The full adjacency list `adj` uses **global** indices.  `subset` is also
/// expressed in global indices.  The returned `Vec<HashSet<usize>>` contains
/// one entry per connected component (global indices).
///
/// Used by `repair_bisection_contiguity` to detect fragmented partitions.
pub fn connected_components_of(adj: &[Vec<usize>], subset: &HashSet<usize>) -> Vec<HashSet<usize>> {
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
    let repair_side =
        |main: HashSet<usize>, other: HashSet<usize>| -> (HashSet<usize>, HashSet<usize>) {
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
pub(crate) fn write_intermediate_round(
    round_dir: &Path,
    assignments: &HashMap<usize, usize>,
) -> Result<(), String> {
    let path = round_dir.join("assignments.json");
    let json =
        serde_json::to_string(assignments).map_err(|e| format!("serialize intermediate: {e}"))?;
    std::fs::write(&path, json).map_err(|e| format!("write intermediate: {e}"))
}

pub(crate) fn write_ilp_solve_report(
    path: &Path,
    formulation: bisect_ilp::IlpFormulation,
    result: bisect_ilp::IlpResult,
    model_artifact: Option<bisect_ilp::IlpModelArtifact>,
) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("create ilp report dir: {e}"))?;
    }
    let tmp_path = path.with_extension("tmp.json");
    let json = if let Some(model_artifact) = model_artifact {
        bisect_ilp::solve_report_json_with_model_artifact(formulation, result, model_artifact)
    } else {
        bisect_ilp::solve_report_json(formulation, result)
    }
    .map_err(|e| format!("serialize ilp solve report: {e}"))?;
    std::fs::write(&tmp_path, json).map_err(|e| format!("write ilp solve report tmp: {e}"))?;
    std::fs::rename(&tmp_path, path).map_err(|e| format!("publish ilp solve report: {e}"))
}

pub(crate) fn write_ilp_master_lp(
    path: &Path,
    adjacency: &[Vec<usize>],
    pop: &[i64],
    k: usize,
    pop_tolerance: f64,
) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("create ilp LP dir: {e}"))?;
    }
    let tmp_path = path.with_extension("tmp.lp");
    let lp = bisect_ilp::master_lp_string(adjacency, pop, k, pop_tolerance)
        .map_err(|e| format!("export ilp master LP: {e}"))?;
    std::fs::write(&tmp_path, lp).map_err(|e| format!("write ilp master LP tmp: {e}"))?;
    std::fs::rename(&tmp_path, path).map_err(|e| format!("publish ilp master LP: {e}"))
}

pub(crate) fn ilp_model_artifact_for_report(
    report_path: &Path,
    model_path: &Path,
) -> Result<bisect_ilp::IlpModelArtifact, String> {
    let parent = report_path
        .parent()
        .ok_or_else(|| format!("ilp report path has no parent: {}", report_path.display()))?;
    let rel_path = model_path
        .strip_prefix(parent)
        .map_err(|e| format!("derive ilp model relative path: {e}"))?
        .to_string_lossy()
        .replace('\\', "/");
    use sha2::Digest;
    let model_bytes = std::fs::read(model_path)
        .map_err(|e| format!("hash ilp model LP {}: {e}", model_path.display()))?;
    let sha256 = format!("{:x}", sha2::Sha256::digest(&model_bytes));
    Ok(bisect_ilp::IlpModelArtifact {
        format: "cplex-lp".to_string(),
        path: rel_path,
        sha256,
    })
}
