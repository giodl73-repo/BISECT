use crate::args::{DiscoveryRefinementArg, ExactArgs, ExactMethodArg};
use anyhow::{anyhow, Context};
use serde::Serialize;
use std::collections::{BTreeMap, BinaryHeap, HashMap, HashSet};

struct DiscoveryGraph<'a> {
    adjacency: &'a [Vec<usize>],
}

impl rgraph_core::DirectedWeightedGraph for DiscoveryGraph<'_> {
    type EdgeId = (usize, usize);

    fn node_count(&self) -> usize {
        self.adjacency.len()
    }

    fn outgoing_edges(&self, source: usize) -> Vec<rgraph_core::WeightedEdge<Self::EdgeId>> {
        self.adjacency[source]
            .iter()
            .map(|&target| rgraph_core::WeightedEdge {
                id: (source, target),
                target,
                weight: 1.0,
            })
            .collect()
    }
}

pub fn run_exact(args: &ExactArgs) -> anyhow::Result<()> {
    match args.method {
        ExactMethodArg::CanonicalExhaustive => run_canonical_exhaustive(args),
        ExactMethodArg::CertifiedRecursive => run_certified_recursive(args),
        ExactMethodArg::CertifiedDiscovery => run_certified_discovery(args),
        ExactMethodArg::BranchAndPrice => run_branch_and_price(args),
    }
}

fn run_certified_discovery(args: &ExactArgs) -> anyhow::Result<()> {
    if args.districts < 2 {
        return Err(anyhow!(
            "certified-discovery currently requires --districts >= 2"
        ));
    }
    let context_text = std::fs::read_to_string(&args.context)
        .with_context(|| format!("read input RCTX {}", args.context.display()))?;
    let context = rplan_io::read_rctx_str(&context_text)
        .with_context(|| format!("parse input RCTX {}", args.context.display()))?;
    let populations = context
        .populations
        .as_ref()
        .ok_or_else(|| anyhow!("RCTX populations are required for certified discovery"))?;
    let root = certified_root_instance_from_context(&context, args.districts)?;
    let adjacency = graph_adjacency(
        context
            .graph
            .as_ref()
            .ok_or_else(|| anyhow!("RCTX graph is required for certified discovery"))?,
    )?;
    let edge_weights = root
        .edges
        .iter()
        .map(|edge| ((edge.left, edge.right), edge.weight as f64))
        .collect::<HashMap<_, _>>();
    let units = (0..root.unit_ids.len()).collect::<HashSet<_>>();
    let niter = if args.discovery_refinement == DiscoveryRefinementArg::NrsV01 {
        100
    } else {
        10
    };
    let nrs_profile = args.discovery_refinement == DiscoveryRefinementArg::NrsV01;
    let mut assignment = if nrs_profile {
        let tpwgts = Some(vec![
            root.k_left as f32 / root.k_parent as f32,
            root.k_right as f32 / root.k_parent as f32,
        ]);
        let (left, _right) = bisect_runner::bisection_runner::split_subgraph_nrs_v0_1(
            &adjacency,
            populations,
            1,
            &edge_weights,
            &units,
            1.005,
            niter,
            Some(args.discovery_seed),
            tpwgts,
            None,
        )
        .map_err(|error| anyhow!("certified discovery METIS split failed: {error}"))?;
        (0..root.unit_ids.len())
            .map(|unit| if left.contains(&unit) { 0_u8 } else { 1_u8 })
            .collect::<Vec<_>>()
    } else if root.k_left == root.k_right {
        let partition = bisect_runner::bisection_runner::run_nway_partition(
            &adjacency,
            populations,
            &edge_weights,
            2,
            1.005,
            niter,
            Some(args.discovery_seed),
        )
        .map_err(|error| anyhow!("certified discovery METIS split failed: {error}"))?;
        (0..root.unit_ids.len())
            .map(|unit| if partition[&unit] == 1 { 0_u8 } else { 1_u8 })
            .collect::<Vec<_>>()
    } else {
        let tpwgts = Some(vec![
            root.k_left as f32 / root.k_parent as f32,
            root.k_right as f32 / root.k_parent as f32,
        ]);
        let (left, _right) = bisect_runner::bisection_runner::split_subgraph(
            &adjacency,
            populations,
            1,
            &edge_weights,
            &units,
            1.005,
            niter,
            Some(args.discovery_seed),
            tpwgts,
            None,
        )
        .map_err(|error| anyhow!("certified discovery METIS split failed: {error}"))?;
        (0..root.unit_ids.len())
            .map(|unit| if left.contains(&unit) { 0_u8 } else { 1_u8 })
            .collect::<Vec<_>>()
    };
    let original_assignment = assignment.clone();
    if nrs_profile {
        assignment = nrs_dfs_tree_cut_candidate(&root, &adjacency, &assignment)?;
    } else {
        let left = assignment
            .iter()
            .enumerate()
            .filter_map(|(unit, &label)| (label == 0).then_some(unit))
            .collect::<HashSet<_>>();
        let right = assignment
            .iter()
            .enumerate()
            .filter_map(|(unit, &label)| (label == 1).then_some(unit))
            .collect::<HashSet<_>>();
        let (left, _) =
            bisect_runner::bisection_runner::repair_bisection_contiguity(&adjacency, left, right);
        for (unit, label) in assignment.iter_mut().enumerate() {
            *label = if left.contains(&unit) { 0 } else { 1 };
        }
    }
    let contiguity_repair_moves = assignment
        .iter()
        .zip(&original_assignment)
        .filter(|(after, before)| after != before)
        .count();
    if root.k_left == root.k_right && assignment[0] == 1 {
        for label in &mut assignment {
            *label = 1 - *label;
        }
    }
    let mut population_improvement_operations = 0;
    let mut population_improvement_units = 0;
    let mut zero_population_cut_moves = 0;
    let mut same_population_swap_moves = 0;
    let mut one_to_two_swap_moves = 0;
    let mut two_to_two_swap_moves = 0;
    if args.discovery_refinement != DiscoveryRefinementArg::Metis {
        (
            population_improvement_operations,
            population_improvement_units,
        ) = improve_discovery_population(
            &root,
            &adjacency,
            &mut assignment,
            (args.discovery_refinement == DiscoveryRefinementArg::NrsV01)
                .then(|| nrs_population_tolerance_scaled_bound(&root)),
        )?;
    }
    if matches!(
        args.discovery_refinement,
        DiscoveryRefinementArg::Fast | DiscoveryRefinementArg::Full
    ) {
        zero_population_cut_moves =
            improve_zero_population_boundary(&root, &adjacency, &mut assignment)?;
        same_population_swap_moves =
            improve_same_population_swaps(&root, &adjacency, &mut assignment)?;
        one_to_two_swap_moves =
            improve_one_to_two_population_swaps(&root, &adjacency, &mut assignment)?;
    }
    if args.discovery_refinement == DiscoveryRefinementArg::Full {
        two_to_two_swap_moves =
            improve_two_to_two_population_swaps(&root, &adjacency, &mut assignment)?;
    }
    let refinement_suffix = if args.discovery_refinement == DiscoveryRefinementArg::Full {
        String::new()
    } else {
        format!("; refinement={:?}", args.discovery_refinement).to_lowercase()
    };
    let partition_type = if nrs_profile { "recursive" } else { "kway" };
    let discovery = bisect_ilp::certified_split_discovery(
        &root,
        "METIS",
        Some(bisect_runner::bisection_runner::detect_gpmetis_version()),
        format!(
            "standard-bisect-discovery; seed={}; niter={}; ufactor=1.005; partition-type={}; zero-population-vertex-floor=1; metis-edge-scaling=heuristic; candidate-initialization=minimum-geoid-rooted-sorted-dfs-tree-edge-cut; candidate-initialization-moves={}; connected-subtree-population-operations={}; connected-subtree-population-units={}; zero-population-cut-moves={}; same-population-swap-moves={}; one-to-two-swap-moves={}; two-to-two-swap-moves={}; certified-objective=raw-u64{}",
            args.discovery_seed,
            niter,
            partition_type,
            contiguity_repair_moves,
            population_improvement_operations,
            population_improvement_units,
            zero_population_cut_moves,
            same_population_swap_moves,
            one_to_two_swap_moves,
            two_to_two_swap_moves,
            refinement_suffix,
        ),
        assignment,
    )?;
    std::fs::create_dir_all(&args.out_dir)
        .with_context(|| format!("create output directory {}", args.out_dir.display()))?;
    std::fs::write(
        args.out_dir.join("certified-split-instance.json"),
        serde_json::to_string_pretty(&root)?,
    )?;
    std::fs::write(
        args.out_dir.join("certified-discovery.json"),
        serde_json::to_string_pretty(&discovery)?,
    )?;
    if args.districts == 2 {
        write_discovery_solution_package(
            args,
            &context,
            &discovery.objective.canonical_assignment,
        )?;
    }
    write_discovery_manifest(args, &root, &discovery)
}

fn nrs_dfs_tree_cut_candidate(
    instance: &bisect_ilp::CertifiedSplitInstance,
    adjacency: &[Vec<usize>],
    assignment: &[u8],
) -> anyhow::Result<Vec<u8>> {
    let n = assignment.len();
    let mut parent = vec![usize::MAX; n];
    let mut order = Vec::with_capacity(n);
    let mut stack = vec![0_usize];
    parent[0] = 0;
    while let Some(unit) = stack.pop() {
        order.push(unit);
        let mut neighbors = adjacency[unit].clone();
        neighbors.sort_unstable_by(|left, right| right.cmp(left));
        for neighbor in neighbors {
            if parent[neighbor] == usize::MAX {
                parent[neighbor] = unit;
                stack.push(neighbor);
            }
        }
    }
    if order.len() != n {
        return Err(anyhow!("NRS input graph is disconnected"));
    }
    let mut children = vec![Vec::new(); n];
    for unit in 1..n {
        children[parent[unit]].push(unit);
    }
    let mut entry = vec![0_usize; n];
    let mut exit = vec![0_usize; n];
    let mut clock = 0_usize;
    let mut traversal = vec![(0_usize, false)];
    while let Some((unit, leaving)) = traversal.pop() {
        if leaving {
            exit[unit] = clock;
            continue;
        }
        entry[unit] = clock;
        clock += 1;
        traversal.push((unit, true));
        for &child in children[unit].iter().rev() {
            traversal.push((child, false));
        }
    }
    let mut subtree_population = instance.populations.clone();
    let mut subtree_initial_left_population = assignment
        .iter()
        .zip(&instance.populations)
        .map(|(&label, &population)| if label == 0 { population } else { 0 })
        .collect::<Vec<_>>();
    let mut subtree_minimum = (0..n).collect::<Vec<_>>();
    for &unit in order.iter().rev() {
        if unit != 0 {
            let ancestor = parent[unit];
            subtree_population[ancestor] += subtree_population[unit];
            subtree_initial_left_population[ancestor] += subtree_initial_left_population[unit];
            subtree_minimum[ancestor] = subtree_minimum[ancestor].min(subtree_minimum[unit]);
        }
    }
    let total_population = subtree_population[0];
    let target_numerator = instance.k_left as i128 * i128::from(total_population);
    let total_initial_left_population = subtree_initial_left_population[0];
    let mut best_deviation = u128::MAX;
    let mut candidates = Vec::new();
    for candidate in 1..n {
        for complement_is_left in [false, true] {
            let population = if complement_is_left {
                total_population - subtree_population[candidate]
            } else {
                subtree_population[candidate]
            };
            let deviation = (instance.k_parent as i128 * i128::from(population) - target_numerator)
                .unsigned_abs();
            let candidate_initial_left_population = if complement_is_left {
                total_initial_left_population - subtree_initial_left_population[candidate]
            } else {
                subtree_initial_left_population[candidate]
            };
            let moved_population = (population - candidate_initial_left_population)
                + (total_initial_left_population - candidate_initial_left_population);
            let minimum = if complement_is_left {
                0
            } else {
                subtree_minimum[candidate]
            };
            if deviation < best_deviation {
                best_deviation = deviation;
                candidates.clear();
            }
            if deviation == best_deviation {
                candidates.push((candidate, complement_is_left, moved_population, minimum));
            }
        }
    }
    let mut best: Option<((u64, i64, usize), usize, bool)> = None;
    for (candidate, complement_is_left, moved_population, minimum) in candidates {
        let cut = instance
            .edges
            .iter()
            .filter_map(|edge| {
                let left_in =
                    entry[candidate] <= entry[edge.left] && entry[edge.left] < exit[candidate];
                let right_in =
                    entry[candidate] <= entry[edge.right] && entry[edge.right] < exit[candidate];
                (left_in != right_in).then_some(edge.weight)
            })
            .sum::<u64>();
        let key = (cut, moved_population, minimum);
        if best.as_ref().is_none_or(|(current, _, _)| key < *current) {
            best = Some((key, candidate, complement_is_left));
        }
    }
    let (_, candidate, complement_is_left) = best.context("NRS DFS tree has no cut edge")?;
    Ok((0..n)
        .map(|unit| {
            let in_subtree = entry[candidate] <= entry[unit] && entry[unit] < exit[candidate];
            if in_subtree != complement_is_left {
                0
            } else {
                1
            }
        })
        .collect())
}

