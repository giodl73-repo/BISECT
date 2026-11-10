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

#[derive(Debug, Clone, PartialEq, Eq)]
struct NrsDfsTreeCutCandidate {
    assignment: Vec<u8>,
    minimum_deviation_candidate_count: usize,
    minimum_deviation_cut_candidate_count: usize,
    minimum_deviation_cut_partition_count: usize,
}

fn nrs_dfs_tree_cut_candidate(
    instance: &bisect_ilp::CertifiedSplitInstance,
    adjacency: &[Vec<usize>],
    assignment: &[u8],
    root: usize,
) -> anyhow::Result<NrsDfsTreeCutCandidate> {
    let n = assignment.len();
    if root >= n {
        return Err(anyhow!("NRS DFS root is outside the unit index"));
    }
    let mut parent = vec![usize::MAX; n];
    let mut order = Vec::with_capacity(n);
    let mut stack = vec![root];
    parent[root] = root;
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
    for unit in 0..n {
        if unit != root {
            children[parent[unit]].push(unit);
        }
    }
    let mut entry = vec![0_usize; n];
    let mut exit = vec![0_usize; n];
    let mut clock = 0_usize;
    let mut traversal = vec![(root, false)];
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
        if unit != root {
            let ancestor = parent[unit];
            subtree_population[ancestor] += subtree_population[unit];
            subtree_initial_left_population[ancestor] += subtree_initial_left_population[unit];
            subtree_minimum[ancestor] = subtree_minimum[ancestor].min(subtree_minimum[unit]);
        }
    }
    let total_population = subtree_population[root];
    let target_numerator = instance.k_left as i128 * i128::from(total_population);
    let total_initial_left_population = subtree_initial_left_population[root];
    let mut best_deviation = u128::MAX;
    let mut candidates = Vec::new();
    for candidate in 0..n {
        if candidate == root {
            continue;
        }
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
                (0..n)
                    .find(|&unit| {
                        !(entry[candidate] <= entry[unit] && entry[unit] < exit[candidate])
                    })
                    .context("NRS DFS complement is empty")?
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
    let minimum_deviation_candidate_count = candidates.len();
    let mut scored_candidates = Vec::with_capacity(candidates.len());
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
        scored_candidates.push((
            cut,
            moved_population,
            minimum,
            candidate,
            complement_is_left,
        ));
    }
    let minimum_cut = scored_candidates
        .iter()
        .map(|(cut, _, _, _, _)| *cut)
        .min()
        .context("NRS DFS tree has no cut edge")?;
    let minimum_deviation_cut_candidate_count = scored_candidates
        .iter()
        .filter(|(cut, _, _, _, _)| *cut == minimum_cut)
        .count();
    let minimum_deviation_cut_partition_count = scored_candidates
        .iter()
        .filter(|(cut, _, _, _, _)| *cut == minimum_cut)
        .map(|(_, _, _, candidate, _)| {
            let unit_zero_in_subtree = entry[*candidate] <= entry[0] && entry[0] < exit[*candidate];
            (0..n)
                .map(|unit| {
                    let in_subtree =
                        entry[*candidate] <= entry[unit] && entry[unit] < exit[*candidate];
                    u8::from(in_subtree != unit_zero_in_subtree)
                })
                .collect::<Vec<_>>()
        })
        .collect::<HashSet<_>>()
        .len();
    let mut best: Option<((u64, i64, usize), usize, bool)> = None;
    for (cut, moved_population, minimum, candidate, complement_is_left) in scored_candidates {
        let key = (cut, moved_population, minimum);
        if best.as_ref().is_none_or(|(current, _, _)| key < *current) {
            best = Some((key, candidate, complement_is_left));
        }
    }
    let (_, candidate, complement_is_left) = best.context("NRS DFS tree has no cut edge")?;
    let assignment = (0..n)
        .map(|unit| {
            let in_subtree = entry[candidate] <= entry[unit] && entry[unit] < exit[candidate];
            if in_subtree != complement_is_left {
                0
            } else {
                1
            }
        })
        .collect();
    Ok(NrsDfsTreeCutCandidate {
        assignment,
        minimum_deviation_candidate_count,
        minimum_deviation_cut_candidate_count,
        minimum_deviation_cut_partition_count,
    })
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

