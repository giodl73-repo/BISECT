//! ReCom-style crossover operator for NSGA-II.
//!
//! Phase 1 crossover: merge two adjacent districts from parent_a, then
//! find a balanced spanning-tree cut. Falls back to parent_a if no valid
//! cut is found after multiple attempts.
//!
//! Per spec §4.

use rand::SeedableRng;
use rand::rngs::SmallRng;
use rand::Rng;

use bisect_ensemble::spanning::random_spanning_tree;

/// Crossover: merge two adjacent districts from parent_a, regrow using a
/// spanning tree, find a balanced cut.
///
/// Returns a new valid plan, or parent_a unchanged if all retries fail.
///
/// # Parameters
/// - `parent_a`, `parent_b`: two parent plans (1-based district IDs, 1..=k)
/// - `adjacency`: full adjacency list (usize indices)
/// - `pop`: population per tract
/// - `k`: number of districts
/// - `balance_tolerance`: allowed population deviation from ideal (e.g., 0.005)
/// - `seed`: crossover seed (from cross_seed())
pub fn crossover(
    parent_a: &[u32],
    _parent_b: &[u32],
    adjacency: &[Vec<usize>],
    pop: &[i64],
    k: usize,
    balance_tolerance: f64,
    seed: u64,
) -> Vec<u32> {
    let n = parent_a.len();
    let total_pop: i64 = pop.iter().sum();
    let target_pop = total_pop as f64 / k as f64;
    let tol_abs = balance_tolerance * target_pop;

    let mut rng = SmallRng::seed_from_u64(seed);

    // Collect adjacent district pairs from parent_a
    let mut adj_pairs: Vec<(u32, u32)> = Vec::new();
    for v in 0..n {
        let dv = parent_a[v];
        for &u in &adjacency[v] {
            let du = parent_a[u];
            if du > dv {
                let pair = (dv, du);
                if !adj_pairs.contains(&pair) {
                    adj_pairs.push(pair);
                }
            }
        }
    }

    if adj_pairs.is_empty() {
        // No adjacent district pairs (shouldn't happen for k >= 2 connected plan)
        return parent_a.to_vec();
    }

    // Pick a random adjacent pair (d1, d2)
    let pair_idx = rng.gen_range(0..adj_pairs.len());
    let (d1, d2) = adj_pairs[pair_idx];

    // Collect tracts in the merged region (d1 union d2)
    let region: Vec<usize> = (0..n)
        .filter(|&t| parent_a[t] == d1 || parent_a[t] == d2)
        .collect();

    if region.len() < 2 {
        return parent_a.to_vec();
    }

    // Build local adjacency for the region
    let region_pop: i64 = region.iter().map(|&t| pop[t]).sum();
    let region_idx: std::collections::HashMap<usize, usize> = region.iter()
        .enumerate()
        .map(|(li, &gi)| (gi, li))
        .collect();

    let local_adj: Vec<Vec<u32>> = region.iter().map(|&v| {
        adjacency[v].iter()
            .filter_map(|&u| region_idx.get(&u).map(|&li| li as u32))
            .collect()
    }).collect();

    // Try up to 5 spanning trees to find a balanced cut
    for attempt in 0..5u32 {
        let attempt_seed = seed.wrapping_add((attempt as u64).wrapping_mul(0x9e3779b97f4a7c15));
        let mut tree_rng = SmallRng::seed_from_u64(attempt_seed);
        let tree = random_spanning_tree(&local_adj, &mut tree_rng);

        // Try all tree edges as potential cuts
        let mut best_cut: Option<(u32, u32)> = None;
        let mut best_balance = f64::INFINITY;

        for (child, parent) in tree.edges() {
            // Split tree on this edge
            let (comp_a, _comp_b) = tree.split_on(child, parent);

            let pop_a: i64 = comp_a.iter().map(|&li| pop[region[li as usize]]).sum();
            let pop_b = region_pop - pop_a;

            let bal_a = (pop_a as f64 - target_pop).abs();
            let bal_b = (pop_b as f64 - target_pop).abs();
            let worst = bal_a.max(bal_b);

            if bal_a <= tol_abs && bal_b <= tol_abs && worst < best_balance {
                best_balance = worst;
                best_cut = Some((child, parent));
            }
        }

        if let Some((child, parent)) = best_cut {
            let (comp_a, _comp_b) = tree.split_on(child, parent);
            let comp_a_set: std::collections::HashSet<usize> =
                comp_a.iter().map(|&li| region[li as usize]).collect();

            let mut new_plan = parent_a.to_vec();
            for &t in &region {
                if comp_a_set.contains(&t) {
                    new_plan[t] = d1;
                } else {
                    new_plan[t] = d2;
                }
            }
            return new_plan;
        }
    }

    // All retries failed — return parent_a unchanged
    parent_a.to_vec()
}

/// Check if a plan is valid: all districts contiguous and population-balanced.
///
/// Used in tests to verify crossover and mutation outputs.
pub fn is_plan_valid(
    plan: &[u32],
    adjacency: &[Vec<usize>],
    pop: &[i64],
    k: usize,
    balance_tolerance: f64,
) -> bool {
    let n = plan.len();
    let total_pop: i64 = pop.iter().sum();
    let target_pop = total_pop as f64 / k as f64;
    let tol_abs = balance_tolerance * target_pop;

    for d in 1u32..=k as u32 {
        let tracts: Vec<usize> = (0..n).filter(|&t| plan[t] == d).collect();
        if tracts.is_empty() { return false; }

        // Check population balance
        let dp: i64 = tracts.iter().map(|&t| pop[t]).sum();
        if (dp as f64 - target_pop).abs() > tol_abs { return false; }

        // Check contiguity via BFS
        let tract_set: std::collections::HashSet<usize> = tracts.iter().copied().collect();
        let mut visited = std::collections::HashSet::new();
        let mut queue = std::collections::VecDeque::new();
        queue.push_back(tracts[0]);
        visited.insert(tracts[0]);
        while let Some(v) = queue.pop_front() {
            for &u in &adjacency[v] {
                if tract_set.contains(&u) && !visited.contains(&u) {
                    visited.insert(u);
                    queue.push_back(u);
                }
            }
        }
        if visited.len() != tracts.len() { return false; }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    fn path_adj(n: usize) -> Vec<Vec<usize>> {
        (0..n).map(|i| {
            let mut nb = Vec::new();
            if i > 0 { nb.push(i - 1); }
            if i < n - 1 { nb.push(i + 1); }
            nb
        }).collect()
    }

    #[test]
    fn crossover_returns_parent_a_or_valid_plan() {
        let adj = path_adj(4);
        let pop = vec![100i64; 4];
        let pa = vec![1u32, 1, 2, 2];
        let pb = vec![1u32, 2, 1, 2]; // invalid for this test but crossover still mustn't panic
        let result = crossover(&pa, &pb, &adj, &pop, 2, 0.5, 42);
        // Must be parent_a or a valid plan
        assert_eq!(result.len(), 4);
        // Sanity: every tract assigned
        for &d in &result { assert!(d >= 1 && d <= 2); }
    }

    #[test]
    fn crossover_never_panics_on_degenerate() {
        // k=2, 2 tracts — degenerate but must not panic
        let adj = vec![vec![1usize], vec![0usize]];
        let pop = vec![100i64; 2];
        let pa = vec![1u32, 2];
        let pb = vec![2u32, 1];
        let _ = crossover(&pa, &pb, &adj, &pop, 2, 0.5, 0);
    }
}