fn improve_discovery_population(
    instance: &bisect_ilp::CertifiedSplitInstance,
    adjacency: &[Vec<usize>],
    assignment: &mut [u8],
    stop_deviation: Option<u128>,
) -> anyhow::Result<(usize, usize)> {
    let total_population = instance.populations.iter().sum::<i64>();
    let mut right_population = instance
        .populations
        .iter()
        .zip(assignment.iter())
        .filter_map(|(&population, &label)| (label == 1).then_some(population))
        .sum::<i64>();
    let target_numerator = instance.k_right as i128 * i128::from(total_population);
    let remainder = target_numerator.rem_euclid(instance.k_parent as i128) as u128;
    let arithmetic_floor = remainder.min(instance.k_parent as u128 - remainder);
    let repair_target = stop_deviation
        .unwrap_or(arithmetic_floor)
        .max(arithmetic_floor);
    let mut operations = 0;
    let mut moved_units = 0;
    loop {
        let signed_deviation = instance.k_parent as i128 * i128::from(right_population)
            - instance.k_right as i128 * i128::from(total_population);
        let current_deviation = signed_deviation.unsigned_abs();
        if current_deviation <= repair_target {
            break;
        }
        let heavy = if signed_deviation > 0 { 1_u8 } else { 0_u8 };
        let Some((units, proposed_right)) = best_connected_population_subtree_move(
            instance,
            adjacency,
            assignment,
            heavy,
            right_population,
            current_deviation,
        )?
        else {
            break;
        };
        for &unit in &units {
            assignment[unit] = 1 - heavy;
        }
        right_population = proposed_right;
        operations += 1;
        moved_units += units.len();
    }
    Ok((operations, moved_units))
}

fn nrs_population_tolerance_scaled_bound(instance: &bisect_ilp::CertifiedSplitInstance) -> u128 {
    let population = instance.populations.iter().sum::<i64>() as u128;
    let smaller_child_seats = instance.k_left.min(instance.k_right) as u128;
    (5 * smaller_child_seats * population + 999) / 1_000
}

fn best_connected_population_subtree_move(
    instance: &bisect_ilp::CertifiedSplitInstance,
    adjacency: &[Vec<usize>],
    assignment: &[u8],
    heavy: u8,
    right_population: i64,
    current_deviation: u128,
) -> anyhow::Result<Option<(Vec<usize>, i64)>> {
    let heavy_units = assignment
        .iter()
        .enumerate()
        .filter_map(|(unit, &label)| (label == heavy).then_some(unit))
        .collect::<Vec<_>>();
    let heavy_seats = if heavy == 0 {
        instance.k_left
    } else {
        instance.k_right
    };
    if heavy_units.len() <= heavy_seats {
        return Ok(None);
    }
    let total_population = instance.populations.iter().sum::<i64>();
    let target_numerator = instance.k_right as i128 * i128::from(total_population);
    let mut roots = (0..16)
        .map(|quantile| heavy_units[quantile * (heavy_units.len() - 1) / 15])
        .collect::<Vec<_>>();
    roots.sort_unstable();
    roots.dedup();
    let mut heavy_adjacency = vec![Vec::new(); assignment.len()];
    for &unit in &heavy_units {
        heavy_adjacency[unit] = adjacency[unit]
            .iter()
            .copied()
            .filter(|&neighbor| assignment[neighbor] == heavy)
            .collect();
        heavy_adjacency[unit].sort_unstable();
    }

    let mut global_best_deviation = current_deviation;
    let mut global_best: Option<((i128, i64, usize, usize, usize), Vec<usize>, i64)> = None;
    for root in roots {
        let n = assignment.len();
        let mut parent = vec![usize::MAX; n];
        let mut order = Vec::with_capacity(heavy_units.len());
        let mut stack = vec![root];
        parent[root] = root;
        while let Some(unit) = stack.pop() {
            order.push(unit);
            for &neighbor in heavy_adjacency[unit].iter().rev() {
                if parent[neighbor] == usize::MAX {
                    parent[neighbor] = unit;
                    stack.push(neighbor);
                }
            }
        }
        if order.len() != heavy_units.len() {
            return Err(anyhow!(
                "NRS heavy child is disconnected during subtree repair"
            ));
        }
        let mut children = vec![Vec::new(); n];
        for &unit in &order {
            if unit != root {
                children[parent[unit]].push(unit);
            }
        }
        let mut entry = vec![usize::MAX; n];
        let mut exit = vec![usize::MAX; n];
        let mut preorder = Vec::with_capacity(heavy_units.len());
        let mut traversal = vec![(root, false)];
        while let Some((unit, leaving)) = traversal.pop() {
            if leaving {
                exit[unit] = preorder.len();
                continue;
            }
            entry[unit] = preorder.len();
            preorder.push(unit);
            traversal.push((unit, true));
            for &child in children[unit].iter().rev() {
                traversal.push((child, false));
            }
        }
        let mut subtree_population = instance.populations.clone();
        let mut subtree_count = vec![1_usize; n];
        let mut subtree_minimum = (0..n).collect::<Vec<_>>();
        let mut touches_light = (0..n)
            .map(|unit| {
                assignment[unit] == heavy
                    && adjacency[unit]
                        .iter()
                        .any(|&neighbor| assignment[neighbor] != heavy)
            })
            .collect::<Vec<_>>();
        for &unit in order.iter().rev() {
            if unit != root {
                let ancestor = parent[unit];
                subtree_population[ancestor] += subtree_population[unit];
                subtree_count[ancestor] += subtree_count[unit];
                subtree_minimum[ancestor] = subtree_minimum[ancestor].min(subtree_minimum[unit]);
                touches_light[ancestor] |= touches_light[unit];
            }
        }
        let mut tree_best_deviation = current_deviation;
        let mut tree_candidates = Vec::new();
        for &candidate in &order {
            if candidate == root
                || subtree_population[candidate] <= 0
                || !touches_light[candidate]
                || heavy_units.len() - subtree_count[candidate] < heavy_seats
                || (instance.k_left == instance.k_right
                    && heavy == 0
                    && entry[candidate] <= entry[0]
                    && entry[0] < exit[candidate])
            {
                continue;
            }
            let proposed_right = if heavy == 1 {
                right_population - subtree_population[candidate]
            } else {
                right_population + subtree_population[candidate]
            };
            let proposed = (instance.k_parent as i128 * i128::from(proposed_right)
                - target_numerator)
                .unsigned_abs();
            if proposed < tree_best_deviation {
                tree_best_deviation = proposed;
                tree_candidates.clear();
            }
            if proposed == tree_best_deviation {
                tree_candidates.push((candidate, proposed_right));
            }
        }
        if tree_best_deviation > global_best_deviation {
            continue;
        }
        if tree_best_deviation < global_best_deviation {
            global_best_deviation = tree_best_deviation;
            global_best = None;
        }
        for (candidate, proposed_right) in tree_candidates {
            if tree_best_deviation != global_best_deviation {
                continue;
            }
            let units = preorder[entry[candidate]..exit[candidate]].to_vec();
            let mut in_subtree = vec![false; n];
            for &unit in &units {
                in_subtree[unit] = true;
            }
            let cut_delta = instance
                .edges
                .iter()
                .filter_map(|edge| {
                    let left_in = in_subtree[edge.left];
                    let right_in = in_subtree[edge.right];
                    if left_in == right_in {
                        return None;
                    }
                    let outside = if left_in { edge.right } else { edge.left };
                    Some(if assignment[outside] == heavy {
                        i128::from(edge.weight)
                    } else {
                        -i128::from(edge.weight)
                    })
                })
                .sum::<i128>();
            let key = (
                cut_delta,
                subtree_population[candidate],
                subtree_minimum[candidate],
                root,
                candidate,
            );
            if global_best
                .as_ref()
                .is_none_or(|(current, _, _)| key < *current)
            {
                global_best = Some((key, units, proposed_right));
            }
        }
    }
    Ok(global_best.map(|(_, units, proposed_right)| (units, proposed_right)))
}

fn discovery_articulation_points(
    graph: &DiscoveryGraph<'_>,
    assignment: &[u8],
    heavy: u8,
) -> anyhow::Result<Vec<usize>> {
    std::thread::scope(|scope| {
        std::thread::Builder::new()
            .name("certified-discovery-articulation".to_string())
            .stack_size(64 * 1024 * 1024)
            .spawn_scoped(scope, || {
                rgraph_core::articulation_points_with_filter(graph, |(from, to)| {
                    assignment[from] == heavy && assignment[to] == heavy
                })
            })
            .map_err(|error| anyhow!("start discovery articulation worker: {error}"))?
            .join()
            .map_err(|_| anyhow!("discovery articulation worker panicked"))?
            .map_err(|error| anyhow!("discovery articulation analysis failed: {error}"))
    })
}

fn improve_zero_population_boundary(
    instance: &bisect_ilp::CertifiedSplitInstance,
    adjacency: &[Vec<usize>],
    assignment: &mut [u8],
) -> anyhow::Result<usize> {
    let graph = DiscoveryGraph { adjacency };
    let edge_weights = instance
        .edges
        .iter()
        .map(|edge| ((edge.left, edge.right), edge.weight))
        .collect::<HashMap<_, _>>();
    let mut moves = 0;
    loop {
        let mut articulations = [HashSet::new(), HashSet::new()];
        for label in [0_u8, 1_u8] {
            articulations[label as usize] =
                discovery_articulation_points(&graph, assignment, label)?
                    .into_iter()
                    .collect();
        }
        let child_counts = [
            assignment.iter().filter(|&&label| label == 0).count(),
            assignment.iter().filter(|&&label| label == 1).count(),
        ];
        let mut best: Option<(i128, usize)> = None;
        for unit in 0..assignment.len() {
            let source = assignment[unit];
            let destination = 1 - source;
            let source_seats = if source == 0 {
                instance.k_left
            } else {
                instance.k_right
            };
            if instance.populations[unit] != 0
                || child_counts[source as usize] <= source_seats
                || articulations[source as usize].contains(&unit)
                || (instance.k_left == instance.k_right && unit == 0 && source == 0)
                || !adjacency[unit]
                    .iter()
                    .any(|&neighbor| assignment[neighbor] == destination)
            {
                continue;
            }
            let delta = adjacency[unit]
                .iter()
                .map(|&neighbor| {
                    let key = (unit.min(neighbor), unit.max(neighbor));
                    let weight = edge_weights[&key] as i128;
                    if assignment[neighbor] == source {
                        weight
                    } else {
                        -weight
                    }
                })
                .sum::<i128>();
            if delta < 0
                && best
                    .as_ref()
                    .is_none_or(|&(best_delta, best_unit)| (delta, unit) < (best_delta, best_unit))
            {
                best = Some((delta, unit));
            }
        }
        let Some((_, unit)) = best else {
            break;
        };
        assignment[unit] = 1 - assignment[unit];
        moves += 1;
    }
    Ok(moves)
}

fn improve_same_population_swaps(
    instance: &bisect_ilp::CertifiedSplitInstance,
    adjacency: &[Vec<usize>],
    assignment: &mut [u8],
) -> anyhow::Result<usize> {
    let graph = DiscoveryGraph { adjacency };
    let edge_weights = instance
        .edges
        .iter()
        .map(|edge| ((edge.left, edge.right), edge.weight))
        .collect::<HashMap<_, _>>();
    let mut moves = 0;
    loop {
        let mut articulations = [HashSet::new(), HashSet::new()];
        for label in [0_u8, 1_u8] {
            articulations[label as usize] =
                discovery_articulation_points(&graph, assignment, label)?
                    .into_iter()
                    .collect();
        }
        let mut groups = [
            BTreeMap::<i64, Vec<usize>>::new(),
            BTreeMap::<i64, Vec<usize>>::new(),
        ];
        for unit in 0..assignment.len() {
            let label = assignment[unit];
            let population = instance.populations[unit];
            if population <= 0
                || articulations[label as usize].contains(&unit)
                || (instance.k_left == instance.k_right && unit == 0 && label == 0)
                || !adjacency[unit]
                    .iter()
                    .any(|&neighbor| assignment[neighbor] != label)
            {
                continue;
            }
            groups[label as usize]
                .entry(population)
                .or_default()
                .push(unit);
        }
        let mut best: Option<(i128, usize, usize)> = None;
        for (population, left_units) in &groups[0] {
            let Some(right_units) = groups[1].get(population) else {
                continue;
            };
            for &left in left_units {
                for &right in right_units {
                    if !adjacency[left]
                        .iter()
                        .any(|&neighbor| assignment[neighbor] == 1 && neighbor != right)
                        || !adjacency[right]
                            .iter()
                            .any(|&neighbor| assignment[neighbor] == 0 && neighbor != left)
                    {
                        continue;
                    }
                    let delta = swap_cut_delta(left, right, adjacency, assignment, &edge_weights);
                    if delta < 0
                        && best
                            .as_ref()
                            .is_none_or(|&(best_delta, best_left, best_right)| {
                                (delta, left, right) < (best_delta, best_left, best_right)
                            })
                    {
                        best = Some((delta, left, right));
                    }
                }
            }
        }
        let Some((_, left, right)) = best else {
            break;
        };
        assignment[left] = 1;
        assignment[right] = 0;
        moves += 1;
    }
    Ok(moves)
}

