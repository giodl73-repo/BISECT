//! Rebalance a fine-level plan after a coarse move.
//! Per spec §2.5: boundary-swap loop, max 200 iterations, failure = reject coarse move.

/// Attempt to rebalance `assignment` so all k districts satisfy `pop_tolerance`.
///
/// Returns `true` if rebalancing succeeded (all districts within tolerance).
/// Returns `false` if it failed after `max_iters` boundary swaps (coarse move should be rejected).
pub fn rebalance(
    assignment: &mut Vec<u32>,
    adj: &[Vec<usize>],
    pop: &[i64],
    k: u32,
    pop_tolerance: f64,
    max_iters: usize,
) -> bool {
    let total_pop: i64 = pop.iter().sum();
    let ideal = total_pop as f64 / k as f64;
    let tol_abs = (pop_tolerance * total_pop as f64) as i64 + 1;

    for _ in 0..max_iters {
        // Compute per-district population
        let mut dist_pop = vec![0i64; k as usize + 1];
        for (t, &d) in assignment.iter().enumerate() {
            dist_pop[d as usize] += pop[t];
        }

        // Check if all districts are within tolerance
        let all_balanced = (1..=k as usize).all(|d| {
            (dist_pop[d] - ideal as i64).abs() <= tol_abs
        });
        if all_balanced { return true; }

        // Find the most overweight district
        let (heavy, _) = (1..=k as usize)
            .map(|d| (d, dist_pop[d] - ideal as i64))
            .filter(|&(_, excess)| excess > 0)
            .max_by_key(|&(_, excess)| excess)
            .unwrap_or((0, 0));
        if heavy == 0 { return true; }

        // Find a boundary tract in `heavy` adjacent to an underweight district
        let moved = assignment.iter().enumerate()
            .filter(|&(_, &d)| d == heavy as u32)
            .find_map(|(t, _)| {
                adj[t].iter().find_map(|&nb| {
                    let nd = assignment[nb] as usize;
                    if nd != heavy && dist_pop[nd] < ideal as i64 {
                        // Check contiguity: heavy district remains connected after removing t
                        if is_connected_without(assignment, adj, t, heavy as u32) {
                            Some((t, nd as u32))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
            });

        match moved {
            Some((tract, target_district)) => {
                assignment[tract] = target_district;
            }
            None => return false, // no valid boundary swap found
        }
    }
    false // exceeded max_iters
}

/// Check if the subgraph induced by `district` remains connected after removing `exclude_tract`.
fn is_connected_without(
    assignment: &[u32],
    adj: &[Vec<usize>],
    exclude_tract: usize,
    district: u32,
) -> bool {
    let tracts: Vec<usize> = assignment.iter().enumerate()
        .filter(|&(t, &d)| d == district && t != exclude_tract)
        .map(|(t, _)| t)
        .collect();
    if tracts.is_empty() { return false; }

    let set: std::collections::HashSet<usize> = tracts.iter().copied().collect();
    let mut visited = std::collections::HashSet::new();
    let mut queue = std::collections::VecDeque::new();
    queue.push_back(tracts[0]);
    visited.insert(tracts[0]);
    while let Some(v) = queue.pop_front() {
        for &nb in &adj[v] {
            if set.contains(&nb) && !visited.contains(&nb) {
                visited.insert(nb);
                queue.push_back(nb);
            }
        }
    }
    visited.len() == tracts.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn path_adj(n: usize) -> Vec<Vec<usize>> {
        (0..n).map(|i| {
            let mut nb = Vec::new();
            if i > 0 { nb.push(i-1); }
            if i < n-1 { nb.push(i+1); }
            nb
        }).collect()
    }

    #[test]
    fn rebalance_already_balanced_returns_true() {
        let adj = path_adj(4);
        let pop = vec![100i64; 4];
        let mut asgn = vec![1u32, 1, 2, 2];
        assert!(rebalance(&mut asgn, &adj, &pop, 2, 0.1, 200));
        assert_eq!(asgn, vec![1, 1, 2, 2]); // unchanged
    }

    #[test]
    fn rebalance_fixes_imbalance() {
        let adj = path_adj(4);
        let pop = vec![100i64; 4];
        // Imbalanced: 3 tracts in district 1, 1 in district 2
        let mut asgn = vec![1u32, 1, 1, 2];
        let result = rebalance(&mut asgn, &adj, &pop, 2, 0.1, 200);
        assert!(result);
        // After rebalancing: 2+2 split (within 10% tolerance of 200 each)
        let pop1: i64 = asgn.iter().enumerate().filter(|&(_, &d)| d == 1).map(|(t, _)| pop[t]).sum();
        let pop2: i64 = asgn.iter().enumerate().filter(|&(_, &d)| d == 2).map(|(t, _)| pop[t]).sum();
        assert!((pop1 - 200).abs() <= 40, "district 1 pop {pop1} not within 10% of 200");
        assert!((pop2 - 200).abs() <= 40, "district 2 pop {pop2} not within 10% of 200");
    }

    #[test]
    fn rebalance_returns_false_on_impossible_case() {
        // All tracts in district 1 with a graph that doesn't allow boundary swaps
        let adj = vec![vec![]; 2]; // 2 isolated nodes, both in district 1 -- no boundary to swap
        let pop = vec![100i64; 2];
        let mut asgn = vec![1u32, 1];
        // This should fail because there's no boundary tract adjacent to district 2
        let result = rebalance(&mut asgn, &adj, &pop, 2, 0.0, 200);
        assert!(!result);
    }
}