fn nrs_population_deviation_scaled(
    instance: &bisect_ilp::CertifiedSplitInstance,
    assignment: &[u8],
) -> u128 {
    let total_population = instance.populations.iter().sum::<i64>();
    let right_population = instance
        .populations
        .iter()
        .zip(assignment)
        .filter_map(|(&population, &label)| (label == 1).then_some(population))
        .sum::<i64>();
    (instance.k_parent as i128 * i128::from(right_population)
        - instance.k_right as i128 * i128::from(total_population))
    .unsigned_abs()
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct NrsFallbackDiagnostics {
    evaluated_candidate_count: usize,
    minimum_deviation_candidate_count: usize,
    minimum_deviation_cut_candidate_count: usize,
    minimum_deviation_cut_partition_count: usize,
}

#[derive(Default)]
struct NrsFallbackDiagnosticAccumulator {
    diagnostics: NrsFallbackDiagnostics,
    minimum_deviation: Option<u128>,
    minimum_cut: Option<u64>,
    partitions: HashSet<Vec<u8>>,
}

impl NrsFallbackDiagnosticAccumulator {
    fn observe(&mut self, key: &(u128, u64, i64, Vec<u8>), assignment: &[u8]) {
        let deviation = key.0;
        let cut = key.1;
        self.diagnostics.evaluated_candidate_count += 1;
        match self.minimum_deviation {
            None => {
                self.minimum_deviation = Some(deviation);
                self.minimum_cut = Some(cut);
                self.diagnostics.minimum_deviation_candidate_count = 1;
                self.diagnostics.minimum_deviation_cut_candidate_count = 1;
                self.partitions.clear();
                self.partitions
                    .insert(canonical_unlabeled_assignment(assignment));
            }
            Some(current_deviation) if deviation < current_deviation => {
                self.minimum_deviation = Some(deviation);
                self.minimum_cut = Some(cut);
                self.diagnostics.minimum_deviation_candidate_count = 1;
                self.diagnostics.minimum_deviation_cut_candidate_count = 1;
                self.partitions.clear();
                self.partitions
                    .insert(canonical_unlabeled_assignment(assignment));
            }
            Some(current_deviation) if deviation == current_deviation => {
                self.diagnostics.minimum_deviation_candidate_count += 1;
                match self.minimum_cut {
                    Some(current_cut) if cut < current_cut => {
                        self.minimum_cut = Some(cut);
                        self.diagnostics.minimum_deviation_cut_candidate_count = 1;
                        self.partitions.clear();
                        self.partitions
                            .insert(canonical_unlabeled_assignment(assignment));
                    }
                    Some(current_cut) if cut == current_cut => {
                        self.diagnostics.minimum_deviation_cut_candidate_count += 1;
                        self.partitions
                            .insert(canonical_unlabeled_assignment(assignment));
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    fn finish(mut self) -> NrsFallbackDiagnostics {
        self.diagnostics.minimum_deviation_cut_partition_count = self.partitions.len();
        self.diagnostics
    }
}

fn canonical_unlabeled_assignment(assignment: &[u8]) -> Vec<u8> {
    let first = assignment.first().copied().unwrap_or(0);
    assignment.iter().map(|&label| label ^ first).collect()
}

struct NrsFallbackCandidate {
    assignment: Vec<u8>,
    operations: usize,
    moved_units: usize,
    diagnostics: NrsFallbackDiagnostics,
}

fn nrs_v0_2_candidate_key(
    instance: &bisect_ilp::CertifiedSplitInstance,
    raw_assignment: &[u8],
    assignment: &[u8],
) -> (u128, u64, i64, Vec<u8>) {
    let deviation = nrs_population_deviation_scaled(instance, assignment);
    let cut = instance
        .edges
        .iter()
        .filter_map(|edge| (assignment[edge.left] != assignment[edge.right]).then_some(edge.weight))
        .sum::<u64>();
    let moved_population = instance
        .populations
        .iter()
        .zip(assignment.iter().zip(raw_assignment))
        .filter_map(|(&population, (&after, &before))| (after != before).then_some(population))
        .sum::<i64>();
    (deviation, cut, moved_population, assignment.to_vec())
}

fn nrs_v0_2_fallback_candidate(
    instance: &bisect_ilp::CertifiedSplitInstance,
    adjacency: &[Vec<usize>],
    raw_assignment: &[u8],
    v0_1_assignment: &[u8],
    v0_1_operations: usize,
    v0_1_moved_units: usize,
) -> anyhow::Result<NrsFallbackCandidate> {
    let mut best = NrsFallbackCandidate {
        assignment: v0_1_assignment.to_vec(),
        operations: v0_1_operations,
        moved_units: v0_1_moved_units,
        diagnostics: NrsFallbackDiagnostics::default(),
    };
    let mut best_key = nrs_v0_2_candidate_key(instance, raw_assignment, &best.assignment);
    let mut diagnostics = NrsFallbackDiagnosticAccumulator::default();
    diagnostics.observe(&best_key, &best.assignment);
    let mut roots = (0..16)
        .map(|quantile| quantile * (raw_assignment.len() - 1) / 15)
        .collect::<Vec<_>>();
    roots.sort_unstable();
    roots.dedup();
    for root in roots.into_iter().filter(|&root| root != 0) {
        let mut assignment =
            nrs_dfs_tree_cut_candidate(instance, adjacency, raw_assignment, root)?.assignment;
        if instance.k_left == instance.k_right && assignment[0] == 1 {
            for label in &mut assignment {
                *label = 1 - *label;
            }
        }
        let (operations, moved_units) = improve_discovery_population(
            instance,
            adjacency,
            &mut assignment,
            Some(nrs_population_tolerance_scaled_bound(instance)),
        )?;
        let key = nrs_v0_2_candidate_key(instance, raw_assignment, &assignment);
        diagnostics.observe(&key, &assignment);
        if key < best_key {
            best_key = key;
            best = NrsFallbackCandidate {
                assignment,
                operations,
                moved_units,
                diagnostics: NrsFallbackDiagnostics::default(),
            };
        }
    }
    best.diagnostics = diagnostics.finish();
    Ok(best)
}

fn nrs_assignment_side_connected(adjacency: &[Vec<usize>], assignment: &[u8], label: u8) -> bool {
    let Some(first) = assignment.iter().position(|&candidate| candidate == label) else {
        return false;
    };
    let expected = assignment
        .iter()
        .filter(|&&candidate| candidate == label)
        .count();
    let mut seen = vec![false; assignment.len()];
    seen[first] = true;
    let mut queue = VecDeque::from([first]);
    let mut count = 1_usize;
    while let Some(unit) = queue.pop_front() {
        for &neighbor in &adjacency[unit] {
            if assignment[neighbor] == label && !seen[neighbor] {
                seen[neighbor] = true;
                count += 1;
                queue.push_back(neighbor);
            }
        }
    }
    count == expected
}

fn nrs_v0_3_bridge_aware_population_floor(
    instance: &bisect_ilp::CertifiedSplitInstance,
    land_adjacency: &[Vec<usize>],
    components: &[Vec<usize>],
    component_of: &[usize],
) -> anyhow::Result<u128> {
    let total_population = instance.populations.iter().sum::<i64>();
    let mut best = u128::MAX;
    let mut subtree_population = vec![0_i64; instance.unit_ids.len()];
    let mut parent = vec![usize::MAX; instance.unit_ids.len()];
    for (component_index, component) in components.iter().enumerate() {
        if component.len() < 2 {
            continue;
        }
        let component_population = component
            .iter()
            .map(|&unit| instance.populations[unit])
            .sum::<i64>();
        let outside_population = total_population - component_population;
        let mut roots = (0..16)
            .map(|quantile| component[quantile * (component.len() - 1) / 15])
            .collect::<Vec<_>>();
        roots.sort_unstable();
        roots.dedup();
        for root in roots {
            for breadth_first in [false, true] {
                for &unit in component {
                    parent[unit] = usize::MAX;
                }
                let mut order = Vec::with_capacity(component.len());
                parent[root] = root;
                if breadth_first {
                    let mut queue = VecDeque::from([root]);
                    while let Some(unit) = queue.pop_front() {
                        order.push(unit);
                        for &neighbor in &land_adjacency[unit] {
                            if component_of[neighbor] == component_index
                                && parent[neighbor] == usize::MAX
                            {
                                parent[neighbor] = unit;
                                queue.push_back(neighbor);
                            }
                        }
                    }
                } else {
                    let mut stack = vec![root];
                    while let Some(unit) = stack.pop() {
                        order.push(unit);
                        for &neighbor in land_adjacency[unit].iter().rev() {
                            if component_of[neighbor] == component_index
                                && parent[neighbor] == usize::MAX
                            {
                                parent[neighbor] = unit;
                                stack.push(neighbor);
                            }
                        }
                    }
                }
                if order.len() != component.len() {
                    return Err(anyhow!("NRS v0.3 land component traversal is incomplete"));
                }
                for &unit in component {
                    subtree_population[unit] = instance.populations[unit];
                }
                for &unit in order.iter().skip(1).rev() {
                    subtree_population[parent[unit]] += subtree_population[unit];
                }
                for outside_is_left in [false, true] {
                    for &candidate in order.iter().skip(1) {
                        for complement_is_left in [false, true] {
                            let selected_population = if complement_is_left {
                                component_population - subtree_population[candidate]
                            } else {
                                subtree_population[candidate]
                            };
                            let left_population = selected_population
                                + if outside_is_left {
                                    outside_population
                                } else {
                                    0
                                };
                            let deviation = (instance.k_parent as i128
                                * i128::from(left_population)
                                - instance.k_left as i128 * i128::from(total_population))
                            .unsigned_abs();
                            best = best.min(deviation);
                        }
                    }
                }
            }
        }
    }
    Ok(best)
}

fn nrs_v0_3_bridge_aware_component_candidate(
    instance: &bisect_ilp::CertifiedSplitInstance,
    adjacency: &[Vec<usize>],
    land_adjacency: &[Vec<usize>],
    raw_assignment: &[u8],
    v0_2_assignment: &[u8],
    v0_2_operations: usize,
    v0_2_moved_units: usize,
) -> anyhow::Result<NrsFallbackCandidate> {
    let mut best = NrsFallbackCandidate {
        assignment: v0_2_assignment.to_vec(),
        operations: v0_2_operations,
        moved_units: v0_2_moved_units,
        diagnostics: NrsFallbackDiagnostics::default(),
    };
    let mut best_key = nrs_v0_2_candidate_key(instance, raw_assignment, &best.assignment);
    let mut diagnostics = NrsFallbackDiagnosticAccumulator::default();
    diagnostics.observe(&best_key, &best.assignment);
    let mut raw_best: Option<((u128, u64, i64, Vec<u8>), Vec<u8>)> = None;
    let n = instance.unit_ids.len();
    let total_population = instance.populations.iter().sum::<i64>();
    let mut component_of = vec![usize::MAX; n];
    let mut components = Vec::new();
    for start in 0..n {
        if component_of[start] != usize::MAX {
            continue;
        }
        let component_index = components.len();
        component_of[start] = component_index;
        let mut component = Vec::new();
        let mut stack = vec![start];
        while let Some(unit) = stack.pop() {
            component.push(unit);
            for &neighbor in land_adjacency[unit].iter().rev() {
                if component_of[neighbor] == usize::MAX {
                    component_of[neighbor] = component_index;
                    stack.push(neighbor);
                }
            }
        }
        component.sort_unstable();
        components.push(component);
    }
    let population_floor = nrs_v0_3_bridge_aware_population_floor(
        instance,
        land_adjacency,
        &components,
        &component_of,
    )?;
    if population_floor > best_key.0 {
        best.diagnostics = diagnostics.finish();
        return Ok(best);
    }
    let mut subtree_population = vec![0_i64; n];
    let mut parent = vec![usize::MAX; n];
    'component_search: for (component_index, component) in components.iter().enumerate() {
        if component.len() < 2 {
            continue;
        }
        let component_population = component
            .iter()
            .map(|&unit| instance.populations[unit])
            .sum::<i64>();
        let outside_population = total_population - component_population;
        let mut roots = (0..16)
            .map(|quantile| component[quantile * (component.len() - 1) / 15])
            .collect::<Vec<_>>();
        roots.sort_unstable();
        roots.dedup();
        for root in roots {
            for breadth_first in [false, true] {
                for &unit in component {
                    parent[unit] = usize::MAX;
                }
                let mut order = Vec::with_capacity(component.len());
                parent[root] = root;
                if breadth_first {
                    let mut queue = VecDeque::from([root]);
                    while let Some(unit) = queue.pop_front() {
                        order.push(unit);
                        for &neighbor in &land_adjacency[unit] {
                            if component_of[neighbor] == component_index
                                && parent[neighbor] == usize::MAX
                            {
                                parent[neighbor] = unit;
                                queue.push_back(neighbor);
                            }
                        }
                    }
                } else {
                    let mut stack = vec![root];
                    while let Some(unit) = stack.pop() {
                        order.push(unit);
                        for &neighbor in land_adjacency[unit].iter().rev() {
                            if component_of[neighbor] == component_index
                                && parent[neighbor] == usize::MAX
                            {
                                parent[neighbor] = unit;
                                stack.push(neighbor);
                            }
                        }
                    }
                }
                if order.len() != component.len() {
                    return Err(anyhow!("NRS v0.3 land component traversal is incomplete"));
                }
                for &unit in component {
                    subtree_population[unit] = instance.populations[unit];
                }
                for &unit in order.iter().skip(1).rev() {
                    subtree_population[parent[unit]] += subtree_population[unit];
                }
                for outside_is_left in [false, true] {
                    let mut tree_best = u128::MAX;
                    let mut tree_candidates = Vec::new();
                    for &candidate in order.iter().skip(1) {
                        for complement_is_left in [false, true] {
                            let selected_population = if complement_is_left {
                                component_population - subtree_population[candidate]
                            } else {
                                subtree_population[candidate]
                            };
                            let left_population = selected_population
                                + if outside_is_left {
                                    outside_population
                                } else {
                                    0
                                };
                            let deviation = (instance.k_parent as i128
                                * i128::from(left_population)
                                - instance.k_left as i128 * i128::from(total_population))
                            .unsigned_abs();
                            if deviation < tree_best {
                                tree_best = deviation;
                                tree_candidates.clear();
                            }
                            if deviation == tree_best {
                                tree_candidates.push((candidate, complement_is_left));
                            }
                        }
                    }
                    if tree_best != population_floor {
                        continue;
                    }
                    for (candidate, complement_is_left) in tree_candidates {
                        let mut in_subtree = vec![false; n];
                        let mut stack = vec![candidate];
                        in_subtree[candidate] = true;
                        while let Some(unit) = stack.pop() {
                            for &neighbor in &land_adjacency[unit] {
                                if parent[neighbor] == unit && !in_subtree[neighbor] {
                                    in_subtree[neighbor] = true;
                                    stack.push(neighbor);
                                }
                            }
                        }
                        let mut assignment = (0..n)
                            .map(|unit| {
                                if component_of[unit] != component_index {
                                    if outside_is_left {
                                        0
                                    } else {
                                        1
                                    }
                                } else {
                                    let selected = in_subtree[unit] != complement_is_left;
                                    if selected {
                                        0
                                    } else {
                                        1
                                    }
                                }
                            })
                            .collect::<Vec<u8>>();
                        if instance.k_left == instance.k_right && assignment[0] == 1 {
                            for label in &mut assignment {
                                *label = 1 - *label;
                            }
                        }
                        if !nrs_assignment_side_connected(adjacency, &assignment, 0)
                            || !nrs_assignment_side_connected(adjacency, &assignment, 1)
                        {
                            continue;
                        }
                        let key = nrs_v0_2_candidate_key(instance, raw_assignment, &assignment);
                        raw_best = Some((key, assignment));
                        break 'component_search;
                    }
                }
            }
        }
    }
    if let Some((raw_key, mut assignment)) = raw_best {
        diagnostics.observe(&raw_key, &assignment);
        if raw_key < best_key {
            best_key = raw_key;
            best = NrsFallbackCandidate {
                assignment: assignment.clone(),
                operations: 0,
                moved_units: 0,
                diagnostics: NrsFallbackDiagnostics::default(),
            };
        }
        let (operations, moved_units) = improve_discovery_population(
            instance,
            adjacency,
            &mut assignment,
            Some(nrs_population_tolerance_scaled_bound(instance)),
        )?;
        let key = nrs_v0_2_candidate_key(instance, raw_assignment, &assignment);
        diagnostics.observe(&key, &assignment);
        if key < best_key {
            best = NrsFallbackCandidate {
                assignment,
                operations,
                moved_units,
                diagnostics: NrsFallbackDiagnostics::default(),
            };
        }
    }
    best.diagnostics = diagnostics.finish();
    Ok(best)
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

fn discovery_articulation_points(graph: &DiscoveryGraph<'_>, assignment: &[u8], heavy: u8) -> anyhow::Result<Vec<usize>> {
    rgraph_core::articulation_points_with_filter(graph, |(from,to)| assignment[from]==heavy && assignment[to]==heavy)
        .map_err(|error| anyhow!("discovery articulation analysis failed: {error}"))
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