fn swap_cut_delta(
    left: usize,
    right: usize,
    adjacency: &[Vec<usize>],
    assignment: &[u8],
    edge_weights: &HashMap<(usize, usize), u64>,
) -> i128 {
    let mut incident_edges = HashSet::new();
    for unit in [left, right] {
        for &neighbor in &adjacency[unit] {
            incident_edges.insert((unit.min(neighbor), unit.max(neighbor)));
        }
    }
    incident_edges
        .into_iter()
        .map(|(from, to)| {
            let before = assignment[from] != assignment[to];
            let after_from = if from == left || from == right {
                1 - assignment[from]
            } else {
                assignment[from]
            };
            let after_to = if to == left || to == right {
                1 - assignment[to]
            } else {
                assignment[to]
            };
            let after = after_from != after_to;
            let before_value = if before { 1_i128 } else { 0 };
            let after_value = if after { 1_i128 } else { 0 };
            i128::from(edge_weights[&(from, to)]) * (after_value - before_value)
        })
        .sum()
}

fn improve_one_to_two_population_swaps(
    instance: &bisect_ilp::CertifiedSplitInstance,
    adjacency: &[Vec<usize>],
    assignment: &mut [u8],
) -> anyhow::Result<usize> {
    let graph = DiscoveryGraph { adjacency };
    let edge_weights = instance
        .edges
        .iter()
        .map(|edge| ((edge.left, edge.right), edge.weight))
        .collect::<HashMap<_, _>>();
    let mut moves = 0;
    loop {
        let mut articulations = [HashSet::new(), HashSet::new()];
        for label in [0_u8, 1_u8] {
            articulations[label as usize] =
                discovery_articulation_points(&graph, assignment, label)?
                    .into_iter()
                    .collect();
        }
        let boundary = [0_u8, 1_u8].map(|label| {
            (0..assignment.len())
                .filter(|&unit| {
                    assignment[unit] == label
                        && instance.populations[unit] > 0
                        && !articulations[label as usize].contains(&unit)
                        && !(instance.k_left == instance.k_right && label == 0 && unit == 0)
                        && adjacency[unit]
                            .iter()
                            .any(|&neighbor| assignment[neighbor] != label)
                })
                .collect::<Vec<_>>()
        });
        let mut candidates = Vec::new();
        for single_side in [0_usize, 1_usize] {
            let pair_side = 1 - single_side;
            let mut pairs_by_population = BTreeMap::<i64, Vec<(usize, usize)>>::new();
            for (index, &first) in boundary[pair_side].iter().enumerate() {
                for &second in &boundary[pair_side][index + 1..] {
                    pairs_by_population
                        .entry(instance.populations[first] + instance.populations[second])
                        .or_default()
                        .push((first, second));
                }
            }
            for &single in &boundary[single_side] {
                let Some(pairs) = pairs_by_population.get(&instance.populations[single]) else {
                    continue;
                };
                for &(first, second) in pairs {
                    let flips = [single, first, second];
                    let delta = multi_flip_cut_delta(&flips, adjacency, assignment, &edge_weights);
                    if delta < 0 {
                        candidates.push((delta, single, first, second));
                    }
                }
            }
        }
        candidates.sort_unstable();
        let mut selected = None;
        for (_, single, first, second) in candidates {
            let mut proposed = assignment.to_vec();
            for unit in [single, first, second] {
                proposed[unit] = 1 - proposed[unit];
            }
            if rgraph_core::assignment_labels_connected(adjacency, &proposed, [0_u8, 1_u8])
                .map_err(|error| anyhow!("discovery swap connectivity failed: {error}"))?
            {
                selected = Some((single, first, second, proposed));
                break;
            }
        }
        let Some((_, _, _, proposed)) = selected else {
            break;
        };
        assignment.copy_from_slice(&proposed);
        moves += 1;
    }
    Ok(moves)
}

fn multi_flip_cut_delta(
    flips: &[usize],
    adjacency: &[Vec<usize>],
    assignment: &[u8],
    edge_weights: &HashMap<(usize, usize), u64>,
) -> i128 {
    let flip_set = flips.iter().copied().collect::<HashSet<_>>();
    let incident_edges = flips
        .iter()
        .flat_map(|&unit| {
            adjacency[unit]
                .iter()
                .map(move |&neighbor| (unit.min(neighbor), unit.max(neighbor)))
        })
        .collect::<HashSet<_>>();
    incident_edges
        .into_iter()
        .map(|(from, to)| {
            let before = assignment[from] != assignment[to];
            let after_from = if flip_set.contains(&from) {
                1 - assignment[from]
            } else {
                assignment[from]
            };
            let after_to = if flip_set.contains(&to) {
                1 - assignment[to]
            } else {
                assignment[to]
            };
            let after = after_from != after_to;
            let before_value = if before { 1_i128 } else { 0 };
            let after_value = if after { 1_i128 } else { 0 };
            i128::from(edge_weights[&(from, to)]) * (after_value - before_value)
        })
        .sum()
}

fn improve_two_to_two_population_swaps(
    instance: &bisect_ilp::CertifiedSplitInstance,
    adjacency: &[Vec<usize>],
    assignment: &mut [u8],
) -> anyhow::Result<usize> {
    const CANDIDATE_LIMIT: usize = 512;
    let edge_weights = instance
        .edges
        .iter()
        .map(|edge| ((edge.left, edge.right), edge.weight))
        .collect::<HashMap<_, _>>();
    let mut moves = 0;
    while moves < 10 {
        let boundary = [0_u8, 1_u8].map(|label| {
            (0..assignment.len())
                .filter(|&unit| {
                    assignment[unit] == label
                        && instance.populations[unit] > 0
                        && !(instance.k_left == instance.k_right && label == 0 && unit == 0)
                        && adjacency[unit]
                            .iter()
                            .any(|&neighbor| assignment[neighbor] != label)
                })
                .collect::<Vec<_>>()
        });
        let pairs = boundary.map(|units| {
            let mut groups = BTreeMap::<i64, Vec<(usize, usize)>>::new();
            for (index, &first) in units.iter().enumerate() {
                for &second in &units[index + 1..] {
                    groups
                        .entry(instance.populations[first] + instance.populations[second])
                        .or_default()
                        .push((first, second));
                }
            }
            groups
        });
        let mut candidates = BinaryHeap::with_capacity(CANDIDATE_LIMIT + 1);
        for (population, left_pairs) in &pairs[0] {
            let Some(right_pairs) = pairs[1].get(population) else {
                continue;
            };
            for &(left_a, left_b) in left_pairs {
                for &(right_a, right_b) in right_pairs {
                    let flips = [left_a, left_b, right_a, right_b];
                    let delta = multi_flip_cut_delta(&flips, adjacency, assignment, &edge_weights);
                    if delta < 0 {
                        let candidate = (delta, left_a, left_b, right_a, right_b);
                        if candidates.len() < CANDIDATE_LIMIT {
                            candidates.push(candidate);
                        } else if candidates.peek().is_some_and(|worst| candidate < *worst) {
                            candidates.pop();
                            candidates.push(candidate);
                        }
                    }
                }
            }
        }
        let mut selected = None;
        for (_, left_a, left_b, right_a, right_b) in candidates.into_sorted_vec() {
            let mut proposed = assignment.to_vec();
            for unit in [left_a, left_b, right_a, right_b] {
                proposed[unit] = 1 - proposed[unit];
            }
            if rgraph_core::assignment_labels_connected(adjacency, &proposed, [0_u8, 1_u8])
                .map_err(|error| anyhow!("discovery swap connectivity failed: {error}"))?
            {
                selected = Some(proposed);
                break;
            }
        }
        let Some(proposed) = selected else {
            break;
        };
        assignment.copy_from_slice(&proposed);
        moves += 1;
    }
    Ok(moves)
}

fn run_certified_recursive(args: &ExactArgs) -> anyhow::Result<()> {
    if args.districts < 2 {
        return Err(anyhow!(
            "certified-recursive currently requires --districts >= 2"
        ));
    }
    let context_text = std::fs::read_to_string(&args.context)
        .with_context(|| format!("read input RCTX {}", args.context.display()))?;
    let context = rplan_io::read_rctx_str(&context_text)
        .with_context(|| format!("parse input RCTX {}", args.context.display()))?;
    if context.units.unit_ids.len() > args.exact_fixture_limit {
        return Err(anyhow!(
            "certified recursive instance has {} units, above --exact-fixture-limit {}",
            context.units.unit_ids.len(),
            args.exact_fixture_limit
        ));
    }
    let root = certified_root_instance_from_context(&context, args.districts)?;
    let tree = bisect_ilp::solve_certified_bisection_tree_bounded(root)?;
    bisect_ilp::verify_certified_bisection_tree_bounded(&tree)?;
    let assignment = recursive_tree_assignment(&context, &tree)?;

    std::fs::create_dir_all(&args.out_dir)
        .with_context(|| format!("create output directory {}", args.out_dir.display()))?;
    let tree_path = args.out_dir.join("certified-bisection-tree.json");
    std::fs::write(&tree_path, serde_json::to_string_pretty(&tree)?)
        .with_context(|| format!("write {}", tree_path.display()))?;
    write_recursive_solution_package(args, &context, &assignment, &tree)?;
    write_recursive_package_manifest(args, &tree)
}

fn certified_root_instance_from_context(
    context: &rplan_core::RplanContext,
    districts: usize,
) -> anyhow::Result<bisect_ilp::CertifiedSplitInstance> {
    let populations = context
        .populations
        .as_ref()
        .ok_or_else(|| anyhow!("RCTX populations are required for certified splitting"))?;
    let (k_left, k_right) = bisect_ilp::canonical_seat_split(districts)?;
    let unit_ids = context.units.unit_ids.clone();
    Ok(bisect_ilp::CertifiedSplitInstance {
        schema_version: bisect_ilp::CERTIFIED_SPLIT_INSTANCE_SCHEMA_VERSION.to_string(),
        model_id: bisect_ilp::CERTIFIED_SPLIT_MODEL_ID.to_string(),
        node_path: String::new(),
        parent_certificate_id: None,
        unit_universe_hash: bisect_ilp::certified_split_unit_universe_hash(&unit_ids)?,
        unit_ids,
        populations: populations.clone(),
        edges: exact_edges_from_context(context, "certified split")?,
        k_parent: districts,
        k_left,
        k_right,
        orientation_rule: bisect_ilp::canonical_orientation_rule(k_left, k_right),
    })
}

#[derive(Serialize)]
struct CertifiedDiscoveryManifest {
    schema_version: String,
    method: String,
    status: String,
    instance_hash: String,
    discovery_id: String,
    seed: u64,
    files: BTreeMap<String, String>,
    claim_boundary: String,
}

fn write_discovery_manifest(
    args: &ExactArgs,
    instance: &bisect_ilp::CertifiedSplitInstance,
    discovery: &bisect_ilp::CertifiedSplitDiscovery,
) -> anyhow::Result<()> {
    let mut files = BTreeMap::new();
    for name in [
        "audit-certificate.json",
        "certified-discovery.json",
        "certified-split-instance.json",
        "discovery.rctx",
        "discovery.rplan",
    ] {
        let path = args.out_dir.join(name);
        if path.is_file() {
            files.insert(name.to_string(), bisect_report::sha256_file(&path)?);
        }
    }
    let manifest = CertifiedDiscoveryManifest {
        schema_version: "certified-split-discovery-package-v1".to_string(),
        method: "certified-discovery".to_string(),
        status: "unproved-incumbent".to_string(),
        instance_hash: instance.hash()?,
        discovery_id: discovery.discovery_id.clone(),
        seed: args.discovery_seed,
        files,
        claim_boundary: "Deterministic connected METIS incumbent and exact objective record; not an optimality proof or certified split.".to_string(),
    };
    std::fs::write(
        args.out_dir.join("certified-discovery-manifest.json"),
        serde_json::to_string_pretty(&manifest)?,
    )?;
    Ok(())
}

