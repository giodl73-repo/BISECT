//! Single boundary-tract flip mutation for NSGA-II.
//!
//! Picks a random boundary tract, flips it to an adjacent district if valid
//! (population balance + contiguity). Returns original plan if no valid flip.
//!
//! Per spec §5.

use rand::rngs::SmallRng;
use rand::seq::SliceRandom;
use rand::SeedableRng;

/// Single boundary-tract flip mutation.
///
/// 1. Collect all boundary tracts (tracts adjacent to a different district)
/// 2. Pick a random boundary tract `t`
/// 3. Collect candidate target districts (neighbours of `t` in a different district)
/// 4. Pick a random target district `d_new`
/// 5. Tentatively assign `plan[t] = d_new`
/// 6. Check validity (population balance + contiguity of the old district)
/// 7. Return updated plan if valid, else return original
///
/// # Parameters
/// - `plan`: district assignment (1-based district IDs, 1..=k)
/// - `adjacency`: full adjacency list (usize indices)
/// - `pop`: population per tract
/// - `k`: number of districts
/// - `balance_tolerance`: allowed population deviation from ideal (e.g., 0.005)
/// - `seed`: mutation seed (from mut_seed())
pub fn mutate(
    plan: &[u32],
    adjacency: &[Vec<usize>],
    pop: &[i64],
    k: usize,
    balance_tolerance: f64,
    seed: u64,
) -> Vec<u32> {
    let n = plan.len();
    let total_pop: i64 = pop.iter().sum();
    let target_pop = total_pop as f64 / k as f64;
    let tol_abs = balance_tolerance * target_pop;

    let mut rng = SmallRng::seed_from_u64(seed);

    // Collect all boundary tracts
    let boundary: Vec<usize> = (0..n)
        .filter(|&t| adjacency[t].iter().any(|&u| plan[u] != plan[t]))
        .collect();

    if boundary.is_empty() {
        return plan.to_vec();
    }

    // Pick a random boundary tract
    let t = *boundary.choose(&mut rng).unwrap();
    let d_old = plan[t];

    // Collect candidate target districts (neighbours of t with different district)
    let mut candidates: Vec<u32> = adjacency[t]
        .iter()
        .map(|&u| plan[u])
        .filter(|&d| d != d_old)
        .collect();
    candidates.sort_unstable();
    candidates.dedup();

    if candidates.is_empty() {
        return plan.to_vec();
    }

    // Pick a random target district
    let d_new = *candidates.choose(&mut rng).unwrap();

    // Compute new population of the affected districts
    let pop_d_old: i64 = (0..n).filter(|&i| plan[i] == d_old).map(|i| pop[i]).sum();
    let pop_d_new: i64 = (0..n).filter(|&i| plan[i] == d_new).map(|i| pop[i]).sum();
    let tract_pop = pop[t];

    let new_pop_d_old = pop_d_old - tract_pop;
    let new_pop_d_new = pop_d_new + tract_pop;

    // Population balance check
    if (new_pop_d_old as f64 - target_pop).abs() > tol_abs {
        return plan.to_vec();
    }
    if (new_pop_d_new as f64 - target_pop).abs() > tol_abs {
        return plan.to_vec();
    }

    // Contiguity check: d_old (minus tract t) must remain connected
    let d_old_tracts: Vec<usize> = (0..n).filter(|&i| plan[i] == d_old && i != t).collect();

    if d_old_tracts.is_empty() {
        // Would create an empty district — invalid
        return plan.to_vec();
    }

    if !is_connected_subset(&d_old_tracts, adjacency) {
        return plan.to_vec();
    }

    // Valid flip — apply
    let mut new_plan = plan.to_vec();
    new_plan[t] = d_new;
    new_plan
}

/// Check if a set of tracts forms a connected subgraph via BFS.
fn is_connected_subset(tracts: &[usize], adjacency: &[Vec<usize>]) -> bool {
    if tracts.len() <= 1 {
        return true;
    }
    rgraph_core::node_subset_connected(adjacency, tracts)
        .expect("validated Pareto adjacency and subset")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn path_adj(n: usize) -> Vec<Vec<usize>> {
        (0..n)
            .map(|i| {
                let mut nb = Vec::new();
                if i > 0 {
                    nb.push(i - 1);
                }
                if i < n - 1 {
                    nb.push(i + 1);
                }
                nb
            })
            .collect()
    }

    fn is_plan_valid_basic(
        plan: &[u32],
        adj: &[Vec<usize>],
        pop: &[i64],
        k: usize,
        tol: f64,
    ) -> bool {
        let n = plan.len();
        let total: i64 = pop.iter().sum();
        let target = total as f64 / k as f64;
        let tol_abs = tol * target;
        for d in 1u32..=k as u32 {
            let tracts: Vec<usize> = (0..n).filter(|&t| plan[t] == d).collect();
            if tracts.is_empty() {
                return false;
            }
            let dp: i64 = tracts.iter().map(|&t| pop[t]).sum();
            if (dp as f64 - target).abs() > tol_abs {
                return false;
            }
            let set: std::collections::HashSet<usize> = tracts.iter().copied().collect();
            let mut vis = std::collections::HashSet::new();
            let mut q = std::collections::VecDeque::new();
            q.push_back(tracts[0]);
            vis.insert(tracts[0]);
            while let Some(v) = q.pop_front() {
                for &u in &adj[v] {
                    if set.contains(&u) && !vis.contains(&u) {
                        vis.insert(u);
                        q.push_back(u);
                    }
                }
            }
            if vis.len() != tracts.len() {
                return false;
            }
        }
        true
    }

    #[test]
    fn mutate_returns_valid_or_unchanged() {
        let adj = path_adj(6);
        let pop = vec![100i64; 6];
        let plan = vec![1u32, 1, 1, 2, 2, 2];
        let result = mutate(&plan, &adj, &pop, 2, 0.5, 42);
        assert_eq!(result.len(), 6);
        // Result must either be original or a valid plan
        let is_unchanged = result == plan;
        let is_valid = is_plan_valid_basic(&result, &adj, &pop, 2, 0.5);
        assert!(
            is_unchanged || is_valid,
            "mutation must return original or valid plan"
        );
    }

    #[test]
    fn mutate_never_panics_degenerate() {
        // k=2, 2 tracts — boundary flip can't be valid, so returns unchanged
        let adj = vec![vec![1usize], vec![0usize]];
        let pop = vec![100i64; 2];
        let plan = vec![1u32, 2];
        let result = mutate(&plan, &adj, &pop, 2, 0.5, 0);
        assert_eq!(
            result, plan,
            "degenerate: should return unchanged (no valid flip)"
        );
    }

    #[test]
    fn mutate_tight_tolerance_returns_unchanged() {
        // Tight balance_tolerance = 0.0 means any flip is likely invalid
        let adj = path_adj(6);
        let pop = vec![100i64, 200, 100, 100, 200, 100]; // unequal pops
        let plan = vec![1u32, 1, 1, 2, 2, 2];
        let result = mutate(&plan, &adj, &pop, 2, 0.0, 99);
        // With 0 tolerance, any flip changes balance — should return unchanged
        assert_eq!(result, plan);
    }

    #[test]
    fn is_connected_subset_path() {
        let adj = path_adj(6);
        assert!(is_connected_subset(&[0, 1, 2], &adj));
        assert!(!is_connected_subset(&[0, 2], &adj)); // gap at 1
    }
}