fn write_discovery_solution_package(
    args: &ExactArgs,
    context: &rplan_core::RplanContext,
    assignment: &[u8],
) -> anyhow::Result<()> {
    let generated_at = args
        .generated_at
        .clone()
        .unwrap_or_else(bisect_report::now_iso8601);
    let jurisdiction = context
        .units
        .state
        .clone()
        .unwrap_or_else(|| "US".to_string());
    let document = rplan_io::RplanDocument {
        rplan_version: rplan_io::RPLAN_V02.to_string(),
        plan: rplan_core::DistrictPlan {
            schema_version: rplan_core::DISTRICT_PLAN_SCHEMA_VERSION.to_string(),
            units: context.units.clone(),
            assignment: assignment.iter().map(|&label| u32::from(label)).collect(),
            k: args.districts,
            display_labels: (1..=args.districts)
                .map(|district| district.to_string())
                .collect(),
            allow_empty_districts: false,
        },
        metadata: rplan_io::RplanMetadataV02 {
            label: "certified-discovery-incumbent".to_string(),
            jurisdiction: jurisdiction.clone(),
            chamber: "congressional".to_string(),
            created_at: generated_at.clone(),
            description: Some(
                "Connected deterministic discovery incumbent; exact proof status is separate."
                    .to_string(),
            ),
        },
        provenance: rplan_io::RplanProvenance {
            producer: BTreeMap::from([
                ("name".to_string(), serde_json::json!("bisect exact")),
                (
                    "method".to_string(),
                    serde_json::json!("certified-discovery"),
                ),
            ]),
            source_hashes: context.source_hashes.entries.clone(),
            conversion_lineage: Vec::new(),
        },
        geometry: None,
        extensions: BTreeMap::new(),
    };
    let profile = exact_legal_profile(
        &jurisdiction,
        &document.metadata.chamber,
        document.plan.units.year.unwrap_or(2020),
        args.tolerance,
    );
    let certificate = rplan_audit::audit_plan(
        &document.plan,
        Some(context),
        &profile,
        rplan_audit::RuntimeProvenance {
            binary_name: "bisect".to_string(),
            binary_version: env!("CARGO_PKG_VERSION").to_string(),
            git_commit: option_env!("GIT_COMMIT").map(str::to_string),
            build_profile: None,
            solver: Some(rplan_audit::SolverProvenance {
                name: "METIS discovery".to_string(),
                version: None,
                mode: Some("unproved-incumbent".to_string()),
                time_limit_secs: None,
                optimality_gap: None,
            }),
        },
        &[rplan_audit::AuditConstraint::Contiguity],
        &generated_at,
    )?;
    if matches!(certificate.result, rplan_audit::AuditResult::Fail) {
        return Err(anyhow!(
            "certified discovery failed contiguity audit: {}",
            serde_json::to_string(&certificate.checks)?
        ));
    }
    std::fs::write(
        args.out_dir.join("discovery.rplan"),
        rplan_io::write_rplan_string(&document)?,
    )?;
    std::fs::write(
        args.out_dir.join("discovery.rctx"),
        rplan_io::write_rctx_string(context)?,
    )?;
    std::fs::write(
        args.out_dir.join("audit-certificate.json"),
        serde_json::to_string_pretty(&certificate)?,
    )?;
    Ok(())
}

fn run_canonical_exhaustive(args: &ExactArgs) -> anyhow::Result<()> {
    if args.districts != 2 {
        return Err(anyhow!(
            "canonical-exhaustive currently requires --districts 2"
        ));
    }
    let context_text = std::fs::read_to_string(&args.context)
        .with_context(|| format!("read input RCTX {}", args.context.display()))?;
    let context = rplan_io::read_rctx_str(&context_text)
        .with_context(|| format!("parse input RCTX {}", args.context.display()))?;
    let instance = exact_instance_from_context(&context, args)?;
    let artifacts = bisect_ilp::solve_exact_canonical_artifacts(&instance)?;
    bisect_ilp::verify_exact_canonical_artifacts(
        &instance,
        &artifacts.certificate,
        &artifacts.proof,
    )?;
    let certificate = artifacts.certificate;

    std::fs::create_dir_all(&args.out_dir)
        .with_context(|| format!("create output directory {}", args.out_dir.display()))?;
    let instance_path = args.out_dir.join("exact-canonical-instance.json");
    let certificate_path = args.out_dir.join("exact-canonical-certificate.json");
    let proof_path = args.out_dir.join("exact-canonical-proof.json");
    std::fs::write(
        &instance_path,
        serde_json::to_string_pretty(&instance).context("serialize exact instance")?,
    )
    .with_context(|| format!("write {}", instance_path.display()))?;
    std::fs::write(
        &certificate_path,
        serde_json::to_string_pretty(&certificate).context("serialize exact certificate")?,
    )
    .with_context(|| format!("write {}", certificate_path.display()))?;
    std::fs::write(
        &proof_path,
        serde_json::to_string_pretty(&artifacts.proof).context("serialize exact proof")?,
    )
    .with_context(|| format!("write {}", proof_path.display()))?;

    if let bisect_ilp::ExactCertificateResult::Optimal { assignment, .. } = &certificate.result {
        write_canonical_solution_package(args, &context, assignment, &certificate)?;
    }
    write_canonical_package_manifest(args, &certificate)
}

fn exact_instance_from_context(
    context: &rplan_core::RplanContext,
    args: &ExactArgs,
) -> anyhow::Result<bisect_ilp::ExactCanonicalInstance> {
    let populations = context
        .populations
        .as_ref()
        .ok_or_else(|| anyhow!("RCTX populations are required for canonical exact solving"))?;
    if context.units.unit_ids.len() > args.exact_fixture_limit {
        return Err(anyhow!(
            "canonical exact instance has {} units, above --exact-fixture-limit {}",
            context.units.unit_ids.len(),
            args.exact_fixture_limit
        ));
    }
    Ok(bisect_ilp::ExactCanonicalInstance {
        schema_version: bisect_ilp::EXACT_INSTANCE_SCHEMA_VERSION.to_string(),
        model_id: bisect_ilp::EXACT_MODEL_ID.to_string(),
        unit_ids: context.units.unit_ids.clone(),
        populations: populations.clone(),
        edges: exact_edges_from_context(context, "canonical exact")?,
        k: args.districts,
    })
}

fn exact_edges_from_context(
    context: &rplan_core::RplanContext,
    purpose: &str,
) -> anyhow::Result<Vec<bisect_ilp::ExactEdge>> {
    let graph = context
        .graph
        .as_ref()
        .ok_or_else(|| anyhow!("RCTX graph is required for {purpose} solving"))?;
    let mut edges = BTreeMap::new();
    for (left, neighbors) in graph.adjacency.iter().enumerate() {
        for edge in neighbors {
            let right =
                usize::try_from(edge.to).map_err(|_| anyhow!("edge target does not fit usize"))?;
            if left >= right {
                continue;
            }
            let weight = match edge.weight {
                None => 1_u64,
                Some(value)
                    if value.is_finite()
                        && value > 0.0
                        && (value - value.round()).abs() < 1e-9
                        && value <= u64::MAX as f64 =>
                {
                    value as u64
                }
                Some(value) => {
                    return Err(anyhow!(
                        "{purpose} edge ({left}, {right}) has non-positive or non-integer weight {value}"
                    ));
                }
            };
            edges.insert(
                (left, right),
                bisect_ilp::ExactEdge {
                    left,
                    right,
                    weight,
                },
            );
        }
    }
    Ok(edges.into_values().collect())
}

fn write_canonical_solution_package(
    args: &ExactArgs,
    context: &rplan_core::RplanContext,
    assignment: &[u8],
    exact_certificate: &bisect_ilp::ExactCanonicalCertificate,
) -> anyhow::Result<()> {
    let generated_at = args
        .generated_at
        .clone()
        .unwrap_or_else(bisect_report::now_iso8601);
    let jurisdiction = context
        .units
        .state
        .clone()
        .unwrap_or_else(|| "XX".to_string());
    let document = rplan_io::RplanDocument {
        rplan_version: rplan_io::RPLAN_V02.to_string(),
        plan: rplan_core::DistrictPlan {
            schema_version: rplan_core::DISTRICT_PLAN_SCHEMA_VERSION.to_string(),
            units: context.units.clone(),
            assignment: assignment.iter().map(|&district| district as u32).collect(),
            k: args.districts,
            display_labels: (1..=args.districts)
                .map(|district| district.to_string())
                .collect(),
            allow_empty_districts: false,
        },
        metadata: rplan_io::RplanMetadataV02 {
            label: "exact-canonical".to_string(),
            jurisdiction: jurisdiction.clone(),
            chamber: "congressional".to_string(),
            created_at: generated_at.clone(),
            description: Some("E0 bounded exact canonical benchmark solution".to_string()),
        },
        provenance: rplan_io::RplanProvenance {
            producer: BTreeMap::from([
                ("name".to_string(), serde_json::json!("bisect exact")),
                ("crate".to_string(), serde_json::json!("bisect-ilp")),
                (
                    "method".to_string(),
                    serde_json::json!("canonical-exhaustive"),
                ),
                (
                    "exact_certificate_id".to_string(),
                    serde_json::json!(exact_certificate.certificate_id),
                ),
            ]),
            source_hashes: context.source_hashes.entries.clone(),
            conversion_lineage: Vec::new(),
        },
        geometry: None,
        extensions: BTreeMap::new(),
    };
    let profile = rplan_audit::LegalProfile {
        schema_version: rplan_audit::LEGAL_PROFILE_SCHEMA_VERSION.to_string(),
        profile_id: "EXACT_CANONICAL_E0_MODEL_V1".to_string(),
        jurisdiction,
        chamber: rplan_audit::Chamber::Congressional,
        year: document.plan.units.year.unwrap_or(2020),
        population_tolerance: rplan_audit::PopulationToleranceRule::Percent {
            max_deviation_percent: args.tolerance,
        },
        contiguity_required: true,
        county_split_rule: rplan_audit::SplitRule::NotEvaluated,
        municipal_split_rule: rplan_audit::SplitRule::NotEvaluated,
        nesting_rule: rplan_audit::NestingRule::NotEvaluated,
        vra_policy: rplan_audit::VraPolicy::NotEvaluated,
    };
    let lineage = rplan_audit::AlgorithmLineage::new(
        "bisect-ilp",
        env!("CARGO_PKG_VERSION"),
        "canonical-exhaustive",
        Vec::new(),
        serde_json::json!({
            "exact_certificate_id": exact_certificate.certificate_id,
            "instance_hash": exact_certificate.instance_hash,
            "model_id": exact_certificate.model_id,
        }),
    )?;
    let audit = rplan_audit::audit_plan_with_lineage(
        &document.plan,
        Some(context),
        &profile,
        rplan_audit::RuntimeProvenance {
            binary_name: "bisect".to_string(),
            binary_version: env!("CARGO_PKG_VERSION").to_string(),
            git_commit: option_env!("GIT_COMMIT").map(str::to_string),
            build_profile: None,
            solver: Some(rplan_audit::SolverProvenance {
                name: "bounded-exhaustive-enumeration".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
                mode: Some("exact-canonical-k2".to_string()),
                time_limit_secs: None,
                optimality_gap: Some(0.0),
            }),
        },
        &[
            rplan_audit::AuditConstraint::PlanShape,
            rplan_audit::AuditConstraint::Population,
            rplan_audit::AuditConstraint::Contiguity,
        ],
        &generated_at,
        Some(lineage),
    )?;
    if matches!(audit.result, rplan_audit::AuditResult::Fail) {
        return Err(anyhow!("exact canonical solution failed RPLAN audit"));
    }
    std::fs::write(
        args.out_dir.join("exact.rplan"),
        rplan_io::write_rplan_string(&document)?,
    )?;
    std::fs::write(
        args.out_dir.join("exact.rctx"),
        rplan_io::write_rctx_string(context)?,
    )?;
    std::fs::write(
        args.out_dir.join("audit-certificate.json"),
        serde_json::to_string_pretty(&audit)?,
    )?;
    Ok(())
}

#[derive(Serialize)]
struct ExactPackageManifest {
    schema_version: String,
    method: String,
    result: String,
    instance_hash: String,
    exact_certificate_id: String,
    exact_proof_id: String,
    declared_population_tolerance_percent: f64,
    files: BTreeMap<String, String>,
    claim_boundary: String,
}

fn write_canonical_package_manifest(
    args: &ExactArgs,
    certificate: &bisect_ilp::ExactCanonicalCertificate,
) -> anyhow::Result<()> {
    let result = match &certificate.result {
        bisect_ilp::ExactCertificateResult::Optimal { .. } => "optimal",
        bisect_ilp::ExactCertificateResult::Infeasible => "infeasible",
    };
    let mut files = BTreeMap::new();
    for name in [
        "exact-canonical-instance.json",
        "exact-canonical-certificate.json",
        "exact-canonical-proof.json",
        "exact.rplan",
        "exact.rctx",
        "audit-certificate.json",
    ] {
        let path = args.out_dir.join(name);
        if path.is_file() {
            files.insert(name.to_string(), bisect_report::sha256_file(&path)?);
        }
    }
    let manifest = ExactPackageManifest {
        schema_version: "exact-canonical-package-manifest-v1".to_string(),
        method: "canonical-exhaustive".to_string(),
        result: result.to_string(),
        instance_hash: certificate.instance_hash.clone(),
        exact_certificate_id: certificate.certificate_id.clone(),
        exact_proof_id: certificate.proof.transcript_id.clone(),
        declared_population_tolerance_percent: args.tolerance,
        files,
        claim_boundary: format!(
            "Bounded E0 k=2 exhaustive certificate with at most {} units; not national exact readiness or legal certification.",
            args.exact_fixture_limit
        ),
    };
    std::fs::write(
        args.out_dir.join("exact-package-manifest.json"),
        serde_json::to_string_pretty(&manifest)?,
    )?;
    Ok(())
}

fn recursive_tree_assignment(
    context: &rplan_core::RplanContext,
    tree: &bisect_ilp::CertifiedBisectionTree,
) -> anyhow::Result<Vec<u32>> {
    let index = context
        .units
        .unit_ids
        .iter()
        .enumerate()
        .map(|(index, unit_id)| (unit_id.as_str(), index))
        .collect::<BTreeMap<_, _>>();
    let mut assignment = vec![u32::MAX; context.units.unit_ids.len()];
    for leaf in &tree.leaves {
        let district = u32::try_from(leaf.district_index)
            .map_err(|_| anyhow!("recursive district index does not fit u32"))?;
        for unit_id in &leaf.unit_ids {
            let unit_index = *index
                .get(unit_id.as_str())
                .ok_or_else(|| anyhow!("tree leaf references unknown unit {unit_id}"))?;
            if assignment[unit_index] != u32::MAX {
                return Err(anyhow!("tree leaf assigns unit {unit_id} more than once"));
            }
            assignment[unit_index] = district;
        }
    }
    if assignment.contains(&u32::MAX) {
        return Err(anyhow!("recursive tree does not assign every context unit"));
    }
    Ok(assignment)
}

fn write_recursive_solution_package(
    args: &ExactArgs,
    context: &rplan_core::RplanContext,
    assignment: &[u32],
    tree: &bisect_ilp::CertifiedBisectionTree,
) -> anyhow::Result<()> {
    let generated_at = args
        .generated_at
        .clone()
        .unwrap_or_else(bisect_report::now_iso8601);
    let jurisdiction = context
        .units
        .state
        .clone()
        .unwrap_or_else(|| "XX".to_string());
    let document = rplan_io::RplanDocument {
        rplan_version: rplan_io::RPLAN_V02.to_string(),
        plan: rplan_core::DistrictPlan {
            schema_version: rplan_core::DISTRICT_PLAN_SCHEMA_VERSION.to_string(),
            units: context.units.clone(),
            assignment: assignment.to_vec(),
            k: args.districts,
            display_labels: (1..=args.districts)
                .map(|district| district.to_string())
                .collect(),
            allow_empty_districts: false,
        },
        metadata: rplan_io::RplanMetadataV02 {
            label: "certified-recursive-bisection".to_string(),
            jurisdiction: jurisdiction.clone(),
            chamber: "congressional".to_string(),
            created_at: generated_at.clone(),
            description: Some(
                "Bounded exact certification of the standard-bisect recursive tree".to_string(),
            ),
        },
        provenance: rplan_io::RplanProvenance {
            producer: BTreeMap::from([
                ("name".to_string(), serde_json::json!("bisect exact")),
                ("crate".to_string(), serde_json::json!("bisect-ilp")),
                (
                    "method".to_string(),
                    serde_json::json!("certified-recursive"),
                ),
                (
                    "certified_tree_id".to_string(),
                    serde_json::json!(tree.tree_id),
                ),
            ]),
            source_hashes: context.source_hashes.entries.clone(),
            conversion_lineage: Vec::new(),
        },
        geometry: None,
        extensions: BTreeMap::new(),
    };
    let profile = rplan_audit::LegalProfile {
        schema_version: rplan_audit::LEGAL_PROFILE_SCHEMA_VERSION.to_string(),
        profile_id: "CERTIFIED_RECURSIVE_BISECTION_BOUNDED_V1".to_string(),
        jurisdiction,
        chamber: rplan_audit::Chamber::Congressional,
        year: document.plan.units.year.unwrap_or(2020),
        population_tolerance: rplan_audit::PopulationToleranceRule::Percent {
            max_deviation_percent: args.tolerance,
        },
        contiguity_required: true,
        county_split_rule: rplan_audit::SplitRule::NotEvaluated,
        municipal_split_rule: rplan_audit::SplitRule::NotEvaluated,
        nesting_rule: rplan_audit::NestingRule::NotEvaluated,
        vra_policy: rplan_audit::VraPolicy::NotEvaluated,
    };
    let lineage = rplan_audit::AlgorithmLineage::new(
        "bisect-ilp",
        env!("CARGO_PKG_VERSION"),
        "certified-recursive",
        Vec::new(),
        serde_json::json!({
            "certified_tree_id": tree.tree_id,
            "root_unit_universe_hash": tree.root_unit_universe_hash,
            "split_count": tree.nodes.len(),
            "leaf_count": tree.leaves.len(),
        }),
    )?;
    let audit = rplan_audit::audit_plan_with_lineage(
        &document.plan,
        Some(context),
        &profile,
        rplan_audit::RuntimeProvenance {
            binary_name: "bisect".to_string(),
            binary_version: env!("CARGO_PKG_VERSION").to_string(),
            git_commit: option_env!("GIT_COMMIT").map(str::to_string),
            build_profile: None,
            solver: Some(rplan_audit::SolverProvenance {
                name: "bounded-certified-recursive-enumeration".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
                mode: Some("standard-bisect-tree".to_string()),
                time_limit_secs: None,
                optimality_gap: Some(0.0),
            }),
        },
        &[
            rplan_audit::AuditConstraint::PlanShape,
            rplan_audit::AuditConstraint::Population,
            rplan_audit::AuditConstraint::Contiguity,
        ],
        &generated_at,
        Some(lineage),
    )?;
    if matches!(audit.result, rplan_audit::AuditResult::Fail) {
        return Err(anyhow!("certified recursive solution failed RPLAN audit"));
    }
    std::fs::write(
        args.out_dir.join("exact.rplan"),
        rplan_io::write_rplan_string(&document)?,
    )?;
    std::fs::write(
        args.out_dir.join("exact.rctx"),
        rplan_io::write_rctx_string(context)?,
    )?;
    std::fs::write(
        args.out_dir.join("audit-certificate.json"),
        serde_json::to_string_pretty(&audit)?,
    )?;
    Ok(())
}

#[derive(Serialize)]
struct CertifiedRecursivePackageManifest {
    schema_version: String,
    method: String,
    tree_id: String,
    root_unit_universe_hash: String,
    districts: usize,
    split_count: usize,
    leaf_count: usize,
    declared_population_tolerance_percent: f64,
    files: BTreeMap<String, String>,
    claim_boundary: String,
}

fn write_recursive_package_manifest(
    args: &ExactArgs,
    tree: &bisect_ilp::CertifiedBisectionTree,
) -> anyhow::Result<()> {
    let mut files = BTreeMap::new();
    for name in [
        "certified-bisection-tree.json",
        "exact.rplan",
        "exact.rctx",
        "audit-certificate.json",
    ] {
        let path = args.out_dir.join(name);
        if path.is_file() {
            files.insert(name.to_string(), bisect_report::sha256_file(&path)?);
        }
    }
    let manifest = CertifiedRecursivePackageManifest {
        schema_version: "certified-recursive-bisection-package-v1".to_string(),
        method: "certified-recursive".to_string(),
        tree_id: tree.tree_id.clone(),
        root_unit_universe_hash: tree.root_unit_universe_hash.clone(),
        districts: tree.k,
        split_count: tree.nodes.len(),
        leaf_count: tree.leaves.len(),
        declared_population_tolerance_percent: args.tolerance,
        files,
        claim_boundary: format!(
            "Bounded exact certification of the sequential standard-bisect tree with at most {} units per split; not global unrestricted-map optimality or block-scale readiness.",
            args.exact_fixture_limit
        ),
    };
    std::fs::write(
        args.out_dir.join("certified-tree-package-manifest.json"),
        serde_json::to_string_pretty(&manifest)?,
    )?;
    Ok(())
}

fn run_branch_and_price(args: &ExactArgs) -> anyhow::Result<()> {
    let context_text = std::fs::read_to_string(&args.context)
        .with_context(|| format!("read input RCTX {}", args.context.display()))?;
    let context = rplan_io::read_rctx_str(&context_text)
        .with_context(|| format!("parse input RCTX {}", args.context.display()))?;
    let graph = context
        .graph
        .as_ref()
        .ok_or_else(|| anyhow!("RCTX graph is required for exact branch-and-price"))?;
    let populations = context
        .populations
        .as_ref()
        .ok_or_else(|| anyhow!("RCTX populations are required for exact branch-and-price"))?;
    let adjacency = graph_adjacency(graph)?;
    let report = bisect_column::solve_branch_price(
        &adjacency,
        populations,
        bisect_column::BranchPriceConfig {
            k: args.districts,
            tolerance: args.tolerance / 100.0,
            formulation_only: args.formulation_only,
            exact_fixture_limit: args.exact_fixture_limit,
        },
    )?;

    std::fs::create_dir_all(&args.out_dir)
        .with_context(|| format!("create output directory {}", args.out_dir.display()))?;
    let report_path = args.out_dir.join("branch-price-report.json");
    std::fs::write(
        &report_path,
        serde_json::to_string_pretty(&report).context("serialize branch-price report")?,
    )
    .with_context(|| format!("write {}", report_path.display()))?;

    let lineage = report
        .algorithm_lineage(env!("CARGO_PKG_VERSION"), Vec::new())
        .context("build branch-price algorithm lineage")?;
    let lineage_path = args.out_dir.join("algorithm-lineage.json");
    std::fs::write(
        &lineage_path,
        serde_json::to_string_pretty(&lineage).context("serialize branch-price lineage")?,
    )
    .with_context(|| format!("write {}", lineage_path.display()))?;
    if report.solution.is_some() {
        write_solution_package(args, &context, &report, lineage)?;
    }
    Ok(())
}

fn write_solution_package(
    args: &ExactArgs,
    context: &rplan_core::RplanContext,
    report: &bisect_column::BranchPriceReport,
    lineage: rplan_audit::AlgorithmLineage,
) -> anyhow::Result<()> {
    let solution = report
        .solution
        .as_ref()
        .ok_or_else(|| anyhow!("branch-price report has no solved plan"))?;
    let generated_at = args
        .generated_at
        .clone()
        .unwrap_or_else(bisect_report::now_iso8601);
    let jurisdiction = context
        .units
        .state
        .clone()
        .unwrap_or_else(|| "US".to_string());
    let document = rplan_io::RplanDocument {
        rplan_version: rplan_io::RPLAN_V02.to_string(),
        plan: rplan_core::DistrictPlan {
            schema_version: rplan_core::DISTRICT_PLAN_SCHEMA_VERSION.to_string(),
            units: context.units.clone(),
            assignment: solution
                .assignment
                .iter()
                .map(|&district| district as u32)
                .collect(),
            k: args.districts,
            display_labels: (1..=args.districts)
                .map(|district| district.to_string())
                .collect(),
            allow_empty_districts: false,
        },
        metadata: rplan_io::RplanMetadataV02 {
            label: "branch-and-price-exact".to_string(),
            jurisdiction: jurisdiction.clone(),
            chamber: "congressional".to_string(),
            created_at: generated_at.clone(),
            description: Some("U.17 branch-and-price exact fixture solution".to_string()),
        },
        provenance: rplan_io::RplanProvenance {
            producer: std::collections::BTreeMap::from([
                ("name".to_string(), serde_json::json!("bisect exact")),
                ("crate".to_string(), serde_json::json!("bisect-column")),
                ("method".to_string(), serde_json::json!("branch-and-price")),
            ]),
            source_hashes: context.source_hashes.entries.clone(),
            conversion_lineage: Vec::new(),
        },
        geometry: None,
        extensions: std::collections::BTreeMap::new(),
    };
    let profile = exact_legal_profile(
        &jurisdiction,
        &document.metadata.chamber,
        document.plan.units.year.unwrap_or(2020),
        args.tolerance,
    );
    let certificate = rplan_audit::audit_plan_with_lineage(
        &document.plan,
        Some(context),
        &profile,
        rplan_audit::RuntimeProvenance {
            binary_name: "bisect".to_string(),
            binary_version: env!("CARGO_PKG_VERSION").to_string(),
            git_commit: option_env!("GIT_COMMIT").map(str::to_string),
            build_profile: None,
            solver: Some(rplan_audit::SolverProvenance {
                name: "branch-and-price".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
                mode: Some(format!("{:?}", report.status)),
                time_limit_secs: None,
                optimality_gap: report.gap,
            }),
        },
        &[
            rplan_audit::AuditConstraint::Population,
            rplan_audit::AuditConstraint::Contiguity,
        ],
        &generated_at,
        Some(lineage),
    )?;
    if matches!(certificate.result, rplan_audit::AuditResult::Fail) {
        return Err(anyhow!("exact branch-price solution failed RPLAN audit"));
    }

    let rplan_path = args.out_dir.join("exact.rplan");
    let rctx_path = args.out_dir.join("exact.rctx");
    let certificate_path = args.out_dir.join("audit-certificate.json");
    std::fs::write(&rplan_path, rplan_io::write_rplan_string(&document)?)
        .with_context(|| format!("write {}", rplan_path.display()))?;
    std::fs::write(&rctx_path, rplan_io::write_rctx_string(context)?)
        .with_context(|| format!("write {}", rctx_path.display()))?;
    std::fs::write(
        &certificate_path,
        serde_json::to_string_pretty(&certificate).context("serialize exact audit certificate")?,
    )
    .with_context(|| format!("write {}", certificate_path.display()))?;
    write_exact_manifest(
        args,
        &document,
        &certificate,
        &profile,
        report,
        &generated_at,
    )
}

fn write_exact_manifest(
    args: &ExactArgs,
    document: &rplan_io::RplanDocument,
    certificate: &rplan_audit::AuditCertificate,
    profile: &rplan_audit::LegalProfile,
    report: &bisect_column::BranchPriceReport,
    generated_at: &str,
) -> anyhow::Result<()> {
    let manifest = bisect_report::PlanManifest {
        label: document.metadata.label.clone(),
        state_code: document
            .plan
            .units
            .state
            .clone()
            .unwrap_or_else(|| document.metadata.jurisdiction.clone()),
        year: document
            .plan
            .units
            .year
            .map(|year| year.to_string())
            .unwrap_or_default(),
        chamber: document.metadata.chamber.clone(),
        num_districts: document.plan.k,
        population_source: "rplan-context".to_string(),
        partition_mode: "branch-and-price".to_string(),
        binary_version: env!("CARGO_PKG_VERSION").to_string(),
        created_at: generated_at.to_string(),
        balance_tolerance_pct: args.tolerance,
        population_balance_valid: true,
        edge_cut: report
            .solution
            .as_ref()
            .map(|solution| solution.objective as f64),
        rplan_path: Some("exact.rplan".to_string()),
        rctx_path: Some("exact.rctx".to_string()),
        audit_certificate_path: Some("audit-certificate.json".to_string()),
        audit_certificate_sha256: Some(bisect_report::sha256_file(
            &args.out_dir.join("audit-certificate.json"),
        )?),
        audit_certificate_content_hash: Some(certificate.content_hash.clone()),
        audit_result: Some(rplan_audit_result_label(&certificate.result).to_string()),
        legal_profile_id: Some(profile.profile_id.clone()),
        context_hash: certificate.context_hash.clone(),
        n_units: document.plan.assignment.len(),
        unit_type: "rplan unit".to_string(),
        ..bisect_report::PlanManifest::default()
    };
    let manifest_path = args.out_dir.join("manifest.json");
    std::fs::write(
        &manifest_path,
        serde_json::to_string_pretty(&manifest).context("serialize exact manifest")?,
    )
    .with_context(|| format!("write {}", manifest_path.display()))?;
    Ok(())
}

fn exact_legal_profile(
    jurisdiction: &str,
    chamber: &str,
    year: u16,
    tolerance_percent: f64,
) -> rplan_audit::LegalProfile {
    let chamber = match chamber {
        "congressional" => rplan_audit::Chamber::Congressional,
        other => rplan_audit::Chamber::Custom(other.to_string()),
    };
    rplan_audit::LegalProfile {
        schema_version: rplan_audit::LEGAL_PROFILE_SCHEMA_VERSION.to_string(),
        profile_id: "BISECT_BRANCH_PRICE_EXACT_V1".to_string(),
        jurisdiction: jurisdiction.to_string(),
        chamber,
        year,
        population_tolerance: rplan_audit::PopulationToleranceRule::Percent {
            max_deviation_percent: tolerance_percent,
        },
        contiguity_required: true,
        county_split_rule: rplan_audit::SplitRule::CountOnly,
        municipal_split_rule: rplan_audit::SplitRule::NotEvaluated,
        nesting_rule: rplan_audit::NestingRule::NotEvaluated,
        vra_policy: rplan_audit::VraPolicy::NotEvaluated,
    }
}

fn rplan_audit_result_label(result: &rplan_audit::AuditResult) -> &'static str {
    match result {
        rplan_audit::AuditResult::Pass => "pass",
        rplan_audit::AuditResult::Fail => "fail",
        rplan_audit::AuditResult::PassWithWarnings => "pass-with-warnings",
    }
}

fn graph_adjacency(graph: &rplan_core::UnitGraph) -> anyhow::Result<Vec<Vec<usize>>> {
    graph
        .adjacency
        .iter()
        .enumerate()
        .map(|(from, edges)| {
            edges
                .iter()
                .map(|edge| {
                    let to = usize::try_from(edge.to).map_err(|_| {
                        anyhow!("graph edge target at unit {from} does not fit usize")
                    })?;
                    if to >= graph.adjacency.len() {
                        Err(anyhow!("edge target {to} out of bounds for unit {from}"))
                    } else {
                        Ok(to)
                    }
                })
                .collect()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    use tempfile::TempDir;

    use rplan_core::{
        CanonicalOrder, EdgeKind, EdgeSemantics, PlanUnitIndex, RplanContext, SourceHashes,
        UnitEdge, UnitGraph, UnitKind, RCTX_VERSION,
    };

    #[test]
    fn run_exact_branch_and_price_emits_report_and_lineage() {
        let tmp = TempDir::new().unwrap();
        let context_path = tmp.path().join("fixture.rctx");
        let out_dir = tmp.path().join("exact");
        std::fs::write(
            &context_path,
            rplan_io::write_rctx_string(&path4_context()).unwrap(),
        )
        .unwrap();

        run_exact(&ExactArgs {
            context: context_path,
            out_dir: out_dir.clone(),
            method: ExactMethodArg::BranchAndPrice,
            districts: 2,
            tolerance: 1.0,
            formulation_only: false,
            exact_fixture_limit: 8,
            generated_at: Some("2026-07-10T12:00:00Z".to_string()),
            discovery_seed: 1,
            discovery_refinement: DiscoveryRefinementArg::Full,
        })
        .unwrap();

        let report_text =
            std::fs::read_to_string(out_dir.join("branch-price-report.json")).unwrap();
        let report: bisect_column::BranchPriceReport = serde_json::from_str(&report_text).unwrap();
        assert_eq!(
            report.status,
            bisect_column::BranchPriceStatus::ExactFixtureOptimal
        );
        assert_eq!(report.solution.unwrap().assignment, vec![0, 0, 1, 1]);

        let lineage_text = std::fs::read_to_string(out_dir.join("algorithm-lineage.json")).unwrap();
        let lineage: rplan_audit::AlgorithmLineage = serde_json::from_str(&lineage_text).unwrap();
        assert_eq!(lineage.producer_crate, "bisect-column");
        assert_eq!(lineage.method, "branch-and-price");

        let exact_plan_text = std::fs::read_to_string(out_dir.join("exact.rplan")).unwrap();
        let exact_context_text = std::fs::read_to_string(out_dir.join("exact.rctx")).unwrap();
        let certificate_text =
            std::fs::read_to_string(out_dir.join("audit-certificate.json")).unwrap();
        let manifest_text = std::fs::read_to_string(out_dir.join("manifest.json")).unwrap();
        let exact_plan = rplan_io::read_rplan_str(&exact_plan_text).unwrap();
        let exact_context = rplan_io::read_rctx_str(&exact_context_text).unwrap();
        let certificate: rplan_audit::AuditCertificate =
            serde_json::from_str(&certificate_text).unwrap();
        rplan_audit::verify_audit_certificate(
            &certificate,
            Some(&exact_plan.plan),
            Some(&exact_context),
        )
        .unwrap();
        let manifest: bisect_report::PlanManifest = serde_json::from_str(&manifest_text).unwrap();
        assert_eq!(manifest.rplan_path.as_deref(), Some("exact.rplan"));
        assert_eq!(
            manifest.audit_certificate_path.as_deref(),
            Some("audit-certificate.json")
        );
        assert_eq!(manifest.edge_cut, Some(1.0));
    }

    #[test]
    fn run_exact_canonical_emits_verified_package() {
        let tmp = TempDir::new().unwrap();
        let context_path = tmp.path().join("fixture.rctx");
        let out_dir = tmp.path().join("canonical");
        std::fs::write(
            &context_path,
            rplan_io::write_rctx_string(&path4_context()).unwrap(),
        )
        .unwrap();

        run_exact(&ExactArgs {
            context: context_path,
            out_dir: out_dir.clone(),
            method: ExactMethodArg::CanonicalExhaustive,
            districts: 2,
            tolerance: 1.0,
            formulation_only: false,
            exact_fixture_limit: 8,
            generated_at: Some("2026-07-10T12:00:00Z".to_string()),
            discovery_seed: 1,
            discovery_refinement: DiscoveryRefinementArg::Full,
        })
        .unwrap();

        let instance: bisect_ilp::ExactCanonicalInstance = serde_json::from_str(
            &std::fs::read_to_string(out_dir.join("exact-canonical-instance.json")).unwrap(),
        )
        .unwrap();
        let exact_certificate: bisect_ilp::ExactCanonicalCertificate = serde_json::from_str(
            &std::fs::read_to_string(out_dir.join("exact-canonical-certificate.json")).unwrap(),
        )
        .unwrap();
        let proof: bisect_ilp::ExactProofTranscript = serde_json::from_str(
            &std::fs::read_to_string(out_dir.join("exact-canonical-proof.json")).unwrap(),
        )
        .unwrap();
        bisect_ilp::verify_exact_canonical_artifacts(&instance, &exact_certificate, &proof)
            .unwrap();
        let plan = rplan_io::read_rplan_str(
            &std::fs::read_to_string(out_dir.join("exact.rplan")).unwrap(),
        )
        .unwrap();
        assert_eq!(plan.plan.assignment, vec![0, 0, 1, 1]);
        let context =
            rplan_io::read_rctx_str(&std::fs::read_to_string(out_dir.join("exact.rctx")).unwrap())
                .unwrap();
        let audit: rplan_audit::AuditCertificate = serde_json::from_str(
            &std::fs::read_to_string(out_dir.join("audit-certificate.json")).unwrap(),
        )
        .unwrap();
        rplan_audit::verify_audit_certificate(&audit, Some(&plan.plan), Some(&context)).unwrap();
        let package: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(out_dir.join("exact-package-manifest.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(package["result"], "optimal");
        assert_eq!(package["method"], "canonical-exhaustive");
        assert_eq!(package["declared_population_tolerance_percent"], 1.0);
        assert_eq!(package["files"].as_object().unwrap().len(), 6);
        assert_eq!(package["exact_proof_id"], proof.transcript_id);
        for (name, expected_hash) in package["files"].as_object().unwrap() {
            assert_eq!(
                bisect_report::sha256_file(&out_dir.join(name)).unwrap(),
                expected_hash.as_str().unwrap()
            );
        }
    }

    #[test]
    fn run_exact_canonical_emits_infeasibility_without_plan() {
        let tmp = TempDir::new().unwrap();
        let context_path = tmp.path().join("infeasible.rctx");
        let out_dir = tmp.path().join("infeasible");
        std::fs::write(
            &context_path,
            rplan_io::write_rctx_string(&three_islands_context()).unwrap(),
        )
        .unwrap();

        run_exact(&ExactArgs {
            context: context_path,
            out_dir: out_dir.clone(),
            method: ExactMethodArg::CanonicalExhaustive,
            districts: 2,
            tolerance: 1.0,
            formulation_only: false,
            exact_fixture_limit: 8,
            generated_at: Some("2026-07-10T12:00:00Z".to_string()),
            discovery_seed: 1,
            discovery_refinement: DiscoveryRefinementArg::Full,
        })
        .unwrap();

        let certificate: bisect_ilp::ExactCanonicalCertificate = serde_json::from_str(
            &std::fs::read_to_string(out_dir.join("exact-canonical-certificate.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(
            certificate.result,
            bisect_ilp::ExactCertificateResult::Infeasible
        );
        assert!(!out_dir.join("exact.rplan").exists());
        let package: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(out_dir.join("exact-package-manifest.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(package["result"], "infeasible");
        assert_eq!(package["files"].as_object().unwrap().len(), 3);
    }

    #[test]
    fn run_exact_canonical_rejects_non_integer_edge_weight() {
        let tmp = TempDir::new().unwrap();
        let context_path = tmp.path().join("bad-weight.rctx");
        let mut context = path4_context();
        context.graph.as_mut().unwrap().adjacency[0][0].weight = Some(1.5);
        context.graph.as_mut().unwrap().adjacency[1][0].weight = Some(1.5);
        std::fs::write(
            &context_path,
            rplan_io::write_rctx_string(&context).unwrap(),
        )
        .unwrap();
        let error = run_exact(&ExactArgs {
            context: context_path,
            out_dir: tmp.path().join("out"),
            method: ExactMethodArg::CanonicalExhaustive,
            districts: 2,
            tolerance: 1.0,
            formulation_only: false,
            exact_fixture_limit: 8,
            generated_at: Some("2026-07-10T12:00:00Z".to_string()),
            discovery_seed: 1,
            discovery_refinement: DiscoveryRefinementArg::Full,
        })
        .unwrap_err();
        assert!(error.to_string().contains("non-positive or non-integer"));
    }

    #[test]
    fn run_exact_certified_recursive_emits_verified_tree_package() {
        let tmp = TempDir::new().unwrap();
        let context_path = tmp.path().join("path8.rctx");
        let out_dir = tmp.path().join("recursive");
        std::fs::write(
            &context_path,
            rplan_io::write_rctx_string(&path_context(8)).unwrap(),
        )
        .unwrap();

        run_exact(&ExactArgs {
            context: context_path,
            out_dir: out_dir.clone(),
            method: ExactMethodArg::CertifiedRecursive,
            districts: 4,
            tolerance: 1.0,
            formulation_only: false,
            exact_fixture_limit: 8,
            generated_at: Some("2026-07-10T12:00:00Z".to_string()),
            discovery_seed: 1,
            discovery_refinement: DiscoveryRefinementArg::Full,
        })
        .unwrap();

        let tree: bisect_ilp::CertifiedBisectionTree = serde_json::from_str(
            &std::fs::read_to_string(out_dir.join("certified-bisection-tree.json")).unwrap(),
        )
        .unwrap();
        bisect_ilp::verify_certified_bisection_tree_bounded(&tree).unwrap();
        assert_eq!(tree.nodes.len(), 3);
        assert_eq!(tree.leaves.len(), 4);
        let plan = rplan_io::read_rplan_str(
            &std::fs::read_to_string(out_dir.join("exact.rplan")).unwrap(),
        )
        .unwrap();
        assert_eq!(plan.plan.assignment, vec![0, 0, 1, 1, 2, 2, 3, 3]);
        let package: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(out_dir.join("certified-tree-package-manifest.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(package["method"], "certified-recursive");
        assert_eq!(package["tree_id"], tree.tree_id);
        assert_eq!(package["files"].as_object().unwrap().len(), 4);
        for (name, expected_hash) in package["files"].as_object().unwrap() {
            assert_eq!(
                bisect_report::sha256_file(&out_dir.join(name)).unwrap(),
                expected_hash.as_str().unwrap()
            );
        }
    }

    #[test]
    fn run_exact_certified_recursive_enforces_fixture_limit() {
        let tmp = TempDir::new().unwrap();
        let context_path = tmp.path().join("path8.rctx");
        std::fs::write(
            &context_path,
            rplan_io::write_rctx_string(&path_context(8)).unwrap(),
        )
        .unwrap();
        let error = run_exact(&ExactArgs {
            context: context_path,
            out_dir: tmp.path().join("out"),
            method: ExactMethodArg::CertifiedRecursive,
            districts: 4,
            tolerance: 1.0,
            formulation_only: false,
            exact_fixture_limit: 7,
            generated_at: Some("2026-07-10T12:00:00Z".to_string()),
            discovery_seed: 1,
            discovery_refinement: DiscoveryRefinementArg::Full,
        })
        .unwrap_err();
        assert!(error.to_string().contains("above --exact-fixture-limit"));
    }

    #[test]
    fn run_exact_certified_discovery_is_deterministic_and_unproved() {
        let tmp = TempDir::new().unwrap();
        let context_path = tmp.path().join("path8.rctx");
        std::fs::write(
            &context_path,
            rplan_io::write_rctx_string(&path_context(8)).unwrap(),
        )
        .unwrap();
        let run = |out_dir: std::path::PathBuf| {
            run_exact(&ExactArgs {
                context: context_path.clone(),
                out_dir,
                method: ExactMethodArg::CertifiedDiscovery,
                districts: 4,
                tolerance: 1.0,
                formulation_only: false,
                exact_fixture_limit: 8,
                generated_at: Some("2026-07-10T12:00:00Z".to_string()),
                discovery_seed: 42,
                discovery_refinement: DiscoveryRefinementArg::Full,
            })
            .unwrap();
        };
        let first = tmp.path().join("first");
        let second = tmp.path().join("second");
        run(first.clone());
        run(second.clone());
        assert_eq!(
            std::fs::read(first.join("certified-discovery.json")).unwrap(),
            std::fs::read(second.join("certified-discovery.json")).unwrap()
        );
        let instance: bisect_ilp::CertifiedSplitInstance = serde_json::from_str(
            &std::fs::read_to_string(first.join("certified-split-instance.json")).unwrap(),
        )
        .unwrap();
        let discovery: bisect_ilp::CertifiedSplitDiscovery = serde_json::from_str(
            &std::fs::read_to_string(first.join("certified-discovery.json")).unwrap(),
        )
        .unwrap();
        let requests =
            bisect_ilp::compile_certified_split_compact_proof_requests(&instance, &discovery)
                .unwrap();
        assert_eq!(requests.len(), 3);
        let manifest: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(first.join("certified-discovery-manifest.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(manifest["status"], "unproved-incumbent");
        assert_eq!(manifest["seed"], 42);
    }

    #[test]
    fn run_exact_certified_discovery_supports_unequal_seat_ratio() {
        let tmp = TempDir::new().unwrap();
        let context_path = tmp.path().join("path10.rctx");
        std::fs::write(
            &context_path,
            rplan_io::write_rctx_string(&path_context(10)).unwrap(),
        )
        .unwrap();
        let out_dir = tmp.path().join("unequal");
        run_exact(&ExactArgs {
            context: context_path,
            out_dir: out_dir.clone(),
            method: ExactMethodArg::CertifiedDiscovery,
            districts: 5,
            tolerance: 1.0,
            formulation_only: false,
            exact_fixture_limit: 10,
            generated_at: Some("2026-07-10T12:00:00Z".to_string()),
            discovery_seed: 42,
            discovery_refinement: DiscoveryRefinementArg::NrsV01,
        })
        .unwrap();
        let instance: bisect_ilp::CertifiedSplitInstance = serde_json::from_str(
            &std::fs::read_to_string(out_dir.join("certified-split-instance.json")).unwrap(),
        )
        .unwrap();
        let discovery: bisect_ilp::CertifiedSplitDiscovery = serde_json::from_str(
            &std::fs::read_to_string(out_dir.join("certified-discovery.json")).unwrap(),
        )
        .unwrap();
        assert_eq!((instance.k_left, instance.k_right), (2, 3));
        assert_eq!(discovery.objective.canonical_assignment.len(), 10);
        assert!(discovery.method.contains("partition-type=recursive"));
        assert!(discovery.method.contains("refinement=nrsv01"));
        assert!(bisect_ilp::certified_split_children_connected(
            &instance,
            &discovery.objective.canonical_assignment
        )
        .unwrap());
    }

    #[test]
    fn nrs_dfs_tree_cut_normalizes_fragmented_candidate_deterministically() {
        let context = path_context(10);
        let instance = certified_root_instance_from_context(&context, 5).unwrap();
        let adjacency = graph_adjacency(context.graph.as_ref().unwrap()).unwrap();
        let raw = (0..10).map(|unit| (unit % 2) as u8).collect::<Vec<_>>();
        let first = nrs_dfs_tree_cut_candidate(&instance, &adjacency, &raw).unwrap();
        let second = nrs_dfs_tree_cut_candidate(&instance, &adjacency, &raw).unwrap();
        assert_eq!(first, second);
        assert!(
            rgraph_core::assignment_labels_connected(&adjacency, &first, [0_u8, 1_u8]).unwrap()
        );
        let left_population = instance
            .populations
            .iter()
            .zip(&first)
            .filter_map(|(&population, &label)| (label == 0).then_some(population))
            .sum::<i64>();
        assert_eq!(left_population, 400);
    }

    #[test]
    fn nrs_dfs_tree_cut_replaces_connected_unbalanced_candidate() {
        let context = path_context(10);
        let instance = certified_root_instance_from_context(&context, 5).unwrap();
        let adjacency = graph_adjacency(context.graph.as_ref().unwrap()).unwrap();
        let raw = vec![0, 0, 0, 0, 0, 0, 1, 1, 1, 1];
        assert!(rgraph_core::assignment_labels_connected(&adjacency, &raw, [0_u8, 1_u8]).unwrap());
        let candidate = nrs_dfs_tree_cut_candidate(&instance, &adjacency, &raw).unwrap();
        assert_ne!(candidate, raw);
        assert!(
            rgraph_core::assignment_labels_connected(&adjacency, &candidate, [0_u8, 1_u8]).unwrap()
        );
        let left_population = instance
            .populations
            .iter()
            .zip(&candidate)
            .filter_map(|(&population, &label)| (label == 0).then_some(population))
            .sum::<i64>();
        assert_eq!(left_population, 400);
    }

    #[test]
    fn zero_population_boundary_cleanup_reduces_cut_without_changing_population() {
        let unit_ids = (0..4).map(|unit| format!("u{unit}")).collect::<Vec<_>>();
        let instance = bisect_ilp::CertifiedSplitInstance {
            schema_version: bisect_ilp::CERTIFIED_SPLIT_INSTANCE_SCHEMA_VERSION.to_string(),
            model_id: bisect_ilp::CERTIFIED_SPLIT_MODEL_ID.to_string(),
            node_path: String::new(),
            parent_certificate_id: None,
            unit_universe_hash: bisect_ilp::certified_split_unit_universe_hash(&unit_ids).unwrap(),
            unit_ids,
            populations: vec![1, 0, 1, 1],
            edges: vec![
                bisect_ilp::ExactEdge {
                    left: 0,
                    right: 1,
                    weight: 1,
                },
                bisect_ilp::ExactEdge {
                    left: 1,
                    right: 2,
                    weight: 1,
                },
                bisect_ilp::ExactEdge {
                    left: 1,
                    right: 3,
                    weight: 1,
                },
                bisect_ilp::ExactEdge {
                    left: 2,
                    right: 3,
                    weight: 1,
                },
            ],
            k_parent: 2,
            k_left: 1,
            k_right: 1,
            orientation_rule: bisect_ilp::SplitOrientationRule::EqualSeatsUnitZeroLeft,
        };
        let adjacency = vec![vec![1], vec![0, 2, 3], vec![1, 3], vec![1, 2]];
        let mut assignment = vec![0, 0, 1, 1];
        let before =
            bisect_ilp::evaluate_certified_split_objective(&instance, &assignment).unwrap();
        assert_eq!(
            improve_zero_population_boundary(&instance, &adjacency, &mut assignment).unwrap(),
            1
        );
        let after = bisect_ilp::evaluate_certified_split_objective(&instance, &assignment).unwrap();
        assert_eq!(assignment, vec![0, 1, 1, 1]);
        assert_eq!(
            after.max_population_deviation_scaled,
            before.max_population_deviation_scaled
        );
        assert!(after.weighted_boundary_cut < before.weighted_boundary_cut);
    }

    #[test]
    fn population_repair_uses_cut_delta_before_geoid_order() {
        let unit_ids = (0..6).map(|unit| format!("u{unit}")).collect::<Vec<_>>();
        let instance = bisect_ilp::CertifiedSplitInstance {
            schema_version: bisect_ilp::CERTIFIED_SPLIT_INSTANCE_SCHEMA_VERSION.to_string(),
            model_id: bisect_ilp::CERTIFIED_SPLIT_MODEL_ID.to_string(),
            node_path: String::new(),
            parent_certificate_id: None,
            unit_universe_hash: bisect_ilp::certified_split_unit_universe_hash(&unit_ids).unwrap(),
            unit_ids,
            populations: vec![2, 1, 1, 1, 1, 1],
            edges: vec![
                bisect_ilp::ExactEdge {
                    left: 0,
                    right: 1,
                    weight: 10,
                },
                bisect_ilp::ExactEdge {
                    left: 1,
                    right: 3,
                    weight: 10,
                },
                bisect_ilp::ExactEdge {
                    left: 2,
                    right: 3,
                    weight: 1,
                },
                bisect_ilp::ExactEdge {
                    left: 0,
                    right: 2,
                    weight: 1,
                },
                bisect_ilp::ExactEdge {
                    left: 1,
                    right: 4,
                    weight: 1,
                },
                bisect_ilp::ExactEdge {
                    left: 2,
                    right: 4,
                    weight: 10,
                },
                bisect_ilp::ExactEdge {
                    left: 4,
                    right: 5,
                    weight: 1,
                },
            ],
            k_parent: 2,
            k_left: 1,
            k_right: 1,
            orientation_rule: bisect_ilp::SplitOrientationRule::EqualSeatsUnitZeroLeft,
        };
        let adjacency = vec![
            vec![1, 2],
            vec![0, 3, 4],
            vec![0, 3, 4],
            vec![1, 2],
            vec![1, 2, 5],
            vec![4],
        ];
        let mut assignment = vec![0, 0, 0, 0, 1, 1];
        assert_eq!(
            improve_discovery_population(&instance, &adjacency, &mut assignment, None).unwrap(),
            (1, 1)
        );
        assert_eq!(assignment, vec![0, 0, 1, 0, 1, 1]);
    }

    #[test]
    fn population_repair_moves_connected_subtree_past_articulation_local_minimum() {
        let unit_ids = (0..4).map(|unit| format!("u{unit}")).collect::<Vec<_>>();
        let instance = bisect_ilp::CertifiedSplitInstance {
            schema_version: bisect_ilp::CERTIFIED_SPLIT_INSTANCE_SCHEMA_VERSION.to_string(),
            model_id: bisect_ilp::CERTIFIED_SPLIT_MODEL_ID.to_string(),
            node_path: String::new(),
            parent_certificate_id: None,
            unit_universe_hash: bisect_ilp::certified_split_unit_universe_hash(&unit_ids).unwrap(),
            unit_ids,
            populations: vec![20, 10, 20, 50],
            edges: vec![
                bisect_ilp::ExactEdge {
                    left: 0,
                    right: 1,
                    weight: 1,
                },
                bisect_ilp::ExactEdge {
                    left: 1,
                    right: 2,
                    weight: 1,
                },
                bisect_ilp::ExactEdge {
                    left: 1,
                    right: 3,
                    weight: 1,
                },
            ],
            k_parent: 2,
            k_left: 1,
            k_right: 1,
            orientation_rule: bisect_ilp::SplitOrientationRule::EqualSeatsUnitZeroLeft,
        };
        let adjacency = vec![vec![1], vec![0, 2, 3], vec![1], vec![1]];
        assert_eq!(nrs_population_tolerance_scaled_bound(&instance), 1);
        let mut within_requested_tolerance = vec![0, 1, 1, 1];
        assert_eq!(
            improve_discovery_population(
                &instance,
                &adjacency,
                &mut within_requested_tolerance,
                Some(60),
            )
            .unwrap(),
            (0, 0)
        );
        assert_eq!(within_requested_tolerance, vec![0, 1, 1, 1]);
        let mut assignment = vec![0, 1, 1, 1];
        assert_eq!(
            improve_discovery_population(&instance, &adjacency, &mut assignment, None).unwrap(),
            (1, 2)
        );
        assert_eq!(assignment, vec![0, 0, 0, 1]);
        assert!(bisect_ilp::certified_split_children_connected(&instance, &assignment).unwrap());
    }

    #[test]
    fn same_population_swap_reduces_cut_and_preserves_balance() {
        let unit_ids = (0..6).map(|unit| format!("u{unit}")).collect::<Vec<_>>();
        let instance = bisect_ilp::CertifiedSplitInstance {
            schema_version: bisect_ilp::CERTIFIED_SPLIT_INSTANCE_SCHEMA_VERSION.to_string(),
            model_id: bisect_ilp::CERTIFIED_SPLIT_MODEL_ID.to_string(),
            node_path: String::new(),
            parent_certificate_id: None,
            unit_universe_hash: bisect_ilp::certified_split_unit_universe_hash(&unit_ids).unwrap(),
            unit_ids,
            populations: vec![1, 2, 3, 1, 2, 3],
            edges: vec![
                bisect_ilp::ExactEdge {
                    left: 0,
                    right: 1,
                    weight: 1,
                },
                bisect_ilp::ExactEdge {
                    left: 1,
                    right: 2,
                    weight: 1,
                },
                bisect_ilp::ExactEdge {
                    left: 3,
                    right: 4,
                    weight: 1,
                },
                bisect_ilp::ExactEdge {
                    left: 4,
                    right: 5,
                    weight: 1,
                },
                bisect_ilp::ExactEdge {
                    left: 2,
                    right: 3,
                    weight: 10,
                },
                bisect_ilp::ExactEdge {
                    left: 0,
                    right: 5,
                    weight: 10,
                },
            ],
            k_parent: 2,
            k_left: 1,
            k_right: 1,
            orientation_rule: bisect_ilp::SplitOrientationRule::EqualSeatsUnitZeroLeft,
        };
        let adjacency = vec![
            vec![1, 5],
            vec![0, 2],
            vec![1, 3],
            vec![2, 4],
            vec![3, 5],
            vec![0, 4],
        ];
        let mut assignment = vec![0, 0, 0, 1, 1, 1];
        let before =
            bisect_ilp::evaluate_certified_split_objective(&instance, &assignment).unwrap();
        assert_eq!(
            improve_same_population_swaps(&instance, &adjacency, &mut assignment).unwrap(),
            1
        );
        let after = bisect_ilp::evaluate_certified_split_objective(&instance, &assignment).unwrap();
        assert_eq!(assignment, vec![0, 0, 1, 1, 1, 0]);
        assert_eq!(
            after.max_population_deviation_scaled,
            before.max_population_deviation_scaled
        );
        assert!(after.weighted_boundary_cut < before.weighted_boundary_cut);
    }

    #[test]
    fn one_to_two_swap_reduces_cut_and_preserves_balance() {
        let unit_ids = (0..7).map(|unit| format!("u{unit}")).collect::<Vec<_>>();
        let mut edges = Vec::new();
        for group in [&[0, 1, 2][..], &[3, 4, 5, 6][..]] {
            for (index, &left) in group.iter().enumerate() {
                for &right in &group[index + 1..] {
                    edges.push(bisect_ilp::ExactEdge {
                        left,
                        right,
                        weight: 1,
                    });
                }
            }
        }
        for (left, right) in [(1, 4), (1, 6), (3, 0), (3, 2), (5, 0), (5, 2)] {
            edges.push(bisect_ilp::ExactEdge {
                left: left.min(right),
                right: left.max(right),
                weight: 10,
            });
        }
        edges.sort_by_key(|edge| (edge.left, edge.right));
        edges.dedup_by_key(|edge| (edge.left, edge.right));
        let instance = bisect_ilp::CertifiedSplitInstance {
            schema_version: bisect_ilp::CERTIFIED_SPLIT_INSTANCE_SCHEMA_VERSION.to_string(),
            model_id: bisect_ilp::CERTIFIED_SPLIT_MODEL_ID.to_string(),
            node_path: String::new(),
            parent_certificate_id: None,
            unit_universe_hash: bisect_ilp::certified_split_unit_universe_hash(&unit_ids).unwrap(),
            unit_ids,
            populations: vec![1, 3, 2, 1, 1, 2, 2],
            edges: edges.clone(),
            k_parent: 2,
            k_left: 1,
            k_right: 1,
            orientation_rule: bisect_ilp::SplitOrientationRule::EqualSeatsUnitZeroLeft,
        };
        let mut adjacency = vec![Vec::new(); 7];
        for edge in edges {
            adjacency[edge.left].push(edge.right);
            adjacency[edge.right].push(edge.left);
        }
        let mut assignment = vec![0, 0, 0, 1, 1, 1, 1];
        let before =
            bisect_ilp::evaluate_certified_split_objective(&instance, &assignment).unwrap();
        assert!(
            improve_one_to_two_population_swaps(&instance, &adjacency, &mut assignment).unwrap()
                >= 1
        );
        let after = bisect_ilp::evaluate_certified_split_objective(&instance, &assignment).unwrap();
        assert_eq!(
            after.max_population_deviation_scaled,
            before.max_population_deviation_scaled
        );
        assert!(after.weighted_boundary_cut < before.weighted_boundary_cut);
        assert!(bisect_ilp::certified_split_children_connected(&instance, &assignment).unwrap());
    }

    #[test]
    fn two_to_two_swap_reduces_cut_and_preserves_balance() {
        let unit_ids = (0..8).map(|unit| format!("u{unit}")).collect::<Vec<_>>();
        let mut edge_weights = BTreeMap::new();
        for group in [&[0, 1, 2, 3][..], &[4, 5, 6, 7][..]] {
            for (index, &left) in group.iter().enumerate() {
                for &right in &group[index + 1..] {
                    edge_weights.insert((left, right), 1);
                }
            }
        }
        for left in [1, 2] {
            for right in [5, 6] {
                edge_weights.insert((left, right), 10);
            }
        }
        for right in [4, 7] {
            for left in [0, 3] {
                edge_weights.insert((left, right), 10);
            }
        }
        let edges = edge_weights
            .iter()
            .map(|(&(left, right), &weight)| bisect_ilp::ExactEdge {
                left,
                right,
                weight,
            })
            .collect::<Vec<_>>();
        let instance = bisect_ilp::CertifiedSplitInstance {
            schema_version: bisect_ilp::CERTIFIED_SPLIT_INSTANCE_SCHEMA_VERSION.to_string(),
            model_id: bisect_ilp::CERTIFIED_SPLIT_MODEL_ID.to_string(),
            node_path: String::new(),
            parent_certificate_id: None,
            unit_universe_hash: bisect_ilp::certified_split_unit_universe_hash(&unit_ids).unwrap(),
            unit_ids,
            populations: vec![1, 2, 3, 4, 1, 2, 3, 4],
            edges: edges.clone(),
            k_parent: 2,
            k_left: 1,
            k_right: 1,
            orientation_rule: bisect_ilp::SplitOrientationRule::EqualSeatsUnitZeroLeft,
        };
        let mut adjacency = vec![Vec::new(); 8];
        for edge in edges {
            adjacency[edge.left].push(edge.right);
            adjacency[edge.right].push(edge.left);
        }
        let mut assignment = vec![0, 0, 0, 0, 1, 1, 1, 1];
        let before =
            bisect_ilp::evaluate_certified_split_objective(&instance, &assignment).unwrap();
        assert!(
            improve_two_to_two_population_swaps(&instance, &adjacency, &mut assignment).unwrap()
                >= 1
        );
        let after = bisect_ilp::evaluate_certified_split_objective(&instance, &assignment).unwrap();
        assert_eq!(
            after.max_population_deviation_scaled,
            before.max_population_deviation_scaled
        );
        assert!(after.weighted_boundary_cut < before.weighted_boundary_cut);
        assert!(bisect_ilp::certified_split_children_connected(&instance, &assignment).unwrap());
    }

    fn path4_context() -> RplanContext {
        path_context(4)
    }

    fn path_context(unit_count: usize) -> RplanContext {
        let mut units = PlanUnitIndex {
            unit_kind: UnitKind::Imported,
            state: Some("TT".to_string()),
            year: Some(2020),
            canonical_order: CanonicalOrder::ExplicitUnitIds,
            unit_ids: (0..unit_count).map(|idx| format!("u{idx:02}")).collect(),
            unit_universe_hash: String::new(),
            source_id: Some("u17-fixture".to_string()),
        };
        units.unit_universe_hash = units.compute_unit_universe_hash().unwrap();
        RplanContext {
            rctx_version: RCTX_VERSION.to_string(),
            context_hash: String::new(),
            units,
            graph: Some(UnitGraph {
                edge_semantics: EdgeSemantics::Undirected,
                adjacency: (0..unit_count)
                    .map(|unit| {
                        let mut neighbors = Vec::new();
                        if unit > 0 {
                            neighbors.push(edge((unit - 1) as u32));
                        }
                        if unit + 1 < unit_count {
                            neighbors.push(edge((unit + 1) as u32));
                        }
                        neighbors
                    })
                    .collect(),
            }),
            populations: Some(vec![100; unit_count]),
            subdivisions: None,
            demographics: None,
            geometry: None,
            source_hashes: SourceHashes {
                entries: BTreeMap::from([(
                    "fixture".to_string(),
                    format!("sha256:{}", "1".repeat(64)),
                )]),
            },
        }
    }

    fn three_islands_context() -> RplanContext {
        let mut units = PlanUnitIndex {
            unit_kind: UnitKind::Imported,
            state: Some("TT".to_string()),
            year: Some(2020),
            canonical_order: CanonicalOrder::ExplicitUnitIds,
            unit_ids: ["a", "b", "c"].into_iter().map(str::to_string).collect(),
            unit_universe_hash: String::new(),
            source_id: Some("exact-infeasible-fixture".to_string()),
        };
        units.unit_universe_hash = units.compute_unit_universe_hash().unwrap();
        RplanContext {
            rctx_version: RCTX_VERSION.to_string(),
            context_hash: String::new(),
            units,
            graph: Some(UnitGraph {
                edge_semantics: EdgeSemantics::Undirected,
                adjacency: vec![vec![], vec![], vec![]],
            }),
            populations: Some(vec![1, 1, 1]),
            subdivisions: None,
            demographics: None,
            geometry: None,
            source_hashes: SourceHashes {
                entries: BTreeMap::from([(
                    "fixture".to_string(),
                    format!("sha256:{}", "2".repeat(64)),
                )]),
            },
        }
    }

    fn edge(to: u32) -> UnitEdge {
        UnitEdge {
            to,
            kind: EdgeKind::Boundary,
            weight: None,
        }
    }
}
